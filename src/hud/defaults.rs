use crate::config::{
    builtin_theme_presets, default_bangs, default_mode_profiles, default_search_engines, ComponentStyles, KeybindsConfig,
};

pub fn defaults_script() -> String {
    let engines =
        serde_json::to_string(&default_search_engines()).unwrap_or_else(|_| "[]".to_string());
    let bangs = serde_json::to_string(&default_bangs()).unwrap_or_else(|_| "[]".to_string());
    let keybinds =
        serde_json::to_string(&KeybindsConfig::default()).unwrap_or_else(|_| "{}".to_string());
    let styles =
        serde_json::to_string(&ComponentStyles::default()).unwrap_or_else(|_| "{}".to_string());
    let mode_profiles =
        serde_json::to_string(&default_mode_profiles()).unwrap_or_else(|_| "[]".to_string());
    let themes: std::collections::BTreeMap<String, crate::config::ThemeColors> = builtin_theme_presets()
        .into_iter()
        .map(|t| (t.id, t.colors))
        .collect();
    let themes = serde_json::to_string(&themes).unwrap_or_else(|_| "{}".to_string());
    format!(
        "const DEFAULT_ENGINES = {engines};\nconst DEFAULT_BANGS = {bangs};\nconst DEFAULT_KEYBINDS = {keybinds};\nconst DEFAULT_STYLES = {styles};\nconst DEFAULT_MODE_PROFILES = {mode_profiles};\nconst BUILTIN_THEME_COLORS = {themes};"
    )
}
