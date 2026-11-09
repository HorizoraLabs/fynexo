use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::base::{
    display_path_preferring_local, synced_json_mtime, synced_json_path, synced_json_save,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct KeybindsConfig {
    pub omnibox: String,
    pub new_tab: String,
    pub close_tab: String,
    pub reopen_closed_tab: String,
    pub next_tab: String,
    pub bookmark_page: String,
    pub find_in_page: String,
    pub reader_mode: String,
    pub force_dark: String,
    pub fullscreen: String,
    pub preferences: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub zoom_reset: String,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            omnibox: "Ctrl+L".to_string(),
            new_tab: "Ctrl+T".to_string(),
            close_tab: "Ctrl+W".to_string(),
            reopen_closed_tab: "Ctrl+Shift+T".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            bookmark_page: "Ctrl+D".to_string(),
            find_in_page: "Ctrl+F".to_string(),
            reader_mode: "Alt+R".to_string(),
            force_dark: "Alt+Shift+D".to_string(),
            fullscreen: "F11".to_string(),
            preferences: "Ctrl+,".to_string(),
            zoom_in: "Ctrl+=".to_string(),
            zoom_out: "Ctrl+-".to_string(),
            zoom_reset: "Ctrl+0".to_string(),
        }
    }
}

impl KeybindsConfig {
    pub fn path() -> PathBuf {
        synced_json_path("keybinds.json")
    }

    pub fn path_display() -> String {
        display_path_preferring_local("keybinds.json")
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("keybinds.json")
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok(kb) = serde_json::from_str::<KeybindsConfig>(&raw) {
                    return kb;
                }
            }
        }
        let def = Self::default();
        def.save();
        def
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            synced_json_save("keybinds.json", &json);
        }
    }
}
