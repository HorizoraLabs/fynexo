use std::sync::OnceLock;

use crate::config::{NewTabBootPayload, RuntimePayload};

const PAGE_TEMPLATE: &str = include_str!("newtab.html");

fn baked_parts() -> &'static (&'static str, &'static str) {
    static PARTS: OnceLock<(&'static str, &'static str)> = OnceLock::new();
    PARTS.get_or_init(|| crate::util::split_template_once(PAGE_TEMPLATE, "__FNX_BOOT_JSON__"))
}

pub fn newtab_html_from_json(payload_json: &str) -> String {
    let (prefix, suffix) = *baked_parts();
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
        let html = PAGE_TEMPLATE.replacen("__FNX_BOOT_JSON__", "{}", 1);
        assert!(!html.contains("__FNX_BOOT_JSON__"));
        assert!(html.contains("const boot = {} || {};"));
    }
}
