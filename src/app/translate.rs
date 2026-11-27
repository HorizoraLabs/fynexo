//! CORS-free in-place page translation backend.
//!
//! Accepts batches of short text segments from the active webpage, translates them
//! asynchronously on a background thread via Google Translate's lightweight JSON endpoint
//! (with in-memory caching & safe chunking), and returns the translated strings to the page
//! for bilingual or in-place replacement without ever leaving or reloading the page.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tao::event_loop::EventLoopProxy;

use super::events::UserEvent;
#[cfg(target_os = "windows")]
use super::suggest::percent_encode_query;

const MAX_CACHE_ENTRIES: usize = 1024;
const MAX_BATCH_ITEMS: usize = 36;
const MAX_SEGMENT_CHARS: usize = 600;
#[cfg(target_os = "windows")]
const MAX_TRANSLATE_BODY_BYTES: usize = 256 * 1024;

static TRANSLATE_CACHE: LazyLock<Mutex<HashMap<(String, String), String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Normalize user-requested translation target language to an allowlisted ISO code.
pub fn normalize_target_lang(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "zh" | "zh-cn" | "zh-hans" => "zh-CN",
        "zh-tw" | "zh-hant" => "zh-TW",
        "en" | "en-us" | "en-gb" => "en",
        "ja" | "jp" => "ja",
        "ko" | "kr" => "ko",
        "fr" => "fr",
        "de" => "de",
        "es" => "es",
        "ru" => "ru",
        "pt" => "pt",
        _ => "zh-CN",
    }
}

