use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use wry::{PageLoadEvent, WebContext, WebViewBuilder};

use crate::adblock::{self, AdblockConfig};
use crate::config::{
    data_dir, data_dir_display, default_site_rules, delete_theme_preset, export_json_to_downloads,
    load_all_themes, save_theme_preset, themes_dir_display, themes_stamp, unique_download_path,
    AppConfig, ComponentStyles, KeybindsConfig, RuntimePayload, SetupBundle, SiteRule,
    SiteRulesStore, ThemeColors, ThemePreset, UserData,
};
use crate::hud;
use crate::pages;
use crate::tabs::TabManager;

#[cfg(target_os = "windows")]
use wry::{MemoryUsageLevel, WebViewBuilderExtWindows, WebViewExtWindows};

pub struct AppState {
    pub config: AppConfig,
    pub styles: ComponentStyles,
    pub themes: Vec<ThemePreset>,
    pub userdata: UserData,
    pub site_rules: Vec<SiteRule>,
    pub adblock: AdblockConfig,
    pub keybinds: KeybindsConfig,
    pub config_mtime: Option<SystemTime>,
    pub styles_mtime: Option<SystemTime>,
    pub shortcuts_mtime: Option<SystemTime>,
    pub rules_mtime: Option<SystemTime>,
    pub adblock_mtime: Option<SystemTime>,
    pub keybinds_mtime: Option<SystemTime>,
    pub themes_stamp: (usize, Option<SystemTime>),
}

impl AppState {
    pub fn load() -> Self {
        let config = AppConfig::load();
        adblock::set_enabled(config.adblock);
        let styles = ComponentStyles::load();
        let themes = load_all_themes();
        let userdata = UserData::load();
        let site_rules = SiteRulesStore::load();
        let adblock_cfg = AdblockConfig::load();
        adblock::sync_dynamic_lists(&adblock_cfg);
        let keybinds = KeybindsConfig::load();

        Self {
            config,
            styles,
            themes,
            userdata,
            site_rules,
            adblock: adblock_cfg,
            keybinds,
            config_mtime: AppConfig::mtime(),
            styles_mtime: ComponentStyles::mtime(),
            shortcuts_mtime: UserData::shortcuts_mtime(),
            rules_mtime: SiteRulesStore::mtime(),
            adblock_mtime: AdblockConfig::mtime(),
            keybinds_mtime: KeybindsConfig::mtime(),
            themes_stamp: themes_stamp(),
        }
    }

    pub fn resolve_active_theme(&self) -> ThemeColors {
        self.themes
            .iter()
            .find(|t| t.id.eq_ignore_ascii_case(&self.config.theme))
            .or_else(|| self.themes.iter().find(|t| t.id == "dark"))
            .or_else(|| self.themes.first())
            .map(|t| t.colors.clone())
            .unwrap_or_default()
    }

    pub fn reload_if_changed(&mut self) -> bool {
        let cur_cfg_mtime = AppConfig::mtime();
        let cur_sty_mtime = ComponentStyles::mtime();
        let cur_sc_mtime = UserData::shortcuts_mtime();
        let cur_rules_mtime = SiteRulesStore::mtime();
        let cur_adb_mtime = AdblockConfig::mtime();
        let cur_kb_mtime = KeybindsConfig::mtime();
        let cur_th_stamp = themes_stamp();
        let mut changed = false;

        if cur_cfg_mtime != self.config_mtime {
            self.config = AppConfig::load();
            adblock::set_enabled(self.config.adblock);
            self.config_mtime = AppConfig::mtime();
            changed = true;
        }
        if cur_sty_mtime != self.styles_mtime {
            self.styles = ComponentStyles::load();
            self.styles_mtime = ComponentStyles::mtime();
            changed = true;
        }
        if cur_sc_mtime != self.shortcuts_mtime {
            self.userdata.reload_shortcuts();
            self.shortcuts_mtime = UserData::shortcuts_mtime();
            changed = true;
        }
        if cur_rules_mtime != self.rules_mtime {
            self.site_rules = SiteRulesStore::load();
            self.rules_mtime = SiteRulesStore::mtime();
            changed = true;
        }
        if cur_adb_mtime != self.adblock_mtime {
            self.adblock = AdblockConfig::load();
            adblock::sync_dynamic_lists(&self.adblock);
            self.adblock_mtime = AdblockConfig::mtime();
            changed = true;
        }
        if cur_kb_mtime != self.keybinds_mtime {
            self.keybinds = KeybindsConfig::load();
            self.keybinds_mtime = KeybindsConfig::mtime();
            changed = true;
        }
        if cur_th_stamp != self.themes_stamp {
            self.themes = load_all_themes();
            self.themes_stamp = themes_stamp();
            changed = true;
        }
        changed
    }

    pub fn force_reload(&mut self) {
        self.config = AppConfig::load();
        adblock::set_enabled(self.config.adblock);
        self.styles = ComponentStyles::load();
        self.themes = load_all_themes();
        self.userdata = UserData::load();
        self.site_rules = SiteRulesStore::load();
        self.adblock = AdblockConfig::load();
        adblock::sync_dynamic_lists(&self.adblock);
        self.keybinds = KeybindsConfig::load();

        self.config_mtime = AppConfig::mtime();
        self.styles_mtime = ComponentStyles::mtime();
        self.shortcuts_mtime = UserData::shortcuts_mtime();
        self.rules_mtime = SiteRulesStore::mtime();
        self.adblock_mtime = AdblockConfig::mtime();
        self.keybinds_mtime = KeybindsConfig::mtime();
        self.themes_stamp = themes_stamp();
    }

