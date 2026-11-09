use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::base::{
    display_path_preferring_local, synced_json_mtime, synced_json_path, synced_json_save,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SiteRule {
    pub id: String,
    pub name: String,
    /// Domain glob (`"*"` for all external sites, `"zhihu.com"`, `"*.bilibili.com"`, etc.)
    pub domain: String,
    pub enabled: bool,
    pub css: String,
    pub js: String,
}

impl Default for SiteRule {
    fn default() -> Self {
        Self {
            id: "rule-1".to_string(),
            name: "Custom rule".to_string(),
            domain: "*".to_string(),
            enabled: true,
            css: String::new(),
            js: String::new(),
        }
    }
}

pub fn default_site_rules() -> Vec<SiteRule> {
    vec![
        SiteRule {
            id: "clean-app-banners".to_string(),
            name: "Hide Open-in-App floating banners".to_string(),
            domain: "*".to_string(),
            enabled: false,
            css: "[class*=\"open-in-app\"], [class*=\"download-app-banner\"], [id*=\"app-banner\"] { display: none !important; }".to_string(),
            js: String::new(),
        },
        SiteRule {
            id: "crisp-code-fonts".to_string(),
            name: "Crisp monospace code blocks".to_string(),
            domain: "github.com".to_string(),
            enabled: false,
            css: "pre, code, .blob-code-inner { font-family: \"JetBrains Mono\", Consolas, ui-monospace, monospace !important; }".to_string(),
            js: String::new(),
        },
    ]
}

pub struct SiteRulesStore;

impl SiteRulesStore {
    pub fn path() -> PathBuf {
        synced_json_path("fynexo_rules.json")
    }

    pub fn path_display() -> String {
        display_path_preferring_local("fynexo_rules.json")
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("fynexo_rules.json")
    }

    pub fn parse_flexible(raw: &str) -> Result<Vec<SiteRule>, String> {
        let val: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| format!("Invalid rules JSON: {}", e))?;
        let arr = if val.is_array() {
            val
        } else if let Some(r) = val.get("rules") {
            r.clone()
        } else {
            return Err("Rules JSON must be an array or an object with \"rules\"".to_string());
        };
        serde_json::from_value::<Vec<SiteRule>>(arr)
            .map_err(|e| format!("Invalid site rule entry: {}", e))
    }

    pub fn load() -> Vec<SiteRule> {
        let p = Self::path();
        if p.exists() {
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok(list) = Self::parse_flexible(&raw) {
                    return list;
                }
            }
        }
        let defs = default_site_rules();
        Self::save(&defs);
        defs
    }

    pub fn save(rules: &[SiteRule]) {
        if let Ok(json) = serde_json::to_string_pretty(rules) {
            synced_json_save("fynexo_rules.json", &json);
        }
    }
}
