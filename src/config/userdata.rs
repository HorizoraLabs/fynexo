use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::base::{
    data_dir, display_path_preferring_local, normalize_url_key, now_unix_secs, synced_json_mtime,
    synced_json_path, synced_json_save,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryItem {
    pub title: String,
    pub url: String,
    pub visited_at: u64,
    pub visits: u32,
}

pub fn default_shortcuts() -> Vec<BookmarkItem> {
    vec![
        BookmarkItem {
            title: "Google".to_string(),
            url: "https://www.google.com".to_string(),
        },
        BookmarkItem {
            title: "GitHub".to_string(),
            url: "https://github.com".to_string(),
        },
        BookmarkItem {
            title: "YouTube".to_string(),
            url: "https://www.youtube.com".to_string(),
        },
        BookmarkItem {
            title: "Bilibili".to_string(),
            url: "https://www.bilibili.com".to_string(),
        },
        BookmarkItem {
            title: "Wikipedia".to_string(),
            url: "https://www.wikipedia.org".to_string(),
        },
        BookmarkItem {
            title: "DuckDuckGo".to_string(),
            url: "https://duckduckgo.com".to_string(),
        },
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UserData {
    pub bookmarks: Vec<BookmarkItem>,
    pub history: Vec<HistoryItem>,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            bookmarks: default_shortcuts(),
            history: Vec::new(),
        }
    }
}

/// Parses either `[ { "title": "...", "url": "..." } ]` OR `{ "shortcuts": [...] }` OR `{ "bookmarks": [...] }`.
pub fn parse_shortcuts_flexible(raw: &str) -> Result<Vec<BookmarkItem>, String> {
    let val: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("Invalid shortcuts JSON: {}", e))?;
    let arr_val = if val.is_array() {
        val
    } else if let Some(s) = val.get("shortcuts") {
        s.clone()
    } else if let Some(b) = val.get("bookmarks") {
        b.clone()
    } else {
        return Err("Shortcuts JSON must be an array or an object with \"shortcuts\"".to_string());
    };

    let items: Vec<BookmarkItem> =
        serde_json::from_value(arr_val).map_err(|e| format!("Invalid shortcut item: {}", e))?;

    let mut cleaned = Vec::with_capacity(items.len());
    for item in items {
        let trimmed_url = item.url.trim();
        if trimmed_url.is_empty() {
            continue;
        }
        let full_url = if !trimmed_url.starts_with("http://")
            && !trimmed_url.starts_with("https://")
            && !trimmed_url.starts_with("fynexo://")
        {
            format!("https://{}", trimmed_url)
        } else {
            trimmed_url.to_string()
        };
        let title = if item.title.trim().is_empty() {
            full_url.clone()
        } else {
            item.title.trim().to_string()
        };
        if !cleaned
            .iter()
            .any(|b: &BookmarkItem| normalize_url_key(&b.url).eq_ignore_ascii_case(normalize_url_key(&full_url)))
        {
            cleaned.push(BookmarkItem {
                title,
                url: full_url,
            });
        }
    }
    Ok(cleaned)
}

impl UserData {
    pub fn shortcuts_path() -> PathBuf {
        synced_json_path("shortcuts.json")
    }

    pub fn shortcuts_path_display() -> String {
        display_path_preferring_local("shortcuts.json")
    }

    pub fn shortcuts_mtime() -> Option<SystemTime> {
        synced_json_mtime("shortcuts.json")
    }

    pub fn history_path() -> PathBuf {
        data_dir().join("fynexo_history.json")
    }

    pub fn load() -> Self {
        let legacy_path = data_dir().join("fynexo_userdata.json");
        let sc_path = Self::shortcuts_path();
        let hist_path = Self::history_path();

        // Seamless migration from legacy combined fynexo_userdata.json if present
        if !sc_path.exists() && legacy_path.exists() {
            if let Ok(raw) = fs::read_to_string(&legacy_path) {
                if let Ok(legacy) = serde_json::from_str::<UserData>(&raw) {
                    legacy.save_shortcuts();
                    legacy.save_history();
                    return legacy;
                }
            }
        }

        let bookmarks = if sc_path.exists() {
            fs::read_to_string(&sc_path)
                .ok()
                .and_then(|raw| parse_shortcuts_flexible(&raw).ok())
                .unwrap_or_else(default_shortcuts)
        } else {
            let defs = default_shortcuts();
            if let Ok(json) = serde_json::to_string_pretty(&defs) {
                synced_json_save("shortcuts.json", &json);
            }
            defs
        };

        let history = if hist_path.exists() {
            fs::read_to_string(&hist_path)
                .ok()
                .and_then(|raw| serde_json::from_str::<Vec<HistoryItem>>(&raw).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { bookmarks, history }
    }

    pub fn reload_shortcuts(&mut self) {
        let sc_path = Self::shortcuts_path();
        if let Ok(raw) = fs::read_to_string(&sc_path) {
            if let Ok(items) = parse_shortcuts_flexible(&raw) {
                self.bookmarks = items;
            }
        }
    }

    pub fn save_shortcuts(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.bookmarks) {
            synced_json_save("shortcuts.json", &json);
        }
    }

    pub fn save_history(&self) {
        if let Ok(json) = serde_json::to_string(&self.history) {
            let _ = fs::write(Self::history_path(), json);
        }
    }

    pub fn shortcuts_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.bookmarks).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn import_shortcuts(&mut self, raw_json: &str, merge: bool) -> Result<usize, String> {
        let incoming = parse_shortcuts_flexible(raw_json)?;
        let count = incoming.len();
        if !merge {
            self.bookmarks = incoming;
        } else {
            for item in incoming {
                if !self.is_bookmarked(&item.url) {
                    self.bookmarks.push(item);
                }
            }
        }
        self.save_shortcuts();
        Ok(count)
    }

