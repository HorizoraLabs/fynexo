//! IPC trust boundary, URL validation, and Rust -> JS bridge authentication.
//!
//! wry exposes `window.ipc.postMessage` to *every* page, including arbitrary
//! websites. We enforce defence-in-depth:
//!  1. Privileged settings/filesystem/rule commands are accepted **only** from
//!     internal pages (`fynexo://` or `fynexo.localhost`) — never from external
//!     origins even with a token.
//!  2. External-page HUD actions (tab switching, navigation, zoom, etc.) are
//!     additionally restricted to an explicit allowlist and require the
//!     per-process `ipc_token()`.
//!  3. Rust -> JS `evaluate_script` bridge calls pass `bridge_token()` so
//!     external page scripts cannot spoof HUD updates.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::OnceLock;

static IPC_TOKEN: OnceLock<String> = OnceLock::new();
static BRIDGE_TOKEN: OnceLock<String> = OnceLock::new();

fn generate_process_secret() -> String {
    let mut out = String::with_capacity(32);
    for salt in 0u64..2 {
        let mut h = RandomState::new().build_hasher();
        h.write_u64(salt);
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        );
        h.write_u32(std::process::id());
        out.push_str(&format!("{:016x}", h.finish()));
    }
    out
}

/// Per-process secret embedded in the HUD initialization script for JS -> Rust IPC.
pub fn ipc_token() -> &'static str {
    IPC_TOKEN.get_or_init(generate_process_secret)
}

/// Independent per-process secret passed from Rust -> JS `evaluate_script` bridge calls.
pub fn bridge_token() -> &'static str {
    BRIDGE_TOKEN.get_or_init(generate_process_secret)
}

/// Is `origin` (the URL of the page that sent the IPC message) an internal Fynexo page?
pub fn is_internal_origin(origin: &str) -> bool {
    let lower = origin.trim().to_ascii_lowercase();
    if lower.starts_with("fynexo://") {
        return true;
    }
    for scheme in ["http://", "https://"] {
        if let Some(rest) = lower.strip_prefix(scheme) {
            let host = rest
                .split(['/', '?', '#'])
                .next()
                .unwrap_or("")
                .split(':')
                .next()
                .unwrap_or("");
            return host == "fynexo.localhost";
        }
    }
    false
}

/// Constant-time string comparison (avoids leaking token prefix via timing).
fn ct_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Commands the HUD overlay may trigger while hosted on external websites.
/// Rule-writing, config import/export, theme deletion, data wiping, and internal
/// directory opening are deliberately excluded and require `is_internal_origin`.
fn is_hud_allowed_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "BOOT_SYNC"
            | "NAVIGATE"
            | "NEW_TAB"
            | "SWITCH_TAB"
            | "TOGGLE_PIN_TAB"
            | "TOGGLE_BACKGROUND_PLAY"
            | "TOGGLE_MUTE_TAB"
            | "GET_RESOURCE_USAGE"
            | "FORCE_TRIM_MEMORY"
            | "SAVE_WORKSPACE"
            | "CLOSE_TAB"
            | "DUPLICATE_TAB"
            | "CLOSE_OTHER_TABS"
            | "CLOSE_TABS_TO_RIGHT"
            | "CLOSE_ALL_TABS"
            | "REOPEN_CLOSED_TAB"
            | "UPDATE_TAB_TITLE"
            | "SAVE_SCROLL"
            | "TOGGLE_FULLSCREEN"
            | "SET_ZOOM"
            | "TOGGLE_BOOKMARK"
            | "OPEN_DOWNLOADS_DIR"
            | "GET_TABS"
            | "SAVE_CONFIG"
            | "SAVE_STYLES"
            | "GET_CONFIG"
            | "WINDOW_DRAG"
            | "WINDOW_MINIMIZE"
            | "WINDOW_MAXIMIZE"
            | "WINDOW_CLOSE"
            | "GO_BACK"
            | "GO_FORWARD"
            | "RELOAD"
    )
}

/// Decide whether an IPC message may be processed.
pub fn authorize_ipc(origin: &str, message: &serde_json::Value) -> bool {
    let Some(cmd) = message.get("type").and_then(|v| v.as_str()) else {
        return false;
    };
    if is_internal_origin(origin) {
        return true;
    }
    if !is_hud_allowed_command(cmd) {
        return false;
    }
    message
        .get("__t")
        .and_then(|t| t.as_str())
        .map(|t| ct_eq(t, ipc_token()))
        .unwrap_or(false)
}

fn has_control_chars(s: &str) -> bool {
    s.bytes().any(|b| b < 0x20 || b == 0x7f)
}

/// True only for external `http://` or `https://` URLs (excluding `fynexo.localhost`).
pub fn is_safe_external_http_url(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() || has_control_chars(trimmed) {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let rest = if let Some(r) = lower.strip_prefix("http://") {
        r
    } else if let Some(r) = lower.strip_prefix("https://") {
        r
    } else {
        return false;
    };
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    !host.is_empty() && host != "fynexo.localhost"
}

fn is_exact_internal_target(lower: &str) -> bool {
    for prefix in ["fynexo://newtab", "fynexo://settings"] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            if rest.is_empty()
                || rest.starts_with('/')
                || rest.starts_with('?')
                || rest.starts_with('#')
            {
                return true;
            }
        }
    }
    false
}

