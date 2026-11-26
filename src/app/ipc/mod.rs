mod config;
mod extensions;
mod tabs;

use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;

use crate::config::export_json_to_downloads;
use crate::tabs::TabManager;
use crate::util::LockExt;

use super::events::UserEvent;
use super::state::AppState;

pub(super) struct IpcCtx<'a> {
    pub source_win_id: usize,
    pub source_tab_id: usize,
    pub app_state: &'a Arc<Mutex<AppState>>,
    pub tab_manager: &'a Arc<Mutex<TabManager>>,
    pub proxy: &'a EventLoopProxy<UserEvent>,
}

#[inline]
pub(super) fn is_zh_lang(lang: &str) -> bool {
    crate::util::is_zh_lang(lang)
}

#[inline]
pub(super) fn is_zh_cfg(app_state: &Arc<Mutex<AppState>>) -> bool {
    is_zh_lang(&app_state.lock_safe().config.language)
}

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

    if !super::security::authorize_ipc_for_source(origin, source_tab_id, &val) {
        #[cfg(debug_assertions)]
        eprintln!("[IPC] rejected untrusted message from {}", origin);
        return;
    }

    let Some(cmd) = val["type"].as_str() else {
        return;
    };

    let ctx = IpcCtx {
        source_win_id,
        source_tab_id,
        app_state,
        tab_manager,
        proxy,
    };

    if tabs::try_handle(cmd, &val, &ctx) {
        return;
    }
    if config::try_handle(cmd, &val, &ctx) {
        return;
    }
    let _ = extensions::try_handle(cmd, &val, &ctx);
}

pub(super) fn export_json_with_toast(
    proxy: &EventLoopProxy<UserEvent>,
    filename: &str,
    json: &str,
    action_label: &str,
    zh: bool,
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
                id: 0,
                name: name_only.to_string(),
                status: "completed".to_string(),
                bytes_received: json.len() as u64,
                total_bytes: json.len() as u64,
                speed_bps: 0,
                saved_path: Some(saved_path.to_string_lossy().to_string()),
            });
            let msg = if zh {
                format!("{} {} 至下载目录", action_label, name_only)
            } else {
                format!("{} {} to Downloads", action_label, name_only)
            };
            let _ = proxy.send_event(UserEvent::Toast(msg));
        }
        Err(err) => {
            let msg = if zh {
                format!("导出失败：{}", err)
            } else {
                format!("Export failed: {}", err)
            };
            let _ = proxy.send_event(UserEvent::Toast(msg));
        }
    }
}
