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

use std::collections::{HashMap, HashSet};
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
use crate::chrome_ext::{self, HudChromeExtension};
use crate::config::data_dir;
use crate::tabs::TabManager;
use crate::util::LockExt;

use self::events::{HudInsetLogical, HudRegionRectLogical};
#[cfg(target_os = "windows")]
use self::memory::{force_empty_working_set, get_process_tree_memory_with_roots};
#[cfg(not(target_os = "windows"))]
use self::memory::get_process_tree_memory_with_roots;
use self::scheduler::RevealScheduler;
use self::tab_pool::TabPool;
use self::url::{is_internal_url, to_webview_url};
#[cfg(target_os = "windows")]
use self::webview::{
    bring_hud_to_top, build_hud_webview, apply_hud_window_region, estimate_initial_regions,
    is_hud_taking_focus, logical_regions_to_physical, place_behind_hud, sync_webview_bounds,
};
use self::webview::{
    build_extension_popup_webview, build_tab_webview, compute_content_bounds,
    effective_zoom_for_url, estimate_initial_inset, ContentBounds, TabLaunch,
};

struct BrowserWindow {
    win_id: usize,
    window: tao::window::Window,
    #[cfg(target_os = "windows")]
    hud_webview: wry::WebView,
    #[cfg(target_os = "windows")]
    hud_inset: HudInsetLogical,
    #[cfg(target_os = "windows")]
    hud_regions_logical: Vec<HudRegionRectLogical>,
    #[cfg(target_os = "windows")]
    hud_has_overlay: bool,
    #[cfg(target_os = "windows")]
    hud_layout_synced: bool,
    #[cfg(target_os = "windows")]
    content_bounds: ContentBounds,
    incognito_window: bool,
    tab_manager: Arc<Mutex<TabManager>>,
    pool: TabPool,
    active_tab_id: usize,
    loading_tabs: HashSet<usize>,
    was_minimized: bool,
    last_maximize_time: std::time::Instant,
}

impl BrowserWindow {
    #[inline]
    fn hud_opt(&self) -> Option<&wry::WebView> {
        #[cfg(target_os = "windows")]
        {
            Some(&self.hud_webview)
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    }

    #[inline]
    fn content_bounds_val(&self) -> ContentBounds {
        #[cfg(target_os = "windows")]
        {
            self.content_bounds
        }
        #[cfg(not(target_os = "windows"))]
        {
            ContentBounds::default()
        }
    }

    #[inline]
    fn should_focus_content(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            !self.hud_has_overlay
        }
        #[cfg(not(target_os = "windows"))]
        {
            true
        }
    }

    fn eval_hud(&self, script: &str) {
        #[cfg(target_os = "windows")]
        {
            let _ = self.hud_webview.evaluate_script(script);
        }
        #[cfg(not(target_os = "windows"))]
        {
            eval_on_active_and_visible(&self.pool, self.active_tab_id, self.pool.visible_id(), script);
        }
    }

    fn eval_hud_and_tabs(&self, script: &str) {
        #[cfg(target_os = "windows")]
        {
            let _ = self.hud_webview.evaluate_script(script);
        }
        eval_on_active_and_visible(&self.pool, self.active_tab_id, self.pool.visible_id(), script);
    }

    fn sync_hud_loading_state(&self, immediate: bool, bridge_tok_js: &str) {
        let script = if self.loading_tabs.contains(&self.active_tab_id) {
            format!(
                "window.__fynexo_trigger_progress__ && window.__fynexo_trigger_progress__({});",
                bridge_tok_js
            )
        } else {
            format!(
                "window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__({}, {});",
                if immediate { "true" } else { "false" },
                bridge_tok_js
            )
        };
        self.eval_hud(&script);
    }

    #[cfg(target_os = "windows")]
    fn recompute_dual_layout(&mut self) {
        let sz = self.window.inner_size();
        if sz.width == 0 || sz.height == 0 {
            return;
        }
        let sf = self.window.scale_factor().max(0.1);
        self.content_bounds = compute_content_bounds(sz.width, sz.height, sf, self.hud_inset);
        if !self.hud_layout_synced {
            self.hud_regions_logical = estimate_initial_regions(
                self.hud_inset,
                sz.width as f64 / sf,
                sz.height as f64 / sf,
            );
        }
        sync_webview_bounds(&self.hud_webview, sz.width, sz.height);
        let phys = logical_regions_to_physical(
            &self.hud_regions_logical,
            sz.width,
            sz.height,
            sf,
            self.content_bounds,
        );
        apply_hud_window_region(&self.hud_webview, &phys);
        bring_hud_to_top(&self.hud_webview);
        if let Some(vis) = self.pool.visible_webview() {
            place_behind_hud(vis, &self.hud_webview);
        }
        self.pool.resize_all(self.content_bounds);
    }
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

    let init_inset = {
        let mut st = app_state.lock_safe();
        let _ = st.reload_if_changed();
        estimate_initial_inset(&st, &active_logical, false)
    };
    let win_size = window.inner_size();
    let sf = window.scale_factor().max(0.1);
    let init_content_bounds = compute_content_bounds(win_size.width, win_size.height, sf, init_inset);

    #[cfg(target_os = "windows")]
    let (hud_webview, hud_regions_logical) = build_hud_webview(
        &window,
        web_context,
        win_id,
        incognito_window,
        init_inset,
        app_state,
        &tab_manager,
        event_proxy,
    )?;

    let initial_webview = build_tab_webview(
        &window,
        web_context,
        TabLaunch {
            win_id,
            tab_id: initial_active_id,
            url: &active_logical,
            visible: true,
            init_bounds: init_content_bounds,
        },
        app_state,
        &tab_manager,
        event_proxy,
    )?;

    #[cfg(target_os = "windows")]
    {
        place_behind_hud(&initial_webview.webview, &hud_webview);
        bring_hud_to_top(&hud_webview);
    }

