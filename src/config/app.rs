use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::base::{
    synced_json_mtime, synced_json_path, synced_json_save, DESKTOP_CHROME_UA, MOBILE_CHROME_UA,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchEngineItem {
    pub id: String,
    pub name: String,
    /// Query template using `{query}` (or `%s`), e.g. `https://www.google.com/search?q={query}`
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BangItem {
    pub trigger: String,
    pub name: String,
    pub url: String,
}

pub fn default_search_engines() -> Vec<SearchEngineItem> {
    vec![
        SearchEngineItem {
            id: "google".to_string(),
            name: "Google".to_string(),
            url: "https://www.google.com/search?q={query}".to_string(),
        },
        SearchEngineItem {
            id: "duckduckgo".to_string(),
            name: "DuckDuckGo".to_string(),
            url: "https://duckduckgo.com/?q={query}".to_string(),
        },
        SearchEngineItem {
            id: "bing".to_string(),
            name: "Bing".to_string(),
            url: "https://www.bing.com/search?q={query}".to_string(),
        },
        SearchEngineItem {
            id: "baidu".to_string(),
            name: "Baidu".to_string(),
            url: "https://www.baidu.com/s?wd={query}".to_string(),
        },
    ]
}

pub fn default_bangs() -> Vec<BangItem> {
    vec![
        BangItem {
            trigger: "!gh".to_string(),
            name: "GitHub".to_string(),
            url: "https://github.com/search?q={query}".to_string(),
        },
        BangItem {
            trigger: "!yt".to_string(),
            name: "YouTube".to_string(),
            url: "https://www.youtube.com/results?search_query={query}".to_string(),
        },
        BangItem {
            trigger: "!b".to_string(),
            name: "Bilibili".to_string(),
            url: "https://search.bilibili.com/all?keyword={query}".to_string(),
        },
        BangItem {
            trigger: "!w".to_string(),
            name: "Wikipedia".to_string(),
            url: "https://en.wikipedia.org/w/index.php?search={query}".to_string(),
        },
        BangItem {
            trigger: "!rd".to_string(),
            name: "Reddit".to_string(),
            url: "https://www.reddit.com/search/?q={query}".to_string(),
        },
        BangItem {
            trigger: "!tr".to_string(),
            name: "Translate".to_string(),
            url: "https://translate.google.com/?sl=auto&tl=zh-CN&text={query}&op=translate"
                .to_string(),
        },
    ]
}

fn default_zoom_one() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}

fn default_ua_mode() -> String {
    "default".to_string()
}

fn default_tab_cache_mode() -> String {
    "balanced".to_string()
}

fn default_language() -> String {
    "zh".to_string()
}

fn default_active_mode() -> String {
    "auto".to_string()
}

fn default_window_mode() -> String {
    "multi".to_string()
}

fn default_perf_preset() -> String {
    "low".to_string()
}

fn default_sleep_minutes() -> u32 {
    15
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct BrowserModeProfile {
    pub id: String,
    pub name_zh: String,
    pub name_en: String,
    pub icon: String,
    pub auto_domains: Vec<String>,
    pub adblock: bool,
    pub force_dark: bool,
    pub ext_side_panel: bool,
    pub ext_reader_mode: bool,
    pub ext_yt_ad_skipper: bool,
    pub ext_gh_harmonizer: bool,
    pub ext_media_booster: bool,
    pub ext_userscript_engine: bool,
    pub ext_mouse_gestures: bool,
    #[serde(default = "default_true")]
    pub dev_drawer_access: bool,
    #[serde(default)]
    pub fps_hud: bool,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub layout: String,
}

impl Default for BrowserModeProfile {
    fn default() -> Self {
        Self {
            id: "custom".to_string(),
            name_zh: "自定义模式".to_string(),
            name_en: "Custom Mode".to_string(),
            icon: "✨".to_string(),
            auto_domains: Vec::new(),
            adblock: true,
            force_dark: false,
            ext_side_panel: false,
            ext_reader_mode: false,
            ext_yt_ad_skipper: false,
            ext_gh_harmonizer: false,
            ext_media_booster: false,
            ext_userscript_engine: true,
            ext_mouse_gestures: true,
            dev_drawer_access: true,
            fps_hud: false,
            theme: String::new(),
            layout: String::new(),
        }
    }
}

pub fn default_mode_profiles() -> Vec<BrowserModeProfile> {
    let s = |v: &str| v.to_string();
    vec![
        BrowserModeProfile {
            id: s("developer"),
            name_zh: s("开发者模式"),
            name_en: s("Developer Mode"),
            icon: s("🛠️"),
            auto_domains: vec![
                s("localhost"),
                s("127.0.0.1"),
                s("0.0.0.0"),
                s("*.local"),
                s("*.test"),
                s("github.com"),
                s("gitlab.com"),
                s("stackoverflow.com"),
                s("crates.io"),
                s("docs.rs"),
                s("npmjs.com"),
                s("developer.mozilla.org"),
            ],
            adblock: false,
            force_dark: false,
            ext_side_panel: true,
            ext_reader_mode: false,
            ext_yt_ad_skipper: false,
            ext_gh_harmonizer: true,
            ext_media_booster: false,
            ext_userscript_engine: true,
            ext_mouse_gestures: true,
            dev_drawer_access: true,
            fps_hud: false,
            theme: String::new(),
            layout: String::new(),
        },
        BrowserModeProfile {
            id: s("study"),
            name_zh: s("学习与阅读模式"),
            name_en: s("Study & Focus Mode"),
            icon: s("📚"),
            auto_domains: vec![
                s("*.wikipedia.org"),
                s("zhihu.com"),
                s("*.zhihu.com"),
                s("arxiv.org"),
                s("leetcode.com"),
                s("leetcode.cn"),
                s("scholar.google.com"),
                s("notion.so"),
                s("coursera.org"),
                s("edx.org"),
            ],
            adblock: true,
            force_dark: false,
            ext_side_panel: true,
            ext_reader_mode: true,
            ext_yt_ad_skipper: false,
            ext_gh_harmonizer: false,
            ext_media_booster: false,
            ext_userscript_engine: true,
            ext_mouse_gestures: true,
            dev_drawer_access: false,
            fps_hud: false,
            theme: String::new(),
            layout: String::new(),
        },
        BrowserModeProfile {
            id: s("media"),
            name_zh: s("影音娱乐模式"),
            name_en: s("Media & Cinema Mode"),
            icon: s("🎬"),
            auto_domains: vec![
                s("youtube.com"),
                s("*.youtube.com"),
                s("bilibili.com"),
                s("*.bilibili.com"),
                s("netflix.com"),
                s("twitch.tv"),
                s("spotify.com"),
                s("v.qq.com"),
                s("iqiyi.com"),
            ],
            adblock: true,
            force_dark: true,
            ext_side_panel: false,
            ext_reader_mode: false,
            ext_yt_ad_skipper: true,
            ext_gh_harmonizer: false,
            ext_media_booster: true,
            ext_userscript_engine: true,
            ext_mouse_gestures: true,
            dev_drawer_access: false,
            fps_hud: false,
            theme: String::new(),
            layout: String::new(),
        },
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct SiteOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom: Option<f64>,
    #[serde(default)]
    pub dark_mode: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabCacheLimits {
    pub max_cached_normal: usize,
    pub hot_normal_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub theme: String,
    pub enforce_perf: bool,
    pub adblock: bool,
    pub restore_session: bool,
    pub force_dark: bool,
    pub incognito: bool,
    #[serde(default = "default_true")]
    pub https_first: bool,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_zoom_one")]
    pub default_zoom: f64,
    #[serde(default = "default_ua_mode")]
    pub user_agent_mode: String,
    #[serde(default)]
    pub custom_user_agent: String,
    #[serde(default = "default_tab_cache_mode")]
    pub tab_cache_mode: String,
    #[serde(default = "default_perf_preset")]
    pub perf_preset: String,
    #[serde(default = "default_true")]
    pub efficiency_mode: bool,
    #[serde(default = "default_true")]
    pub auto_trim_ram: bool,
    #[serde(default = "default_sleep_minutes")]
    pub tab_sleep_minutes: u32,
    #[serde(default = "default_true")]
    pub hw_accel: bool,
    #[serde(default)]
    pub reduce_animations: bool,
    #[serde(default)]
    pub ext_side_panel: bool,
    #[serde(default)]
    pub ext_reader_mode: bool,
    #[serde(default)]
    pub ext_yt_ad_skipper: bool,
    #[serde(default)]
    pub ext_gh_harmonizer: bool,
    #[serde(default)]
    pub ext_media_booster: bool,
    #[serde(default = "default_true")]
    pub ext_userscript_engine: bool,
    #[serde(default = "default_true")]
    pub ext_mouse_gestures: bool,
    #[serde(default = "default_active_mode")]
    pub active_mode: String,
    #[serde(default = "default_mode_profiles")]
    pub mode_profiles: Vec<BrowserModeProfile>,
    #[serde(default)]
    pub site_overrides: std::collections::BTreeMap<String, SiteOverride>,
    pub search_engine: String,
    pub homepage: String,
    #[serde(default = "default_window_mode")]
    pub window_mode: String,
    #[serde(default)]
    pub downloads_dir: String,
    #[serde(default = "default_search_engines")]
    pub search_engines: Vec<SearchEngineItem>,
    #[serde(default = "default_bangs")]
    pub bangs: Vec<BangItem>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            enforce_perf: false,
            adblock: true,
            restore_session: true,
            force_dark: false,
            incognito: false,
            https_first: true,
            language: default_language(),
            default_zoom: 1.0,
            user_agent_mode: "default".to_string(),
            custom_user_agent: String::new(),
            tab_cache_mode: "balanced".to_string(),
            perf_preset: default_perf_preset(),
            efficiency_mode: true,
            auto_trim_ram: true,
            tab_sleep_minutes: default_sleep_minutes(),
            hw_accel: true,
            reduce_animations: false,
            ext_side_panel: false,
            ext_reader_mode: false,
            ext_yt_ad_skipper: false,
            ext_gh_harmonizer: false,
            ext_media_booster: false,
            ext_userscript_engine: true,
            ext_mouse_gestures: true,
            active_mode: default_active_mode(),
            mode_profiles: default_mode_profiles(),
            site_overrides: std::collections::BTreeMap::new(),
            search_engine: "google".to_string(),
            homepage: "fynexo://newtab".to_string(),
            window_mode: default_window_mode(),
            downloads_dir: String::new(),
            search_engines: default_search_engines(),
            bangs: default_bangs(),
        }
    }
}

impl AppConfig {
    pub fn is_multi_window(&self) -> bool {
        !self.window_mode.trim().eq_ignore_ascii_case("single")
    }

    pub fn normalize_tab_cache_mode(raw: &str) -> &'static str {
        match raw.trim().to_ascii_lowercase().as_str() {
            "minimal" | "ultra" => "minimal",
            "eco" => "eco",
            "medium" => "medium",
            "smooth" => "smooth",
            _ => "balanced",
        }
    }

    pub fn normalize_perf_preset(raw: &str) -> &'static str {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ultra_low" | "ultra" | "minimal" => "ultra_low",
            "low" | "eco" => "low",
            "medium" | "normal" => "medium",
            "high" | "smooth" | "max" => "high",
            "custom" => "custom",
            _ => "low",
        }
    }

    pub fn sanitize_perf_fields(&mut self) {
        self.tab_cache_mode = Self::normalize_tab_cache_mode(&self.tab_cache_mode).to_string();
        self.perf_preset = Self::normalize_perf_preset(&self.perf_preset).to_string();
        if self.tab_sleep_minutes > 240 {
            self.tab_sleep_minutes = 240;
        }
    }

    pub fn apply_perf_preset(&mut self, raw_preset: &str) {
        match Self::normalize_perf_preset(raw_preset) {
            "ultra_low" => {
                self.perf_preset = "ultra_low".to_string();
                self.tab_cache_mode = "minimal".to_string();
                self.efficiency_mode = true;
                self.auto_trim_ram = true;
                self.tab_sleep_minutes = 5;
                self.enforce_perf = true;
                self.reduce_animations = true;
                self.ext_side_panel = false;
                self.ext_reader_mode = false;
                self.ext_yt_ad_skipper = false;
                self.ext_gh_harmonizer = false;
                self.ext_media_booster = false;
            }
            "low" => {
                self.perf_preset = "low".to_string();
                self.tab_cache_mode = "balanced".to_string();
                self.efficiency_mode = true;
                self.auto_trim_ram = true;
                self.tab_sleep_minutes = 15;
                self.enforce_perf = false;
                self.reduce_animations = false;
            }
            "medium" => {
                self.perf_preset = "medium".to_string();
                self.tab_cache_mode = "medium".to_string();
                self.efficiency_mode = true;
                self.auto_trim_ram = false;
                self.tab_sleep_minutes = 30;
                self.enforce_perf = false;
                self.reduce_animations = false;
            }
            "high" => {
                self.perf_preset = "high".to_string();
                self.tab_cache_mode = "smooth".to_string();
                self.efficiency_mode = false;
                self.auto_trim_ram = false;
                self.tab_sleep_minutes = 0;
                self.enforce_perf = false;
                self.reduce_animations = false;
            }
            _ => {
                self.perf_preset = "custom".to_string();
            }
        }
    }

    pub fn tab_cache_limits(&self) -> TabCacheLimits {
        let is_mobile = cfg!(any(target_os = "android", target_os = "ios"));
        match Self::normalize_tab_cache_mode(&self.tab_cache_mode) {
            "minimal" => TabCacheLimits {
                max_cached_normal: 1,
                hot_normal_count: 1,
            },
            "eco" => TabCacheLimits {
                max_cached_normal: 2,
                hot_normal_count: 1,
            },
            "medium" => TabCacheLimits {
                max_cached_normal: if is_mobile { 3 } else { 5 },
                hot_normal_count: 2,
            },
            "smooth" => TabCacheLimits {
                max_cached_normal: if is_mobile { 4 } else { 8 },
                hot_normal_count: 3,
            },
            _ => TabCacheLimits {
                max_cached_normal: if is_mobile { 2 } else { 3 },
                hot_normal_count: 1,
            },
        }
    }

    /// Returns an estimated `(min_mb, max_mb)` multi-process RAM footprint matching the UI calculator.
    pub fn estimate_ram_mb(&self) -> (u32, u32) {
        let limits = self.tab_cache_limits();
        let bg_slots = limits.max_cached_normal.saturating_sub(1) as f64;
        let base_min = if self.hw_accel { 105.0 } else { 90.0 };
        let base_max = if self.hw_accel { 145.0 } else { 125.0 };

        let per_tab_min = if self.efficiency_mode { 22.0 } else { 42.0 };
        let per_tab_max = if self.efficiency_mode { 36.0 } else { 68.0 };

        let mut ext_count = 0usize;
        for flag in [
            self.ext_side_panel,
            self.ext_reader_mode,
            self.ext_yt_ad_skipper,
            self.ext_gh_harmonizer,
            self.ext_media_booster,
        ] {
            if flag {
                ext_count += 1;
            }
        }
        let ext_overhead = (ext_count as f64) * 14.0;
        let ad_penalty = if self.adblock { 0.0 } else { 35.0 };
        let trim_bonus = if self.auto_trim_ram { 0.90 } else { 1.0 };
        let fx_bonus = if self.enforce_perf { 0.92 } else { 1.0 };

        let min_mb = ((base_min + bg_slots * per_tab_min + ext_overhead + ad_penalty)
            * trim_bonus
            * fx_bonus)
            .round() as u32;
        let max_mb = ((base_max + bg_slots * per_tab_max + ext_overhead * 1.6 + ad_penalty * 1.8)
            * trim_bonus
            * fx_bonus)
            .round() as u32;
        (min_mb, max_mb.max(min_mb + 20))
    }
    pub fn resolved_user_agent(&self) -> Option<String> {
        match self.user_agent_mode.as_str() {
            "desktop" => Some(DESKTOP_CHROME_UA.to_string()),
            "mobile" => Some(MOBILE_CHROME_UA.to_string()),
            "custom" => {
                let trimmed = self.custom_user_agent.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            _ => None,
        }
    }

    pub fn config_path() -> PathBuf {
        synced_json_path("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                    if config.search_engines.is_empty() {
                        config.search_engines = default_search_engines();
                    }
                    if config.mode_profiles.is_empty() {
                        config.mode_profiles = default_mode_profiles();
                    }
                    if !config.default_zoom.is_finite() || config.default_zoom < 0.3 {
                        config.default_zoom = 1.0;
                    } else {
                        config.default_zoom = config.default_zoom.clamp(0.5, 2.5);
                    }
                    config.sanitize_perf_fields();
                    return config;
                }
            }
        }
        let default_config = Self::default();
        default_config.save();
        default_config
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            synced_json_save("config.json", &json);
        }
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_profiles_are_sane() {
        let p = default_mode_profiles();
        let ids: Vec<&str> = p.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["developer", "study", "media"]);
        assert!(p[0].auto_domains.iter().any(|d| d == "localhost"));
        assert!(p.iter().all(|m| m.theme.is_empty() && m.layout.is_empty()));
    }

    #[test]
    fn mode_profile_roundtrip_keeps_theme_and_layout() {
        let m = BrowserModeProfile {
            theme: "nord".to_string(),
            layout: "vertical".to_string(),
            ..BrowserModeProfile::default()
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: BrowserModeProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn legacy_mode_profile_without_theme_layout_still_loads() {
        let back: BrowserModeProfile =
            serde_json::from_str(r#"{"id":"x","name_zh":"x","name_en":"x"}"#).unwrap();
        assert_eq!(back.id, "x");
        assert!(back.theme.is_empty() && back.layout.is_empty());
        assert!(back.dev_drawer_access);
    }

    #[test]
    fn app_config_mainstream_defaults_are_safe() {
        let mut cfg = AppConfig::default();
        assert!(!cfg.enforce_perf);
        assert!(cfg.https_first);
        assert_eq!(cfg.window_mode, "multi");
        assert_eq!(cfg.perf_preset, "low");
        assert!(cfg.efficiency_mode);
        assert!(cfg.auto_trim_ram);
        assert_eq!(cfg.tab_sleep_minutes, 15);
        assert!(cfg.hw_accel);
        assert!(cfg.is_multi_window());
        cfg.window_mode = "single".to_string();
        assert!(!cfg.is_multi_window());
    }
    #[test]
    fn tab_cache_mode_normalization_and_limits() {
        let mut cfg = AppConfig::default();
        assert_eq!(cfg.tab_cache_mode, "balanced");
        let bal = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "minimal".to_string();
        let min = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "ECO".to_string();
        let eco = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "medium".to_string();
        let med = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "smooth".to_string();
        let smooth = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "unknown".to_string();
        assert_eq!(cfg.tab_cache_limits(), bal);
        assert!(min.max_cached_normal <= eco.max_cached_normal);
        assert!(eco.max_cached_normal <= bal.max_cached_normal);
        assert!(bal.max_cached_normal <= med.max_cached_normal);
        assert!(med.max_cached_normal <= smooth.max_cached_normal);
    }

    #[test]
    fn perf_presets_scale_memory_cleanly() {
        let mut cfg = AppConfig::default();
        cfg.apply_perf_preset("ultra_low");
        let (u_min, u_max) = cfg.estimate_ram_mb();
        assert!(cfg.enforce_perf);
        assert_eq!(cfg.tab_cache_mode, "minimal");

        cfg.apply_perf_preset("low");
        let (l_min, l_max) = cfg.estimate_ram_mb();
        assert!(!cfg.enforce_perf);
        assert!(cfg.efficiency_mode);

        cfg.apply_perf_preset("medium");
        let (m_min, m_max) = cfg.estimate_ram_mb();

        cfg.apply_perf_preset("high");
        let (h_min, h_max) = cfg.estimate_ram_mb();
        assert!(!cfg.efficiency_mode);

        assert!(u_min < l_min && l_min < m_min && m_min < h_min);
        assert!(u_max < l_max && l_max < m_max && m_max < h_max);
    }
}
