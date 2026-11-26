use std::sync::OnceLock;

use crate::config::{NewTabBootPayload, RuntimePayload};

const PAGE_TEMPLATE: &str = include_str!("newtab.html");
const FONTS_CSS: &str = include_str!("../hud/fonts.css");
const TOKENS_CSS: &str = include_str!("../hud/tokens.css");
const PAGE_CSS: &str = include_str!("newtab.css");
const PAGE_JS: &str = include_str!("newtab.js");

fn assemble_template() -> String {
    let shared_prelude = format!(
        "{}\n{}\n{}",
        crate::hud::defaults::defaults_script(),
        include_str!("../hud/i18n.js"),
        include_str!("../hud/icons.js")
    );
    let combined_css = format!("{}\n{}\n{}", FONTS_CSS, TOKENS_CSS, PAGE_CSS);
    PAGE_TEMPLATE
        .replacen("__FNX_NEWTAB_CSS__", &combined_css, 1)
        .replacen("__FNX_NEWTAB_JS__", PAGE_JS, 1)
        .replacen("__FNX_DEFAULTS__;", &shared_prelude, 1)
}

fn baked_parts() -> &'static (String, String) {
    static PARTS: OnceLock<(String, String)> = OnceLock::new();
    PARTS.get_or_init(|| {
        let assembled = assemble_template();
        match assembled.split_once("__FNX_BOOT_JSON__") {
            Some((prefix, suffix)) => (prefix.to_string(), suffix.to_string()),
            None => (assembled, String::new()),
        }
    })
}

pub fn newtab_html_from_json(payload_json: &str) -> String {
    let (prefix, suffix) = baked_parts();
    crate::util::join_split_template(prefix, payload_json, suffix)
}

pub fn newtab_html(payload: &RuntimePayload) -> String {
    let slim = NewTabBootPayload::from_runtime(payload);
    let payload_json = crate::util::json_for_html_script(&slim);
    newtab_html_from_json(&payload_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtab_template_replaces_placeholder_cleanly() {
        let html = assemble_template().replacen("__FNX_BOOT_JSON__", "{}", 1);
        assert!(!html.contains("__FNX_NEWTAB_CSS__"));
        assert!(!html.contains("__FNX_NEWTAB_JS__"));
        assert!(!html.contains("__FNX_DEFAULTS__"));
        assert!(!html.contains("__FNX_BOOT_JSON__"));
        assert!(html.contains("const boot = {} || {};"));
    }
}
