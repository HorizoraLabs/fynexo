//! Live search-engine autocomplete suggestions (CORS-free, privacy-gated).
//!
//! Fetches lightweight JSON suggestion arrays from the active search engine
//! (Google, Bing, DuckDuckGo, or Baidu) on a detached background worker so
//! neither `fynexo://newtab` nor the HUD Omnibox hits browser CORS limits.
//! Never sends requests when Incognito is active or when the input is a URL,
//! command (`>`), bang (`!`), or math expression.

use std::sync::atomic::{AtomicU64, Ordering};
use tao::event_loop::EventLoopProxy;

use super::events::UserEvent;

static LATEST_SUGGEST_REQ: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "windows")]
const MAX_SUGGEST_BODY_BYTES: usize = 64 * 1024;
const MAX_SUGGEST_RESULTS: usize = 6;

/// Returns `true` when `raw_query` is a normal search query eligible for live
/// online autocomplete (not empty, not a URL/domain, not a command/bang/math).
pub fn should_fetch_online_suggestions(raw_query: &str) -> bool {
    let q = raw_query.trim();
    if q.is_empty() || q.chars().count() > 120 {
        return false;
    }
    if q.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return false;
    }
    if q.starts_with('>') || q.starts_with('!') || q.starts_with('/') {
        return false;
    }
    let lower = q.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("fynexo://")
        || lower.starts_with("about:")
        || lower.starts_with("file://")
        || lower.starts_with("data:")
        || lower.starts_with("localhost")
    {
        return false;
    }
    // Skip obvious bare domains/URLs without spaces (e.g. "github.com/foo", "example.org").
    if !q.contains(char::is_whitespace) && q.contains('.') {
        let host_part = lower.split('/').next().unwrap_or("");
        if let Some((_, tld)) = host_part.rsplit_once('.') {
            if tld.len() >= 2 && tld.len() <= 12 && tld.chars().all(|c| c.is_ascii_alphabetic()) {
                return false;
            }
        }
    }
    // Skip pure math expressions like "12*34" or "(5+3)/2".
    if q.len() >= 3
        && q.bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')' | b'.' | b' ' | b'^'))
        && q.bytes().any(|b| matches!(b, b'+' | b'-' | b'*' | b'/' | b'%' | b'^'))
    {
        return false;
    }
    true
}

/// RFC 3986 UTF-8 percent-encoding for query parameter values.
pub fn percent_encode_query(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for b in input.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(*b, b'-' | b'_' | b'.' | b'~') {
            out.push(*b as char);
        } else {
            out.push('%');
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0f) as usize] as char);
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SuggestProvider {
    Google,
    Bing,
    DuckDuckGo,
    Baidu,
}

fn resolve_provider(engine_id: &str, engine_url_hint: Option<&str>, zh: bool) -> SuggestProvider {
    let id = engine_id.trim().to_ascii_lowercase();
    let hint = engine_url_hint.unwrap_or("").trim().to_ascii_lowercase();
    if id.contains("baidu") || hint.contains("baidu.com") {
        SuggestProvider::Baidu
    } else if id.contains("bing") || hint.contains("bing.com") {
        SuggestProvider::Bing
    } else if id.contains("duck") || id == "ddg" || hint.contains("duckduckgo.com") {
        SuggestProvider::DuckDuckGo
    } else if id.contains("google") || hint.contains("google.") {
        SuggestProvider::Google
    } else if zh {
        SuggestProvider::Bing
    } else {
        SuggestProvider::Google
    }
}

/// Build an allowlisted HTTPS suggestion endpoint URL for the active search engine.
pub fn build_suggest_url(
    engine_id: &str,
    engine_url_hint: Option<&str>,
    query: &str,
    zh: bool,
) -> String {
    let encoded = percent_encode_query(query.trim());
    match resolve_provider(engine_id, engine_url_hint, zh) {
        SuggestProvider::Google => {
            let hl = if zh { "zh-CN" } else { "en" };
            format!(
                "https://suggestqueries.google.com/complete/search?client=firefox&hl={}&q={}",
                hl, encoded
            )
        }
        SuggestProvider::Bing => {
            let mkt = if zh { "zh-CN" } else { "en-US" };
            format!("https://api.bing.com/osjson.aspx?mkt={}&query={}", mkt, encoded)
        }
        SuggestProvider::DuckDuckGo => {
            format!("https://duckduckgo.com/ac/?q={}&type=list", encoded)
        }
        SuggestProvider::Baidu => {
            format!("https://www.baidu.com/sugrec?prod=pc&wd={}", encoded)
        }
    }
}

