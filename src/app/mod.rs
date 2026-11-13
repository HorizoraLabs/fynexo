pub mod events;
pub mod ipc;
pub mod memory;
pub mod security;
pub mod state;
pub mod tab_pool;
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
use self::tab_pool::TabPool;
use self::url::to_webview_url;
use self::webview::{build_tab_webview, TabLaunch};

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
    let mut win_builder = WindowBuilder::new()
        .with_title("Fynexo")
        .with_background_color((9, 9, 11, 255));
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
    let (initial_active_id, active_logical) = {
        let tm = tab_manager.lock_safe();
        (tm.active_id, tm.active_url())
    };
    adblock::set_active_page_url(&active_logical);

    let initial_webview = build_tab_webview(
        &window,
        &mut web_context,
        TabLaunch {
            tab_id: initial_active_id,
            url: &active_logical,
            visible: true,
        },
        &app_state,
        &tab_manager,
        &event_proxy,
    )?;

    let state_for_event = Arc::clone(&app_state);
    let tabs_for_event = Arc::clone(&tab_manager);
    let event_proxy_for_run = event_proxy.clone();
    let reveal_scheduler = RevealScheduler::new(event_proxy.clone());
    let mut last_maximize_time = std::time::Instant::now();

    let mut active_tab_id = initial_active_id;
    let mut was_minimized = false;
    let mut pool = TabPool::new(initial_active_id, initial_webview);
    {
        let tm = tab_manager.lock_safe();
        let initial_muted = tm.tab_meta(initial_active_id).map(|(_, m)| m).unwrap_or(false);
        if let Some(wv) = pool.get_webview(initial_active_id) {
            set_webview_os_muted(wv, initial_muted);
        }
    }
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

                if let Some(fg) = pool.get_webview(active_tab_id) {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = fg.set_memory_usage_level(MemoryUsageLevel::Normal);
                    }
                    let _ = fg.load_url(&target);
                }
            }
            Event::UserEvent(UserEvent::SwitchTab { id, url }) => {
                perform_switch_tab(
                    id,
                    &url,
                    &mut active_tab_id,
                    &mut pool,
                    &window,
                    &mut web_context,
                    &state_for_event,
                    &tabs_for_event,
                    &reveal_scheduler,
                    &bridge_tok_js,
                );
            }
            Event::UserEvent(UserEvent::ResetWorkspace {
                active_id,
                active_url,
            }) => {
                pool.reset_all();
                active_tab_id = 0;
                perform_switch_tab(
                    active_id,
                    &active_url,
                    &mut active_tab_id,
                    &mut pool,
                    &window,
                    &mut web_context,
                    &state_for_event,
                    &tabs_for_event,
                    &reveal_scheduler,
                    &bridge_tok_js,
                );
            }
            Event::UserEvent(UserEvent::TabPaintReady(tab_id)) => {
                if pool.pending_reveal_id() == Some(tab_id) && active_tab_id == tab_id {
                    complete_pending_reveal(
                        tab_id,
                        &mut pool,
                        &window,
                        &state_for_event,
                        &tabs_for_event,
                        &bridge_tok_js,
                    );
                }
            }
            Event::UserEvent(UserEvent::RevealPendingTab { id, gen }) => {
                if pool.is_pending_match(id, gen) && active_tab_id == id {
                    complete_pending_reveal(
                        id,
                        &mut pool,
                        &window,
                        &state_for_event,
                        &tabs_for_event,
                        &bridge_tok_js,
                    );
                }
            }
            Event::UserEvent(UserEvent::ToggleBackgroundPlay { id, active }) => {
                if !active && id != active_tab_id {
                    pool.pause_media(id);
                    let changed = tab_manager.lock_safe().set_tab_playing_audio(id, false);
                    apply_pool_policy(&mut pool, &state_for_event, &tabs_for_event, active_tab_id);
                    if changed {
                        let _ = event_proxy_for_run.send_event(UserEvent::SyncTabs);
                    }
                }
            }
            Event::UserEvent(UserEvent::ToggleTabMute { id, muted }) => {
                let mute_script = format!(
                    "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({}, {});",
                    if muted { "true" } else { "false" },
                    bridge_tok_js
                );
                if let Some(wv) = pool.get_webview(id) {
                    set_webview_os_muted(wv, muted);
                    let _ = wv.evaluate_script(&mute_script);
                }
            }
            Event::UserEvent(UserEvent::BootSync(tab_id)) => {
                let (tab_state, tab_url, scroll_y, keep_bg, is_tab_muted) = {
                    let tm = tabs_for_event.lock_safe();
                    let (u, sy) = tm.url_and_scroll_for(tab_id);
                    let (bg, muted) = tm.tab_meta(tab_id).unwrap_or((false, false));
                    (tm.get_state(), u, sy, bg, muted)
                };
                let is_full = window.fullscreen().is_some();
                if let (Ok(rt_json), Ok(tabs_json)) = (
                    serialize_runtime_for_url(&state_for_event.lock_safe(), &tab_url),
                    serde_json::to_string(&tab_state),
                ) {
                    let full_str = if is_full { "true" } else { "false" };
                    let muted_str = if is_tab_muted { "true" } else { "false" };
                    let mut script = format!(
                        "window.__fynexo_boot_sync__ && window.__fynexo_boot_sync__({}, {}, {}); window.__fynexo_set_fullscreen__ && window.__fynexo_set_fullscreen__({}, {}); window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({}, {});",
                        rt_json, tabs_json, bridge_tok_js, full_str, bridge_tok_js, muted_str, bridge_tok_js
                    );
                    if scroll_y > 24.0 {
                        script.push_str(&format!(
                            " window.__fynexo_restore_scroll__ && window.__fynexo_restore_scroll__({}, {});",
                            scroll_y, bridge_tok_js
                        ));
                    }
                    if let Some(wv) = pool.get_webview(tab_id) {
                        set_webview_os_muted(wv, is_tab_muted);
                        script.push(' ');
                        script.push_str(&nav_state_js(wv, &bridge_tok_js));
                        let _ = wv.evaluate_script(&script);
                    }
                    if tab_id != active_tab_id {
                        pool.park_inactive(tab_id, keep_bg, &bridge_tok_js);
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncNavState(tab_id)) => {
                if let Some(wv) = pool.get_webview(tab_id) {
                    let script = nav_state_js(wv, &bridge_tok_js);
                    let _ = wv.evaluate_script(&script);
                }
            }
            Event::UserEvent(UserEvent::SyncConfig) => {
                let (active_url, vis_id) = (
                    tabs_for_event.lock_safe().active_url(),
                    pool.visible_id(),
                );
                if let Ok(json) =
                    serialize_runtime_for_url(&state_for_event.lock_safe(), &active_url)
                {
                    let script = format!(
                        "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                        json, bridge_tok_js
                    );
                    eval_on_active_and_visible(&pool, active_tab_id, vis_id, &script);
                }
                apply_pool_policy(&mut pool, &state_for_event, &tabs_for_event, active_tab_id);
            }
            Event::UserEvent(UserEvent::SyncTabs) => {
                let (current_tabs, tm_active_id, tm_active_url) = {
                    let tm = tabs_for_event.lock_safe();
                    (tm.tabs.clone(), tm.active_id, tm.active_url())
                };
                if tm_active_id != 0 && tm_active_id != active_tab_id {
                    perform_switch_tab(
                        tm_active_id,
                        &tm_active_url,
                        &mut active_tab_id,
                        &mut pool,
                        &window,
                        &mut web_context,
                        &state_for_event,
                        &tabs_for_event,
                        &reveal_scheduler,
                        &bridge_tok_js,
                    );
                }
                pool.prune_closed_tabs(&current_tabs);
                if pool.visible_id().is_none()
                    && pool.contains(active_tab_id)
                    && pool.pending_reveal_id() != Some(active_tab_id)
                {
                    let sz = window.inner_size();
                    pool.reveal_tab(active_tab_id, (sz.width, sz.height), &bridge_tok_js);
                }
                let live_ids = pool.live_ids();
                let tab_state = {
                    let mut tm = tabs_for_event.lock_safe();
                    tm.sync_suspended_flags(&live_ids);
                    tm.get_state()
                };
                if let Ok(json) = serde_json::to_string(&tab_state) {
                    let script = format!(
                        "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({}, {});",
                        json, bridge_tok_js
                    );
                    eval_on_active_and_visible(&pool, active_tab_id, pool.visible_id(), &script);
                }
            }
            Event::UserEvent(UserEvent::PageFinished { tab_id, url }) => {
                let (expected_url, scroll_y, is_tab_muted) = {
                    let tm = tabs_for_event.lock_safe();
                    let (u, sy) = tm.url_and_scroll_for(tab_id);
                    let muted = tm.tab_meta(tab_id).map(|(_, m)| m).unwrap_or(false);
                    (u, sy, muted)
                };
                if let Some(wv) = pool.get_webview(tab_id) {
                    set_webview_os_muted(wv, is_tab_muted);
                    let mut script = String::new();
                    if is_tab_muted {
                        script.push_str(&format!(
                            "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__(true, {}); ",
                            bridge_tok_js
                        ));
                    }
                    if tab_id == active_tab_id && pool.visible_id() == Some(tab_id) {
                        script.push_str(&format!(
                            "window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(false, {}); ",
                            bridge_tok_js
                        ));
                    }
                    if scroll_y > 24.0 && url == expected_url {
                        script.push_str(&format!(
                            "window.__fynexo_restore_scroll__ && window.__fynexo_restore_scroll__({}, {}); ",
                            scroll_y, bridge_tok_js
                        ));
                    }
                    script.push_str(&nav_state_js(wv, &bridge_tok_js));
                    let _ = wv.evaluate_script(&script);
                }
                if active_tab_id == tab_id {
                    if let Some(gen) = pool.pending_reveal_gen(tab_id) {
                        reveal_scheduler.schedule(
                            tab_id,
                            gen,
                            std::time::Duration::from_millis(120),
                        );
                    }
                }
            }
            Event::UserEvent(UserEvent::Toast(msg)) => {
                if let Ok(json_msg) = serde_json::to_string(&msg) {
                    let script = format!(
                        "window.__fynexo_toast__ && window.__fynexo_toast__({}, {});",
                        json_msg, bridge_tok_js
                    );
                    if let Some(vis) = pool.visible_webview() {
                        let _ = vis.evaluate_script(&script);
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
                eval_on_active_and_visible(&pool, active_tab_id, pool.visible_id(), &script);
            }
            Event::UserEvent(UserEvent::ApplyZoom(factor)) => {
                if let Some(wv) = pool.get_webview(active_tab_id) {
                    let _ = wv.zoom(factor);
                }
                if let Some(vid) = pool.visible_id() {
                    if vid != active_tab_id {
                        if let Some(vis) = pool.get_webview(vid) {
                            let _ = vis.zoom(factor);
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::ClearSiteData) => {
                if let Some(fg) = pool.active_or_visible_webview(active_tab_id) {
                    let _ = fg.clear_all_browsing_data();
                }
            }
            Event::UserEvent(UserEvent::SyncAdblockFilters) => {
                #[cfg(target_os = "windows")]
                {
                    let adb = state_for_event.lock_safe().adblock.clone();
                    if let Some(wv) = pool.get_webview(active_tab_id) {
                        adblock::sync_windows_custom_filters(wv, &adb);
                    }
                    if let Some(vid) = pool.visible_id() {
                        if vid != active_tab_id {
                            if let Some(vis) = pool.get_webview(vid) {
                                adblock::sync_windows_custom_filters(vis, &adb);
                            }
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::UpdateUserAgent(ua_opt)) => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(wv) = pool.get_webview(active_tab_id) {
                        adblock::apply_windows_user_agent(wv, ua_opt.as_deref());
                    }
                    if let Some(vid) = pool.visible_id() {
                        if vid != active_tab_id {
                            if let Some(vis) = pool.get_webview(vid) {
                                adblock::apply_windows_user_agent(vis, ua_opt.as_deref());
                            }
                        }
                    }
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
                pool.set_all_memory_low();
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
                pool.clear_all();
                tabs_for_event.lock_safe().save();
                crate::util::flush_pending_writes();
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::GoBack) => {
                navigate_webview_history(&pool, active_tab_id, false, &bridge_tok_js);
            }
            Event::UserEvent(UserEvent::GoForward) => {
                navigate_webview_history(&pool, active_tab_id, true, &bridge_tok_js);
            }
            Event::UserEvent(UserEvent::Reload) => {
                if let Some(fg) = pool.get_webview(active_tab_id) {
                    let _ = fg.reload();
                }
            }
            Event::UserEvent(UserEvent::OpenDevtools) => {
                #[cfg(all(not(target_os = "android"), debug_assertions))]
                if let Some(fg) = pool.get_webview(active_tab_id) {
                    fg.open_devtools();
                }
                #[cfg(not(all(not(target_os = "android"), debug_assertions)))]
                let _ = event_proxy_for_run.send_event(UserEvent::Toast(
                    "DevTools unavailable in this build".to_string(),
                ));
            }
            Event::UserEvent(UserEvent::ReloadTab(id)) => {
                if let Some(wv) = pool.get_webview(id) {
                    let _ = wv.reload();
                } else if let Some(target_url) = tabs_for_event.lock_safe().switch_tab(id) {
                    adblock::set_active_page_url(&target_url);
                    let _ = event_proxy_for_run.send_event(UserEvent::SwitchTab {
                        id,
                        url: target_url,
                    });
                    let _ = event_proxy_for_run.send_event(UserEvent::SyncTabs);
                }
            }
            Event::UserEvent(UserEvent::EvalScript(script)) => {
                if let Some(vis) = pool.visible_webview() {
                    let _ = vis.evaluate_script(&script);
                }
            }
            Event::UserEvent(UserEvent::TrimWorkingSet) => {
                #[cfg(target_os = "windows")]
                {
                    pool.set_all_memory_low();
                    let _ = force_empty_working_set();
                    if let Some(vis) = pool.visible_webview() {
                        let _ = vis.set_memory_usage_level(MemoryUsageLevel::Normal);
                        let (ws, commit) = get_current_process_memory_mb();
                        let script = format!(
                            "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({{ host_mb: {:.1}, commit_mb: {:.1} }}, {}); window.__fynexo_toast__ && window.__fynexo_toast__('Working set trimmed ({:.1} MB)', {});",
                            ws, commit, bridge_tok_js, ws, bridge_tok_js
                        );
                        let _ = vis.evaluate_script(&script);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width == 0 && size.height == 0 {
                    was_minimized = true;
                    #[cfg(target_os = "windows")]
                    {
                        pool.set_all_memory_low();
                        let _ = force_empty_working_set();
                    }
                } else {
                    pool.resize_all(size.width, size.height);
                    if was_minimized {
                        was_minimized = false;
                        #[cfg(target_os = "windows")]
                        if let Some(vis) = pool.visible_webview() {
                            let _ = vis.set_memory_usage_level(MemoryUsageLevel::Normal);
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Moved(_),
                ..
            } => {
                pool.notify_parent_moved();
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(true),
                ..
            } => {
                if let Some(vis) = pool.visible_webview() {
                    let _ = vis.focus();
                    #[cfg(target_os = "windows")]
                    {
                        let _ = vis.set_memory_usage_level(MemoryUsageLevel::Normal);
                    }
                }
                // Zero-polling instant hot-reload when user edits any *.json file externally
                let changed = state_for_event.lock_safe().reload_if_changed();
                if changed {
                    let active_url = tabs_for_event.lock_safe().active_url();
                    let (incognito, json_res) = {
                        let st = state_for_event.lock_safe();
                        #[cfg(target_os = "windows")]
                        if let Some(vis) = pool.visible_webview() {
                            adblock::sync_windows_custom_filters(vis, &st.adblock);
                        }
                        (st.config.incognito, serialize_runtime_for_url(&st, &active_url))
                    };
                    tabs_for_event.lock_safe().set_incognito(incognito);
                    if let Ok(json) = json_res {
                        let script = format!(
                            "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                            json, bridge_tok_js
                        );
                        eval_on_active_and_visible(&pool, active_tab_id, pool.visible_id(), &script);
                    }
                    apply_pool_policy(&mut pool, &state_for_event, &tabs_for_event, active_tab_id);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                pool.clear_all();
                tabs_for_event.lock_safe().save();
                crate::util::flush_pending_writes();
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });

    #[allow(unreachable_code)]
    Ok(())
}

type RevealEntry = (std::time::Instant, usize, u64);
type RevealQueue = Arc<(Mutex<Vec<RevealEntry>>, std::sync::Condvar)>;

struct RevealScheduler {
    proxy: EventLoopProxy<UserEvent>,
    queue: RevealQueue,
}

impl RevealScheduler {
    fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        let queue: RevealQueue = Arc::new((Mutex::new(Vec::new()), std::sync::Condvar::new()));
        let worker_q = Arc::clone(&queue);
        let worker_proxy = proxy.clone();
        let _ = std::thread::Builder::new()
            .name("fnx-reveal-timer".to_string())
            .spawn(move || {
                let (lock, cv) = &*worker_q;
                loop {
                    let due_events: Vec<(usize, u64)> = {
                        let mut guard = lock.lock_safe();
                        loop {
                            let now = std::time::Instant::now();
                            if let Some(earliest) = guard.iter().map(|(at, _, _)| *at).min() {
                                if earliest <= now {
                                    break;
                                }
                                let (g, _) = cv
                                    .wait_timeout(guard, earliest.saturating_duration_since(now))
                                    .unwrap_or_else(|p| p.into_inner());
                                guard = g;
                            } else {
                                guard = cv.wait(guard).unwrap_or_else(|p| p.into_inner());
                            }
                        }
                        let now = std::time::Instant::now();
                        let mut fired = Vec::new();
                        let mut i = 0;
                        while i < guard.len() {
                            if guard[i].0 <= now {
                                let (_, id, gen) = guard.swap_remove(i);
                                fired.push((id, gen));
                            } else {
                                i += 1;
                            }
                        }
                        fired
                    };
                    for (id, gen) in due_events {
                        let _ = worker_proxy.send_event(UserEvent::RevealPendingTab { id, gen });
                    }
                }
            });
        Self { proxy, queue }
    }

    fn proxy(&self) -> &EventLoopProxy<UserEvent> {
        &self.proxy
    }

    fn schedule(&self, id: usize, gen: u64, delay: std::time::Duration) {
        let (lock, cv) = &*self.queue;
        let mut guard = lock.lock_safe();
        guard.retain(|&(_, tid, _)| tid != id);
        guard.push((std::time::Instant::now() + delay, id, gen));
        cv.notify_one();
    }
}

#[allow(clippy::too_many_arguments)]
fn perform_switch_tab(
    id: usize,
    url: &str,
    active_tab_id: &mut usize,
    pool: &mut TabPool,
    window: &tao::window::Window,
    web_context: &mut WebContext,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    reveal_scheduler: &RevealScheduler,
    bridge_tok_js: &str,
) {
    let sz = window.inner_size();
    let win_size = (sz.width, sz.height);
    let old_id = *active_tab_id;
    if id == old_id && pool.contains(id) {
        pool.unpark_active(id, bridge_tok_js);
        if pool.visible_id() != Some(id) && pool.pending_reveal_id() != Some(id) {
            pool.reveal_tab(id, win_size, bridge_tok_js);
        }
        return;
    }

    if pool.pending_reveal_id().is_some_and(|pid| pid != id) {
        pool.clear_pending();
    }

    if old_id != 0 && old_id != id {
        let old_has_bg = tabs_for_bg(tab_manager, old_id);
        pool.park_inactive(old_id, old_has_bg, bridge_tok_js);
        if !old_has_bg {
            tab_manager.lock_safe().set_tab_playing_audio(old_id, false);
        }
    }

    *active_tab_id = id;
    pool.touch_lru(id);

    let (tab_state, is_tab_muted) = {
        let tm = tab_manager.lock_safe();
        let muted = tm.tab_meta(id).map(|(_, m)| m).unwrap_or(false);
        (tm.get_state(), muted)
    };

    if pool.contains(id) {
        pool.clear_pending();
        let wait_behind = pool.visible_id().is_some_and(|vid| vid != id);
        if wait_behind {
            if let Some(vis) = pool.visible_webview() {
                sync_tabs_and_mute(vis, &tab_state, is_tab_muted, bridge_tok_js);
            }
            pool.prepare_pending_behind(id, win_size);
        }
        pool.unpark_active(id, bridge_tok_js);
        if let Some(wv) = pool.get_webview(id) {
            let st = app_state.lock_safe();
            sync_foreground_env(
                wv,
                &st,
                window.fullscreen().is_some(),
                bridge_tok_js,
                Some(url),
            );
            sync_tabs_and_mute(wv, &tab_state, is_tab_muted, bridge_tok_js);
            let nav_js = nav_state_js(wv, bridge_tok_js);
            let _ = wv.evaluate_script(&nav_js);
            if wait_behind {
                let frame_script = format!(
                    "window.__fynexo_request_reveal_frame__ && window.__fynexo_request_reveal_frame__({});",
                    bridge_tok_js
                );
                let _ = wv.evaluate_script(&frame_script);
            }
        }
        if wait_behind {
            let gen = pool.set_pending_reveal(id);
            reveal_scheduler.schedule(id, gen, std::time::Duration::from_millis(90));
        } else {
            pool.reveal_tab(id, win_size, bridge_tok_js);
        }
    } else {
        let has_visible_backing = pool.visible_id().is_some();
        let target = to_webview_url(url);

        if let Some(tv) = pool.take_idle_for(id) {
            #[cfg(target_os = "windows")]
            {
                let _ = tv.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
            }
            {
                let st = app_state.lock_safe();
                sync_foreground_env(
                    &tv.webview,
                    &st,
                    window.fullscreen().is_some(),
                    bridge_tok_js,
                    None,
                );
            }
            let _ = tv.webview.load_url(&target);
            pool.insert_live(id, tv);
        } else {
            match build_tab_webview(
                window,
                web_context,
                TabLaunch {
                    tab_id: id,
                    url: &target,
                    visible: !has_visible_backing,
                },
                app_state,
                tab_manager,
                reveal_scheduler.proxy(),
            ) {
                Ok(new_tv) => {
                    pool.insert_live(id, new_tv);
                }
                Err(e) => {
                    eprintln!("[Fynexo] Failed to create tab webview: {:?}", e);
                }
            }
        }

        if !has_visible_backing {
            pool.reveal_tab(id, win_size, bridge_tok_js);
        } else {
            if let Some(vis) = pool.visible_webview() {
                sync_tabs_and_mute(vis, &tab_state, is_tab_muted, bridge_tok_js);
                let prog_script = format!(
                    "window.__fynexo_trigger_progress__ && window.__fynexo_trigger_progress__({});",
                    bridge_tok_js
                );
                let _ = vis.evaluate_script(&prog_script);
            }
            pool.prepare_pending_behind(id, win_size);
            let gen = pool.set_pending_reveal(id);
            reveal_scheduler.schedule(id, gen, std::time::Duration::from_millis(4000));
        }
    }

    apply_pool_policy(pool, app_state, tab_manager, *active_tab_id);
}

fn apply_pool_policy(
    pool: &mut TabPool,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    active_id: usize,
) {
    let limits = app_state.lock_safe().config.tab_cache_limits();
    let tabs = tab_manager.lock_safe().tabs.clone();
    pool.evict_excess_normal_tabs(&tabs, active_id, limits.max_cached_normal);
    pool.apply_memory_tiers(active_id, limits.hot_normal_count);
    let live_ids = pool.live_ids();
    tab_manager.lock_safe().sync_suspended_flags(&live_ids);
}

fn complete_pending_reveal(
    tab_id: usize,
    pool: &mut TabPool,
    window: &tao::window::Window,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    bridge_tok_js: &str,
) {
    let (tab_state, is_tab_muted) = {
        let tm = tab_manager.lock_safe();
        let muted = tm.tab_meta(tab_id).map(|(_, m)| m).unwrap_or(false);
        (tm.get_state(), muted)
    };
    pool.unpark_active(tab_id, bridge_tok_js);
    if let Some(wv) = pool.get_webview(tab_id) {
        sync_tabs_and_mute(wv, &tab_state, is_tab_muted, bridge_tok_js);
    }
    let sz = window.inner_size();
    pool.reveal_tab(tab_id, (sz.width, sz.height), bridge_tok_js);
    apply_pool_policy(pool, app_state, tab_manager, tab_id);
}

fn tabs_for_bg(tab_manager: &Arc<Mutex<TabManager>>, id: usize) -> bool {
    tab_manager
        .lock_safe()
        .tab_meta(id)
        .map(|(bg, _)| bg)
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn set_webview_os_muted(webview: &wry::WebView, muted: bool) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
    use windows::core::Interface;
    unsafe {
        if let Ok(core) = webview.controller().CoreWebView2() {
            if let Ok(wv8) = core.cast::<ICoreWebView2_8>() {
                let _ = wv8.SetIsMuted(muted);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn set_webview_os_muted(_webview: &wry::WebView, _muted: bool) {}

fn sync_tabs_and_mute(
    webview: &wry::WebView,
    tab_state: &crate::tabs::TabState,
    is_tab_muted: bool,
    bridge_tok_js: &str,
) {
    set_webview_os_muted(webview, is_tab_muted);
    let mut script = String::new();
    if let Ok(tabs_json) = serde_json::to_string(tab_state) {
        script.push_str(&format!(
            "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({}, {}); ",
            tabs_json, bridge_tok_js
        ));
    }
    script.push_str(&format!(
        "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({}, {});",
        if is_tab_muted { "true" } else { "false" },
        bridge_tok_js
    ));
    let _ = webview.evaluate_script(&script);
}

fn eval_on_active_and_visible(
    pool: &TabPool,
    active_id: usize,
    visible_id: Option<usize>,
    script: &str,
) {
    if let Some(wv) = pool.get_webview(active_id) {
        let _ = wv.evaluate_script(script);
    }
    if let Some(vid) = visible_id {
        if vid != active_id {
            if let Some(vis) = pool.get_webview(vid) {
                let _ = vis.evaluate_script(script);
            }
        }
    }
}

/// Only send full `RuntimePayload` (local paths, full history, disabled rules, adblock lists)
/// to internal Fynexo pages; external websites receive the stripped `HudBootPayload`.
fn serialize_runtime_for_url(
    state: &AppState,
    active_url: &str,
) -> Result<String, serde_json::Error> {
    if security::is_internal_origin(active_url) {
        serde_json::to_string(&state.payload())
    } else {
        state.hud_json()
    }
}

/// Synchronizes zoom, fullscreen state, and (on Windows) custom adblock filters & User-Agent
/// onto a WebView entering or returning to the foreground.
fn sync_foreground_env(
    webview: &wry::WebView,
    state: &AppState,
    is_fullscreen: bool,
    bridge_tok_js: &str,
    push_runtime_for_url: Option<&str>,
) {
    let zoom_factor = state.config.default_zoom.clamp(0.5, 2.5);
    let _ = webview.zoom(zoom_factor);

    #[cfg(target_os = "windows")]
    {
        adblock::sync_windows_custom_filters(webview, &state.adblock);
        let ua = state.config.resolved_user_agent();
        adblock::apply_windows_user_agent(webview, ua.as_deref());
    }

    let mut scripts = String::new();
    if let Some(url) = push_runtime_for_url {
        if let Ok(rt_json) = serialize_runtime_for_url(state, url) {
            scripts.push_str(&format!(
                "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {}); ",
                rt_json, bridge_tok_js
            ));
        }
    }
    scripts.push_str(&format!(
        "window.__fynexo_set_fullscreen__ && window.__fynexo_set_fullscreen__({}, {});",
        if is_fullscreen { "true" } else { "false" },
        bridge_tok_js
    ));
    let _ = webview.evaluate_script(&scripts);
}

fn nav_state_js(webview: &wry::WebView, bridge_tok_js: &str) -> String {
    let can_back = webview.can_go_back().unwrap_or(false);
    let can_fwd = webview.can_go_forward().unwrap_or(false);
    format!(
        "window.__fynexo_set_nav_state__ && window.__fynexo_set_nav_state__({}, {}, {});",
        if can_back { "true" } else { "false" },
        if can_fwd { "true" } else { "false" },
        bridge_tok_js
    )
}

fn navigate_webview_history(
    pool: &TabPool,
    active_tab_id: usize,
    forward: bool,
    bridge_tok_js: &str,
) {
    let Some(fg) = pool.get_webview(active_tab_id) else {
        return;
    };
    let can_nav = if forward {
        fg.can_go_forward().unwrap_or(false)
    } else {
        fg.can_go_back().unwrap_or(false)
    };
    let ok = can_nav
        && if forward {
            fg.go_forward().is_ok()
        } else {
            fg.go_back().is_ok()
        };
    if !ok {
        let script = format!(
            "window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {}); {}",
            bridge_tok_js,
            nav_state_js(fg, bridge_tok_js)
        );
        let _ = fg.evaluate_script(&script);
    }
}
