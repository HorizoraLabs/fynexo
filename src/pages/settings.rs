use crate::config::RuntimePayload;

const PAGE_TEMPLATE: &str = include_str!("settings.html");

pub fn settings_html(payload: &RuntimePayload) -> String {
    let payload_json = crate::util::json_for_html_script(payload);
    PAGE_TEMPLATE.replacen("__FNX_BOOT_JSON__", &payload_json, 1)
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
