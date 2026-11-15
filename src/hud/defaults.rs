use crate::config::{default_bangs, default_search_engines, ComponentStyles, KeybindsConfig};

pub fn defaults_script() -> String {
    let engines =
        serde_json::to_string(&default_search_engines()).unwrap_or_else(|_| "[]".to_string());
    let bangs = serde_json::to_string(&default_bangs()).unwrap_or_else(|_| "[]".to_string());
    let keybinds =
        serde_json::to_string(&KeybindsConfig::default()).unwrap_or_else(|_| "{}".to_string());
    let styles =
        serde_json::to_string(&ComponentStyles::default()).unwrap_or_else(|_| "{}".to_string());
    format!(
        "const DEFAULT_ENGINES = {engines};\nconst DEFAULT_BANGS = {bangs};\nconst DEFAULT_KEYBINDS = {keybinds};\nconst DEFAULT_STYLES = {styles};"
    )
}
