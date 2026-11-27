#[inline]
pub fn to_webview_url(url: &str) -> String {
    let trimmed = url.trim();
    let (base, frag) = match trimmed.find('#') {
        Some(idx) => (&trimmed[..idx], &trimmed[idx..]),
        None => (trimmed, ""),
    };
    let internal_route = if base.eq_ignore_ascii_case("fynexo://newtab")
        || base.starts_with("fynexo://newtab")
        || base.eq_ignore_ascii_case("http://fynexo.localhost/newtab")
    {
        Some("newtab")
    } else if base.eq_ignore_ascii_case("fynexo://settings")
        || base.starts_with("fynexo://settings")
        || base.eq_ignore_ascii_case("http://fynexo.localhost/settings")
    {
        Some("settings")
    } else if base.eq_ignore_ascii_case("fynexo://history")
        || base.starts_with("fynexo://history")
        || base.eq_ignore_ascii_case("http://fynexo.localhost/history")
    {
        Some("history")
    } else if base.eq_ignore_ascii_case("fynexo://bookmarks")
        || base.starts_with("fynexo://bookmarks")
        || base.eq_ignore_ascii_case("http://fynexo.localhost/bookmarks")
    {
        Some("bookmarks")
    } else if base.eq_ignore_ascii_case("fynexo://downloads")
        || base.starts_with("fynexo://downloads")
        || base.eq_ignore_ascii_case("http://fynexo.localhost/downloads")
    {
        Some("downloads")
    } else {
        None
    };

    if let Some(route) = internal_route {
        #[cfg(any(target_os = "windows", target_os = "android"))]
        {
            return format!("http://fynexo.localhost/{}{}", route, frag);
        }
        #[cfg(not(any(target_os = "windows", target_os = "android")))]
        {
            return format!("fynexo://localhost/{}{}", route, frag);
        }
    }
    trimmed.to_string()
}

#[inline]
pub fn normalize_logical_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("fynexo://settings")
        || trimmed.contains("fynexo.localhost/settings")
        || trimmed.contains("fynexo://localhost/settings")
        || trimmed.contains("fynexo.settings")
    {
        "fynexo://settings".to_string()
    } else if trimmed.starts_with("fynexo://history")
        || trimmed.contains("fynexo.localhost/history")
        || trimmed.contains("fynexo://localhost/history")
        || trimmed.contains("fynexo.history")
    {
        "fynexo://history".to_string()
    } else if trimmed.starts_with("fynexo://bookmarks")
        || trimmed.contains("fynexo.localhost/bookmarks")
        || trimmed.contains("fynexo://localhost/bookmarks")
        || trimmed.contains("fynexo.bookmarks")
    {
        "fynexo://bookmarks".to_string()
    } else if trimmed.starts_with("fynexo://downloads")
        || trimmed.contains("fynexo.localhost/downloads")
        || trimmed.contains("fynexo://localhost/downloads")
        || trimmed.contains("fynexo.downloads")
    {
        "fynexo://downloads".to_string()
    } else if trimmed.starts_with("fynexo://newtab")
        || trimmed.contains("fynexo.localhost/newtab")
        || trimmed.contains("fynexo://localhost/newtab")
        || trimmed.contains("fynexo.newtab")
        || trimmed.trim_end_matches('/') == "http://fynexo.localhost"
        || trimmed.trim_end_matches('/') == "fynexo://localhost"
    {
        "fynexo://newtab".to_string()
    } else {
        trimmed.to_string()
    }
}

#[inline]
pub fn is_internal_url(url: &str) -> bool {
    let norm = normalize_logical_url(url);
    norm == "fynexo://newtab"
        || norm == "fynexo://settings"
        || norm == "fynexo://history"
        || norm == "fynexo://bookmarks"
        || norm == "fynexo://downloads"
}

