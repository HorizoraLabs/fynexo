use std::sync::OnceLock;

/// HUD runtime (plain JavaScript, lint/syntax-checkable with `node --check`).
/// Placeholders: __FNX_ICONS__, __FNX_DEFAULTS__, __FNX_BASE_CSS__, __FNX_SCRIPTFORGE__, __FNX_EXTENSIONS__,
/// __FNX_INITIAL_JSON__ (and __FNX_IPC_TOKEN__, __FNX_BRIDGE_TOKEN__, filled in by `hud::generate_hud_script`).
const HUD_RUNTIME_TEMPLATE: &str = include_str!("runtime.js");
const ICONS_SCRIPT: &str = include_str!("icons.js");
const SCRIPTFORGE_SCRIPT: &str = include_str!("scriptforge.js");
const EXTENSIONS_SCRIPT: &str = include_str!("extensions.js");
const BASE_STYLES_CSS: &str = include_str!("styles.css");
const INITIAL_JSON_PLACEHOLDER: &str = "__FNX_INITIAL_JSON__";

/// Pre-baked static HUD prefix and suffix split around `__FNX_INITIAL_JSON__`.
/// Built once per process so creating new tabs avoids re-serializing CSS and
/// re-scanning the entire runtime script.
pub fn baked_template_parts() -> &'static (String, String) {
    static BAKED: OnceLock<(String, String)> = OnceLock::new();
    BAKED.get_or_init(|| {
        let base_css_json =
            serde_json::to_string(BASE_STYLES_CSS).unwrap_or_else(|_| "\"\"".to_string());
        let mobile_flag = if cfg!(any(target_os = "android", target_os = "ios")) {
            "true"
        } else {
            "false"
        };
        let defaults_js = super::defaults::defaults_script();
        let filled = HUD_RUNTIME_TEMPLATE
            .replacen("__FNX_MOBILE_HOST__", mobile_flag, 1)
            .replacen("__FNX_ICONS__", ICONS_SCRIPT, 1)
            .replacen("__FNX_DEFAULTS__", &defaults_js, 1)
            .replacen("__FNX_BASE_CSS__", &base_css_json, 1)
            .replacen("__FNX_SCRIPTFORGE__;", SCRIPTFORGE_SCRIPT, 1)
            .replacen("__FNX_EXTENSIONS__;", EXTENSIONS_SCRIPT, 1);
        match filled.split_once(INITIAL_JSON_PLACEHOLDER) {
            Some((prefix, suffix)) => (prefix.to_string(), suffix.to_string()),
            None => (filled, String::new()),
        }
    })
}

pub fn join_baked_parts((prefix, suffix): &(String, String), initial_json: &str) -> String {
    crate::util::join_split_template(prefix, initial_json, suffix)
}

pub fn assemble_hud_script(initial_json: &str) -> String {
    join_baked_parts(baked_template_parts(), initial_json)
}
