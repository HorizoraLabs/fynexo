use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use super::base::{now_unix_secs, root_data_dir};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct ProfileEntry {
    pub id: String,
    pub name: String,
    pub avatar_color: String,
    pub created_at: u64,
}

impl Default for ProfileEntry {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Personal".to_string(),
            avatar_color: "#7C86FF".to_string(),
            created_at: 0,
        }
    }
}

pub fn sanitize_profile_id(raw: &str) -> Option<String> {
    let clean: String = raw
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = clean.trim_matches(['-', '_']);
    if trimmed.is_empty() || trimmed.len() > 32 || trimmed == "." || trimmed == ".." {
        return None;
    }
    Some(trimmed.to_string())
}

static ACTIVE_PROFILE_CACHE: OnceLock<String> = OnceLock::new();

/// Active profile ID from `--profile=<id>` CLI arg or `FYNEXO_PROFILE` env var (defaults to `"default"`).
pub fn active_profile_id() -> &'static str {
    ACTIVE_PROFILE_CACHE
        .get_or_init(|| {
            let mut args = std::env::args().skip(1);
            while let Some(arg) = args.next() {
                if let Some(val) = arg.strip_prefix("--profile=") {
                    if let Some(id) = sanitize_profile_id(val) {
                        return id;
                    }
                } else if arg == "--profile" {
                    if let Some(val) = args.next() {
                        if let Some(id) = sanitize_profile_id(&val) {
                            return id;
                        }
                    }
                }
            }
            if let Ok(env_val) = std::env::var("FYNEXO_PROFILE") {
                if let Some(id) = sanitize_profile_id(&env_val) {
                    return id;
                }
            }
            "default".to_string()
        })
        .as_str()
}

fn profiles_registry_path() -> PathBuf {
    root_data_dir().join("profiles.json")
}

pub fn load_profiles() -> Vec<ProfileEntry> {
    let mut list: Vec<ProfileEntry> = fs::read_to_string(profiles_registry_path())
        .ok()
        .and_then(|raw| serde_json::from_str::<Vec<ProfileEntry>>(&raw).ok())
        .unwrap_or_default();

    list.retain(|p| sanitize_profile_id(&p.id).is_some());
    if !list.iter().any(|p| p.id == "default") {
        list.insert(0, ProfileEntry::default());
    }
    let active = active_profile_id();
    if active != "default" && !list.iter().any(|p| p.id == active) {
        list.push(ProfileEntry {
            id: active.to_string(),
            name: active.to_string(),
            avatar_color: "#38BDF8".to_string(),
            created_at: now_unix_secs(),
        });
    }
    list
}

pub fn save_profiles(list: &[ProfileEntry]) {
    if let Ok(json) = serde_json::to_string_pretty(list) {
        let _ = crate::util::atomic_write(profiles_registry_path(), json.as_bytes());
    }
}

pub fn create_profile(raw_name: &str, color: Option<&str>) -> Result<ProfileEntry, String> {
    let name = raw_name.trim();
    if name.is_empty() || name.len() > 40 {
        return Err("Profile name must be 1..40 characters".to_string());
    }
    let mut list = load_profiles();
    let base_slug = sanitize_profile_id(name).unwrap_or_else(|| "profile".to_string());
    let mut id = if base_slug == "default" {
        "profile-1".to_string()
    } else {
        base_slug.clone()
    };
    let mut n = 2u32;
    while list.iter().any(|p| p.id == id) {
        id = format!("{}-{}", base_slug, n);
        n += 1;
    }
    let palette = ["#7C86FF", "#38BDF8", "#34D399", "#F472B6", "#FB923C", "#A78BFA"];
    let avatar_color = color
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with('#') && (s.len() == 7 || s.len() == 4))
        .unwrap_or_else(|| palette[list.len() % palette.len()].to_string());

    let entry = ProfileEntry {
        id: id.clone(),
        name: name.to_string(),
        avatar_color,
        created_at: now_unix_secs(),
    };
    let _ = fs::create_dir_all(root_data_dir().join("profiles").join(&id));
    list.push(entry.clone());
    save_profiles(&list);
    Ok(entry)
}

pub fn delete_profile(id: &str) -> Result<bool, String> {
    let clean = sanitize_profile_id(id).ok_or_else(|| "Invalid profile id".to_string())?;
    if clean == "default" {
        return Err("Cannot delete default profile".to_string());
    }
    if clean == active_profile_id() {
        return Err("Cannot delete the currently running profile".to_string());
    }
    let mut list = load_profiles();
    let before = list.len();
    list.retain(|p| p.id != clean);
    if list.len() == before {
        return Ok(false);
    }
    save_profiles(&list);
    let pdir = root_data_dir().join("profiles").join(&clean);
    if pdir.is_dir() {
        let _ = fs::remove_dir_all(&pdir);
    }
    Ok(true)
}

pub fn launch_profile_process(profile_id: &str) -> Result<(), String> {
    let clean = sanitize_profile_id(profile_id).ok_or_else(|| "Invalid profile id".to_string())?;
    let exe = std::env::current_exe().map_err(|e| format!("current_exe failed: {}", e))?;
    std::process::Command::new(exe)
        .arg(format!("--profile={}", clean))
        .spawn()
        .map_err(|e| format!("Failed launching profile: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_ids_are_sanitised_safely() {
        assert_eq!(sanitize_profile_id("Work Account").as_deref(), Some("work-account"));
        assert_eq!(sanitize_profile_id("../evil").as_deref(), Some("evil"));
        assert_eq!(sanitize_profile_id("   "), None);
        assert_eq!(sanitize_profile_id(".."), None);
    }
}
