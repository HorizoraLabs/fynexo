pub mod defaults;
pub mod icons;
pub mod runtime;
pub mod styles;

use crate::config::{HudBootPayload, RuntimePayload, SiteRule};

pub fn generate_hud_script(payload: &RuntimePayload, tabs: Option<&crate::tabs::TabState>) -> String {
    let history_slice = &payload.history[..payload.history.len().min(25)];
    let enabled_rules: Vec<&SiteRule> = payload.site_rules.iter().filter(|r| r.enabled).collect();
    let slim_boot = HudBootPayload {
        config: &payload.config,
        styles: &payload.styles,
        active_theme: &payload.active_theme,
        keybinds: &payload.keybinds,
        enabled_rules,
        bookmarks: &payload.bookmarks,
        history: history_slice,
        tabs,
    };
    let initial_json = serde_json::to_string(&slim_boot).unwrap_or_else(|_| "{}".to_string());
    // Embed the per-process IPC secret (placeholder appears once, before any user data).
    build_script(&initial_json)
}

fn build_script(initial_json: &str) -> String {
    runtime::assemble_hud_script(initial_json)
        .replacen("__FNX_IPC_TOKEN__", crate::app::security::ipc_token(), 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_script_has_no_unresolved_placeholders_and_embeds_token() {
        let script = build_script("{}");
        for ph in ["__FNX_ICONS__", "__FNX_DEFAULTS__", "__FNX_BASE_CSS__", "__FNX_INITIAL_JSON__", "__FNX_IPC_TOKEN__"] {
            assert!(!script.contains(ph), "unresolved placeholder {ph}");
        }
        assert!(script.contains(crate::app::security::ipc_token()));
    }

    #[test]
    fn user_data_is_never_rescanned_for_placeholders() {
        let evil = r#"{"x":"__FNX_ICONS__ __FNX_IPC_TOKEN__"}"#;
        let out = runtime::assemble_hud_script(evil);
        assert!(out.contains(evil));
    }
}