    pub fn payload(&self) -> RuntimePayload {
        RuntimePayload {
            config: self.config.clone(),
            styles: self.styles.clone(),
            active_theme: self.resolve_active_theme(),
            themes: self.themes.clone(),
            themes_dir: themes_dir_display(),
            styles_path: ComponentStyles::styles_path_display(),
            shortcuts_path: UserData::shortcuts_path_display(),
            rules_path: SiteRulesStore::path_display(),
            adblock_path: AdblockConfig::path_display(),
            keybinds_path: KeybindsConfig::path_display(),
            data_dir: data_dir_display(),
            bookmarks: self.userdata.bookmarks.clone(),
            history: self.userdata.history.iter().take(40).cloned().collect(),
            site_rules: self.site_rules.clone(),
            adblock: self.adblock.clone(),
            keybinds: self.keybinds.clone(),
            blocked_ads: adblock::blocked_count(),
        }
    }
}

pub enum UserEvent {
    Navigate(String),
    SwitchTab { id: usize, url: String },
    ToggleBackgroundPlay { id: usize, active: bool },
    ToggleTabMute { id: usize, muted: bool },
    BootSync,
    SyncConfig,
    SyncTabs,
    RestoreScroll(f64),
    Toast(String),
    ToggleFullscreen(Option<bool>),
    ApplyZoom(f64),
    ClearSiteData,
    SyncAdblockFilters,
    UpdateUserAgent(Option<String>),
    WindowMinimize,
    WindowMaximize,
    WindowClose,
    WindowDrag,
    GoBack,
    GoForward,
    Reload,
    EvalScript(String),
    TrimWorkingSet,
}

#[cfg(target_os = "windows")]
fn get_current_process_memory_mb() -> (f64, f64) {
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let mut pmc = PROCESS_MEMORY_COUNTERS::default();
        let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, size).is_ok() {
            let working_set_mb = (pmc.WorkingSetSize as f64) / (1024.0 * 1024.0);
            let pagefile_mb = (pmc.PagefileUsage as f64) / (1024.0 * 1024.0);
            (working_set_mb, pagefile_mb)
        } else {
            (0.0, 0.0)
        }
    }
}

#[cfg(target_os = "windows")]
fn force_empty_working_set() -> f64 {
    use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
    use windows::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
    let (ws, _) = get_current_process_memory_mb();
    ws
}

#[inline]
fn to_webview_url(url: &str) -> String {
    let trimmed = url.trim();
    let internal_route = if trimmed.eq_ignore_ascii_case("fynexo://newtab")
        || trimmed.starts_with("fynexo://newtab")
        || trimmed.eq_ignore_ascii_case("http://fynexo.localhost/newtab")
    {
        Some("newtab")
    } else if trimmed.eq_ignore_ascii_case("fynexo://settings")
        || trimmed.starts_with("fynexo://settings")
        || trimmed.eq_ignore_ascii_case("http://fynexo.localhost/settings")
    {
        Some("settings")
    } else {
        None
    };

    if let Some(route) = internal_route {
        #[cfg(any(target_os = "windows", target_os = "android"))]
        {
            return format!("http://fynexo.localhost/{}", route);
        }
        #[cfg(not(any(target_os = "windows", target_os = "android")))]
        {
            return format!("fynexo://localhost/{}", route);
        }
    }
    trimmed.to_string()
}

#[inline]
fn normalize_logical_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("fynexo://settings")
        || trimmed.contains("fynexo.localhost/settings")
        || trimmed.contains("fynexo.settings")
    {
        "fynexo://settings".to_string()
    } else if trimmed.starts_with("fynexo://newtab")
        || trimmed.contains("fynexo.localhost/newtab")
        || trimmed.contains("fynexo.newtab")
    {
        "fynexo://newtab".to_string()
    } else {
        trimmed.to_string()
    }
}

