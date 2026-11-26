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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub folder: String,
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
            folder: String::new(),
        },
        BookmarkItem {
            title: "GitHub".to_string(),
            url: "https://github.com".to_string(),
            folder: String::new(),
        },
        BookmarkItem {
            title: "YouTube".to_string(),
            url: "https://www.youtube.com".to_string(),
            folder: String::new(),
        },
        BookmarkItem {
            title: "Bilibili".to_string(),
            url: "https://www.bilibili.com".to_string(),
            folder: String::new(),
        },
        BookmarkItem {
            title: "Wikipedia".to_string(),
            url: "https://www.wikipedia.org".to_string(),
            folder: String::new(),
        },
        BookmarkItem {
            title: "DuckDuckGo".to_string(),
            url: "https://duckduckgo.com".to_string(),
            folder: String::new(),
        },
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UserData {
    pub bookmarks: Vec<BookmarkItem>,
    pub history: Vec<HistoryItem>,
    pub recent_searches: Vec<String>,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            bookmarks: default_shortcuts(),
            history: Vec::new(),
            recent_searches: Vec::new(),
        }
    }
}

fn decode_basic_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn sanitize_folder_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.len() <= 80 {
        trimmed.to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

fn is_generic_root_folder(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "bookmarks"
            | "bookmarks bar"
            | "bookmarks menu"
            | "other bookmarks"
            | "favorites"
            | "favorites bar"
            | "other favorites"
            | "书签栏"
            | "其他书签"
            | "收藏夹栏"
    )
}

/// Parses Netscape Bookmark HTML (`<!DOCTYPE NETSCAPE-Bookmark-file-1>` exported by Chrome/Edge/Firefox),
/// preserving nested `<H3>` / `<DL>` folder hierarchies.
fn parse_netscape_bookmarks_html(raw: &str) -> Vec<BookmarkItem> {
    let mut items = Vec::new();
    let lower = raw.to_ascii_lowercase();
    let mut folder_stack: Vec<String> = Vec::new();
    let mut pending_folder: Option<String> = None;
    let mut pos = 0;

    while pos < lower.len() {
        let slice = &lower[pos..];
        let next_a = slice.find("<a ");
        let next_h3 = slice.find("<h3");
        let next_dl = slice.find("<dl");
        let next_dl_end = slice.find("</dl");

        let next_token = [
            next_a.map(|i| (i, 0u8)),
            next_h3.map(|i| (i, 1u8)),
            next_dl.map(|i| (i, 2u8)),
            next_dl_end.map(|i| (i, 3u8)),
        ]
        .into_iter()
        .flatten()
        .min_by_key(|(i, _)| *i);

        let Some((rel, kind)) = next_token else {
            break;
        };
        let tag_start = pos + rel;

        match kind {
            1 => {
                // <H3 ...>Folder Name</H3>
                let Some(gt_rel) = raw[tag_start..].find('>') else {
                    break;
                };
                let after_tag = tag_start + gt_rel + 1;
                if let Some(close_rel) = lower[after_tag..].find("</h3>") {
                    let fname = sanitize_folder_name(&decode_basic_html_entities(
                        &raw[after_tag..after_tag + close_rel],
                    ));
                    if !fname.is_empty()
                        && !(folder_stack.iter().all(|s| s.is_empty())
                            && is_generic_root_folder(&fname))
                    {
                        pending_folder = Some(fname);
                    } else {
                        pending_folder = None;
                    }
                    pos = after_tag + close_rel + 5;
                } else {
                    pos = after_tag;
                }
            }
            2 => {
                // <DL ...>
                folder_stack.push(pending_folder.take().unwrap_or_default());
                pos = match raw[tag_start..].find('>') {
                    Some(gt_rel) => tag_start + gt_rel + 1,
                    None => break,
                };
            }
            3 => {
                // </DL>
                folder_stack.pop();
                pending_folder = None;
                pos = match raw[tag_start..].find('>') {
                    Some(gt_rel) => tag_start + gt_rel + 1,
                    None => break,
                };
            }
            _ => {
                // <A HREF="...">Title</A>
                let Some(gt_rel) = raw[tag_start..].find('>') else {
                    break;
                };
                let tag_end = tag_start + gt_rel;
                let attrs = &raw[tag_start..tag_end];
                let attrs_lower = &lower[tag_start..tag_end];
                let mut href_val = None;
                if let Some(h_rel) = attrs_lower.find("href=") {
                    let after = &attrs[h_rel + 5..];
                    let trimmed = after.trim_start();
                    if let Some(q) = trimmed.chars().next().filter(|c| *c == '"' || *c == '\'') {
                        if let Some(end_q) = trimmed[1..].find(q) {
                            href_val = Some(decode_basic_html_entities(&trimmed[1..1 + end_q]));
                        }
                    } else {
                        let end_ws = trimmed
                            .find(|c: char| c.is_ascii_whitespace())
                            .unwrap_or(trimmed.len());
                        href_val = Some(decode_basic_html_entities(&trimmed[..end_ws]));
                    }
                }

                let after_tag = tag_end + 1;
                let close_rel = lower[after_tag..].find("</a>");
                let title_raw = match close_rel {
                    Some(c_rel) => &raw[after_tag..after_tag + c_rel],
                    None => "",
                };
                let title = decode_basic_html_entities(title_raw).trim().to_string();

                let folder = folder_stack
                    .iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" / ");

                if let Some(u) = href_val {
                    let u_trim = u.trim();
                    if u_trim.starts_with("http://")
                        || u_trim.starts_with("https://")
                        || u_trim.starts_with("fynexo://")
                    {
                        items.push(BookmarkItem {
                            title,
                            url: u_trim.to_string(),
                            folder: sanitize_folder_name(&folder),
                        });
                    }
                }
                pos = match close_rel {
                    Some(c_rel) => after_tag + c_rel + 4,
                    None => after_tag,
                };
            }
        }
    }
    items
}

