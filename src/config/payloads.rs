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
    #[serde(default)]
    pub workspaces: Vec<crate::tabs::WorkspaceSnapshot>,
    #[serde(default)]
    pub chrome_extensions: Vec<crate::chrome_ext::ChromeExtensionEntry>,
    #[serde(default)]
    pub chrome_ext_engine_active: bool,
    #[serde(default)]
    pub site_permissions:
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
    #[serde(default)]
    pub profiles: Vec<super::profiles::ProfileEntry>,
    #[serde(default)]
    pub active_profile_id: String,
    #[serde(default)]
    pub app_version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DownloadEntry {
    #[serde(default)]
    pub id: u64,
    pub name: String,
    pub status: String,
    pub updated_at: u64,
    #[serde(default)]
    pub bytes_received: u64,
    #[serde(default)]
    pub total_bytes: u64,
    #[serde(default)]
    pub speed_bps: u64,
    #[serde(skip)]
    pub saved_path: String,
}

/// Slim boot payload injected into external webpages (`hud.rs`) so disabled rules, inactive `.json`
/// themes, filesystem paths, and adblock lists consume 0 bytes and are never exposed to external pages.
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
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub mode_themes: std::collections::BTreeMap<String, &'a ThemeColors>,
    #[serde(skip_serializing_if = "<[DownloadEntry]>::is_empty")]
    pub downloads: &'a [DownloadEntry],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub chrome_extensions: Vec<crate::chrome_ext::HudChromeExtension>,
    pub chrome_ext_engine_active: bool,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub site_permissions:
        &'a std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "<[super::profiles::ProfileEntry]>::is_empty")]
    pub profiles: &'a [super::profiles::ProfileEntry],
    pub active_profile_id: &'a str,
}

pub fn collect_mode_themes<'a>(
    config: &AppConfig,
    themes: &'a [ThemePreset],
) -> std::collections::BTreeMap<String, &'a ThemeColors> {
    let mut map = std::collections::BTreeMap::new();
    for p in &config.mode_profiles {
        if !p.theme.is_empty() && !map.contains_key(&p.theme) {
            if let Some(found) = themes.iter().find(|t| t.id == p.theme) {
                map.insert(p.theme.clone(), &found.colors);
            }
        }
    }
    map
}

impl<'a> HudBootPayload<'a> {
    /// Build a minimal, external-safe HUD payload from a full `RuntimePayload`.
    pub fn from_runtime(payload: &'a RuntimePayload, tabs: Option<&'a crate::tabs::TabState>) -> Self {
        let history_slice = &payload.history[..payload.history.len().min(25)];
        let enabled_rules: Vec<&SiteRule> = payload.site_rules.iter().filter(|r| r.enabled).collect();
        let mode_themes = collect_mode_themes(&payload.config, &payload.themes);
        let chrome_extensions = payload
            .chrome_extensions
            .iter()
            .map(crate::chrome_ext::HudChromeExtension::from)
            .collect();
        Self {
            config: &payload.config,
            styles: &payload.styles,
            active_theme: &payload.active_theme,
            keybinds: &payload.keybinds,
            enabled_rules,
            bookmarks: &payload.bookmarks,
            history: history_slice,
            tabs,
            mode_themes,
            downloads: &[],
            chrome_extensions,
            chrome_ext_engine_active: payload.chrome_ext_engine_active,
            site_permissions: &payload.site_permissions,
            profiles: &payload.profiles,
            active_profile_id: &payload.active_profile_id,
        }
    }
}

/// Minimal privacy-preserving boot payload injected into Windows Dual-WebView content tabs.
/// Deliberately omits bookmarks, browsing history, downloads, and installed Chrome extensions.
#[derive(Serialize, Clone, Debug)]
pub struct ContentBootPayload<'a> {
    pub config: &'a AppConfig,
    pub active_theme: &'a ThemeColors,
    pub keybinds: &'a KeybindsConfig,
    pub enabled_rules: Vec<&'a SiteRule>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    pub adblock_whitelist: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<&'a crate::tabs::TabState>,
}

/// Slim boot payload for `fynexo://newtab` so opening a new tab avoids cloning or
/// serializing themes, site rules, keybinds, adblock domain lists, or filesystem paths.
#[derive(Serialize, Clone, Debug)]
pub struct NewTabBootPayload<'a> {
    pub config: &'a AppConfig,
    pub styles: &'a ComponentStyles,
    pub active_theme: &'a ThemeColors,
    pub bookmarks: &'a [BookmarkItem],
    pub history: &'a [HistoryItem],
    pub blocked_ads: usize,
}

impl<'a> NewTabBootPayload<'a> {
    pub fn from_runtime(payload: &'a RuntimePayload) -> Self {
        let history_slice = &payload.history[..payload.history.len().min(40)];
        Self {
            config: &payload.config,
            styles: &payload.styles,
            active_theme: &payload.active_theme,
            bookmarks: &payload.bookmarks,
            history: history_slice,
            blocked_ads: payload.blocked_ads,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::app::BrowserModeProfile;

    #[test]
    fn collect_mode_themes_includes_only_referenced_themes() {
        let cfg = AppConfig {
            mode_profiles: vec![BrowserModeProfile {
                id: "dev".to_string(),
                theme: "my-custom".to_string(),
                ..BrowserModeProfile::default()
            }],
            ..AppConfig::default()
        };
        let themes = vec![
            ThemePreset {
                id: "my-custom".to_string(),
                name: "My Custom".to_string(),
                author: String::new(),
                description: String::new(),
                builtin: false,
                colors: ThemeColors::default(),
            },
            ThemePreset {
                id: "unused-secret".to_string(),
                name: "Unused".to_string(),
                author: String::new(),
                description: String::new(),
                builtin: false,
                colors: ThemeColors::default(),
            },
        ];
        let map = collect_mode_themes(&cfg, &themes);
        assert!(map.contains_key("my-custom"));
        assert!(!map.contains_key("unused-secret"));
        assert_eq!(map.len(), 1);
    }
}
