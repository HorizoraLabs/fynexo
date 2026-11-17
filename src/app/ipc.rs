use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;

use crate::adblock::{self, AdblockConfig};
#[cfg(target_os = "windows")]
use crate::config::data_dir;
use crate::config::{
    default_site_rules, delete_theme_preset, export_json_to_downloads, load_all_themes,
    save_theme_preset, themes_stamp, AppConfig, ComponentStyles, KeybindsConfig, SetupBundle,
    SiteRule, SiteRulesStore, ThemePreset, UserData,
};
use crate::tabs::TabManager;
use crate::util::LockExt;

use super::events::UserEvent;
use super::state::AppState;
use super::url::normalize_logical_url;

pub fn handle_ipc_message(
    source_win_id: usize,
    source_tab_id: usize,
    origin: &str,
    body: &str,
    app_state: &Arc<Mutex<AppState>>,
    tab_manager: &Arc<Mutex<TabManager>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(body) else {
        return;
    };

    if !super::security::authorize_ipc(origin, &val) {
        #[cfg(debug_assertions)]
        eprintln!("[IPC] rejected untrusted message from {}", origin);
        return;
    }

    match val["type"].as_str() {
        Some("BOOT_SYNC") => {
            let _ = proxy.send_event(UserEvent::BootSync {
                win_id: source_win_id,
                tab_id: source_tab_id,
            });
        }
        Some("TAB_PAINT_READY") => {
            let _ = proxy.send_event(UserEvent::TabPaintReady {
                win_id: source_win_id,
                tab_id: source_tab_id,
            });
        }
        Some("NAVIGATE") => {
            if let Some(raw_url) = val["url"].as_str() {
                let target_url = raw_url.trim();
                if target_url.is_empty() {
                    return;
                }
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
                    let lower_raw = target_url.to_ascii_lowercase();
                    if lower_raw.starts_with("javascript:")
                        || lower_raw.starts_with("data:")
                        || lower_raw.starts_with("vbscript:")
                        || lower_raw.starts_with("file:")
                    {
                        return;
                    }
                    format!("https://{}", target_url)
                } else {
                    normalize_logical_url(target_url)
                };
                if !super::security::is_safe_navigation_target(&formatted_url) {
                    return;
                }
                let is_active_source = {
                    let mut tm = tab_manager.lock_safe();
                    tm.update_tab_info(source_tab_id, None, Some(&formatted_url));
                    tm.active_id == source_tab_id
                };
                if is_active_source {
                    adblock::set_active_page_url(&formatted_url);
                    let _ = proxy.send_event(UserEvent::Navigate {
                        win_id: source_win_id,
                        url: formatted_url,
                    });
                }
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
            }
        }
        Some("NEW_WINDOW") => {
            let (is_multi, homepage) = {
                let st = app_state.lock_safe();
                (st.config.is_multi_window(), st.config.homepage.clone())
            };
            let want_incog = val["incognito"].as_bool().unwrap_or(false);
            let target_opt = val["url"]
                .as_str()
                .map(|s| normalize_logical_url(s.trim()))
                .filter(|u| !u.is_empty() && super::security::is_safe_navigation_target(u));

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if is_multi {
                let _ = proxy.send_event(UserEvent::NewWindow {
                    url: target_opt,
                    incognito: want_incog,
                });
                return;
            }
            let _ = (is_multi, want_incog);
            let fallback_url = target_opt.unwrap_or(homepage);
            adblock::set_active_page_url(&fallback_url);
            let (new_id, nav_url) = tab_manager.lock_safe().new_tab(&fallback_url);
            let _ = proxy.send_event(UserEvent::SwitchTab {
                win_id: source_win_id,
                id: new_id,
                url: nav_url,
            });
            let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
            let _ = proxy.send_event(UserEvent::Toast(
                "Single-window mode: opened in current window".to_string(),
            ));
        }
        Some("DETACH_TAB") => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let (is_multi, default_url) = {
                    let st = app_state.lock_safe();
                    (st.config.is_multi_window(), st.config.homepage.clone())
                };
                if !is_multi {
                    let _ = proxy.send_event(UserEvent::Toast(
                        "Switch to Multi-Window mode in Settings to detach tabs".to_string(),
                    ));
                    return;
                }
                let target_tid = val["id"]
                    .as_u64()
                    .map(|n| n as usize)
                    .unwrap_or(source_tab_id);
                let (tab_url, tab_incog, can_close) = {
                    let tm = tab_manager.lock_safe();
                    let url = tm.url_and_scroll_for(target_tid).0;
                    (url, tm.is_incognito(), tm.tabs.len() > 1)
                };
                if can_close {
                    let outcome =
                        tab_manager
                            .lock_safe()
                            .close_tab(target_tid, &default_url, true);
                    if let Ok(Some(next_url)) = outcome {
                        let next_id = tab_manager.lock_safe().active_id;
                        if next_id != target_tid {
                            let _ = proxy.send_event(UserEvent::SwitchTab {
                                win_id: source_win_id,
                                id: next_id,
                                url: next_url,
                            });
                        }
                    }
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                }
                let _ = proxy.send_event(UserEvent::NewWindow {
                    url: Some(tab_url),
                    incognito: tab_incog,
                });
            }
            #[cfg(any(target_os = "android", target_os = "ios"))]
            let _ = proxy.send_event(UserEvent::Toast(
                "Multi-window tab detach is desktop-only".to_string(),
            ));
        }
        Some("NEW_TAB") => {
            let homepage = app_state.lock_safe().config.homepage.clone();
            let raw_target = val["url"].as_str().unwrap_or(&homepage);
            let target = normalize_logical_url(raw_target);
            if !super::security::is_safe_navigation_target(&target) {
                return;
            }
            let link_title: Option<String> = val["title"].as_str().and_then(|raw| {
                let collapsed: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
                let trimmed = collapsed.trim();
                if trimmed.is_empty() {
                    None
                } else if trimmed.chars().count() > 120 {
                    Some(trimmed.chars().take(120).collect())
                } else {
                    Some(trimmed.to_string())
                }
            });
            let in_bg = val["background"].as_bool().unwrap_or(false);
            if in_bg {
                {
                    let mut tm = tab_manager.lock_safe();
                    let (new_id, _) = tm.new_tab_with_focus(&target, false);
                    if let Some(ref t) = link_title {
                        tm.update_tab_info(new_id, Some(t), None);
                    }
                }
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                let _ = proxy.send_event(UserEvent::Toast("Opened in background tab".to_string()));
            } else {
                adblock::set_active_page_url(&target);
                let (new_id, nav_url) = {
                    let mut tm = tab_manager.lock_safe();
                    let (id, url) = tm.new_tab(&target);
                    if let Some(ref t) = link_title {
                        tm.update_tab_info(id, Some(t), None);
                    }
                    (id, url)
                };
                let _ = proxy.send_event(UserEvent::SwitchTab {
                    win_id: source_win_id,
                    id: new_id,
                    url: nav_url,
                });
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
            }
        }
        Some("REORDER_TAB") => {
            if let (Some(from_id), Some(to_id)) =
                (val["from_id"].as_u64(), val["to_id"].as_u64())
            {
                if tab_manager
                    .lock_safe()
                    .reorder_tab(from_id as usize, to_id as usize)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                }
            }
        }
        Some("SWITCH_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                if let Some(target_url) =
                    tab_manager.lock_safe().switch_tab(id as usize)
                {
                    adblock::set_active_page_url(&target_url);
                    let _ = proxy.send_event(UserEvent::SwitchTab {
                        win_id: source_win_id,
                        id: id as usize,
                        url: target_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                }
            }
        }
        Some("TOGGLE_PIN_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                if let Some(pinned) =
                    tab_manager.lock_safe().toggle_pin_tab(id as usize)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::Toast(
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
                    tab_manager.lock_safe().toggle_background_play(id as usize)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::Toast(
                        if active {
                            "Background running active (Never resets)".to_string()
                        } else {
                            "Background running disabled".to_string()
                        },
                    ));
                    let _ = proxy.send_event(UserEvent::ToggleBackgroundPlay {
                        win_id: source_win_id,
                        id: id as usize,
                        active,
                    });
                }
            }
        }
        Some("TOGGLE_MUTE_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                if let Some(muted) =
                    tab_manager.lock_safe().toggle_mute_tab(id as usize)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::ToggleTabMute {
                        win_id: source_win_id,
                        id: id as usize,
                        muted,
                    });
                    let _ = proxy.send_event(UserEvent::Toast(
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
            let _ = proxy.send_event(UserEvent::RequestResourceUsage(source_win_id));
        }
        Some("FORCE_TRIM_MEMORY") => {
            let _ = proxy.send_event(UserEvent::TrimWorkingSet(source_win_id));
        }
        Some("SAVE_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                let clean_name: String = name.trim().chars().take(64).collect();
                if clean_name.is_empty() {
                    return;
                }
                tab_manager.lock_safe().save_workspace(&clean_name);
                let _ = proxy.send_event(UserEvent::SyncConfig);
                let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' saved", clean_name)));
            }
        }
        Some("LOAD_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                if let Some(target_url) = tab_manager.lock_safe().load_workspace(name) {
                    let next_id = tab_manager.lock_safe().active_id;
                    adblock::set_active_page_url(&target_url);
                    let _ = proxy.send_event(UserEvent::ResetWorkspace {
                        win_id: source_win_id,
                        active_id: next_id,
                        active_url: target_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' restored", name)));
                }
            }
        }
        Some("DELETE_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                if TabManager::delete_workspace(name) {
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' deleted", name)));
                }
            }
        }
        Some("CLOSE_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                let closed_id = id as usize;
                let force = val["force"].as_bool().unwrap_or(false);
                let default_url = app_state.lock_safe().config.homepage.clone();
                let outcome = tab_manager.lock_safe().close_tab(
                    closed_id,
                    &default_url,
                    force,
                );
                match outcome {
                    Ok(Some(next_url)) => {
                        let next_id = tab_manager.lock_safe().active_id;
                        adblock::set_active_page_url(&next_url);
                        if next_id == closed_id {
                            let _ = proxy.send_event(UserEvent::Navigate {
                                win_id: source_win_id,
                                url: next_url,
                            });
                        } else {
                            let _ = proxy.send_event(UserEvent::SwitchTab {
                                win_id: source_win_id,
                                id: next_id,
                                url: next_url,
                            });
                        }
                        let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    }
                    Ok(None) => {
                        let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    }
                    Err(msg) => {
                        let _ = proxy.send_event(UserEvent::Toast(msg.to_string()));
                    }
                }
            }
        }
        Some("DUPLICATE_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                if let Some((new_id, nav_url)) =
                    tab_manager.lock_safe().duplicate_tab(id as usize)
                {
                    adblock::set_active_page_url(&nav_url);
                    let _ = proxy.send_event(UserEvent::SwitchTab {
                        win_id: source_win_id,
                        id: new_id,
                        url: nav_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                }
            }
        }
        Some("CLOSE_OTHER_TABS") => {
            if let Some(id) = val["id"].as_u64() {
                let default_url = app_state.lock_safe().config.homepage.clone();
                let nav_opt = tab_manager
                    .lock_safe()
                    .close_other_tabs(id as usize, &default_url);
                if let Some(url) = nav_opt {
                    let next_id = tab_manager.lock_safe().active_id;
                    adblock::set_active_page_url(&url);
                    let _ = proxy.send_event(UserEvent::SwitchTab {
                        win_id: source_win_id,
                        id: next_id,
                        url,
                    });
                }
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
            }
        }
        Some("CLOSE_TABS_TO_RIGHT") => {
            if let Some(id) = val["id"].as_u64() {
                let default_url = app_state.lock_safe().config.homepage.clone();
                let nav_opt = tab_manager
                    .lock_safe()
                    .close_tabs_to_right(id as usize, &default_url);
                if let Some(url) = nav_opt {
                    let next_id = tab_manager.lock_safe().active_id;
                    adblock::set_active_page_url(&url);
                    let _ = proxy.send_event(UserEvent::SwitchTab {
                        win_id: source_win_id,
                        id: next_id,
                        url,
                    });
                }
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
            }
        }
        Some("CLOSE_ALL_TABS") => {
            let default_url = app_state.lock_safe().config.homepage.clone();
            let nav_url = tab_manager.lock_safe().close_all_tabs(&default_url);
            let next_id = tab_manager.lock_safe().active_id;
            adblock::set_active_page_url(&nav_url);
            let _ = proxy.send_event(UserEvent::SwitchTab {
                win_id: source_win_id,
                id: next_id,
                url: nav_url,
            });
            let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
        }
        Some("REOPEN_CLOSED_TAB") => {
            if let Some((url, _scroll_y)) =
                tab_manager.lock_safe().reopen_closed_tab()
            {
                let next_id = tab_manager.lock_safe().active_id;
                adblock::set_active_page_url(&url);
                let _ = proxy.send_event(UserEvent::SwitchTab {
                    win_id: source_win_id,
                    id: next_id,
                    url,
                });
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                let _ = proxy.send_event(UserEvent::Toast("Reopened closed tab".to_string()));
            } else {
                let _ = proxy.send_event(UserEvent::Toast(
                    "No recently closed tabs".to_string(),
                ));
            }
        }
        Some("UPDATE_TAB_TITLE") => {
            let title = val["title"].as_str();
            let norm_url = val["url"].as_str().map(normalize_logical_url);
            if matches!(norm_url.as_deref(), Some("about:blank") | Some("")) {
                return;
            }
            let (changed, is_active_source, tab_incog) = {
                let mut tm = tab_manager.lock_safe();
                let ch = tm.update_tab_info(source_tab_id, title, norm_url.as_deref());
                (ch, tm.active_id == source_tab_id, tm.is_incognito())
            };
            if is_active_source {
                if let Some(ref u) = norm_url {
                    adblock::set_active_page_url(u);
                }
            }
            if changed {
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                let _ = proxy.send_event(UserEvent::SyncNavState {
                    win_id: source_win_id,
                    tab_id: source_tab_id,
                });
            }
            if !tab_incog {
                if let (Some(t), Some(u)) = (title, norm_url.as_deref()) {
                    let mut st = app_state.lock_safe();
                    if !st.config.incognito {
                        st.userdata.record_visit(t, u);
                    }
                }
            }
        }
        Some("UPDATE_TAB_MEDIA_STATE") => {
            if let Some(playing) = val["playing"].as_bool() {
                if tab_manager
                    .lock_safe()
                    .set_tab_playing_audio(source_tab_id, playing)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                }
            }
        }
        Some("SAVE_SCROLL") => {
            if let Some(scroll_y) = val["scroll_y"].as_f64() {
                let mut tm = tab_manager.lock_safe();
                if tm.update_tab_scroll(source_tab_id, scroll_y) {
                    tm.save();
                }
            }
        }
        Some("TOGGLE_FULLSCREEN") => {
            let explicit = val.get("force").and_then(|v| v.as_bool());
            let _ = proxy.send_event(UserEvent::ToggleFullscreen {
                win_id: source_win_id,
                explicit,
            });
        }
        Some("SET_ZOOM") => {
            let new_zoom = {
                let mut locked = app_state.lock_safe();
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
            let _ = proxy.send_event(UserEvent::ApplyZoom(new_zoom));
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(format!(
                "Zoom: {}%",
                (new_zoom * 100.0).round() as i32
            )));
        }
        Some("CLEAR_SITE_DATA") => {
            let _ = proxy.send_event(UserEvent::ClearSiteData(source_win_id));
            let _ = proxy.send_event(UserEvent::Toast(
                "Cleared WebView cache & site data".to_string(),
            ));
        }
        Some("TOGGLE_BOOKMARK") => {
            if let Some(raw_url) = val["url"].as_str() {
                let norm_url = normalize_logical_url(raw_url);
                if !super::security::is_safe_bookmark_url(&norm_url) {
                    return;
                }
                let title = val["title"].as_str().unwrap_or(&norm_url);
                let mut locked = app_state.lock_safe();
                let added = locked.userdata.toggle_bookmark(title, &norm_url);
                locked.shortcuts_mtime = UserData::shortcuts_mtime();
                drop(locked);
                let _ = proxy.send_event(UserEvent::SyncConfig);
                let _ = proxy.send_event(UserEvent::Toast(
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
                let norm_url = normalize_logical_url(raw_url);
                if !super::security::is_safe_bookmark_url(&norm_url) {
                    return;
                }
                let title = val["title"].as_str().unwrap_or("");
                let mut locked = app_state.lock_safe();
                if locked.userdata.add_bookmark(title, &norm_url) {
                    locked.shortcuts_mtime = UserData::shortcuts_mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast("Shortcut added".to_string()));
                }
            }
        }
        Some("REMOVE_BOOKMARK") => {
            if let Some(raw_url) = val["url"].as_str() {
                let mut locked = app_state.lock_safe();
                if locked.userdata.remove_bookmark(raw_url) {
                    locked.shortcuts_mtime = UserData::shortcuts_mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast("Shortcut removed".to_string()));
                }
            }
        }
        Some("IMPORT_SHORTCUTS_JSON") => {
            if let Some(raw_json) = val["json"].as_str() {
                let merge = val["merge"].as_bool().unwrap_or(true);
                let mut locked = app_state.lock_safe();
                match locked.userdata.import_shortcuts(raw_json, merge) {
                    Ok(count) => {
                        locked.shortcuts_mtime = UserData::shortcuts_mtime();
                        drop(locked);
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(format!(
                            "Imported {} shortcuts",
                            count
                        )));
                    }
                    Err(err) => {
                        drop(locked);
                        let _ = proxy.send_event(UserEvent::Toast(err));
                    }
                }
            }
        }
        Some("EXPORT_SHORTCUTS") => {
            let json = app_state.lock_safe().userdata.shortcuts_json_pretty();
            export_json_with_toast(proxy, "fynexo_shortcuts.json", &json, "Exported");
        }
        Some("RESET_SHORTCUTS") => {
            let mut locked = app_state.lock_safe();
            locked.userdata.reset_shortcuts();
            locked.shortcuts_mtime = UserData::shortcuts_mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(
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
                                let mut locked = app_state.lock_safe();
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
                                let _ = proxy.send_event(UserEvent::SyncConfig);
                                let _ = proxy.send_event(UserEvent::Toast(
                                    format!("Imported theme '{}'", theme_name),
                                ));
                            }
                            Err(err) => {
                                let _ = proxy.send_event(UserEvent::Toast(
                                    format!("Failed to save theme: {}", err),
                                ));
                            }
                        }
                    }
                    Err(err) => {
                        let _ = proxy.send_event(UserEvent::Toast(err));
                    }
                }
            }
        }
        Some("DELETE_THEME") => {
            if let Some(id) = val["id"].as_str() {
                if delete_theme_preset(id) {
                    let mut locked = app_state.lock_safe();
                    if locked.config.theme.eq_ignore_ascii_case(id) {
                        locked.config.theme = "dark".to_string();
                        locked.config.save();
                        locked.config_mtime = AppConfig::mtime();
                    }
                    locked.themes = load_all_themes();
                    locked.themes_stamp = themes_stamp();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast(format!("Deleted {}.json", id)));
                }
            }
        }
        Some("EXPORT_THEME") => {
            let req_id = val["id"].as_str().unwrap_or("");
            let preset = {
                let locked = app_state.lock_safe();
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
                export_json_with_toast(proxy, &fname, &json, "Exported");
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
                    let mut locked = app_state.lock_safe();
                    locked.site_rules = new_rules;
                    locked.rules_mtime = SiteRulesStore::mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        Some("IMPORT_RULES_JSON") => {
            if let Some(raw_json) = val["json"].as_str() {
                match SiteRulesStore::parse_flexible(raw_json) {
                    Ok(incoming) => {
                        let count = incoming.len();
                        let mut locked = app_state.lock_safe();
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
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(format!(
                            "Imported {} site rules",
                            count
                        )));
                    }
                    Err(err) => {
                        let _ = proxy.send_event(UserEvent::Toast(err));
                    }
                }
            }
        }
        Some("EXPORT_RULES") => {
            let rules = app_state.lock_safe().site_rules.clone();
            if let Ok(json) = serde_json::to_string_pretty(&rules) {
                export_json_with_toast(proxy, "fynexo_rules.json", &json, "Exported");
            }
        }
        Some("RESET_RULES") => {
            let defs = default_site_rules();
            SiteRulesStore::save(&defs);
            let mut locked = app_state.lock_safe();
            locked.site_rules = defs;
            locked.rules_mtime = SiteRulesStore::mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(
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
                    let mut locked = app_state.lock_safe();
                    locked.adblock = new_adb;
                    locked.adblock_mtime = AdblockConfig::mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncAdblockFilters);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        Some("IMPORT_ADBLOCK_JSON") => {
            if let Some(raw_json) = val["json"].as_str() {
                match serde_json::from_str::<AdblockConfig>(raw_json) {
                    Ok(mut incoming) => {
                        incoming.sanitize();
                        let mut locked = app_state.lock_safe();
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
                        let _ = proxy.send_event(UserEvent::SyncAdblockFilters);
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(
                            "Imported adblock lists".to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(format!(
                            "Invalid adblock JSON: {}",
                            e
                        )));
                    }
                }
            }
        }
        Some("EXPORT_ADBLOCK") => {
            let adb = app_state.lock_safe().adblock.clone();
            if let Ok(json) = serde_json::to_string_pretty(&adb) {
                export_json_with_toast(proxy, "fynexo_adblock.json", &json, "Exported");
            }
        }
        Some("SAVE_KEYBINDS") => {
            if let Some(kb_val) = val.get("keybinds") {
                if let Ok(new_kb) =
                    serde_json::from_value::<KeybindsConfig>(kb_val.clone())
                {
                    new_kb.save();
                    let mut locked = app_state.lock_safe();
                    locked.keybinds = new_kb;
                    locked.keybinds_mtime = KeybindsConfig::mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        Some("IMPORT_KEYBINDS_JSON") => {
            if let Some(raw_json) = val["json"].as_str() {
                match serde_json::from_str::<KeybindsConfig>(raw_json) {
                    Ok(new_kb) => {
                        new_kb.save();
                        let mut locked = app_state.lock_safe();
                        locked.keybinds = new_kb;
                        locked.keybinds_mtime = KeybindsConfig::mtime();
                        drop(locked);
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(
                            "Imported keybinds".to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(format!(
                            "Invalid keybinds JSON: {}",
                            e
                        )));
                    }
                }
            }
        }
        Some("EXPORT_KEYBINDS") => {
            let kb = app_state.lock_safe().keybinds.clone();
            if let Ok(json) = serde_json::to_string_pretty(&kb) {
                export_json_with_toast(proxy, "fynexo_keybinds.json", &json, "Exported");
            }
        }
        Some("RESET_KEYBINDS") => {
            let def = KeybindsConfig::default();
            def.save();
            let mut locked = app_state.lock_safe();
            locked.keybinds = def;
            locked.keybinds_mtime = KeybindsConfig::mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(
                "Keybinds reset to defaults".to_string(),
            ));
        }
        Some("EXPORT_FULL_SETUP") => {
            let bundle = {
                let locked = app_state.lock_safe();
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
                export_json_with_toast(proxy, "fynexo-setup-bundle.json", &json, "Exported full setup");
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
                            let mut locked = app_state.lock_safe();
                            locked.userdata.bookmarks = bundle.shortcuts;
                            locked.userdata.save_shortcuts();
                            locked.force_reload();
                        }
                        tab_manager.lock_safe().set_incognito(incog);
                    let _ = proxy.send_event(UserEvent::SyncAutofill(!incog));

                        let _ = proxy.send_event(UserEvent::SyncAdblockFilters);
                        let _ = proxy.send_event(UserEvent::ApplyZoom(zoom));
                        let _ = proxy.send_event(UserEvent::UpdateUserAgent(ua));
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(
                            "Full Fynexo setup imported!".to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(format!(
                            "Invalid setup bundle JSON: {}",
                            e
                        )));
                    }
                }
            }
        }
        Some("CLEAR_HISTORY") => {
            let mut locked = app_state.lock_safe();
            locked.userdata.clear_history();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast("Browsing history cleared".to_string()));
        }
        Some("DELETE_HISTORY_ITEM") => {
            if let Some(u) = val["url"].as_str() {
                let removed = app_state.lock_safe().userdata.remove_history_item(u);
                if removed {
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        Some("OPEN_DATA_DIR") => {
            #[cfg(target_os = "windows")]
            adblock::open_folder_in_explorer(data_dir());
        }
        Some("OPEN_DOWNLOADS_DIR") => {
            #[cfg(target_os = "windows")]
            {
                let custom = app_state.lock_safe().config.downloads_dir.clone();
                let dir = crate::config::resolved_downloads_dir(&custom);
                adblock::open_folder_in_explorer(&dir);
            }
        }
        Some("OPEN_DOWNLOAD_FILE") => {
            if let Some(fname) = val["filename"].as_str() {
                #[cfg(target_os = "windows")]
                {
                    let custom = app_state.lock_safe().config.downloads_dir.clone();
                    let dir = crate::config::resolved_downloads_dir(&custom);
                    if !adblock::open_download_file(&dir, fname) {
                        let _ = proxy.send_event(UserEvent::Toast(
                            "File not found in Downloads".to_string(),
                        ));
                    }
                }
                #[cfg(not(target_os = "windows"))]
                let _ = fname;
            }
        }
        Some("GET_TABS") => {
            let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
        }
        Some("PATCH_HUD_PREFS") => {
            let mut config_dirty = false;
            let mut styles_dirty = false;
            let mut rules_dirty = false;
            let mut next_incognito: Option<bool> = None;

            {
                let mut locked = app_state.lock_safe();
                if let Some(v) = val["force_dark"].as_bool() {
                    if locked.config.force_dark != v {
                        locked.config.force_dark = v;
                        config_dirty = true;
                    }
                }
                if let Some(v) = val["adblock"].as_bool() {
                    if locked.config.adblock != v {
                        locked.config.adblock = v;
                        adblock::set_enabled(v);
                        config_dirty = true;
                    }
                }
                if let Some(v) = val["https_first"].as_bool() {
                    if locked.config.https_first != v {
                        locked.config.https_first = v;
                        config_dirty = true;
                    }
                }
                if let Some(v) = val["incognito"].as_bool() {
                    if locked.config.incognito != v {
                        locked.config.incognito = v;
                        next_incognito = Some(v);
                        config_dirty = true;
                    }
                }
                if let Some(raw_lang) = val["language"].as_str() {
                    let norm_lang = if raw_lang.trim().eq_ignore_ascii_case("en") {
                        "en"
                    } else {
                        "zh"
                    };
                    if locked.config.language != norm_lang {
                        locked.config.language = norm_lang.to_string();
                        config_dirty = true;
                    }
                }
                if let Some(raw_mode) = val["active_mode"].as_str() {
                    let norm_mode = raw_mode.trim().to_ascii_lowercase();
                    if !norm_mode.is_empty() && locked.config.active_mode != norm_mode {
                        locked.config.active_mode = norm_mode;
                        config_dirty = true;
                    }
                }
                if let Some(raw_wmode) = val["window_mode"].as_str() {
                    let norm_wm = if raw_wmode.trim().eq_ignore_ascii_case("single") {
                        "single"
                    } else {
                        "multi"
                    };
                    if locked.config.window_mode != norm_wm {
                        locked.config.window_mode = norm_wm.to_string();
                        config_dirty = true;
                    }
                }
                type ConfigBoolSlot = (&'static str, fn(&mut AppConfig) -> &mut bool);
                let ext_fields: [ConfigBoolSlot; 7] = [
                    ("ext_side_panel", |c| &mut c.ext_side_panel),
                    ("ext_reader_mode", |c| &mut c.ext_reader_mode),
                    ("ext_yt_ad_skipper", |c| &mut c.ext_yt_ad_skipper),
                    ("ext_gh_harmonizer", |c| &mut c.ext_gh_harmonizer),
                    ("ext_media_booster", |c| &mut c.ext_media_booster),
                    ("ext_userscript_engine", |c| &mut c.ext_userscript_engine),
                    ("ext_mouse_gestures", |c| &mut c.ext_mouse_gestures),
                ];
                for (key, slot) in ext_fields {
                    if let Some(v) = val[key].as_bool() {
                        let target = slot(&mut locked.config);
                        if *target != v {
                            *target = v;
                            config_dirty = true;
                        }
                    }
                }
                if let Some(ov_obj) = val.get("site_override").and_then(|v| v.as_object()) {
                    if let Some(raw_dom) = ov_obj.get("domain").and_then(|v| v.as_str()) {
                        let domain = raw_dom.trim().to_ascii_lowercase();
                        if !domain.is_empty() {
                            let mut entry = locked
                                .config
                                .site_overrides
                                .get(&domain)
                                .cloned()
                                .unwrap_or_default();
                            if let Some(zoom_val) = ov_obj.get("zoom") {
                                if zoom_val.is_null() {
                                    entry.zoom = None;
                                } else if let Some(z) = zoom_val.as_f64() {
                                    if z <= 0.0 || !z.is_finite() {
                                        entry.zoom = None;
                                    } else {
                                        entry.zoom =
                                            Some((z.clamp(0.5, 2.5) * 100.0).round() / 100.0);
                                    }
                                }
                            }
                            if let Some(dm) = ov_obj.get("dark_mode").and_then(|v| v.as_str()) {
                                let norm_dm = match dm.trim().to_ascii_lowercase().as_str() {
                                    "on" => "on",
                                    "off" => "off",
                                    _ => "auto",
                                };
                                entry.dark_mode = if norm_dm == "auto" {
                                    String::new()
                                } else {
                                    norm_dm.to_string()
                                };
                            }
                            let empty_dark =
                                entry.dark_mode.is_empty() || entry.dark_mode == "auto";
                            if entry.zoom.is_none() && empty_dark {
                                if locked.config.site_overrides.remove(&domain).is_some() {
                                    config_dirty = true;
                                }
                            } else if locked.config.site_overrides.get(&domain) != Some(&entry) {
                                locked.config.site_overrides.insert(domain, entry);
                                config_dirty = true;
                            }
                        }
                    }
                }
                if let Some(preset_raw) = val["perf_preset"].as_str() {
                    let norm_p = AppConfig::normalize_perf_preset(preset_raw);
                    if norm_p != "custom" {
                        locked.config.apply_perf_preset(norm_p);
                        config_dirty = true;
                    } else if locked.config.perf_preset != "custom" {
                        locked.config.perf_preset = "custom".to_string();
                        config_dirty = true;
                    }
                }
                let perf_bools: [ConfigBoolSlot; 5] = [
                    ("efficiency_mode", |c| &mut c.efficiency_mode),
                    ("auto_trim_ram", |c| &mut c.auto_trim_ram),
                    ("enforce_perf", |c| &mut c.enforce_perf),
                    ("reduce_animations", |c| &mut c.reduce_animations),
                    ("hw_accel", |c| &mut c.hw_accel),
                ];
                for (key, slot) in perf_bools {
                    if let Some(v) = val[key].as_bool() {
                        let target = slot(&mut locked.config);
                        if *target != v {
                            *target = v;
                            config_dirty = true;
                        }
                    }
                }
                if let Some(mins) = val["tab_sleep_minutes"].as_u64() {
                    let clamped = (mins as u32).min(240);
                    if locked.config.tab_sleep_minutes != clamped {
                        locked.config.tab_sleep_minutes = clamped;
                        config_dirty = true;
                    }
                }
                if let Some(mode_raw) = val["tab_cache_mode"].as_str() {
                    let norm = AppConfig::normalize_tab_cache_mode(mode_raw);
                    if locked.config.tab_cache_mode != norm {
                        locked.config.tab_cache_mode = norm.to_string();
                        config_dirty = true;
                    }
                }
                if let Some(engine_id) = val["search_engine"].as_str() {
                    if locked.config.search_engines.iter().any(|e| e.id == engine_id)
                        && locked.config.search_engine != engine_id
                    {
                        locked.config.search_engine = engine_id.to_string();
                        config_dirty = true;
                    }
                }
                if let Some(bb_raw) = val["bookmarks_bar"].as_str() {
                    let norm_bb = match bb_raw.trim().to_ascii_lowercase().as_str() {
                        "always" => Some("always"),
                        "hidden" => Some("hidden"),
                        "newtab" => Some("newtab"),
                        _ => None,
                    };
                    if let Some(bb) = norm_bb {
                        if locked.styles.topbar.bookmarks_bar != bb {
                            locked.styles.topbar.bookmarks_bar = bb.to_string();
                            styles_dirty = true;
                        }
                    }
                }
                let picked_rule_id = |dom: &str| -> String {
                    let slug: String = dom
                        .chars()
                        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
                        .collect();
                    format!("picked-{}", slug)
                };
                if let Some(rule_obj) = val.get("append_site_css_rule").and_then(|v| v.as_object()) {
                    let domain = rule_obj
                        .get("domain")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_ascii_lowercase()
                        .trim_start_matches("www.")
                        .to_string();
                    let selector = rule_obj
                        .get("selector")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    let valid_selector = !selector.is_empty()
                        && selector.len() <= 512
                        && !selector.contains('{')
                        && !selector.contains('}')
                        && !selector.contains(';')
                        && !selector.contains('\r')
                        && !selector.contains('\n');
                    if !domain.is_empty() && valid_selector {
                        let css_line = format!("{} {{ display: none !important; }}", selector);
                        let rule_id = picked_rule_id(&domain);
                        if let Some(existing) =
                            locked.site_rules.iter_mut().find(|r| r.id == rule_id)
                        {
                            if !existing.enabled {
                                existing.enabled = true;
                                rules_dirty = true;
                            }
                            if !existing.css.contains(&css_line) {
                                existing.css = if existing.css.trim().is_empty() {
                                    css_line
                                } else {
                                    format!("{}\n{}", existing.css.trim_end(), css_line)
                                };
                                rules_dirty = true;
                            }
                        } else {
                            locked.site_rules.push(SiteRule {
                                id: rule_id,
                                name: format!("Picked hidden elements ({})", domain),
                                domain,
                                enabled: true,
                                version: None,
                                css: css_line,
                                js: String::new(),
                            });
                            rules_dirty = true;
                        }
                    }
                }
                if let Some(raw_dom) = val["clear_picked_site_rules"].as_str() {
                    let domain = raw_dom
                        .trim()
                        .to_ascii_lowercase()
                        .trim_start_matches("www.")
                        .to_string();
                    if !domain.is_empty() {
                        let rule_id = picked_rule_id(&domain);
                        let before_len = locked.site_rules.len();
                        locked.site_rules.retain(|r| r.id != rule_id);
                        if locked.site_rules.len() != before_len {
                            rules_dirty = true;
                        }
                    }
                }
                if config_dirty {
                    locked.config.save();
                    locked.config_mtime = AppConfig::mtime();
                }
                if styles_dirty {
                    locked.styles.save();
                    locked.styles_mtime = ComponentStyles::mtime();
                }
                if rules_dirty {
                    SiteRulesStore::save(&locked.site_rules);
                    locked.rules_mtime = SiteRulesStore::mtime();
                }
            }

            if let Some(incog) = next_incognito {
                tab_manager.lock_safe().set_incognito(incog);
                let _ = proxy.send_event(UserEvent::SyncAutofill(!incog));
            }
            if config_dirty || styles_dirty || rules_dirty {
                let _ = proxy.send_event(UserEvent::SyncConfig);
            }
        }
        Some("SAVE_CONFIG") => {
            if let Some(cfg_val) = val.get("config") {
                if let Ok(mut new_cfg) =
                    serde_json::from_value::<AppConfig>(cfg_val.clone())
                {
                    new_cfg.default_zoom = new_cfg.default_zoom.clamp(0.5, 2.5);
                    new_cfg.sanitize_perf_fields();
                    adblock::set_enabled(new_cfg.adblock);
                    let zoom = new_cfg.default_zoom;
                    let ua = new_cfg.resolved_user_agent();
                    let incog = new_cfg.incognito;

                    let mut locked = app_state.lock_safe();
                    locked.config = new_cfg;
                    locked.config.save();
                    locked.config_mtime = AppConfig::mtime();
                    drop(locked);

                    tab_manager.lock_safe().set_incognito(incog);
                    let _ = proxy.send_event(UserEvent::SyncAutofill(!incog));
                    let _ = proxy.send_event(UserEvent::ApplyZoom(zoom));
                    let _ = proxy.send_event(UserEvent::UpdateUserAgent(ua));
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        Some("SAVE_STYLES") => {
            if let Some(sty_val) = val.get("styles") {
                if let Ok(new_styles) =
                    serde_json::from_value::<ComponentStyles>(sty_val.clone())
                {
                    let mut locked = app_state.lock_safe();
                    locked.styles = new_styles;
                    locked.styles.save();
                    locked.styles_mtime = ComponentStyles::mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        Some("RESET_STYLES") => {
            let mut locked = app_state.lock_safe();
            locked.styles = ComponentStyles::default();
            locked.styles.save();
            locked.styles_mtime = ComponentStyles::mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
        }
        Some("RELOAD_STYLES") => {
            let mut locked = app_state.lock_safe();
            locked.force_reload();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
        }
        Some("GET_CONFIG") => {
            let _ = proxy.send_event(UserEvent::SyncConfig);
        }
        Some("WINDOW_DRAG") => {
            let _ = proxy.send_event(UserEvent::WindowDrag(source_win_id));
        }
        Some("WINDOW_MINIMIZE") => {
            let _ = proxy.send_event(UserEvent::WindowMinimize(source_win_id));
        }
        Some("WINDOW_MAXIMIZE") => {
            let _ = proxy.send_event(UserEvent::WindowMaximize(source_win_id));
        }
        Some("WINDOW_CLOSE") => {
            let _ = proxy.send_event(UserEvent::WindowClose(source_win_id));
        }
        Some("GO_BACK") => {
            let _ = proxy.send_event(UserEvent::GoBack(source_win_id));
        }
        Some("GO_FORWARD") => {
            let _ = proxy.send_event(UserEvent::GoForward(source_win_id));
        }
        Some("SYNC_NAV_STATE") => {
            let _ = proxy.send_event(UserEvent::SyncNavState {
                win_id: source_win_id,
                tab_id: source_tab_id,
            });
        }
        Some("RELOAD") => {
            let _ = proxy.send_event(UserEvent::Reload(source_win_id));
        }
        Some("OPEN_DEVTOOLS") => {
            let _ = proxy.send_event(UserEvent::OpenDevtools(source_win_id));
        }
        Some("RELOAD_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                let _ = proxy.send_event(UserEvent::ReloadTab {
                    win_id: source_win_id,
                    id: id as usize,
                });
            }
        }
        _ => {}
    }
}

fn export_json_with_toast(
    proxy: &EventLoopProxy<UserEvent>,
    filename: &str,
    json: &str,
    action_label: &str,
) {
    match export_json_to_downloads(filename, json) {
        Ok(saved_path) => {
            #[cfg(target_os = "android")]
            let _ = proxy.send_event(UserEvent::EvalScript {
                win_id: 1,
                script: "if(window.__fynexo_android__&&typeof window.__fynexo_android__.flushPendingExports==='function'){window.__fynexo_android__.flushPendingExports();}".to_string(),
            });
            let name_only = saved_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(filename);
            let _ = proxy.send_event(UserEvent::DownloadUpdate {
                name: name_only.to_string(),
                status: "completed".to_string(),
            });
            let _ = proxy.send_event(UserEvent::Toast(format!(
                "{} {} to Downloads",
                action_label, name_only
            )));
        }
        Err(err) => {
            let _ = proxy.send_event(UserEvent::Toast(format!("Export failed: {}", err)));
        }
    }
}
