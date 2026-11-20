use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;
use wry::{PageLoadEvent, WebContext, WebViewBuilder};

#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

use crate::adblock;
use crate::config::{resolved_downloads_dir, unique_download_path_in};
use crate::hud;
use crate::pages;
use crate::tabs::TabManager;
use crate::util::LockExt;

use super::events::UserEvent;
use super::ipc::handle_ipc_message;
use super::state::AppState;
use super::url::{normalize_logical_url, to_webview_url};

pub struct TabWebView {
    pub webview: wry::WebView,
    pub bound_tab_id: Arc<AtomicUsize>,
}

pub struct TabLaunch<'a> {
    pub win_id: usize,
    pub tab_id: usize,
    pub url: &'a str,
    pub visible: bool,
}

#[cfg(target_os = "windows")]
pub fn windows_browser_args(hw_accel: bool) -> String {
    let gpu_args = if hw_accel {
        "--enable-gpu-rasterization --enable-zero-copy"
    } else {
        "--disable-gpu-rasterization --disable-gpu-compositing"
    };
    format!(
        "{} \
        --enable-quic \
        --enable-smooth-scrolling \
        --disable-background-networking \
        --disable-component-update \
        --disable-domain-reliability \
        --disable-breakpad \
        --no-default-browser-check \
        --no-first-run \
        --disable-features=msWebOOUI,msPdfOOUI,Prerender2,TopicsAPI,Translate,MediaRouter,OptimizationHints \
        --enable-features=OverlayScrollbar,OverlayScrollbarWinStyle,OverlayScrollbarWinStyleAnimation,FluentOverlayScrollbar",
        gpu_args
    )
}

