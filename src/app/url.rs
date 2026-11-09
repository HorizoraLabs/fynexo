#[inline]
pub fn to_webview_url(url: &str) -> String {
    let trimmed = url.trim();
    let internal_route = if trimmed.eq_ignore_ascii_case("fynexo://newtab")
        || trimmed.starts_with("fynexo://newtab")
        || trimmed.eq_ignore_ascii_case("http://fynexo.localhost/newtab")
    {
        Some("newtab")
    } else if trimmed.eq_ignore_ascii_case("fynexo://settings")
        || trimmed.starts_with("fynexo://settings")
        || trimmed.eq_ignore_ascii_case("http://fynexo.localhost/settings")
    {
        Some("settings")
    } else {
        None
    };

    if let Some(route) = internal_route {
        #[cfg(any(target_os = "windows", target_os = "android"))]
        {
            return format!("http://fynexo.localhost/{}", route);
        }
        #[cfg(not(any(target_os = "windows", target_os = "android")))]
        {
            return format!("fynexo://localhost/{}", route);
        }
    }
    trimmed.to_string()
}

#[inline]
pub fn normalize_logical_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("fynexo://settings")
        || trimmed.contains("fynexo.localhost/settings")
        || trimmed.contains("fynexo.settings")
    {
        "fynexo://settings".to_string()
    } else if trimmed.starts_with("fynexo://newtab")
        || trimmed.contains("fynexo.localhost/newtab")
        || trimmed.contains("fynexo.newtab")
        || trimmed.trim_end_matches('/') == "http://fynexo.localhost"
        || trimmed.trim_end_matches('/') == "fynexo://localhost"
    {
        "fynexo://newtab".to_string()
    } else {
        trimmed.to_string()
    }
}