fn build_tab_webview(
    window: &tao::window::Window,
    web_context: &mut WebContext,
    target_url: &str,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    event_proxy: &EventLoopProxy<UserEvent>,
) -> Result<wry::WebView, Box<dyn Error>> {
    let initial_payload = {
        let mut st = app_state.lock().unwrap();
        let _ = st.reload_if_changed();
        st.payload()
    };

    let mut builder = WebViewBuilder::new_with_web_context(web_context)
        .with_background_color((9, 9, 11, 255))
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
        let (new_id, nav_url) = tabs_for_new_window.lock().unwrap().new_tab(&logical);
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
            let scroll_y = tabs_for_load.lock().unwrap().active_scroll_y();
            if scroll_y > 24.0 {
                let _ = proxy_for_load.send_event(UserEvent::RestoreScroll(scroll_y));
            }
        }
    });

    // Custom URI Scheme: fynexo://newtab & fynexo://settings
    let state_for_proto = Arc::clone(app_state);
    builder = builder.with_custom_protocol("fynexo".to_string(), move |_id, req| {
        let uri = req.uri().to_string();
        let path = req.uri().path();
        let host = req.uri().host().unwrap_or("");

        let is_settings = uri.contains("settings")
            || host == "settings"
            || path.starts_with("/settings")
            || path == "settings";

        let is_newtab = !is_settings
            && (uri.contains("newtab")
                || host == "newtab"
                || path.starts_with("/newtab")
                || path == "newtab"
                || path == "/"
                || path.is_empty());

        if is_settings || is_newtab {
            let mut st = state_for_proto.lock().unwrap();
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

    let initial_tabs = tab_manager.lock().unwrap().get_state();
    let initial_script = hud::generate_hud_script(&initial_payload, Some(&initial_tabs));
    let initial_url = to_webview_url(target_url);

    let state_for_ipc = Arc::clone(app_state);
    let tabs_for_ipc = Arc::clone(tab_manager);
    let proxy_for_ipc = event_proxy.clone();

    let webview = builder
        .with_devtools(cfg!(debug_assertions))
        .with_initialization_script_for_main_only(&initial_script, true)
        .with_ipc_handler(move |req| {
            let body = req.body();

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                match val["type"].as_str() {
                    Some("BOOT_SYNC") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::BootSync);
                    }
                    Some("NAVIGATE") => {
                        if let Some(target_url) = val["url"].as_str() {
                            let formatted_url = if target_url.starts_with("fynexo://") {
                                normalize_logical_url(target_url)
                            } else if target_url.eq_ignore_ascii_case("fynexo:settings")
                                || target_url.eq_ignore_ascii_case("settings")
                                || target_url.eq_ignore_ascii_case("about:settings")
                            {
                                "fynexo://settings".to_string()
                            } else if target_url.eq_ignore_ascii_case("fynexo:newtab")
                                || target_url.eq_ignore_ascii_case("newtab")
                                || target_url.eq_ignore_ascii_case("about:newtab")
                            {
                                "fynexo://newtab".to_string()
                            } else if !target_url.starts_with("http://")
                                && !target_url.starts_with("https://")
                            {
                                format!("https://{}", target_url)
                            } else {
                                normalize_logical_url(target_url)
                            };
                            adblock::set_active_page_url(&formatted_url);
                            tabs_for_ipc
                                .lock()
                                .unwrap()
                                .update_active_info(None, Some(&formatted_url));
                            let _ = proxy_for_ipc.send_event(UserEvent::Navigate(formatted_url));
                            let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                        }
                    }
                    Some("NEW_TAB") => {
                        let homepage = state_for_ipc.lock().unwrap().config.homepage.clone();
                        let raw_target = val["url"].as_str().unwrap_or(&homepage);
                        let target = normalize_logical_url(raw_target);
                        adblock::set_active_page_url(&target);
                        let (new_id, nav_url) = tabs_for_ipc.lock().unwrap().new_tab(&target);
                        let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: new_id, url: nav_url });
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                    }
                    Some("SWITCH_TAB") => {
                        if let Some(id) = val["id"].as_u64() {
                            if let Some(target_url) =
                                tabs_for_ipc.lock().unwrap().switch_tab(id as usize)
                            {
                                adblock::set_active_page_url(&target_url);
                                let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: id as usize, url: target_url });
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                            }
                        }
                    }
                    Some("TOGGLE_PIN_TAB") => {
                        if let Some(id) = val["id"].as_u64() {
                            if let Some(pinned) =
                                tabs_for_ipc.lock().unwrap().toggle_pin_tab(id as usize)
                            {
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                    if pinned {
                                        "Tab pinned".to_string()
                                    } else {
                                        "Tab unpinned".to_string()
                                    },
                                ));
                            }
                        }
                    }
                    Some("TOGGLE_BACKGROUND_PLAY") => {
                        if let Some(id) = val["id"].as_u64() {
                            if let Some(active) =
                                tabs_for_ipc.lock().unwrap().toggle_background_play(id as usize)
                            {
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                    if active {
                                        "Background running active (Never resets)".to_string()
                                    } else {
                                        "Background running disabled".to_string()
                                    },
                                ));
                                let _ = proxy_for_ipc.send_event(UserEvent::ToggleBackgroundPlay {
                                    id: id as usize,
                                    active,
                                });
                            }
                        }
                    }
                    Some("TOGGLE_MUTE_TAB") => {
                        if let Some(id) = val["id"].as_u64() {
                            if let Some(muted) =
                                tabs_for_ipc.lock().unwrap().toggle_mute_tab(id as usize)
                            {
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                                let _ = proxy_for_ipc.send_event(UserEvent::ToggleTabMute {
                                    id: id as usize,
                                    muted,
                                });
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                    if muted {
                                        "Tab audio muted".to_string()
                                    } else {
                                        "Tab audio unmuted".to_string()
                                    },
                                ));
                            }
                        }
                    }
                    Some("GET_RESOURCE_USAGE") => {
                        #[cfg(target_os = "windows")]
                        {
                            let (ws_mb, commit_mb) = get_current_process_memory_mb();
                            let script = format!(
                                "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({{ host_mb: {:.1}, commit_mb: {:.1} }});",
                                ws_mb, commit_mb
                            );
                            let _ = proxy_for_ipc.send_event(UserEvent::EvalScript(script));
                        }
                    }
                    Some("FORCE_TRIM_MEMORY") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::TrimWorkingSet);
                    }
                    Some("SAVE_WORKSPACE") => {
                        if let Some(name) = val["name"].as_str() {
                            tabs_for_ipc.lock().unwrap().save_workspace(name);
                            let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!("Workspace '{}' saved", name)));
                            let workspaces = crate::tabs::TabManager::list_workspaces();
                            if let Ok(ws_json) = serde_json::to_string(&workspaces) {
                                let script = format!(
                                    "window.__fynexo_set_workspaces__ && window.__fynexo_set_workspaces__({});",
                                    ws_json
                                );
                                let _ = proxy_for_ipc.send_event(UserEvent::EvalScript(script));
                            }
                        }
                    }
                    Some("LOAD_WORKSPACE") => {
                        if let Some(name) = val["name"].as_str() {
                            if let Some(target_url) = tabs_for_ipc.lock().unwrap().load_workspace(name) {
                                let next_id = tabs_for_ipc.lock().unwrap().active_id;
                                adblock::set_active_page_url(&target_url);
                                let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: next_id, url: target_url });
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!("Workspace '{}' restored", name)));
                            }
                        }
                    }
                    Some("DELETE_WORKSPACE") => {
                        if let Some(name) = val["name"].as_str() {
                            let deleted = crate::tabs::TabManager::delete_workspace(name);
                            if deleted {
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!("Workspace '{}' deleted", name)));
                                let workspaces = crate::tabs::TabManager::list_workspaces();
                                if let Ok(ws_json) = serde_json::to_string(&workspaces) {
                                    let script = format!(
                                        "window.__fynexo_set_workspaces__ && window.__fynexo_set_workspaces__({});",
                                        ws_json
                                    );
                                    let _ = proxy_for_ipc.send_event(UserEvent::EvalScript(script));
                                }
                            }
                        }
                    }
                    Some("LIST_WORKSPACES") => {
                        let workspaces = crate::tabs::TabManager::list_workspaces();
                        if let Ok(ws_json) = serde_json::to_string(&workspaces) {
                            let script = format!(
                                "window.__fynexo_set_workspaces__ && window.__fynexo_set_workspaces__({});",
                                ws_json
                            );
                            let _ = proxy_for_ipc.send_event(UserEvent::EvalScript(script));
                        }
                    }
                    Some("CLOSE_TAB") => {
                        if let Some(id) = val["id"].as_u64() {
                            let force = val["force"].as_bool().unwrap_or(false);
                            let default_url = state_for_ipc.lock().unwrap().config.homepage.clone();
                            let outcome = tabs_for_ipc.lock().unwrap().close_tab(
                                id as usize,
                                &default_url,
                                force,
                            );
                            match outcome {
                                Ok(Some(next_url)) => {
                                    let next_id = tabs_for_ipc.lock().unwrap().active_id;
                                    adblock::set_active_page_url(&next_url);
                                    let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: next_id, url: next_url });
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                                }
                                Ok(None) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                                }
                                Err(msg) => {
                                    let _ = proxy_for_ipc
                                        .send_event(UserEvent::Toast(msg.to_string()));
                                }
                            }
                        }
                    }
                    Some("DUPLICATE_TAB") => {
                        if let Some(id) = val["id"].as_u64() {
                            if let Some((new_id, nav_url)) =
                                tabs_for_ipc.lock().unwrap().duplicate_tab(id as usize)
                            {
                                adblock::set_active_page_url(&nav_url);
                                let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: new_id, url: nav_url });
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                            }
                        }
                    }
                    Some("CLOSE_OTHER_TABS") => {
                        if let Some(id) = val["id"].as_u64() {
                            let default_url = state_for_ipc.lock().unwrap().config.homepage.clone();
                            let nav_opt = tabs_for_ipc
                                .lock()
                                .unwrap()
                                .close_other_tabs(id as usize, &default_url);
                            if let Some(url) = nav_opt {
                                let next_id = tabs_for_ipc.lock().unwrap().active_id;
                                adblock::set_active_page_url(&url);
                                let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: next_id, url });
                            }
                            let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                        }
                    }
                    Some("CLOSE_TABS_TO_RIGHT") => {
                        if let Some(id) = val["id"].as_u64() {
                            let default_url = state_for_ipc.lock().unwrap().config.homepage.clone();
                            let nav_opt = tabs_for_ipc
                                .lock()
                                .unwrap()
                                .close_tabs_to_right(id as usize, &default_url);
                            if let Some(url) = nav_opt {
                                let next_id = tabs_for_ipc.lock().unwrap().active_id;
                                adblock::set_active_page_url(&url);
                                let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: next_id, url });
                            }
                            let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                        }
                    }
                    Some("CLOSE_ALL_TABS") => {
                        let default_url = state_for_ipc.lock().unwrap().config.homepage.clone();
                        let nav_url = tabs_for_ipc.lock().unwrap().close_all_tabs(&default_url);
                        let next_id = tabs_for_ipc.lock().unwrap().active_id;
                        adblock::set_active_page_url(&nav_url);
                        let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: next_id, url: nav_url });
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                    }
                    Some("REOPEN_CLOSED_TAB") => {
                        if let Some((url, _scroll_y)) =
                            tabs_for_ipc.lock().unwrap().reopen_closed_tab()
                        {
                            let next_id = tabs_for_ipc.lock().unwrap().active_id;
                            adblock::set_active_page_url(&url);
                            let _ = proxy_for_ipc.send_event(UserEvent::SwitchTab { id: next_id, url });
                            let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                            let _ = proxy_for_ipc
                                .send_event(UserEvent::Toast("Reopened closed tab".to_string()));
                        } else {
                            let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                "No recently closed tabs".to_string(),
                            ));
                        }
                    }
                    Some("UPDATE_TAB_TITLE") => {
                        let title = val["title"].as_str();
                        let norm_url = val["url"].as_str().map(normalize_logical_url);
                        if let Some(ref u) = norm_url {
                            adblock::set_active_page_url(u);
                        }
                        if tabs_for_ipc
                            .lock()
                            .unwrap()
                            .update_active_info(title, norm_url.as_deref())
                        {
                            let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                        }
                        if let (Some(t), Some(u)) = (title, norm_url.as_deref()) {
                            let mut st = state_for_ipc.lock().unwrap();
                            if !st.config.incognito {
                                st.userdata.record_visit(t, u);
                            }
                        }
                    }
                    Some("SAVE_SCROLL") => {
                        if let Some(scroll_y) = val["scroll_y"].as_f64() {
                            let mut tm = tabs_for_ipc.lock().unwrap();
                            if tm.update_active_scroll(scroll_y) {
                                tm.save();
                            }
                        }
                    }
                    Some("TOGGLE_FULLSCREEN") => {
                        let explicit = val.get("force").and_then(|v| v.as_bool());
                        let _ = proxy_for_ipc.send_event(UserEvent::ToggleFullscreen(explicit));
                    }
                    Some("SET_ZOOM") => {
                        let new_zoom = {
                            let mut locked = state_for_ipc.lock().unwrap();
                            let target = if let Some(abs_z) = val["zoom"].as_f64() {
                                abs_z
                            } else if let Some(delta) = val["delta"].as_f64() {
                                locked.config.default_zoom + delta
                            } else {
                                1.0
                            };
                            let clamped = ((target * 100.0).round() / 100.0).clamp(0.5, 2.5);
                            locked.config.default_zoom = clamped;
                            locked.config.save();
                            locked.config_mtime = AppConfig::mtime();
                            clamped
                        };
                        let _ = proxy_for_ipc.send_event(UserEvent::ApplyZoom(new_zoom));
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                        let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                            "Zoom: {}%",
                            (new_zoom * 100.0).round() as i32
                        )));
                    }
                    Some("CLEAR_SITE_DATA") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::ClearSiteData);
                        let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                            "Cleared WebView cache & site data".to_string(),
                        ));
                    }
                    Some("TOGGLE_BOOKMARK") => {
                        if let Some(raw_url) = val["url"].as_str() {
                            let norm_url = normalize_logical_url(raw_url);
                            let title = val["title"].as_str().unwrap_or(&norm_url);
                            let mut locked = state_for_ipc.lock().unwrap();
                            let added = locked.userdata.toggle_bookmark(title, &norm_url);
                            locked.shortcuts_mtime = UserData::shortcuts_mtime();
                            drop(locked);
                            let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                            let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                if added {
                                    "Saved to shortcuts".to_string()
                                } else {
                                    "Removed shortcut".to_string()
                                },
                            ));
                        }
                    }
                    Some("ADD_BOOKMARK") => {
                        if let Some(raw_url) = val["url"].as_str() {
                            let title = val["title"].as_str().unwrap_or("");
                            let mut locked = state_for_ipc.lock().unwrap();
                            if locked.userdata.add_bookmark(title, raw_url) {
                                locked.shortcuts_mtime = UserData::shortcuts_mtime();
                                drop(locked);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                let _ = proxy_for_ipc
                                    .send_event(UserEvent::Toast("Shortcut added".to_string()));
                            }
                        }
                    }
                    Some("REMOVE_BOOKMARK") => {
                        if let Some(raw_url) = val["url"].as_str() {
                            let mut locked = state_for_ipc.lock().unwrap();
                            if locked.userdata.remove_bookmark(raw_url) {
                                locked.shortcuts_mtime = UserData::shortcuts_mtime();
                                drop(locked);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                let _ = proxy_for_ipc
                                    .send_event(UserEvent::Toast("Shortcut removed".to_string()));
                            }
                        }
                    }
                    Some("IMPORT_SHORTCUTS_JSON") => {
                        if let Some(raw_json) = val["json"].as_str() {
                            let merge = val["merge"].as_bool().unwrap_or(true);
                            let mut locked = state_for_ipc.lock().unwrap();
                            match locked.userdata.import_shortcuts(raw_json, merge) {
                                Ok(count) => {
                                    locked.shortcuts_mtime = UserData::shortcuts_mtime();
                                    drop(locked);
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Imported {} shortcuts",
                                        count
                                    )));
                                }
                                Err(err) => {
                                    drop(locked);
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(err));
                                }
                            }
                        }
                    }
                    Some("EXPORT_SHORTCUTS") => {
                        let json = state_for_ipc.lock().unwrap().userdata.shortcuts_json_pretty();
                        match export_json_to_downloads("fynexo_shortcuts.json", &json) {
                            Ok(saved_path) => {
                                let name_only = saved_path
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("fynexo_shortcuts.json");
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                    "Exported {} to Downloads",
                                    name_only
                                )));
                            }
                            Err(err) => {
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                    "Export failed: {}",
                                    err
                                )));
                            }
                        }
                    }
                    Some("RESET_SHORTCUTS") => {
                        let mut locked = state_for_ipc.lock().unwrap();
                        locked.userdata.reset_shortcuts();
                        locked.shortcuts_mtime = UserData::shortcuts_mtime();
                        drop(locked);
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                        let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                            "Shortcuts reset to defaults".to_string(),
                        ));
                    }
                    Some("IMPORT_THEME_JSON") => {
                        if let Some(raw_json) = val["json"].as_str() {
                            let fname = val["filename"].as_str().unwrap_or("custom.json");
                            let stem = std::path::Path::new(fname)
                                .file_stem()
                                .and_then(|s| s.to_str());
                            match ThemePreset::parse_flexible(raw_json, stem) {
                                Ok(mut preset) => {
                                    preset.builtin = false;
                                    match save_theme_preset(&preset) {
                                        Ok(saved_id) => {
                                            let theme_name = preset.name.clone();
                                            let mut locked = state_for_ipc.lock().unwrap();
                                            locked.config.theme = saved_id;
                                            locked.config.save();
                                            locked.config_mtime = AppConfig::mtime();
                                            if locked.styles.colors.use_custom_colors {
                                                locked.styles.colors.use_custom_colors = false;
                                                locked.styles.save();
                                                locked.styles_mtime = ComponentStyles::mtime();
                                            }
                                            locked.themes = load_all_themes();
                                            locked.themes_stamp = themes_stamp();
                                            drop(locked);
                                            let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                            let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                                format!("Imported theme '{}'", theme_name),
                                            ));
                                        }
                                        Err(err) => {
                                            let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                                format!("Failed to save theme: {}", err),
                                            ));
                                        }
                                    }
                                }
                                Err(err) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(err));
                                }
                            }
                        }
                    }
                    Some("DELETE_THEME") => {
                        if let Some(id) = val["id"].as_str() {
                            if delete_theme_preset(id) {
                                let mut locked = state_for_ipc.lock().unwrap();
                                if locked.config.theme.eq_ignore_ascii_case(id) {
                                    locked.config.theme = "dark".to_string();
                                    locked.config.save();
                                    locked.config_mtime = AppConfig::mtime();
                                }
                                locked.themes = load_all_themes();
                                locked.themes_stamp = themes_stamp();
                                drop(locked);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                let _ = proxy_for_ipc
                                    .send_event(UserEvent::Toast(format!("Deleted {}.json", id)));
                            }
                        }
                    }
                    Some("EXPORT_THEME") => {
                        let req_id = val["id"].as_str().unwrap_or("");
                        let preset = {
                            let locked = state_for_ipc.lock().unwrap();
                            let target = if req_id.is_empty() {
                                locked.config.theme.as_str()
                            } else {
                                req_id
                            };
                            locked
                                .themes
                                .iter()
                                .find(|t| t.id.eq_ignore_ascii_case(target))
                                .cloned()
                                .unwrap_or_default()
                        };
                        if let Ok(json) = serde_json::to_string_pretty(&preset) {
                            let fname = format!("{}.json", preset.id);
                            match export_json_to_downloads(&fname, &json) {
                                Ok(saved_path) => {
                                    let name_only = saved_path
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or(&fname);
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Exported {} to Downloads",
                                        name_only
                                    )));
                                }
                                Err(err) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Export failed: {}",
                                        err
                                    )));
                                }
                            }
                        }
                    }
                    Some("OPEN_THEMES_DIR") => {
                        #[cfg(target_os = "windows")]
                        {
                            let local = std::path::PathBuf::from("themes");
                            if local.is_dir() {
                                adblock::open_folder_in_explorer(&local);
                            } else {
                                adblock::open_folder_in_explorer(&crate::config::themes_dir());
                            }
                        }
                    }
                    Some("SAVE_RULES") => {
                        if let Some(rules_val) = val.get("rules") {
                            if let Ok(new_rules) =
                                serde_json::from_value::<Vec<SiteRule>>(rules_val.clone())
                            {
                                SiteRulesStore::save(&new_rules);
                                let mut locked = state_for_ipc.lock().unwrap();
                                locked.site_rules = new_rules;
                                locked.rules_mtime = SiteRulesStore::mtime();
                                drop(locked);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                            }
                        }
                    }
                    Some("IMPORT_RULES_JSON") => {
                        if let Some(raw_json) = val["json"].as_str() {
                            match SiteRulesStore::parse_flexible(raw_json) {
                                Ok(incoming) => {
                                    let count = incoming.len();
                                    let mut locked = state_for_ipc.lock().unwrap();
                                    for r in incoming {
                                        if let Some(existing) = locked
                                            .site_rules
                                            .iter_mut()
                                            .find(|x| x.id == r.id)
                                        {
                                            *existing = r;
                                        } else {
                                            locked.site_rules.push(r);
                                        }
                                    }
                                    SiteRulesStore::save(&locked.site_rules);
                                    locked.rules_mtime = SiteRulesStore::mtime();
                                    drop(locked);
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Imported {} site rules",
                                        count
                                    )));
                                }
                                Err(err) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(err));
                                }
                            }
                        }
                    }
                    Some("EXPORT_RULES") => {
                        let rules = state_for_ipc.lock().unwrap().site_rules.clone();
                        if let Ok(json) = serde_json::to_string_pretty(&rules) {
                            if let Ok(p) = export_json_to_downloads("fynexo_rules.json", &json) {
                                let name_only = p
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("fynexo_rules.json");
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                    "Exported {} to Downloads",
                                    name_only
                                )));
                            }
                        }
                    }
                    Some("RESET_RULES") => {
                        let defs = default_site_rules();
                        SiteRulesStore::save(&defs);
                        let mut locked = state_for_ipc.lock().unwrap();
                        locked.site_rules = defs;
                        locked.rules_mtime = SiteRulesStore::mtime();
                        drop(locked);
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                        let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                            "Site rules reset to defaults".to_string(),
                        ));
                    }
                    Some("SAVE_ADBLOCK") => {
                        if let Some(adb_val) = val.get("adblock") {
                            if let Ok(mut new_adb) =
                                serde_json::from_value::<AdblockConfig>(adb_val.clone())
                            {
                                new_adb.sanitize();
                                new_adb.save();
                                adblock::sync_dynamic_lists(&new_adb);
                                let mut locked = state_for_ipc.lock().unwrap();
                                locked.adblock = new_adb;
                                locked.adblock_mtime = AdblockConfig::mtime();
                                drop(locked);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncAdblockFilters);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                            }
                        }
                    }
                    Some("IMPORT_ADBLOCK_JSON") => {
                        if let Some(raw_json) = val["json"].as_str() {
                            match serde_json::from_str::<AdblockConfig>(raw_json) {
                                Ok(mut incoming) => {
                                    incoming.sanitize();
                                    let mut locked = state_for_ipc.lock().unwrap();
                                    for d in incoming.custom_blocked_domains {
                                        if !locked.adblock.custom_blocked_domains.contains(&d) {
                                            locked.adblock.custom_blocked_domains.push(d);
                                        }
                                    }
                                    for d in incoming.whitelisted_domains {
                                        if !locked.adblock.whitelisted_domains.contains(&d) {
                                            locked.adblock.whitelisted_domains.push(d);
                                        }
                                    }
                                    locked.adblock.save();
                                    adblock::sync_dynamic_lists(&locked.adblock);
                                    locked.adblock_mtime = AdblockConfig::mtime();
                                    drop(locked);
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncAdblockFilters);
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                        "Imported adblock lists".to_string(),
                                    ));
                                }
                                Err(e) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Invalid adblock JSON: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                    Some("EXPORT_ADBLOCK") => {
                        let adb = state_for_ipc.lock().unwrap().adblock.clone();
                        if let Ok(json) = serde_json::to_string_pretty(&adb) {
                            if let Ok(p) = export_json_to_downloads("fynexo_adblock.json", &json) {
                                let name_only = p
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("fynexo_adblock.json");
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                    "Exported {} to Downloads",
                                    name_only
                                )));
                            }
                        }
                    }
                    Some("SAVE_KEYBINDS") => {
                        if let Some(kb_val) = val.get("keybinds") {
                            if let Ok(new_kb) =
                                serde_json::from_value::<KeybindsConfig>(kb_val.clone())
                            {
                                new_kb.save();
                                let mut locked = state_for_ipc.lock().unwrap();
                                locked.keybinds = new_kb;
                                locked.keybinds_mtime = KeybindsConfig::mtime();
                                drop(locked);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                            }
                        }
                    }
                    Some("IMPORT_KEYBINDS_JSON") => {
                        if let Some(raw_json) = val["json"].as_str() {
                            match serde_json::from_str::<KeybindsConfig>(raw_json) {
                                Ok(new_kb) => {
                                    new_kb.save();
                                    let mut locked = state_for_ipc.lock().unwrap();
                                    locked.keybinds = new_kb;
                                    locked.keybinds_mtime = KeybindsConfig::mtime();
                                    drop(locked);
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                        "Imported keybinds".to_string(),
                                    ));
                                }
                                Err(e) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Invalid keybinds JSON: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                    Some("EXPORT_KEYBINDS") => {
                        let kb = state_for_ipc.lock().unwrap().keybinds.clone();
                        if let Ok(json) = serde_json::to_string_pretty(&kb) {
                            if let Ok(p) = export_json_to_downloads("fynexo_keybinds.json", &json) {
                                let name_only = p
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("fynexo_keybinds.json");
                                let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                    "Exported {} to Downloads",
                                    name_only
                                )));
                            }
                        }
                    }
                    Some("RESET_KEYBINDS") => {
                        let def = KeybindsConfig::default();
                        def.save();
                        let mut locked = state_for_ipc.lock().unwrap();
                        locked.keybinds = def;
                        locked.keybinds_mtime = KeybindsConfig::mtime();
                        drop(locked);
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                        let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                            "Keybinds reset to defaults".to_string(),
                        ));
                    }
                    Some("EXPORT_FULL_SETUP") => {
                        let bundle = {
                            let locked = state_for_ipc.lock().unwrap();
                            SetupBundle {
                                fynexo_bundle_version: 1,
                                config: locked.config.clone(),
                                styles: locked.styles.clone(),
                                custom_themes: locked
                                    .themes
                                    .iter()
                                    .filter(|t| !t.builtin)
                                    .cloned()
                                    .collect(),
                                shortcuts: locked.userdata.bookmarks.clone(),
                                site_rules: locked.site_rules.clone(),
                                adblock: locked.adblock.clone(),
                                keybinds: locked.keybinds.clone(),
                            }
                        };
                        if let Ok(json) = serde_json::to_string_pretty(&bundle) {
                            match export_json_to_downloads("fynexo-setup-bundle.json", &json) {
                                Ok(p) => {
                                    let name_only = p
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("fynexo-setup-bundle.json");
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Exported full setup ({}) to Downloads",
                                        name_only
                                    )));
                                }
                                Err(err) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Export failed: {}",
                                        err
                                    )));
                                }
                            }
                        }
                    }
                    Some("IMPORT_FULL_SETUP_JSON") => {
                        if let Some(raw_json) = val["json"].as_str() {
                            match serde_json::from_str::<SetupBundle>(raw_json) {
                                Ok(mut bundle) => {
                                    for mut th in bundle.custom_themes {
                                        th.builtin = false;
                                        let _ = save_theme_preset(&th);
                                    }
                                    bundle.adblock.sanitize();
                                    bundle.adblock.save();
                                    adblock::sync_dynamic_lists(&bundle.adblock);
                                    adblock::set_enabled(bundle.config.adblock);
                                    bundle.config.save();
                                    bundle.styles.save();
                                    bundle.keybinds.save();
                                    SiteRulesStore::save(&bundle.site_rules);

                                    let zoom = bundle.config.default_zoom.clamp(0.5, 2.5);
                                    let ua = bundle.config.resolved_user_agent();
                                    let incog = bundle.config.incognito;

                                    {
                                        let mut locked = state_for_ipc.lock().unwrap();
                                        locked.userdata.bookmarks = bundle.shortcuts;
                                        locked.userdata.save_shortcuts();
                                        locked.force_reload();
                                    }
                                    tabs_for_ipc.lock().unwrap().set_incognito(incog);

                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncAdblockFilters);
                                    let _ = proxy_for_ipc.send_event(UserEvent::ApplyZoom(zoom));
                                    let _ = proxy_for_ipc.send_event(UserEvent::UpdateUserAgent(ua));
                                    let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(
                                        "Full Fynexo setup imported!".to_string(),
                                    ));
                                }
                                Err(e) => {
                                    let _ = proxy_for_ipc.send_event(UserEvent::Toast(format!(
                                        "Invalid setup bundle JSON: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                    Some("CLEAR_HISTORY") => {
                        let mut locked = state_for_ipc.lock().unwrap();
                        locked.userdata.clear_history();
                        drop(locked);
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                        let _ = proxy_for_ipc
                            .send_event(UserEvent::Toast("Browsing history cleared".to_string()));
                    }
                    Some("OPEN_DATA_DIR") => {
                        #[cfg(target_os = "windows")]
                        adblock::open_folder_in_explorer(data_dir());
                    }
                    Some("OPEN_DOWNLOADS_DIR") => {
                        #[cfg(target_os = "windows")]
                        adblock::open_folder_in_explorer(&crate::config::downloads_dir());
                    }
                    Some("GET_TABS") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncTabs);
                    }
                    Some("SAVE_CONFIG") => {
                        if let Some(cfg_val) = val.get("config") {
                            if let Ok(mut new_cfg) =
                                serde_json::from_value::<AppConfig>(cfg_val.clone())
                            {
                                new_cfg.default_zoom = new_cfg.default_zoom.clamp(0.5, 2.5);
                                adblock::set_enabled(new_cfg.adblock);
                                let zoom = new_cfg.default_zoom;
                                let ua = new_cfg.resolved_user_agent();
                                let incog = new_cfg.incognito;

                                let mut locked = state_for_ipc.lock().unwrap();
                                locked.config = new_cfg;
                                locked.config.save();
                                locked.config_mtime = AppConfig::mtime();
                                drop(locked);

                                tabs_for_ipc.lock().unwrap().set_incognito(incog);
                                let _ = proxy_for_ipc.send_event(UserEvent::ApplyZoom(zoom));
                                let _ = proxy_for_ipc.send_event(UserEvent::UpdateUserAgent(ua));
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                            }
                        }
                    }
                    Some("SAVE_STYLES") => {
                        if let Some(sty_val) = val.get("styles") {
                            if let Ok(new_styles) =
                                serde_json::from_value::<ComponentStyles>(sty_val.clone())
                            {
                                let mut locked = state_for_ipc.lock().unwrap();
                                locked.styles = new_styles;
                                locked.styles.save();
                                locked.styles_mtime = ComponentStyles::mtime();
                                drop(locked);
                                let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                            }
                        }
                    }
                    Some("RESET_STYLES") => {
                        let mut locked = state_for_ipc.lock().unwrap();
                        locked.styles = ComponentStyles::default();
                        locked.styles.save();
                        locked.styles_mtime = ComponentStyles::mtime();
                        drop(locked);
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                    }
                    Some("RELOAD_STYLES") => {
                        let mut locked = state_for_ipc.lock().unwrap();
                        locked.force_reload();
                        drop(locked);
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                    }
                    Some("GET_CONFIG") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::SyncConfig);
                    }
                    Some("WINDOW_DRAG") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::WindowDrag);
                    }
                    Some("WINDOW_MINIMIZE") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::WindowMinimize);
                    }
                    Some("WINDOW_MAXIMIZE") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::WindowMaximize);
                    }
                    Some("WINDOW_CLOSE") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::WindowClose);
                    }
                    Some("GO_BACK") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::GoBack);
                    }
                    Some("GO_FORWARD") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::GoForward);
                    }
                    Some("RELOAD") => {
                        let _ = proxy_for_ipc.send_event(UserEvent::Reload);
                    }
                    _ => {}
                }
            }
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

