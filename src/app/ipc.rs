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
#[cfg(target_os = "windows")]
use super::memory::get_current_process_memory_mb;
use super::state::AppState;
use super::url::normalize_logical_url;

pub fn handle_ipc_message(
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
            let _ = proxy.send_event(UserEvent::BootSync(source_tab_id));
        }
        Some("TAB_PAINT_READY") => {
            let _ = proxy.send_event(UserEvent::TabPaintReady(source_tab_id));
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
                    let _ = proxy.send_event(UserEvent::Navigate(formatted_url));
                }
                let _ = proxy.send_event(UserEvent::SyncTabs);
            }
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
                let _ = proxy.send_event(UserEvent::SyncTabs);
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
                    id: new_id,
                    url: nav_url,
                });
                let _ = proxy.send_event(UserEvent::SyncTabs);
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
                    let _ = proxy.send_event(UserEvent::SyncTabs);
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
                        id: id as usize,
                        url: target_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs);
                }
            }
        }
        Some("TOGGLE_PIN_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                if let Some(pinned) =
                    tab_manager.lock_safe().toggle_pin_tab(id as usize)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs);
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
                    let _ = proxy.send_event(UserEvent::SyncTabs);
                    let _ = proxy.send_event(UserEvent::Toast(
                        if active {
                            "Background running active (Never resets)".to_string()
                        } else {
                            "Background running disabled".to_string()
                        },
                    ));
                    let _ = proxy.send_event(UserEvent::ToggleBackgroundPlay {
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
                    let _ = proxy.send_event(UserEvent::SyncTabs);
                    let _ = proxy.send_event(UserEvent::ToggleTabMute {
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
            #[cfg(target_os = "windows")]
            {
                let (ws_mb, commit_mb) = get_current_process_memory_mb();
                let btok = serde_json::to_string(super::security::bridge_token())
                    .unwrap_or_else(|_| "\"\"".to_string());
                let script = format!(
                    "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({{ host_mb: {:.1}, commit_mb: {:.1} }}, {});",
                    ws_mb, commit_mb, btok
                );
                let _ = proxy.send_event(UserEvent::EvalScript(script));
            }
        }
        Some("FORCE_TRIM_MEMORY") => {
            let _ = proxy.send_event(UserEvent::TrimWorkingSet);
        }
        Some("SAVE_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                let clean_name: String = name.trim().chars().take(64).collect();
                if clean_name.is_empty() {
                    return;
                }
                tab_manager.lock_safe().save_workspace(&clean_name);
                let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' saved", clean_name)));
            }
        }
        Some("LOAD_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                if let Some(target_url) = tab_manager.lock_safe().load_workspace(name) {
                    let next_id = tab_manager.lock_safe().active_id;
                    adblock::set_active_page_url(&target_url);
                    let _ = proxy.send_event(UserEvent::ResetWorkspace {
                        active_id: next_id,
                        active_url: target_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs);
                    let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' restored", name)));
                }
            }
        }
        Some("DELETE_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                if TabManager::delete_workspace(name) {
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
                            let _ = proxy.send_event(UserEvent::Navigate(next_url));
                        } else {
                            let _ = proxy.send_event(UserEvent::SwitchTab {
                                id: next_id,
                                url: next_url,
                            });
                        }
                        let _ = proxy.send_event(UserEvent::SyncTabs);
                    }
                    Ok(None) => {
                        let _ = proxy.send_event(UserEvent::SyncTabs);
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
                        id: new_id,
                        url: nav_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs);
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
                        id: next_id,
                        url,
                    });
                }
                let _ = proxy.send_event(UserEvent::SyncTabs);
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
                        id: next_id,
                        url,
                    });
                }
                let _ = proxy.send_event(UserEvent::SyncTabs);
            }
        }
        Some("CLOSE_ALL_TABS") => {
            let default_url = app_state.lock_safe().config.homepage.clone();
            let nav_url = tab_manager.lock_safe().close_all_tabs(&default_url);
            let next_id = tab_manager.lock_safe().active_id;
            adblock::set_active_page_url(&nav_url);
            let _ = proxy.send_event(UserEvent::SwitchTab {
                id: next_id,
                url: nav_url,
            });
            let _ = proxy.send_event(UserEvent::SyncTabs);
        }
        Some("REOPEN_CLOSED_TAB") => {
            if let Some((url, _scroll_y)) =
                tab_manager.lock_safe().reopen_closed_tab()
            {
                let next_id = tab_manager.lock_safe().active_id;
                adblock::set_active_page_url(&url);
                let _ = proxy.send_event(UserEvent::SwitchTab {
                    id: next_id,
                    url,
                });
                let _ = proxy.send_event(UserEvent::SyncTabs);
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
            let (changed, is_active_source) = {
                let mut tm = tab_manager.lock_safe();
                let ch = tm.update_tab_info(source_tab_id, title, norm_url.as_deref());
                (ch, tm.active_id == source_tab_id)
            };
            if is_active_source {
                if let Some(ref u) = norm_url {
                    adblock::set_active_page_url(u);
                }
            }
            if changed {
                let _ = proxy.send_event(UserEvent::SyncTabs);
                let _ = proxy.send_event(UserEvent::SyncNavState(source_tab_id));
            }
            if let (Some(t), Some(u)) = (title, norm_url.as_deref()) {
                let mut st = app_state.lock_safe();
                if !st.config.incognito {
                    st.userdata.record_visit(t, u);
                }
            }
        }
        Some("UPDATE_TAB_MEDIA_STATE") => {
            if let Some(playing) = val["playing"].as_bool() {
                if tab_manager
                    .lock_safe()
                    .set_tab_playing_audio(source_tab_id, playing)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs);
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
            let _ = proxy.send_event(UserEvent::ToggleFullscreen(explicit));
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
            let _ = proxy.send_event(UserEvent::ClearSiteData);
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
        Some("OPEN_DATA_DIR") => {
            #[cfg(target_os = "windows")]
            adblock::open_folder_in_explorer(data_dir());
        }
        Some("OPEN_DOWNLOADS_DIR") => {
            #[cfg(target_os = "windows")]
            adblock::open_folder_in_explorer(&crate::config::downloads_dir());
        }
        Some("OPEN_DOWNLOAD_FILE") => {
            if let Some(fname) = val["filename"].as_str() {
                #[cfg(target_os = "windows")]
                {
                    if !adblock::open_download_file(&crate::config::downloads_dir(), fname) {
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
            let _ = proxy.send_event(UserEvent::SyncTabs);
        }
        Some("PATCH_HUD_PREFS") => {
            let mut config_dirty = false;
            let mut styles_dirty = false;
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
                if let Some(v) = val["incognito"].as_bool() {
                    if locked.config.incognito != v {
                        locked.config.incognito = v;
                        next_incognito = Some(v);
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
                if config_dirty {
                    locked.config.save();
                    locked.config_mtime = AppConfig::mtime();
                }
                if styles_dirty {
                    locked.styles.save();
                    locked.styles_mtime = ComponentStyles::mtime();
                }
            }

            if let Some(incog) = next_incognito {
                tab_manager.lock_safe().set_incognito(incog);
            }
            if config_dirty || styles_dirty {
                let _ = proxy.send_event(UserEvent::SyncConfig);
            }
        }
        Some("SAVE_CONFIG") => {
            if let Some(cfg_val) = val.get("config") {
                if let Ok(mut new_cfg) =
                    serde_json::from_value::<AppConfig>(cfg_val.clone())
                {
                    new_cfg.default_zoom = new_cfg.default_zoom.clamp(0.5, 2.5);
                    new_cfg.tab_cache_mode =
                        AppConfig::normalize_tab_cache_mode(&new_cfg.tab_cache_mode).to_string();
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
            let _ = proxy.send_event(UserEvent::WindowDrag);
        }
        Some("WINDOW_MINIMIZE") => {
            let _ = proxy.send_event(UserEvent::WindowMinimize);
        }
        Some("WINDOW_MAXIMIZE") => {
            let _ = proxy.send_event(UserEvent::WindowMaximize);
        }
        Some("WINDOW_CLOSE") => {
            let _ = proxy.send_event(UserEvent::WindowClose);
        }
        Some("GO_BACK") => {
            let _ = proxy.send_event(UserEvent::GoBack);
        }
        Some("GO_FORWARD") => {
            let _ = proxy.send_event(UserEvent::GoForward);
        }
        Some("SYNC_NAV_STATE") => {
            let _ = proxy.send_event(UserEvent::SyncNavState(source_tab_id));
        }
        Some("RELOAD") => {
            let _ = proxy.send_event(UserEvent::Reload);
        }
        Some("OPEN_DEVTOOLS") => {
            let _ = proxy.send_event(UserEvent::OpenDevtools);
        }
        Some("RELOAD_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                let _ = proxy.send_event(UserEvent::ReloadTab(id as usize));
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
            let name_only = saved_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(filename);
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
