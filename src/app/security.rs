//! IPC trust boundary.
//!
//! wry exposes `window.ipc.postMessage` to *every* page, including arbitrary
//! websites. Without a check, any site could send privileged messages
//! (SAVE_RULES, IMPORT_FULL_SETUP_JSON, CLEAR_SITE_DATA, ...).
//!
//! Trust is granted when either:
//!  1. the message originates from an internal page (`fynexo://` or
//!     `*.fynexo.localhost`), or
//!  2. the message carries the per-process secret token, which is only embedded
//!     in the HUD initialization script (captured in a closure, never exposed
//!     as a global that page scripts can read).

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::OnceLock;

static IPC_TOKEN: OnceLock<String> = OnceLock::new();

/// Per-process random secret (128 bits from the OS-seeded std `RandomState`).
pub fn ipc_token() -> &'static str {
    IPC_TOKEN.get_or_init(|| {
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
    })
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

/// Decide whether an IPC message may be processed.
pub fn authorize_ipc(origin: &str, message: &serde_json::Value) -> bool {
    if is_internal_origin(origin) {
        return true;
    }
    message
        .get("__t")
        .and_then(|t| t.as_str())
        .map(|t| ct_eq(t, ipc_token()))
        .unwrap_or(false)
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
    fn external_origin_requires_valid_token() {
        let evil = "https://evil.example/";
        assert!(!authorize_ipc(evil, &json!({"type": "SAVE_RULES"})));
        assert!(!authorize_ipc(evil, &json!({"type": "SAVE_RULES", "__t": "guess"})));
        assert!(authorize_ipc(evil, &json!({"type": "NEW_TAB", "__t": ipc_token()})));
        assert!(authorize_ipc("fynexo://settings", &json!({"type": "SAVE_RULES"})));
    }

    #[test]
    fn token_is_stable_and_long() {
        assert_eq!(ipc_token(), ipc_token());
        assert_eq!(ipc_token().len(), 32);
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