pub fn build_tab_webview(
    window: &tao::window::Window,
    web_context: &mut WebContext,
    launch: TabLaunch<'_>,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    event_proxy: &EventLoopProxy<UserEvent>,
) -> Result<TabWebView, Box<dyn Error>> {
    let win_id = launch.win_id;
    let target_url = launch.url;
    let bound_tab_id = Arc::new(AtomicUsize::new(launch.tab_id));
    let (initial_tabs, tm_incog) = {
        let tm = tab_manager.lock_safe();
        (tm.get_state(), tm.is_incognito())
    };
    #[cfg(target_os = "windows")]
    let adblock_cfg;
    #[cfg(target_os = "windows")]
    let boot_hw_accel;
    #[cfg(target_os = "windows")]
    let boot_chrome_ext;
    let (incognito, resolved_ua, default_zoom, allow_devtools, initial_script) = {
        let mut st = app_state.lock_safe();
        let _ = st.reload_if_changed();
        #[cfg(target_os = "windows")]
        {
            adblock_cfg = st.adblock.clone();
            boot_hw_accel = st.boot_hw_accel;
            boot_chrome_ext = st.boot_chrome_ext_enabled;
        }
        let script = hud::generate_hud_script_from_state(&st, Some(&initial_tabs));
        (
            st.config.incognito || tm_incog,
            st.config.resolved_user_agent(),
            st.config.default_zoom,
            cfg!(debug_assertions) || st.config.allow_page_eval,
            script,
        )
    };

    let mut builder = WebViewBuilder::new_with_web_context(web_context)
        .with_background_color((9, 9, 11, 255))
        .with_visible(launch.visible)
        .with_incognito(incognito)
        .with_back_forward_navigation_gestures(true);

    if let Some(ua) = resolved_ua {
        builder = builder.with_user_agent(&ua);
    }

    #[cfg(target_os = "windows")]
    {
        let args = windows_browser_args(boot_hw_accel);
        builder = builder
            .with_additional_browser_args(&args)
            .with_browser_extensions_enabled(boot_chrome_ext);
    }

    // Intercept New Window Requests (target="_blank", window.open) -> Block Ad Popups or Convert to Internal Tabs
    let proxy_for_new_window = event_proxy.clone();
    let tabs_for_new_window = Arc::clone(tab_manager);
    builder = builder.with_new_window_req_handler(move |url, _features| {
        if adblock::should_block_url(&url) {
            adblock::record_block();
            return wry::NewWindowResponse::Deny;
        }
        #[cfg(target_os = "windows")]
        if adblock::launch_external_scheme_windows(&url) {
            return wry::NewWindowResponse::Deny;
        }
        // Allow external http(s) URLs and validated chrome-extension:// pages; deny fynexo://, file://, javascript:, data:, etc.
        if !super::security::is_safe_external_http_url(&url)
            && !super::security::is_safe_chrome_extension_url(&url)
        {
            return wry::NewWindowResponse::Deny;
        }
        let logical = normalize_logical_url(&url);
        let (new_id, nav_url) = tabs_for_new_window.lock_safe().new_tab(&logical);
        let _ = proxy_for_new_window.send_event(UserEvent::SwitchTab {
            win_id,
            id: new_id,
            url: nav_url,
        });
        let _ = proxy_for_new_window.send_event(UserEvent::SyncTabs(win_id));
        wry::NewWindowResponse::Deny
    });

    // Main-Frame Navigation Guard: Block Ad Redirects, Data-URI Phishing & Route External Schemes Safely
    #[allow(unused_variables)]
    let proxy_for_nav = event_proxy.clone();
    builder = builder.with_navigation_handler(move |url| {
        let lower = url.to_ascii_lowercase();
        let is_standard = lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("fynexo://")
            || super::security::is_safe_chrome_extension_url(&url)
            || lower.starts_with("about:")
            || lower.starts_with("blob:")
            || lower.starts_with("javascript:");

        if !is_standard {
            #[cfg(target_os = "windows")]
            if adblock::launch_external_scheme_windows(&url) {
                let _ =
                    proxy_for_nav.send_event(UserEvent::Toast("Opened in system app".to_string()));
            }
            return false;
        }

        if adblock::should_block_url(&url) {
            adblock::record_block();
            return false;
        }

        true
    });

    // Native File Download Handlers (Desktop WebView2 / WebKit)
    let proxy_for_dl_start = event_proxy.clone();
    let state_for_dl = Arc::clone(app_state);
    builder = builder.with_download_started_handler(move |_url, dest_path| {
        let suggested = dest_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("download.bin");
        let custom_dir = state_for_dl.lock_safe().config.downloads_dir.clone();
        let base_dir = resolved_downloads_dir(&custom_dir);
        let final_path = unique_download_path_in(&base_dir, suggested);
        let display_name = final_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        *dest_path = final_path;
        let _ = proxy_for_dl_start.send_event(UserEvent::DownloadUpdate {
            name: display_name,
            status: "downloading".to_string(),
        });
        true
    });

    let proxy_for_dl_done = event_proxy.clone();
    builder = builder.with_download_completed_handler(move |_url, path_opt, success| {
        let name = path_opt
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        let _ = proxy_for_dl_done.send_event(UserEvent::DownloadUpdate {
            name,
            status: if success {
                "completed".to_string()
            } else {
                "failed".to_string()
            },
        });
    });

    // Restore per-tab reading scroll depth & reveal pending tabs after page load finishes
    let tab_id_for_load = Arc::clone(&bound_tab_id);
    let proxy_for_load = event_proxy.clone();
    builder = builder.with_on_page_load_handler(move |event, page_url| {
        if matches!(event, PageLoadEvent::Finished) {
            let norm = normalize_logical_url(&page_url);
            if matches!(norm.as_str(), "about:blank" | "") {
                return;
            }
            let tid = tab_id_for_load.load(Ordering::Acquire);
            if tid != 0 {
                let _ = proxy_for_load.send_event(UserEvent::PageFinished {
                    win_id,
                    tab_id: tid,
                    url: norm,
                });
            }
        }
    });

    // Custom URI Scheme: fynexo://newtab & fynexo://settings
    let state_for_proto = Arc::clone(app_state);
    builder = builder.with_custom_protocol("fynexo".to_string(), move |_id, req| {
        let route = super::security::resolve_internal_route(
            req.uri().host().unwrap_or(""),
            req.uri().path(),
        );
        let is_settings = route == Some("settings");
        let is_newtab = route == Some("newtab");

        if is_settings || is_newtab {
            let html = {
                let mut st = state_for_proto.lock_safe();
                let _ = st.reload_if_changed();
                if is_settings {
                    let payload = st.payload();
                    drop(st);
                    pages::settings_html(&payload)
                } else {
                    let boot_json = st.newtab_boot_json();
                    drop(st);
                    pages::newtab_html_from_json(&boot_json)
                }
            };
            wry::http::Response::builder()
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Cache-Control", "no-store")
                .body(std::borrow::Cow::from(html.into_bytes()))
                .unwrap_or_else(|_| wry::http::Response::new(std::borrow::Cow::Borrowed(&b""[..])))
        } else {
            let not_found = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>404 — Page Not Found · Fynexo</title>
<style>
  *,*::before,*::after{box-sizing:border-box}
  body{margin:0;min-height:100vh;background:#09090b;color:#f4f4f6;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;display:flex;align-items:center;justify-content:center;padding:24px}
  .card{width:100%;max-width:420px;background:#18181b;border:1px solid #27272a;border-top-color:#3f3f46;border-radius:12px;padding:24px;display:flex;flex-direction:column;gap:14px}
  .badge{align-self:flex-start;font-family:"JetBrains Mono",ui-monospace,Consolas,monospace;font-size:11px;font-weight:600;color:#818cf8;background:rgba(99,102,241,0.14);border:1px solid rgba(99,102,241,0.3);border-radius:6px;padding:2px 8px;letter-spacing:.03em}
  h1{margin:0;font-size:18px;font-weight:600;letter-spacing:-.02em;color:#f4f4f6}
  p{margin:0;font-size:13px;line-height:1.55;color:#a1a1aa}
  .btn{margin-top:4px;align-self:flex-start;height:32px;padding:0 14px;border-radius:8px;border:1px solid #6366f1;background:#6366f1;color:#fff;font-size:12.5px;font-weight:500;text-decoration:none;display:inline-flex;align-items:center;gap:6px}
  .btn:hover{background:#4f46e5;border-color:#4f46e5}
</style>
</head>
<body>
  <main class="card">
    <span class="badge">404</span>
    <h1>Internal page not found</h1>
    <p>This internal Studio route doesn't exist or has moved. Return to the Start Page to continue browsing.</p>
    <a class="btn" href="fynexo://newtab">Back to Start Page</a>
  </main>
</body>
</html>"##;
            wry::http::Response::builder()
                .status(404)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(std::borrow::Cow::from(not_found.as_bytes()))
                .unwrap_or_else(|_| wry::http::Response::new(std::borrow::Cow::Borrowed(&b""[..])))
        }
    });

    let initial_url = to_webview_url(target_url);

    let state_for_ipc = Arc::clone(app_state);
    let tabs_for_ipc = Arc::clone(tab_manager);
    let proxy_for_ipc = event_proxy.clone();
    let tab_id_for_ipc = Arc::clone(&bound_tab_id);

    let builder = builder
        .with_devtools(allow_devtools)
        .with_initialization_script_for_main_only(&initial_script, true)
        .with_ipc_handler(move |req| {
            let tid = tab_id_for_ipc.load(Ordering::Acquire);
            if tid == 0 {
                return;
            }
            let origin = req.uri().to_string();
            handle_ipc_message(
                win_id,
                tid,
                &origin,
                req.body(),
                &state_for_ipc,
                &tabs_for_ipc,
                &proxy_for_ipc,
            );
        })
        .with_url(&initial_url);

    #[cfg(target_os = "windows")]
    let webview = {
        let win_size = window.inner_size();
        let init_bounds = physical_bounds(win_size.width, win_size.height);
        let wv = builder.with_bounds(init_bounds).build_as_child(window)?;
        enable_clip_siblings(&wv);
        sync_webview_bounds(&wv, win_size.width, win_size.height);
        wv
    };

    #[cfg(not(target_os = "windows"))]
    let webview = builder.build(window)?;

    let tab_id_for_win = Arc::clone(&bound_tab_id);
    let proxy_for_win = event_proxy.clone();

    #[cfg(not(target_os = "windows"))]
    let _ = (&tab_id_for_win, &proxy_for_win);

    #[cfg(target_os = "windows")]
    {
        adblock::attach_windows_adblock(&webview, &adblock_cfg);
        adblock::attach_windows_resilience(
            &webview,
            incognito,
            win_id,
            tab_id_for_win,
            proxy_for_win,
        );
    }

    if (default_zoom - 1.0).abs() > 0.01 {
        let _ = webview.zoom(default_zoom);
    }

    Ok(TabWebView {
        webview,
        bound_tab_id,
    })
}

pub fn build_extension_popup_webview(
    window: &tao::window::Window,
    web_context: &mut WebContext,
    popup_url: &str,
    app_state: &Arc<Mutex<AppState>>,
    event_proxy: &EventLoopProxy<UserEvent>,
    target_win_id: usize,
) -> Result<wry::WebView, Box<dyn Error>> {
    if !super::security::is_safe_chrome_extension_url(popup_url) {
        return Err("Invalid chrome-extension popup URL".into());
    }

    #[cfg(target_os = "windows")]
    let (boot_hw_accel, boot_chrome_ext);
    let (incognito, resolved_ua, allow_devtools) = {
        let st = app_state.lock_safe();
        #[cfg(target_os = "windows")]
        {
            boot_hw_accel = st.boot_hw_accel;
            boot_chrome_ext = st.boot_chrome_ext_enabled;
        }
        (
            st.config.incognito,
            st.config.resolved_user_agent(),
            cfg!(debug_assertions) || st.config.allow_page_eval,
        )
    };

    let mut builder = WebViewBuilder::new_with_web_context(web_context)
        .with_background_color((24, 24, 27, 255))
        .with_visible(true)
        .with_incognito(incognito);

    if let Some(ua) = resolved_ua {
        builder = builder.with_user_agent(&ua);
    }

    #[cfg(target_os = "windows")]
    {
        let args = windows_browser_args(boot_hw_accel);
        builder = builder
            .with_additional_browser_args(&args)
            .with_browser_extensions_enabled(boot_chrome_ext);
    }

    let proxy_for_nw = event_proxy.clone();
    builder = builder.with_new_window_req_handler(move |url, _| {
        if super::security::is_safe_external_http_url(&url)
            || super::security::is_safe_chrome_extension_url(&url)
        {
            let _ = proxy_for_nw.send_event(UserEvent::OpenUrlInWindow {
                win_id: target_win_id,
                url,
            });
        }
        wry::NewWindowResponse::Deny
    });

    builder = builder.with_navigation_handler(|url| {
        super::security::is_safe_chrome_extension_url(&url)
            || super::security::is_safe_external_http_url(&url)
            || url == "about:blank"
    });

    let proxy_for_ipc = event_proxy.clone();
    let esc_script = "window.addEventListener('keydown',function(e){if(e.key==='Escape'&&window.ipc){window.ipc.postMessage('{\"type\":\"CLOSE_CHROME_EXT_POPUP\"}');}},true);";
    let builder = builder
        .with_devtools(allow_devtools)
        .with_initialization_script(esc_script)
        .with_ipc_handler(move |req| {
            let origin = req.uri().to_string();
            if super::security::is_safe_chrome_extension_url(&origin) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(req.body()) {
                    if v.get("type").and_then(|t| t.as_str()) == Some("CLOSE_CHROME_EXT_POPUP") {
                        let _ = proxy_for_ipc.send_event(UserEvent::CloseChromeExtPopup);
                    }
                }
            }
        })
        .with_url(popup_url);

    #[cfg(target_os = "windows")]
    let webview = {
        let sz = window.inner_size();
        let wv = builder
            .with_bounds(physical_bounds(sz.width, sz.height))
            .build_as_child(window)?;
        sync_webview_bounds(&wv, sz.width, sz.height);
        wv
    };

    #[cfg(not(target_os = "windows"))]
    let webview = builder.build(window)?;

    Ok(webview)
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn physical_bounds(width: u32, height: u32) -> wry::Rect {
    wry::Rect {
        position: wry::dpi::Position::Physical(wry::dpi::PhysicalPosition::new(0, 0)),
        size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(
            width.max(1),
            height.max(1),
        )),
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn sync_webview_bounds(webview: &wry::WebView, width: u32, height: u32) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
    use wry::WebViewExtWindows;
    let w = width.max(1) as i32;
    let h = height.max(1) as i32;
    let mut rc = RECT::default();
    if unsafe { GetClientRect(webview.hwnd(), &mut rc).is_ok() }
        && (rc.right - rc.left) == w
        && (rc.bottom - rc.top) == h
    {
        return;
    }
    let _ = webview.set_bounds(physical_bounds(width, height));
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn enable_clip_siblings(webview: &wry::WebView) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW, GWL_STYLE, WS_CLIPSIBLINGS,
    };
    use wry::WebViewExtWindows;
    let hwnd = webview.hwnd();
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        if (style & WS_CLIPSIBLINGS.0 as i32) == 0 {
            let _ = SetWindowLongW(hwnd, GWL_STYLE, style | WS_CLIPSIBLINGS.0 as i32);
        }
    }
}
