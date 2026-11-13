use std::sync::OnceLock;

use crate::config::RuntimePayload;

const PAGE_TEMPLATE: &str = include_str!("settings.html");

fn baked_parts() -> &'static (&'static str, &'static str) {
    static PARTS: OnceLock<(&'static str, &'static str)> = OnceLock::new();
    PARTS.get_or_init(|| crate::util::split_template_once(PAGE_TEMPLATE, "__FNX_BOOT_JSON__"))
}

pub fn settings_html(payload: &RuntimePayload) -> String {
    let payload_json = crate::util::json_for_html_script(payload);
    let (prefix, suffix) = *baked_parts();
    crate::util::join_split_template(prefix, &payload_json, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_template_replaces_placeholder_cleanly() {
        let html = PAGE_TEMPLATE.replacen("__FNX_BOOT_JSON__", "{}", 1);
        assert!(!html.contains("__FNX_BOOT_JSON__"));
        assert!(html.contains("const boot = {} || {};"));
    }
}
