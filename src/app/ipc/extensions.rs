use crate::adblock::{self, AdblockConfig};
use crate::config::{default_site_rules, SiteRule, SiteRulesStore};
use crate::util::LockExt;

use super::super::events::UserEvent;
use super::{export_json_with_toast, is_zh_cfg, is_zh_lang, IpcCtx};

pub(super) fn try_handle(cmd: &str, val: &serde_json::Value, ctx: &IpcCtx<'_>) -> bool {
    let IpcCtx {
        source_win_id,
        source_tab_id: _,
        app_state,
        tab_manager: _,
        proxy,
    } = *ctx;
    let _ = source_win_id;
    match cmd {
        "SAVE_RULES" => {
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
        "IMPORT_RULES_JSON" => {
            if let Some(raw_json) = val["json"].as_str() {
                let zh = is_zh_cfg(app_state);
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
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("已导入 {} 条站点规则", count)
                        } else {
                            format!("Imported {} site rules", count)
                        }));
                    }
                    Err(err) => {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("导入站点规则失败：{}", err)
                        } else {
                            err
                        }));
                    }
                }
            }
        }
        "EXPORT_RULES" => {
            let (rules, zh) = {
                let st = app_state.lock_safe();
                (st.site_rules.clone(), is_zh_lang(&st.config.language))
            };
            if let Ok(json) = serde_json::to_string_pretty(&rules) {
                export_json_with_toast(
                    proxy,
                    "fynexo_rules.json",
                    &json,
                    if zh { "已导出" } else { "Exported" },
                    zh,
                );
            }
        }
        "RESET_RULES" => {
            let defs = default_site_rules();
            SiteRulesStore::save(&defs);
            let mut locked = app_state.lock_safe();
            let zh = is_zh_lang(&locked.config.language);
            locked.site_rules = defs;
            locked.rules_mtime = SiteRulesStore::mtime();
            drop(locked);
            let _ = proxy.send_event(UserEvent::SyncConfig);
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "站点规则已恢复默认".to_string()
            } else {
                "Site rules reset to defaults".to_string()
            }));
        }
        "SAVE_ADBLOCK" => {
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
        "IMPORT_ADBLOCK_JSON" => {
            if let Some(raw_json) = val["json"].as_str() {
                let zh = is_zh_cfg(app_state);
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
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            "已导入广告拦截规则列表".to_string()
                        } else {
                            "Imported adblock lists".to_string()
                        }));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("广告拦截规则 JSON 无效：{}", e)
                        } else {
                            format!("Invalid adblock JSON: {}", e)
                        }));
                    }
                }
            }
        }
        "EXPORT_ADBLOCK" => {
            let (adb, zh) = {
                let st = app_state.lock_safe();
                (st.adblock.clone(), is_zh_lang(&st.config.language))
            };
            if let Ok(json) = serde_json::to_string_pretty(&adb) {
                export_json_with_toast(
                    proxy,
                    "fynexo_adblock.json",
                    &json,
                    if zh { "已导出" } else { "Exported" },
                    zh,
                );
            }
        }
        "INSTALL_CHROME_EXT_STORE" => {
            #[cfg(not(target_os = "windows"))]
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "Chrome 扩展仅支持 Windows 桌面端".to_string()
                } else {
                    "Chrome extensions are only supported on Windows desktop".to_string()
                }));
            }
            #[cfg(target_os = "windows")]
            {
                let raw_in = val["input"]
                    .as_str()
                    .or_else(|| val["url"].as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let Some((ext_id, store_src)) = crate::chrome_ext::crx::parse_store_input(&raw_in)
                else {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "无效的 Chrome/Edge 商店链接或 32 位扩展 ID".to_string()
                    } else {
                        "Invalid Chrome/Edge Web Store URL or 32-char extension ID".to_string()
                    }));
                    return true;
                };
                let (lang, src_name, store_url) = {
                    let st = app_state.lock_safe();
                    let src = match store_src {
                        crate::chrome_ext::crx::StoreSource::Edge => "edge_store",
                        crate::chrome_ext::crx::StoreSource::Chrome => "chrome_store",
                    };
                    let u = if raw_in.starts_with("http://") || raw_in.starts_with("https://") {
                        raw_in.clone()
                    } else {
                        format!(
                            "https://chromewebstore.google.com/detail/{}",
                            ext_id
                        )
                    };
                    (st.config.language.clone(), src, u)
                };
                let _ = proxy.send_event(UserEvent::Toast(
                    if is_zh_lang(&lang) {
                        format!("正在从商店下载扩展 {} ...", &ext_id[..8])
                    } else {
                        format!("Downloading extension {} ...", &ext_id[..8])
                    },
                ));
                let p = proxy.clone();
                std::thread::spawn(move || {
                    match crate::chrome_ext::download_store_crx(&ext_id, store_src)
                        .and_then(|bytes| {
                            crate::chrome_ext::install_from_archive_bytes(
                                Some(&ext_id),
                                &bytes,
                                src_name,
                                &store_url,
                                &lang,
                            )
                        }) {
                        Ok(entry) => {
                            let _ = p.send_event(UserEvent::InstallChromeExtUnpacked {
                                entry: Box::new(entry),
                            });
                        }
                        Err(err) => {
                            let _ = p.send_event(UserEvent::Toast(if is_zh_lang(&lang) {
                                format!("扩展安装失败：{}", err)
                            } else {
                                format!("Extension install failed: {}", err)
                            }));
                        }
                    }
                });
            }
        }
        "INSTALL_CHROME_EXT_FILE" => {
            #[cfg(not(target_os = "windows"))]
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "Chrome 扩展仅支持 Windows 桌面端".to_string()
                } else {
                    "Chrome extensions are only supported on Windows desktop".to_string()
                }));
            }
            #[cfg(target_os = "windows")]
            {
                let b64 = val["base64"].as_str().unwrap_or("");
                if b64.is_empty() || b64.len() > 68 * 1024 * 1024 {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "扩展文件为空或超过 48 MB 限制".to_string()
                    } else {
                        "Extension file is empty or exceeds 48 MB limit".to_string()
                    }));
                    return true;
                }
                let Ok(bytes) = crate::chrome_ext::crx::base64_decode(b64) else {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "扩展压缩包 Base64 编码无效".to_string()
                    } else {
                        "Invalid base64 extension archive".to_string()
                    }));
                    return true;
                };
                if bytes.len() > 48 * 1024 * 1024 {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "扩展压缩包大小超过 48 MB 限制".to_string()
                    } else {
                        "Extension archive exceeds 48 MB limit".to_string()
                    }));
                    return true;
                }
                let filename = val["filename"].as_str().unwrap_or("extension.crx").to_string();
                let id_hint = filename
                    .strip_suffix(".crx")
                    .or_else(|| filename.strip_suffix(".zip"))
                    .filter(|s| crate::chrome_ext::crx::is_valid_chrome_ext_id(s))
                    .map(|s| s.to_string());
                let lang = app_state.lock_safe().config.language.clone();
                let p = proxy.clone();
                std::thread::spawn(move || {
                    match crate::chrome_ext::install_from_archive_bytes(
                        id_hint.as_deref(),
                        &bytes,
                        "crx_file",
                        "",
                        &lang,
                    ) {
                        Ok(entry) => {
                            let _ = p.send_event(UserEvent::InstallChromeExtUnpacked {
                                entry: Box::new(entry),
                            });
                        }
                        Err(err) => {
                            let _ = p.send_event(UserEvent::Toast(if is_zh_lang(&lang) {
                                format!("安装包安装失败：{}", err)
                            } else {
                                format!("Failed installing archive: {}", err)
                            }));
                        }
                    }
                });
            }
        }
        "INSTALL_CHROME_EXT_DIR" => {
            #[cfg(not(target_os = "windows"))]
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "Chrome 扩展仅支持 Windows 桌面端".to_string()
                } else {
                    "Chrome extensions are only supported on Windows desktop".to_string()
                }));
            }
            #[cfg(target_os = "windows")]
            {
                let raw_path = val["path"].as_str().unwrap_or("").trim().trim_matches('"').to_string();
                if raw_path.is_empty() {
                    let zh = is_zh_cfg(app_state);
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "请输入包含 manifest.json 的已解压扩展文件夹路径".to_string()
                    } else {
                        "Enter unpacked extension folder path containing manifest.json".to_string()
                    }));
                    return true;
                }
                let lang = app_state.lock_safe().config.language.clone();
                let p = proxy.clone();
                std::thread::spawn(move || {
                    let dir = std::path::PathBuf::from(&raw_path);
                    match crate::chrome_ext::install_from_local_dir(&dir, &lang) {
                        Ok(entry) => {
                            let _ = p.send_event(UserEvent::InstallChromeExtUnpacked {
                                entry: Box::new(entry),
                            });
                        }
                        Err(err) => {
                            let _ = p.send_event(UserEvent::Toast(if is_zh_lang(&lang) {
                                format!("读取扩展文件夹失败：{}", err)
                            } else {
                                format!("Failed loading folder: {}", err)
                            }));
                        }
                    }
                });
            }
        }
        "TOGGLE_CHROME_EXT" => {
            if let Some(id) = val["id"].as_str() {
                let want = val["enabled"].as_bool();
                let target_enabled = {
                    let mut st = app_state.lock_safe();
                    if let Some(entry) = st.chrome_extensions.iter_mut().find(|e| e.id == id) {
                        entry.enabled = want.unwrap_or(!entry.enabled);
                        let en = entry.enabled;
                        crate::chrome_ext::save_registry(&st.chrome_extensions);
                        Some(en)
                    } else {
                        None
                    }
                };
                if let Some(en) = target_enabled {
                    let _ = proxy.send_event(UserEvent::ToggleChromeExt {
                        id: id.to_string(),
                        enabled: en,
                    });
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "PIN_CHROME_EXT" => {
            if let Some(id) = val["id"].as_str() {
                let mut st = app_state.lock_safe();
                if let Some(entry) = st.chrome_extensions.iter_mut().find(|e| e.id == id) {
                    entry.pinned = !entry.pinned;
                    crate::chrome_ext::save_registry(&st.chrome_extensions);
                    drop(st);
                    let _ = proxy.send_event(UserEvent::SyncConfig);
                }
            }
        }
        "REMOVE_CHROME_EXT" => {
            if let Some(id) = val["id"].as_str() {
                let _ = proxy.send_event(UserEvent::RemoveChromeExt { id: id.to_string() });
            }
        }
        "OPEN_CHROME_EXT_POPUP" => {
            if let Some(id) = val["id"].as_str() {
                let _ = proxy.send_event(UserEvent::OpenChromeExtPopup {
                    win_id: source_win_id,
                    id: id.to_string(),
                });
            }
        }
        "OPEN_CHROME_EXT_OPTIONS" => {
            if let Some(id) = val["id"].as_str() {
                let _ = proxy.send_event(UserEvent::OpenChromeExtOptions {
                    win_id: source_win_id,
                    id: id.to_string(),
                });
            }
        }
        "OPEN_CHROME_EXT_DIR" => {
            #[cfg(target_os = "windows")]
            adblock::open_folder_in_explorer(&crate::chrome_ext::chrome_extensions_dir());
            #[cfg(not(target_os = "windows"))]
            {
                let zh = is_zh_cfg(app_state);
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    "Chrome 扩展目录仅支持 Windows 桌面端".to_string()
                } else {
                    "Chrome extensions directory is Windows desktop-only".to_string()
                }));
            }
        }
        _ => return false,
    }
    true
}
