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
