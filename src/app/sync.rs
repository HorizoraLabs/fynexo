//! Portable WebDAV & GitHub Gist Cloud Sync (Bookmarks, Preferences, Site Rules, Custom Themes).
//!
//! Uses native Windows WinInet HTTPS on desktop (zero extra runtime dependencies).
//! Secrets (`sync_token`) are never included inside the uploaded sync bundle.

use serde::{Deserialize, Serialize};
use tao::event_loop::EventLoopProxy;

use crate::config::{AppConfig, BookmarkItem, SiteRule, ThemePreset};

use super::events::UserEvent;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct CloudSyncBundle {
    pub fynexo_sync_version: u32,
    pub updated_at: u64,
    pub bookmarks: Vec<BookmarkItem>,
    pub site_rules: Vec<SiteRule>,
    pub custom_themes: Vec<ThemePreset>,
    pub theme: String,
    pub language: String,
    pub search_engine: String,
    pub homepage: String,
    pub adblock: bool,
    pub force_dark: bool,
    pub https_first: bool,
}

impl Default for CloudSyncBundle {
    fn default() -> Self {
        Self {
            fynexo_sync_version: 1,
            updated_at: 0,
            bookmarks: Vec::new(),
            site_rules: Vec::new(),
            custom_themes: Vec::new(),
            theme: "dark".to_string(),
            language: "zh".to_string(),
            search_engine: "google".to_string(),
            homepage: "fynexo://newtab".to_string(),
            adblock: true,
            force_dark: false,
            https_first: true,
        }
    }
}

