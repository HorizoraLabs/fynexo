pub mod events;
pub mod ipc;
pub mod memory;
pub mod scheduler;
pub mod security;
pub mod state;
pub mod tab_pool;
pub mod url;
pub mod webview;

pub use events::UserEvent;
pub use state::AppState;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
};
use wry::WebContext;

#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewExtWindows};

use crate::adblock;
use crate::config::data_dir;
use crate::tabs::TabManager;
use crate::util::LockExt;

#[cfg(target_os = "windows")]
use self::memory::{force_empty_working_set, get_process_tree_memory_with_roots};
#[cfg(not(target_os = "windows"))]
use self::memory::get_process_tree_memory_with_roots;
use self::scheduler::RevealScheduler;
use self::tab_pool::TabPool;
use self::url::to_webview_url;
use self::webview::{build_tab_webview, TabLaunch};

struct BrowserWindow {
    win_id: usize,
    window: tao::window::Window,
    incognito_window: bool,
    tab_manager: Arc<Mutex<TabManager>>,
    pool: TabPool,
    active_tab_id: usize,
    was_minimized: bool,
    last_maximize_time: std::time::Instant,
}

#[allow(clippy::too_many_arguments)]
fn spawn_browser_window(
    win_id: usize,
    restore_primary: bool,
    start_url: Option<String>,
    incognito_window: bool,
    cascade_index: usize,
    event_loop_target: &EventLoopWindowTarget<UserEvent>,
    web_context: &mut WebContext,
    app_state: &Arc<Mutex<AppState>>,
    event_proxy: &EventLoopProxy<UserEvent>,
) -> Result<BrowserWindow, Box<dyn Error>> {
    let (homepage, restore_session, global_incog) = {
        let st = app_state.lock_safe();
        (
            st.config.homepage.clone(),
            st.config.restore_session,
            st.config.incognito,
        )
    };
    let effective_incog = global_incog || incognito_window;
    let init_target = start_url.unwrap_or_else(|| homepage.clone());

    let mut tm = if restore_primary {
        TabManager::load_or_new(&init_target, restore_session, effective_incog)
    } else {
        TabManager::new(&init_target, effective_incog)
    };
    tm.set_persist_session(restore_primary && !incognito_window);
    let tab_manager = Arc::new(Mutex::new(tm));

    let title = if incognito_window {
        "Fynexo · Private"
    } else {
        "Fynexo"
    };

    #[allow(unused_mut)]
    let mut win_builder = WindowBuilder::new()
        .with_title(title)
        .with_background_color((9, 9, 11, 255));
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        win_builder = win_builder
            .with_decorations(false)
            .with_resizable(true)
            .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 840.0));
        if cascade_index == 0 {
            win_builder = win_builder.with_maximized(true);
        } else {
            let offset = 40.0 + ((cascade_index % 6) as f64) * 28.0;
            win_builder = win_builder.with_position(tao::dpi::LogicalPosition::new(offset, offset));
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = cascade_index;

    let window = win_builder.build(event_loop_target)?;
    let (initial_active_id, active_logical, initial_muted) = {
        let tm_guard = tab_manager.lock_safe();
        let aid = tm_guard.active_id;
        let muted = tm_guard.tab_meta(aid).map(|(_, m)| m).unwrap_or(false);
        (aid, tm_guard.active_url(), muted)
    };
    adblock::set_active_page_url(&active_logical);

    let initial_webview = build_tab_webview(
        &window,
        web_context,
        TabLaunch {
            win_id,
            tab_id: initial_active_id,
            url: &active_logical,
            visible: true,
        },
        app_state,
        &tab_manager,
        event_proxy,
    )?;

    let pool = TabPool::new(initial_active_id, initial_webview);
    if let Some(wv) = pool.get_webview(initial_active_id) {
        set_webview_os_muted(wv, initial_muted);
    }

    Ok(BrowserWindow {
        win_id,
        window,
        incognito_window,
        tab_manager,
        pool,
        active_tab_id: initial_active_id,
        was_minimized: false,
        last_maximize_time: std::time::Instant::now(),
    })
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let app_state = Arc::new(Mutex::new(AppState::load()));
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let event_proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();
    let mut web_context = WebContext::new(Some(data_dir().join("webview_profile")));

    let primary_win = spawn_browser_window(
        1,
        true,
        None,
        false,
        0,
        &event_loop,
        &mut web_context,
        &app_state,
        &event_proxy,
    )?;

    let mut windows: HashMap<usize, BrowserWindow> = HashMap::new();
    let mut win_by_tao_id: HashMap<WindowId, usize> = HashMap::new();
    win_by_tao_id.insert(primary_win.window.id(), primary_win.win_id);
    windows.insert(primary_win.win_id, primary_win);

    let mut next_win_id: usize = 2;
    let mut focused_win_id: usize = 1;

    let state_for_event = Arc::clone(&app_state);
    let event_proxy_for_run = event_proxy.clone();
    let reveal_scheduler = RevealScheduler::new(event_proxy.clone());
    let bridge_tok_js =
        serde_json::to_string(security::bridge_token()).unwrap_or_else(|_| "\"\"".to_string());

    event_loop.run(move |event, event_loop_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => (),
            Event::UserEvent(UserEvent::NewWindow { url, incognito }) => {
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                {
                    let is_multi = state_for_event.lock_safe().config.is_multi_window();
                    if is_multi {
                        let new_wid = next_win_id;
                        next_win_id = next_win_id.saturating_add(1);
                        let cascade = windows.len();
                        match spawn_browser_window(
                            new_wid,
                            false,
                            url,
                            incognito,
                            cascade,
                            event_loop_target,
                            &mut web_context,
                            &state_for_event,
                            &event_proxy_for_run,
                        ) {
                            Ok(bw) => {
                                win_by_tao_id.insert(bw.window.id(), new_wid);
                                bw.window.set_focus();
                                windows.insert(new_wid, bw);
                                focused_win_id = new_wid;
                            }
                            Err(e) => {
                                eprintln!("[Fynexo] Failed to spawn window: {:?}", e);
                            }
                        }
                    }
                }
                #[cfg(any(target_os = "android", target_os = "ios"))]
                let _ = (url, incognito, event_loop_target, &mut next_win_id);
            }
            Event::UserEvent(UserEvent::Navigate { win_id, url }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    let target = to_webview_url(&url);
                    #[cfg(debug_assertions)]
                    println!("[Navigation Request w{}]: {} -> {}", win_id, url, target);
                    if let Some(fg) = bw.pool.get_webview(bw.active_tab_id) {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = fg.set_memory_usage_level(MemoryUsageLevel::Normal);
                        }
                        let _ = fg.load_url(&target);
                    }
                }
            }
            Event::UserEvent(UserEvent::SwitchTab { win_id, id, url }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    perform_switch_tab(
                        bw,
                        id,
                        &url,
                        &mut web_context,
                        &state_for_event,
                        &reveal_scheduler,
                        &bridge_tok_js,
                    );
                }
            }
            Event::UserEvent(UserEvent::ResetWorkspace {
                win_id,
                active_id,
                active_url,
            }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    bw.pool.reset_all();
                    bw.active_tab_id = 0;
                    perform_switch_tab(
                        bw,
                        active_id,
                        &active_url,
                        &mut web_context,
                        &state_for_event,
                        &reveal_scheduler,
                        &bridge_tok_js,
                    );
                }
            }
            Event::UserEvent(UserEvent::TabPaintReady { win_id, tab_id }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    if bw.pool.pending_reveal_id() == Some(tab_id) && bw.active_tab_id == tab_id {
                        complete_pending_reveal(bw, tab_id, &state_for_event, &bridge_tok_js);
                    }
                }
            }
            Event::UserEvent(UserEvent::RevealPendingTab { win_id, id, gen }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    if bw.pool.is_pending_match(id, gen) && bw.active_tab_id == id {
                        complete_pending_reveal(bw, id, &state_for_event, &bridge_tok_js);
                    }
                }
            }
            Event::UserEvent(UserEvent::ToggleBackgroundPlay { win_id, id, active }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    if !active && id != bw.active_tab_id {
                        bw.pool.pause_media(id);
                        let changed = bw.tab_manager.lock_safe().set_tab_playing_audio(id, false);
                        apply_pool_policy(bw, &state_for_event);
                        if changed {
                            let _ = event_proxy_for_run.send_event(UserEvent::SyncTabs(win_id));
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::ToggleTabMute { win_id, id, muted }) => {
                if let Some(bw) = windows.get(&win_id) {
                    let mute_script = format!(
                        "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({}, {});",
                        if muted { "true" } else { "false" },
                        bridge_tok_js
                    );
                    if let Some(wv) = bw.pool.get_webview(id) {
                        set_webview_os_muted(wv, muted);
                        let _ = wv.evaluate_script(&mute_script);
                    }
                }
            }
            Event::UserEvent(UserEvent::BootSync { win_id, tab_id }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    let (tab_state, tab_url, scroll_y, keep_bg, is_tab_muted) = {
                        let tm = bw.tab_manager.lock_safe();
                        let (u, sy) = tm.url_and_scroll_for(tab_id);
                        let (bg, muted) = tm.tab_meta(tab_id).unwrap_or((false, false));
                        (tm.get_state(), u, sy, bg, muted)
                    };
                    let is_full = bw.window.fullscreen().is_some();
                    if let (Ok(rt_json), Ok(tabs_json)) = (
                        serialize_runtime_for_win(
                            &state_for_event.lock_safe(),
                            &tab_url,
                            bw.incognito_window,
                        ),
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
                        if let Some(wv) = bw.pool.get_webview(tab_id) {
                            set_webview_os_muted(wv, is_tab_muted);
                            script.push(' ');
                            script.push_str(&nav_state_js(wv, &bridge_tok_js));
                            let _ = wv.evaluate_script(&script);
                        }
                        if tab_id != bw.active_tab_id {
                            bw.pool.park_inactive(tab_id, keep_bg, &bridge_tok_js);
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncNavState { win_id, tab_id }) => {
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(wv) = bw.pool.get_webview(tab_id) {
                        let script = nav_state_js(wv, &bridge_tok_js);
                        let _ = wv.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncConfig) => {
                for bw in windows.values_mut() {
                    let (active_url, vis_id) =
                        (bw.tab_manager.lock_safe().active_url(), bw.pool.visible_id());
                    if let Ok(json) = serialize_runtime_for_win(
                        &state_for_event.lock_safe(),
                        &active_url,
                        bw.incognito_window,
                    ) {
                        let script = format!(
                            "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                            json, bridge_tok_js
                        );
                        eval_on_active_and_visible(&bw.pool, bw.active_tab_id, vis_id, &script);
                    }
                    apply_pool_policy(bw, &state_for_event);
                }
            }
            Event::UserEvent(UserEvent::SyncTabs(win_id)) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    let (current_tabs, tm_active_id, tm_active_url) = {
                        let tm = bw.tab_manager.lock_safe();
                        (tm.tabs.clone(), tm.active_id, tm.active_url())
                    };
                    if tm_active_id != 0 && tm_active_id != bw.active_tab_id {
                        perform_switch_tab(
                            bw,
                            tm_active_id,
                            &tm_active_url,
                            &mut web_context,
                            &state_for_event,
                            &reveal_scheduler,
                            &bridge_tok_js,
                        );
                    }
                    bw.pool.prune_closed_tabs(&current_tabs);
                    if bw.pool.visible_id().is_none()
                        && bw.pool.contains(bw.active_tab_id)
                        && bw.pool.pending_reveal_id() != Some(bw.active_tab_id)
                    {
                        let sz = bw.window.inner_size();
                        bw.pool
                            .reveal_tab(bw.active_tab_id, (sz.width, sz.height), &bridge_tok_js);
                    }
                    let live_ids = bw.pool.live_ids();
                    let tab_state = {
                        let mut tm = bw.tab_manager.lock_safe();
                        tm.sync_suspended_flags(&live_ids);
                        tm.get_state()
                    };
                    if let Ok(json) = serde_json::to_string(&tab_state) {
                        let script = format!(
                            "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({}, {});",
                            json, bridge_tok_js
                        );
                        eval_on_active_and_visible(
                            &bw.pool,
                            bw.active_tab_id,
                            bw.pool.visible_id(),
                            &script,
                        );
                    }
                }
            }
            Event::UserEvent(UserEvent::PageFinished {
                win_id,
                tab_id,
                url,
            }) => {
                if let Some(bw) = windows.get(&win_id) {
                    let (expected_url, scroll_y, is_tab_muted) = {
                        let tm = bw.tab_manager.lock_safe();
                        let (u, sy) = tm.url_and_scroll_for(tab_id);
                        let muted = tm.tab_meta(tab_id).map(|(_, m)| m).unwrap_or(false);
                        (u, sy, muted)
                    };
                    if let Some(wv) = bw.pool.get_webview(tab_id) {
                        set_webview_os_muted(wv, is_tab_muted);
                        let mut script = String::new();
                        if is_tab_muted {
                            script.push_str(&format!(
                                "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__(true, {}); ",
                                bridge_tok_js
                            ));
                        }
                        if tab_id == bw.active_tab_id && bw.pool.visible_id() == Some(tab_id) {
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
                    if bw.active_tab_id == tab_id {
                        if let Some(gen) = bw.pool.pending_reveal_gen(tab_id) {
                            reveal_scheduler.schedule(
                                win_id,
                                tab_id,
                                gen,
                                std::time::Duration::from_millis(120),
                            );
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::DownloadUpdate { name, status }) => {
                let (downloads_json, toast_msg) = {
                    let mut st = state_for_event.lock_safe();
                    st.record_download(name.clone(), status.clone());
                    let zh = st.config.language.eq_ignore_ascii_case("zh");
                    let msg = match status.as_str() {
                        "downloading" => {
                            if zh {
                                format!("正在下载：{}", name)
                            } else {
                                format!("Downloading {}", name)
                            }
                        }
                        "completed" => {
                            if zh {
                                format!("已保存到下载目录：{}", name)
                            } else {
                                format!("Saved {} to Downloads", name)
                            }
                        }
                        _ => {
                            if zh {
                                format!("下载已取消或失败：{}", name)
                            } else {
                                format!("Download failed: {}", name)
                            }
                        }
                    };
                    (serde_json::to_string(&st.recent_downloads), msg)
                };
                if let Ok(dl_json) = &downloads_json {
                    let dl_script = format!(
                        "window.__fynexo_set_downloads__ && window.__fynexo_set_downloads__({}, {});",
                        dl_json, bridge_tok_js
                    );
                    for bw in windows.values() {
                        if let Some(vis) = bw.pool.visible_webview() {
                            let _ = vis.evaluate_script(&dl_script);
                        }
                    }
                }
                if let Ok(json_msg) = serde_json::to_string(&toast_msg) {
                    let t_script = format!(
                        "window.__fynexo_toast__ && window.__fynexo_toast__({}, {});",
                        json_msg, bridge_tok_js
                    );
                    if let Some(vis) = target_visible_webview(&windows, focused_win_id) {
                        let _ = vis.evaluate_script(&t_script);
                    }
                }
            }
            Event::UserEvent(UserEvent::Toast(msg)) => {
                if let Ok(json_msg) = serde_json::to_string(&msg) {
                    let script = format!(
                        "window.__fynexo_toast__ && window.__fynexo_toast__({}, {});",
                        json_msg, bridge_tok_js
                    );
                    if let Some(vis) = target_visible_webview(&windows, focused_win_id) {
                        let _ = vis.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::ToggleFullscreen { win_id, explicit }) => {
                if let Some(bw) = windows.get(&win_id) {
                    let is_currently_full = bw.window.fullscreen().is_some();
                    let next_full = explicit.unwrap_or(!is_currently_full);
                    if next_full {
                        bw.window
                            .set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
                    } else {
                        bw.window.set_fullscreen(None);
                    }
                    let script = format!(
                        "window.__fynexo_set_fullscreen__ && window.__fynexo_set_fullscreen__({}, {});",
                        if next_full { "true" } else { "false" },
                        bridge_tok_js
                    );
                    eval_on_active_and_visible(
                        &bw.pool,
                        bw.active_tab_id,
                        bw.pool.visible_id(),
                        &script,
                    );
                }
            }
            Event::UserEvent(UserEvent::ApplyZoom(factor)) => {
                for bw in windows.values() {
                    if let Some(wv) = bw.pool.get_webview(bw.active_tab_id) {
                        let _ = wv.zoom(factor);
                    }
                    if let Some(vid) = bw.pool.visible_id() {
                        if vid != bw.active_tab_id {
                            if let Some(vis) = bw.pool.get_webview(vid) {
                                let _ = vis.zoom(factor);
                            }
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::ClearSiteData(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(fg) = bw.pool.active_or_visible_webview(bw.active_tab_id) {
                        let _ = fg.clear_all_browsing_data();
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncAdblockFilters) => {
                #[cfg(target_os = "windows")]
                {
                    let adb = state_for_event.lock_safe().adblock.clone();
                    for bw in windows.values() {
                        if let Some(wv) = bw.pool.get_webview(bw.active_tab_id) {
                            adblock::sync_windows_custom_filters(wv, &adb);
                        }
                        if let Some(vid) = bw.pool.visible_id() {
                            if vid != bw.active_tab_id {
                                if let Some(vis) = bw.pool.get_webview(vid) {
                                    adblock::sync_windows_custom_filters(vis, &adb);
                                }
                            }
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncAutofill(enabled)) => {
                #[cfg(target_os = "windows")]
                for bw in windows.values() {
                    let effective = enabled && !bw.incognito_window;
                    for wv in bw.pool.all_webviews() {
                        adblock::apply_windows_autofill(wv, effective);
                    }
                }
                #[cfg(not(target_os = "windows"))]
                let _ = enabled;
            }
            Event::UserEvent(UserEvent::RendererCrashed {
                win_id,
                tab_id,
                unresponsive,
            }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    bw.tab_manager.lock_safe().save();
                    crate::util::flush_pending_writes();
                    let zh = state_for_event
                        .lock_safe()
                        .config
                        .language
                        .eq_ignore_ascii_case("zh");
                    if unresponsive {
                        let msg = if zh {
                            "当前页面响应缓慢，正在恢复..."
                        } else {
                            "Page became unresponsive, recovering..."
                        };
                        let _ = event_proxy_for_run.send_event(UserEvent::Toast(msg.to_string()));
                    } else {
                        bw.pool.drop_crashed(tab_id);
                        let msg = if zh {
                            "标签页渲染进程意外退出，已自动为你恢复"
                        } else {
                            "Tab renderer crashed — recovered automatically"
                        };
                        let _ = event_proxy_for_run.send_event(UserEvent::Toast(msg.to_string()));
                        if bw.active_tab_id == tab_id {
                            let url = bw.tab_manager.lock_safe().url_and_scroll_for(tab_id).0;
                            perform_switch_tab(
                                bw,
                                tab_id,
                                &url,
                                &mut web_context,
                                &state_for_event,
                                &reveal_scheduler,
                                &bridge_tok_js,
                            );
                        }
                        let _ = event_proxy_for_run.send_event(UserEvent::SyncTabs(win_id));
                    }
                }
            }
            Event::UserEvent(UserEvent::UpdateUserAgent(ua_opt)) => {
                #[cfg(target_os = "windows")]
                {
                    for bw in windows.values() {
                        if let Some(wv) = bw.pool.get_webview(bw.active_tab_id) {
                            adblock::apply_windows_user_agent(wv, ua_opt.as_deref());
                        }
                        if let Some(vid) = bw.pool.visible_id() {
                            if vid != bw.active_tab_id {
                                if let Some(vis) = bw.pool.get_webview(vid) {
                                    adblock::apply_windows_user_agent(vis, ua_opt.as_deref());
                                }
                            }
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                let _ = ua_opt;
            }
            Event::UserEvent(UserEvent::WindowDrag(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    let _ = bw.window.drag_window();
                }
            }
            Event::UserEvent(UserEvent::WindowMinimize(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    bw.window.set_minimized(true);
                    #[cfg(target_os = "windows")]
                    {
                        let (eff, trim) = {
                            let st = state_for_event.lock_safe();
                            (st.config.efficiency_mode, st.config.auto_trim_ram)
                        };
                        if eff {
                            bw.pool.set_all_memory_low();
                        }
                        if trim {
                            let _ = force_empty_working_set();
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::WindowMaximize(win_id)) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    let now = std::time::Instant::now();
                    if now.duration_since(bw.last_maximize_time)
                        >= std::time::Duration::from_millis(300)
                    {
                        bw.last_maximize_time = now;
                        let is_max = bw.window.is_maximized();
                        bw.window.set_maximized(!is_max);
                    }
                }
            }
            Event::UserEvent(UserEvent::WindowClose(win_id)) => {
                close_browser_window(
                    win_id,
                    &mut windows,
                    &mut win_by_tao_id,
                    &mut focused_win_id,
                    control_flow,
                );
            }
            Event::UserEvent(UserEvent::GoBack(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    navigate_webview_history(&bw.pool, bw.active_tab_id, false, &bridge_tok_js);
                }
            }
            Event::UserEvent(UserEvent::GoForward(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    navigate_webview_history(&bw.pool, bw.active_tab_id, true, &bridge_tok_js);
                }
            }
            Event::UserEvent(UserEvent::Reload(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(fg) = bw.pool.get_webview(bw.active_tab_id) {
                        let _ = fg.reload();
                    }
                }
            }
            Event::UserEvent(UserEvent::OpenDevtools(win_id)) => {
                #[cfg(not(target_os = "android"))]
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(fg) = bw.pool.get_webview(bw.active_tab_id) {
                        fg.open_devtools();
                    }
                }
                #[cfg(target_os = "android")]
                {
                    let _ = win_id;
                    let _ = event_proxy_for_run.send_event(UserEvent::Toast(
                        "Native DevTools only supported on Desktop".to_string(),
                    ));
                }
            }
            Event::UserEvent(UserEvent::ReloadTab { win_id, id }) => {
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(wv) = bw.pool.get_webview(id) {
                        let _ = wv.reload();
                    } else if let Some(target_url) = bw.tab_manager.lock_safe().switch_tab(id) {
                        adblock::set_active_page_url(&target_url);
                        let _ = event_proxy_for_run.send_event(UserEvent::SwitchTab {
                            win_id,
                            id,
                            url: target_url,
                        });
                        let _ = event_proxy_for_run.send_event(UserEvent::SyncTabs(win_id));
                    }
                }
            }
            Event::UserEvent(UserEvent::EvalScript { win_id, script }) => {
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(vis) = bw.pool.visible_webview() {
                        let _ = vis.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::RequestResourceUsage(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(vis) = bw.pool.visible_webview() {
                        let stats_json = build_resource_stats_json(&windows);
                        let script = format!(
                            "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({}, {});",
                            stats_json, bridge_tok_js
                        );
                        let _ = vis.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::TrimWorkingSet(win_id)) => {
                #[cfg(target_os = "windows")]
                {
                    let eff = state_for_event.lock_safe().config.efficiency_mode;
                    if eff {
                        for bw in windows.values() {
                            bw.pool.set_all_memory_low();
                        }
                    }
                    let _ = force_empty_working_set();
                    if let Some(bw) = windows.get(&win_id) {
                        if let Some(vis) = bw.pool.visible_webview() {
                            let _ = vis.set_memory_usage_level(MemoryUsageLevel::Normal);
                        }
                    }
                    let stats_json = build_resource_stats_json(&windows);
                    let total_mb = stats_json["total_mb"].as_f64().unwrap_or(0.0);
                    if let Some(bw) = windows.get(&win_id) {
                        if let Some(vis) = bw.pool.visible_webview() {
                            let script = format!(
                                "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({}, {}); window.__fynexo_toast__ && window.__fynexo_toast__('Working set trimmed ({:.0} MB total)', {});",
                                stats_json, bridge_tok_js, total_mb, bridge_tok_js
                            );
                            let _ = vis.evaluate_script(&script);
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                let _ = win_id;
            }
            Event::UserEvent(UserEvent::SweepIdleTabs) => {
                let (sleep_mins, auto_trim) = {
                    let st = state_for_event.lock_safe();
                    (st.config.tab_sleep_minutes, st.config.auto_trim_ram)
                };
                if sleep_mins > 0 {
                    let timeout = std::time::Duration::from_secs((sleep_mins as u64) * 60);
                    let mut any_slept = false;
                    let win_ids: Vec<usize> = windows.keys().copied().collect();
                    for wid in win_ids {
                        if let Some(bw) = windows.get_mut(&wid) {
                            let tabs = bw.tab_manager.lock_safe().tabs.clone();
                            let slept = bw.pool.sleep_idle_tabs(&tabs, bw.active_tab_id, timeout);
                            if slept > 0 {
                                any_slept = true;
                                let live_ids = bw.pool.live_ids();
                                bw.tab_manager.lock_safe().sync_suspended_flags(&live_ids);
                                let _ = event_proxy_for_run.send_event(UserEvent::SyncTabs(wid));
                            }
                        }
                    }
                    #[cfg(target_os = "windows")]
                    if any_slept && auto_trim {
                        let _ = force_empty_working_set();
                    }
                    #[cfg(not(target_os = "windows"))]
                    let _ = (any_slept, auto_trim);
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
                ..
            } => {
                if let Some(&wid) = win_by_tao_id.get(&window_id) {
                    if let Some(bw) = windows.get_mut(&wid) {
                        if size.width == 0 && size.height == 0 {
                            bw.was_minimized = true;
                            #[cfg(target_os = "windows")]
                            {
                                let (eff, trim) = {
                                    let st = state_for_event.lock_safe();
                                    (st.config.efficiency_mode, st.config.auto_trim_ram)
                                };
                                if eff {
                                    bw.pool.set_all_memory_low();
                                }
                                if trim {
                                    let _ = force_empty_working_set();
                                }
                            }
                        } else {
                            bw.pool.resize_all(size.width, size.height);
                            if bw.was_minimized {
                                bw.was_minimized = false;
                                #[cfg(target_os = "windows")]
                                if let Some(vis) = bw.pool.visible_webview() {
                                    let _ = vis.set_memory_usage_level(MemoryUsageLevel::Normal);
                                }
                            }
                        }
                    }
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Moved(_),
                ..
            } => {
                if let Some(&wid) = win_by_tao_id.get(&window_id) {
                    if let Some(bw) = windows.get(&wid) {
                        bw.pool.notify_parent_moved();
                    }
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Focused(true),
                ..
            } => {
                if let Some(&wid) = win_by_tao_id.get(&window_id) {
                    focused_win_id = wid;
                    if let Some(bw) = windows.get(&wid) {
                        let active_url = bw.tab_manager.lock_safe().active_url();
                        adblock::set_active_page_url(&active_url);
                        if let Some(vis) = bw.pool.visible_webview() {
                            let _ = vis.focus();
                            #[cfg(target_os = "windows")]
                            {
                                let _ = vis.set_memory_usage_level(MemoryUsageLevel::Normal);
                            }
                        }
                    }
                    // Zero-polling instant hot-reload when user edits any *.json file externally
                    let changed = state_for_event.lock_safe().reload_if_changed();
                    if changed {
                        let global_incog = state_for_event.lock_safe().config.incognito;
                        for b in windows.values_mut() {
                            let aurl = b.tab_manager.lock_safe().active_url();
                            b.tab_manager
                                .lock_safe()
                                .set_incognito(global_incog || b.incognito_window);
                            let json_res = {
                                let st = state_for_event.lock_safe();
                                #[cfg(target_os = "windows")]
                                if let Some(vis) = b.pool.visible_webview() {
                                    adblock::sync_windows_custom_filters(vis, &st.adblock);
                                }
                                serialize_runtime_for_win(&st, &aurl, b.incognito_window)
                            };
                            if let Ok(json) = json_res {
                                let script = format!(
                                    "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                                    json, bridge_tok_js
                                );
                                eval_on_active_and_visible(
                                    &b.pool,
                                    b.active_tab_id,
                                    b.pool.visible_id(),
                                    &script,
                                );
                            }
                            apply_pool_policy(b, &state_for_event);
                        }
                    }
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if let Some(&wid) = win_by_tao_id.get(&window_id) {
                    close_browser_window(
                        wid,
                        &mut windows,
                        &mut win_by_tao_id,
                        &mut focused_win_id,
                        control_flow,
                    );
                }
            }
            _ => (),
        }
    });

    #[allow(unreachable_code)]
    Ok(())
}

fn target_visible_webview(
    windows: &HashMap<usize, BrowserWindow>,
    focused_win_id: usize,
) -> Option<&wry::WebView> {
    windows
        .get(&focused_win_id)
        .or_else(|| windows.values().next())
        .and_then(|bw| bw.pool.visible_webview())
}

fn close_browser_window(
    win_id: usize,
    windows: &mut HashMap<usize, BrowserWindow>,
    win_by_tao_id: &mut HashMap<WindowId, usize>,
    focused_win_id: &mut usize,
    control_flow: &mut ControlFlow,
) {
    if let Some(mut bw) = windows.remove(&win_id) {
        win_by_tao_id.remove(&bw.window.id());
        bw.pool.clear_all();
        bw.tab_manager.lock_safe().save();
    }
    if windows.is_empty() {
        crate::util::flush_pending_writes();
        *control_flow = ControlFlow::Exit;
    } else if *focused_win_id == win_id {
        if let Some(&next_id) = windows.keys().next() {
            *focused_win_id = next_id;
        }
    }
}

fn perform_switch_tab(
    bw: &mut BrowserWindow,
    id: usize,
    url: &str,
    web_context: &mut WebContext,
    app_state: &Arc<Mutex<AppState>>,
    reveal_scheduler: &RevealScheduler,
    bridge_tok_js: &str,
) {
    let sz = bw.window.inner_size();
    let win_size = (sz.width, sz.height);
    let old_id = bw.active_tab_id;
    if id == old_id && bw.pool.contains(id) {
        bw.pool.unpark_active(id, bridge_tok_js);
        if bw.pool.visible_id() != Some(id) && bw.pool.pending_reveal_id() != Some(id) {
            bw.pool.reveal_tab(id, win_size, bridge_tok_js);
        }
        return;
    }

    if bw.pool.pending_reveal_id().is_some_and(|pid| pid != id) {
        bw.pool.clear_pending();
    }

    if old_id != 0 && old_id != id {
        let old_has_bg = tabs_for_bg(&bw.tab_manager, old_id);
        bw.pool.park_inactive(old_id, old_has_bg, bridge_tok_js);
        if !old_has_bg {
            bw.tab_manager.lock_safe().set_tab_playing_audio(old_id, false);
        }
    }

    bw.active_tab_id = id;
    bw.pool.touch_lru(id);

    let (tab_state, is_tab_muted) = {
        let tm = bw.tab_manager.lock_safe();
        let muted = tm.tab_meta(id).map(|(_, m)| m).unwrap_or(false);
        (tm.get_state(), muted)
    };

    if bw.pool.contains(id) {
        let has_visible_backing = bw.pool.visible_id().is_some() && bw.pool.visible_id() != Some(id);
        if has_visible_backing {
            bw.pool.prepare_pending_behind(id, win_size);
        }
        bw.pool.unpark_active(id, bridge_tok_js);
        if let Some(wv) = bw.pool.get_webview(id) {
            {
                let st = app_state.lock_safe();
                sync_foreground_env(
                    wv,
                    &st,
                    bw.window.fullscreen().is_some(),
                    bridge_tok_js,
                    Some(url),
                    bw.incognito_window,
                );
            }
            sync_tabs_and_mute(wv, &tab_state, is_tab_muted, bridge_tok_js);
            let nav_js = nav_state_js(wv, bridge_tok_js);
            let _ = wv.evaluate_script(&nav_js);
            if has_visible_backing {
                let frame_script = format!(
                    "window.__fynexo_request_reveal_frame__ && window.__fynexo_request_reveal_frame__({});",
                    bridge_tok_js
                );
                let _ = wv.evaluate_script(&frame_script);
            }
        }
        if has_visible_backing {
            let gen = bw.pool.set_pending_reveal(id);
            reveal_scheduler.schedule(bw.win_id, id, gen, std::time::Duration::from_millis(90));
        } else {
            bw.pool.reveal_tab(id, win_size, bridge_tok_js);
        }
    } else {
        let has_visible_backing = bw.pool.visible_id().is_some();
        let target = to_webview_url(url);

        if let Some(tv) = bw.pool.take_idle_for(id) {
            #[cfg(target_os = "windows")]
            {
                let _ = tv.webview.set_memory_usage_level(MemoryUsageLevel::Normal);
            }
            {
                let st = app_state.lock_safe();
                sync_foreground_env(
                    &tv.webview,
                    &st,
                    bw.window.fullscreen().is_some(),
                    bridge_tok_js,
                    None,
                    bw.incognito_window,
                );
            }
            let _ = tv.webview.load_url(&target);
            bw.pool.insert_live(id, tv);
        } else {
            match build_tab_webview(
                &bw.window,
                web_context,
                TabLaunch {
                    win_id: bw.win_id,
                    tab_id: id,
                    url: &target,
                    visible: !has_visible_backing,
                },
                app_state,
                &bw.tab_manager,
                reveal_scheduler.proxy(),
            ) {
                Ok(new_tv) => {
                    bw.pool.insert_live(id, new_tv);
                }
                Err(e) => {
                    eprintln!("[Fynexo] Failed to create tab webview: {:?}", e);
                }
            }
        }

        if !has_visible_backing {
            bw.pool.reveal_tab(id, win_size, bridge_tok_js);
        } else {
            if let Some(vis) = bw.pool.visible_webview() {
                sync_tabs_and_mute(vis, &tab_state, is_tab_muted, bridge_tok_js);
                let prog_script = format!(
                    "window.__fynexo_trigger_progress__ && window.__fynexo_trigger_progress__({});",
                    bridge_tok_js
                );
                let _ = vis.evaluate_script(&prog_script);
            }
            bw.pool.prepare_pending_behind(id, win_size);
            let gen = bw.pool.set_pending_reveal(id);
            reveal_scheduler.schedule(bw.win_id, id, gen, std::time::Duration::from_millis(4000));
        }
    }

    apply_pool_policy(bw, app_state);
}

fn apply_pool_policy(bw: &mut BrowserWindow, app_state: &Arc<Mutex<AppState>>) {
    let (limits, eff, auto_trim) = {
        let st = app_state.lock_safe();
        (
            st.config.tab_cache_limits(),
            st.config.efficiency_mode,
            st.config.auto_trim_ram,
        )
    };
    let tabs = bw.tab_manager.lock_safe().tabs.clone();
    let evicted =
        bw.pool
            .evict_excess_normal_tabs(&tabs, bw.active_tab_id, limits.max_cached_normal);
    bw.pool
        .apply_memory_tiers(bw.active_tab_id, limits.hot_normal_count, eff);
    let live_ids = bw.pool.live_ids();
    bw.tab_manager.lock_safe().sync_suspended_flags(&live_ids);
    #[cfg(target_os = "windows")]
    if evicted > 0 && auto_trim {
        let _ = force_empty_working_set();
    }
    #[cfg(not(target_os = "windows"))]
    let _ = (evicted, auto_trim);
}

fn complete_pending_reveal(
    bw: &mut BrowserWindow,
    tab_id: usize,
    app_state: &Arc<Mutex<AppState>>,
    bridge_tok_js: &str,
) {
    let (tab_state, is_tab_muted) = {
        let tm = bw.tab_manager.lock_safe();
        let muted = tm.tab_meta(tab_id).map(|(_, m)| m).unwrap_or(false);
        (tm.get_state(), muted)
    };
    bw.pool.unpark_active(tab_id, bridge_tok_js);
    if let Some(wv) = bw.pool.get_webview(tab_id) {
        sync_tabs_and_mute(wv, &tab_state, is_tab_muted, bridge_tok_js);
    }
    let sz = bw.window.inner_size();
    bw.pool
        .reveal_tab(tab_id, (sz.width, sz.height), bridge_tok_js);
    apply_pool_policy(bw, app_state);
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

fn build_resource_stats_json(windows: &HashMap<usize, BrowserWindow>) -> serde_json::Value {
    #[allow(unused_mut)]
    let mut browser_pids: Vec<u32> = Vec::new();
    let mut live_tabs: usize = 0;
    let mut total_tabs: usize = 0;
    for bw in windows.values() {
        live_tabs += bw.pool.live_count();
        total_tabs += bw.tab_manager.lock_safe().tabs.len();
        #[cfg(target_os = "windows")]
        for wv in bw.pool.all_webviews() {
            let mut pid: u32 = 0;
            unsafe {
                if wv.webview().BrowserProcessId(&mut pid).is_ok()
                    && pid != 0
                    && !browser_pids.contains(&pid)
                {
                    browser_pids.push(pid);
                }
            }
        }
    }
    let snap = get_process_tree_memory_with_roots(&browser_pids);
    serde_json::json!({
        "host_mb": (snap.host_mb * 10.0).round() / 10.0,
        "webview_mb": (snap.webview_mb * 10.0).round() / 10.0,
        "total_mb": (snap.total_mb * 10.0).round() / 10.0,
        "commit_mb": (snap.commit_mb * 10.0).round() / 10.0,
        "process_count": snap.process_count,
        "live_tabs": live_tabs,
        "total_tabs": total_tabs,
        "window_count": windows.len(),
    })
}

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

/// Only send full `RuntimePayload` to internal Fynexo pages; external websites receive stripped `HudBootPayload`.
/// When a secondary window is opened in InPrivate mode, `incognito` is reflected in that window's payload only.
fn serialize_runtime_for_win(
    state: &AppState,
    active_url: &str,
    win_incognito: bool,
) -> Result<String, serde_json::Error> {
    if security::is_internal_origin(active_url) {
        if win_incognito && !state.config.incognito {
            let mut p = state.payload();
            p.config.incognito = true;
            serde_json::to_string(&p)
        } else {
            serde_json::to_string(&state.payload())
        }
    } else if win_incognito && !state.config.incognito {
        let mut cfg = state.config.clone();
        cfg.incognito = true;
        let active_theme = state
            .resolve_active_theme_ref()
            .cloned()
            .unwrap_or_default();
        let enabled_rules: Vec<&crate::config::SiteRule> =
            state.site_rules.iter().filter(|r| r.enabled).collect();
        let history_slice = &state.userdata.history[..state.userdata.history.len().min(25)];
        let slim = crate::config::HudBootPayload {
            config: &cfg,
            styles: &state.styles,
            active_theme: &active_theme,
            keybinds: &state.keybinds,
            enabled_rules,
            bookmarks: &state.userdata.bookmarks,
            history: history_slice,
            tabs: None,
            mode_themes: crate::config::collect_mode_themes(&cfg, &state.themes),
            downloads: &state.recent_downloads,
        };
        serde_json::to_string(&slim)
    } else {
        state.hud_json()
    }
}

fn sync_foreground_env(
    webview: &wry::WebView,
    state: &AppState,
    is_fullscreen: bool,
    bridge_tok_js: &str,
    push_runtime_for_url: Option<&str>,
    win_incognito: bool,
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
        if let Ok(rt_json) = serialize_runtime_for_win(state, url, win_incognito) {
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
