use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::adblock::AdblockConfig;

pub const DESKTOP_CHROME_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36";
pub const MOBILE_CHROME_UA: &str = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36";

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
            search_engine: "google".to_string(),
            homepage: "fynexo://newtab".to_string(),
            search_engines: default_search_engines(),
            bangs: default_bangs(),
        }
    }
}

impl AppConfig {
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ThemeColors {
    pub bg: String,
    pub panel: String,
    pub surface: String,
    pub elevated: String,
    #[serde(alias = "borderColor")]
    pub border: String,
    #[serde(alias = "borderHover")]
    pub border_hover: String,
    pub fg: String,
    pub secondary: String,
    pub muted: String,
    pub accent: String,
    #[serde(alias = "accentBg")]
    pub accent_bg: String,
    pub hover: String,
    #[serde(alias = "tabActive")]
    pub tab_active: String,
    #[serde(alias = "tabInactive")]
    pub tab_inactive: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            bg: "#19191c".to_string(),
            panel: "#29292e".to_string(),
            surface: "#212126".to_string(),
            elevated: "#35353c".to_string(),
            border: "#37373f".to_string(),
            border_hover: "#4b4b56".to_string(),
            fg: "#f0f0f3".to_string(),
            secondary: "#c2c2cb".to_string(),
            muted: "#8a8a95".to_string(),
            accent: "#7a88f7".to_string(),
            accent_bg: "#262942".to_string(),
            hover: "#33333a".to_string(),
            tab_active: "#29292e".to_string(),
            tab_inactive: "transparent".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ThemePreset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(alias = "desc")]
    pub description: String,
    #[serde(default)]
    pub builtin: bool,
    pub colors: ThemeColors,
}

impl Default for ThemePreset {
    fn default() -> Self {
        Self {
            id: "dark".to_string(),
            name: "Obsidian".to_string(),
            author: "Fynexo".to_string(),
            description: "Layered warm-charcoal studio deck".to_string(),
            builtin: true,
            colors: ThemeColors::default(),
        }
    }
}

pub fn builtin_theme_presets() -> Vec<ThemePreset> {
    vec![
        ThemePreset {
            id: "dark".to_string(),
            name: "Obsidian".to_string(),
            author: "Fynexo".to_string(),
            description: "Layered warm-charcoal studio deck".to_string(),
            builtin: true,
            colors: ThemeColors::default(),
        },
        ThemePreset {
            id: "light".to_string(),
            name: "Alabaster".to_string(),
            author: "Fynexo".to_string(),
            description: "Soft stone tray & warm daylight paper".to_string(),
            builtin: true,
            colors: ThemeColors {
                bg: "#e6e7eb".to_string(),
                panel: "#f7f7f9".to_string(),
                surface: "#ededf1".to_string(),
                elevated: "#ffffff".to_string(),
                border: "#d5d6dc".to_string(),
                border_hover: "#b8bac4".to_string(),
                fg: "#18181b".to_string(),
                secondary: "#4a4a54".to_string(),
                muted: "#767682".to_string(),
                accent: "#4f46e5".to_string(),
                accent_bg: "#e5e7ff".to_string(),
                hover: "#eaeaf0".to_string(),
                tab_active: "#f7f7f9".to_string(),
                tab_inactive: "transparent".to_string(),
            },
        },
        ThemePreset {
            id: "oled".to_string(),
            name: "Carbon".to_string(),
            author: "Fynexo".to_string(),
            description: "Deep pitch tray & matte carbon deck".to_string(),
            builtin: true,
            colors: ThemeColors {
                bg: "#09090b".to_string(),
                panel: "#1c1c20".to_string(),
                surface: "#141417".to_string(),
                elevated: "#28282e".to_string(),
                border: "#2e2e36".to_string(),
                border_hover: "#42424d".to_string(),
                fg: "#ededf0".to_string(),
                secondary: "#b8b8c2".to_string(),
                muted: "#82828d".to_string(),
                accent: "#38bdf8".to_string(),
                accent_bg: "#142a38".to_string(),
                hover: "#25252b".to_string(),
                tab_active: "#1c1c20".to_string(),
                tab_inactive: "transparent".to_string(),
            },
        },
        ThemePreset {
            id: "nord".to_string(),
            name: "Fjord".to_string(),
            author: "Fynexo".to_string(),
            description: "Arctic slate tray & soft nordic deck".to_string(),
            builtin: true,
            colors: ThemeColors {
                bg: "#1b1f27".to_string(),
                panel: "#2b303c".to_string(),
                surface: "#232832".to_string(),
                elevated: "#373e4d".to_string(),
                border: "#3b4252".to_string(),
                border_hover: "#4c566a".to_string(),
                fg: "#eceff4".to_string(),
                secondary: "#c8d0e0".to_string(),
                muted: "#8c96aa".to_string(),
                accent: "#88c0d0".to_string(),
                accent_bg: "#263642".to_string(),
                hover: "#343b49".to_string(),
                tab_active: "#2b303c".to_string(),
                tab_inactive: "transparent".to_string(),
            },
        },
        ThemePreset {
            id: "matrix".to_string(),
            name: "Veridian".to_string(),
            author: "Fynexo".to_string(),
            description: "Forest darkroom tray & pine studio deck".to_string(),
            builtin: true,
            colors: ThemeColors {
                bg: "#111714".to_string(),
                panel: "#1e2923".to_string(),
                surface: "#17201b".to_string(),
                elevated: "#28372f".to_string(),
                border: "#2d3e35".to_string(),
                border_hover: "#3f574b".to_string(),
                fg: "#e6f5ec".to_string(),
                secondary: "#b0cebd".to_string(),
                muted: "#7a9c89".to_string(),
                accent: "#34d399".to_string(),
                accent_bg: "#18362a".to_string(),
                hover: "#25332b".to_string(),
                tab_active: "#1e2923".to_string(),
                tab_inactive: "transparent".to_string(),
            },
        },
    ]
}