impl CloudSyncBundle {
    pub fn build(
        config: &AppConfig,
        bookmarks: &[BookmarkItem],
        site_rules: &[SiteRule],
        themes: &[ThemePreset],
    ) -> Self {
        Self {
            fynexo_sync_version: 1,
            updated_at: crate::config::now_unix_secs(),
            bookmarks: bookmarks.to_vec(),
            site_rules: site_rules.to_vec(),
            custom_themes: themes.iter().filter(|t| !t.builtin).cloned().collect(),
            theme: config.theme.clone(),
            language: config.language.clone(),
            search_engine: config.search_engine.clone(),
            homepage: config.homepage.clone(),
            adblock: config.adblock,
            force_dark: config.force_dark,
            https_first: config.https_first,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SyncTargetConfig {
    pub provider: String, // "webdav" | "gist"
    pub url: String,
    pub username: String,
    pub token: String,
}

pub fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn parse_https_url(raw: &str) -> Result<(String, u16, String), String> {
    let trimmed = raw.trim();
    let rest = trimmed
        .strip_prefix("https://")
        .ok_or_else(|| "Cloud Sync requires an https:// URL".to_string())?;
    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => {
            let pn = p.parse::<u16>().map_err(|_| "Invalid port".to_string())?;
            (h.to_string(), pn)
        }
        _ => (authority.to_string(), 443u16),
    };
    if host.is_empty() || host.contains('@') {
        return Err("Invalid host in sync URL".to_string());
    }
    Ok((host, port, path.to_string()))
}

#[cfg(target_os = "windows")]
fn wininet_https_request(
    method: &str,
    url: &str,
    headers: &str,
    body: Option<&[u8]>,
) -> Result<(u32, Vec<u8>), String> {
    use std::ffi::c_void;
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpOpenRequestW, HttpQueryInfoW, HttpSendRequestW, InternetCloseHandle, InternetConnectW,
        InternetOpenW, InternetReadFile, InternetSetOptionW, HTTP_QUERY_FLAG_NUMBER,
        HTTP_QUERY_STATUS_CODE, INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_NO_COOKIES,
        INTERNET_FLAG_RELOAD, INTERNET_FLAG_SECURE, INTERNET_OPEN_TYPE_PRECONFIG,
        INTERNET_OPTION_CONNECT_TIMEOUT, INTERNET_OPTION_RECEIVE_TIMEOUT,
        INTERNET_OPTION_SEND_TIMEOUT, INTERNET_SERVICE_HTTP,
    };

    let (host, port, path) = parse_https_url(url)?;

    unsafe {
        let h_net = InternetOpenW(
            w!("FynexoSync/1.0"),
            INTERNET_OPEN_TYPE_PRECONFIG.0,
            None,
            None,
            0,
        );
        if h_net.is_null() {
            return Err("WinInet init failed".to_string());
        }
        let timeout_ms: u32 = 12_000;
        let opt_ptr = (&timeout_ms as *const u32).cast::<c_void>();
        let opt_len = std::mem::size_of::<u32>() as u32;
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_CONNECT_TIMEOUT, Some(opt_ptr), opt_len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_RECEIVE_TIMEOUT, Some(opt_ptr), opt_len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_SEND_TIMEOUT, Some(opt_ptr), opt_len);

        let host_hs = HSTRING::from(host.as_str());
        let h_conn = InternetConnectW(
            h_net,
            &host_hs,
            port,
            None,
            None,
            INTERNET_SERVICE_HTTP,
            0,
            Some(0),
        );
        if h_conn.is_null() {
            let _ = InternetCloseHandle(h_net);
            return Err("Unable to connect to sync host".to_string());
        }

        let verb_hs = HSTRING::from(method);
        let path_hs = HSTRING::from(path.as_str());
        let h_req = HttpOpenRequestW(
            h_conn,
            &verb_hs,
            &path_hs,
            None,
            None,
            None,
            INTERNET_FLAG_SECURE
                | INTERNET_FLAG_RELOAD
                | INTERNET_FLAG_NO_CACHE_WRITE
                | INTERNET_FLAG_NO_COOKIES,
            Some(0),
        );
        if h_req.is_null() {
            let _ = InternetCloseHandle(h_conn);
            let _ = InternetCloseHandle(h_net);
            return Err("Failed to open HTTPS request".to_string());
        }

        let hdr_u16: Vec<u16> = headers.encode_utf16().collect();
        let hdr_slice = if hdr_u16.is_empty() {
            None
        } else {
            Some(hdr_u16.as_slice())
        };
        let (body_ptr, body_len) = match body {
            Some(b) if !b.is_empty() => (Some(b.as_ptr().cast::<c_void>()), b.len() as u32),
            _ => (None, 0),
        };
        if HttpSendRequestW(h_req, hdr_slice, body_ptr, body_len).is_err() {
            let _ = InternetCloseHandle(h_req);
            let _ = InternetCloseHandle(h_conn);
            let _ = InternetCloseHandle(h_net);
            return Err("HTTPS request failed".to_string());
        }

        let mut status: u32 = 0;
        let mut status_len = std::mem::size_of::<u32>() as u32;
        let _ = HttpQueryInfoW(
            h_req,
            HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
            Some((&mut status as *mut u32).cast::<c_void>()),
            &mut status_len,
            None,
        );

        let mut out = Vec::with_capacity(8192);
        let mut buf = [0u8; 8192];
        loop {
            let mut read: u32 = 0;
            if InternetReadFile(
                h_req,
                buf.as_mut_ptr().cast::<c_void>(),
                buf.len() as u32,
                &mut read,
            )
            .is_err()
            {
                break;
            }
            if read == 0 {
                break;
            }
            if out.len().saturating_add(read as usize) > 4 * 1024 * 1024 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
        }

        let _ = InternetCloseHandle(h_req);
        let _ = InternetCloseHandle(h_conn);
        let _ = InternetCloseHandle(h_net);
        Ok((status, out))
    }
}