fn sanitize_suggestion(raw: &str) -> Option<String> {
    let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 140 {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse suggestion JSON returned by Google/Bing/DDG OpenSearch (`[q, [s1, s2, ...]]`),
/// Baidu `sugrec` (`{"g":[{"q":"..."}]}`), or DDG phrase objects (`[{"phrase":"..."}]`).
pub fn parse_suggestions_response(body: &[u8], orig_query: &str) -> Vec<String> {
    let text = match std::str::from_utf8(body) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let val: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut raw_list: Vec<&str> = Vec::new();
    if let Some(arr) = val.as_array() {
        // OpenSearch format: [ "query", ["sug1", "sug2", ...] ]
        if arr.len() >= 2 {
            if let Some(sugs) = arr[1].as_array() {
                for item in sugs {
                    if let Some(s) = item.as_str() {
                        raw_list.push(s);
                    }
                }
            }
        }
        // Fallback DDG object array: [ { "phrase": "sug1" }, ... ]
        if raw_list.is_empty() {
            for item in arr {
                if let Some(s) = item.get("phrase").and_then(|v| v.as_str()) {
                    raw_list.push(s);
                }
            }
        }
    } else if let Some(g_arr) = val.get("g").and_then(|v| v.as_array()) {
        // Baidu sugrec: { "q": "...", "g": [ { "q": "sug1" }, ... ] }
        for item in g_arr {
            if let Some(s) = item.get("q").and_then(|v| v.as_str()) {
                raw_list.push(s);
            }
        }
    }

    let orig_lower = orig_query.trim().to_ascii_lowercase();
    let mut out: Vec<String> = Vec::with_capacity(MAX_SUGGEST_RESULTS);
    for candidate in raw_list {
        let Some(clean) = sanitize_suggestion(candidate) else {
            continue;
        };
        let lower = clean.to_ascii_lowercase();
        if lower == orig_lower || out.iter().any(|existing| existing.to_ascii_lowercase() == lower) {
            continue;
        }
        out.push(clean);
        if out.len() >= MAX_SUGGEST_RESULTS {
            break;
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn wininet_fetch_suggest_bytes(url: &str) -> Result<Vec<u8>, ()> {
    use core::ffi::c_void;
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
        InternetSetOptionW, HTTP_QUERY_FLAG_NUMBER, HTTP_QUERY_STATUS_CODE,
        INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_NO_COOKIES, INTERNET_FLAG_RELOAD,
        INTERNET_FLAG_SECURE, INTERNET_OPEN_TYPE_PRECONFIG, INTERNET_OPTION_CONNECT_TIMEOUT,
        INTERNET_OPTION_RECEIVE_TIMEOUT, INTERNET_OPTION_SEND_TIMEOUT,
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
            return Err(());
        }

        let timeout_ms: u32 = 2200;
        let opt_ptr = (&timeout_ms as *const u32).cast::<c_void>();
        let opt_len = std::mem::size_of::<u32>() as u32;
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_CONNECT_TIMEOUT, Some(opt_ptr), opt_len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_RECEIVE_TIMEOUT, Some(opt_ptr), opt_len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_SEND_TIMEOUT, Some(opt_ptr), opt_len);

        let url_hs = HSTRING::from(url);
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
            return Err(());
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
            return Err(());
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
                return Err(());
            }
            if read == 0 {
                break;
            }
            if out.len().saturating_add(read as usize) > MAX_SUGGEST_BODY_BYTES {
                let _ = InternetCloseHandle(h_url);
                let _ = InternetCloseHandle(h_net);
                return Err(());
            }
            out.extend_from_slice(&buf[..read as usize]);
        }

        let _ = InternetCloseHandle(h_url);
        let _ = InternetCloseHandle(h_net);
        Ok(out)
    }
}