/// Parse Google Translate `translate_a/single` JSON payload (`[[["translated","orig",...],...],...]`).
pub fn parse_google_translate_json(bytes: &[u8]) -> Option<String> {
    let val: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    let sentences = val.get(0)?.as_array()?;
    let mut out = String::new();
    for item in sentences {
        if let Some(part) = item.get(0).and_then(|v| v.as_str()) {
            out.push_str(part);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[cfg(target_os = "windows")]
fn wininet_translate_text(target_lang: &str, text: &str) -> Result<String, String> {
    use std::ffi::c_void;
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
        InternetSetOptionW, HTTP_QUERY_FLAG_NUMBER, HTTP_QUERY_STATUS_CODE,
        INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_NO_COOKIES, INTERNET_FLAG_RELOAD,
        INTERNET_FLAG_SECURE, INTERNET_OPEN_TYPE_PRECONFIG, INTERNET_OPTION_CONNECT_TIMEOUT,
        INTERNET_OPTION_RECEIVE_TIMEOUT, INTERNET_OPTION_SEND_TIMEOUT,
    };

    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        target_lang,
        percent_encode_query(text)
    );

    unsafe {
        let h_net = InternetOpenW(
            w!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"),
            INTERNET_OPEN_TYPE_PRECONFIG.0,
            None,
            None,
            0,
        );
        if h_net.is_null() {
            return Err("WinInet initialization failed".to_string());
        }

        let timeout_ms: u32 = 8000;
        let opt_ptr = (&timeout_ms as *const u32).cast::<c_void>();
        let opt_len = std::mem::size_of::<u32>() as u32;
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_CONNECT_TIMEOUT, Some(opt_ptr), opt_len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_RECEIVE_TIMEOUT, Some(opt_ptr), opt_len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_SEND_TIMEOUT, Some(opt_ptr), opt_len);

        let url_hs = HSTRING::from(url.as_str());
        let h_url = InternetOpenUrlW(
            h_net,
            &url_hs,
            None,
            INTERNET_FLAG_RELOAD
                | INTERNET_FLAG_SECURE
                | INTERNET_FLAG_NO_COOKIES
                | INTERNET_FLAG_NO_CACHE_WRITE,
            None,
        );
        if h_url.is_null() {
            let _ = InternetCloseHandle(h_net);
            return Err("Translation endpoint unreachable".to_string());
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
            return Err(format!("HTTP {}", status_code));
        }

        let mut out = Vec::with_capacity(4096);
        let mut buf = [0u8; 4096];
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
                return Err("Network read error".to_string());
            }
            if read == 0 {
                break;
            }
            if out.len().saturating_add(read as usize) > MAX_TRANSLATE_BODY_BYTES {
                let _ = InternetCloseHandle(h_url);
                let _ = InternetCloseHandle(h_net);
                return Err("Response too large".to_string());
            }
            out.extend_from_slice(&buf[..read as usize]);
        }

        let _ = InternetCloseHandle(h_url);
        let _ = InternetCloseHandle(h_net);
        parse_google_translate_json(&out).ok_or_else(|| "Invalid translation response".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn wininet_translate_text(_target_lang: &str, _text: &str) -> Result<String, String> {
    Err("Native in-place translation is currently available on Windows desktop".to_string())
}

fn cache_get(lang: &str, text: &str) -> Option<String> {
    TRANSLATE_CACHE
        .lock()
        .ok()
        .and_then(|g| g.get(&(lang.to_string(), text.to_string())).cloned())
}

fn cache_put(lang: &str, orig: &str, translated: &str) {
    if let Ok(mut g) = TRANSLATE_CACHE.lock() {
        if g.len() >= MAX_CACHE_ENTRIES {
            g.clear();
        }
        g.insert((lang.to_string(), orig.to_string()), translated.to_string());
    }
}

/// Spawn a background worker to translate a batch of `(id, text)` segments and emit
/// `UserEvent::TranslateResult` back to the requesting tab.
pub fn spawn_translate_batch(
    win_id: usize,
    tab_id: usize,
    req_id: u64,
    raw_target_lang: &str,
    raw_items: Vec<(u32, String)>,
    proxy: EventLoopProxy<UserEvent>,
) {
    let target_lang = normalize_target_lang(raw_target_lang).to_string();
    let cleaned: Vec<(u32, String)> = raw_items
        .into_iter()
        .take(MAX_BATCH_ITEMS)
        .filter_map(|(id, text)| {
            let trimmed: String = text
                .replace(['\r', '\n'], " ")
                .trim()
                .chars()
                .take(MAX_SEGMENT_CHARS)
                .collect();
            if trimmed.is_empty() {
                None
            } else {
                Some((id, trimmed))
            }
        })
        .collect();

    if cleaned.is_empty() {
        return;
    }

    std::thread::spawn(move || {
        let mut results: Vec<(u32, String)> = Vec::with_capacity(cleaned.len());
        let mut uncached: Vec<(u32, String)> = Vec::new();

        for (id, txt) in cleaned {
            if let Some(hit) = cache_get(&target_lang, &txt) {
                results.push((id, hit));
            } else {
                uncached.push((id, txt));
            }
        }

        let mut last_err: Option<String> = None;

        // Pack uncached segments into small newline-separated chunks (~1200 encoded bytes each)
        // so we minimize round-trips while keeping 1:1 line alignment.
        let mut idx = 0usize;
        while idx < uncached.len() {
            let mut chunk: Vec<(u32, String)> = Vec::new();
            let mut est_len = 0usize;
            while idx < uncached.len() {
                let seg_len = uncached[idx].1.len() + 4;
                if !chunk.is_empty() && (est_len + seg_len > 900 || chunk.len() >= 10) {
                    break;
                }
                est_len += seg_len;
                chunk.push(uncached[idx].clone());
                idx += 1;
            }

            let joined = chunk
                .iter()
                .map(|(_, s)| s.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            match wininet_translate_text(&target_lang, &joined) {
                Ok(tr_all) => {
                    let lines: Vec<&str> = tr_all.lines().collect();
                    if lines.len() == chunk.len() {
                        for ((id, orig), tr_line) in chunk.iter().zip(lines) {
                            let clean = tr_line.trim().to_string();
                            if !clean.is_empty() {
                                cache_put(&target_lang, orig, &clean);
                                results.push((*id, clean));
                            }
                        }
                    } else {
                        // Fallback: translate items individually if the service merged/split lines.
                        for (id, orig) in chunk {
                            match wininet_translate_text(&target_lang, &orig) {
                                Ok(single) => {
                                    let clean = single.trim().to_string();
                                    if !clean.is_empty() {
                                        cache_put(&target_lang, &orig, &clean);
                                        results.push((id, clean));
                                    }
                                }
                                Err(e) => {
                                    last_err = Some(e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    last_err = Some(e);
                    break;
                }
            }
        }

        let _ = proxy.send_event(UserEvent::TranslateResult {
            win_id,
            tab_id,
            req_id,
            target_lang,
            items: results,
            error: last_err,
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_target_lang_handles_aliases() {
        assert_eq!(normalize_target_lang("zh"), "zh-CN");
        assert_eq!(normalize_target_lang("zh-TW"), "zh-TW");
        assert_eq!(normalize_target_lang("EN"), "en");
        assert_eq!(normalize_target_lang("jp"), "ja");
        assert_eq!(normalize_target_lang("unknown"), "zh-CN");
    }

    #[test]
    fn parse_google_translate_json_joins_segments() {
        let sample = r#"[[["你好，","Hello, ",null,null,10],["世界","world",null,null,10]],null,"en"]"#.as_bytes();
        assert_eq!(
            parse_google_translate_json(sample),
            Some("你好，世界".to_string())
        );
    }
}