fn sanitize_theme_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if (c == '-' || c == '_' || c == ' ') && !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "custom-theme".to_string()
    } else {
        trimmed
    }
}

impl ThemePreset {
    /// Parses either standard nested `{ id, name, colors: { ... } }` OR flat `{ name, bg, panel, accent, ... }`.
    pub fn parse_flexible(raw_json: &str, fallback_stem: Option<&str>) -> Result<Self, String> {
        let val: serde_json::Value =
            serde_json::from_str(raw_json).map_err(|e| format!("Invalid theme JSON: {}", e))?;
        let obj = val
            .as_object()
            .ok_or_else(|| "Theme JSON must be a JSON object".to_string())?;

        let colors: ThemeColors = if let Some(colors_val) = obj.get("colors") {
            serde_json::from_value(colors_val.clone())
                .map_err(|e| format!("Invalid colors field: {}", e))?
        } else {
            serde_json::from_value(val.clone())
                .map_err(|e| format!("Invalid flat theme colors: {}", e))?
        };

        let raw_name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let raw_id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        let id = if !raw_id.is_empty() {
            sanitize_theme_id(raw_id)
        } else if let Some(stem) = fallback_stem {
            sanitize_theme_id(stem)
        } else if !raw_name.is_empty() {
            sanitize_theme_id(raw_name)
        } else {
            "custom-theme".to_string()
        };

        let name = if !raw_name.is_empty() {
            raw_name.to_string()
        } else {
            let mut chars = id.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => "Custom Theme".to_string(),
            }
        };

        let author = obj
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("Custom")
            .trim()
            .to_string();

        let description = obj
            .get("description")
            .or_else(|| obj.get("desc"))
            .and_then(|v| v.as_str())
            .unwrap_or("Custom JSON palette")
            .trim()
            .to_string();

        let builtin = matches!(id.as_str(), "dark" | "light" | "oled" | "nord" | "matrix")
            && obj.get("builtin").and_then(|v| v.as_bool()).unwrap_or(false);

        Ok(ThemePreset {
            id,
            name,
            author,
            description,
            builtin,
            colors,
        })
    }
}