fn percent_decode_url_param(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h1 = (bytes[i + 1] as char).to_digit(16);
            let h2 = (bytes[i + 2] as char).to_digit(16);
            if let (Some(a), Some(b)) = (h1, h2) {
                out.push(((a << 4) | b) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

struct ParsedHttpUrl<'a> {
    host: String,
    path: String,
    query: Option<&'a str>,
    frag: Option<&'a str>,
}

fn split_http_url(url: &str) -> Option<ParsedHttpUrl<'_>> {
    let trimmed = url.trim();
    let rest = if let Some(r) = trimmed.strip_prefix("https://") {
        r
    } else {
        trimmed.strip_prefix("http://")?
    };
    let (before_frag, frag) = match rest.find('#') {
        Some(idx) => (&rest[..idx], Some(&rest[idx..])),
        None => (rest, None),
    };
    let (before_q, query) = match before_frag.find('?') {
        Some(idx) => (&before_frag[..idx], Some(&before_frag[idx + 1..])),
        None => (before_frag, None),
    };
    let (authority, path) = match before_q.find('/') {
        Some(idx) => (&before_q[..idx], &before_q[idx..]),
        None => (before_q, ""),
    };
    let host_no_user = authority.rsplit('@').next().unwrap_or(authority);
    let host = host_no_user
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    Some(ParsedHttpUrl {
        host,
        path: path.to_ascii_lowercase(),
        query,
        frag,
    })
}

/// Unwrap common outbound redirect interstitials (Zhihu, CSDN, Juejin, QQ, Facebook, Reddit, Google, Steam, etc.).
pub fn unwrap_redirect_url(raw: &str) -> Option<String> {
    let parsed = split_http_url(raw)?;
    let query = parsed.query?;
    let host = parsed.host;
    let path = parsed.path;
    let param_keys: &[&str] = match host.as_str() {
        "link.zhihu.com" | "link.csdn.net" | "link.juejin.cn" => &["target"],
        "link.jianshu.com" => &["t"],
        "c.pc.qq.com" => &["pfurl", "url"],
        "weixin110.qq.com" => &["url"],
        "t.umblr.com" => &["z"],
        "l.facebook.com" | "lm.facebook.com" | "l.instagram.com" => &["u"],
        "out.reddit.com" => &["url"],
        "www.google.com" | "google.com" if path == "/url" => &["q", "url"],
        "www.youtube.com" | "youtube.com" if path == "/redirect" => &["q"],
        "steamcommunity.com" if path == "/linkfilter" || path == "/linkfilter/" => &["u", "url"],
        "afdian.com" | "afdian.net" | "gitee.com" | "sspai.com" if path == "/link" => &["target"],
        "www.oschina.net" if path == "/action/gotolink" => &["url"],
        "weibo.cn" if path == "/sinaurl" => &["u", "toasturl"],
        "mail.qq.com" if path.contains("readtemplate") => &["gourl"],
        _ => return None,
    };

    for pair in query.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if param_keys.iter().any(|pk| k.eq_ignore_ascii_case(pk)) {
            let decoded = percent_decode_url_param(v);
            let target = decoded.trim();
            if (target.starts_with("https://") || target.starts_with("http://"))
                && target.len() <= 8192
                && !target.bytes().any(|b| b < 0x20 || b == 0x7f)
            {
                return Some(target.to_string());
            }
        }
    }
    None
}

