use std::sync::OnceLock;

use serde::Serialize;

use crate::config::RuntimePayload;

const PAGE_TEMPLATE: &str = include_str!("settings.html");
const TOKENS_CSS: &str = include_str!("../hud/tokens.css");
const PAGE_CSS: &str = include_str!("settings.css");
const PAGE_JS: &str = include_str!("settings.js");

fn assemble_template() -> String {
    let shared_prelude = format!(
        "{}\n{}",
        crate::hud::defaults::defaults_script(),
        include_str!("../hud/icons.js")
    );
    let combined_css = format!("{}\n{}", TOKENS_CSS, PAGE_CSS);
    PAGE_TEMPLATE
        .replacen("__FNX_SETTINGS_CSS__", &combined_css, 1)
        .replacen("__FNX_SETTINGS_JS__", PAGE_JS, 1)
        .replacen("__FNX_DEFAULTS__;", &shared_prelude, 1)
}

#[derive(Serialize)]
struct BakedSupabaseConfig {
    url: &'static str,
    #[serde(rename = "anonKey")]
    anon_key: &'static str,
    schema: &'static str,
    table: &'static str,
}

fn baked_parts() -> &'static (String, String) {
    static PARTS: OnceLock<(String, String)> = OnceLock::new();
    PARTS.get_or_init(|| {
        let raw_url = option_env!("DATABASE_URL").unwrap_or("").trim();
        let safe_url = if raw_url.to_ascii_lowercase().starts_with("postgres://")
            || raw_url.to_ascii_lowercase().starts_with("postgresql://")
        {
            ""
        } else {
            raw_url
        };
        let supa = BakedSupabaseConfig {
            url: safe_url,
            anon_key: option_env!("DATABASE_ANON_KEY").unwrap_or("").trim(),
            schema: option_env!("DATABASE_SCHEMA").unwrap_or("").trim(),
            table: match option_env!("DATABASE_TABLE") {
                Some(t) if !t.trim().is_empty() => t.trim(),
                _ => "extensions",
            },
        };
        let supa_json = crate::util::json_for_html_script(&supa);
        let assembled = assemble_template();
        let (prefix, suffix) = match assembled.split_once("__FNX_BOOT_JSON__") {
            Some((p, s)) => (p.to_string(), s.to_string()),
            None => (assembled, String::new()),
        };
        (
            prefix.replacen("__FNX_SUPABASE_JSON__", &supa_json, 1),
            suffix,
        )
    })
}

pub fn settings_html(payload: &RuntimePayload) -> String {
    let payload_json = crate::util::json_for_html_script(payload);
    let (prefix, suffix) = baked_parts();
    crate::util::join_split_template(prefix, &payload_json, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_template_replaces_placeholder_cleanly() {
        let html = assemble_template()
            .replacen("__FNX_SUPABASE_JSON__", "{}", 1)
            .replacen("__FNX_BOOT_JSON__", "{}", 1);
        assert!(!html.contains("__FNX_SETTINGS_CSS__"));
        assert!(!html.contains("__FNX_SETTINGS_JS__"));
        assert!(!html.contains("__FNX_DEFAULTS__"));
        assert!(!html.contains("__FNX_SUPABASE_JSON__"));
        assert!(!html.contains("__FNX_BOOT_JSON__"));
        assert!(html.contains("const SUPABASE_CONFIG = {} || {};"));
        assert!(html.contains("const boot = {} || {};"));
    }
}