#[cfg(target_os = "windows")]
fn perform_sync_push(cfg: &SyncTargetConfig, bundle: &CloudSyncBundle) -> Result<String, String> {
    let json = serde_json::to_string_pretty(bundle).map_err(|e| e.to_string())?;
    match cfg.provider.as_str() {
        "gist" => {
            let gist_id = cfg.url.trim().trim_matches('/');
            let gist_id = gist_id.rsplit('/').next().unwrap_or(gist_id);
            if gist_id.is_empty() || cfg.token.trim().is_empty() {
                return Err("Gist ID and GitHub Token are required".to_string());
            }
            let api_url = format!("https://api.github.com/gists/{}", gist_id);
            let payload = serde_json::json!({
                "files": {
                    "fynexo_sync.json": {
                        "content": json
                    }
                }
            });
            let body_bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
            let headers = format!(
                "Authorization: Bearer {}\r\nAccept: application/vnd.github+json\r\nContent-Type: application/json; charset=utf-8\r\n",
                cfg.token.trim()
            );
            let (status, _) = wininet_https_request("PATCH", &api_url, &headers, Some(&body_bytes))?;
            if (200..300).contains(&status) {
                Ok("GitHub Gist synced".to_string())
            } else {
                Err(format!("GitHub Gist HTTP {}", status))
            }
        }
        _ => {
            if cfg.url.trim().is_empty() {
                return Err("WebDAV URL is required".to_string());
            }
            let mut headers = "Content-Type: application/json; charset=utf-8\r\n".to_string();
            if !cfg.username.trim().is_empty() || !cfg.token.trim().is_empty() {
                let auth = base64_encode(format!("{}:{}", cfg.username.trim(), cfg.token.trim()).as_bytes());
                headers.push_str(&format!("Authorization: Basic {}\r\n", auth));
            }
            let (status, _) = wininet_https_request("PUT", cfg.url.trim(), &headers, Some(json.as_bytes()))?;
            if (200..300).contains(&status) {
                Ok("WebDAV synced".to_string())
            } else {
                Err(format!("WebDAV HTTP {}", status))
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn perform_sync_pull(cfg: &SyncTargetConfig) -> Result<CloudSyncBundle, String> {
    match cfg.provider.as_str() {
        "gist" => {
            let gist_id = cfg.url.trim().trim_matches('/');
            let gist_id = gist_id.rsplit('/').next().unwrap_or(gist_id);
            if gist_id.is_empty() {
                return Err("Gist ID is required".to_string());
            }
            let api_url = format!("https://api.github.com/gists/{}", gist_id);
            let mut headers = "Accept: application/vnd.github+json\r\n".to_string();
            if !cfg.token.trim().is_empty() {
                headers.push_str(&format!("Authorization: Bearer {}\r\n", cfg.token.trim()));
            }
            let (status, body) = wininet_https_request("GET", &api_url, &headers, None)?;
            if !(200..300).contains(&status) {
                return Err(format!("GitHub Gist HTTP {}", status));
            }
            let root: serde_json::Value =
                serde_json::from_slice(&body).map_err(|e| format!("Invalid Gist JSON: {}", e))?;
            let content = root["files"]["fynexo_sync.json"]["content"]
                .as_str()
                .ok_or_else(|| "fynexo_sync.json not found in Gist".to_string())?;
            serde_json::from_str::<CloudSyncBundle>(content)
                .map_err(|e| format!("Invalid sync bundle: {}", e))
        }
        _ => {
            if cfg.url.trim().is_empty() {
                return Err("WebDAV URL is required".to_string());
            }
            let mut headers = String::new();
            if !cfg.username.trim().is_empty() || !cfg.token.trim().is_empty() {
                let auth = base64_encode(format!("{}:{}", cfg.username.trim(), cfg.token.trim()).as_bytes());
                headers.push_str(&format!("Authorization: Basic {}\r\n", auth));
            }
            let (status, body) = wininet_https_request("GET", cfg.url.trim(), &headers, None)?;
            if !(200..300).contains(&status) {
                return Err(format!("WebDAV HTTP {}", status));
            }
            serde_json::from_slice::<CloudSyncBundle>(&body)
                .map_err(|e| format!("Invalid sync bundle: {}", e))
        }
    }
}

pub fn spawn_cloud_sync_push(
    cfg: SyncTargetConfig,
    bundle: CloudSyncBundle,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || {
            let res = perform_sync_push(&cfg, &bundle);
            match res {
                Ok(_) => {
                    let _ = proxy.send_event(UserEvent::CloudSyncResult {
                        action: "push".to_string(),
                        ok: true,
                        message: if zh {
                            "云端同步上传成功".to_string()
                        } else {
                            "Cloud sync upload complete".to_string()
                        },
                        pulled_bundle: None,
                    });
                }
                Err(e) => {
                    let _ = proxy.send_event(UserEvent::CloudSyncResult {
                        action: "push".to_string(),
                        ok: false,
                        message: if zh {
                            format!("云端上传失败：{}", e)
                        } else {
                            format!("Cloud upload failed: {}", e)
                        },
                        pulled_bundle: None,
                    });
                }
            }
        });
    }
    #[cfg(target_os = "android")]
    {
        let json = match serde_json::to_string_pretty(&bundle) {
            Ok(j) => j,
            Err(e) => {
                let _ = proxy.send_event(UserEvent::CloudSyncResult {
                    action: "push".to_string(),
                    ok: false,
                    message: format!("Serialize error: {}", e),
                    pulled_bundle: None,
                });
                return;
            }
        };
        let (method, url, headers, body) = match cfg.provider.as_str() {
            "gist" => {
                let gist_id = cfg.url.trim().trim_matches('/');
                let gist_id = gist_id.rsplit('/').next().unwrap_or(gist_id);
                if cfg.token.trim().is_empty() {
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "请先填写 GitHub Token".to_string()
                    } else {
                        "GitHub Token is required".to_string()
                    }));
                    return;
                }
                let payload = serde_json::json!({
                    "description": "Fynexo Browser Cloud Sync",
                    "public": false,
                    "files": { "fynexo_sync.json": { "content": json } }
                });
                let hdrs = serde_json::json!({
                    "Authorization": format!("Bearer {}", cfg.token.trim()),
                    "Accept": "application/vnd.github+json",
                    "Content-Type": "application/json; charset=utf-8"
                });
                if gist_id.is_empty() {
                    ("POST", "https://api.github.com/gists".to_string(), hdrs.to_string(), Some(payload.to_string()))
                } else {
                    ("PATCH", format!("https://api.github.com/gists/{}", gist_id), hdrs.to_string(), Some(payload.to_string()))
                }
            }
            _ => {
                if !cfg.url.trim().starts_with("https://") {
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "请填写有效的 HTTPS WebDAV 地址".to_string()
                    } else {
                        "Valid HTTPS WebDAV URL is required".to_string()
                    }));
                    return;
                }
                let mut hdr_map = serde_json::Map::new();
                hdr_map.insert(
                    "Content-Type".to_string(),
                    serde_json::Value::String("application/json; charset=utf-8".to_string()),
                );
                if !cfg.username.trim().is_empty() || !cfg.token.trim().is_empty() {
                    let auth = base64_encode(
                        format!("{}:{}", cfg.username.trim(), cfg.token.trim()).as_bytes(),
                    );
                    hdr_map.insert(
                        "Authorization".to_string(),
                        serde_json::Value::String(format!("Basic {}", auth)),
                    );
                }
                (
                    "PUT",
                    cfg.url.trim().to_string(),
                    serde_json::Value::Object(hdr_map).to_string(),
                    Some(json),
                )
            }
        };
        let _ = proxy.send_event(UserEvent::AndroidCloudSyncHttp {
            action: "push".to_string(),
            provider: cfg.provider,
            method: method.to_string(),
            url,
            headers_json: headers,
            body,
        });
    }
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        let _ = (cfg, bundle);
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "云端同步暂仅支持 Windows 与 Android".to_string()
        } else {
            "Cloud sync is supported on Windows and Android".to_string()
        }));
    }
}

