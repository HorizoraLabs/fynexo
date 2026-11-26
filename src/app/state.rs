use std::time::{Duration, Instant, SystemTime};

use crate::adblock::{self, AdblockConfig};
use crate::chrome_ext::{self, ChromeExtensionEntry, HudChromeExtension};
use crate::config::{
    active_profile_id, data_dir_display, load_all_themes, load_profiles, now_unix_secs,
    themes_dir_display, themes_stamp, AppConfig, ComponentStyles, DownloadEntry, KeybindsConfig,
    ProfileEntry, RuntimePayload, SitePermissionsStore, SiteRule, SiteRulesStore, ThemeColors,
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
    pub site_permissions: SitePermissionsStore,
    pub profiles: Vec<ProfileEntry>,
    pub active_profile_id: String,
    pub chrome_extensions: Vec<ChromeExtensionEntry>,
    pub boot_hw_accel: bool,
    pub boot_chrome_ext_enabled: bool,
    pub recent_downloads: Vec<DownloadEntry>,
    pub config_mtime: Option<SystemTime>,
    pub styles_mtime: Option<SystemTime>,
    pub shortcuts_mtime: Option<SystemTime>,
    pub rules_mtime: Option<SystemTime>,
    pub adblock_mtime: Option<SystemTime>,
    pub keybinds_mtime: Option<SystemTime>,
    pub permissions_mtime: Option<SystemTime>,
    pub themes_stamp: (usize, Option<SystemTime>),
    last_reload_check: Instant,
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
        let site_permissions = SitePermissionsStore::load();
        let profiles = load_profiles();
        let active_profile = active_profile_id().to_string();
        let chrome_extensions = chrome_ext::load_registry();
        let boot_hw_accel = config.hw_accel;
        let boot_chrome_ext_enabled =
            cfg!(target_os = "windows") && config.chrome_extensions_enabled;

        Self {
            config,
            styles,
            themes,
            userdata,
            site_rules,
            adblock: adblock_cfg,
            keybinds,
            site_permissions,
            profiles,
            active_profile_id: active_profile,
            chrome_extensions,
            boot_hw_accel,
            boot_chrome_ext_enabled,
            recent_downloads: Vec::new(),
            config_mtime: AppConfig::mtime(),
            styles_mtime: ComponentStyles::mtime(),
            shortcuts_mtime: UserData::shortcuts_mtime(),
            rules_mtime: SiteRulesStore::mtime(),
            adblock_mtime: AdblockConfig::mtime(),
            keybinds_mtime: KeybindsConfig::mtime(),
            permissions_mtime: SitePermissionsStore::mtime(),
            themes_stamp: themes_stamp(),
            last_reload_check: Instant::now(),
        }
    }

    /// Upsert a download item with live progress metrics.
    /// Returns `Some(status)` only when a download starts or transitions to a terminal state,
    /// so progress ticks never spam toasts.
    #[allow(clippy::too_many_arguments)]
    pub fn record_download(
        &mut self,
        id: u64,
        name: String,
        status: String,
        bytes_received: u64,
        total_bytes: u64,
        speed_bps: u64,
        saved_path: Option<String>,
    ) -> Option<String> {
        let now = now_unix_secs();
        let pos_opt = if id != 0 {
            self.recent_downloads.iter().position(|d| d.id == id)
        } else {
            self.recent_downloads.iter().position(|d| d.name == name)
        };
        if let Some(pos) = pos_opt {
            let status_changed = self.recent_downloads[pos].status != status;
            self.recent_downloads[pos].name = name;
            self.recent_downloads[pos].status = status.clone();
            self.recent_downloads[pos].updated_at = now;
            self.recent_downloads[pos].bytes_received = bytes_received;
            self.recent_downloads[pos].total_bytes = total_bytes;
            self.recent_downloads[pos].speed_bps = speed_bps;
            if let Some(p) = saved_path {
                if !p.is_empty() {
                    self.recent_downloads[pos].saved_path = p;
                }
            }
            if status_changed && pos > 0 {
                let item = self.recent_downloads.remove(pos);
                self.recent_downloads.insert(0, item);
            }
            if status_changed {
                Some(status)
            } else {
                None
            }
        } else {
            self.recent_downloads.insert(
                0,
                DownloadEntry {
                    id,
                    name,
                    status: status.clone(),
                    updated_at: now,
                    bytes_received,
                    total_bytes,
                    speed_bps,
                    saved_path: saved_path.unwrap_or_default(),
                },
            );
            if self.recent_downloads.len() > 20 {
                self.recent_downloads.truncate(20);
            }
            Some(status)
        }
    }

    pub fn resolve_download_path(&self, id: u64, filename: &str) -> Option<std::path::PathBuf> {
        let found = if id != 0 {
            self.recent_downloads.iter().find(|d| d.id == id)
        } else {
            None
        }
        .or_else(|| self.recent_downloads.iter().find(|d| d.name == filename));
        found.and_then(|d| {
            let p = d.saved_path.trim();
            if p.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(p))
            }
        })
    }

    pub fn resolve_active_theme_ref(&self) -> Option<&ThemeColors> {
        self.themes
            .iter()
            .find(|t| t.id.eq_ignore_ascii_case(&self.config.theme))
            .or_else(|| self.themes.iter().find(|t| t.id == "dark"))
            .or_else(|| self.themes.first())
            .map(|t| &t.colors)
    }

    pub fn resolve_active_theme(&self) -> ThemeColors {
        self.resolve_active_theme_ref().cloned().unwrap_or_default()
    }

    fn default_theme_colors() -> &'static ThemeColors {
        static DEFAULT: std::sync::OnceLock<ThemeColors> = std::sync::OnceLock::new();
        DEFAULT.get_or_init(ThemeColors::default)
    }

    pub fn reload_if_changed(&mut self) -> bool {
        const RELOAD_THROTTLE: Duration = Duration::from_millis(1500);
        if self.last_reload_check.elapsed() < RELOAD_THROTTLE {
            return false;
        }
        self.last_reload_check = Instant::now();
        let cur_cfg_mtime = AppConfig::mtime();
        let cur_sty_mtime = ComponentStyles::mtime();
        let cur_sc_mtime = UserData::shortcuts_mtime();
        let cur_rules_mtime = SiteRulesStore::mtime();
        let cur_adb_mtime = AdblockConfig::mtime();
        let cur_kb_mtime = KeybindsConfig::mtime();
        let cur_perm_mtime = SitePermissionsStore::mtime();
        let cur_th_stamp = themes_stamp();
        let mut changed = false;

        if cur_cfg_mtime != self.config_mtime {
            self.config = AppConfig::load();
            adblock::set_enabled(self.config.adblock);
            self.config_mtime = cur_cfg_mtime;
            changed = true;
        }
        if cur_sty_mtime != self.styles_mtime {
            self.styles = ComponentStyles::load();
            self.styles_mtime = cur_sty_mtime;
            changed = true;
        }
        if cur_sc_mtime != self.shortcuts_mtime {
            self.userdata.reload_shortcuts();
            self.shortcuts_mtime = cur_sc_mtime;
            changed = true;
        }
        if cur_rules_mtime != self.rules_mtime {
            self.site_rules = SiteRulesStore::load();
            self.rules_mtime = cur_rules_mtime;
            changed = true;
        }
        if cur_adb_mtime != self.adblock_mtime {
            self.adblock = AdblockConfig::load();
            adblock::sync_dynamic_lists(&self.adblock);
            self.adblock_mtime = cur_adb_mtime;
            changed = true;
        }
        if cur_kb_mtime != self.keybinds_mtime {
            self.keybinds = KeybindsConfig::load();
            self.keybinds_mtime = cur_kb_mtime;
            changed = true;
        }
        if cur_perm_mtime != self.permissions_mtime {
            self.site_permissions = SitePermissionsStore::load();
            self.permissions_mtime = cur_perm_mtime;
            changed = true;
        }
        if cur_th_stamp != self.themes_stamp {
            self.themes = load_all_themes();
            self.themes_stamp = cur_th_stamp;
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
        self.site_permissions = SitePermissionsStore::load();
        self.profiles = load_profiles();

        self.config_mtime = AppConfig::mtime();
        self.styles_mtime = ComponentStyles::mtime();
        self.shortcuts_mtime = UserData::shortcuts_mtime();
        self.rules_mtime = SiteRulesStore::mtime();
        self.adblock_mtime = AdblockConfig::mtime();
        self.keybinds_mtime = KeybindsConfig::mtime();
        self.permissions_mtime = SitePermissionsStore::mtime();
        self.themes_stamp = themes_stamp();
        self.last_reload_check = Instant::now();
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
            workspaces: crate::tabs::TabManager::list_workspaces(),
            chrome_extensions: self.chrome_extensions.clone(),
            chrome_ext_engine_active: self.boot_chrome_ext_enabled,
            site_permissions: self.site_permissions.origins.clone(),
            profiles: self.profiles.clone(),
            active_profile_id: self.active_profile_id.clone(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Serialize the lightweight external-safe HUD payload directly from `&self`
    /// without cloning `themes`, `adblock`, or calling filesystem `canonicalize` paths.
    pub fn hud_json_with_tabs(
        &self,
        tabs: Option<&crate::tabs::TabState>,
    ) -> Result<String, serde_json::Error> {
        let active_theme = self
            .resolve_active_theme_ref()
            .unwrap_or_else(|| Self::default_theme_colors());
        let enabled_rules: Vec<&SiteRule> = self.site_rules.iter().filter(|r| r.enabled).collect();
        let history_slice = &self.userdata.history[..self.userdata.history.len().min(25)];
        let chrome_extensions: Vec<HudChromeExtension> = self
            .chrome_extensions
            .iter()
            .map(HudChromeExtension::from)
            .collect();
        let slim = crate::config::HudBootPayload {
            config: &self.config,
            styles: &self.styles,
            active_theme,
            keybinds: &self.keybinds,
            enabled_rules,
            bookmarks: &self.userdata.bookmarks,
            history: history_slice,
            tabs,
            mode_themes: crate::config::collect_mode_themes(&self.config, &self.themes),
            downloads: &self.recent_downloads,
            chrome_extensions,
            chrome_ext_engine_active: self.boot_chrome_ext_enabled,
            site_permissions: &self.site_permissions.origins,
            profiles: &self.profiles,
            active_profile_id: &self.active_profile_id,
        };
        serde_json::to_string(&slim)
    }

    pub fn hud_json(&self) -> Result<String, serde_json::Error> {
        self.hud_json_with_tabs(None)
    }

    /// Serialize minimal privacy-preserving boot JSON for Windows Dual-WebView content tabs
    /// (omits bookmarks, browsing history, downloads, and Chrome extension inventory).
    pub fn content_json_with_tabs(
        &self,
        tabs: Option<&crate::tabs::TabState>,
    ) -> Result<String, serde_json::Error> {
        let active_theme = self
            .resolve_active_theme_ref()
            .unwrap_or_else(|| Self::default_theme_colors());
        let enabled_rules: Vec<&SiteRule> = self.site_rules.iter().filter(|r| r.enabled).collect();
        let slim = crate::config::ContentBootPayload {
            config: &self.config,
            active_theme,
            keybinds: &self.keybinds,
            enabled_rules,
            adblock_whitelist: &self.adblock.whitelisted_domains,
            tabs,
        };
        serde_json::to_string(&slim)
    }

    /// Serialize only the fields consumed by `fynexo://newtab` (`config`, `styles`,
    /// `active_theme`, `bookmarks`, `history[..40]`, `blocked_ads`) safely for inline `<script>`.
    pub fn newtab_boot_json(&self) -> String {
        let active_theme = self
            .resolve_active_theme_ref()
            .unwrap_or_else(|| Self::default_theme_colors());
        let history_slice = &self.userdata.history[..self.userdata.history.len().min(40)];
        let slim = crate::config::NewTabBootPayload {
            config: &self.config,
            styles: &self.styles,
            active_theme,
            bookmarks: &self.userdata.bookmarks,
            history: history_slice,
            blocked_ads: adblock::blocked_count(),
        };
        crate::util::json_for_html_script(&slim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_if_changed_throttles_rapid_consecutive_calls() {
        crate::util::isolate_data_dir();
        let mut st = AppState::load();
        assert!(!st.reload_if_changed());
        st.last_reload_check = Instant::now() - Duration::from_secs(5);
        assert!(!st.reload_if_changed());
        assert!(st.last_reload_check.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn record_download_tracks_transitions_without_progress_toast_spam() {
        crate::util::isolate_data_dir();
        let mut st = AppState::load();
        assert_eq!(
            st.record_download(1, "a.zip".into(), "downloading".into(), 0, 1000, 0, None),
            Some("downloading".to_string())
        );
        assert_eq!(
            st.record_download(1, "a.zip".into(), "downloading".into(), 500, 1000, 250, None),
            None
        );
        assert_eq!(
            st.record_download(1, "a.zip".into(), "completed".into(), 1000, 1000, 0, Some("C:/a.zip".into())),
            Some("completed".to_string())
        );
        assert_eq!(st.recent_downloads.len(), 1);
        assert_eq!(st.recent_downloads[0].saved_path, "C:/a.zip");
    }
}
