use std::error::Error;
use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;
use wry::{PageLoadEvent, WebContext, WebViewBuilder};

#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

use crate::adblock;
use crate::config::unique_download_path;
use crate::hud;
use crate::pages;
use crate::tabs::TabManager;
use crate::util::LockExt;

use super::events::UserEvent;
use super::ipc::handle_ipc_message;
use super::state::AppState;
use super::url::{normalize_logical_url, to_webview_url};

pub fn build_tab_webview(
    window: &tao::window::Window,
    web_context: &mut WebContext,
    target_url: &str,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    event_proxy: &EventLoopProxy<UserEvent>,
) -> Result<wry::WebView, Box<dyn Error>> {
    let initial_payload = {
        let mut st = app_state.lock_safe();
        let _ = st.reload_if_changed();
        st.payload()
    };

    let mut builder = WebViewBuilder::new_with_web_context(web_context)
        .with_background_color((9, 9, 11, 255))
        .with_incognito(initial_payload.config.incognito)
        .with_back_forward_navigation_gestures(true);

    if let Some(ua) = initial_payload.config.resolved_user_agent() {
        builder = builder.with_user_agent(&ua);
    }

    #[cfg(target_os = "windows")]
    {
        // Deep lightweight optimization: single renderer process, HTTP/3 QUIC, GPU zero-copy rasterization, zero telemetry
        builder = builder.with_additional_browser_args(
            "--renderer-process-limit=1 \
            --js-flags=\"--max-old-space-size=256 --optimize-for-size\" \
            --enable-gpu-rasterization \
            --enable-zero-copy \
            --enable-quic \
            --enable-smooth-scrolling \
            --disable-backgrounding-occluded-windows \
            --disable-renderer-backgrounding \
            --disable-background-networking \
            --disable-component-update \
            --disable-domain-reliability \
            --disable-sync \
            --disable-breakpad \
            --disable-extensions \
            --disable-speech-api \
            --no-default-browser-check \
            --no-first-run \
            --disable-client-side-phishing-detection \
            --disable-hang-monitor \
            --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,Prerender2,TopicsAPI,Translate,MediaRouter,OptimizationHints,AutofillServerCommunication \
            --enable-features=OverlayScrollbar,OverlayScrollbarWinStyle,OverlayScrollbarWinStyleAnimation,FluentOverlayScrollbar \
            --disk-cache-size=33554432 \
            ",
        );
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
        let logical = normalize_logical_url(&url);
        let (new_id, nav_url) = tabs_for_new_window.lock_safe().new_tab(&logical);
        let _ = proxy_for_new_window.send_event(UserEvent::SwitchTab { id: new_id, url: nav_url });
        let _ = proxy_for_new_window.send_event(UserEvent::SyncTabs);
        wry::NewWindowResponse::Deny
    });

    // Main-Frame Navigation Guard: Block Ad Redirects & Route External Schemes Safely
    #[allow(unused_variables)]
    let proxy_for_nav = event_proxy.clone();
    builder = builder.with_navigation_handler(move |url| {
        let lower = url.to_ascii_lowercase();
        let is_standard = lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("fynexo://")
            || lower.starts_with("about:")
            || lower.starts_with("data:")
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
    builder = builder.with_download_started_handler(move |_url, dest_path| {
        let suggested = dest_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("download.bin");
        let final_path = unique_download_path(suggested);
        let display_name = final_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        *dest_path = final_path;
        let _ = proxy_for_dl_start
            .send_event(UserEvent::Toast(format!("Downloading {}", display_name)));
        true
    });

    let proxy_for_dl_done = event_proxy.clone();
    builder = builder.with_download_completed_handler(move |_url, path_opt, success| {
        if success {
            let name = path_opt
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let _ = proxy_for_dl_done
                .send_event(UserEvent::Toast(format!("Saved {} to Downloads", name)));
        } else {
            let _ = proxy_for_dl_done
                .send_event(UserEvent::Toast("Download cancelled or failed".to_string()));
        }
    });

    // Restore per-tab reading scroll depth after page load finishes
    let tabs_for_load = Arc::clone(tab_manager);
    let proxy_for_load = event_proxy.clone();
    builder = builder.with_on_page_load_handler(move |event, _url| {
        if matches!(event, PageLoadEvent::Finished) {
            let scroll_y = tabs_for_load.lock_safe().active_scroll_y();
            if scroll_y > 24.0 {
                let _ = proxy_for_load.send_event(UserEvent::RestoreScroll(scroll_y));
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
            let mut st = state_for_proto.lock_safe();
            let _ = st.reload_if_changed();
            let payload = st.payload();
            drop(st);
            let html = if is_settings {
                pages::settings_html(&payload)
            } else {
                pages::newtab_html(&payload)
            };
            wry::http::Response::builder()
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Cache-Control", "no-store")
                .body(std::borrow::Cow::from(html.into_bytes()))
                .unwrap()
        } else {
            let not_found = "<!DOCTYPE html><html><body style='background:#09090b;color:#f4f4f6;font-family:sans-serif;padding:30px;'><h2>404 - Fynexo Internal Page Not Found</h2></body></html>";
            wry::http::Response::builder()
                .status(404)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(std::borrow::Cow::from(not_found.as_bytes()))
                .unwrap()
        }
    });

    let initial_tabs = tab_manager.lock_safe().get_state();
    let initial_script = hud::generate_hud_script(&initial_payload, Some(&initial_tabs));
    let initial_url = to_webview_url(target_url);

    let state_for_ipc = Arc::clone(app_state);
    let tabs_for_ipc = Arc::clone(tab_manager);
    let proxy_for_ipc = event_proxy.clone();

    let webview = builder
        .with_devtools(cfg!(debug_assertions))
        .with_initialization_script_for_main_only(&initial_script, true)
        .with_ipc_handler(move |req| {
            let origin = req.uri().to_string();
            handle_ipc_message(&origin, req.body(), &state_for_ipc, &tabs_for_ipc, &proxy_for_ipc);
        })
        .with_url(&initial_url)
        .build(window)?;

    #[cfg(target_os = "windows")]
    adblock::attach_windows_adblock(&webview, &initial_payload.adblock);

    if (initial_payload.config.default_zoom - 1.0).abs() > 0.01 {
        let _ = webview.zoom(initial_payload.config.default_zoom);
    }

    Ok(webview)
}