pub fn spawn_cloud_sync_pull(
    cfg: SyncTargetConfig,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || match perform_sync_pull(&cfg) {
            Ok(bundle) => {
                let _ = proxy.send_event(UserEvent::CloudSyncResult {
                    action: "pull".to_string(),
                    ok: true,
                    message: if zh {
                        format!("已从云端拉取 {} 条书签与配置", bundle.bookmarks.len())
                    } else {
                        format!("Pulled {} bookmarks & settings from cloud", bundle.bookmarks.len())
                    },
                    pulled_bundle: Some(Box::new(bundle)),
                });
            }
            Err(e) => {
                let _ = proxy.send_event(UserEvent::CloudSyncResult {
                    action: "pull".to_string(),
                    ok: false,
                    message: if zh {
                        format!("云端拉取失败：{}", e)
                    } else {
                        format!("Cloud pull failed: {}", e)
                    },
                    pulled_bundle: None,
                });
            }
        });
    }
    #[cfg(target_os = "android")]
    {
        let (url, headers) = match cfg.provider.as_str() {
            "gist" => {
                let gist_id = cfg.url.trim().trim_matches('/');
                let gist_id = gist_id.rsplit('/').next().unwrap_or(gist_id);
                if gist_id.is_empty() {
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "请先填写 Gist ID".to_string()
                    } else {
                        "Gist ID is required".to_string()
                    }));
                    return;
                }
                let mut hdr_map = serde_json::Map::new();
                hdr_map.insert(
                    "Accept".to_string(),
                    serde_json::Value::String("application/vnd.github+json".to_string()),
                );
                if !cfg.token.trim().is_empty() {
                    hdr_map.insert(
                        "Authorization".to_string(),
                        serde_json::Value::String(format!("Bearer {}", cfg.token.trim())),
                    );
                }
                (
                    format!("https://api.github.com/gists/{}", gist_id),
                    serde_json::Value::Object(hdr_map).to_string(),
                )
            }
            _ => {
                if !cfg.url.trim().starts_with("https://") {
                    let _ = proxy.send_event(UserEvent::Toast(if zh {
                        "请填写有效的 HTTPS WebDAV 地址".to_string()
                    } else {
                        "Valid HTTPS WebDAV URL is required".to_string()
                    }));
                    return;
                }
                let mut hdr_map = serde_json::Map::new();
                if !cfg.username.trim().is_empty() || !cfg.token.trim().is_empty() {
                    let auth = base64_encode(
                        format!("{}:{}", cfg.username.trim(), cfg.token.trim()).as_bytes(),
                    );
                    hdr_map.insert(
                        "Authorization".to_string(),
                        serde_json::Value::String(format!("Basic {}", auth)),
                    );
                }
                (
                    cfg.url.trim().to_string(),
                    serde_json::Value::Object(hdr_map).to_string(),
                )
            }
        };
        let _ = proxy.send_event(UserEvent::AndroidCloudSyncHttp {
            action: "pull".to_string(),
            provider: cfg.provider,
            method: "GET".to_string(),
            url,
            headers_json: headers,
            body: None,
        });
    }
    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        let _ = cfg;
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "云端同步暂仅支持 Windows 与 Android".to_string()
        } else {
            "Cloud sync is supported on Windows and Android".to_string()
        }));
    }
}

