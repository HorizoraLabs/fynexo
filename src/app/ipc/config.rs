use crate::adblock;
#[cfg(target_os = "windows")]
use crate::config::data_dir;
use crate::config::{
    create_profile, delete_profile, delete_theme_preset, launch_profile_process, load_all_themes,
    load_profiles, save_theme_preset, themes_stamp, AppConfig, ComponentStyles, KeybindsConfig,
    SetupBundle, SitePermissionsStore, SiteRule, SiteRulesStore, ThemePreset, UserData,
};
use crate::util::LockExt;

use super::super::events::UserEvent;
use super::super::security;
use super::super::url::normalize_logical_url;
use super::{export_json_with_toast, is_zh_cfg, is_zh_lang, IpcCtx};

pub(super) fn try_handle(cmd: &str, val: &serde_json::Value, ctx: &IpcCtx<'_>) -> bool {
    let IpcCtx {
        source_win_id,
        source_tab_id: _,
        app_state,
        tab_manager,
        proxy,
    } = *ctx;
    match cmd {
        "TOGGLE_BOOKMARK" => {
            if let Some(raw_url) = val["url"].as_str() {
                let norm_url = normalize_logical_url(raw_url);
                if !security::is_safe_bookmark_url(&norm_url) {
                    return true;
                }
                let title = val["title"].as_str().unwrap_or(&norm_url);
                let folder = val["folder"].as_str().unwrap_or("");
                let mut locked = app_state.lock_safe();
                let zh = is_zh_lang(&locked.config.language);
                let added = locked
                    .userdata
                    .toggle_bookmark_in_folder(title, &norm_url, folder);
                locked.shortcuts_mtime = UserData::shortcuts_mtime();
                drop(locked);
                let _ = proxy.send_event(UserEvent::SyncConfig);
                let _ = proxy.send_event(UserEvent::Toast(
                    if added {
                        if zh { "已保存到快捷方式".to_string() } else { "Saved to shortcuts".to_string() }
                    } else {
                        if zh { "已移除快捷方式".to_string() } else { "Removed shortcut".to_string() }
                    },
                ));
            }
        }
        "ADD_BOOKMARK" => {
            if let Some(raw_url) = val["url"].as_str() {
                let norm_url = normalize_logical_url(raw_url);
                if !security::is_safe_bookmark_url(&norm_url) {
                    return true;
                }
                let title = val["title"].as_str().unwrap_or("");
                let folder = val["folder"].as_str().unwrap_or("");
                let mut locked = app_state.lock_safe();
                let zh = is_zh_lang(&locked.config.language);
                if locked
                    .userdata
                    .add_bookmark_in_folder(title, &norm_url, folder)
                {
                    locked.shortcuts_mtime = UserData::shortcuts_mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "已添加快捷方式".to_string()
                    } else {
                        "Shortcut added".to_string()
                    }));
                }
            }
        }
        "SET_BOOKMARK_FOLDER" => {
            if let Some(raw_url) = val["url"].as_str() {
                let folder = val["folder"].as_str().unwrap_or("");
                let mut locked = app_state.lock_safe();
                if locked.userdata.set_bookmark_folder(raw_url, folder) {
                    locked.shortcuts_mtime = UserData::shortcuts_mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "EDIT_BOOKMARK" => {
            if let (Some(orig_url), Some(new_url)) =
                (val["orig_url"].as_str(), val["url"].as_str())
            {
                let norm_new = normalize_logical_url(new_url);
                if !security::is_safe_bookmark_url(&norm_new) {
                    return true;
                }
                let title = val["title"].as_str().unwrap_or("");
                let folder = val["folder"].as_str().unwrap_or("");
                let mut locked = app_state.lock_safe();
                let zh = is_zh_lang(&locked.config.language);
                if locked
                    .userdata
                    .edit_bookmark(orig_url, title, &norm_new, folder)
                {
                    locked.shortcuts_mtime = UserData::shortcuts_mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "已更新书签".to_string()
                    } else {
                        "Bookmark updated".to_string()
                    }));
                }
            }
        }
        "REMOVE_BOOKMARK" => {
            if let Some(raw_url) = val["url"].as_str() {
                let mut locked = app_state.lock_safe();
                let zh = is_zh_lang(&locked.config.language);
                if locked.userdata.remove_bookmark(raw_url) {
                    locked.shortcuts_mtime = UserData::shortcuts_mtime();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "已移除快捷方式".to_string()
                    } else {
                        "Shortcut removed".to_string()
                    }));
                }
            }
        }
        "IMPORT_SHORTCUTS_JSON" => {
            if let Some(raw_json) = val["json"].as_str() {
                let merge = val["merge"].as_bool().unwrap_or(true);
                let mut locked = app_state.lock_safe();
                let zh = is_zh_lang(&locked.config.language);
                match locked.userdata.import_shortcuts(raw_json, merge) {
                    Ok(count) => {
                        locked.shortcuts_mtime = UserData::shortcuts_mtime();
                        drop(locked);
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("已导入 {} 个快捷方式", count)
                        } else {
                            format!("Imported {} shortcuts", count)
                        }));
                    }
                    Err(err) => {
                        drop(locked);
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("导入失败：{}", err)
                        } else {
                            err
                        }));
                    }
                }
            }
        }
        "EXPORT_SHORTCUTS" => {
            let (json, zh) = {
                let st = app_state.lock_safe();
                (
                    st.userdata.shortcuts_json_pretty(),
                    is_zh_lang(&st.config.language),
                )
            };
            export_json_with_toast(
                proxy,
                "fynexo_shortcuts.json",
                &json,
                if zh { "已导出" } else { "Exported" },
                zh,
            );
        }
        "RESET_SHORTCUTS" => {
            let mut locked = app_state.lock_safe();
            let zh = is_zh_lang(&locked.config.language);
            locked.userdata.reset_shortcuts();
            locked.shortcuts_mtime = UserData::shortcuts_mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "快捷方式已恢复默认".to_string()
            } else {
                "Shortcuts reset to defaults".to_string()
            }));
        }
        "IMPORT_THEME_JSON" => {
            if let Some(raw_json) = val["json"].as_str() {
                let zh = is_zh_cfg(app_state);
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
                                let _ = proxy.send_event(UserEvent::Toast(if zh {
                                    format!("已导入主题「{}」", theme_name)
                                } else {
                                    format!("Imported theme '{}'", theme_name)
                                }));
                            }
                            Err(err) => {
                                let _ = proxy.send_event(UserEvent::Toast(if zh {
                                    format!("保存主题失败：{}", err)
                                } else {
                                    format!("Failed to save theme: {}", err)
                                }));
                            }
                        }
                    }
                    Err(err) => {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("导入主题失败：{}", err)
                        } else {
                            err
                        }));
                    }
                }
            }
        }
        "DELETE_THEME" => {
            if let Some(id) = val["id"].as_str() {
                if delete_theme_preset(id) {
                    let mut locked = app_state.lock_safe();
                    let zh = is_zh_lang(&locked.config.language);
                    if locked.config.theme.eq_ignore_ascii_case(id) {
                        locked.config.theme = "dark".to_string();
                        locked.config.save();
                        locked.config_mtime = AppConfig::mtime();
                    }
                    locked.themes = load_all_themes();
                    locked.themes_stamp = themes_stamp();
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        format!("已删除 {}.json", id)
                    } else {
                        format!("Deleted {}.json", id)
                    }));
                }
            }
        }
        "EXPORT_THEME" => {
            let req_id = val["id"].as_str().unwrap_or("");
            let (preset, zh) = {
                let locked = app_state.lock_safe();
                let target = if req_id.is_empty() {
                    locked.config.theme.as_str()
                } else {
                    req_id
                };
                (
                    locked
                        .themes
                        .iter()
                        .find(|t| t.id.eq_ignore_ascii_case(target))
                        .cloned()
                        .unwrap_or_default(),
                    is_zh_lang(&locked.config.language),
                )
            };
            if let Ok(json) = serde_json::to_string_pretty(&preset) {
                let fname = format!("{}.json", preset.id);
                export_json_with_toast(
                    proxy,
                    &fname,
                    &json,
                    if zh { "已导出" } else { "Exported" },
                    zh,
                );
            }
        }
        "OPEN_THEMES_DIR" => {
            #[cfg(target_os = "windows")]
            {
                let local = std::path::PathBuf::from("themes");
                if local.is_dir() {
                    adblock::open_folder_in_explorer(&local);
                } else {
                    adblock::open_folder_in_explorer(&crate::config::themes_dir());
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "手机端请使用「导入 / 导出主题 .json」管理主题文件".to_string()
                } else {
                    "On mobile, use Import / Export Theme .json to manage themes".to_string()
                }));
            }
        }
        "SAVE_KEYBINDS" => {
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
        "IMPORT_KEYBINDS_JSON" => {
            if let Some(raw_json) = val["json"].as_str() {
                let zh = is_zh_cfg(app_state);
                match serde_json::from_str::<KeybindsConfig>(raw_json) {
                    Ok(new_kb) => {
                        new_kb.save();
                        let mut locked = app_state.lock_safe();
                        locked.keybinds = new_kb;
                        locked.keybinds_mtime = KeybindsConfig::mtime();
                        drop(locked);
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            "已导入快捷键配置".to_string()
                        } else {
                            "Imported keybinds".to_string()
                        }));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("快捷键 JSON 无效：{}", e)
                        } else {
                            format!("Invalid keybinds JSON: {}", e)
                        }));
                    }
                }
            }
        }
        "EXPORT_KEYBINDS" => {
            let (kb, zh) = {
                let st = app_state.lock_safe();
                (st.keybinds.clone(), is_zh_lang(&st.config.language))
            };
            if let Ok(json) = serde_json::to_string_pretty(&kb) {
                export_json_with_toast(
                    proxy,
                    "fynexo_keybinds.json",
                    &json,
                    if zh { "已导出" } else { "Exported" },
                    zh,
                );
            }
        }
        "RESET_KEYBINDS" => {
            let def = KeybindsConfig::default();
            def.save();
            let mut locked = app_state.lock_safe();
            let zh = is_zh_lang(&locked.config.language);
            locked.keybinds = def;
            locked.keybinds_mtime = KeybindsConfig::mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "快捷键已恢复默认".to_string()
            } else {
                "Keybinds reset to defaults".to_string()
            }));
        }
        "EXPORT_FULL_SETUP" => {
            let (bundle, zh) = {
                let locked = app_state.lock_safe();
                (
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
                    },
                    is_zh_lang(&locked.config.language),
                )
            };
            if let Ok(json) = serde_json::to_string_pretty(&bundle) {
                export_json_with_toast(
                    proxy,
                    "fynexo-setup-bundle.json",
                    &json,
                    if zh { "已导出完整配置" } else { "Exported full setup" },
                    zh,
                );
            }
        }
        "IMPORT_FULL_SETUP_JSON" => {
            if let Some(raw_json) = val["json"].as_str() {
                let zh = is_zh_cfg(app_state);
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
                        bundle.config.allow_page_eval = false;
                        sanitize_saved_config(&mut bundle.config);
                        bundle.config.save();
                        bundle.styles.save();
                        bundle.keybinds.save();
                        SiteRulesStore::save(&bundle.site_rules);

                        let zoom = bundle.config.default_zoom.clamp(0.5, 2.5);
                        let ua = bundle.config.resolved_user_agent();
                        let incog = bundle.config.incognito;

                        let next_zh = {
                            let mut locked = app_state.lock_safe();
                            locked.userdata.bookmarks = bundle.shortcuts;
                            locked.userdata.save_shortcuts();
                            locked.force_reload();
                            is_zh_lang(&locked.config.language)
                        };
                        tab_manager.lock_safe().set_incognito(incog);
                        let _ = proxy.send_event(UserEvent::SyncAutofill(!incog));
                        let _ = proxy.send_event(UserEvent::SyncDevTools(false));

                        let _ = proxy.send_event(UserEvent::SyncAdblockFilters);
                        let _ = proxy.send_event(UserEvent::ApplyZoom(zoom));
                        let _ = proxy.send_event(UserEvent::UpdateUserAgent(ua));
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(if next_zh {
                            "已导入 Fynexo 完整配置！".to_string()
                        } else {
                            "Full Fynexo setup imported!".to_string()
                        }));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("完整配置 JSON 无效：{}", e)
                        } else {
                            format!("Invalid setup bundle JSON: {}", e)
                        }));
                    }
                }
            }
        }
        "CLEAR_HISTORY" => {
            let mut locked = app_state.lock_safe();
            let zh = is_zh_lang(&locked.config.language);
            locked.userdata.clear_history();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "已清除浏览历史记录".to_string()
            } else {
                "Browsing history cleared".to_string()
            }));
        }
        "DELETE_HISTORY_ITEM" => {
            if let Some(u) = val["url"].as_str() {
                let removed = app_state.lock_safe().userdata.remove_history_item(u);
                if removed {
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "DELETE_HISTORY_BATCH" => {
            let urls: Vec<String> = val["urls"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let searches: Vec<String> = val["searches"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let (removed_total, zh) = {
                let mut locked = app_state.lock_safe();
                let n1 = locked.userdata.remove_history_items(&urls);
                let n2 = locked.userdata.remove_searches(&searches);
                (n1 + n2, is_zh_lang(&locked.config.language))
            };
            if removed_total > 0 {
                let _ = proxy.send_event(UserEvent::SyncConfig);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    format!("已删除 {} 条历史记录", removed_total)
                } else {
                    format!("Removed {} history items", removed_total)
                }));
            }
        }
        "CLEAR_SEARCHES" => {
            let (cleared, zh) = {
                let mut locked = app_state.lock_safe();
                let c = locked.userdata.clear_searches();
                (c, is_zh_lang(&locked.config.language))
            };
            if cleared {
                let _ = proxy.send_event(UserEvent::SyncConfig);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "已清空搜索记录".to_string()
                } else {
                    "Search history cleared".to_string()
                }));
            }
        }
        "RECORD_SEARCH" => {
            if let Some(q) = val["query"].as_str() {
                let req_incognito = val["incognito"].as_bool().unwrap_or(false);
                let mut locked = app_state.lock_safe();
                if !locked.config.incognito && !req_incognito && locked.userdata.record_search(q) {
                    drop(locked);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "REMOVE_SEARCH" => {
            if let Some(q) = val["query"].as_str() {
                let removed = app_state.lock_safe().userdata.remove_search(q);
                if removed {
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "FETCH_SEARCH_SUGGESTIONS" => {
            let query = val["query"].as_str().unwrap_or("").trim().to_string();
            let req_id = val["req_id"].as_u64().unwrap_or(0);
            let req_incognito = val["incognito"].as_bool().unwrap_or(false);
            if query.is_empty() || req_incognito {
                return true;
            }
            let (incognito, engine_id, engine_url_hint, zh) = {
                let locked = app_state.lock_safe();
                let eng = val["engine"]
                    .as_str()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(&locked.config.search_engine)
                    .to_string();
                let hint = locked
                    .config
                    .search_engines
                    .iter()
                    .find(|e| e.id.eq_ignore_ascii_case(&eng))
                    .map(|e| e.url.clone());
                (
                    locked.config.incognito,
                    eng,
                    hint,
                    is_zh_lang(&locked.config.language),
                )
            };
            if !incognito {
                super::super::suggest::spawn_fetch_suggestions(
                    source_win_id,
                    req_id,
                    query,
                    engine_id,
                    engine_url_hint,
                    zh,
                    proxy.clone(),
                );
            }
        }
        "OPEN_DATA_DIR" => {
            #[cfg(target_os = "windows")]
            adblock::open_folder_in_explorer(data_dir());
            #[cfg(not(target_os = "windows"))]
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "应用配置目录位于手机内部沙盒存储，请使用「导出全量备份」".to_string()
                } else {
                    "App data lives in private sandbox storage; use Export Full Backup".to_string()
                }));
            }
        }
        "OPEN_DOWNLOADS_DIR" => {
            #[cfg(target_os = "windows")]
            {
                let custom = app_state.lock_safe().config.downloads_dir.clone();
                let dir = crate::config::resolved_downloads_dir(&custom);
                adblock::open_folder_in_explorer(&dir);
            }
            #[cfg(target_os = "android")]
            {
                let _ = proxy.send_event(UserEvent::EvalScript {
                    win_id: 1,
                    script: "if(window.__fynexo_android__&&typeof window.__fynexo_android__.openDownloads==='function'){window.__fynexo_android__.openDownloads();}".to_string(),
                });
            }
            #[cfg(not(any(target_os = "windows", target_os = "android")))]
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "请在系统文件管理器中查看下载目录".to_string()
                } else {
                    "Open Downloads in your system file manager".to_string()
                }));
            }
        }
        "OPEN_DOWNLOAD_FILE" => {
            if let Some(fname) = val["filename"].as_str() {
                #[cfg(target_os = "windows")]
                {
                    let dl_id = val["id"].as_u64().unwrap_or(0);
                    let (custom, zh, exact_path) = {
                        let st = app_state.lock_safe();
                        (
                            st.config.downloads_dir.clone(),
                            is_zh_lang(&st.config.language),
                            st.resolve_download_path(dl_id, fname),
                        )
                    };
                    let dir = crate::config::resolved_downloads_dir(&custom);
                    let opened = exact_path
                        .as_deref()
                        .map(adblock::open_download_path)
                        .unwrap_or(false)
                        || adblock::open_download_file(&dir, fname);
                    if !opened {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            "找不到已下载的文件（可能已被移动或删除）".to_string()
                        } else {
                            "Downloaded file not found (may have been moved or deleted)".to_string()
                        }));
                    }
                }
                #[cfg(target_os = "android")]
                {
                    let _ = fname;
                    let _ = proxy.send_event(UserEvent::EvalScript {
                        win_id: 1,
                        script: "if(window.__fynexo_android__&&typeof window.__fynexo_android__.openDownloads==='function'){window.__fynexo_android__.openDownloads();}".to_string(),
                    });
                }
                #[cfg(not(any(target_os = "windows", target_os = "android")))]
                let _ = fname;
            }
        }
        "REVEAL_DOWNLOAD_FILE" => {
            if let Some(fname) = val["filename"].as_str() {
                #[cfg(target_os = "windows")]
                {
                    let dl_id = val["id"].as_u64().unwrap_or(0);
                    let (custom, exact_path) = {
                        let st = app_state.lock_safe();
                        (
                            st.config.downloads_dir.clone(),
                            st.resolve_download_path(dl_id, fname),
                        )
                    };
                    let dir = crate::config::resolved_downloads_dir(&custom);
                    if let Some(p) = exact_path {
                        let _ = adblock::reveal_download_path(&p, &dir);
                    } else {
                        let _ = adblock::reveal_download_in_explorer(&dir, fname);
                    }
                }
                #[cfg(target_os = "android")]
                {
                    let _ = fname;
                    let _ = proxy.send_event(UserEvent::EvalScript {
                        win_id: 1,
                        script: "if(window.__fynexo_android__&&typeof window.__fynexo_android__.openDownloads==='function'){window.__fynexo_android__.openDownloads();}".to_string(),
                    });
                }
                #[cfg(not(any(target_os = "windows", target_os = "android")))]
                let _ = fname;
            }
        }
        "CANCEL_DOWNLOAD" => {
            if let Some(id) = val["id"].as_u64() {
                let _ = proxy.send_event(UserEvent::CancelDownload(id));
            }
        }
        "SET_SITE_PERMISSION" => {
            if let (Some(origin), Some(kind), Some(state)) = (
                val["origin"].as_str(),
                val["kind"].as_str(),
                val["state"].as_str(),
            ) {
                let changed = {
                    let mut locked = app_state.lock_safe();
                    let c = locked.site_permissions.set(origin, kind, state);
                    if c {
                        locked.permissions_mtime = SitePermissionsStore::mtime();
                    }
                    c
                };
                if changed {
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "CLEAR_SITE_PERMISSIONS" => {
            let origin_opt = val["origin"].as_str().filter(|s| !s.trim().is_empty());
            let (changed, zh) = {
                let mut locked = app_state.lock_safe();
                let c = match origin_opt {
                    Some(orig) => locked.site_permissions.clear_origin(orig),
                    None => locked.site_permissions.clear_all(),
                };
                if c {
                    locked.permissions_mtime = SitePermissionsStore::mtime();
                }
                (c, is_zh_lang(&locked.config.language))
            };
            if changed {
                let _ = proxy.send_event(UserEvent::SyncConfig);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "已重置网站权限".to_string()
                } else {
                    "Site permissions reset".to_string()
                }));
            }
        }
        "CREATE_PROFILE" => {
            if let Some(name) = val["name"].as_str() {
                let zh = is_zh_cfg(app_state);
                let color = val["color"].as_str();
                match create_profile(name, color) {
                    Ok(entry) => {
                        let display = entry.name.clone();
                        let mut locked = app_state.lock_safe();
                        locked.profiles = load_profiles();
                        drop(locked);
                        let _ = proxy.send_event(UserEvent::SyncConfig);
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("已创建配置文件「{}」", display)
                        } else {
                            format!("Created profile '{}'", display)
                        }));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(e));
                    }
                }
            }
        }
        "DELETE_PROFILE" => {
            if let Some(id) = val["id"].as_str() {
                let (active_id, zh) = {
                    let st = app_state.lock_safe();
                    (st.active_profile_id.clone(), is_zh_lang(&st.config.language))
                };
                if id == active_id {
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "无法删除当前正在使用的配置文件".to_string()
                    } else {
                        "Cannot delete the active profile".to_string()
                    }));
                } else {
                    match delete_profile(id) {
                        Ok(true) => {
                            let mut locked = app_state.lock_safe();
                            locked.profiles = load_profiles();
                            drop(locked);
                            let _ = proxy.send_event(UserEvent::SyncConfig);
                            let _ = proxy.send_event(UserEvent::Toast(if zh {
                                "已删除配置文件".to_string()
                            } else {
                                "Profile deleted".to_string()
                            }));
                        }
                        Ok(false) => {}
                        Err(e) => {
                            let _ = proxy.send_event(UserEvent::Toast(e));
                        }
                    }
                }
            }
        }
        "LAUNCH_PROFILE" => {
            if let Some(id) = val["id"].as_str() {
                let zh = is_zh_cfg(app_state);
                match launch_profile_process(id) {
                    Ok(()) => {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            "正在独立窗口中启动配置文件...".to_string()
                        } else {
                            "Launching profile window...".to_string()
                        }));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(e));
                    }
                }
            }
        }
        "REGISTER_DEFAULT_BROWSER" => {
            let zh = is_zh_cfg(app_state);
            match super::super::os_integration::register_default_browser_and_open_settings() {
                Ok(()) => {
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "已注册 Fynexo，请在系统默认应用中选择设为默认".to_string()
                    } else {
                        "Fynexo registered — select it in Windows Default Apps".to_string()
                    }));
                }
                Err(e) => {
                    let _ = proxy.send_event(UserEvent::Toast(e));
                }
            }
        }
        "PATCH_HUD_PREFS" => {
            let mut config_dirty = false;
            let mut styles_dirty = false;
            let mut rules_dirty = false;
            let mut next_incognito: Option<bool> = None;
            let mut next_ua: Option<Option<String>> = None;

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
                if let Some(v) = val["ask_download_location"].as_bool() {
                    if locked.config.ask_download_location != v {
                        locked.config.ask_download_location = v;
                        config_dirty = true;
                    }
                }
                if let Some(raw_tp) = val["translate_provider"].as_str() {
                    let norm_tp = AppConfig::normalize_translate_provider(raw_tp);
                    if locked.config.translate_provider != norm_tp {
                        locked.config.translate_provider = norm_tp.to_string();
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
                if let Some(raw_ua_mode) = val["user_agent_mode"].as_str() {
                    let norm_ua = match raw_ua_mode.trim().to_ascii_lowercase().as_str() {
                        "desktop" => "desktop",
                        "mobile" => "mobile",
                        "custom" => "custom",
                        _ => "default",
                    };
                    if locked.config.user_agent_mode != norm_ua {
                        locked.config.user_agent_mode = norm_ua.to_string();
                        next_ua = Some(locked.config.resolved_user_agent());
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
                let perf_bools: [ConfigBoolSlot; 6] = [
                    ("efficiency_mode", |c| &mut c.efficiency_mode),
                    ("auto_trim_ram", |c| &mut c.auto_trim_ram),
                    ("enforce_perf", |c| &mut c.enforce_perf),
                    ("reduce_animations", |c| &mut c.reduce_animations),
                    ("hw_accel", |c| &mut c.hw_accel),
                    ("chrome_extensions_enabled", |c| &mut c.chrome_extensions_enabled),
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
                if let Some(layout_raw) = val["topbar_layout"].as_str() {
                    let norm_l = match layout_raw.trim().to_ascii_lowercase().as_str() {
                        "deck" => Some("deck"),
                        "compact" => Some("compact"),
                        "vertical" => Some("vertical"),
                        _ => None,
                    };
                    if let Some(l) = norm_l {
                        if locked.styles.topbar.layout != l {
                            locked.styles.topbar.layout = l.to_string();
                            styles_dirty = true;
                        }
                    }
                }
                if let Some(theme_raw) = val["theme"].as_str() {
                    let tid = theme_raw.trim();
                    if !tid.is_empty() && locked.themes.iter().any(|t| t.id == tid) {
                        if locked.config.theme != tid {
                            locked.config.theme = tid.to_string();
                            config_dirty = true;
                        }
                        if locked.styles.colors.use_custom_colors {
                            locked.styles.colors.use_custom_colors = false;
                            styles_dirty = true;
                        }
                    }
                }
                if let Some(acc_raw) = val["accent_color"].as_str() {
                    let acc = acc_raw.trim();
                    let is_hex = acc.len() == 7
                        && acc.starts_with('#')
                        && acc[1..].chars().all(|c| c.is_ascii_hexdigit());
                    if is_hex {
                        if !locked.styles.colors.use_custom_colors {
                            let active = locked.resolve_active_theme();
                            locked.styles.colors.bg = active.bg;
                            locked.styles.colors.panel = active.panel;
                            locked.styles.colors.surface = active.surface;
                            locked.styles.colors.elevated = active.elevated;
                            locked.styles.colors.border = active.border;
                            locked.styles.colors.border_hover = active.border_hover;
                            locked.styles.colors.fg = active.fg;
                            locked.styles.colors.secondary = active.secondary;
                            locked.styles.colors.muted = active.muted;
                            locked.styles.colors.hover = active.hover;
                            locked.styles.colors.tab_active = active.tab_active;
                            locked.styles.colors.tab_inactive = active.tab_inactive;
                            locked.styles.colors.use_custom_colors = true;
                        }
                        locked.styles.colors.accent = acc.to_string();
                        locked.styles.colors.accent_bg = format!("{}26", acc);
                        styles_dirty = true;
                    }
                }
                if let Some(preset_raw) = val["ui_preset"].as_str() {
                    match preset_raw.trim().to_ascii_lowercase().as_str() {
                        "compact" => {
                            locked.styles.topbar.layout = "compact".to_string();
                            locked.styles.topbar.height = "36px".to_string();
                            locked.styles.topbar.background = "var(--fnx-panel)".to_string();
                            locked.styles.topbar.custom_css.clear();
                            locked.styles.tabs.height = "26px".to_string();
                            locked.styles.tabs.border_radius = "6px".to_string();
                            locked.styles.tabs.gap = "3px".to_string();
                            locked.styles.address_bar.border_radius = "6px".to_string();
                            locked.styles.address_bar.height = "26px".to_string();
                            styles_dirty = true;
                        }
                        "deck" => {
                            locked.styles.topbar.layout = "deck".to_string();
                            locked.styles.topbar.height = "40px".to_string();
                            locked.styles.topbar.titlebar_height = "42px".to_string();
                            locked.styles.topbar.background = "var(--fnx-panel)".to_string();
                            locked.styles.topbar.custom_css.clear();
                            locked.styles.tabs.height = "30px".to_string();
                            locked.styles.tabs.border_radius = "8px".to_string();
                            locked.styles.tabs.gap = "4px".to_string();
                            locked.styles.address_bar.border_radius = "8px".to_string();
                            locked.styles.address_bar.height = "29px".to_string();
                            styles_dirty = true;
                        }
                        "pill" => {
                            locked.styles.topbar.layout = "deck".to_string();
                            locked.styles.topbar.height = "42px".to_string();
                            locked.styles.topbar.titlebar_height = "44px".to_string();
                            locked.styles.topbar.button_radius = "999px".to_string();
                            locked.styles.tabs.height = "32px".to_string();
                            locked.styles.tabs.border_radius = "999px".to_string();
                            locked.styles.tabs.gap = "6px".to_string();
                            locked.styles.address_bar.border_radius = "999px".to_string();
                            locked.styles.address_bar.height = "30px".to_string();
                            styles_dirty = true;
                        }
                        "vertical" => {
                            locked.styles.topbar.layout = "vertical".to_string();
                            locked.styles.topbar.vertical_width = "216px".to_string();
                            locked.styles.topbar.height = "40px".to_string();
                            locked.styles.tabs.height = "32px".to_string();
                            locked.styles.tabs.border_radius = "8px".to_string();
                            locked.styles.tabs.gap = "4px".to_string();
                            locked.styles.address_bar.border_radius = "8px".to_string();
                            locked.styles.address_bar.height = "29px".to_string();
                            styles_dirty = true;
                        }
                        "glass" => {
                            locked.styles.topbar.background =
                                "color-mix(in srgb, var(--fnx-panel) 76%, transparent)".to_string();
                            locked.styles.topbar.custom_css =
                                "backdrop-filter: blur(16px) saturate(140%); -webkit-backdrop-filter: blur(16px) saturate(140%);".to_string();
                            locked.styles.tabs.border_radius = "10px".to_string();
                            locked.styles.address_bar.border_radius = "10px".to_string();
                            styles_dirty = true;
                        }
                        _ => {}
                    }
                }
                if let Some(tb_obj) = val.get("toolbar_btn").and_then(|v| v.as_object()) {
                    let k = tb_obj.get("key").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(vis) = tb_obj.get("visible").and_then(|v| v.as_bool()) {
                        match k {
                            "copy" => {
                                locked.styles.topbar.show_copy_btn = vis;
                                styles_dirty = true;
                            }
                            "star" => {
                                locked.styles.topbar.show_star_btn = vis;
                                styles_dirty = true;
                            }
                            "shield" => {
                                locked.styles.topbar.show_shield_btn = vis;
                                styles_dirty = true;
                            }
                            "downloads" => {
                                locked.styles.topbar.show_downloads_btn = vis;
                                styles_dirty = true;
                            }
                            _ => {}
                        }
                    }
                }
                if let Some(v) = val["mobile_auto_hide_on_scroll"].as_bool() {
                    if locked.styles.mobile_bar.auto_hide_on_scroll != v {
                        locked.styles.mobile_bar.auto_hide_on_scroll = v;
                        styles_dirty = true;
                    }
                }
                if let Some(nt_obj) = val.get("newtab_prefs").and_then(|v| v.as_object()) {
                    if let Some(v) = nt_obj.get("show_clock").and_then(|v| v.as_bool()) {
                        locked.styles.newtab.show_clock = v;
                        styles_dirty = true;
                    }
                    if let Some(v) = nt_obj.get("use_24h_clock").and_then(|v| v.as_bool()) {
                        locked.styles.newtab.use_24h_clock = v;
                        styles_dirty = true;
                    }
                    if let Some(v) = nt_obj.get("show_shortcuts").and_then(|v| v.as_bool()) {
                        locked.styles.newtab.show_shortcuts = v;
                        styles_dirty = true;
                    }
                    if let Some(v) = nt_obj.get("shortcuts_columns").and_then(|v| v.as_u64()) {
                        let cols = (v as u8).min(6);
                        locked.styles.newtab.shortcuts_columns = if cols == 1 { 0 } else { cols };
                        styles_dirty = true;
                    }
                    if let Some(v) = nt_obj.get("show_history").and_then(|v| v.as_bool()) {
                        locked.styles.newtab.show_history = v;
                        styles_dirty = true;
                    }
                    if let Some(v) = nt_obj.get("background_image").and_then(|v| v.as_str()) {
                        let trimmed = v.trim();
                        if trimmed.len() <= 512 && !trimmed.contains('<') && !trimmed.contains('>') {
                            locked.styles.newtab.background_image = trimmed.to_string();
                            styles_dirty = true;
                        }
                    }
                    if let Some(v) = nt_obj.get("background_dim").and_then(|v| v.as_u64()) {
                        locked.styles.newtab.background_dim = (v as u8).min(90);
                        styles_dirty = true;
                    }
                    if let Some(v) = nt_obj.get("background_blur").and_then(|v| v.as_u64()) {
                        locked.styles.newtab.background_blur = (v as u8).min(24);
                        styles_dirty = true;
                    }
                }
                if let Some(boost_obj) = val.get("site_boost").and_then(|v| v.as_object()) {
                    let domain = boost_obj
                        .get("domain")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_ascii_lowercase()
                        .trim_start_matches("www.")
                        .to_string();
                    let css = boost_obj
                        .get("css")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    let valid_dom = !domain.is_empty()
                        && domain.len() <= 128
                        && !domain.contains('*')
                        && domain
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'));
                    let valid_css = css.len() <= 16384 && !css.contains('<') && !css.contains('>');
                    if valid_dom && valid_css {
                        let slug: String = domain
                            .chars()
                            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
                            .collect();
                        let rule_id = format!("boost-{}", slug);
                        if css.is_empty() {
                            let before = locked.site_rules.len();
                            locked.site_rules.retain(|r| r.id != rule_id);
                            if locked.site_rules.len() != before {
                                rules_dirty = true;
                            }
                        } else if let Some(existing) =
                            locked.site_rules.iter_mut().find(|r| r.id == rule_id)
                        {
                            existing.enabled = true;
                            existing.css = css.to_string();
                            existing.js.clear();
                            rules_dirty = true;
                        } else {
                            locked.site_rules.push(SiteRule {
                                id: rule_id,
                                name: format!("Site Boost ({})", domain),
                                domain,
                                enabled: true,
                                version: None,
                                css: css.to_string(),
                                js: String::new(),
                            });
                            rules_dirty = true;
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
            if let Some(ua_val) = next_ua {
                let _ = proxy.send_event(UserEvent::UpdateUserAgent(ua_val));
            }
            if config_dirty || styles_dirty || rules_dirty {
                let _ = proxy.send_event(UserEvent::SyncConfig);
            }
        }
        "SAVE_CONFIG" => {
            if let Some(cfg_val) = val.get("config") {
                if let Ok(mut new_cfg) =
                    serde_json::from_value::<AppConfig>(cfg_val.clone())
                {
                    sanitize_saved_config(&mut new_cfg);
                    let next_adblock = new_cfg.adblock;
                    let next_zoom = new_cfg.default_zoom;
                    let next_overrides = new_cfg.site_overrides.clone();
                    let next_ua = new_cfg.resolved_user_agent();
                    let next_incog = new_cfg.incognito;
                    let next_allow_eval = new_cfg.allow_page_eval;

                    let (adblock_changed, zoom_changed, ua_changed, incog_changed, eval_changed) = {
                        let mut locked = app_state.lock_safe();
                        let adblock_ch = locked.config.adblock != next_adblock;
                        let zoom_ch = (locked.config.default_zoom - next_zoom).abs() > 0.001
                            || locked.config.site_overrides != next_overrides;
                        let ua_ch = locked.config.resolved_user_agent() != next_ua;
                        let incog_ch = locked.config.incognito != next_incog;
                        let eval_ch = locked.config.allow_page_eval != next_allow_eval;

                        locked.config = new_cfg;
                        locked.config.save();
                        locked.config_mtime = AppConfig::mtime();
                        (adblock_ch, zoom_ch, ua_ch, incog_ch, eval_ch)
                    };

                    if adblock_changed {
                        adblock::set_enabled(next_adblock);
                        let _ = proxy.send_event(UserEvent::SyncAdblockFilters);
                    }
                    if incog_changed {
                        tab_manager.lock_safe().set_incognito(next_incog);
                        let _ = proxy.send_event(UserEvent::SyncAutofill(!next_incog));
                    }
                    if eval_changed {
                        let _ = proxy.send_event(UserEvent::SyncDevTools(next_allow_eval));
                    }
                    if zoom_changed {
                        let _ = proxy.send_event(UserEvent::ApplyZoom(next_zoom));
                    }
                    if ua_changed {
                        let _ = proxy.send_event(UserEvent::UpdateUserAgent(next_ua));
                    }
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "SAVE_STYLES" => {
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
        "RESET_STYLES" => {
            let mut locked = app_state.lock_safe();
            locked.styles = ComponentStyles::default();
            locked.styles.save();
            locked.styles_mtime = ComponentStyles::mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
        }
        "RELOAD_STYLES" => {
            let mut locked = app_state.lock_safe();
            locked.force_reload();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
        }
        "GET_CONFIG" => {
            let _ = proxy.send_event(UserEvent::SyncConfig);
        }
        _ => return false,
    }
    true
}

fn sanitize_saved_config(cfg: &mut AppConfig) {
    if !cfg.default_zoom.is_finite() || cfg.default_zoom < 0.3 {
        cfg.default_zoom = 1.0;
    } else {
        cfg.default_zoom = ((cfg.default_zoom * 100.0).round() / 100.0).clamp(0.5, 2.5);
    }
    cfg.sanitize_perf_fields();

    let raw_hp = cfg.homepage.trim();
    let hp = if raw_hp.is_empty() {
        "fynexo://newtab".to_string()
    } else {
        let norm = normalize_logical_url(raw_hp);
        if security::is_safe_navigation_target(&norm) {
            norm
        } else {
            "fynexo://newtab".to_string()
        }
    };
    cfg.homepage = hp;

    cfg.user_agent_mode = match cfg.user_agent_mode.trim().to_ascii_lowercase().as_str() {
        "desktop" => "desktop",
        "mobile" => "mobile",
        "custom" => "custom",
        _ => "default",
    }
    .to_string();
    cfg.custom_user_agent = cfg.custom_user_agent.trim().to_string();
    cfg.window_mode = if cfg.window_mode.trim().eq_ignore_ascii_case("single") {
        "single".to_string()
    } else {
        "multi".to_string()
    };
    cfg.language = if cfg.language.trim().eq_ignore_ascii_case("en") {
        "en".to_string()
    } else {
        "zh".to_string()
    };
    let norm_mode = cfg.active_mode.trim().to_ascii_lowercase();
    cfg.active_mode = if norm_mode.is_empty() {
        "auto".to_string()
    } else {
        norm_mode
    };
    cfg.downloads_dir = cfg.downloads_dir.trim().to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_saved_config_normalizes_and_rejects_unsafe_homepage() {
        let mut cfg = AppConfig {
            homepage: "javascript:alert(1)".to_string(),
            default_zoom: f64::NAN,
            user_agent_mode: "MOBILE".to_string(),
            window_mode: "SINGLE".to_string(),
            language: "EN".to_string(),
            active_mode: "  ".to_string(),
            ..AppConfig::default()
        };
        sanitize_saved_config(&mut cfg);
        assert_eq!(cfg.homepage, "fynexo://newtab");
        assert_eq!(cfg.default_zoom, 1.0);
        assert_eq!(cfg.user_agent_mode, "mobile");
        assert_eq!(cfg.window_mode, "single");
        assert_eq!(cfg.language, "en");
        assert_eq!(cfg.active_mode, "auto");
    }
}