/// Dispatch an asynchronous search suggestion lookup for the active engine.
pub fn spawn_fetch_suggestions(
    win_id: usize,
    req_id: u64,
    query: String,
    engine_id: String,
    engine_url_hint: Option<String>,
    zh: bool,
    proxy: EventLoopProxy<UserEvent>,
) {
    if !should_fetch_online_suggestions(&query) {
        return;
    }
    let url = build_suggest_url(&engine_id, engine_url_hint.as_deref(), &query, zh);
    let ticket = LATEST_SUGGEST_REQ.fetch_add(1, Ordering::SeqCst) + 1;

    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || {
            if LATEST_SUGGEST_REQ.load(Ordering::SeqCst) != ticket {
                return;
            }
            let Ok(bytes) = wininet_fetch_suggest_bytes(&url) else {
                return;
            };
            if LATEST_SUGGEST_REQ.load(Ordering::SeqCst) != ticket {
                return;
            }
            let suggestions = parse_suggestions_response(&bytes, &query);
            let _ = proxy.send_event(UserEvent::SearchSuggestions {
                win_id,
                req_id,
                query,
                engine: engine_id,
                suggestions,
            });
        });
    }

    #[cfg(target_os = "android")]
    {
        let _ = ticket;
        let _ = proxy.send_event(UserEvent::AndroidFetchSuggestions {
            win_id,
            req_id,
            url,
            engine: engine_id,
            query,
        });
    }

    #[cfg(not(any(target_os = "windows", target_os = "android")))]
    {
        let _ = (win_id, req_id, url, ticket, proxy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_fetch_filters_urls_commands_bangs_and_math() {
        assert!(should_fetch_online_suggestions("azur promilia"));
        assert!(should_fetch_online_suggestions("rust async"));
        assert!(should_fetch_online_suggestions("天气预报"));
        assert!(!should_fetch_online_suggestions(""));
        assert!(!should_fetch_online_suggestions("   "));
        assert!(!should_fetch_online_suggestions(">settings"));
        assert!(!should_fetch_online_suggestions("!gh rust"));
        assert!(!should_fetch_online_suggestions("https://github.com"));
        assert!(!should_fetch_online_suggestions("fynexo://settings"));
        assert!(!should_fetch_online_suggestions("github.com/rust-lang"));
        assert!(!should_fetch_online_suggestions("12 * 34 + 5"));
    }

    #[test]
    fn build_suggest_url_targets_expected_engine_endpoints() {
        let g = build_suggest_url("google", None, "rust lang", true);
        assert!(g.starts_with("https://suggestqueries.google.com/complete/search?client=firefox&hl=zh-CN&q=rust%20lang"));
        let b = build_suggest_url("bing", None, "hello", false);
        assert!(b.starts_with("https://api.bing.com/osjson.aspx?mkt=en-US&query=hello"));
        let d = build_suggest_url("duckduckgo", None, "privacy", false);
        assert_eq!(d, "https://duckduckgo.com/ac/?q=privacy&type=list");
        let bd = build_suggest_url("baidu", None, "中文", true);
        assert!(bd.starts_with("https://www.baidu.com/sugrec?prod=pc&wd=%E4%B8%AD%E6%96%87"));
    }

    #[test]
    fn parse_suggestions_handles_opensearch_baidu_and_ddg_formats() {
        let google_json = br#"["azur",["azur","azur promilia","azur lane","Azur Promilia","azur promilia taiwan"]]"#;
        let g = parse_suggestions_response(google_json, "azur");
        assert_eq!(g, vec!["azur promilia", "azur lane", "azur promilia taiwan"]);

        let baidu_json = r#"{"q":"rust","g":[{"q":"rust语言"},{"q":"rust下载"},{"q":"rust"}]}"#.as_bytes();
        let bd = parse_suggestions_response(baidu_json, "rust");
        assert_eq!(bd, vec!["rust语言", "rust下载"]);

        let ddg_json = br#"[{"phrase":"fynexo browser"},{"phrase":"fynexo github"}]"#;
        let ddg = parse_suggestions_response(ddg_json, "fynexo");
        assert_eq!(ddg, vec!["fynexo browser", "fynexo github"]);
    }
}