fn is_tracking_query_param(host: &str, path: &str, key_lower: &str, val: &str) -> bool {
    if key_lower.starts_with("utm_")
        || key_lower.starts_with("hsa_")
        || key_lower.starts_with("pk_")
        || key_lower.starts_with("mtm_")
    {
        return true;
    }
    if matches!(
        key_lower,
        "fbclid"
            | "gclid"
            | "gclsrc"
            | "dclid"
            | "gbraid"
            | "wbraid"
            | "msclkid"
            | "yclid"
            | "twclid"
            | "ttclid"
            | "li_fat_id"
            | "epik"
            | "sccid"
            | "rdt_cid"
            | "sclid"
            | "gad_source"
            | "gad_campaignid"
            | "mc_cid"
            | "mc_eid"
            | "_hsenc"
            | "_hsmi"
            | "hsctatracking"
            | "mkt_tok"
            | "vero_id"
            | "vero_conv"
            | "oly_enc_id"
            | "oly_anon_id"
            | "rb_clickid"
            | "wickedid"
            | "igshid"
            | "igsh"
            | "ref_src"
            | "ref_url"
            | "spm_id_from"
            | "vd_source"
            | "from_spmid"
            | "share_source"
            | "share_medium"
            | "share_plat"
            | "share_session_id"
            | "share_tag"
            | "share_times"
            | "bbid"
            | "is_story_h5"
            | "buvid"
            | "up_id"
            | "unique_k"
            | "spm"
            | "scm"
            | "pvid"
            | "algo_exp_id"
            | "algo_pvid"
            | "utparam"
            | "ali_trackid"
            | "isappinstalled"
            | "from_wecom"
            | "_ga"
            | "_gl"
            | "_ke"
            | "_bta_tid"
            | "_bta_c"
            | "trk"
            | "trkcampaign"
            | "sc_channel"
            | "sc_campaign"
            | "sc_medium"
            | "sc_content"
            | "sc_outcome"
            | "guccounter"
            | "guce_referrer"
            | "guce_referrer_sig"
            | "irclickid"
            | "irgwc"
            | "zanpid"
            | "cvid"
            | "ocid"
    ) {
        return true;
    }

    // Site-specific tracking parameters that are safe only on their respective hosts
    if (host == "youtube.com"
        || host.ends_with(".youtube.com")
        || host == "youtu.be"
        || host == "open.spotify.com")
        && matches!(key_lower, "si" | "pp")
    {
        return true;
    }
    if (host == "youtube.com" || host.ends_with(".youtube.com") || host == "youtu.be")
        && key_lower == "feature"
        && matches!(val, "shared" | "youtu.be" | "share")
    {
        return true;
    }
    if (host == "x.com"
        || host == "twitter.com"
        || host.ends_with(".x.com")
        || host.ends_with(".twitter.com"))
        && path.contains("/status/")
        && matches!(key_lower, "s" | "t")
    {
        return true;
    }
    if (host.contains(".amazon.") || host.starts_with("amazon."))
        && matches!(
            key_lower,
            "pd_rd_w"
                | "pd_rd_r"
                | "pd_rd_wg"
                | "pf_rd_p"
                | "pf_rd_r"
                | "pf_rd_s"
                | "pf_rd_t"
                | "pf_rd_i"
                | "pf_rd_m"
                | "dib"
                | "dib_tag"
                | "linkcode"
                | "ascsubtag"
                | "ref_"
        )
    {
        return true;
    }
    false
}

/// Strip tracking query parameters from an `http(s)://` URL while preserving meaningful params & fragment.
pub fn strip_tracking_params(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(parsed) = split_http_url(trimmed) else {
        return trimmed.to_string();
    };
    let Some(query) = parsed.query else {
        return trimmed.to_string();
    };
    let host = parsed.host;
    let path = parsed.path;
    let frag = parsed.frag;
    let Some(q_pos) = trimmed.find('?') else {
        return trimmed.to_string();
    };
    let base = &trimmed[..q_pos];
    let mut kept: Vec<&str> = Vec::new();
    let mut changed = false;
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let k_lower = k.to_ascii_lowercase();
        if is_tracking_query_param(&host, &path, &k_lower, v) {
            changed = true;
        } else {
            kept.push(pair);
        }
    }
    if !changed {
        return trimmed.to_string();
    }
    let mut out = String::with_capacity(trimmed.len());
    out.push_str(base);
    if !kept.is_empty() {
        out.push('?');
        out.push_str(&kept.join("&"));
    }
    if let Some(f) = frag {
        out.push_str(f);
    }
    out
}

