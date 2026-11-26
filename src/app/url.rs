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
    norm == "fynexo://newtab" || norm == "fynexo://settings"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_webview_and_back_roundtrip_for_internal_pages() {
        for logical in ["fynexo://newtab", "fynexo://settings"] {
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
}