/// Parse an Android HTTP sync callback body into a `CloudSyncResult` event.
pub fn parse_pulled_bundle_response(provider: &str, body: &str) -> Result<CloudSyncBundle, String> {
    if provider == "gist" {
        let root: serde_json::Value =
            serde_json::from_str(body).map_err(|e| format!("Invalid Gist JSON: {}", e))?;
        let content = root["files"]["fynexo_sync.json"]["content"]
            .as_str()
            .ok_or_else(|| "fynexo_sync.json not found in Gist".to_string())?;
        serde_json::from_str::<CloudSyncBundle>(content)
            .map_err(|e| format!("Invalid sync bundle: {}", e))
    } else {
        serde_json::from_str::<CloudSyncBundle>(body)
            .map_err(|e| format!("Invalid sync bundle: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_matches_rfc_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"user:pass"), "dXNlcjpwYXNz");
    }

    #[test]
    fn sync_bundle_roundtrip_preserves_bookmarks_and_excludes_secrets() {
        let cfg = AppConfig::default();
        let b = CloudSyncBundle::build(&cfg, &[], &[], &[]);
        let json = serde_json::to_string(&b).unwrap();
        assert!(!json.contains("sync_token"));
        let parsed: CloudSyncBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.fynexo_sync_version, 1);
    }
}
