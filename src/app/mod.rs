pub mod events;
pub mod ipc;
pub mod memory;
pub mod security;
pub mod state;
pub mod url;
pub mod webview;

pub use events::UserEvent;
pub use state::AppState;

use std::error::Error;
use std::sync::{Arc, Mutex};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use wry::WebContext;

#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewExtWindows};

use crate::adblock;
use crate::config::data_dir;
use crate::tabs::TabManager;
use crate::util::LockExt;

#[cfg(target_os = "windows")]
use self::memory::{force_empty_working_set, get_current_process_memory_mb};
use self::url::to_webview_url;
use self::webview::build_tab_webview;

pub fn run() -> Result<(), Box<dyn Error>> {
    let app_state = Arc::new(Mutex::new(AppState::load()));
    let initial_payload = app_state.lock_safe().payload();

    let tab_manager = Arc::new(Mutex::new(TabManager::load_or_new(
        &initial_payload.config.homepage,
        initial_payload.config.restore_session,
        initial_payload.config.incognito,
    )));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let event_proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    // Cross-Platform Window Setup (Frameless on Desktop, Native System Insets on Mobile)
    #[allow(unused_mut)]
    let mut win_builder = WindowBuilder::new().with_title("Fynexo");
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        win_builder = win_builder
            .with_decorations(false)
            .with_resizable(true)
            .with_maximized(true)
            .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 840.0));
    }
    let window = win_builder.build(&event_loop)?;

    let mut web_context = WebContext::new(Some(data_dir().join("webview_profile")));
    let active_logical = tab_manager.lock_safe().active_url();
    adblock::set_active_page_url(&active_logical);

    let initial_webview = build_tab_webview(
        &window,
        &mut web_context,
        &active_logical,
        &app_state,
        &tab_manager,
        &event_proxy,
    )?;

    let state_for_event = Arc::clone(&app_state);
    let tabs_for_event = Arc::clone(&tab_manager);
    let event_proxy_for_run = event_proxy.clone();
    let mut last_maximize_time = std::time::Instant::now();

    let initial_active_id = tab_manager.lock_safe().active_id;
    let mut active_tab_id = initial_active_id;
    let mut foreground_webview: Option<wry::WebView> = Some(initial_webview);
    let mut idle_normal_webview: Option<wry::WebView> = None;
    let mut background_webviews: std::collections::HashMap<usize, wry::WebView> = std::collections::HashMap::new();
    let bridge_tok_js =
        serde_json::to_string(security::bridge_token()).unwrap_or_else(|_| "\"\"".to_string());

    // Run the Window Event Lifecycle
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => (),
            Event::UserEvent(UserEvent::Navigate(url)) => {
                let target = to_webview_url(&url);
                #[cfg(debug_assertions)]
                println!("[Navigation Request]: {} -> {}", url, target);
                if let Some(ref mut fg) = foreground_webview {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = fg.set_memory_usage_level(MemoryUsageLevel::Low);
                        let _ = fg.set_memory_usage_level(MemoryUsageLevel::Normal);
                    }
                    let _ = fg.load_url(&target);
                }
            }
            Event::UserEvent(UserEvent::SwitchTab { id, url }) => {
                if id == active_tab_id && !background_webviews.contains_key(&id) {
                    return;
                }
                let target = to_webview_url(&url);

                let old_has_bg = {
                    let tm = tabs_for_event.lock_safe();
                    tm.tabs.iter().find(|t| t.id == active_tab_id).map(|t| t.background_play).unwrap_or(false)
                };

                if old_has_bg {
                    if let Some(old_wv) = foreground_webview.take() {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = old_wv.set_memory_usage_level(MemoryUsageLevel::Low);
                        }
                        let _ = old_wv.set_visible(false);
                        background_webviews.insert(active_tab_id, old_wv);
                    }
                } else {
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(
                            "document.querySelectorAll('video, audio').forEach(el => { try { el.pause(); } catch(_) {} });"
                        );
                    }
                }

                if let Some(bg_wv) = background_webviews.remove(&id) {
                    if let Some(norm_wv) = foreground_webview.take() {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = norm_wv.set_memory_usage_level(MemoryUsageLevel::Low);
                        }
                        let _ = norm_wv.set_visible(false);
                        // Unload the departing webpage so idle_normal_webview keeps 0 page DOM/JS RAM.
                        let _ = norm_wv.load_url("about:blank");
                        idle_normal_webview = Some(norm_wv);
                    }
                    #[cfg(target_os = "windows")]
                    {
                        let _ = bg_wv.set_memory_usage_level(MemoryUsageLevel::Normal);
                    }
                    let _ = bg_wv.set_visible(true);
                    foreground_webview = Some(bg_wv);
                } else {
                    if foreground_webview.is_none() {
                        if let Some(idle) = idle_normal_webview.take() {
                            foreground_webview = Some(idle);
                        } else {
                            match build_tab_webview(
                                &window,
                                &mut web_context,
                                &target,
                                &state_for_event,
                                &tabs_for_event,
                                &event_proxy_for_run,
                            ) {
                                Ok(new_wv) => {
                                    foreground_webview = Some(new_wv);
                                }
                                Err(e) => {
                                    eprintln!("[Fynexo] Failed to create tab webview: {:?}", e);
                                }
                            }
                        }
                    }

                    if let Some(ref mut fg) = foreground_webview {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = fg.set_memory_usage_level(MemoryUsageLevel::Low);
                            let _ = fg.set_memory_usage_level(MemoryUsageLevel::Normal);
                        }
                        let _ = fg.set_visible(true);
                        let _ = fg.load_url(&target);
                    }
                }

                active_tab_id = id;

                let (tab_state, is_tab_muted) = {
                    let tm = tabs_for_event.lock_safe();
                    let muted = tm.tabs.iter().find(|t| t.id == id).map(|t| t.muted).unwrap_or(false);
                    (tm.get_state(), muted)
                };
                if let Ok(tabs_json) = serde_json::to_string(&tab_state) {
                    let script = format!(
                        "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({}, {});",
                        tabs_json, bridge_tok_js
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
                let mute_script = format!(
                    "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({}, {});",
                    if is_tab_muted { "true" } else { "false" },
                    bridge_tok_js
                );
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.evaluate_script(&mute_script);
                }

                background_webviews.retain(|&tab_id, _| {
                    tabs_for_event.lock_safe().tabs.iter().any(|t| t.id == tab_id && t.background_play)
                });
            }
            Event::UserEvent(UserEvent::ToggleBackgroundPlay { id, active }) => {
                if !active {
                    if let Some(wv) = background_webviews.remove(&id) {
                        let _ = wv.evaluate_script(
                            "document.querySelectorAll('video, audio').forEach(el => { try { el.pause(); } catch(_) {} });"
                        );
                        drop(wv);
                    }
                }
            }
            Event::UserEvent(UserEvent::ToggleTabMute { id, muted }) => {
                let mute_script = format!(
                    "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({}, {});",
                    if muted { "true" } else { "false" },
                    bridge_tok_js
                );
                if id == active_tab_id {
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&mute_script);
                    }
                } else if let Some(bg_wv) = background_webviews.get(&id) {
                    let _ = bg_wv.evaluate_script(&mute_script);
                }
            }
            Event::UserEvent(UserEvent::BootSync) => {
                let payload = state_for_event.lock_safe().payload();
                let (tab_state, scroll_y, active_url) = {
                    let tm = tabs_for_event.lock_safe();
                    (tm.get_state(), tm.active_scroll_y(), tm.active_url())
                };
                if let (Ok(rt_json), Ok(tabs_json)) = (
                    serialize_runtime_for_url(&payload, &active_url),
                    serde_json::to_string(&tab_state),
                ) {
                    let script = if scroll_y > 24.0 {
                        format!(
                            "window.__fynexo_boot_sync__ && window.__fynexo_boot_sync__({}, {}, {}); window.__fynexo_restore_scroll__ && window.__fynexo_restore_scroll__({}, {});",
                            rt_json, tabs_json, bridge_tok_js, scroll_y, bridge_tok_js
                        )
                    } else {
                        format!(
                            "window.__fynexo_boot_sync__ && window.__fynexo_boot_sync__({}, {}, {});",
                            rt_json, tabs_json, bridge_tok_js
                        )
                    };
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncConfig) => {
                let payload = state_for_event.lock_safe().payload();
                let active_url = tabs_for_event.lock_safe().active_url();
                if let Ok(json) = serialize_runtime_for_url(&payload, &active_url) {
                    let script = format!(
                        "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                        json, bridge_tok_js
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncTabs) => {
                let state = tabs_for_event.lock_safe().get_state();
                if let Ok(json) = serde_json::to_string(&state) {
                    let script = format!(
                        "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({}, {});",
                        json, bridge_tok_js
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
                background_webviews.retain(|&tab_id, wv| {
                    let keep = tabs_for_event.lock_safe().tabs.iter().any(|t| t.id == tab_id && t.background_play);
                    if !keep {
                        let _ = wv.evaluate_script(
                            "document.querySelectorAll('video, audio').forEach(el => { try { el.pause(); } catch(_) {} });"
                        );
                    }
                    keep
                });
            }
            Event::UserEvent(UserEvent::RestoreScroll(scroll_y)) => {
                if scroll_y > 24.0 {
                    let script = format!(
                        "window.__fynexo_restore_scroll__ && window.__fynexo_restore_scroll__({}, {});",
                        scroll_y, bridge_tok_js
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::Toast(msg)) => {
                if let Ok(json_msg) = serde_json::to_string(&msg) {
                    let script = format!(
                        "window.__fynexo_toast__ && window.__fynexo_toast__({}, {});",
                        json_msg, bridge_tok_js
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::ToggleFullscreen(explicit)) => {
                let is_currently_full = window.fullscreen().is_some();
                let next_full = explicit.unwrap_or(!is_currently_full);
                if next_full {
                    window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
                } else {
                    window.set_fullscreen(None);
                }
                let script = format!(
                    "window.__fynexo_set_fullscreen__ && window.__fynexo_set_fullscreen__({}, {});",
                    if next_full { "true" } else { "false" },
                    bridge_tok_js
                );
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.evaluate_script(&script);
                }
            }
            Event::UserEvent(UserEvent::ApplyZoom(factor)) => {
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.zoom(factor);
                }
            }
            Event::UserEvent(UserEvent::ClearSiteData) => {
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.clear_all_browsing_data();
                }
            }
            Event::UserEvent(UserEvent::SyncAdblockFilters) => {
                #[cfg(target_os = "windows")]
                {
                    let adb = state_for_event.lock_safe().adblock.clone();
                    if let Some(ref fg) = foreground_webview {
                        adblock::sync_windows_custom_filters(fg, &adb);
                    }
                }
            }
            Event::UserEvent(UserEvent::UpdateUserAgent(ua_opt)) => {
                #[cfg(target_os = "windows")]
                if let Some(ref fg) = foreground_webview {
                    adblock::apply_windows_user_agent(fg, ua_opt.as_deref());
                }
                #[cfg(not(target_os = "windows"))]
                let _ = ua_opt;
            }
            Event::UserEvent(UserEvent::WindowDrag) => {
                let _ = window.drag_window();
            }
            Event::UserEvent(UserEvent::WindowMinimize) => {
                window.set_minimized(true);
                #[cfg(target_os = "windows")]
                {
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.set_memory_usage_level(MemoryUsageLevel::Low);
                    }
                    for bg_wv in background_webviews.values() {
                        let _ = bg_wv.set_memory_usage_level(MemoryUsageLevel::Low);
                    }
                    if let Some(ref idl) = idle_normal_webview {
                        let _ = idl.set_memory_usage_level(MemoryUsageLevel::Low);
                    }
                }
            }
            Event::UserEvent(UserEvent::WindowMaximize) => {
                let now = std::time::Instant::now();
                if now.duration_since(last_maximize_time) >= std::time::Duration::from_millis(300) {
                    last_maximize_time = now;
                    let is_max = window.is_maximized();
                    window.set_maximized(!is_max);
                }
            }
            Event::UserEvent(UserEvent::WindowClose) => {
                background_webviews.clear();
                idle_normal_webview = None;
                foreground_webview = None;
                tabs_for_event.lock_safe().save();
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::GoBack) => {
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.go_back();
                }
            }
            Event::UserEvent(UserEvent::GoForward) => {
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.go_forward();
                }
            }
            Event::UserEvent(UserEvent::Reload) => {
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.reload();
                }
            }
            Event::UserEvent(UserEvent::EvalScript(script)) => {
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.evaluate_script(&script);
                }
            }
            Event::UserEvent(UserEvent::TrimWorkingSet) => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.set_memory_usage_level(MemoryUsageLevel::Low);
                    }
                    for bg_wv in background_webviews.values() {
                        let _ = bg_wv.set_memory_usage_level(MemoryUsageLevel::Low);
                    }
                    if let Some(ref idl) = idle_normal_webview {
                        let _ = idl.set_memory_usage_level(MemoryUsageLevel::Low);
                    }
                    let _ = force_empty_working_set();
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.set_memory_usage_level(MemoryUsageLevel::Normal);
                        let (ws, commit) = get_current_process_memory_mb();
                        let script = format!(
                            "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({{ host_mb: {:.1}, commit_mb: {:.1} }}, {}); window.__fynexo_toast__ && window.__fynexo_toast__('Working set trimmed ({:.1} MB)', {});",
                            ws, commit, bridge_tok_js, ws, bridge_tok_js
                        );
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(true),
                ..
            } => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.set_memory_usage_level(MemoryUsageLevel::Normal);
                    }
                }
                // Zero-polling instant hot-reload when user edits any *.json file externally
                let changed = state_for_event.lock_safe().reload_if_changed();
                if changed {
                    let payload = state_for_event.lock_safe().payload();
                    tabs_for_event
                        .lock_safe()
                        .set_incognito(payload.config.incognito);
                    #[cfg(target_os = "windows")]
                    if let Some(ref fg) = foreground_webview {
                        adblock::sync_windows_custom_filters(fg, &payload.adblock);
                    }
                    let active_url = tabs_for_event.lock_safe().active_url();
                    if let Ok(json) = serialize_runtime_for_url(&payload, &active_url) {
                        let script = format!(
                            "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                            json, bridge_tok_js
                        );
                        if let Some(ref fg) = foreground_webview {
                            let _ = fg.evaluate_script(&script);
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                background_webviews.clear();
                idle_normal_webview = None;
                foreground_webview = None;
                tabs_for_event.lock_safe().save();
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });

    #[allow(unreachable_code)]
    Ok(())
}

/// Only send full `RuntimePayload` (local paths, full history, disabled rules, adblock lists)
/// to internal Fynexo pages; external websites receive the stripped `HudBootPayload`.
fn serialize_runtime_for_url(
    payload: &crate::config::RuntimePayload,
    active_url: &str,
) -> Result<String, serde_json::Error> {
    if security::is_internal_origin(active_url) {
        serde_json::to_string(payload)
    } else {
        let slim = crate::config::HudBootPayload::from_runtime(payload, None);
        serde_json::to_string(&slim)
    }
}