pub fn themes_dir() -> PathBuf {
    let dir = data_dir().join("themes");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn themes_dir_display() -> String {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local = PathBuf::from("themes");
        if local.is_dir() {
            return fs::canonicalize(&local)
                .unwrap_or(local)
                .to_string_lossy()
                .replace("\\\\?\\", "");
        }
    }
    let p = themes_dir();
    fs::canonicalize(&p)
        .unwrap_or(p)
        .to_string_lossy()
        .replace("\\\\?\\", "")
}

fn sync_local_themes_to_formal(formal_dir: &Path) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local_dir = PathBuf::from("themes");
        if local_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&local_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|e| e.to_str()) == Some("json") {
                        if let Some(fname) = p.file_name() {
                            let dest = formal_dir.join(fname);
                            let local_mt = fs::metadata(&p).and_then(|m| m.modified()).ok();
                            let dest_mt = fs::metadata(&dest).and_then(|m| m.modified()).ok();
                            if dest_mt.is_none() || local_mt > dest_mt {
                                let _ = fs::copy(&p, &dest);
                            }
                        }
                    }
                }
            }
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = formal_dir;
    }
}

pub fn load_all_themes() -> Vec<ThemePreset> {
    let dir = themes_dir();
    sync_local_themes_to_formal(&dir);

    let has_any_json = fs::read_dir(&dir)
        .ok()
        .map(|it| {
            it.flatten()
                .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        })
        .unwrap_or(false);

    if !has_any_json {
        for preset in builtin_theme_presets() {
            let _ = save_theme_preset(&preset);
        }
    } else {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            for preset in builtin_theme_presets() {
                let _ = save_theme_preset(&preset);
            }
        }
    }

    let mut list: Vec<ThemePreset> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let stem = p.file_stem().and_then(|s| s.to_str());
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok(mut preset) = ThemePreset::parse_flexible(&raw, stem) {
                    if let Some(s) = stem {
                        let clean_stem = sanitize_theme_id(s);
                        if preset.id.is_empty() {
                            preset.id = clean_stem;
                        }
                    }
                    if !list.iter().any(|existing| existing.id == preset.id) {
                        list.push(preset);
                    }
                }
            }
        }
    }

    if list.is_empty() {
        return builtin_theme_presets();
    }

    let builtin_order = ["dark", "light", "oled", "nord", "matrix"];
    list.sort_by(|a, b| {
        let pos_a = builtin_order.iter().position(|&k| k == a.id);
        let pos_b = builtin_order.iter().position(|&k| k == b.id);
        match (pos_a, pos_b) {
            (Some(ia), Some(ib)) => ia.cmp(&ib),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    list
}

pub fn save_theme_preset(preset: &ThemePreset) -> Result<String, String> {
    let clean_id = sanitize_theme_id(&preset.id);
    let mut normalized = preset.clone();
    normalized.id = clean_id.clone();
    let json = serde_json::to_string_pretty(&normalized).map_err(|e| e.to_string())?;
    let fname = format!("{}.json", clean_id);

    let formal_path = themes_dir().join(&fname);
    fs::write(&formal_path, &json).map_err(|e| e.to_string())?;

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local_dir = PathBuf::from("themes");
        if local_dir.is_dir() {
            let _ = fs::write(local_dir.join(&fname), &json);
        }
    }

    Ok(clean_id)
}

pub fn delete_theme_preset(theme_id: &str) -> bool {
    let clean_id = sanitize_theme_id(theme_id);
    if matches!(
        clean_id.as_str(),
        "dark" | "light" | "oled" | "nord" | "matrix"
    ) {
        return false;
    }
    let fname = format!("{}.json", clean_id);
    #[allow(unused_mut)]
    let mut removed = fs::remove_file(themes_dir().join(&fname)).is_ok();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local_file = PathBuf::from("themes").join(&fname);
        if local_file.exists() && fs::remove_file(local_file).is_ok() {
            removed = true;
        }
    }
    removed
}