    let pool = TabPool::new(initial_active_id, initial_webview);
    if let Some(wv) = pool.get_webview(initial_active_id) {
        set_webview_os_muted(wv, initial_muted);
    }

    Ok(BrowserWindow {
        win_id,
        window,
        #[cfg(target_os = "windows")]
        hud_webview,
        #[cfg(target_os = "windows")]
        hud_inset: init_inset,
        #[cfg(target_os = "windows")]
        hud_regions_logical,
        #[cfg(target_os = "windows")]
        hud_has_overlay: false,
        #[cfg(target_os = "windows")]
        hud_layout_synced: false,
        #[cfg(target_os = "windows")]
        content_bounds: init_content_bounds,
        incognito_window,
        tab_manager,
        pool,
        active_tab_id: initial_active_id,
        loading_tabs: HashSet::new(),
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

    {
        let st = app_state.lock_safe();
        if st.boot_chrome_ext_enabled {
            if let Some(wv) = primary_win.pool.visible_webview() {
                chrome_ext::windows_bridge::sync_all_extensions(
                    wv,
                    st.chrome_extensions.clone(),
                    event_proxy.clone(),
                );
            }
        }
    }

    let mut windows: HashMap<usize, BrowserWindow> = HashMap::new();
    let mut win_by_tao_id: HashMap<WindowId, usize> = HashMap::new();
    win_by_tao_id.insert(primary_win.window.id(), primary_win.win_id);
    windows.insert(primary_win.win_id, primary_win);

    let mut next_win_id: usize = 2;
    let mut focused_win_id: usize = 1;
    #[allow(clippy::type_complexity)]
    let mut ext_popup: Option<(tao::window::Window, wry::WebView, String)> = None;

    let state_for_event = Arc::clone(&app_state);
    let event_proxy_for_run = event_proxy.clone();
    let reveal_scheduler = RevealScheduler::new(event_proxy.clone());
    let bridge_tok_js =
        serde_json::to_string(security::bridge_token()).unwrap_or_else(|_| "\"\"".to_string());

    event_loop.run(move |event, event_loop_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => (),
            Event::UserEvent(UserEvent::SyncHudLayout {
                win_id,
                inset,
                regions,
                has_overlay,
            }) => {
                #[cfg(target_os = "windows")]
                if let Some(bw) = windows.get_mut(&win_id) {
                    bw.hud_inset = inset;
                    bw.hud_regions_logical = regions;
                    bw.hud_has_overlay = has_overlay;
                    bw.hud_layout_synced = true;
                    bw.recompute_dual_layout();
                }
                #[cfg(not(target_os = "windows"))]
                let _ = (win_id, inset, regions, has_overlay);
            }
            Event::UserEvent(UserEvent::ContentAction {
                win_id,
                action,
                payload,
            }) => {
                if let Some(bw) = windows.get(&win_id) {
                    if let Some(wv) = bw.pool.active_or_visible_webview(bw.active_tab_id) {
                        if action == "focus_content" {
                            let _ = wv.focus();
                        } else {
                            if matches!(
                                action.as_str(),
                                "open_find" | "start_element_picker" | "toggle_dev_drawer"
                            ) {
                                let _ = wv.focus();
                            }
                            if let (Ok(act_json), Ok(pay_json)) = (
                                serde_json::to_string(&action),
                                serde_json::to_string(&payload),
                            ) {
                                let script = format!(
                                    "window.__fynexo_content_action__ && window.__fynexo_content_action__({}, {}, {});",
                                    act_json, pay_json, bridge_tok_js
                                );
                                let _ = wv.evaluate_script(&script);
                            }
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::HudAction { win_id, action }) => {
                if let Some(bw) = windows.get(&win_id) {
                    #[cfg(target_os = "windows")]
                    if matches!(
                        action.as_str(),
                        "open_omnibox" | "open_command_palette" | "toggle_side_panel"
                    ) {
                        let _ = bw.hud_webview.focus();
                    }
                    if let Ok(act_json) = serde_json::to_string(&action) {
                        let script = format!(
                            "window.__fynexo_hud_action__ && window.__fynexo_hud_action__({}, {});",
                            act_json, bridge_tok_js
                        );
                        bw.eval_hud(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::ContentPointerDown { win_id }) => {
                #[cfg(target_os = "windows")]
                if let Some(bw) = windows.get(&win_id) {
                    if bw.hud_has_overlay {
                        let script = format!(
                            "window.__fynexo_hud_action__ && window.__fynexo_hud_action__(\"dismiss_overlays\", {});",
                            bridge_tok_js
                        );
                        let _ = bw.hud_webview.evaluate_script(&script);
                    }
                }
                #[cfg(not(target_os = "windows"))]
                let _ = win_id;
            }
            Event::UserEvent(UserEvent::ContentPageMeta { win_id, tab_id, meta }) => {
                #[cfg(target_os = "windows")]
                if let Some(bw) = windows.get(&win_id) {
                    if bw.active_tab_id == tab_id {
                        if let Ok(meta_json) = serde_json::to_string(&meta) {
                            let script = format!(
                                "window.__fynexo_set_page_meta__ && window.__fynexo_set_page_meta__({}, {});",
                                meta_json, bridge_tok_js
                            );
                            let _ = bw.hud_webview.evaluate_script(&script);
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                let _ = (win_id, tab_id, meta);
            }
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
                    bw.pool.clear_runtime_hash(bw.active_tab_id);
                    let target = to_webview_url(&url);
                    let zoom_factor = effective_zoom_for_url(&state_for_event.lock_safe(), &url);
                    #[cfg(debug_assertions)]
                    println!("[Navigation Request w{}]: {} -> {}", win_id, url, target);
                    if is_internal_url(&url) {
                        bw.loading_tabs.remove(&bw.active_tab_id);
                        bw.sync_hud_loading_state(true, &bridge_tok_js);
                    } else {
                        bw.loading_tabs.insert(bw.active_tab_id);
                        bw.sync_hud_loading_state(false, &bridge_tok_js);
                    }
                    if let Some(fg) = bw.pool.get_webview(bw.active_tab_id) {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = fg.set_memory_usage_level(MemoryUsageLevel::Normal);
                        }
                        let _ = fg.zoom(zoom_factor);
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
                    bw.loading_tabs.clear();
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
                    let target_tid = if tab_id == 0 { bw.active_tab_id } else { tab_id };
                    let (tab_state, tab_url, scroll_y, keep_bg, is_tab_muted) = {
                        let tm = bw.tab_manager.lock_safe();
                        let (u, sy) = tm.url_and_scroll_for(target_tid);
                        let (bg, muted) = tm.tab_meta(target_tid).unwrap_or((false, false));
                        (tm.get_state(), u, sy, bg, muted)
                    };
                    let is_full = bw.window.fullscreen().is_some();
                    let runtime_url = if tab_id == 0 {
                        "fynexo://settings"
                    } else {
                        tab_url.as_str()
                    };
                    if let (Ok(rt_json), Ok(tabs_json)) = (
                        serialize_runtime_for_win(
                            &state_for_event.lock_safe(),
                            runtime_url,
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
                        if tab_id == 0 {
                            let is_active_loading = bw.loading_tabs.contains(&bw.active_tab_id);
                            script.push_str(&if is_active_loading {
                                format!(
                                    " window.__fynexo_trigger_progress__ && window.__fynexo_trigger_progress__({});",
                                    bridge_tok_js
                                )
                            } else {
                                format!(
                                    " window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {});",
                                    bridge_tok_js
                                )
                            });
                            #[cfg(target_os = "windows")]
                            {
                                if let Some(fg) = bw.pool.active_or_visible_webview(bw.active_tab_id) {
                                    script.push(' ');
                                    script.push_str(&nav_state_js(fg, &bridge_tok_js));
                                }
                                let _ = bw.hud_webview.evaluate_script(&script);
                            }
                        } else {
                            let rt_hash = crate::util::fnv1a_hash64(rt_json.as_bytes());
                            bw.pool.should_push_runtime(tab_id, rt_hash);
                            if scroll_y > 24.0 {
                                script.push_str(&format!(
                                    " window.__fynexo_restore_scroll__ && window.__fynexo_restore_scroll__({}, {});",
                                    scroll_y, bridge_tok_js
                                ));
                            }
                            if let Some(wv) = bw.pool.get_webview(tab_id) {
                                set_webview_os_muted(wv, is_tab_muted);
                                let nav_js = nav_state_js(wv, &bridge_tok_js);
                                script.push(' ');
                                script.push_str(&nav_js);
                                let _ = wv.evaluate_script(&script);
                                #[cfg(target_os = "windows")]
                                if tab_id == bw.active_tab_id {
                                    let _ = bw.hud_webview.evaluate_script(&nav_js);
                                }
                            }
                            if tab_id != bw.active_tab_id {
                                bw.pool.park_inactive(tab_id, keep_bg, &bridge_tok_js);
                            }
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncNavState { win_id, tab_id }) => {
                if let Some(bw) = windows.get(&win_id) {
                    let target_tid = if tab_id == 0 { bw.active_tab_id } else { tab_id };
                    if let Some(wv) = bw.pool.get_webview(target_tid) {
                        let script = nav_state_js(wv, &bridge_tok_js);
                        let _ = wv.evaluate_script(&script);
                        #[cfg(target_os = "windows")]
                        if target_tid == bw.active_tab_id {
                            let _ = bw.hud_webview.evaluate_script(&script);
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncConfig) => {
                for bw in windows.values_mut() {
                    let live_ids = bw.pool.live_ids();
                    let tab_urls: Vec<(usize, String)> = {
                        let tm = bw.tab_manager.lock_safe();
                        live_ids
                            .into_iter()
                            .map(|tid| (tid, tm.url_and_scroll_for(tid).0))
                            .collect()
                    };
                    let st = state_for_event.lock_safe();
                    let mut internal_payload: Option<(String, u64)> = None;
                    let mut external_payload: Option<(String, u64)> = None;
                    if let Ok(hud_json) =
                        serialize_runtime_for_win(&st, "fynexo://settings", bw.incognito_window)
                    {
                        let h = crate::util::fnv1a_hash64(hud_json.as_bytes());
                        internal_payload = Some((hud_json.clone(), h));
                        #[cfg(target_os = "windows")]
                        {
                            let hud_script = format!(
                                "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                                hud_json, bridge_tok_js
                            );
                            let _ = bw.hud_webview.evaluate_script(&hud_script);
                        }
                    }
                    for (tid, tab_url) in tab_urls {
                        let z = effective_zoom_for_url(&st, &tab_url);
                        if let Some(wv) = bw.pool.get_webview(tid) {
                            let _ = wv.zoom(z);
                        }
                        let is_int = security::is_internal_origin(&tab_url);
                        let cached = if is_int {
                            internal_payload.as_ref()
                        } else {
                            if external_payload.is_none() {
                                if let Ok(json) = serialize_runtime_for_win(
                                    &st,
                                    "https://external.invalid/",
                                    bw.incognito_window,
                                ) {
                                    let h = crate::util::fnv1a_hash64(json.as_bytes());
                                    external_payload = Some((json, h));
                                }
                            }
                            external_payload.as_ref()
                        };
                        if let Some((json, hash)) = cached {
                            if bw.pool.should_push_runtime(tid, *hash) {
                                if let Some(wv) = bw.pool.get_webview(tid) {
                                    let script = format!(
                                        "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                                        json, bridge_tok_js
                                    );
                                    let _ = wv.evaluate_script(&script);
                                }
                            }
                        }
                    }
                    drop(st);
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
                        let cbounds = bw.content_bounds_val();
                        let hud_ptr = bw.hud_opt().map(|w| w as *const wry::WebView);
                        let focus_tab = bw.should_focus_content();
                        bw.pool.reveal_tab(
                            bw.active_tab_id,
                            cbounds,
                            hud_ptr.map(|p| unsafe { &*p }),
                            focus_tab,
                            &bridge_tok_js,
                        );
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
                        bw.eval_hud_and_tabs(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::PageStarted { win_id, tab_id, url }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    if is_internal_url(&url) {
                        bw.loading_tabs.remove(&tab_id);
                        if bw.active_tab_id == tab_id {
                            bw.sync_hud_loading_state(true, &bridge_tok_js);
                        }
                    } else {
                        bw.loading_tabs.insert(tab_id);
                        if bw.active_tab_id == tab_id {
                            let script = format!(
                                "window.__fynexo_trigger_progress__ && window.__fynexo_trigger_progress__({});",
                                bridge_tok_js
                            );
                            bw.eval_hud_and_tabs(&script);
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::PageFinished {
                win_id,
                tab_id,
                url,
            }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    bw.loading_tabs.remove(&tab_id);
                    let (expected_url, scroll_y, is_tab_muted) = {
                        let tm = bw.tab_manager.lock_safe();
                        let (u, sy) = tm.url_and_scroll_for(tab_id);
                        let muted = tm.tab_meta(tab_id).map(|(_, m)| m).unwrap_or(false);
                        (u, sy, muted)
                    };
                    let site_zoom = effective_zoom_for_url(&state_for_event.lock_safe(), &url);
                    if let Some(wv) = bw.pool.get_webview(tab_id) {
                        set_webview_os_muted(wv, is_tab_muted);
                        let _ = wv.zoom(site_zoom);
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
                        let nav_js = nav_state_js(wv, &bridge_tok_js);
                        script.push_str(&nav_js);
                        let _ = wv.evaluate_script(&script);
                        #[cfg(target_os = "windows")]
                        if tab_id == bw.active_tab_id {
                            let hud_script = format!(
                                "{} window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(false, {});",
                                nav_js, bridge_tok_js
                            );
                            let _ = bw.hud_webview.evaluate_script(&hud_script);
                        }
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
                    let zh = !st.config.language.trim().eq_ignore_ascii_case("en");
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
                        bw.eval_hud(&dl_script);
                    }
                }
                if let Ok(json_msg) = serde_json::to_string(&toast_msg) {
                    let t_script = format!(
                        "window.__fynexo_toast__ && window.__fynexo_toast__({}, {});",
                        json_msg, bridge_tok_js
                    );
                    if let Some(hud) = target_hud_webview(&windows, focused_win_id) {
                        let _ = hud.evaluate_script(&t_script);
                    }
                }
            }
            Event::UserEvent(UserEvent::Toast(msg)) => {
                if let Ok(json_msg) = serde_json::to_string(&msg) {
                    let script = format!(
                        "window.__fynexo_toast__ && window.__fynexo_toast__({}, {});",
                        json_msg, bridge_tok_js
                    );
                    if let Some(hud) = target_hud_webview(&windows, focused_win_id) {
                        let _ = hud.evaluate_script(&script);
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
                    bw.eval_hud_and_tabs(&script);
                }
            }
            Event::UserEvent(UserEvent::ApplyZoom(_factor)) => {
                let st = state_for_event.lock_safe();
                for bw in windows.values() {
                    let url = bw.tab_manager.lock_safe().url_and_scroll_for(bw.active_tab_id).0;
                    let z = effective_zoom_for_url(&st, &url);
                    if let Some(wv) = bw.pool.get_webview(bw.active_tab_id) {
                        let _ = wv.zoom(z);
                    }
                    if let Some(vid) = bw.pool.visible_id() {
                        if vid != bw.active_tab_id {
                            let vurl = bw.tab_manager.lock_safe().url_and_scroll_for(vid).0;
                            let vz = effective_zoom_for_url(&st, &vurl);
                            if let Some(vis) = bw.pool.get_webview(vid) {
                                let _ = vis.zoom(vz);
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
                    let zh = !state_for_event
                        .lock_safe()
                        .config
                        .language
                        .trim()
                        .eq_ignore_ascii_case("en");
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
                    navigate_webview_history(bw, false, &bridge_tok_js);
                }
            }
            Event::UserEvent(UserEvent::GoForward(win_id)) => {
                if let Some(bw) = windows.get(&win_id) {
                    navigate_webview_history(bw, true, &bridge_tok_js);
                }
            }
            Event::UserEvent(UserEvent::Reload(win_id)) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    bw.pool.clear_runtime_hash(bw.active_tab_id);
                    let active_url = bw
                        .tab_manager
                        .lock_safe()
                        .url_and_scroll_for(bw.active_tab_id)
                        .0;
                    if is_internal_url(&active_url) {
                        bw.loading_tabs.remove(&bw.active_tab_id);
                        bw.sync_hud_loading_state(true, &bridge_tok_js);
                    } else {
                        bw.loading_tabs.insert(bw.active_tab_id);
                        bw.sync_hud_loading_state(false, &bridge_tok_js);
                    }
                    if let Some(fg) = bw.pool.get_webview(bw.active_tab_id) {
                        let _ = fg.reload();
                    }
                }
            }
            Event::UserEvent(UserEvent::OpenDevtools(win_id)) => {
                #[cfg(not(target_os = "android"))]
                {
                    let (allow_eval, zh) = {
                        let st = state_for_event.lock_safe();
                        (
                            cfg!(debug_assertions) || st.config.allow_page_eval,
                            crate::util::is_zh_lang(&st.config.language),
                        )
                    };
                    if !allow_eval {
                        let _ = event_proxy_for_run.send_event(UserEvent::Toast(
                            crate::util::i18n(
                                zh,
                                "安全保护：请先在「设置 -> 常规」开启「允许控制台 JS 执行与原生 DevTools」",
                                "Protected: enable 'Allow Console JS Eval & Native DevTools' in Settings -> General first",
                            )
                            .to_string(),
                        ));
                    } else if let Some(bw) = windows.get(&win_id) {
                        if let Some(fg) = bw.pool.get_webview(bw.active_tab_id) {
                            fg.open_devtools();
                        }
                    }
                }
                #[cfg(target_os = "android")]
                {
                    let _ = win_id;
                    let zh = crate::util::is_zh_lang(&state_for_event.lock_safe().config.language);
                    let _ = event_proxy_for_run.send_event(UserEvent::Toast(
                        crate::util::i18n(
                            zh,
                            "原生开发者工具仅支持桌面端",
                            "Native DevTools only supported on Desktop",
                        )
                        .to_string(),
                    ));
                }
            }
            Event::UserEvent(UserEvent::ReloadTab { win_id, id }) => {
                if let Some(bw) = windows.get_mut(&win_id) {
                    bw.pool.clear_runtime_hash(id);
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
                    let stats_json = build_resource_stats_json(&windows);
                    let script = format!(
                        "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({}, {});",
                        stats_json, bridge_tok_js
                    );
                    bw.eval_hud_and_tabs(&script);
                }
            }
            Event::UserEvent(UserEvent::TrimWorkingSet(win_id)) => {
                #[cfg(target_os = "windows")]
                {
                    let (eff, zh) = {
                        let st = state_for_event.lock_safe();
                        (
                            st.config.efficiency_mode,
                            !st.config.language.trim().eq_ignore_ascii_case("en"),
                        )
                    };
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
                        let toast_txt = if zh {
                            format!("内存已压缩清理（共 {:.0} MB）", total_mb)
                        } else {
                            format!("Working set trimmed ({:.0} MB total)", total_mb)
                        };
                        if let Ok(json_msg) = serde_json::to_string(&toast_txt) {
                            let hud_script = format!(
                                "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({}, {}); window.__fynexo_toast__ && window.__fynexo_toast__({}, {});",
                                stats_json, bridge_tok_js, json_msg, bridge_tok_js
                            );
                            let _ = bw.hud_webview.evaluate_script(&hud_script);
                        }
                        let perf_script = format!(
                            "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({}, {});",
                            stats_json, bridge_tok_js
                        );
                        if let Some(vis) = bw.pool.visible_webview() {
                            let _ = vis.evaluate_script(&perf_script);
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
            Event::UserEvent(UserEvent::InstallChromeExtUnpacked { entry }) => {
                let entry = *entry;
                let (boot_en, is_zh, eid, dpath, des_en, name) = {
                    let mut st = state_for_event.lock_safe();
                    let en_boot = st.boot_chrome_ext_enabled;
                    let zh = !st.config.language.trim().eq_ignore_ascii_case("en");
                    if let Some(pos) = st.chrome_extensions.iter().position(|e| e.id == entry.id) {
                        st.chrome_extensions[pos] = entry.clone();
                    } else {
                        st.chrome_extensions.insert(0, entry.clone());
                    }
                    chrome_ext::save_registry(&st.chrome_extensions);
                    (
                        en_boot,
                        zh,
                        entry.id.clone(),
                        entry.dir_path.clone(),
                        entry.enabled,
                        entry.name.clone(),
                    )
                };
                if boot_en {
                    if let Some(wv) = target_visible_webview(&windows, focused_win_id) {
                        chrome_ext::windows_bridge::add_extension(
                            wv,
                            eid,
                            dpath,
                            des_en,
                            event_proxy_for_run.clone(),
                        );
                    }
                } else {
                    let msg = if is_zh {
                        format!(
                            "扩展「{}」已解包；开启 Chrome 扩展引擎并重启即可生效",
                            name
                        )
                    } else {
                        format!(
                            "Unpacked '{}'; enable Chrome Extensions engine & restart to run",
                            name
                        )
                    };
                    let _ = event_proxy_for_run.send_event(UserEvent::Toast(msg));
                }
                let _ = event_proxy_for_run.send_event(UserEvent::SyncConfig);
            }
            Event::UserEvent(UserEvent::ChromeExtRuntimeReady {
                entry_id,
                runtime_id,
                enabled,
                error,
            }) => {
                if let Some(err) = error {
                    let zh = !state_for_event
                        .lock_safe()
                        .config
                        .language
                        .trim()
                        .eq_ignore_ascii_case("en");
                    let _ = event_proxy_for_run.send_event(UserEvent::Toast(if zh {
                        format!("扩展加载出错：{}", err)
                    } else {
                        format!("Extension load error: {}", err)
                    }));
                } else {
                    let (ext_name, zh) = {
                        let mut st = state_for_event.lock_safe();
                        let zh = !st.config.language.trim().eq_ignore_ascii_case("en");
                        let mut nm = entry_id.clone();
                        if let Some(entry) =
                            st.chrome_extensions.iter_mut().find(|e| e.id == entry_id)
                        {
                            if !runtime_id.is_empty() {
                                entry.runtime_id = runtime_id;
                            }
                            entry.enabled = enabled;
                            nm = entry.name.clone();
                            chrome_ext::save_registry(&st.chrome_extensions);
                        }
                        (nm, zh)
                    };
                    if enabled {
                        let msg = if zh {
                            format!("已启用 Chrome 扩展：{}", ext_name)
                        } else {
                            format!("Chrome extension active: {}", ext_name)
                        };
                        let _ = event_proxy_for_run.send_event(UserEvent::Toast(msg));
                    }
                    let _ = event_proxy_for_run.send_event(UserEvent::SyncConfig);
                }
            }
            Event::UserEvent(UserEvent::ChromeExtSynced(loaded)) => {
                let mut changed = false;
                {
                    let mut st = state_for_event.lock_safe();
                    for entry in st.chrome_extensions.iter_mut() {
                        if let Some((rid, _, _)) = loaded.iter().find(|(rid, nm, _)| {
                            (!entry.runtime_id.is_empty() && rid == &entry.runtime_id)
                                || rid == &entry.id
                                || nm.eq_ignore_ascii_case(&entry.name)
                        }) {
                            if entry.runtime_id != *rid {
                                entry.runtime_id = rid.clone();
                                changed = true;
                            }
                        }
                    }
                    if changed {
                        chrome_ext::save_registry(&st.chrome_extensions);
                    }
                }
                if changed {
                    let _ = event_proxy_for_run.send_event(UserEvent::SyncConfig);
                }
            }
            Event::UserEvent(UserEvent::ToggleChromeExt { id, enabled }) => {
                let (boot_en, rid) = {
                    let st = state_for_event.lock_safe();
                    let rid = st
                        .chrome_extensions
                        .iter()
                        .find(|e| e.id == id)
                        .map(|e| e.runtime_id.clone())
                        .unwrap_or_default();
                    (st.boot_chrome_ext_enabled, rid)
                };
                if !enabled {
                    if let Some((_, _, pid)) = &ext_popup {
                        if pid == &id {
                            ext_popup = None;
                        }
                    }
                }
                if boot_en {
                    if let Some(wv) = target_visible_webview(&windows, focused_win_id) {
                        chrome_ext::windows_bridge::set_extension_enabled(
                            wv,
                            id,
                            rid,
                            enabled,
                            event_proxy_for_run.clone(),
                        );
                    }
                }
            }
            Event::UserEvent(UserEvent::RemoveChromeExt { id }) => {
                if let Some((_, _, pid)) = &ext_popup {
                    if pid == &id {
                        ext_popup = None;
                    }
                }
                let removed = {
                    let mut st = state_for_event.lock_safe();
                    let opt = st
                        .chrome_extensions
                        .iter()
                        .position(|e| e.id == id)
                        .map(|pos| st.chrome_extensions.remove(pos));
                    if opt.is_some() {
                        chrome_ext::save_registry(&st.chrome_extensions);
                    }
                    (st.boot_chrome_ext_enabled, opt)
                };
                if let (boot_en, Some(entry)) = removed {
                    chrome_ext::remove_extension_files(&entry);
                    if boot_en {
                        if let Some(wv) = target_visible_webview(&windows, focused_win_id) {
                            chrome_ext::windows_bridge::remove_extension(
                                wv,
                                entry.id,
                                entry.runtime_id,
                                event_proxy_for_run.clone(),
                            );
                            return;
                        }
                    }
                    let _ = event_proxy_for_run.send_event(UserEvent::SyncConfig);
                }
            }
            Event::UserEvent(UserEvent::OpenChromeExtPopup { win_id, id }) => {
                if let Some((_, _, open_id)) = &ext_popup {
                    if open_id == &id {
                        ext_popup = None;
                        return;
                    }
                }
                ext_popup = None;
                let (boot_en, zh, found) = {
                    let st = state_for_event.lock_safe();
                    (
                        st.boot_chrome_ext_enabled,
                        !st.config.language.trim().eq_ignore_ascii_case("en"),
                        st.chrome_extensions.iter().find(|e| e.id == id).cloned(),
                    )
                };
                if !boot_en {
                    let msg = if zh {
                        "Chrome 扩展引擎未启用，请先在设置 -> 性能中启用并重启"
                    } else {
                        "Chrome Extensions engine is off — enable in Performance Settings and restart"
                    };
                    let _ = event_proxy_for_run.send_event(UserEvent::Toast(msg.to_string()));
                    return;
                }
                if let Some(entry) = found {
                    if !entry.enabled {
                        let msg = if zh {
                            format!("扩展「{}」已停用，请先启用", entry.name)
                        } else {
                            format!("Extension '{}' is disabled", entry.name)
                        };
                        let _ = event_proxy_for_run.send_event(UserEvent::Toast(msg));
                        return;
                    }
                    let host_id = if !entry.runtime_id.is_empty() {
                        entry.runtime_id.clone()
                    } else {
                        entry.id.clone()
                    };
                    if entry.popup_path.is_empty() {
                        if !entry.options_path.is_empty() {
                            let _ = event_proxy_for_run.send_event(UserEvent::OpenChromeExtOptions {
                                win_id,
                                id: entry.id,
                            });
                        } else {
                            let msg = if zh {
                                format!("「{}」在网页内直接生效，无独立弹窗", entry.name)
                            } else {
                                format!("'{}' runs automatically on pages (no popup UI)", entry.name)
                            };
                            let _ = event_proxy_for_run.send_event(UserEvent::Toast(msg));
                        }
                        return;
                    }
                    let popup_url = format!("chrome-extension://{}/{}", host_id, entry.popup_path);
                    #[allow(unused_mut)]
                    let mut p_builder = WindowBuilder::new()
                        .with_title(&entry.name)
                        .with_background_color((24, 24, 27, 255));
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    {
                        p_builder = p_builder
                            .with_decorations(true)
                            .with_resizable(true)
                            .with_always_on_top(true)
                            .with_inner_size(tao::dpi::LogicalSize::new(400.0, 560.0));
                        if let Some(bw) = windows.get(&win_id).or_else(|| windows.get(&focused_win_id)) {
                            if let Ok(pos) = bw.window.outer_position() {
                                let sf = bw.window.scale_factor().max(1.0);
                                let sz = bw.window.outer_size();
                                let lx = (pos.x as f64 + sz.width as f64) / sf - 430.0;
                                let ly = (pos.y as f64) / sf + 78.0;
                                p_builder = p_builder.with_position(
                                    tao::dpi::LogicalPosition::new(lx.max(20.0), ly.max(20.0)),
                                );
                            }
                        }
                    }
                    match p_builder.build(event_loop_target) {
                        Ok(p_win) => match build_extension_popup_webview(
                            &p_win,
                            &mut web_context,
                            &popup_url,
                            &state_for_event,
                            &event_proxy_for_run,
                            win_id,
                        ) {
                            Ok(p_wv) => {
                                p_win.set_focus();
                                ext_popup = Some((p_win, p_wv, entry.id));
                            }
                            Err(e) => {
                                let _ = event_proxy_for_run.send_event(UserEvent::Toast(if zh {
                                    format!("打开扩展弹窗失败：{}", e)
                                } else {
                                    format!("Failed opening popup: {}", e)
                                }));
                            }
                        },
                        Err(e) => {
                            let _ = event_proxy_for_run.send_event(UserEvent::Toast(if zh {
                                format!("创建扩展弹窗失败：{}", e)
                            } else {
                                format!("Popup window failed: {}", e)
                            }));
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::OpenChromeExtOptions { win_id, id }) => {
                let (boot_en, found) = {
                    let st = state_for_event.lock_safe();
                    (
                        st.boot_chrome_ext_enabled,
                        st.chrome_extensions.iter().find(|e| e.id == id).cloned(),
                    )
                };
                if !boot_en {
                    return;
                }
                if let Some(entry) = found {
                    let host_id = if !entry.runtime_id.is_empty() {
                        entry.runtime_id
                    } else {
                        entry.id
                    };
                    let page = if !entry.options_path.is_empty() {
                        entry.options_path
                    } else {
                        entry.popup_path
                    };
                    if page.is_empty() {
                        return;
                    }
                    let target_url = format!("chrome-extension://{}/{}", host_id, page);
                    if security::is_safe_chrome_extension_url(&target_url) {
                        let _ = event_proxy_for_run.send_event(UserEvent::OpenUrlInWindow {
                            win_id,
                            url: target_url,
                        });
                    }
                }
            }
            Event::UserEvent(UserEvent::CloseChromeExtPopup) => {
                ext_popup = None;
            }
            Event::UserEvent(UserEvent::OpenUrlInWindow { win_id, url }) => {
                let target_wid = if windows.contains_key(&win_id) {
                    Some(win_id)
                } else {
                    windows.keys().next().copied()
                };
                if let Some(actual_wid) = target_wid {
                    if let Some(bw) = windows.get_mut(&actual_wid) {
                        let (new_id, nav_url) = bw.tab_manager.lock_safe().new_tab(&url);
                        perform_switch_tab(
                            bw,
                            new_id,
                            &nav_url,
                            &mut web_context,
                            &state_for_event,
                            &reveal_scheduler,
                            &bridge_tok_js,
                        );
                        let _ = event_proxy_for_run.send_event(UserEvent::SyncTabs(actual_wid));
                    }
                }
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
                ..
            } => {
                if let Some((p_win, p_wv, _)) = &ext_popup {
                    if p_win.id() == window_id {
                        #[cfg(target_os = "windows")]
                        sync_webview_bounds(p_wv, size.width, size.height);
                        #[cfg(not(target_os = "windows"))]
                        let _ = (p_wv, size);
                        return;
                    }
                }
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
                            #[cfg(target_os = "windows")]
                            {
                                bw.recompute_dual_layout();
                            }
                            #[cfg(not(target_os = "windows"))]
                            {
                                bw.pool.resize_all(ContentBounds::default());
                            }
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
                event: WindowEvent::ScaleFactorChanged { .. },
                ..
            } => {
                #[cfg(target_os = "windows")]
                if let Some(&wid) = win_by_tao_id.get(&window_id) {
                    if let Some(bw) = windows.get_mut(&wid) {
                        bw.recompute_dual_layout();
                    }
                }
                #[cfg(not(target_os = "windows"))]
                let _ = window_id;
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Moved(_),
                ..
            } => {
                if let Some(&wid) = win_by_tao_id.get(&window_id) {
                    if let Some(bw) = windows.get(&wid) {
                        #[cfg(target_os = "windows")]
                        unsafe {
                            let _ = bw.hud_webview.controller().NotifyParentWindowPositionChanged();
                        }
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
                        if !bw.should_focus_content() {
                            #[cfg(target_os = "windows")]
                            let _ = bw.hud_webview.focus();
                        } else {
                            #[cfg(target_os = "windows")]
                            let hud_wants_focus = is_hud_taking_focus(&bw.hud_webview);
                            #[cfg(not(target_os = "windows"))]
                            let hud_wants_focus = false;
                            if !hud_wants_focus {
                                if let Some(vis) = bw.pool.visible_webview() {
                                    let _ = vis.focus();
                                    #[cfg(target_os = "windows")]
                                    {
                                        let _ = vis.set_memory_usage_level(MemoryUsageLevel::Normal);
                                    }
                                }
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
                            let (hud_json_res, page_json_res, zoom_f) = {
                                let st = state_for_event.lock_safe();
                                #[cfg(target_os = "windows")]
                                if let Some(vis) = b.pool.visible_webview() {
                                    adblock::sync_windows_custom_filters(vis, &st.adblock);
                                }
                                (
                                    serialize_runtime_for_win(&st, "fynexo://settings", b.incognito_window),
                                    serialize_runtime_for_win(&st, &aurl, b.incognito_window),
                                    effective_zoom_for_url(&st, &aurl),
                                )
                            };
                            #[cfg(target_os = "windows")]
                            if let Ok(hjson) = hud_json_res {
                                let hscript = format!(
                                    "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({}, {});",
                                    hjson, bridge_tok_js
                                );
                                let _ = b.hud_webview.evaluate_script(&hscript);
                            }
                            #[cfg(not(target_os = "windows"))]
                            let _ = hud_json_res;
                            if let Some(vis) = b.pool.visible_webview() {
                                let _ = vis.zoom(zoom_f);
                            }
                            if let Ok(json) = page_json_res {
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
                if let Some((p_win, _, _)) = &ext_popup {
                    if p_win.id() == window_id {
                        ext_popup = None;
                        return;
                    }
                }
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

fn target_hud_webview(
    windows: &HashMap<usize, BrowserWindow>,
    focused_win_id: usize,
) -> Option<&wry::WebView> {
    let bw = windows
        .get(&focused_win_id)
        .or_else(|| windows.values().next())?;
    #[cfg(target_os = "windows")]
    {
        Some(&bw.hud_webview)
    }
    #[cfg(not(target_os = "windows"))]
    {
        bw.pool.visible_webview()
    }
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
    let cbounds = bw.content_bounds_val();
    let hud_ptr = bw.hud_opt().map(|w| w as *const wry::WebView);
    let focus_tab = bw.should_focus_content();
    let old_id = bw.active_tab_id;
    if id == old_id && bw.pool.contains(id) {
        bw.pool.unpark_active(id, bridge_tok_js);
        if bw.pool.visible_id() != Some(id) && bw.pool.pending_reveal_id() != Some(id) {
            bw.pool.reveal_tab(
                id,
                cbounds,
                hud_ptr.map(|p| unsafe { &*p }),
                focus_tab,
                bridge_tok_js,
            );
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
            bw.pool.prepare_pending_behind(id, cbounds);
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
            #[cfg(target_os = "windows")]
            if let Ok(tabs_json) = serde_json::to_string(&tab_state) {
                let progress_js = if bw.loading_tabs.contains(&id) {
                    format!(
                        "window.__fynexo_trigger_progress__ && window.__fynexo_trigger_progress__({});",
                        bridge_tok_js
                    )
                } else {
                    format!(
                        "window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {});",
                        bridge_tok_js
                    )
                };
                let hud_js = format!(
                    "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({}, {}); {} {}",
                    tabs_json, bridge_tok_js, nav_js, progress_js
                );
                let _ = bw.hud_webview.evaluate_script(&hud_js);
            }
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
            bw.pool.reveal_tab(
                id,
                cbounds,
                hud_ptr.map(|p| unsafe { &*p }),
                focus_tab,
                bridge_tok_js,
            );
        }
    } else {
        let has_visible_backing = bw.pool.visible_id().is_some();
        let target = to_webview_url(url);
        let is_internal = is_internal_url(url);
        if is_internal {
            bw.loading_tabs.remove(&id);
        } else {
            bw.loading_tabs.insert(id);
        }

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
                let z = effective_zoom_for_url(&st, url);
                let _ = tv.webview.zoom(z);
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
                    init_bounds: cbounds,
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
            bw.pool.reveal_tab(
                id,
                cbounds,
                hud_ptr.map(|p| unsafe { &*p }),
                focus_tab,
                bridge_tok_js,
            );
            bw.sync_hud_loading_state(is_internal, bridge_tok_js);
        } else {
            let progress_js = if is_internal {
                format!(
                    "window.__fynexo_finish_progress__ && window.__fynexo_finish_progress__(true, {});",
                    bridge_tok_js
                )
            } else {
                format!(
                    "window.__fynexo_trigger_progress__ && window.__fynexo_trigger_progress__({});",
                    bridge_tok_js
                )
            };
            if let Ok(tabs_json) = serde_json::to_string(&tab_state) {
                let hud_js = format!(
                    "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({}, {}); {}",
                    tabs_json, bridge_tok_js, progress_js
                );
                bw.eval_hud(&hud_js);
            }
            if let Some(vis) = bw.pool.visible_webview() {
                sync_tabs_and_mute(vis, &tab_state, is_tab_muted, bridge_tok_js);
                let _ = vis.evaluate_script(&progress_js);
            }
            bw.pool.prepare_pending_behind(id, cbounds);
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
    bw.loading_tabs.retain(|tid| live_ids.contains(tid));
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
        #[cfg(target_os = "windows")]
        {
            let nav_js = nav_state_js(wv, bridge_tok_js);
            let _ = bw.hud_webview.evaluate_script(&nav_js);
            if !bw.loading_tabs.contains(&tab_id) {
                bw.sync_hud_loading_state(false, bridge_tok_js);
            }
        }
    }
    let cbounds = bw.content_bounds_val();
    let hud_ptr = bw.hud_opt().map(|w| w as *const wry::WebView);
    let focus_tab = bw.should_focus_content();
    bw.pool.reveal_tab(
        tab_id,
        cbounds,
        hud_ptr.map(|p| unsafe { &*p }),
        focus_tab,
        bridge_tok_js,
    );
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
        {
            let mut hpid: u32 = 0;
            unsafe {
                if bw.hud_webview.webview().BrowserProcessId(&mut hpid).is_ok()
                    && hpid != 0
                    && !browser_pids.contains(&hpid)
                {
                    browser_pids.push(hpid);
                }
            }
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
        let chrome_extensions: Vec<HudChromeExtension> = state
            .chrome_extensions
            .iter()
            .map(HudChromeExtension::from)
            .collect();
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
            chrome_extensions,
            chrome_ext_engine_active: state.boot_chrome_ext_enabled,
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
    let url_for_zoom = push_runtime_for_url.unwrap_or("");
    let zoom_factor = if url_for_zoom.is_empty() {
        state.config.default_zoom.clamp(0.5, 2.5)
    } else {
        effective_zoom_for_url(state, url_for_zoom)
    };
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

fn navigate_webview_history(bw: &BrowserWindow, forward: bool, bridge_tok_js: &str) {
    let Some(fg) = bw.pool.get_webview(bw.active_tab_id) else {
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
        bw.eval_hud_and_tabs(&script);
    }
}
