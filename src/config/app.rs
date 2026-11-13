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

fn default_ua_mode() -> String {
    "default".to_string()
}

fn default_tab_cache_mode() -> String {
    "balanced".to_string()
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
    #[serde(default = "default_zoom_one")]
    pub default_zoom: f64,
    #[serde(default = "default_ua_mode")]
    pub user_agent_mode: String,
    #[serde(default)]
    pub custom_user_agent: String,
    #[serde(default = "default_tab_cache_mode")]
    pub tab_cache_mode: String,
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
            default_zoom: 1.0,
            user_agent_mode: "default".to_string(),
            custom_user_agent: String::new(),
            tab_cache_mode: "balanced".to_string(),
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
