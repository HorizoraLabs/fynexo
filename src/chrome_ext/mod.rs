pub mod crx;
pub mod windows_bridge;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{
    data_dir, now_unix_secs, synced_json_path, synced_json_save,
};

use self::crx::{
    base64_encode, extract_zip_safe, is_valid_chrome_ext_id, StoreSource,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct ChromeExtensionEntry {
    pub id: String,
    pub runtime_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub dir_path: String,
    pub popup_path: String,
    pub options_path: String,
    pub icon_data_url: String,
    pub enabled: bool,
    pub pinned: bool,
    pub source: String,
    pub store_url: String,
    pub installed_at: u64,
}

impl Default for ChromeExtensionEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            runtime_id: String::new(),
            name: String::new(),
            version: String::new(),
            description: String::new(),
            dir_path: String::new(),
            popup_path: String::new(),
            options_path: String::new(),
            icon_data_url: String::new(),
            enabled: true,
            pinned: false,
            source: "crx_file".to_string(),
            store_url: String::new(),
            installed_at: 0,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct HudChromeExtension {
    pub id: String,
    pub runtime_id: String,
    pub name: String,
    pub version: String,
    pub popup_path: String,
    pub options_path: String,
    pub icon_data_url: String,
    pub enabled: bool,
    pub pinned: bool,
}

impl From<&ChromeExtensionEntry> for HudChromeExtension {
    fn from(e: &ChromeExtensionEntry) -> Self {
        Self {
            id: e.id.clone(),
            runtime_id: e.runtime_id.clone(),
            name: e.name.clone(),
            version: e.version.clone(),
            popup_path: e.popup_path.clone(),
            options_path: e.options_path.clone(),
            icon_data_url: e.icon_data_url.clone(),
            enabled: e.enabled,
            pinned: e.pinned,
        }
    }
}

pub fn chrome_extensions_dir() -> PathBuf {
    let dir = data_dir().join("chrome_extensions");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn chrome_extensions_dir_display() -> String {
    let p = chrome_extensions_dir();
    fs::canonicalize(&p)
        .unwrap_or(p)
        .to_string_lossy()
        .replace("\\\\?\\", "")
}

pub fn load_registry() -> Vec<ChromeExtensionEntry> {
    let path = synced_json_path("chrome_extensions.json");
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(mut list) = serde_json::from_str::<Vec<ChromeExtensionEntry>>(&raw) {
            list.retain(|e| {
                !e.id.is_empty() && !e.dir_path.is_empty() && Path::new(&e.dir_path).is_dir()
            });
            return list;
        }
    }
    Vec::new()
}

pub fn save_registry(list: &[ChromeExtensionEntry]) {
    if let Ok(json) = serde_json::to_string_pretty(list) {
        synced_json_save("chrome_extensions.json", &json);
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParsedManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub popup_path: String,
    pub options_path: String,
    pub icon_data_url: String,
}

fn load_locale_messages(dir: &Path, locale: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let msg_path = dir.join("_locales").join(locale).join("messages.json");
    let Ok(raw) = fs::read_to_string(&msg_path) else {
        return map;
    };
    let clean = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
    let Ok(serde_json::Value::Object(obj)) = serde_json::from_str::<serde_json::Value>(clean)
    else {
        return map;
    };
    for (k, v) in obj {
        if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
            map.insert(k.to_ascii_lowercase(), msg.to_string());
        }
    }
    map
}

fn resolve_i18n_string(raw: &str, tables: &[HashMap<String, String>]) -> String {
    let trimmed = raw.trim();
    if let Some(inner) = trimmed
        .strip_prefix("__MSG_")
        .and_then(|s| s.strip_suffix("__"))
    {
        let key = inner.trim().to_ascii_lowercase();
        for tbl in tables {
            if let Some(found) = tbl.get(&key) {
                return found.clone();
            }
        }
    }
    trimmed.to_string()
}

fn pick_icon_rel_path(val: &serde_json::Value) -> Option<String> {
    if let Some(s) = val.as_str() {
        let clean = s.trim().trim_start_matches('/').to_string();
        return if clean.is_empty() { None } else { Some(clean) };
    }
    let obj = val.as_object()?;
    for pref in ["48", "32", "64", "128", "16", "24", "96", "256"] {
        if let Some(s) = obj.get(pref).and_then(|v| v.as_str()) {
            let clean = s.trim().trim_start_matches('/').to_string();
            if !clean.is_empty() {
                return Some(clean);
            }
        }
    }
    for v in obj.values() {
        if let Some(s) = v.as_str() {
            let clean = s.trim().trim_start_matches('/').to_string();
            if !clean.is_empty() {
                return Some(clean);
            }
        }
    }
    None
}

fn load_icon_data_url(ext_dir: &Path, rel_path: &str) -> String {
    if rel_path.contains("..") || rel_path.contains('\0') {
        return String::new();
    }
    let clean_rel = rel_path.replace('\\', "/");
    let full = ext_dir.join(clean_rel.trim_start_matches('/'));
    let Ok(bytes) = fs::read(&full) else {
        return String::new();
    };
    if bytes.is_empty() || bytes.len() > 256 * 1024 {
        return String::new();
    }
    let ext = full
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "gif" => "image/gif",
        _ => "image/png",
    };
    format!("data:{};base64,{}", mime, base64_encode(&bytes))
}