/// Parses JSON (`[...]`, `{ "shortcuts": [...] }`, `{ "bookmarks": [...] }`) or Chrome/Edge/Firefox `bookmarks.html`.
pub fn parse_shortcuts_flexible(raw: &str) -> Result<Vec<BookmarkItem>, String> {
    let trimmed_raw = raw.trim_start();
    let lower_head = trimmed_raw[..trimmed_raw.len().min(512)].to_ascii_lowercase();
    let items: Vec<BookmarkItem> = if trimmed_raw.starts_with('<')
        || lower_head.contains("netscape-bookmark-file")
    {
        let parsed = parse_netscape_bookmarks_html(raw);
        if parsed.is_empty() {
            return Err("No valid http/https bookmarks found in HTML file".to_string());
        }
        parsed
    } else {
        let val: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| format!("Invalid shortcuts JSON: {}", e))?;
        let arr_val = if val.is_array() {
            val
        } else if let Some(s) = val.get("shortcuts") {
            s.clone()
        } else if let Some(b) = val.get("bookmarks") {
            b.clone()
        } else {
            return Err(
                "Shortcuts JSON must be an array or an object with \"shortcuts\"".to_string(),
            );
        };
        serde_json::from_value(arr_val).map_err(|e| format!("Invalid shortcut item: {}", e))?
    };

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
                folder: sanitize_folder_name(&item.folder),
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

    pub fn searches_path() -> PathBuf {
        data_dir().join("fynexo_searches.json")
    }

    pub fn load() -> Self {
        let legacy_path = data_dir().join("fynexo_userdata.json");
        let sc_path = Self::shortcuts_path();
        let hist_path = Self::history_path();
        let search_path = Self::searches_path();

        // Seamless migration from legacy combined fynexo_userdata.json if present
        if !sc_path.exists() && legacy_path.exists() {
            if let Ok(raw) = fs::read_to_string(&legacy_path) {
                if let Ok(legacy) = serde_json::from_str::<UserData>(&raw) {
                    legacy.save_shortcuts();
                    legacy.save_history();
                    legacy.save_searches();
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

        let recent_searches = if search_path.exists() {
            fs::read_to_string(&search_path)
                .ok()
                .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            bookmarks,
            history,
            recent_searches,
        }
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
            crate::util::schedule_atomic_write(
                Self::history_path(),
                json.into_bytes(),
                std::time::Duration::ZERO,
            );
            crate::util::flush_pending_writes();
        }
    }

    pub fn save_history_debounced(&self) {
        if let Ok(json) = serde_json::to_string(&self.history) {
            crate::util::schedule_atomic_write(
                Self::history_path(),
                json.into_bytes(),
                std::time::Duration::from_millis(350),
            );
        }
    }

    pub fn save_searches(&self) {
        if let Ok(json) = serde_json::to_string(&self.recent_searches) {
            crate::util::schedule_atomic_write(
                Self::searches_path(),
                json.into_bytes(),
                std::time::Duration::ZERO,
            );
            crate::util::flush_pending_writes();
        }
    }

    pub fn save_searches_debounced(&self) {
        if let Ok(json) = serde_json::to_string(&self.recent_searches) {
            crate::util::schedule_atomic_write(
                Self::searches_path(),
                json.into_bytes(),
                std::time::Duration::from_millis(350),
            );
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
        self.toggle_bookmark_in_folder(title, url, "")
    }

    pub fn toggle_bookmark_in_folder(&mut self, title: &str, url: &str, folder: &str) -> bool {
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
                    folder: sanitize_folder_name(folder),
                },
            );
            self.save_shortcuts();
            true
        }
    }

    pub fn add_bookmark(&mut self, title: &str, url: &str) -> bool {
        self.add_bookmark_in_folder(title, url, "")
    }

    pub fn add_bookmark_in_folder(&mut self, title: &str, url: &str, folder: &str) -> bool {
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
                folder: sanitize_folder_name(folder),
            },
        );
        self.save_shortcuts();
        true
    }

    pub fn set_bookmark_folder(&mut self, url: &str, folder: &str) -> bool {
        let key = normalize_url_key(url);
        let clean = sanitize_folder_name(folder);
        if let Some(bm) = self
            .bookmarks
            .iter_mut()
            .find(|b| normalize_url_key(&b.url).eq_ignore_ascii_case(key))
        {
            if bm.folder != clean {
                bm.folder = clean;
                self.save_shortcuts();
                return true;
            }
        }
        false
    }

    pub fn edit_bookmark(
        &mut self,
        orig_url: &str,
        title: &str,
        new_url: &str,
        folder: &str,
    ) -> bool {
        let trimmed_url = new_url.trim();
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
        let orig_key = normalize_url_key(orig_url);
        let Some(pos) = self
            .bookmarks
            .iter()
            .position(|b| normalize_url_key(&b.url).eq_ignore_ascii_case(orig_key))
        else {
            return false;
        };
        let clean_title = if title.trim().is_empty() {
            full_url.clone()
        } else {
            title.trim().to_string()
        };
        let clean_folder = sanitize_folder_name(folder);
        let bm = &mut self.bookmarks[pos];
        bm.title = clean_title;
        bm.url = full_url;
        bm.folder = clean_folder;
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

        // Deduplicate noisy SPA title mutations on the active page:
        //  - Exact same URL & title within 30s -> skip disk write and visit inflation.
        //  - Same URL within 30s with a new title -> refresh title without incrementing visits.
        if let Some(head) = self.history.first_mut() {
            if normalize_url_key(&head.url).eq_ignore_ascii_case(key)
                && now.saturating_sub(head.visited_at) < 30
            {
                if head.title == clean_title {
                    return false;
                }
                head.title = clean_title.to_string();
                head.url = trimmed_url.to_string();
                self.save_history_debounced();
                return true;
            }
        }

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
        self.save_history_debounced();
        true
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.recent_searches.clear();
        self.save_history();
        self.save_searches();
    }

    pub fn record_search(&mut self, raw_query: &str) -> bool {
        let cleaned: String = raw_query.chars().filter(|c| !c.is_control()).collect();
        let trimmed = cleaned.trim();
        if trimmed.is_empty()
            || trimmed.chars().count() > 160
            || trimmed.starts_with('>')
            || trimmed.starts_with('!')
        {
            return false;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("fynexo://")
            || lower.starts_with("about:")
        {
            return false;
        }
        if let Some(pos) = self
            .recent_searches
            .iter()
            .position(|q| q.eq_ignore_ascii_case(trimmed))
        {
            if pos == 0 && self.recent_searches[0] == trimmed {
                return false;
            }
            self.recent_searches.remove(pos);
        }
        self.recent_searches.insert(0, trimmed.to_string());
        if self.recent_searches.len() > 24 {
            self.recent_searches.truncate(24);
        }
        self.save_searches_debounced();
        true
    }

    pub fn remove_search(&mut self, query: &str) -> bool {
        let trimmed = query.trim();
        let prev = self.recent_searches.len();
        self.recent_searches
            .retain(|q| !q.eq_ignore_ascii_case(trimmed));
        if self.recent_searches.len() != prev {
            self.save_searches_debounced();
            true
        } else {
            false
        }
    }

    pub fn remove_history_item(&mut self, url: &str) -> bool {
        let key = normalize_url_key(url);
        let prev = self.history.len();
        self.history
            .retain(|h| !normalize_url_key(&h.url).eq_ignore_ascii_case(key));
        if self.history.len() != prev {
            self.save_history_debounced();
            true
        } else {
            false
        }
    }

    pub fn remove_history_items(&mut self, urls: &[String]) -> usize {
        if urls.is_empty() {
            return 0;
        }
        let keys: Vec<String> = urls
            .iter()
            .map(|u| normalize_url_key(u).to_ascii_lowercase())
            .collect();
        let prev = self.history.len();
        self.history.retain(|h| {
            let k = normalize_url_key(&h.url).to_ascii_lowercase();
            !keys.iter().any(|target| target == &k)
        });
        let removed = prev.saturating_sub(self.history.len());
        if removed > 0 {
            self.save_history();
        }
        removed
    }

    pub fn remove_searches(&mut self, queries: &[String]) -> usize {
        if queries.is_empty() {
            return 0;
        }
        let prev = self.recent_searches.len();
        self.recent_searches
            .retain(|q| !queries.iter().any(|t| q.eq_ignore_ascii_case(t.trim())));
        let removed = prev.saturating_sub(self.recent_searches.len());
        if removed > 0 {
            self.save_searches();
        }
        removed
    }

    pub fn clear_searches(&mut self) -> bool {
        if self.recent_searches.is_empty() {
            return false;
        }
        self.recent_searches.clear();
        self.save_searches();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_visit_deduplicates_rapid_spa_title_updates() {
        crate::util::isolate_data_dir();
        let mut ud = UserData {
            bookmarks: Vec::new(),
            history: Vec::new(),
            recent_searches: Vec::new(),
        };
        assert!(ud.record_visit("Video A", "https://example.com/watch?v=1"));
        assert_eq!(ud.history.len(), 1);
        assert_eq!(ud.history[0].visits, 1);

        // Identical rapid callback: skipped without disk write or visit inflation.
        assert!(!ud.record_visit("Video A", "https://example.com/watch?v=1"));
        assert_eq!(ud.history[0].visits, 1);

        // Dynamic title update on the same URL within 30s: updates title, keeps visits = 1.
        assert!(ud.record_visit("(1) Video A", "https://example.com/watch?v=1"));
        assert_eq!(ud.history[0].title, "(1) Video A");
        assert_eq!(ud.history[0].visits, 1);

        // Internal pages are never recorded.
        assert!(!ud.record_visit("Settings", "fynexo://settings"));
        assert_eq!(ud.history.len(), 1);

        // Single-item deletion respects URL normalization.
        assert!(ud.remove_history_item("https://example.com/watch?v=1"));
        assert!(ud.history.is_empty());
        assert!(!ud.remove_history_item("https://example.com/watch?v=1"));

        // Recent search deduplication, URL filtering, removal, and clearing.
        assert!(ud.record_search("azur promilia taiwan"));
        assert!(ud.record_search("rust async"));
        assert!(!ud.record_search("https://example.com"));
        assert!(!ud.record_search(">settings"));
        assert!(ud.record_search("Azur Promilia Taiwan"));
        assert_eq!(ud.recent_searches, vec!["Azur Promilia Taiwan", "rust async"]);
        assert!(ud.remove_search("azur promilia taiwan"));
        assert_eq!(ud.recent_searches, vec!["rust async"]);
        ud.clear_history();
        assert!(ud.recent_searches.is_empty());
    }

    #[test]
    fn parse_shortcuts_flexible_supports_chrome_bookmarks_html() {
        let html = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<DL><p>
    <DT><H3>Bookmarks bar</H3>
    <DL><p>
        <DT><A HREF="https://github.com/rust-lang/rust?a=1&amp;b=2" ADD_DATE="1700000000">Rust &amp; Friends</A>
        <DT><H3>Dev</H3>
        <DL><p>
            <DT><A HREF="https://docs.rs/">docs.rs</A>
            <DT><H3>Cloud</H3>
            <DL><p>
                <DT><A HREF="https://dash.cloudflare.com/">Cloudflare</A>
            </DL><p>
        </DL><p>
        <DT><A HREF="javascript:alert(1)">Bad XSS</A>
    </DL><p>
</DL><p>"#;
        let parsed = parse_shortcuts_flexible(html).expect("parse html bookmarks");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].title, "Rust & Friends");
        assert_eq!(parsed[0].url, "https://github.com/rust-lang/rust?a=1&b=2");
        assert_eq!(parsed[0].folder, "");
        assert_eq!(parsed[1].url, "https://docs.rs/");
        assert_eq!(parsed[1].folder, "Dev");
        assert_eq!(parsed[2].url, "https://dash.cloudflare.com/");
        assert_eq!(parsed[2].folder, "Dev / Cloud");
    }
}