    pub fn reset_shortcuts(&mut self) {
        self.bookmarks = default_shortcuts();
        self.save_shortcuts();
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        let key = normalize_url_key(url);
        self.bookmarks
            .iter()
            .any(|b| normalize_url_key(&b.url).eq_ignore_ascii_case(key))
    }

    pub fn toggle_bookmark(&mut self, title: &str, url: &str) -> bool {
        let trimmed_url = url.trim();
        if trimmed_url.is_empty()
            || trimmed_url.starts_with("fynexo://")
            || trimmed_url.starts_with("http://fynexo.")
            || trimmed_url.starts_with("about:")
        {
            return false;
        }
        let key = normalize_url_key(trimmed_url);
        if let Some(pos) = self
            .bookmarks
            .iter()
            .position(|b| normalize_url_key(&b.url).eq_ignore_ascii_case(key))
        {
            self.bookmarks.remove(pos);
            self.save_shortcuts();
            false
        } else {
            let clean_title = if title.trim().is_empty() {
                trimmed_url
            } else {
                title.trim()
            };
            self.bookmarks.insert(
                0,
                BookmarkItem {
                    title: clean_title.to_string(),
                    url: trimmed_url.to_string(),
                },
            );
            self.save_shortcuts();
            true
        }
    }

    pub fn add_bookmark(&mut self, title: &str, url: &str) -> bool {
        let trimmed_url = url.trim();
        if trimmed_url.is_empty() {
            return false;
        }
        let full_url = if !trimmed_url.starts_with("http://")
            && !trimmed_url.starts_with("https://")
            && !trimmed_url.starts_with("fynexo://")
        {
            format!("https://{}", trimmed_url)
        } else {
            trimmed_url.to_string()
        };
        if self.is_bookmarked(&full_url) {
            return false;
        }
        let clean_title = if title.trim().is_empty() {
            full_url
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_start_matches("www.")
                .split('/')
                .next()
                .unwrap_or(&full_url)
                .to_string()
        } else {
            title.trim().to_string()
        };
        self.bookmarks.insert(
            0,
            BookmarkItem {
                title: clean_title,
                url: full_url,
            },
        );
        self.save_shortcuts();
        true
    }

    pub fn remove_bookmark(&mut self, url: &str) -> bool {
        let key = normalize_url_key(url);
        let before = self.bookmarks.len();
        self.bookmarks
            .retain(|b| !normalize_url_key(&b.url).eq_ignore_ascii_case(key));
        let changed = self.bookmarks.len() != before;
        if changed {
            self.save_shortcuts();
        }
        changed
    }

    pub fn record_visit(&mut self, title: &str, url: &str) -> bool {
        let trimmed_url = url.trim();
        if trimmed_url.is_empty()
            || trimmed_url.starts_with("fynexo://")
            || trimmed_url.starts_with("http://fynexo.")
            || trimmed_url.starts_with("https://fynexo.")
            || trimmed_url.starts_with("about:")
            || trimmed_url.starts_with("data:")
            || trimmed_url.starts_with("blob:")
        {
            return false;
        }

        let clean_title = title.trim();
        if clean_title.is_empty() || clean_title == "New tab" || clean_title == "Start page" {
            return false;
        }

        let key = normalize_url_key(trimmed_url);
        let now = now_unix_secs();

        if let Some(pos) = self
            .history
            .iter()
            .position(|h| normalize_url_key(&h.url).eq_ignore_ascii_case(key))
        {
            let mut item = self.history.remove(pos);
            item.title = clean_title.to_string();
            item.url = trimmed_url.to_string();
            item.visited_at = now;
            item.visits = item.visits.saturating_add(1);
            self.history.insert(0, item);
        } else {
            self.history.insert(
                0,
                HistoryItem {
                    title: clean_title.to_string(),
                    url: trimmed_url.to_string(),
                    visited_at: now,
                    visits: 1,
                },
            );
            if self.history.len() > 200 {
                self.history.truncate(200);
            }
        }
        self.save_history();
        true
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.save_history();
    }
}