fn normalize_ext_page_path(raw: Option<&str>) -> String {
    let s = raw.unwrap_or("").trim().replace('\\', "/");
    let trimmed = s.trim_start_matches('/').trim_start_matches("./");
    if trimmed.contains("..") {
        String::new()
    } else {
        trimmed.to_string()
    }
}

pub fn parse_unpacked_manifest(dir: &Path, preferred_lang: &str) -> Result<ParsedManifest, String> {
    let mpath = dir.join("manifest.json");
    let raw = fs::read_to_string(&mpath)
        .map_err(|_| "manifest.json not found in extension folder".to_string())?;
    let clean = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
    let val: serde_json::Value =
        serde_json::from_str(clean).map_err(|e| format!("Invalid manifest.json: {}", e))?;

    let default_locale = val
        .get("default_locale")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    let mut locale_candidates: Vec<&str> = Vec::new();
    if preferred_lang.eq_ignore_ascii_case("zh") {
        locale_candidates.extend(["zh_CN", "zh-CN", "zh"]);
    }
    if !default_locale.is_empty() {
        locale_candidates.push(default_locale);
    }
    locale_candidates.extend(["en", "en_US"]);

    let mut tables = Vec::new();
    for loc in locale_candidates {
        let tbl = load_locale_messages(dir, loc);
        if !tbl.is_empty() {
            tables.push(tbl);
        }
    }

    let raw_name = val.get("name").and_then(|v| v.as_str()).unwrap_or("Extension");
    let name = {
        let r = resolve_i18n_string(raw_name, &tables);
        if r.is_empty() {
            "Chrome Extension".to_string()
        } else {
            r
        }
    };
    let version = val
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0")
        .trim()
        .to_string();
    let raw_desc = val
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let description = resolve_i18n_string(raw_desc, &tables);

    let popup_raw = val
        .pointer("/action/default_popup")
        .and_then(|v| v.as_str())
        .or_else(|| {
            val.pointer("/browser_action/default_popup")
                .and_then(|v| v.as_str())
        })
        .or_else(|| {
            val.pointer("/page_action/default_popup")
                .and_then(|v| v.as_str())
        });
    let popup_path = normalize_ext_page_path(popup_raw);

    let options_raw = val
        .pointer("/options_ui/page")
        .and_then(|v| v.as_str())
        .or_else(|| val.get("options_page").and_then(|v| v.as_str()));
    let options_path = normalize_ext_page_path(options_raw);

    let icon_rel = val
        .pointer("/action/default_icon")
        .and_then(pick_icon_rel_path)
        .or_else(|| {
            val.pointer("/browser_action/default_icon")
                .and_then(pick_icon_rel_path)
        })
        .or_else(|| val.get("icons").and_then(pick_icon_rel_path));
    let icon_data_url = icon_rel
        .as_deref()
        .map(|rel| load_icon_data_url(dir, rel))
        .unwrap_or_default();

    Ok(ParsedManifest {
        name,
        version,
        description,
        popup_path,
        options_path,
        icon_data_url,
    })
}

