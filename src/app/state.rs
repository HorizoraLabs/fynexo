use std::time::SystemTime;

use crate::adblock::{self, AdblockConfig};
use crate::config::{
    data_dir_display, load_all_themes, themes_dir_display, themes_stamp, AppConfig,
    ComponentStyles, KeybindsConfig, RuntimePayload, SiteRule, SiteRulesStore, ThemeColors,
    ThemePreset, UserData,
};

pub struct AppState {
    pub config: AppConfig,
    pub styles: ComponentStyles,
    pub themes: Vec<ThemePreset>,
    pub userdata: UserData,
    pub site_rules: Vec<SiteRule>,
    pub adblock: AdblockConfig,
    pub keybinds: KeybindsConfig,
    pub config_mtime: Option<SystemTime>,
    pub styles_mtime: Option<SystemTime>,
    pub shortcuts_mtime: Option<SystemTime>,
    pub rules_mtime: Option<SystemTime>,
    pub adblock_mtime: Option<SystemTime>,
    pub keybinds_mtime: Option<SystemTime>,
    pub themes_stamp: (usize, Option<SystemTime>),
}

impl AppState {
    pub fn load() -> Self {
        let config = AppConfig::load();
        adblock::set_enabled(config.adblock);
        let styles = ComponentStyles::load();
        let themes = load_all_themes();
        let userdata = UserData::load();
        let site_rules = SiteRulesStore::load();
        let adblock_cfg = AdblockConfig::load();
        adblock::sync_dynamic_lists(&adblock_cfg);
        let keybinds = KeybindsConfig::load();

        Self {
            config,
            styles,
            themes,
            userdata,
            site_rules,
            adblock: adblock_cfg,
            keybinds,
            config_mtime: AppConfig::mtime(),
            styles_mtime: ComponentStyles::mtime(),
            shortcuts_mtime: UserData::shortcuts_mtime(),
            rules_mtime: SiteRulesStore::mtime(),
            adblock_mtime: AdblockConfig::mtime(),
            keybinds_mtime: KeybindsConfig::mtime(),
            themes_stamp: themes_stamp(),
        }
    }

    pub fn resolve_active_theme(&self) -> ThemeColors {
        self.themes
            .iter()
            .find(|t| t.id.eq_ignore_ascii_case(&self.config.theme))
            .or_else(|| self.themes.iter().find(|t| t.id == "dark"))
            .or_else(|| self.themes.first())
            .map(|t| t.colors.clone())
            .unwrap_or_default()
    }

    pub fn reload_if_changed(&mut self) -> bool {
        let cur_cfg_mtime = AppConfig::mtime();
        let cur_sty_mtime = ComponentStyles::mtime();
        let cur_sc_mtime = UserData::shortcuts_mtime();
        let cur_rules_mtime = SiteRulesStore::mtime();
        let cur_adb_mtime = AdblockConfig::mtime();
        let cur_kb_mtime = KeybindsConfig::mtime();
        let cur_th_stamp = themes_stamp();
        let mut changed = false;

        if cur_cfg_mtime != self.config_mtime {
            self.config = AppConfig::load();
            adblock::set_enabled(self.config.adblock);
            self.config_mtime = AppConfig::mtime();
            changed = true;
        }
        if cur_sty_mtime != self.styles_mtime {
            self.styles = ComponentStyles::load();
            self.styles_mtime = ComponentStyles::mtime();
            changed = true;
        }
        if cur_sc_mtime != self.shortcuts_mtime {
            self.userdata.reload_shortcuts();
            self.shortcuts_mtime = UserData::shortcuts_mtime();
            changed = true;
        }
        if cur_rules_mtime != self.rules_mtime {
            self.site_rules = SiteRulesStore::load();
            self.rules_mtime = SiteRulesStore::mtime();
            changed = true;
        }
        if cur_adb_mtime != self.adblock_mtime {
            self.adblock = AdblockConfig::load();
            adblock::sync_dynamic_lists(&self.adblock);
            self.adblock_mtime = AdblockConfig::mtime();
            changed = true;
        }
        if cur_kb_mtime != self.keybinds_mtime {
            self.keybinds = KeybindsConfig::load();
            self.keybinds_mtime = KeybindsConfig::mtime();
            changed = true;
        }
        if cur_th_stamp != self.themes_stamp {
            self.themes = load_all_themes();
            self.themes_stamp = themes_stamp();
            changed = true;
        }
        changed
    }

    pub fn force_reload(&mut self) {
        self.config = AppConfig::load();
        adblock::set_enabled(self.config.adblock);
        self.styles = ComponentStyles::load();
        self.themes = load_all_themes();
        self.userdata = UserData::load();
        self.site_rules = SiteRulesStore::load();
        self.adblock = AdblockConfig::load();
        adblock::sync_dynamic_lists(&self.adblock);
        self.keybinds = KeybindsConfig::load();

        self.config_mtime = AppConfig::mtime();
        self.styles_mtime = ComponentStyles::mtime();
        self.shortcuts_mtime = UserData::shortcuts_mtime();
        self.rules_mtime = SiteRulesStore::mtime();
        self.adblock_mtime = AdblockConfig::mtime();
        self.keybinds_mtime = KeybindsConfig::mtime();
        self.themes_stamp = themes_stamp();
    }

    pub fn payload(&self) -> RuntimePayload {
        RuntimePayload {
            config: self.config.clone(),
            styles: self.styles.clone(),
            active_theme: self.resolve_active_theme(),
            themes: self.themes.clone(),
            themes_dir: themes_dir_display(),
            styles_path: ComponentStyles::styles_path_display(),
            shortcuts_path: UserData::shortcuts_path_display(),
            rules_path: SiteRulesStore::path_display(),
            adblock_path: AdblockConfig::path_display(),
            keybinds_path: KeybindsConfig::path_display(),
            data_dir: data_dir_display(),
            bookmarks: self.userdata.bookmarks.clone(),
            history: self.userdata.history.iter().take(40).cloned().collect(),
            site_rules: self.site_rules.clone(),
            adblock: self.adblock.clone(),
            keybinds: self.keybinds.clone(),
            blocked_ads: adblock::blocked_count(),
        }
    }

    /// Serialize the lightweight external-safe HUD payload directly from `&self`
    /// without cloning `themes`, `adblock`, or calling filesystem `canonicalize` paths.
    pub fn hud_json(&self) -> Result<String, serde_json::Error> {
        let active_theme = self.resolve_active_theme();
        let enabled_rules: Vec<&SiteRule> = self.site_rules.iter().filter(|r| r.enabled).collect();
        let history_slice = &self.userdata.history[..self.userdata.history.len().min(25)];
        let slim = crate::config::HudBootPayload {
            config: &self.config,
            styles: &self.styles,
            active_theme: &active_theme,
            keybinds: &self.keybinds,
            enabled_rules,
            bookmarks: &self.userdata.bookmarks,
            history: history_slice,
            tabs: None,
        };
        serde_json::to_string(&slim)
    }
}