pub fn run() -> Result<(), Box<dyn Error>> {
    let app_state = Arc::new(Mutex::new(AppState::load()));
    let initial_payload = app_state.lock().unwrap().payload();

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
    let active_logical = tab_manager.lock().unwrap().active_url();
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

    let initial_active_id = tab_manager.lock().unwrap().active_id;
    let mut active_tab_id = initial_active_id;
    let mut foreground_webview: Option<wry::WebView> = Some(initial_webview);
    let mut idle_normal_webview: Option<wry::WebView> = None;
    let mut background_webviews: std::collections::HashMap<usize, wry::WebView> = std::collections::HashMap::new();

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
                    let tm = tabs_for_event.lock().unwrap();
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
                    let tm = tabs_for_event.lock().unwrap();
                    let muted = tm.tabs.iter().find(|t| t.id == id).map(|t| t.muted).unwrap_or(false);
                    (tm.get_state(), muted)
                };
                if let Ok(tabs_json) = serde_json::to_string(&tab_state) {
                    let script = format!("window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({});", tabs_json);
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
                let mute_script = format!(
                    "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({});",
                    if is_tab_muted { "true" } else { "false" }
                );
                if let Some(ref fg) = foreground_webview {
                    let _ = fg.evaluate_script(&mute_script);
                }

                background_webviews.retain(|&tab_id, _| {
                    tabs_for_event.lock().unwrap().tabs.iter().any(|t| t.id == tab_id && t.background_play)
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
                    "window.__fnx_set_tab_muted__ && window.__fnx_set_tab_muted__({});",
                    if muted { "true" } else { "false" }
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
                let payload = state_for_event.lock().unwrap().payload();
                let (tab_state, scroll_y) = {
                    let tm = tabs_for_event.lock().unwrap();
                    (tm.get_state(), tm.active_scroll_y())
                };
                if let (Ok(rt_json), Ok(tabs_json)) = (
                    serde_json::to_string(&payload),
                    serde_json::to_string(&tab_state),
                ) {
                    let script = if scroll_y > 24.0 {
                        format!(
                            "window.__fynexo_boot_sync__ && window.__fynexo_boot_sync__({}, {}); window.__fynexo_restore_scroll__ && window.__fynexo_restore_scroll__({});",
                            rt_json, tabs_json, scroll_y
                        )
                    } else {
                        format!(
                            "window.__fynexo_boot_sync__ && window.__fynexo_boot_sync__({}, {});",
                            rt_json, tabs_json
                        )
                    };
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncConfig) => {
                let payload = state_for_event.lock().unwrap().payload();
                if let Ok(json) = serde_json::to_string(&payload) {
                    let script = format!(
                        "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({});",
                        json
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::SyncTabs) => {
                let state = tabs_for_event.lock().unwrap().get_state();
                if let Ok(json) = serde_json::to_string(&state) {
                    let script = format!(
                        "window.__fynexo_set_tabs__ && window.__fynexo_set_tabs__({});",
                        json
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::RestoreScroll(scroll_y)) => {
                if scroll_y > 24.0 {
                    let script = format!(
                        "window.__fynexo_restore_scroll__ && window.__fynexo_restore_scroll__({});",
                        scroll_y
                    );
                    if let Some(ref fg) = foreground_webview {
                        let _ = fg.evaluate_script(&script);
                    }
                }
            }
            Event::UserEvent(UserEvent::Toast(msg)) => {
                if let Ok(json_msg) = serde_json::to_string(&msg) {
                    let script = format!(
                        "window.__fynexo_toast__ && window.__fynexo_toast__({});",
                        json_msg
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
                    "window.__fynexo_set_fullscreen__ && window.__fynexo_set_fullscreen__({});",
                    if next_full { "true" } else { "false" }
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
                    let adb = state_for_event.lock().unwrap().adblock.clone();
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
                tabs_for_event.lock().unwrap().save();
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
                            "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({{ host_mb: {:.1}, commit_mb: {:.1} }}); window.__fynexo_toast__ && window.__fynexo_toast__('Working set trimmed ({:.1} MB)');",
                            ws, commit, ws
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
                let changed = state_for_event.lock().unwrap().reload_if_changed();
                if changed {
                    let payload = state_for_event.lock().unwrap().payload();
                    tabs_for_event
                        .lock()
                        .unwrap()
                        .set_incognito(payload.config.incognito);
                    #[cfg(target_os = "windows")]
                    if let Some(ref fg) = foreground_webview {
                        adblock::sync_windows_custom_filters(fg, &payload.adblock);
                    }
                    if let Ok(json) = serde_json::to_string(&payload) {
                        let script = format!(
                            "window.__fynexo_set_runtime__ && window.__fynexo_set_runtime__({});",
                            json
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
                tabs_for_event.lock().unwrap().save();
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });

    #[allow(unreachable_code)]
    Ok(())
}
