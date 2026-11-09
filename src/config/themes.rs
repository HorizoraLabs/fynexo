use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::base::data_dir;

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

pub fn sanitize_theme_id(raw: &str) -> String {
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
