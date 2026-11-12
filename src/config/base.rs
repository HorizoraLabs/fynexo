use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DESKTOP_CHROME_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36";
pub const MOBILE_CHROME_UA: &str = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36";

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

            for name in [
                "config.json",
                "styles.json",
                "shortcuts.json",
                "rules.json",
                "adblock.json",
                "keybinds.json",
            ] {
                let (clean, legacy) = candidate_names(name);
                let target_clean = dir.join(&clean);
                let target_legacy = dir.join(&legacy);
                if !target_clean.exists() && !target_legacy.exists() {
                    if let Some(local) = resolve_local_config(name) {
                        let _ = fs::copy(&local, &target_clean);
                    }
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
    if crate::util::atomic_write(&target, json_content).is_ok() {
        return Ok(target);
    }
    let fallback = data_dir().join(filename);
    crate::util::atomic_write(&fallback, json_content).map_err(|e| e.to_string())?;
    Ok(fallback)
}

#[inline]
pub fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[inline]
pub fn normalize_url_key(url: &str) -> &str {
    let trimmed = url.trim();
    if trimmed.len() > 8 && trimmed.ends_with('/') && trimmed.matches('/').count() == 3 {
        &trimmed[..trimmed.len() - 1]
    } else {
        trimmed
    }
}

pub(crate) fn candidate_names(file_name: &str) -> (String, String) {
    if let Some(stripped) = file_name.strip_prefix("fynexo_") {
        (stripped.to_string(), file_name.to_string())
    } else {
        (file_name.to_string(), format!("fynexo_{}", file_name))
    }
}

/// Directories that may hold project-local preset JSON files (desktop only):
/// `<cwd>/config` (dev: `cargo run` from the repo) and `<exe dir>/config`
/// (portable distribution). Resolved once so launching from another working
/// directory still finds the presets next to the executable.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn local_config_dirs() -> &'static [PathBuf] {
    static DIRS: OnceLock<Vec<PathBuf>> = OnceLock::new();
    DIRS.get_or_init(|| {
        let mut dirs = vec![PathBuf::from("config")];
        if let Some(exe_cfg) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("config")))
        {
            let same_as_cwd = fs::canonicalize(&exe_cfg).ok() == fs::canonicalize("config").ok();
            if !same_as_cwd {
                dirs.push(exe_cfg);
            }
        }
        dirs
    })
}

#[cfg_attr(any(target_os = "android", target_os = "ios"), allow(dead_code))]
pub(crate) fn resolve_local_config(file_name: &str) -> Option<PathBuf> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let (clean, legacy) = candidate_names(file_name);
        for dir in local_config_dirs() {
            for name in [&clean, &legacy] {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        // Legacy layout: `fynexo_*.json` directly in the working directory.
        let legacy_root = PathBuf::from(&legacy);
        if legacy_root.is_file() {
            return Some(legacy_root);
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let _ = file_name;
    None
}

/// Existing local `config/` directory to write fresh files into (desktop only).
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn writable_local_config_dir() -> Option<&'static PathBuf> {
    local_config_dirs().iter().find(|d| d.is_dir())
}

pub(crate) fn resolve_formal_path(file_name: &str) -> PathBuf {
    let (clean, legacy) = candidate_names(file_name);
    let formal_clean = data_dir().join(&clean);
    if formal_clean.exists() {
        return formal_clean;
    }
    let formal_legacy = data_dir().join(&legacy);
    if formal_legacy.exists() {
        return formal_legacy;
    }
    data_dir().join(&clean)
}

pub(crate) fn synced_json_path(file_name: &str) -> PathBuf {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local_opt = resolve_local_config(file_name);
        let formal = resolve_formal_path(file_name);
        if let Some(ref local) = local_opt {
            let local_mt = fs::metadata(local).and_then(|m| m.modified()).ok();
            let formal_mt = fs::metadata(&formal).and_then(|m| m.modified()).ok();
            if formal_mt.is_none() || local_mt > formal_mt {
                let _ = fs::copy(local, &formal);
            }
        }
        formal
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        resolve_formal_path(file_name)
    }
}

pub(crate) fn synced_json_save(file_name: &str, content: &str) {
    let formal = resolve_formal_path(file_name);
    let _ = crate::util::atomic_write(&formal, content);
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Some(local) = resolve_local_config(file_name) {
            let _ = crate::util::atomic_write(&local, content);
        } else if let Some(dir) = writable_local_config_dir() {
            let (clean, _) = candidate_names(file_name);
            let _ = crate::util::atomic_write(dir.join(clean), content);
        }
    }
}

pub(crate) fn synced_json_mtime(file_name: &str) -> Option<SystemTime> {
    let formal_mt = fs::metadata(resolve_formal_path(file_name))
        .and_then(|m| m.modified())
        .ok();
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let local_mt = resolve_local_config(file_name)
            .and_then(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
        formal_mt.max(local_mt)
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        formal_mt
    }
}

pub(crate) fn display_path_preferring_local(file_name: &str) -> String {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Some(local) = resolve_local_config(file_name) {
            return fs::canonicalize(&local)
                .unwrap_or(local)
                .to_string_lossy()
                .replace("\\\\?\\", "");
        }
    }
    let p = resolve_formal_path(file_name);
    fs::canonicalize(&p)
        .unwrap_or(p)
        .to_string_lossy()
        .replace("\\\\?\\", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_names_handle_both_naming_schemes() {
        assert_eq!(candidate_names("styles.json"), ("styles.json".into(), "fynexo_styles.json".into()));
        assert_eq!(candidate_names("fynexo_styles.json"), ("styles.json".into(), "fynexo_styles.json".into()));
    }

    #[test]
    fn normalize_url_key_strips_only_bare_trailing_slash() {
        assert_eq!(normalize_url_key("https://a.com/"), "https://a.com");
        assert_eq!(normalize_url_key("https://a.com/path/"), "https://a.com/path/");
        assert_eq!(normalize_url_key("  https://a.com  "), "https://a.com");
    }

    #[test]
    fn unique_download_path_sanitises_and_avoids_collisions() {
        let p = unique_download_path("..\\evil:<name>?.txt");
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        assert!(!name.contains(['<', '>', ':', '?', '\\', '/']));
        assert!(unique_download_path("").file_name().is_some());
    }
}
