use std::sync::OnceLock;

/// HUD runtime (plain JavaScript, lint/syntax-checkable with `node --check`).
/// Placeholders: __FNX_ICONS__, __FNX_I18N__, __FNX_DEFAULTS__, __FNX_BASE_CSS__, __FNX_SCRIPTFORGE__,
/// __FNX_EXTENSIONS__, __FNX_INITIAL_JSON__ (and __FNX_IPC_TOKEN__, __FNX_BRIDGE_TOKEN__, __FNX_ROLE__).
const HUD_RUNTIME_TEMPLATE: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_runtime.min.js"));
const CONTENT_AGENT_TEMPLATE: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_content_agent.min.js"));
const ICONS_SCRIPT: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_icons.min.js"));
const I18N_SCRIPT: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_i18n.min.js"));
const SCRIPTFORGE_SCRIPT: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_scriptforge.min.js"));
const EXTENSIONS_SCRIPT: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_extensions.min.js"));
const BASE_TOKENS_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_tokens.min.css"));
const BASE_STYLES_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_styles.min.css"));
const CONTENT_STYLES_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/hud_content_styles.min.css"));
const INITIAL_JSON_PLACEHOLDER: &str = "__FNX_INITIAL_JSON__";

/// HUD runtime role determining whether this WebView hosts only chrome (`HudShell`),
/// only page content (`ContentTab`), or legacy single-WebView chrome+content (`Embedded`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudRole {
    Embedded,
    HudShell,
    ContentTab,
}

impl HudRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::HudShell => "hud",
            Self::ContentTab => "content",
        }
    }
}

pub fn hud_shell_html() -> &'static str {
    concat!(
        "<!DOCTYPE html>",
        "<html lang=\"en\" data-fynexo-hud-shell=\"1\">",
        "<head>",
        "<meta charset=\"utf-8\">",
        "<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">",
        "<title>Fynexo HUD</title>",
        "<style>",
        include_str!("fonts.css"),
        "\nhtml,body{margin:0;padding:0;width:100%;height:100%;overflow:hidden;background:transparent!important;user-select:none;}</style>",
        "</head>",
        "<body></body>",
        "</html>"
    )
}

fn split_initial_json(filled: String) -> (String, String) {
    match filled.split_once(INITIAL_JSON_PLACEHOLDER) {
        Some((prefix, suffix)) => (prefix.to_string(), suffix.to_string()),
        None => (filled, String::new()),
    }
}

fn mobile_flag_str() -> &'static str {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        "true"
    } else {
        "false"
    }
}

/// Pre-baked static HUD prefix and suffix for legacy single-WebView (`Embedded`) mode.
pub fn baked_template_parts() -> &'static (String, String) {
    static BAKED: OnceLock<(String, String)> = OnceLock::new();
    BAKED.get_or_init(|| {
        let combined_css = format!("{}\n{}", BASE_TOKENS_CSS, BASE_STYLES_CSS);
        let base_css_json =
            serde_json::to_string(&combined_css).unwrap_or_else(|_| "\"\"".to_string());
        let defaults_js = super::defaults::defaults_script();
        let filled = HUD_RUNTIME_TEMPLATE
            .replacen("__FNX_MOBILE_HOST__", mobile_flag_str(), 1)
            .replacen("__FNX_ICONS__", ICONS_SCRIPT, 1)
            .replacen("__FNX_I18N__;", I18N_SCRIPT, 1)
            .replacen("__FNX_DEFAULTS__", &defaults_js, 1)
            .replacen("__FNX_BASE_CSS__", &base_css_json, 1)
            .replacen("__FNX_SCRIPTFORGE__;", SCRIPTFORGE_SCRIPT, 1)
            .replacen("__FNX_EXTENSIONS__;", EXTENSIONS_SCRIPT, 1);
        split_initial_json(filled)
    })
}

/// Pre-baked HUD shell template (`HudRole::HudShell`): omits external-page ScriptForge and
/// built-in content extensions since `hud_webview` hosts only Fynexo browser chrome.
pub fn baked_hud_shell_parts() -> &'static (String, String) {
    static BAKED_SHELL: OnceLock<(String, String)> = OnceLock::new();
    BAKED_SHELL.get_or_init(|| {
        let combined_css = format!("{}\n{}", BASE_TOKENS_CSS, BASE_STYLES_CSS);
        let base_css_json =
            serde_json::to_string(&combined_css).unwrap_or_else(|_| "\"\"".to_string());
        let defaults_js = super::defaults::defaults_script();
        let filled = HUD_RUNTIME_TEMPLATE
            .replacen("__FNX_MOBILE_HOST__", mobile_flag_str(), 1)
            .replacen("__FNX_ICONS__", ICONS_SCRIPT, 1)
            .replacen("__FNX_I18N__;", I18N_SCRIPT, 1)
            .replacen("__FNX_DEFAULTS__", &defaults_js, 1)
            .replacen("__FNX_BASE_CSS__", &base_css_json, 1)
            .replacen("__FNX_SCRIPTFORGE__;", "", 1)
            .replacen("__FNX_EXTENSIONS__;", "", 1);
        split_initial_json(filled)
    })
}

/// Pre-baked lightweight content-tab agent (`HudRole::ContentTab`): strips ~700 KB of unused
/// browser chrome UI, icons, and topbar CSS so external web tabs stay fast and lean.
pub fn baked_content_agent_parts() -> &'static (String, String) {
    static BAKED_CONTENT: OnceLock<(String, String)> = OnceLock::new();
    BAKED_CONTENT.get_or_init(|| {
        let content_css_json =
            serde_json::to_string(CONTENT_STYLES_CSS).unwrap_or_else(|_| "\"\"".to_string());
        let defaults_js = super::defaults::defaults_script();
        let filled = CONTENT_AGENT_TEMPLATE
            .replacen("__FNX_MOBILE_HOST__", mobile_flag_str(), 1)
            .replacen("__FNX_I18N__;", I18N_SCRIPT, 1)
            .replacen("__FNX_DEFAULTS__", &defaults_js, 1)
            .replacen("__FNX_CONTENT_CSS__", &content_css_json, 1)
            .replacen("__FNX_SCRIPTFORGE__;", SCRIPTFORGE_SCRIPT, 1)
            .replacen("__FNX_EXTENSIONS__;", EXTENSIONS_SCRIPT, 1);
        split_initial_json(filled)
    })
}

pub fn join_baked_parts((prefix, suffix): &(String, String), initial_json: &str) -> String {
    crate::util::join_split_template(prefix, initial_json, suffix)
}

pub fn assemble_hud_script(initial_json: &str) -> String {
    join_baked_parts(baked_template_parts(), initial_json)
}
