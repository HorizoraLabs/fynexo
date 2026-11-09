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
    if fs::write(&target, json_content).is_ok() {
        return Ok(target);
    }
    let fallback = data_dir().join(filename);
    fs::write(&fallback, json_content).map_err(|e| e.to_string())?;
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

pub(crate) fn resolve_local_config(file_name: &str) -> Option<PathBuf> {
    let (clean, legacy) = candidate_names(file_name);
    let p_clean_cfg = Path::new("config").join(&clean);
    if p_clean_cfg.exists() {
        return Some(p_clean_cfg);
    }
    let p_legacy_cfg = Path::new("config").join(&legacy);
    if p_legacy_cfg.exists() {
        return Some(p_legacy_cfg);
    }
    let p_legacy_root = PathBuf::from(&legacy);
    if p_legacy_root.exists() {
        return Some(p_legacy_root);
    }
    let p_clean_root = PathBuf::from(&clean);
    if p_clean_root.exists() {
        return Some(p_clean_root);
    }
    None
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
    let _ = fs::write(&formal, content);
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Some(local) = resolve_local_config(file_name) {
            let _ = fs::write(&local, content);
        } else if Path::new("config").is_dir() {
            let (clean, _) = candidate_names(file_name);
            let _ = fs::write(Path::new("config").join(clean), content);
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
