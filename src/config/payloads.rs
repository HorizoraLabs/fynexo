use serde::{Deserialize, Serialize};

use crate::adblock::AdblockConfig;
use super::app::AppConfig;
use super::keybinds::KeybindsConfig;
use super::rules::{default_site_rules, SiteRule};
use super::styles::ComponentStyles;
use super::themes::{ThemeColors, ThemePreset};
use super::userdata::{default_shortcuts, BookmarkItem, HistoryItem};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SetupBundle {
    pub fynexo_bundle_version: u32,
    pub config: AppConfig,
    pub styles: ComponentStyles,
    pub custom_themes: Vec<ThemePreset>,
    pub shortcuts: Vec<BookmarkItem>,
    pub site_rules: Vec<SiteRule>,
    pub adblock: AdblockConfig,
    pub keybinds: KeybindsConfig,
}

impl Default for SetupBundle {
    fn default() -> Self {
        Self {
            fynexo_bundle_version: 1,
            config: AppConfig::default(),
            styles: ComponentStyles::default(),
            custom_themes: Vec::new(),
            shortcuts: default_shortcuts(),
            site_rules: default_site_rules(),
            adblock: AdblockConfig::default(),
            keybinds: KeybindsConfig::default(),
        }
    }
}

/// Full runtime payload for `fynexo://settings`, `fynexo://newtab`, and runtime sync.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimePayload {
    pub config: AppConfig,
    pub styles: ComponentStyles,
    pub active_theme: ThemeColors,
    pub themes: Vec<ThemePreset>,
    pub themes_dir: String,
    pub styles_path: String,
    pub shortcuts_path: String,
    pub rules_path: String,
    pub adblock_path: String,
    pub keybinds_path: String,
    pub data_dir: String,
    pub bookmarks: Vec<BookmarkItem>,
    pub history: Vec<HistoryItem>,
    pub site_rules: Vec<SiteRule>,
    pub adblock: AdblockConfig,
    pub keybinds: KeybindsConfig,
    pub blocked_ads: usize,
}

/// Slim boot payload injected into external webpages (`hud.rs`) so disabled rules, inactive `.json`
/// themes, and adblock lists consume 0 bytes of per-webpage DOM/JS memory.
#[derive(Serialize, Clone, Debug)]
pub struct HudBootPayload<'a> {
    pub config: &'a AppConfig,
    pub styles: &'a ComponentStyles,
    pub active_theme: &'a ThemeColors,
    pub keybinds: &'a KeybindsConfig,
    pub enabled_rules: Vec<&'a SiteRule>,
    pub bookmarks: &'a [BookmarkItem],
    pub history: &'a [HistoryItem],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<&'a crate::tabs::TabState>,
}