/// Lightweight fingerprint `(file_count, latest_mtime)` across `themes/*.json` (zero polling threads).
pub fn themes_stamp() -> (usize, Option<SystemTime>) {
    let mut count = 0usize;
    let mut latest: Option<SystemTime> = None;

    let mut scan = |dir: &Path| {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("json") {
                    count += 1;
                    if let Ok(mt) = e.metadata().and_then(|m| m.modified()) {
                        latest = Some(latest.map_or(mt, |cur| cur.max(mt)));
                    }
                }
            }
        }
    };

    scan(&themes_dir());
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local = PathBuf::from("themes");
        if local.is_dir() {
            scan(&local);
        }
    }
    (count, latest)
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ColorStyles {
    pub use_custom_colors: bool,
    pub bg: String,
    pub panel: String,
    pub surface: String,
    pub elevated: String,
    pub border: String,
    pub border_hover: String,
    pub fg: String,
    pub secondary: String,
    pub muted: String,
    pub accent: String,
    pub accent_bg: String,
    pub hover: String,
    pub tab_active: String,
    pub tab_inactive: String,
}

impl Default for ColorStyles {
    fn default() -> Self {
        Self {
            use_custom_colors: false,
            bg: "#19191c".to_string(),
            panel: "#29292e".to_string(),
            surface: "#212126".to_string(),
            elevated: "#35353c".to_string(),
            border: "#37373f".to_string(),
            border_hover: "#4b4b56".to_string(),
            fg: "#f0f0f3".to_string(),
            secondary: "#c2c2cb".to_string(),
            muted: "#8a8a95".to_string(),
            accent: "#7a88f7".to_string(),
            accent_bg: "#262942".to_string(),
            hover: "#33333a".to_string(),
            tab_active: "#29292e".to_string(),
            tab_inactive: "transparent".to_string(),
        }
    }
}

fn default_topbar_layout() -> String {
    "deck".to_string()
}

fn default_bookmarks_bar() -> String {
    "always".to_string()
}