/// Unwrap known outbound redirect pages (up to 3 hops) and strip tracking query parameters.
pub fn sanitize_and_clean_url(raw: &str, enabled: bool) -> String {
    let trimmed = raw.trim();
    if !enabled
        || (!trimmed.starts_with("http://") && !trimmed.starts_with("https://"))
    {
        return trimmed.to_string();
    }
    let mut cur = trimmed.to_string();
    for _ in 0..3 {
        if let Some(next) = unwrap_redirect_url(&cur) {
            cur = next;
        } else {
            break;
        }
    }
    strip_tracking_params(&cur)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_webview_and_back_roundtrip_for_internal_pages() {
        for logical in [
            "fynexo://newtab",
            "fynexo://settings",
            "fynexo://history",
            "fynexo://bookmarks",
        ] {
            let physical = to_webview_url(logical);
            assert_ne!(physical, logical);
            assert_eq!(normalize_logical_url(&physical), logical);
        }
        let with_hash = to_webview_url("fynexo://settings#sec-privacy");
        assert!(with_hash.ends_with("/settings#sec-privacy"));
        assert_eq!(normalize_logical_url(&with_hash), "fynexo://settings");
    }

    #[test]
    fn external_urls_pass_through_untouched() {
        assert_eq!(to_webview_url("  https://example.com/a?b=1 "), "https://example.com/a?b=1");
        assert_eq!(normalize_logical_url("https://example.com/"), "https://example.com/");
    }

    #[test]
    fn root_fynexo_hosts_normalise_to_newtab() {
        assert_eq!(normalize_logical_url("http://fynexo.localhost"), "fynexo://newtab");
        assert_eq!(normalize_logical_url("http://fynexo.localhost/"), "fynexo://newtab");
        assert_eq!(normalize_logical_url("fynexo://localhost"), "fynexo://newtab");
    }

    #[test]
    fn internal_url_detection_handles_logical_and_physical_forms() {
        assert!(is_internal_url("fynexo://newtab"));
        assert!(is_internal_url("http://fynexo.localhost/newtab"));
        assert!(is_internal_url("fynexo://settings"));
        assert!(is_internal_url("http://fynexo.localhost/settings#sec-appearance"));
        assert!(!is_internal_url("https://example.com/"));
        assert!(!is_internal_url("https://github.com"));
    }

    #[test]
    fn url_cleaner_unwraps_redirects_and_strips_tracking_params() {
        assert_eq!(
            sanitize_and_clean_url(
                "https://link.zhihu.com/?target=https%3A%2F%2Fgithub.com%2Fhorizoralabs%2Ffynexo%3Futm_source%3Dzhihu%26tab%3Dreadme",
                true
            ),
            "https://github.com/horizoralabs/fynexo?tab=readme"
        );
        assert_eq!(
            sanitize_and_clean_url(
                "https://www.bilibili.com/video/BV1xx411c7mD/?spm_id_from=333.1007.tianma&vd_source=abc&t=42#reply",
                true
            ),
            "https://www.bilibili.com/video/BV1xx411c7mD/?t=42#reply"
        );
        assert_eq!(
            sanitize_and_clean_url(
                "https://www.youtube.com/watch?v=dQw4w9WgXcQ&si=abcdef123&feature=shared",
                true
            ),
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
        assert_eq!(
            sanitize_and_clean_url(
                "https://x.com/rustlang/status/12345?s=20&t=xyz",
                true
            ),
            "https://x.com/rustlang/status/12345"
        );
        // Preserves functional params when disabled or non-tracking
        assert_eq!(
            sanitize_and_clean_url("https://example.com/?utm_source=x&q=rust", false),
            "https://example.com/?utm_source=x&q=rust"
        );
        assert_eq!(
            sanitize_and_clean_url("https://mp.weixin.qq.com/s?__biz=1&mid=2&idx=1&sn=abc&chksm=def&utm_medium=social", true),
            "https://mp.weixin.qq.com/s?__biz=1&mid=2&idx=1&sn=abc&chksm=def"
        );
    }
}
