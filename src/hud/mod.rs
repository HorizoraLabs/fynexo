pub mod defaults;
pub mod runtime;

use std::sync::OnceLock;

use crate::config::{HudBootPayload, RuntimePayload};

pub use runtime::{hud_shell_html, HudRole};

pub fn generate_hud_script(payload: &RuntimePayload, tabs: Option<&crate::tabs::TabState>) -> String {
    let slim_boot = HudBootPayload::from_runtime(payload, tabs);
    let initial_json = serde_json::to_string(&slim_boot).unwrap_or_else(|_| "{}".to_string());
    build_script_for_role(&initial_json, HudRole::Embedded)
}

pub fn generate_hud_script_from_state(
    state: &crate::app::AppState,
    tabs: Option<&crate::tabs::TabState>,
) -> String {
    generate_hud_script_for_role(state, tabs, HudRole::Embedded)
}

pub fn generate_hud_script_for_role(
    state: &crate::app::AppState,
    tabs: Option<&crate::tabs::TabState>,
    role: HudRole,
) -> String {
    let initial_json = state
        .hud_json_with_tabs(tabs)
        .unwrap_or_else(|_| "{}".to_string());
    build_script_for_role(&initial_json, role)
}

fn baked_role_parts(slot: &OnceLock<(String, String)>, role: HudRole) -> &(String, String) {
    slot.get_or_init(|| {
        let (raw_prefix, raw_suffix) = runtime::baked_template_parts();
        let ipc = crate::app::security::ipc_token();
        let bridge = crate::app::security::bridge_token();
        let inject = |part: &str| {
            part.replacen("__FNX_IPC_TOKEN__", ipc, 1)
                .replacen("__FNX_BRIDGE_TOKEN__", bridge, 1)
                .replacen("__FNX_ROLE__", role.as_str(), 1)
        };
        (inject(raw_prefix), inject(raw_suffix))
    })
}

fn build_script_for_role(initial_json: &str, role: HudRole) -> String {
    static BAKED_EMBEDDED: OnceLock<(String, String)> = OnceLock::new();
    static BAKED_HUD_SHELL: OnceLock<(String, String)> = OnceLock::new();
    static BAKED_CONTENT_TAB: OnceLock<(String, String)> = OnceLock::new();
    let slot = match role {
        HudRole::Embedded => &BAKED_EMBEDDED,
        HudRole::HudShell => &BAKED_HUD_SHELL,
        HudRole::ContentTab => &BAKED_CONTENT_TAB,
    };
    let parts = baked_role_parts(slot, role);
    runtime::join_baked_parts(parts, initial_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_script_has_no_unresolved_placeholders_and_embeds_tokens() {
        for (role, expected) in [
            (HudRole::Embedded, "const __FNX_ROLE = 'embedded';"),
            (HudRole::HudShell, "const __FNX_ROLE = 'hud';"),
            (HudRole::ContentTab, "const __FNX_ROLE = 'content';"),
        ] {
            let script = build_script_for_role("{}", role);
            for ph in [
                "__FNX_MOBILE_HOST__",
                "__FNX_ICONS__",
                "__FNX_DEFAULTS__",
                "__FNX_BASE_CSS__",
                "__FNX_SCRIPTFORGE__",
                "__FNX_EXTENSIONS__",
                "__FNX_INITIAL_JSON__",
                "__FNX_IPC_TOKEN__",
                "__FNX_BRIDGE_TOKEN__",
                "__FNX_ROLE__",
            ] {
                assert!(!script.contains(ph), "unresolved placeholder {ph} for {:?}", role);
            }
            assert!(script.contains(crate::app::security::ipc_token()));
            assert!(script.contains(crate::app::security::bridge_token()));
            assert!(script.contains(expected));
        }
        assert!(hud_shell_html().contains("data-fynexo-hud-shell=\"1\""));
    }

    #[test]
    fn user_data_is_never_rescanned_for_placeholders() {
        let evil = r#"{"x":"__FNX_ICONS__ __FNX_IPC_TOKEN__ __FNX_BRIDGE_TOKEN__"}"#;
        let out = runtime::assemble_hud_script(evil);
        assert!(out.contains(evil));
    }
}
