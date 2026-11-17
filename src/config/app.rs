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
    #[serde(default = "default_search_engines")]
    pub search_engines: Vec<SearchEngineItem>,
    #[serde(default = "default_bangs")]
    pub bangs: Vec<BangItem>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            enforce_perf: true,
            adblock: true,
            restore_session: true,
            force_dark: false,
            incognito: false,
            language: default_language(),
            default_zoom: 1.0,
            user_agent_mode: "default".to_string(),
            custom_user_agent: String::new(),
            tab_cache_mode: "balanced".to_string(),
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
            search_engines: default_search_engines(),
            bangs: default_bangs(),
        }
    }
}

impl AppConfig {
    pub fn normalize_tab_cache_mode(raw: &str) -> &'static str {
        match raw.trim().to_ascii_lowercase().as_str() {
            "eco" => "eco",
            "smooth" => "smooth",
            _ => "balanced",
        }
    }

    pub fn tab_cache_limits(&self) -> TabCacheLimits {
        let is_mobile = cfg!(any(target_os = "android", target_os = "ios"));
        match Self::normalize_tab_cache_mode(&self.tab_cache_mode) {
            "eco" => TabCacheLimits {
                max_cached_normal: 2,
                hot_normal_count: 1,
            },
            "smooth" => TabCacheLimits {
                max_cached_normal: if is_mobile { 4 } else { 6 },
                hot_normal_count: 2,
            },
            _ => TabCacheLimits {
                max_cached_normal: if is_mobile { 2 } else { 3 },
                hot_normal_count: 1,
            },
        }
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
                    config.tab_cache_mode =
                        Self::normalize_tab_cache_mode(&config.tab_cache_mode).to_string();
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
    fn tab_cache_mode_normalization_and_limits() {
        let mut cfg = AppConfig::default();
        assert_eq!(cfg.tab_cache_mode, "balanced");
        let bal = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "ECO".to_string();
        let eco = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "smooth".to_string();
        let smooth = cfg.tab_cache_limits();
        cfg.tab_cache_mode = "unknown".to_string();
        assert_eq!(cfg.tab_cache_limits(), bal);
        assert!(eco.max_cached_normal <= bal.max_cached_normal);
        assert!(bal.max_cached_normal <= smooth.max_cached_normal);
    }
}
