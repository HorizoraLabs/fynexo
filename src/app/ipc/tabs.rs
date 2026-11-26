use crate::adblock;
use crate::config::AppConfig;
use crate::tabs::TabManager;
use crate::util::LockExt;

use super::super::events::{HudInsetLogical, HudRegionRectLogical, UserEvent};
use super::super::security;
use super::super::url::normalize_logical_url;
use super::{is_zh_cfg, is_zh_lang, IpcCtx};

pub(super) fn try_handle(cmd: &str, val: &serde_json::Value, ctx: &IpcCtx<'_>) -> bool {
    let IpcCtx {
        source_win_id,
        source_tab_id,
        app_state,
        tab_manager,
        proxy,
    } = *ctx;
    match cmd {
        "BOOT_SYNC" => {
            let _ = proxy.send_event(UserEvent::BootSync {
                win_id: source_win_id,
                tab_id: source_tab_id,
            });
        }
        "TAB_PAINT_READY" => {
            if source_tab_id == 0 {
                return true;
            }
            let _ = proxy.send_event(UserEvent::TabPaintReady {
                win_id: source_win_id,
                tab_id: source_tab_id,
            });
        }
        "NAVIGATE" => {
            if let Some(raw_url) = val["url"].as_str() {
                let target_url = raw_url.trim();
                if target_url.is_empty() {
                    return true;
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
                        return true;
                    }
                    format!("https://{}", target_url)
                } else {
                    let https_first = app_state.lock_safe().config.https_first;
                    let norm = normalize_logical_url(target_url);
                    apply_https_first_if_needed(&norm, https_first)
                };
                if !security::is_safe_navigation_target(&formatted_url) {
                    return true;
                }
                let nav_url_with_frag = if formatted_url == "fynexo://settings" {
                    if let Some(hash_idx) = target_url.find('#') {
                        format!("fynexo://settings{}", &target_url[hash_idx..])
                    } else {
                        formatted_url.clone()
                    }
                } else {
                    formatted_url.clone()
                };
                let (is_active_source, reused_settings_id) = {
                    let mut tm = tab_manager.lock_safe();
                    let target_tid = if source_tab_id == 0 {
                        tm.active_id
                    } else {
                        source_tab_id
                    };
                    if formatted_url == "fynexo://settings" {
                        if let Some(existing_id) = tm.find_tab_by_logical_url("fynexo://settings") {
                            if existing_id != target_tid {
                                tm.switch_tab(existing_id);
                                (true, Some(existing_id))
                            } else {
                                tm.update_tab_info(target_tid, None, Some(&formatted_url));
                                (tm.active_id == target_tid, None)
                            }
                        } else {
                            tm.update_tab_info(target_tid, None, Some(&formatted_url));
                            (tm.active_id == target_tid, None)
                        }
                    } else {
                        tm.update_tab_info(target_tid, None, Some(&formatted_url));
                        (tm.active_id == target_tid, None)
                    }
                };
                if let Some(existing_id) = reused_settings_id {
                    adblock::set_active_page_url(&formatted_url);
                    let _ = proxy.send_event(UserEvent::SwitchTab {
                        win_id: source_win_id,
                        id: existing_id,
                        url: nav_url_with_frag,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    return true;
                }
                if is_active_source {
                    adblock::set_active_page_url(&formatted_url);
                    let _ = proxy.send_event(UserEvent::Navigate {
                        win_id: source_win_id,
                        url: nav_url_with_frag,
                    });
                }
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
            }
        }
        "NEW_WINDOW" => {
            let (is_multi, homepage, zh) = {
                let st = app_state.lock_safe();
                (
                    st.config.is_multi_window(),
                    st.config.homepage.clone(),
                    is_zh_lang(&st.config.language),
                )
            };
            let want_incog = val["incognito"].as_bool().unwrap_or(false);
            let target_opt = val["url"]
                .as_str()
                .map(|s| normalize_logical_url(s.trim()))
                .filter(|u| !u.is_empty() && security::is_safe_navigation_target(u));

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if is_multi {
                let _ = proxy.send_event(UserEvent::NewWindow {
                    url: target_opt,
                    incognito: want_incog,
                });
                return true;
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
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "单窗口模式：已在当前窗口打开".to_string()
            } else {
                "Single-window mode: opened in current window".to_string()
            }));
        }
        "DETACH_TAB" => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let (is_multi, default_url, zh) = {
                    let st = app_state.lock_safe();
                    (
                        st.config.is_multi_window(),
                        st.config.homepage.clone(),
                        is_zh_lang(&st.config.language),
                    )
                };
                if !is_multi {
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "请在设置中切换为多窗口模式后再分离标签页".to_string()
                    } else {
                        "Switch to Multi-Window mode in Settings to detach tabs".to_string()
                    }));
                    return true;
                }
                let target_tid = val["id"]
                    .as_u64()
                    .map(|n| n as usize)
                    .unwrap_or_else(|| {
                        if source_tab_id == 0 {
                            tab_manager.lock_safe().active_id
                        } else {
                            source_tab_id
                        }
                    });
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
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "多窗口分离标签页仅支持桌面端".to_string()
                } else {
                    "Multi-window tab detach is desktop-only".to_string()
                }));
            }
        }
        "NEW_TAB" => {
            let (homepage, https_first, zh) = {
                let st = app_state.lock_safe();
                (
                    st.config.homepage.clone(),
                    st.config.https_first,
                    is_zh_lang(&st.config.language),
                )
            };
            let raw_target = val["url"].as_str().unwrap_or(&homepage);
            let target =
                apply_https_first_if_needed(&normalize_logical_url(raw_target), https_first);
            if !security::is_safe_navigation_target(&target) {
                return true;
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
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "已在后台标签页打开".to_string()
                } else {
                    "Opened in background tab".to_string()
                }));
            } else {
                adblock::set_active_page_url(&target);
                let (target_id, nav_url) = {
                    let mut tm = tab_manager.lock_safe();
                    if target == "fynexo://settings" {
                        if let Some(existing_id) = tm.find_tab_by_logical_url("fynexo://settings") {
                            let url = tm
                                .switch_tab(existing_id)
                                .unwrap_or_else(|| target.clone());
                            (existing_id, url)
                        } else {
                            let (id, url) = tm.new_tab(&target);
                            if let Some(ref t) = link_title {
                                tm.update_tab_info(id, Some(t), None);
                            }
                            (id, url)
                        }
                    } else {
                        let (id, url) = tm.new_tab(&target);
                        if let Some(ref t) = link_title {
                            tm.update_tab_info(id, Some(t), None);
                        }
                        (id, url)
                    }
                };
                let switch_url = if nav_url == "fynexo://settings" {
                    if let Some(hash_idx) = raw_target.find('#') {
                        format!("fynexo://settings{}", &raw_target[hash_idx..])
                    } else {
                        nav_url
                    }
                } else {
                    nav_url
                };
                let _ = proxy.send_event(UserEvent::SwitchTab {
                    win_id: source_win_id,
                    id: target_id,
                    url: switch_url,
                });
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
            }
        }
        "REORDER_TAB" => {
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
        "SWITCH_TAB" => {
            if let Some(id) = val["id"].as_u64() {
                let target_url_opt = tab_manager.lock_safe().switch_tab(id as usize);
                if let Some(target_url) = target_url_opt {
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
        "TOGGLE_PIN_TAB" => {
            if let Some(id) = val["id"].as_u64() {
                let pinned_opt = tab_manager.lock_safe().toggle_pin_tab(id as usize);
                if let Some(pinned) = pinned_opt {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::Toast(
                        if pinned {
                            if zh { "已固定标签页".to_string() } else { "Tab pinned".to_string() }
                        } else {
                            if zh { "已取消固定标签页".to_string() } else { "Tab unpinned".to_string() }
                        },
                    ));
                }
            }
        }
        "TOGGLE_BACKGROUND_PLAY" => {
            if let Some(id) = val["id"].as_u64() {
                let active_opt = tab_manager.lock_safe().toggle_background_play(id as usize);
                if let Some(active) = active_opt {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::Toast(
                        if active {
                            if zh {
                                "后台保活已开启（永不休眠）".to_string()
                            } else {
                                "Background running active (Never resets)".to_string()
                            }
                        } else {
                            if zh {
                                "后台保活已关闭".to_string()
                            } else {
                                "Background running disabled".to_string()
                            }
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
        "TOGGLE_MUTE_TAB" => {
            if let Some(id) = val["id"].as_u64() {
                let muted_opt = tab_manager.lock_safe().toggle_mute_tab(id as usize);
                if let Some(muted) = muted_opt {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::ToggleTabMute {
                        win_id: source_win_id,
                        id: id as usize,
                        muted,
                    });
                    let _ = proxy.send_event(UserEvent::Toast(
                        if muted {
                            if zh { "已静音此标签页".to_string() } else { "Tab audio muted".to_string() }
                        } else {
                            if zh { "已取消静音".to_string() } else { "Tab audio unmuted".to_string() }
                        },
                    ));
                }
            }
        }
        "GET_RESOURCE_USAGE" => {
            let _ = proxy.send_event(UserEvent::RequestResourceUsage(source_win_id));
        }
        "FORCE_TRIM_MEMORY" => {
            let _ = proxy.send_event(UserEvent::TrimWorkingSet(source_win_id));
        }
        "SAVE_WORKSPACE" => {
            if let Some(name) = val["name"].as_str() {
                let clean_name: String = name.trim().chars().take(64).collect();
                if clean_name.is_empty() {
                    return true;
                }
                tab_manager.lock_safe().save_workspace(&clean_name);
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::SyncConfig);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    format!("已保存工作区「{}」", clean_name)
                } else {
                    format!("Workspace '{}' saved", clean_name)
                }));
            }
        }
        "LOAD_WORKSPACE" => {
            if let Some(name) = val["name"].as_str() {
                let loaded = {
                    let mut tm = tab_manager.lock_safe();
                    tm.load_workspace(name)
                        .map(|target_url| (tm.active_id, target_url))
                };
                if let Some((next_id, target_url)) = loaded {
                    let zh = is_zh_cfg(app_state);
                    adblock::set_active_page_url(&target_url);
                    let _ = proxy.send_event(UserEvent::ResetWorkspace {
                        win_id: source_win_id,
                        active_id: next_id,
                        active_url: target_url,
                    });
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        format!("已恢复工作区「{}」", name)
                    } else {
                        format!("Workspace '{}' restored", name)
                    }));
                }
            }
        }
        "DELETE_WORKSPACE" => {
            if let Some(name) = val["name"].as_str() {
                if TabManager::delete_workspace(name) {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        format!("已删除工作区「{}」", name)
                    } else {
                        format!("Workspace '{}' deleted", name)
                    }));
                }
            }
        }
        "CLOSE_TAB" => {
            if let Some(id) = val["id"].as_u64() {
                let closed_id = id as usize;
                let force = val["force"].as_bool().unwrap_or(false);
                let (default_url, zh) = {
                    let st = app_state.lock_safe();
                    (st.config.homepage.clone(), is_zh_lang(&st.config.language))
                };
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
                        let out = if zh && msg.contains("pinned") {
                            "该标签页已固定 — 请右键点击或点击图钉取消固定后再关闭".to_string()
                        } else {
                            msg.to_string()
                        };
                        let _ = proxy.send_event(UserEvent::Toast(out));
                    }
                }
            }
        }
        "DUPLICATE_TAB" => {
            if let Some(id) = val["id"].as_u64() {
                let dup_opt = tab_manager.lock_safe().duplicate_tab(id as usize);
                if let Some((new_id, nav_url)) = dup_opt {
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
        "CLOSE_OTHER_TABS" => {
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
        "CLOSE_TABS_TO_RIGHT" => {
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
        "CLOSE_ALL_TABS" => {
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
        "REOPEN_CLOSED_TAB" => {
            let reopened = {
                let mut tm = tab_manager.lock_safe();
                tm.reopen_closed_tab()
                    .map(|(url, _scroll_y)| (tm.active_id, url))
            };
            if let Some((next_id, url)) = reopened {
                let zh = is_zh_cfg(app_state);
                adblock::set_active_page_url(&url);
                let _ = proxy.send_event(UserEvent::SwitchTab {
                    win_id: source_win_id,
                    id: next_id,
                    url,
                });
                let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "已恢复关闭的标签页".to_string()
                } else {
                    "Reopened closed tab".to_string()
                }));
            } else {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "没有最近关闭的标签页".to_string()
                } else {
                    "No recently closed tabs".to_string()
                }));
            }
        }
        "UPDATE_TAB_TITLE" => {
            if source_tab_id == 0 {
                return true;
            }
            let title = val["title"].as_str();
            let norm_url = val["url"].as_str().map(normalize_logical_url);
            if matches!(norm_url.as_deref(), Some("about:blank") | Some("")) {
                return true;
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
        "UPDATE_TAB_MEDIA_STATE" => {
            if source_tab_id == 0 {
                return true;
            }
            if let Some(playing) = val["playing"].as_bool() {
                if tab_manager
                    .lock_safe()
                    .set_tab_playing_audio(source_tab_id, playing)
                {
                    let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
                }
            }
        }
        "SAVE_SCROLL" => {
            if source_tab_id == 0 {
                return true;
            }
            if let Some(scroll_y) = val["scroll_y"].as_f64() {
                let mut tm = tab_manager.lock_safe();
                if tm.update_tab_scroll(source_tab_id, scroll_y) {
                    tm.save();
                }
            }
        }
        "TOGGLE_FULLSCREEN" => {
            let explicit = val.get("force").and_then(|v| v.as_bool());
            let _ = proxy.send_event(UserEvent::ToggleFullscreen {
                win_id: source_win_id,
                explicit,
            });
        }
        "SET_ZOOM" => {
            let (new_zoom, zh) = {
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
                (clamped, is_zh_lang(&locked.config.language))
            };
            let _ = proxy.send_event(UserEvent::ApplyZoom(new_zoom));
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                format!("缩放：{}%", (new_zoom * 100.0).round() as i32)
            } else {
                format!("Zoom: {}%", (new_zoom * 100.0).round() as i32)
            }));
        }
        "CLEAR_SITE_DATA" => {
            let zh = is_zh_cfg(app_state);
            let _ = proxy.send_event(UserEvent::ClearSiteData(source_win_id));
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "已清除网页缓存与站点数据".to_string()
            } else {
                "Cleared WebView cache & site data".to_string()
            }));
        }
        "GET_TABS" => {
            let _ = proxy.send_event(UserEvent::SyncTabs(source_win_id));
        }
        "WINDOW_DRAG" => {
            let _ = proxy.send_event(UserEvent::WindowDrag(source_win_id));
        }
        "WINDOW_MINIMIZE" => {
            let _ = proxy.send_event(UserEvent::WindowMinimize(source_win_id));
        }
        "WINDOW_MAXIMIZE" => {
            let _ = proxy.send_event(UserEvent::WindowMaximize(source_win_id));
        }
        "WINDOW_CLOSE" => {
            let _ = proxy.send_event(UserEvent::WindowClose(source_win_id));
        }
        "GO_BACK" => {
            let _ = proxy.send_event(UserEvent::GoBack(source_win_id));
        }
        "GO_FORWARD" => {
            let _ = proxy.send_event(UserEvent::GoForward(source_win_id));
        }
        "SYNC_NAV_STATE" => {
            let tid = if source_tab_id == 0 {
                tab_manager.lock_safe().active_id
            } else {
                source_tab_id
            };
            let _ = proxy.send_event(UserEvent::SyncNavState {
                win_id: source_win_id,
                tab_id: tid,
            });
        }
        "RELOAD" => {
            let _ = proxy.send_event(UserEvent::Reload(source_win_id));
        }
        "OPEN_DEVTOOLS" => {
            let _ = proxy.send_event(UserEvent::OpenDevtools(source_win_id));
        }
        "RELOAD_TAB" => {
            if let Some(id) = val["id"].as_u64() {
                let _ = proxy.send_event(UserEvent::ReloadTab {
                    win_id: source_win_id,
                    id: id as usize,
                });
            }
        }
        "SYNC_HUD_LAYOUT" => {
            let ci = &val["content_inset"];
            let inset = HudInsetLogical {
                top: ci["top"].as_f64().unwrap_or(0.0).clamp(0.0, 2000.0),
                left: ci["left"].as_f64().unwrap_or(0.0).clamp(0.0, 2000.0),
                right: ci["right"].as_f64().unwrap_or(0.0).clamp(0.0, 2000.0),
                bottom: ci["bottom"].as_f64().unwrap_or(0.0).clamp(0.0, 2000.0),
            };
            let mut regions = Vec::new();
            if let Some(arr) = val["regions"].as_array() {
                for item in arr.iter().take(32) {
                    let w = item["w"].as_f64().unwrap_or(0.0);
                    let h = item["h"].as_f64().unwrap_or(0.0);
                    if w > 0.5 && h > 0.5 && w.is_finite() && h.is_finite() {
                        regions.push(HudRegionRectLogical {
                            x: item["x"].as_f64().unwrap_or(0.0),
                            y: item["y"].as_f64().unwrap_or(0.0),
                            w,
                            h,
                            r: item["r"].as_f64().unwrap_or(0.0).max(0.0),
                        });
                    }
                }
            }
            let has_overlay = val["has_overlay"].as_bool().unwrap_or(false);
            let _ = proxy.send_event(UserEvent::SyncHudLayout {
                win_id: source_win_id,
                inset,
                regions,
                has_overlay,
            });
        }
        "CONTENT_ACTION" => {
            if let Some(action) = val["action"].as_str() {
                let trimmed = action.trim();
                if !trimmed.is_empty() && trimmed.len() <= 64 {
                    let _ = proxy.send_event(UserEvent::ContentAction {
                        win_id: source_win_id,
                        action: trimmed.to_string(),
                        payload: val.get("payload").cloned().unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
        "HUD_ACTION" => {
            if let Some(action) = val["action"].as_str() {
                let trimmed = action.trim();
                if !trimmed.is_empty() && trimmed.len() <= 64 {
                    let _ = proxy.send_event(UserEvent::HudAction {
                        win_id: source_win_id,
                        action: trimmed.to_string(),
                    });
                }
            }
        }
        "CONTENT_POINTER_DOWN" => {
            let _ = proxy.send_event(UserEvent::ContentPointerDown {
                win_id: source_win_id,
            });
        }
        "CONTENT_PAGE_META" => {
            if source_tab_id == 0 {
                return true;
            }
            let _ = proxy.send_event(UserEvent::ContentPageMeta {
                win_id: source_win_id,
                tab_id: source_tab_id,
                meta: val.get("meta").cloned().unwrap_or(serde_json::Value::Null),
            });
        }
        _ => return false,
    }
    true
}

fn is_local_or_lan_host(host: &str) -> bool {
    let h = host.trim().to_ascii_lowercase();
    if h.is_empty()
        || h == "localhost"
        || h == "fynexo.localhost"
        || h == "0.0.0.0"
        || h == "::1"
        || h == "[::1]"
        || h.ends_with(".localhost")
        || h.ends_with(".local")
        || h.ends_with(".test")
        || h.starts_with("127.")
        || h.starts_with("10.")
        || h.starts_with("192.168.")
    {
        return true;
    }
    if let Some(rest) = h.strip_prefix("172.") {
        if let Some(oct2) = rest.split('.').next().and_then(|s| s.parse::<u8>().ok()) {
            if (16..=31).contains(&oct2) {
                return true;
            }
        }
    }
    false
}

fn apply_https_first_if_needed(url: &str, https_first: bool) -> String {
    if !https_first {
        return url.to_string();
    }
    let Some(rest) = url.strip_prefix("http://") else {
        return url.to_string();
    };
    let host = adblock::extract_host(url).unwrap_or("");
    if is_local_or_lan_host(host) {
        url.to_string()
    } else {
        format!("https://{}", rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_first_upgrades_public_http_and_keeps_local_dev_hosts() {
        assert_eq!(
            apply_https_first_if_needed("http://example.com/path?a=1", true),
            "https://example.com/path?a=1"
        );
        assert_eq!(
            apply_https_first_if_needed("http://example.com/path", false),
            "http://example.com/path"
        );
        assert_eq!(
            apply_https_first_if_needed("http://localhost:3000/api", true),
            "http://localhost:3000/api"
        );
        assert_eq!(
            apply_https_first_if_needed("http://127.0.0.1:8080/", true),
            "http://127.0.0.1:8080/"
        );
        assert_eq!(
            apply_https_first_if_needed("http://192.168.1.50/admin", true),
            "http://192.168.1.50/admin"
        );
        assert_eq!(
            apply_https_first_if_needed("http://172.20.0.2:5173/", true),
            "http://172.20.0.2:5173/"
        );
    }
}
