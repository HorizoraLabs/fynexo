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

use super::events::{HudInsetLogical, HudRegionRectLogical, UserEvent};
use super::ipc::handle_ipc_message;
use super::state::AppState;
use super::url::{normalize_logical_url, to_webview_url};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContentBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HudRegionRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub r: i32,
}

pub struct TabWebView {
    pub webview: wry::WebView,
    pub bound_tab_id: Arc<AtomicUsize>,
}

pub struct TabLaunch<'a> {
    pub win_id: usize,
    pub tab_id: usize,
    pub url: &'a str,
    pub visible: bool,
    pub init_bounds: ContentBounds,
}

fn parse_px_f64(raw: &str, default_val: f64) -> f64 {
    let trimmed = raw.trim();
    let num_part = trimmed.strip_suffix("px").unwrap_or(trimmed).trim();
    num_part
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(default_val)
}

pub fn estimate_initial_inset(
    state: &AppState,
    active_url: &str,
    is_fullscreen: bool,
) -> HudInsetLogical {
    if is_fullscreen || !state.styles.topbar.visible_on_desktop {
        return HudInsetLogical::default();
    }
    let layout = state.styles.topbar.layout.trim().to_ascii_lowercase();
    let nav_h = parse_px_f64(&state.styles.topbar.height, 40.0);
    let titlebar_h = parse_px_f64(&state.styles.topbar.titlebar_height, 42.0);
    let bb = state.styles.topbar.bookmarks_bar.trim().to_ascii_lowercase();
    let is_newtab = normalize_logical_url(active_url) == "fynexo://newtab";
    let bm_shown = bb == "always" || (bb == "newtab" && is_newtab);
    let bm_extra = if bm_shown { 29.0 } else { 1.0 };

    if layout == "vertical" {
        let vw = parse_px_f64(&state.styles.topbar.vertical_width, 216.0);
        HudInsetLogical {
            top: nav_h + bm_extra,
            left: vw,
            right: 0.0,
            bottom: 0.0,
        }
    } else if layout == "compact" {
        HudInsetLogical {
            top: nav_h + bm_extra,
            left: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    } else {
        HudInsetLogical {
            top: titlebar_h + nav_h + bm_extra,
            left: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }
}

pub fn estimate_initial_regions(
    inset: HudInsetLogical,
    win_w_logical: f64,
    win_h_logical: f64,
) -> Vec<HudRegionRectLogical> {
    let mut regions = Vec::new();
    let lw = win_w_logical.max(1.0);
    let lh = win_h_logical.max(1.0);
    if inset.top > 0.5 {
        regions.push(HudRegionRectLogical {
            x: 0.0,
            y: 0.0,
            w: lw,
            h: inset.top,
            r: 0.0,
        });
    }
    if inset.left > 0.5 {
        regions.push(HudRegionRectLogical {
            x: 0.0,
            y: inset.top.max(0.0),
            w: inset.left,
            h: (lh - inset.top.max(0.0)).max(1.0),
            r: 0.0,
        });
    }
    regions
}

pub fn compute_content_bounds(
    win_w: u32,
    win_h: u32,
    sf: f64,
    inset: HudInsetLogical,
) -> ContentBounds {
    let scale = if sf.is_finite() && sf > 0.1 { sf } else { 1.0 };
    let w = win_w.max(1) as i32;
    let h = win_h.max(1) as i32;
    let x = ((inset.left.max(0.0) * scale).round() as i32).clamp(0, w.saturating_sub(1));
    let y = ((inset.top.max(0.0) * scale).round() as i32).clamp(0, h.saturating_sub(1));
    let r = (inset.right.max(0.0) * scale).round() as i32;
    let b = (inset.bottom.max(0.0) * scale).round() as i32;
    ContentBounds {
        x,
        y,
        width: (w - x - r).max(1) as u32,
        height: (h - y - b).max(1) as u32,
    }
}

pub fn logical_regions_to_physical(
    regions: &[HudRegionRectLogical],
    win_w: u32,
    win_h: u32,
    sf: f64,
    content_bounds: ContentBounds,
) -> Vec<HudRegionRect> {
    let scale = if sf.is_finite() && sf > 0.1 { sf } else { 1.0 };
    let max_w = win_w as i32;
    let max_h = win_h as i32;
    let mut out = Vec::with_capacity(regions.len());
    for reg in regions {
        let mut x = (reg.x * scale).round() as i32;
        let mut y = (reg.y * scale).round() as i32;
        let mut x2 = ((reg.x + reg.w) * scale).round() as i32;
        let mut y2 = ((reg.y + reg.h) * scale).round() as i32;

        // Snap contiguous HUD base bars directly to content_bounds to eliminate 1px DPI seams.
        if reg.r <= 0.1 {
            let right_edge = content_bounds.x + content_bounds.width as i32;
            if x <= 1
                && y <= 1
                && content_bounds.y > 0
                && (y2 - content_bounds.y).abs() <= 2
                && x2 >= max_w.saturating_sub(2)
            {
                x = 0;
                y = 0;
                x2 = max_w;
                y2 = content_bounds.y;
            } else if x <= 1
                && content_bounds.x > 0
                && (x2 - content_bounds.x).abs() <= 2
                && y2 >= max_h.saturating_sub(2)
            {
                x = 0;
                y = content_bounds.y;
                x2 = content_bounds.x;
                y2 = max_h;
            } else if right_edge < max_w
                && (x - right_edge).abs() <= 2
                && x2 >= max_w.saturating_sub(2)
                && y2 >= max_h.saturating_sub(2)
            {
                x = right_edge;
                y = content_bounds.y;
                x2 = max_w;
                y2 = max_h;
            }
        }

        x = x.clamp(0, max_w);
        y = y.clamp(0, max_h);
        x2 = x2.clamp(0, max_w);
        y2 = y2.clamp(0, max_h);
        let w = x2 - x;
        let h = y2 - y;
        if w > 0 && h > 0 {
            out.push(HudRegionRect {
                x,
                y,
                w,
                h,
                r: (reg.r.max(0.0) * scale).round() as i32,
            });
        }
    }
    out
}

fn is_local_dev_host(host: &str) -> bool {
    if host.is_empty() || host == "fynexo.localhost" {
        return false;
    }
    if host == "localhost"
        || host == "0.0.0.0"
        || host == "::1"
        || host == "[::1]"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".test")
    {
        return true;
    }
    host.starts_with("127.") || host.starts_with("10.") || host.starts_with("192.168.")
}

fn site_override_key_for_url(raw_url: &str) -> Option<String> {
    if super::security::is_internal_origin(raw_url) {
        return None;
    }
    let lower = raw_url.trim().to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let (hostname, port) = if let Some(after_bracket) = host_port.strip_prefix('[') {
        if let Some((ip6, tail)) = after_bracket.split_once(']') {
            let p = tail.strip_prefix(':').unwrap_or("");
            (format!("[{}]", ip6), p)
        } else {
            (host_port.to_string(), "")
        }
    } else if let Some((h, p)) = host_port.rsplit_once(':') {
        (h.to_string(), p)
    } else {
        (host_port.to_string(), "")
    };
    if hostname.is_empty() {
        return None;
    }
    if !port.is_empty() && (is_local_dev_host(&hostname) || (port != "80" && port != "443")) {
        Some(format!("{}:{}", hostname, port))
    } else {
        Some(hostname)
    }
}

pub fn effective_zoom_for_url(state: &AppState, raw_url: &str) -> f64 {
    let base = state.config.default_zoom;
    let z = site_override_key_for_url(raw_url)
        .and_then(|key| state.config.site_overrides.get(&key))
        .and_then(|ov| ov.zoom)
        .unwrap_or(base);
    z.clamp(0.5, 2.5)
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
    let (initial_script, boot_hw_accel, boot_chrome_ext, adblock_cfg, incognito, resolved_ua, default_zoom, allow_devtools) = {
        let mut st = app_state.lock_safe();
        let _ = st.reload_if_changed();
        let role = hud::HudRole::ContentTab;
        let script = hud::generate_hud_script_for_role(&st, Some(&initial_tabs), role);
        let zoom = effective_zoom_for_url(&st, target_url);
        (
            script,
            st.boot_hw_accel,
            st.boot_chrome_ext_enabled,
            st.adblock.clone(),
            st.config.incognito || tm_incog,
            st.config.resolved_user_agent(),
            zoom,
            cfg!(debug_assertions) || st.config.allow_page_eval,
        )
    };
    #[cfg(not(target_os = "windows"))]
    let (incognito, resolved_ua, default_zoom, allow_devtools, initial_script) = {
        let mut st = app_state.lock_safe();
        let _ = st.reload_if_changed();
        let script = hud::generate_hud_script_from_state(&st, Some(&initial_tabs));
        let zoom = effective_zoom_for_url(&st, target_url);
        (
            st.config.incognito || tm_incog,
            st.config.resolved_user_agent(),
            zoom,
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
        let norm = normalize_logical_url(&page_url);
        if matches!(norm.as_str(), "about:blank" | "") {
            return;
        }
        let tid = tab_id_for_load.load(Ordering::Acquire);
        if tid == 0 {
            return;
        }
        match event {
            PageLoadEvent::Started => {
                let _ = proxy_for_load.send_event(UserEvent::PageStarted {
                    win_id,
                    tab_id: tid,
                });
            }
            PageLoadEvent::Finished => {
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
        let wv = builder
            .with_bounds(physical_bounds_rect(launch.init_bounds))
            .build_as_child(window)?;
        enable_clip_siblings(&wv);
        sync_webview_bounds_rect(&wv, launch.init_bounds);
        wv
    };

    #[cfg(not(target_os = "windows"))]
    let webview = {
        let _ = launch.init_bounds;
        builder.build(window)?
    };

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
#[allow(clippy::too_many_arguments)]
pub fn build_hud_webview(
    window: &tao::window::Window,
    web_context: &mut WebContext,
    win_id: usize,
    incognito_window: bool,
    initial_inset: HudInsetLogical,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    event_proxy: &EventLoopProxy<UserEvent>,
) -> Result<(wry::WebView, Vec<HudRegionRectLogical>), Box<dyn Error>> {
    let initial_tabs = tab_manager.lock_safe().get_state();
    let (incognito, resolved_ua, boot_hw_accel, boot_chrome_ext, allow_devtools, hud_script) = {
        let mut st = app_state.lock_safe();
        let _ = st.reload_if_changed();
        let script = hud::generate_hud_script_for_role(&st, Some(&initial_tabs), hud::HudRole::HudShell);
        (
            st.config.incognito || incognito_window,
            st.config.resolved_user_agent(),
            st.boot_hw_accel,
            st.boot_chrome_ext_enabled,
            cfg!(debug_assertions) || st.config.allow_page_eval,
            script,
        )
    };

    let mut builder = WebViewBuilder::new_with_web_context(web_context)
        .with_transparent(true)
        .with_background_color((0, 0, 0, 0))
        .with_visible(true)
        .with_incognito(incognito);

    if let Some(ua) = resolved_ua {
        builder = builder.with_user_agent(&ua);
    }

    let args = windows_browser_args(boot_hw_accel);
    builder = builder
        .with_additional_browser_args(&args)
        .with_browser_extensions_enabled(boot_chrome_ext);

    builder = builder.with_navigation_handler(|url| {
        let lower = url.trim().to_ascii_lowercase();
        lower == "http://fynexo.localhost/hud"
            || lower.starts_with("http://fynexo.localhost/hud?")
            || lower == "about:blank"
    });

    builder = builder.with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny);

    builder = builder.with_custom_protocol("fynexo".to_string(), move |_id, req| {
        let route = super::security::resolve_internal_route(
            req.uri().host().unwrap_or(""),
            req.uri().path(),
        );
        if route == Some("hud") {
            wry::http::Response::builder()
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Cache-Control", "no-store")
                .body(std::borrow::Cow::Borrowed(hud::hud_shell_html().as_bytes()))
                .unwrap_or_else(|_| wry::http::Response::new(std::borrow::Cow::Borrowed(&b""[..])))
        } else {
            wry::http::Response::builder()
                .status(404)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(std::borrow::Cow::Borrowed(&b"Not Found"[..]))
                .unwrap_or_else(|_| wry::http::Response::new(std::borrow::Cow::Borrowed(&b""[..])))
        }
    });

    let state_for_ipc = Arc::clone(app_state);
    let tabs_for_ipc = Arc::clone(tab_manager);
    let proxy_for_ipc = event_proxy.clone();

    let builder = builder
        .with_devtools(allow_devtools)
        .with_initialization_script_for_main_only(&hud_script, true)
        .with_ipc_handler(move |req| {
            let origin = req.uri().to_string();
            handle_ipc_message(
                win_id,
                0,
                &origin,
                req.body(),
                &state_for_ipc,
                &tabs_for_ipc,
                &proxy_for_ipc,
            );
        })
        .with_url("http://fynexo.localhost/hud");

    let win_size = window.inner_size();
    let sf = window.scale_factor().max(0.1);
    let wv = builder
        .with_bounds(physical_bounds(win_size.width, win_size.height))
        .build_as_child(window)?;
    enable_clip_siblings(&wv);
    sync_webview_bounds(&wv, win_size.width, win_size.height);
    bring_hud_to_top(&wv);

    let init_regions_logical = estimate_initial_regions(
        initial_inset,
        win_size.width as f64 / sf,
        win_size.height as f64 / sf,
    );
    let cbounds = compute_content_bounds(win_size.width, win_size.height, sf, initial_inset);
    let phys_regions = logical_regions_to_physical(
        &init_regions_logical,
        win_size.width,
        win_size.height,
        sf,
        cbounds,
    );
    apply_hud_window_region(&wv, &phys_regions);

    Ok((wv, init_regions_logical))
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn physical_bounds_rect(bounds: ContentBounds) -> wry::Rect {
    wry::Rect {
        position: wry::dpi::Position::Physical(wry::dpi::PhysicalPosition::new(bounds.x, bounds.y)),
        size: wry::dpi::Size::Physical(wry::dpi::PhysicalSize::new(
            bounds.width.max(1),
            bounds.height.max(1),
        )),
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn physical_bounds(width: u32, height: u32) -> wry::Rect {
    physical_bounds_rect(ContentBounds {
        x: 0,
        y: 0,
        width: width.max(1),
        height: height.max(1),
    })
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn sync_webview_bounds_rect(webview: &wry::WebView, bounds: ContentBounds) {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::Graphics::Gdi::ScreenToClient;
    use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetParent, GetWindowRect};
    use wry::WebViewExtWindows;
    let w = bounds.width.max(1) as i32;
    let h = bounds.height.max(1) as i32;
    let mut rc = RECT::default();
    let mut wrc = RECT::default();
    let hwnd = webview.hwnd();
    if unsafe { GetClientRect(hwnd, &mut rc).is_ok() }
        && (rc.right - rc.left) == w
        && (rc.bottom - rc.top) == h
    {
        if let Ok(parent) = unsafe { GetParent(hwnd) } {
            if !parent.is_invalid() && unsafe { GetWindowRect(hwnd, &mut wrc).is_ok() } {
                let mut pt = POINT {
                    x: wrc.left,
                    y: wrc.top,
                };
                if unsafe { ScreenToClient(parent, &mut pt).as_bool() }
                    && pt.x == bounds.x
                    && pt.y == bounds.y
                {
                    return;
                }
            }
        }
    }
    let _ = webview.set_bounds(physical_bounds_rect(bounds));
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn sync_webview_bounds(webview: &wry::WebView, width: u32, height: u32) {
    sync_webview_bounds_rect(
        webview,
        ContentBounds {
            x: 0,
            y: 0,
            width: width.max(1),
            height: height.max(1),
        },
    );
}

#[cfg(target_os = "windows")]
pub(super) fn is_hud_taking_focus(hud_wv: &wry::WebView) -> bool {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
    use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, IsChild, WindowFromPoint};
    use wry::WebViewExtWindows;

    let hud_hwnd = hud_wv.hwnd();
    unsafe {
        let focused = GetFocus();
        if !focused.is_invalid()
            && (focused == hud_hwnd || IsChild(hud_hwnd, focused).as_bool())
        {
            return true;
        }
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_ok() {
            let target = WindowFromPoint(pt);
            if !target.is_invalid()
                && (target == hud_hwnd || IsChild(hud_hwnd, target).as_bool())
            {
                return true;
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn bring_hud_to_top(webview: &wry::WebView) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
    use wry::WebViewExtWindows;
    unsafe {
        let _ = SetWindowPos(
            webview.hwnd(),
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub(super) fn place_behind_hud(webview: &wry::WebView, hud_wv: &wry::WebView) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
    use wry::WebViewExtWindows;
    unsafe {
        let _ = SetWindowPos(
            webview.hwnd(),
            Some(hud_wv.hwnd()),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

#[cfg(target_os = "windows")]
pub(super) fn apply_hud_window_region(hud_wv: &wry::WebView, rects: &[HudRegionRect]) {
    use windows::Win32::Graphics::Gdi::{
        CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject, SetWindowRgn, HGDIOBJ, HRGN,
        RGN_OR,
    };
    use wry::WebViewExtWindows;
    unsafe {
        let mut acc: Option<HRGN> = None;
        for rect in rects {
            if rect.w <= 0 || rect.h <= 0 {
                continue;
            }
            let rgn = if rect.r > 0 {
                let diam = (rect.r * 2).min(rect.w).min(rect.h);
                CreateRoundRectRgn(
                    rect.x,
                    rect.y,
                    rect.x + rect.w + 1,
                    rect.y + rect.h + 1,
                    diam,
                    diam,
                )
            } else {
                CreateRectRgn(rect.x, rect.y, rect.x + rect.w, rect.y + rect.h)
            };
            if rgn.is_invalid() {
                continue;
            }
            if let Some(prev) = acc {
                let _ = CombineRgn(Some(prev), Some(prev), Some(rgn), RGN_OR);
                let _ = DeleteObject(HGDIOBJ(rgn.0));
            } else {
                acc = Some(rgn);
            }
        }
        let final_rgn = match acc {
            Some(r) => r,
            None => CreateRectRgn(0, 0, 0, 0),
        };
        if !final_rgn.is_invalid() {
            let ok = SetWindowRgn(hud_wv.hwnd(), Some(final_rgn), true);
            if ok == 0 {
                let _ = DeleteObject(HGDIOBJ(final_rgn.0));
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_bounds_respect_insets_and_dpi() {
        let b = compute_content_bounds(
            1920,
            1080,
            1.25,
            HudInsetLogical {
                top: 80.0,
                left: 200.0,
                right: 380.0,
                bottom: 0.0,
            },
        );
        assert_eq!(b.x, 250);
        assert_eq!(b.y, 100);
        assert_eq!(b.width, 1920 - 250 - 475);
        assert_eq!(b.height, 980);
    }

    #[test]
    fn site_override_zoom_matches_host_and_port() {
        crate::util::isolate_data_dir();
        let mut st = AppState::load();
        st.config.default_zoom = 1.0;
        st.config.site_overrides.insert(
            "example.com".to_string(),
            crate::config::SiteOverride {
                zoom: Some(1.3),
                dark_mode: String::new(),
            },
        );
        assert!((effective_zoom_for_url(&st, "https://example.com/path") - 1.3).abs() < 1e-6);
        assert!((effective_zoom_for_url(&st, "fynexo://newtab") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn logical_regions_to_physical_snaps_chrome_without_clamping_modals() {
        let sf = 1.25;
        let inset = HudInsetLogical {
            top: 80.0,
            left: 200.0,
            right: 320.0,
            bottom: 0.0,
        };
        let bounds = compute_content_bounds(1920, 1080, sf, inset);
        let input = vec![
            // Topbar
            HudRegionRectLogical { x: 0.0, y: 0.0, w: 1536.0, h: 79.6, r: 0.0 },
            // Vertical sidebar
            HudRegionRectLogical { x: 0.0, y: 80.0, w: 199.4, h: 784.0, r: 0.0 },
            // Right side panel
            HudRegionRectLogical { x: 1216.2, y: 80.0, w: 319.8, h: 784.0, r: 0.0 },
            // Full-width / centered modal card (must NOT be squashed to topbar height)
            HudRegionRectLogical { x: 0.0, y: 0.0, w: 1536.0, h: 864.0, r: 0.0 },
            // Floating modal card
            HudRegionRectLogical { x: 400.0, y: 120.0, w: 736.0, h: 420.0, r: 12.0 },
        ];
        let out = logical_regions_to_physical(&input, 1920, 1080, sf, bounds);
        assert_eq!(out.len(), 5);
        assert_eq!(out[0], HudRegionRect { x: 0, y: 0, w: 1920, h: bounds.y, r: 0 });
        assert_eq!(
            out[1],
            HudRegionRect { x: 0, y: bounds.y, w: bounds.x, h: 1080 - bounds.y, r: 0 }
        );
        let right_x = bounds.x + bounds.width as i32;
        assert_eq!(
            out[2],
            HudRegionRect {
                x: right_x,
                y: bounds.y,
                w: 1920 - right_x,
                h: 1080 - bounds.y,
                r: 0,
            }
        );
        assert_eq!(out[3], HudRegionRect { x: 0, y: 0, w: 1920, h: 1080, r: 0 });
        assert_eq!(out[4], HudRegionRect { x: 500, y: 150, w: 920, h: 525, r: 15 });
    }
}