/// Allowed navigation targets from IPC (`http://`, `https://`, `fynexo://newtab`, `fynexo://settings`, `about:blank`).
/// Rejects `file://`, `javascript:`, `data:`, `vbscript:`, etc.
pub fn is_safe_navigation_target(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() || has_control_chars(trimmed) {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || is_exact_internal_target(&lower)
        || lower == "about:blank"
}

/// Allowed URLs for bookmarks / shortcuts (`http://`, `https://`, internal newtab/settings).
pub fn is_safe_bookmark_url(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() || has_control_chars(trimmed) {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || is_exact_internal_target(&lower)
}

/// Resolve which internal page a custom-protocol request targets.
/// Exact matching only: `fynexo://settings-x` or `/foo/settings` are NOT settings.
pub fn resolve_internal_route(host: &str, path: &str) -> Option<&'static str> {
    let host = host.trim().to_ascii_lowercase();
    let first_seg = path
        .trim_matches('/')
        .split('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    let key = if host.is_empty() || host == "localhost" || host.ends_with(".localhost") {
        first_seg
    } else {
        host
    };

    match key.as_str() {
        "settings" => Some("settings"),
        "" | "newtab" => Some("newtab"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn internal_origins_are_recognised() {
        assert!(is_internal_origin("fynexo://settings"));
        assert!(is_internal_origin("http://fynexo.localhost/settings"));
        assert!(is_internal_origin("https://fynexo.localhost:8080/newtab"));
        assert!(!is_internal_origin("https://example.com/"));
        assert!(!is_internal_origin("https://fynexo.localhost.evil.com/"));
        assert!(!is_internal_origin("https://evil.com/?u=fynexo.localhost"));
        assert!(!is_internal_origin(""));
    }

    #[test]
    fn external_origin_cannot_call_privileged_commands_even_with_token() {
        let evil = "https://evil.example/";
        let tok = ipc_token();
        for cmd in [
            "SAVE_RULES",
            "IMPORT_RULES_JSON",
            "IMPORT_FULL_SETUP_JSON",
            "SAVE_ADBLOCK",
            "SAVE_KEYBINDS",
            "DELETE_THEME",
            "CLEAR_SITE_DATA",
            "CLEAR_HISTORY",
            "OPEN_DATA_DIR",
            "OPEN_THEMES_DIR",
            "EXPORT_SHORTCUTS",
            "RESET_STYLES",
            "LOAD_WORKSPACE",
        ] {
            assert!(!authorize_ipc(evil, &json!({"type": cmd, "__t": tok})), "{cmd} leaked");
        }
        assert!(authorize_ipc("fynexo://settings", &json!({"type": "SAVE_RULES"})));
    }

    #[test]
    fn external_origin_hud_commands_require_valid_token() {
        let evil = "https://evil.example/";
        assert!(!authorize_ipc(evil, &json!({"type": "NEW_TAB"})));
        assert!(!authorize_ipc(evil, &json!({"type": "NEW_TAB", "__t": "guess"})));
        assert!(authorize_ipc(evil, &json!({"type": "NEW_TAB", "__t": ipc_token()})));
        assert!(!authorize_ipc(evil, &json!({"type": "UNKNOWN_CMD", "__t": ipc_token()})));
    }

    #[test]
    fn tokens_are_distinct_stable_and_long() {
        assert_eq!(ipc_token(), ipc_token());
        assert_eq!(bridge_token(), bridge_token());
        assert_ne!(ipc_token(), bridge_token());
        assert_eq!(ipc_token().len(), 32);
        assert_eq!(bridge_token().len(), 32);
    }

    #[test]
    fn url_validators_block_dangerous_schemes() {
        assert!(is_safe_external_http_url("https://example.com/a"));
        assert!(!is_safe_external_http_url("http://fynexo.localhost/settings"));
        assert!(!is_safe_external_http_url("fynexo://settings"));
        assert!(!is_safe_external_http_url("file:///C:/Windows/win.ini"));
        assert!(!is_safe_external_http_url("javascript:alert(1)"));
        assert!(!is_safe_external_http_url("data:text/html,<script>"));

        assert!(is_safe_navigation_target("fynexo://settings#sec-rules"));
        assert!(!is_safe_navigation_target("fynexo://settings-evil"));
        assert!(is_safe_navigation_target("https://rust-lang.org"));
        assert!(!is_safe_navigation_target("javascript:alert(1)"));
        assert!(!is_safe_navigation_target("file:///etc/passwd"));
        assert!(!is_safe_navigation_target("data:text/html,hi"));
        assert!(!is_safe_navigation_target("https://a\r\n.com"));
        assert!(!is_safe_navigation_target("https://a.com\0"));

        assert!(is_safe_bookmark_url("https://example.com"));
        assert!(!is_safe_bookmark_url("javascript:alert(1)"));
        assert!(!is_safe_bookmark_url("data:text/html,1"));
    }

    #[test]
    fn route_resolution_is_exact() {
        assert_eq!(resolve_internal_route("settings", "/"), Some("settings"));
        assert_eq!(resolve_internal_route("newtab", ""), Some("newtab"));
        assert_eq!(resolve_internal_route("fynexo.localhost", "/settings"), Some("settings"));
        assert_eq!(resolve_internal_route("localhost", "/newtab"), Some("newtab"));
        assert_eq!(resolve_internal_route("localhost", "/"), Some("newtab"));
        assert_eq!(resolve_internal_route("settings-x", "/"), None);
        assert_eq!(resolve_internal_route("localhost", "/a/settings"), None);
        assert_eq!(resolve_internal_route("fynexo.localhost", "/settingsfoo"), None);
    }
}
