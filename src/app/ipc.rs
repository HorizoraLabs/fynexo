use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;

use crate::adblock::{self, AdblockConfig};
use crate::config::{
    data_dir, default_site_rules, delete_theme_preset, export_json_to_downloads,
    load_all_themes, save_theme_preset, themes_stamp, AppConfig, ComponentStyles,
    KeybindsConfig, SetupBundle, SiteRule, SiteRulesStore, ThemePreset, UserData,
};
use crate::tabs::TabManager;
use crate::util::LockExt;

use super::events::UserEvent;
use super::memory::get_current_process_memory_mb;
use super::state::AppState;
use super::url::normalize_logical_url;

pub fn handle_ipc_message(
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
            let _ = proxy.send_event(UserEvent::BootSync);
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
                tab_manager
                    .lock_safe()
                    .update_active_info(None, Some(&formatted_url));
                let _ = proxy.send_event(UserEvent::Navigate(formatted_url));
                let _ = proxy.send_event(UserEvent::SyncTabs);
            }
        }
        Some("NEW_TAB") => {
            let homepage = app_state.lock_safe().config.homepage.clone();
            let raw_target = val["url"].as_str().unwrap_or(&homepage);
            let target = normalize_logical_url(raw_target);
            adblock::set_active_page_url(&target);
            let (new_id, nav_url) = tab_manager.lock_safe().new_tab(&target);
            let _ = proxy.send_event(UserEvent::SwitchTab {
                id: new_id,
                url: nav_url,
            });
            let _ = proxy.send_event(UserEvent::SyncTabs);
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
                let script = format!(
                    "window.__fynexo_on_resource_stats__ && window.__fynexo_on_resource_stats__({{ host_mb: {:.1}, commit_mb: {:.1} }});",
                    ws_mb, commit_mb
                );
                let _ = proxy.send_event(UserEvent::EvalScript(script));
            }
        }
        Some("FORCE_TRIM_MEMORY") => {
            let _ = proxy.send_event(UserEvent::TrimWorkingSet);
        }
        Some("SAVE_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                tab_manager.lock_safe().save_workspace(name);
                let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' saved", name)));
                let workspaces = TabManager::list_workspaces();
                if let Ok(ws_json) = serde_json::to_string(&workspaces) {
                    let script = format!(
                        "window.__fynexo_set_workspaces__ && window.__fynexo_set_workspaces__({});",
                        ws_json
                    );
                    let _ = proxy.send_event(UserEvent::EvalScript(script));
                }
            }
        }
        Some("LOAD_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                if let Some(target_url) = tab_manager.lock_safe().load_workspace(name) {
                    let next_id = tab_manager.lock_safe().active_id;
                    adblock::set_active_page_url(&target_url);
                    let _ = proxy.send_event(UserEvent::SwitchTab {
                        id: next_id,
                        url: target_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs);
                    let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' restored", name)));
                }
            }
        }
        Some("DELETE_WORKSPACE") => {
            if let Some(name) = val["name"].as_str() {
                let deleted = TabManager::delete_workspace(name);
                if deleted {
                    let _ = proxy.send_event(UserEvent::Toast(format!("Workspace '{}' deleted", name)));
                    let workspaces = TabManager::list_workspaces();
                    if let Ok(ws_json) = serde_json::to_string(&workspaces) {
                        let script = format!(
                            "window.__fynexo_set_workspaces__ && window.__fynexo_set_workspaces__({});",
                            ws_json
                        );
                        let _ = proxy.send_event(UserEvent::EvalScript(script));
                    }
                }
            }
        }
        Some("LIST_WORKSPACES") => {
            let workspaces = TabManager::list_workspaces();
            if let Ok(ws_json) = serde_json::to_string(&workspaces) {
                let script = format!(
                    "window.__fynexo_set_workspaces__ && window.__fynexo_set_workspaces__({});",
                    ws_json
                );
                let _ = proxy.send_event(UserEvent::EvalScript(script));
            }
        }
        Some("CLOSE_TAB") => {
            if let Some(id) = val["id"].as_u64() {
                let force = val["force"].as_bool().unwrap_or(false);
                let default_url = app_state.lock_safe().config.homepage.clone();
                let outcome = tab_manager.lock_safe().close_tab(
                    id as usize,
                    &default_url,
                    force,
                );
                match outcome {
                    Ok(Some(next_url)) => {
                        let next_id = tab_manager.lock_safe().active_id;
                        adblock::set_active_page_url(&next_url);
                        let _ = proxy.send_event(UserEvent::SwitchTab {
                            id: next_id,
                            url: next_url,
                        });
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
            if let Some(ref u) = norm_url {
                adblock::set_active_page_url(u);
            }
            if tab_manager
                .lock_safe()
                .update_active_info(title, norm_url.as_deref())
            {
                let _ = proxy.send_event(UserEvent::SyncTabs);
            }
            if let (Some(t), Some(u)) = (title, norm_url.as_deref()) {
                let mut st = app_state.lock_safe();
                if !st.config.incognito {
                    st.userdata.record_visit(t, u);
                }
            }
        }
        Some("SAVE_SCROLL") => {
            if let Some(scroll_y) = val["scroll_y"].as_f64() {
                let mut tm = tab_manager.lock_safe();
                if tm.update_active_scroll(scroll_y) {
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
                let title = val["title"].as_str().unwrap_or("");
                let mut locked = app_state.lock_safe();
                if locked.userdata.add_bookmark(title, raw_url) {
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
            match export_json_to_downloads("fynexo_shortcuts.json", &json) {
                Ok(saved_path) => {
                    let name_only = saved_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("fynexo_shortcuts.json");
                    let _ = proxy.send_event(UserEvent::Toast(format!(
                        "Exported {} to Downloads",
                        name_only
                    )));
                }
                Err(err) => {
                    let _ = proxy.send_event(UserEvent::Toast(format!(
                        "Export failed: {}",
                        err
                    )));
                }
            }
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
                match export_json_to_downloads(&fname, &json) {
                    Ok(saved_path) => {
                        let name_only = saved_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or(&fname);
                        let _ = proxy.send_event(UserEvent::Toast(format!(
                            "Exported {} to Downloads",
                            name_only
                        )));
                    }
                    Err(err) => {
                        let _ = proxy.send_event(UserEvent::Toast(format!(
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
                if let Ok(p) = export_json_to_downloads("fynexo_rules.json", &json) {
                    let name_only = p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("fynexo_rules.json");
                    let _ = proxy.send_event(UserEvent::Toast(format!(
                        "Exported {} to Downloads",
                        name_only
                    )));
                }
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
                if let Ok(p) = export_json_to_downloads("fynexo_adblock.json", &json) {
                    let name_only = p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("fynexo_adblock.json");
                    let _ = proxy.send_event(UserEvent::Toast(format!(
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
                if let Ok(p) = export_json_to_downloads("fynexo_keybinds.json", &json) {
                    let name_only = p
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("fynexo_keybinds.json");
                    let _ = proxy.send_event(UserEvent::Toast(format!(
                        "Exported {} to Downloads",
                        name_only
                    )));
                }
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
                match export_json_to_downloads("fynexo-setup-bundle.json", &json) {
                    Ok(p) => {
                        let name_only = p
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("fynexo-setup-bundle.json");
                        let _ = proxy.send_event(UserEvent::Toast(format!(
                            "Exported full setup ({}) to Downloads",
                            name_only
                        )));
                    }
                    Err(err) => {
                        let _ = proxy.send_event(UserEvent::Toast(format!(
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
        Some("GET_TABS") => {
            let _ = proxy.send_event(UserEvent::SyncTabs);
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
        Some("RELOAD") => {
            let _ = proxy.send_event(UserEvent::Reload);
        }
        _ => {}
    }
}