fn fnv1a_hex(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in s.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (h & 0xFFFF_FFFF) as u32)
}

fn sanitize_slug(id_hint: Option<&str>, name: &str) -> String {
    if let Some(id) = id_hint {
        if is_valid_chrome_ext_id(id.trim()) {
            return id.trim().to_string();
        }
    }
    let ascii: String = name
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(24)
        .collect();
    let stem = if ascii.is_empty() { "ext" } else { ascii.as_str() };
    format!("local-{}-{}", stem, fnv1a_hex(&format!("{}-{}", name, now_unix_secs())))
}

pub fn install_from_archive_bytes(
    id_hint: Option<&str>,
    raw: &[u8],
    source: &str,
    store_url: &str,
    lang: &str,
) -> Result<ChromeExtensionEntry, String> {
    let require_proof = source == "chrome_store" || source == "edge_store";
    let (zip_bytes, crx_ext_id) =
        crate::chrome_ext::crx::verify_and_strip_crx(raw, id_hint, require_proof)?;
    let root = chrome_extensions_dir();
    let stage = root.join(format!("_staging_{}", now_unix_secs()));
    let _ = fs::remove_dir_all(&stage);

    if let Err(e) = extract_zip_safe(zip_bytes, &stage) {
        let _ = fs::remove_dir_all(&stage);
        return Err(e);
    }

    let manifest = match parse_unpacked_manifest(&stage, lang) {
        Ok(m) => m,
        Err(e) => {
            let _ = fs::remove_dir_all(&stage);
            return Err(e);
        }
    };

    let effective_id_hint = id_hint.or(crx_ext_id.as_deref());
    let slug = sanitize_slug(effective_id_hint, &manifest.name);
    let target_dir = root.join(&slug);
    let _ = fs::remove_dir_all(&target_dir);
    if let Err(e) = fs::rename(&stage, &target_dir) {
        let _ = fs::remove_dir_all(&stage);
        return Err(format!("Failed moving unpacked extension: {}", e));
    }

    let runtime_id = if is_valid_chrome_ext_id(&slug) {
        slug.clone()
    } else {
        String::new()
    };
    let pinned = !manifest.popup_path.is_empty() || !manifest.options_path.is_empty();

    Ok(ChromeExtensionEntry {
        id: slug,
        runtime_id,
        name: manifest.name,
        version: manifest.version,
        description: manifest.description,
        dir_path: target_dir.to_string_lossy().to_string(),
        popup_path: manifest.popup_path,
        options_path: manifest.options_path,
        icon_data_url: manifest.icon_data_url,
        enabled: true,
        pinned,
        source: source.to_string(),
        store_url: store_url.to_string(),
        installed_at: now_unix_secs(),
    })
}

fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    files: &mut usize,
    total_bytes: &mut usize,
) -> Result<(), String> {
    const MAX_FILES: usize = 4096;
    const MAX_TOTAL: usize = 128 * 1024 * 1024;
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            continue;
        }
        let target = dst.join(entry.file_name());
        if meta.is_dir() {
            copy_dir_recursive(&path, &target, files, total_bytes)?;
        } else if meta.is_file() {
            *files += 1;
            if *files > MAX_FILES {
                return Err("Extension folder exceeds 4096 files limit".to_string());
            }
            *total_bytes += meta.len() as usize;
            if *total_bytes > MAX_TOTAL {
                return Err("Extension folder exceeds 128 MB limit".to_string());
            }
            fs::copy(&path, &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn install_from_local_dir(src_dir: &Path, lang: &str) -> Result<ChromeExtensionEntry, String> {
    if !src_dir.is_dir() || !src_dir.join("manifest.json").is_file() {
        return Err("Selected folder does not contain manifest.json".to_string());
    }
    let manifest = parse_unpacked_manifest(src_dir, lang)?;
    let slug = sanitize_slug(None, &manifest.name);
    let root = chrome_extensions_dir();
    let target_dir = root.join(&slug);
    let _ = fs::remove_dir_all(&target_dir);
    let mut files = 0usize;
    let mut bytes = 0usize;
    if let Err(e) = copy_dir_recursive(src_dir, &target_dir, &mut files, &mut bytes) {
        let _ = fs::remove_dir_all(&target_dir);
        return Err(e);
    }
    let pinned = !manifest.popup_path.is_empty() || !manifest.options_path.is_empty();
    Ok(ChromeExtensionEntry {
        id: slug,
        runtime_id: String::new(),
        name: manifest.name,
        version: manifest.version,
        description: manifest.description,
        dir_path: target_dir.to_string_lossy().to_string(),
        popup_path: manifest.popup_path,
        options_path: manifest.options_path,
        icon_data_url: manifest.icon_data_url,
        enabled: true,
        pinned,
        source: "local_dir".to_string(),
        store_url: String::new(),
        installed_at: now_unix_secs(),
    })
}

pub fn remove_extension_files(entry: &ChromeExtensionEntry) {
    let root = chrome_extensions_dir();
    let Ok(canon_root) = fs::canonicalize(&root) else {
        return;
    };
    let target = PathBuf::from(&entry.dir_path);
    let Ok(canon_target) = fs::canonicalize(&target) else {
        return;
    };
    if canon_target != canon_root && canon_target.starts_with(&canon_root) {
        let _ = fs::remove_dir_all(canon_target);
    }
}

#[cfg(target_os = "windows")]
fn wininet_fetch_bytes(url: &str, max_bytes: usize) -> Result<Vec<u8>, String> {
    use core::ffi::c_void;
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
        HTTP_QUERY_FLAG_NUMBER, HTTP_QUERY_STATUS_CODE, INTERNET_FLAG_RELOAD, INTERNET_FLAG_SECURE,
        INTERNET_OPEN_TYPE_PRECONFIG,
    };

    unsafe {
        let h_net = InternetOpenW(
            w!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"),
            INTERNET_OPEN_TYPE_PRECONFIG.0,
            None,
            None,
            0,
        );
        if h_net.is_null() {
            return Err("Failed to initialize WinInet session".to_string());
        }

        let url_hs = HSTRING::from(url);
        let h_url = InternetOpenUrlW(
            h_net,
            &url_hs,
            None,
            INTERNET_FLAG_RELOAD | INTERNET_FLAG_SECURE,
            None,
        );
        if h_url.is_null() {
            let _ = InternetCloseHandle(h_net);
            return Err("Network request failed (check connection/proxy)".to_string());
        }

        let mut status_code: u32 = 0;
        let mut status_len: u32 = std::mem::size_of::<u32>() as u32;
        let q_ok = HttpQueryInfoW(
            h_url,
            HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
            Some((&mut status_code as *mut u32).cast::<c_void>()),
            &mut status_len,
            None,
        )
        .is_ok();
        if q_ok && status_code != 200 {
            let _ = InternetCloseHandle(h_url);
            let _ = InternetCloseHandle(h_net);
            return Err(format!("Store server returned HTTP {}", status_code));
        }

        let mut out = Vec::with_capacity(64 * 1024);
        let mut buf = vec![0u8; 32 * 1024];
        loop {
            let mut read: u32 = 0;
            if InternetReadFile(
                h_url,
                buf.as_mut_ptr().cast::<c_void>(),
                buf.len() as u32,
                &mut read,
            )
            .is_err()
            {
                let _ = InternetCloseHandle(h_url);
                let _ = InternetCloseHandle(h_net);
                return Err("Interrupted while downloading extension package".to_string());
            }
            if read == 0 {
                break;
            }
            let next_len = out.len().saturating_add(read as usize);
            if next_len > max_bytes {
                let _ = InternetCloseHandle(h_url);
                let _ = InternetCloseHandle(h_net);
                return Err("Extension package exceeds 64 MB download limit".to_string());
            }
            out.extend_from_slice(&buf[..read as usize]);
        }

        let _ = InternetCloseHandle(h_url);
        let _ = InternetCloseHandle(h_net);
        Ok(out)
    }
}

#[cfg(target_os = "windows")]
pub fn download_store_crx(ext_id: &str, source: StoreSource) -> Result<Vec<u8>, String> {
    const MAX_CRX_BYTES: usize = 64 * 1024 * 1024;
    if !is_valid_chrome_ext_id(ext_id) {
        return Err("Invalid 32-character Chrome/Edge extension ID".to_string());
    }
    let chrome_url = format!(
        "https://clients2.google.com/service/update2/crx?response=redirect&prodversion=134.0.6998.89&acceptformat=crx2,crx3&x=id%3D{}%26uc",
        ext_id
    );
    let edge_url = format!(
        "https://edge.microsoft.com/extensionwebstorebase/v1/crx?response=redirect&prod=chromiumcrx&prodchannel=&x=id%3D{}%26installsource%3Dondemand%26uc",
        ext_id
    );
    let (primary, fallback) = match source {
        StoreSource::Edge => (&edge_url, &chrome_url),
        StoreSource::Chrome => (&chrome_url, &edge_url),
    };
    match wininet_fetch_bytes(primary, MAX_CRX_BYTES) {
        Ok(bytes) if bytes.len() >= 16 => Ok(bytes),
        primary_res => match wininet_fetch_bytes(fallback, MAX_CRX_BYTES) {
            Ok(bytes) if bytes.len() >= 16 => Ok(bytes),
            Err(e2) => match primary_res {
                Err(e1) => Err(format!("{} (fallback: {})", e1, e2)),
                Ok(_) => Err(format!("Empty store response ({})", e2)),
            },
            Ok(_) => Err("Empty store response".to_string()),
        },
    }
}

#[cfg(not(target_os = "windows"))]
pub fn download_store_crx(_ext_id: &str, _source: StoreSource) -> Result<Vec<u8>, String> {
    Err("Chrome extensions are supported on Windows desktop only".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_parsing_resolves_i18n_and_icon() {
        let dir = std::env::temp_dir().join("fynexo_manifest_i18n_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("_locales").join("zh_CN")).unwrap();
        fs::write(
            dir.join("_locales").join("zh_CN").join("messages.json"),
            "\u{feff}{\"extName\":{\"message\":\"测试扩展\"},\"extDesc\":{\"message\":\"中文描述\"}}",
        )
        .unwrap();
        fs::write(dir.join("icon48.png"), b"\x89PNG\r\n\x1a\n1234").unwrap();
        fs::write(
            dir.join("manifest.json"),
            r#"{
                "manifest_version": 3,
                "name": "__MSG_extName__",
                "description": "__MSG_extDesc__",
                "version": "2.4.1",
                "default_locale": "zh_CN",
                "action": {
                    "default_popup": "/popup.html",
                    "default_icon": { "16": "icon48.png", "48": "icon48.png" }
                },
                "options_ui": { "page": "options.html" }
            }"#,
        )
        .unwrap();

        let parsed = parse_unpacked_manifest(&dir, "zh").unwrap();
        assert_eq!(parsed.name, "测试扩展");
        assert_eq!(parsed.description, "中文描述");
        assert_eq!(parsed.version, "2.4.1");
        assert_eq!(parsed.popup_path, "popup.html");
        assert_eq!(parsed.options_path, "options.html");
        assert!(parsed.icon_data_url.starts_with("data:image/png;base64,"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_dir_install_copies_and_registers_cleanly() {
        crate::util::isolate_data_dir();
        let src = std::env::temp_dir().join("fynexo_local_ext_src");
        let _ = fs::remove_dir_all(&src);
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("manifest.json"),
            r#"{"manifest_version":3,"name":"Demo Ext","version":"0.9","action":{"default_popup":"index.html"}}"#,
        )
        .unwrap();
        fs::write(src.join("index.html"), b"<html></html>").unwrap();

        let entry = install_from_local_dir(&src, "en").unwrap();
        assert_eq!(entry.name, "Demo Ext");
        assert_eq!(entry.popup_path, "index.html");
        assert!(entry.pinned);
        assert!(Path::new(&entry.dir_path).join("index.html").is_file());

        remove_extension_files(&entry);
        assert!(!Path::new(&entry.dir_path).exists());
        let _ = fs::remove_dir_all(&src);
    }
}