fn default_titlebar_height() -> String {
    "40px".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TopbarStyles {
    pub visible_on_desktop: bool,
    pub visible_on_mobile: bool,
    #[serde(default = "default_topbar_layout")]
    pub layout: String,
    #[serde(default = "default_bookmarks_bar")]
    pub bookmarks_bar: String,
    pub height: String,
    #[serde(default = "default_titlebar_height")]
    pub titlebar_height: String,
    pub padding_x: String,
    pub background: String,
    pub border_bottom: String,
    pub button_size: String,
    pub button_radius: String,
    pub custom_css: String,
}

impl Default for TopbarStyles {
    fn default() -> Self {
        Self {
            visible_on_desktop: true,
            visible_on_mobile: false,
            layout: "deck".to_string(),
            bookmarks_bar: "always".to_string(),
            height: "38px".to_string(),
            titlebar_height: "40px".to_string(),
            padding_x: "6px".to_string(),
            background: "var(--fnx-panel)".to_string(),
            border_bottom: "1px solid var(--fnx-border)".to_string(),
            button_size: "28px".to_string(),
            button_radius: "6px".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TabStyles {
    pub height: String,
    pub width: String,
    pub min_width: String,
    pub gap: String,
    pub border_radius: String,
    pub font_size: String,
    pub show_indicator_dot: bool,
    pub close_on_hover_only: bool,
    pub custom_css: String,
}

impl Default for TabStyles {
    fn default() -> Self {
        Self {
            height: "36px".to_string(),
            width: "184px".to_string(),
            min_width: "136px".to_string(),
            gap: "0px".to_string(),
            border_radius: "6px".to_string(),
            font_size: "12px".to_string(),
            show_indicator_dot: true,
            close_on_hover_only: false,
            custom_css: String::new(),
        }
    }
}

fn default_addr_mode() -> String {
    "inline".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AddressBarStyles {
    #[serde(default = "default_addr_mode")]
    pub mode: String,
    pub position: String,
    pub height: String,
    pub width: String,
    pub border_radius: String,
    pub font_size: String,
    pub background: String,
    pub border: String,
    pub custom_css: String,
}

impl Default for AddressBarStyles {
    fn default() -> Self {
        Self {
            mode: "inline".to_string(),
            position: "center".to_string(),
            height: "30px".to_string(),
            width: "340px".to_string(),
            border_radius: "9px".to_string(),
            font_size: "12.5px".to_string(),
            background: "var(--fnx-bg)".to_string(),
            border: "1px solid var(--fnx-border)".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SearchModalStyles {
    pub width: String,
    pub top_offset: String,
    pub border_radius: String,
    pub input_font_size: String,
    pub backdrop_color: String,
    pub background: String,
    pub border: String,
    pub custom_css: String,
}

impl Default for SearchModalStyles {
    fn default() -> Self {
        Self {
            width: "580px".to_string(),
            top_offset: "58px".to_string(),
            border_radius: "12px".to_string(),
            input_font_size: "14.5px".to_string(),
            backdrop_color: "rgba(0, 0, 0, 0.56)".to_string(),
            background: "var(--fnx-panel)".to_string(),
            border: "1px solid var(--fnx-border-hover)".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MobileBarStyles {
    pub visible: bool,
    pub position: String,
    pub height: String,
    pub padding_x: String,
    pub background: String,
    pub border: String,
    pub pill_height: String,
    pub pill_radius: String,
    pub button_size: String,
    pub auto_hide_on_keyboard: bool,
    pub custom_css: String,
}

impl Default for MobileBarStyles {
    fn default() -> Self {
        Self {
            visible: true,
            position: "bottom".to_string(),
            height: "54px".to_string(),
            padding_x: "8px".to_string(),
            background: "var(--fnx-panel)".to_string(),
            border: "1px solid var(--fnx-border)".to_string(),
            pill_height: "38px".to_string(),
            pill_radius: "11px".to_string(),
            button_size: "38px".to_string(),
            auto_hide_on_keyboard: true,
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct TabDrawerStyles {
    pub max_height: String,
    pub border_radius: String,
    pub card_radius: String,
    pub card_padding: String,
    pub custom_css: String,
}

impl Default for TabDrawerStyles {
    fn default() -> Self {
        Self {
            max_height: "80vh".to_string(),
            border_radius: "16px".to_string(),
            card_radius: "10px".to_string(),
            card_padding: "11px 13px".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct SettingsPageStyles {
    pub full_width: bool,
    pub sidebar_width: String,
    pub content_padding: String,
    pub card_radius: String,
    pub custom_css: String,
}

impl Default for SettingsPageStyles {
    fn default() -> Self {
        Self {
            full_width: true,
            sidebar_width: "244px".to_string(),
            content_padding: "32px 40px".to_string(),
            card_radius: "10px".to_string(),
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct NewtabStyles {
    pub title: String,
    pub subtitle: String,
    pub show_clock: bool,
    pub use_24h_clock: bool,
    pub show_shortcuts: bool,
    /// `0` = responsive auto-fit grid; `2..=6` = fixed column count on desktop
    pub shortcuts_columns: u8,
    pub show_history: bool,
    pub custom_css: String,
}

impl Default for NewtabStyles {
    fn default() -> Self {
        Self {
            title: "Fynexo".to_string(),
            subtitle: "Single-WebView studio browser · Zero-FX compositing".to_string(),
            show_clock: true,
            use_24h_clock: true,
            show_shortcuts: true,
            shortcuts_columns: 0,
            show_history: true,
            custom_css: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct ComponentStyles {
    pub colors: ColorStyles,
    pub topbar: TopbarStyles,
    pub tabs: TabStyles,
    pub address_bar: AddressBarStyles,
    pub search_modal: SearchModalStyles,
    pub mobile_bar: MobileBarStyles,
    pub tab_drawer: TabDrawerStyles,
    pub settings_page: SettingsPageStyles,
    pub newtab: NewtabStyles,
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct KeybindsConfig {
    pub omnibox: String,
    pub new_tab: String,
    pub close_tab: String,
    pub reopen_closed_tab: String,
    pub next_tab: String,
    pub bookmark_page: String,
    pub find_in_page: String,
    pub reader_mode: String,
    pub force_dark: String,
    pub fullscreen: String,
    pub preferences: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub zoom_reset: String,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            omnibox: "Ctrl+L".to_string(),
            new_tab: "Ctrl+T".to_string(),
            close_tab: "Ctrl+W".to_string(),
            reopen_closed_tab: "Ctrl+Shift+T".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            bookmark_page: "Ctrl+D".to_string(),
            find_in_page: "Ctrl+F".to_string(),
            reader_mode: "Alt+R".to_string(),
            force_dark: "Alt+Shift+D".to_string(),
            fullscreen: "F11".to_string(),
            preferences: "Ctrl+,".to_string(),
            zoom_in: "Ctrl+=".to_string(),
            zoom_out: "Ctrl+-".to_string(),
            zoom_reset: "Ctrl+0".to_string(),
        }
    }
}

impl KeybindsConfig {
    pub fn path() -> PathBuf {
        synced_json_path("fynexo_keybinds.json")
    }

    pub fn path_display() -> String {
        display_path_preferring_local("fynexo_keybinds.json")
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("fynexo_keybinds.json")
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok(kb) = serde_json::from_str::<KeybindsConfig>(&raw) {
                    return kb;
                }
            }
        }
        let def = Self::default();
        def.save();
        def
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            synced_json_save("fynexo_keybinds.json", &json);
        }
    }
}

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

static DATA_DIR_CACHE: OnceLock<PathBuf> = OnceLock::new();

pub fn data_dir() -> &'static Path {
    DATA_DIR_CACHE.get_or_init(|| {
        if let Some(override_dir) = std::env::var_os("FYNEXO_DATA_DIR") {
            let p = PathBuf::from(override_dir);
            if fs::create_dir_all(&p).is_ok() {
                return p;
            }
        }

        #[cfg(target_os = "android")]
        {
            let pkg_files = PathBuf::from("/data/data/com.horizoralabs.fynexo/files");
            if pkg_files.exists() || fs::create_dir_all(&pkg_files).is_ok() {
                return pkg_files;
            }
            let fallback = std::env::temp_dir().join("fynexo");
            let _ = fs::create_dir_all(&fallback);
            return fallback;
        }

        #[cfg(target_os = "ios")]
        {
            if let Some(home) = std::env::var_os("HOME") {
                let docs = PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("Fynexo");
                if fs::create_dir_all(&docs).is_ok() {
                    return docs;
                }
            }
            return std::env::temp_dir();
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let base = std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
                .unwrap_or_else(|| PathBuf::from("."));
            let dir = base.join("Fynexo");
            let _ = fs::create_dir_all(&dir);

            for legacy_name in [
                "fynexo_config.json",
                "fynexo_styles.json",
                "fynexo_shortcuts.json",
                "fynexo_rules.json",
                "fynexo_adblock.json",
                "fynexo_keybinds.json",
            ] {
                let target = dir.join(legacy_name);
                let local = PathBuf::from(legacy_name);
                if !target.exists() && local.exists() {
                    let _ = fs::copy(&local, &target);
                }
            }
            dir
        }
    })
}

pub fn data_dir_display() -> String {
    let p = data_dir().to_path_buf();
    fs::canonicalize(&p)
        .unwrap_or(p)
        .to_string_lossy()
        .replace("\\\\?\\", "")
}

pub fn downloads_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        let pub_dl = PathBuf::from("/storage/emulated/0/Download");
        if pub_dl.is_dir() {
            return pub_dl;
        }
    }
    let dir = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|p| PathBuf::from(p).join("Downloads"))
        .unwrap_or_else(|| data_dir().join("Downloads"));
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn unique_download_path(suggested_filename: &str) -> PathBuf {
    let base_dir = downloads_dir();
    let raw_name = Path::new(suggested_filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download.bin");

    let sanitized: String = raw_name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();

    let clean_name = if sanitized.trim().is_empty() {
        "download.bin".to_string()
    } else {
        sanitized
    };

    let candidate = base_dir.join(&clean_name);
    if !candidate.exists() {
        return candidate;
    }

    let path_obj = Path::new(&clean_name);
    let stem = path_obj
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let ext = path_obj.extension().and_then(|e| e.to_str()).unwrap_or("");

    for i in 1..1000 {
        let numbered = if ext.is_empty() {
            format!("{} ({})", stem, i)
        } else {
            format!("{} ({}).{}", stem, i, ext)
        };
        let next = base_dir.join(numbered);
        if !next.exists() {
            return next;
        }
    }
    candidate
}

pub fn export_json_to_downloads(filename: &str, json_content: &str) -> Result<PathBuf, String> {
    let target = unique_download_path(filename);
    if fs::write(&target, json_content).is_ok() {
        return Ok(target);
    }
    let fallback = data_dir().join(filename);
    fs::write(&fallback, json_content).map_err(|e| e.to_string())?;
    Ok(fallback)
}

#[inline]
fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[inline]
fn normalize_url_key(url: &str) -> &str {
    let trimmed = url.trim();
    if trimmed.len() > 8 && trimmed.ends_with('/') && trimmed.matches('/').count() == 3 {
        &trimmed[..trimmed.len() - 1]
    } else {
        trimmed
    }
}

/// Syncs a workspace JSON file (`./<name>`) with `data_dir()/<name>` when present on desktop.
pub(crate) fn synced_json_path(file_name: &str) -> PathBuf {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local = PathBuf::from(file_name);
        let formal = data_dir().join(file_name);
        if local.exists() {
            let local_mt = fs::metadata(&local).and_then(|m| m.modified()).ok();
            let formal_mt = fs::metadata(&formal).and_then(|m| m.modified()).ok();
            if formal_mt.is_none() || local_mt > formal_mt {
                let _ = fs::copy(&local, &formal);
            }
        }
        formal
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        data_dir().join(file_name)
    }
}

pub(crate) fn synced_json_save(file_name: &str, content: &str) {
    let formal = data_dir().join(file_name);
    let _ = fs::write(&formal, content);
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local = PathBuf::from(file_name);
        if local.exists() {
            let _ = fs::write(&local, content);
        }
    }
}

pub(crate) fn synced_json_mtime(file_name: &str) -> Option<SystemTime> {
    let formal_mt = fs::metadata(data_dir().join(file_name))
        .and_then(|m| m.modified())
        .ok();
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local_mt = fs::metadata(file_name).and_then(|m| m.modified()).ok();
        return formal_mt.max(local_mt);
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        formal_mt
    }
}

pub(crate) fn display_path_preferring_local(file_name: &str) -> String {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local = PathBuf::from(file_name);
        if local.exists() {
            return fs::canonicalize(&local)
                .unwrap_or(local)
                .to_string_lossy()
                .replace("\\\\?\\", "");
        }
    }
    let p = data_dir().join(file_name);
    fs::canonicalize(&p)
        .unwrap_or(p)
        .to_string_lossy()
        .replace("\\\\?\\", "")
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        synced_json_path("fynexo_config.json")
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
            synced_json_save("fynexo_config.json", &json);
        }
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("fynexo_config.json")
    }
}

impl ComponentStyles {
    pub fn styles_path() -> PathBuf {
        synced_json_path("fynexo_styles.json")
    }

    pub fn styles_path_display() -> String {
        display_path_preferring_local("fynexo_styles.json")
    }

    pub fn load() -> Self {
        let path = Self::styles_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(styles) = serde_json::from_str::<ComponentStyles>(&content) {
                    return styles;
                }
            }
        }
        let default_styles = Self::default();
        default_styles.save();
        default_styles
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            synced_json_save("fynexo_styles.json", &json);
        }
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("fynexo_styles.json")
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
        synced_json_path("fynexo_shortcuts.json")
    }

    pub fn shortcuts_path_display() -> String {
        display_path_preferring_local("fynexo_shortcuts.json")
    }

    pub fn shortcuts_mtime() -> Option<SystemTime> {
        synced_json_mtime("fynexo_shortcuts.json")
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
                synced_json_save("fynexo_shortcuts.json", &json);
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
            synced_json_save("fynexo_shortcuts.json", &json);
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
