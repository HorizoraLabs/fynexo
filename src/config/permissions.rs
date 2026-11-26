use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use super::base::{synced_json_mtime, synced_json_path, synced_json_save};

pub const PERMISSION_KINDS: [&str; 5] = [
    "camera",
    "microphone",
    "geolocation",
    "notifications",
    "clipboard",
];

/// Extract normalized `scheme://host[:port]` origin from an `http://` or `https://` URL.
pub fn normalize_permission_origin(url: &str) -> Option<String> {
    let trimmed = url.trim();
    let lower = trimmed.to_ascii_lowercase();
    let (scheme, rest) = if let Some(r) = lower.strip_prefix("https://") {
        ("https", r)
    } else {
        let r = lower.strip_prefix("http://")?;
        ("http", r)
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or("");
    if host_port.is_empty() || host_port.contains(' ') {
        return None;
    }
    let host_only = host_port.split(':').next().unwrap_or("");
    if host_only.is_empty() || host_only == "fynexo.localhost" {
        return None;
    }
    Some(format!("{}://{}", scheme, host_port))
}

pub fn normalize_permission_kind(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "camera" | "video" => Some("camera"),
        "microphone" | "mic" | "audio" => Some("microphone"),
        "geolocation" | "location" => Some("geolocation"),
        "notifications" | "notification" => Some("notifications"),
        "clipboard" | "clipboard_read" => Some("clipboard"),
        _ => None,
    }
}

pub fn normalize_permission_state(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "allow" | "allowed" | "grant" => "allow",
        "deny" | "denied" | "block" => "deny",
        _ => "ask",
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default)]
pub struct SitePermissionsStore {
    pub origins: BTreeMap<String, BTreeMap<String, String>>,
}

impl SitePermissionsStore {
    pub fn path() -> PathBuf {
        synced_json_path("site_permissions.json")
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("site_permissions.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(mut store) = serde_json::from_str::<Self>(&raw) {
                    store.sanitize();
                    return store;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            synced_json_save("site_permissions.json", &json);
        }
    }

    pub fn sanitize(&mut self) {
        let mut clean = BTreeMap::new();
        for (raw_origin, map) in &self.origins {
            let Some(origin) = normalize_permission_origin(raw_origin) else {
                continue;
            };
            let mut clean_map = BTreeMap::new();
            for (k, v) in map {
                if let Some(kind) = normalize_permission_kind(k) {
                    let state = normalize_permission_state(v);
                    if state != "ask" {
                        clean_map.insert(kind.to_string(), state.to_string());
                    }
                }
            }
            if !clean_map.is_empty() {
                clean.insert(origin, clean_map);
            }
        }
        self.origins = clean;
    }

    pub fn get(&self, uri: &str, kind: &str) -> Option<&str> {
        let origin = normalize_permission_origin(uri)?;
        let norm_kind = normalize_permission_kind(kind)?;
        self.origins
            .get(&origin)
            .and_then(|m| m.get(norm_kind))
            .map(|s| s.as_str())
    }

    pub fn set(&mut self, uri: &str, kind: &str, state: &str) -> bool {
        let Some(origin) = normalize_permission_origin(uri) else {
            return false;
        };
        let Some(norm_kind) = normalize_permission_kind(kind) else {
            return false;
        };
        let norm_state = normalize_permission_state(state);
        if norm_state == "ask" {
            let mut changed = false;
            if let Some(map) = self.origins.get_mut(&origin) {
                changed = map.remove(norm_kind).is_some();
                if map.is_empty() {
                    self.origins.remove(&origin);
                }
            }
            if changed {
                self.save();
            }
            return changed;
        }
        let entry = self.origins.entry(origin).or_default();
        let prev = entry.get(norm_kind).map(|s| s.as_str());
        if prev == Some(norm_state) {
            return false;
        }
        entry.insert(norm_kind.to_string(), norm_state.to_string());
        self.save();
        true
    }

    pub fn clear_origin(&mut self, uri: &str) -> bool {
        let Some(origin) = normalize_permission_origin(uri) else {
            return false;
        };
        if self.origins.remove(&origin).is_some() {
            self.save();
            true
        } else {
            false
        }
    }

    pub fn clear_all(&mut self) -> bool {
        if self.origins.is_empty() {
            return false;
        }
        self.origins.clear();
        self.save();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_origin_normalization_and_store_crud() {
        crate::util::isolate_data_dir();
        assert_eq!(
            normalize_permission_origin("https://Meet.Google.com/abc?x=1"),
            Some("https://meet.google.com".to_string())
        );
        assert_eq!(
            normalize_permission_origin("http://localhost:3000/app"),
            Some("http://localhost:3000".to_string())
        );
        assert_eq!(normalize_permission_origin("fynexo://settings"), None);
        assert_eq!(normalize_permission_origin("http://fynexo.localhost/hud"), None);

        let mut store = SitePermissionsStore::default();
        assert!(store.set("https://meet.google.com/room", "camera", "allow"));
        assert!(store.set("https://meet.google.com/room", "microphone", "deny"));
        assert_eq!(store.get("https://meet.google.com/other", "camera"), Some("allow"));
        assert_eq!(store.get("https://meet.google.com/other", "microphone"), Some("deny"));
        assert_eq!(store.get("https://meet.google.com/other", "geolocation"), None);

        // Resetting back to "ask" cleans up individual entries and empty origins
        assert!(store.set("https://meet.google.com", "camera", "ask"));
        assert_eq!(store.get("https://meet.google.com", "camera"), None);
        assert!(store.set("https://meet.google.com", "microphone", "ask"));
        assert!(store.origins.is_empty());
    }
}
