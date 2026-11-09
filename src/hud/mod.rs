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
    runtime::assemble_hud_script(&initial_json)
}
