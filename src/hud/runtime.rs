use super::defaults::DEFAULTS_SCRIPT;
use super::icons::ICONS_SCRIPT;
use super::styles::BASE_STYLES_CSS;

/// HUD runtime (plain JavaScript, lint/syntax-checkable with `node --check`).
/// Placeholders: __FNX_ICONS__, __FNX_DEFAULTS__, __FNX_BASE_CSS__, __FNX_INITIAL_JSON__
/// (and __FNX_IPC_TOKEN__, filled in by `hud::generate_hud_script`).
const HUD_RUNTIME_TEMPLATE: &str = include_str!("runtime.js");

pub fn assemble_hud_script(initial_json: &str) -> String {
    let base_css_json =
        serde_json::to_string(BASE_STYLES_CSS).unwrap_or_else(|_| "\"\"".to_string());
    // Static, trusted fragments first; untrusted/user-derived JSON last so its
    // content is never re-scanned for placeholders.
    HUD_RUNTIME_TEMPLATE
        .replacen("__FNX_ICONS__", ICONS_SCRIPT, 1)
        .replacen("__FNX_DEFAULTS__", DEFAULTS_SCRIPT, 1)
        .replacen("__FNX_BASE_CSS__", &base_css_json, 1)
        .replacen("__FNX_INITIAL_JSON__", initial_json, 1)
}
