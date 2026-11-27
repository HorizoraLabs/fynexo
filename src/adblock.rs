use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, RwLock};
use std::time::SystemTime;

use crate::config::{
    display_path_preferring_local, synced_json_mtime, synced_json_path, synced_json_save,
};

static ADBLOCK_ENABLED: AtomicBool = AtomicBool::new(true);
static BLOCKED_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Curated zero-false-positive advertising, popunder, and telemetry root domains.
/// Never includes CDNs, auth providers, video streams, or font hosts.
pub const AD_DOMAINS: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "google-analytics.com",
    "googletagmanager.com",
    "googletagservices.com",
    "2mdn.net",
    "adnxs.com",
    "rubiconproject.com",
    "pubmatic.com",
    "criteo.com",
    "criteo.net",
    "taboola.com",
    "outbrain.com",
    "scorecardresearch.com",
    "quantserve.com",
    "adsrvr.org",
    "moatads.com",
    "amazon-adsystem.com",
    "adform.net",
    "openx.net",
    "casalemedia.com",
    "smartadserver.com",
    "teads.tv",
    "popads.net",
    "popcash.net",
    "propellerads.com",
    "exoclick.com",
    "juicyads.com",
    "trafficjunky.net",
    "adsterra.com",
    "revcontent.com",
    "mgid.com",
    "zemanta.com",
    "sharethrough.com",
    "indexexchange.com",
    "3lift.com",
    "bidswitch.net",
    "contextweb.com",
    "lijit.com",
    "sovrn.com",
    "advertising.com",
    "ads-twitter.com",
    "adcolony.com",
    "applovin.com",
    "vungle.com",
    "chartboost.com",
    "inmobi.com",
    "flurry.com",
    "mixpanel.com",
    "hotjar.com",
    "clarity.ms",
    "nr-data.net",
    "mc.yandex.ru",
    "hm.baidu.com",
    "pos.baidu.com",
    "cpro.baidu.com",
    "eclick.baidu.com",
    "cbjs.baidu.com",
    "tanx.com",
    "mmstat.com",
    "cnzz.com",
    "umeng.com",
    "miaozhen.com",
];

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct AdblockConfig {
    pub custom_blocked_domains: Vec<String>,
    pub whitelisted_domains: Vec<String>,
}

pub fn normalize_domain_entry(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return None;
    }
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .or_else(|| trimmed.strip_prefix("*://"))
        .unwrap_or(&trimmed);
    let host_part = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim_start_matches("*.")
        .trim_start_matches("www.")
        .trim_matches('.');
    let host = host_part.split(':').next().unwrap_or("").trim();
    if host.is_empty()
        || host.len() > 253
        || !host.contains('.')
        || host.contains("..")
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
        || host
            .split('.')
            .any(|lbl| lbl.is_empty() || lbl.starts_with('-') || lbl.ends_with('-'))
    {
        None
    } else {
        Some(host.to_string())
    }
}

impl AdblockConfig {
    pub fn path() -> PathBuf {
        synced_json_path("adblock.json")
    }

    pub fn path_display() -> String {
        display_path_preferring_local("adblock.json")
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("adblock.json")
    }

    pub fn sanitize(&mut self) {
        let mut clean_blocked = Vec::with_capacity(self.custom_blocked_domains.len());
        for d in &self.custom_blocked_domains {
            if let Some(norm) = normalize_domain_entry(d) {
                if !clean_blocked.contains(&norm) {
                    clean_blocked.push(norm);
                }
            }
        }
        self.custom_blocked_domains = clean_blocked;

        let mut clean_white = Vec::with_capacity(self.whitelisted_domains.len());
        for d in &self.whitelisted_domains {
            if let Some(norm) = normalize_domain_entry(d) {
                if !clean_white.contains(&norm) {
                    clean_white.push(norm);
                }
            }
        }
        self.whitelisted_domains = clean_white;
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok(mut cfg) = serde_json::from_str::<AdblockConfig>(&raw) {
                    cfg.sanitize();
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        default_cfg.save();
        default_cfg
    }

    pub fn save(&self) {
        let mut normalized = self.clone();
        normalized.sanitize();
        if let Ok(json) = serde_json::to_string_pretty(&normalized) {
            synced_json_save("adblock.json", &json);
        }
    }
}

#[derive(Default, Clone)]
struct DynamicAdblockState {
    custom_blocked: HashSet<String>,
    whitelisted: HashSet<String>,
    active_page_host: String,
}

static AD_DOMAIN_SET: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::with_capacity(AD_DOMAINS.len() * 2);
    for &d in AD_DOMAINS {
        set.insert(d);
    }
    set
});

static DYNAMIC_ADBLOCK: LazyLock<RwLock<Arc<DynamicAdblockState>>> =
    LazyLock::new(|| RwLock::new(Arc::new(DynamicAdblockState::default())));
static HAS_DYNAMIC_RULES: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn set_enabled(enabled: bool) {
    ADBLOCK_ENABLED.store(enabled, Ordering::Relaxed);
}

#[inline]
pub fn is_enabled() -> bool {
    ADBLOCK_ENABLED.load(Ordering::Relaxed)
}

#[inline]
pub fn blocked_count() -> usize {
    BLOCKED_COUNT.load(Ordering::Relaxed)
}

#[inline]
pub fn record_block() {
    BLOCKED_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn sync_dynamic_lists(cfg: &AdblockConfig) {
    if let Ok(mut guard) = DYNAMIC_ADBLOCK.write() {
        let mut next = (**guard).clone();
        next.custom_blocked = cfg
            .custom_blocked_domains
            .iter()
            .filter_map(|d| normalize_domain_entry(d))
            .collect();
        next.whitelisted = cfg
            .whitelisted_domains
            .iter()
            .filter_map(|d| normalize_domain_entry(d))
            .collect();
        HAS_DYNAMIC_RULES.store(
            !next.custom_blocked.is_empty() || !next.whitelisted.is_empty(),
            Ordering::Relaxed,
        );
        *guard = Arc::new(next);
    }
}

pub fn set_active_page_url(url: &str) {
    let host = extract_host(url)
        .map(|h| h.to_ascii_lowercase())
        .unwrap_or_default();
    if let Ok(mut guard) = DYNAMIC_ADBLOCK.write() {
        if guard.active_page_host != host {
            let mut next = (**guard).clone();
            next.active_page_host = host;
            *guard = Arc::new(next);
        }
    }
}

/// Zero-allocation host extractor from an HTTP(S) URL.
pub fn extract_host(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = host_port
        .split(':')
        .next()
        .unwrap_or(host_port)
        .trim_end_matches('.');
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

#[inline]
fn matches_domain_set(lower_host: &str, set: &HashSet<String>) -> bool {
    if set.is_empty() || lower_host.is_empty() {
        return false;
    }
    let mut cur = lower_host;
    loop {
        if set.contains(cur) {
            return true;
        }
        match cur.find('.') {
            Some(idx) => {
                cur = &cur[idx + 1..];
                if !cur.contains('.') {
                    break;
                }
            }
            None => break,
        }
    }
    false
}

/// Hierarchical domain lookup checking whitelist first, then static + custom blocklists.
/// When `page_host` is `Some(host)`, whitelist evaluation is scoped strictly to that tab's
/// host rather than the process-wide `active_page_host`, eliminating cross-tab/cross-window
/// whitelist races.
pub fn is_blocked_host_for_page(host: &str, page_host: Option<&str>) -> bool {
    if host.is_empty() || host.starts_with("fynexo.") || host == "localhost" {
        return false;
    }

    let lower_cow: std::borrow::Cow<'_, str> = if host.bytes().all(|b| !b.is_ascii_uppercase()) {
        std::borrow::Cow::Borrowed(host)
    } else {
        std::borrow::Cow::Owned(host.to_ascii_lowercase())
    };
    let lower = lower_cow.as_ref();

    let custom_matched = if HAS_DYNAMIC_RULES.load(Ordering::Relaxed) {
        let snapshot = DYNAMIC_ADBLOCK.read().ok().map(|guard| Arc::clone(&*guard));
        if let Some(snap) = snapshot {
            let effective_page_host = page_host.unwrap_or(&snap.active_page_host);
            if !snap.whitelisted.is_empty()
                && (matches_domain_set(effective_page_host, &snap.whitelisted)
                    || matches_domain_set(lower, &snap.whitelisted))
            {
                return false;
            }
            matches_domain_set(lower, &snap.custom_blocked)
        } else {
            false
        }
    } else {
        false
    };

    if custom_matched {
        return true;
    }

    let mut cur = lower;
    loop {
        if AD_DOMAIN_SET.contains(cur) {
            return true;
        }
        match cur.find('.') {
            Some(idx) => {
                cur = &cur[idx + 1..];
                if !cur.contains('.') {
                    break;
                }
            }
            None => break,
        }
    }
    false
}

/// Hierarchical domain lookup using the global active page host fallback.
pub fn is_blocked_host(host: &str) -> bool {
    is_blocked_host_for_page(host, None)
}

/// Detects unmistakable first-party YouTube ad-serving/ad-telemetry URLs on `*.youtube.com`
/// (never touches `googlevideo.com` media streams, `/youtubei/v1/player`, or watchtime pings).
pub fn is_youtube_first_party_ad_url(lower_host: &str, url: &str) -> bool {
    if lower_host != "youtube.com" && !lower_host.ends_with(".youtube.com") {
        return false;
    }
    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let Some(slash_idx) = after_scheme.find('/') else {
        return false;
    };
    let path_and_query = after_scheme[slash_idx..].to_ascii_lowercase();
    path_and_query.starts_with("/pagead/")
        || path_and_query.starts_with("/api/stats/ads")
        || path_and_query.starts_with("/ptracking")
        || path_and_query.starts_with("/get_midroll_info")
        || path_and_query.starts_with("/ad_data_204")
}

#[inline]
pub fn should_block_url_for_page(url: &str, page_host: Option<&str>) -> bool {
    if !is_enabled() {
        return false;
    }
    match extract_host(url) {
        Some(host) => {
            if is_blocked_host_for_page(host, page_host) {
                return true;
            }
            let lower = host.to_ascii_lowercase();
            if is_youtube_first_party_ad_url(&lower, url) {
                if HAS_DYNAMIC_RULES.load(Ordering::Relaxed) {
                    if let Some(snap) = DYNAMIC_ADBLOCK.read().ok().map(|g| Arc::clone(&*g)) {
                        let eff_page = page_host.unwrap_or(&snap.active_page_host);
                        if !snap.whitelisted.is_empty()
                            && (matches_domain_set(eff_page, &snap.whitelisted)
                                || matches_domain_set(&lower, &snap.whitelisted))
                        {
                            return false;
                        }
                    }
                }
                return true;
            }
            false
        }
        None => false,
    }
}

#[inline]
pub fn should_block_url(url: &str) -> bool {
    should_block_url_for_page(url, None)
}

#[cfg(target_os = "windows")]
pub fn sync_windows_custom_filters(webview: &wry::WebView, cfg: &AdblockConfig) {
    use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
    use windows::core::HSTRING;
    use wry::WebViewExtWindows;

    let core_wv = webview.webview();
    unsafe {
        for domain in &cfg.custom_blocked_domains {
            if let Some(clean) = normalize_domain_entry(domain) {
                let sub_pattern = HSTRING::from(format!("*://*.{}/*", clean));
                let root_pattern = HSTRING::from(format!("*://{}/*", clean));
                let _ = core_wv.AddWebResourceRequestedFilter(
                    &sub_pattern,
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                );
                let _ = core_wv.AddWebResourceRequestedFilter(
                    &root_pattern,
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn attach_windows_adblock(
    webview: &wry::WebView,
    cfg: &AdblockConfig,
    page_host: Arc<RwLock<String>>,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
    use webview2_com::{take_pwstr, WebResourceRequestedEventHandler};
    use windows::core::{w, HSTRING, PWSTR};
    use wry::WebViewExtWindows;

    let core_wv = webview.webview();
    let env = webview.environment();

    unsafe {
        for &domain in AD_DOMAINS {
            let sub_pattern = HSTRING::from(format!("*://*.{}/*", domain));
            let root_pattern = HSTRING::from(format!("*://{}/*", domain));
            let _ = core_wv.AddWebResourceRequestedFilter(
                &sub_pattern,
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            );
            let _ = core_wv.AddWebResourceRequestedFilter(
                &root_pattern,
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            );
        }
    }

    sync_windows_custom_filters(webview, cfg);

    unsafe {
        let mut token: i64 = 0;
        let _ = core_wv.add_WebResourceRequested(
            &WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
                if !is_enabled() {
                    return Ok(());
                }
                let Some(args) = args else {
                    return Ok(());
                };
                let req = args.Request()?;
                let mut uri_pwstr = PWSTR::null();
                req.Uri(&mut uri_pwstr)?;
                let uri = take_pwstr(uri_pwstr);

                let mut tab_host = page_host
                    .read()
                    .ok()
                    .map(|g| g.clone())
                    .unwrap_or_default();
                if tab_host.is_empty() {
                    if let Some(wv_ref) = sender.as_ref() {
                        let mut src_pwstr = PWSTR::null();
                        if wv_ref.Source(&mut src_pwstr).is_ok() {
                            let src = take_pwstr(src_pwstr);
                            if let Some(h) = extract_host(&src) {
                                tab_host = h.to_ascii_lowercase();
                            }
                        }
                    }
                }

                if should_block_url_for_page(&uri, Some(&tab_host)) {
                    record_block();
                    if let Ok(resp) = env.CreateWebResourceResponse(
                        None,
                        204,
                        w!("No Content"),
                        w!("Cache-Control: max-age=3600"),
                    ) {
                        let _ = args.SetResponse(&resp);
                    }
                }
                Ok(())
            })),
            &mut token,
        );
    }
}

/// Builds the Chrome DevTools Protocol `Emulation.setUserAgentOverride` JSON payload so
/// Chromium Client Hints (`Sec-CH-UA*`, `navigator.userAgentData`, `navigator.platform`)
/// stay consistent with the configured User-Agent string (or resets them when `None`).
pub fn cdp_user_agent_override_params(custom_ua: Option<&str>) -> String {
    let Some(raw) = custom_ua.map(str::trim).filter(|s| !s.is_empty()) else {
        return r#"{"userAgent":""}"#.to_string();
    };
    let lower = raw.to_ascii_lowercase();
    let is_android = lower.contains("android");
    let is_ios = lower.contains("iphone") || lower.contains("ipad") || lower.contains("ipod");
    let is_mobile = is_android || is_ios || lower.contains("mobile");
    let is_macos = !is_ios && (lower.contains("mac os x") || lower.contains("macintosh"));
    let is_linux = !is_android && lower.contains("linux");

    let chrome_full = raw
        .split_whitespace()
        .find_map(|tok| tok.strip_prefix("Chrome/").or_else(|| tok.strip_prefix("CriOS/")))
        .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .unwrap_or("134.0.0.0");
    let chrome_major = chrome_full.split('.').next().unwrap_or("134");

    let (ch_platform, ch_platform_ver, nav_platform, arch, bitness, model) = if is_android {
        let ver = raw
            .split("Android ")
            .nth(1)
            .and_then(|rest| rest.split([';', ')']).next())
            .map(str::trim)
            .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.'))
            .map(|v| {
                if v.contains('.') {
                    v.to_string()
                } else {
                    format!("{v}.0.0")
                }
            })
            .unwrap_or_else(|| "14.0.0".to_string());
        let dev_model = if lower.contains("pixel 8") {
            "Pixel 8"
        } else {
            ""
        };
        ("Android", ver, "Linux armv8l", "", "", dev_model)
    } else if is_ios {
        ("iOS", "17.0.0".to_string(), "iPhone", "", "", "")
    } else if is_macos {
        ("macOS", "14.5.0".to_string(), "MacIntel", "arm", "64", "")
    } else if is_linux {
        ("Linux", "6.8.0".to_string(), "Linux x86_64", "x86", "64", "")
    } else {
        ("Windows", "15.0.0".to_string(), "Win32", "x86", "64", "")
    };

    serde_json::json!({
        "userAgent": raw,
        "platform": nav_platform,
        "userAgentMetadata": {
            "brands": [
                { "brand": "Not:A-Brand", "version": "24" },
                { "brand": "Chromium", "version": chrome_major },
                { "brand": "Google Chrome", "version": chrome_major }
            ],
            "fullVersionList": [
                { "brand": "Not:A-Brand", "version": "24.0.0.0" },
                { "brand": "Chromium", "version": chrome_full },
                { "brand": "Google Chrome", "version": chrome_full }
            ],
            "fullVersion": chrome_full,
            "platform": ch_platform,
            "platformVersion": ch_platform_ver,
            "architecture": arch,
            "model": model,
            "mobile": is_mobile,
            "bitness": bitness,
            "wow64": false
        }
    })
    .to_string()
}

#[cfg(target_os = "windows")]
static DEFAULT_WINDOWS_UA: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));

#[cfg(target_os = "windows")]
pub fn capture_default_windows_ua(webview: &wry::WebView) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings2;
    use windows::core::Interface;
    use wry::WebViewExtWindows;

    if DEFAULT_WINDOWS_UA.read().ok().and_then(|g| g.clone()).is_some() {
        return;
    }
    let core_wv = webview.webview();
    unsafe {
        if let Ok(settings) = core_wv.Settings() {
            if let Ok(settings2) = settings.cast::<ICoreWebView2Settings2>() {
                let mut pw = windows::core::PWSTR::null();
                if settings2.UserAgent(&mut pw).is_ok() {
                    let orig = webview2_com::take_pwstr(pw);
                    if !orig.is_empty()
                        && orig != crate::config::DESKTOP_CHROME_UA
                        && orig != crate::config::MOBILE_CHROME_UA
                    {
                        if let Ok(mut g) = DEFAULT_WINDOWS_UA.write() {
                            *g = Some(orig);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn apply_windows_user_agent(webview: &wry::WebView, custom_ua: Option<&str>) {
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings2;
    use windows::core::{w, Interface, HSTRING};
    use wry::WebViewExtWindows;

    capture_default_windows_ua(webview);

    let core_wv = webview.webview();
    unsafe {
        if let Ok(settings) = core_wv.Settings() {
            if let Ok(settings2) = settings.cast::<ICoreWebView2Settings2>() {
                let target_ua = match custom_ua {
                    Some(ua) if !ua.trim().is_empty() => ua.to_string(),
                    _ => DEFAULT_WINDOWS_UA
                        .read()
                        .ok()
                        .and_then(|g| g.clone())
                        .unwrap_or_default(),
                };
                let h = HSTRING::from(target_ua);
                let _ = settings2.SetUserAgent(&h);
            }
        }

        // Synchronize Chromium Client Hints (Sec-CH-UA*, navigator.userAgentData, navigator.platform)
        let cdp_params = HSTRING::from(cdp_user_agent_override_params(custom_ua));
        let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|_, _| Ok(())));
        let _ = core_wv.CallDevToolsProtocolMethod(
            w!("Emulation.setUserAgentOverride"),
            &cdp_params,
            &handler,
        );
    }
}

#[cfg(target_os = "windows")]
pub fn apply_windows_autofill(
    webview: &wry::WebView,
    password_autosave: bool,
    general_autofill: bool,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings4;
    use windows::core::Interface;
    use wry::WebViewExtWindows;

    let core_wv = webview.webview();
    unsafe {
        if let Ok(settings) = core_wv.Settings() {
            if let Ok(settings4) = settings.cast::<ICoreWebView2Settings4>() {
                let _ = settings4.SetIsPasswordAutosaveEnabled(password_autosave);
                let _ = settings4.SetIsGeneralAutofillEnabled(general_autofill);
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn apply_windows_devtools(webview: &wry::WebView, enabled: bool) {
    use wry::WebViewExtWindows;

    let core_wv = webview.webview();
    unsafe {
        if let Ok(settings) = core_wv.Settings() {
            let _ = settings.SetAreDevToolsEnabled(enabled);
        }
    }
}

#[cfg(target_os = "windows")]
pub fn attach_windows_resilience(
    webview: &wry::WebView,
    password_autosave: bool,
    general_autofill: bool,
    win_id: usize,
    tab_id: Arc<AtomicUsize>,
    proxy: tao::event_loop::EventLoopProxy<crate::app::events::UserEvent>,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_PROCESS_FAILED_KIND, COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED,
        COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED,
        COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE,
    };
    use webview2_com::ProcessFailedEventHandler;
    use wry::WebViewExtWindows;

    apply_windows_autofill(webview, password_autosave, general_autofill);

    let core_wv = webview.webview();
    unsafe {
        let mut token: i64 = 0;
        let _ = core_wv.add_ProcessFailed(
            &ProcessFailedEventHandler::create(Box::new(move |_, args| {
                let tid = tab_id.load(Ordering::Acquire);
                if tid == 0 {
                    return Ok(());
                }
                let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
                if let Some(a) = args {
                    let _ = a.ProcessFailedKind(&mut kind);
                }
                let unresponsive =
                    kind == COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE;
                if kind == COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED
                    || kind == COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED
                    || unresponsive
                {
                    let _ = proxy.send_event(crate::app::events::UserEvent::RendererCrashed {
                        win_id,
                        tab_id: tid,
                        unresponsive,
                    });
                }
                Ok(())
            })),
            &mut token,
        );
    }
}

#[cfg(target_os = "windows")]
pub fn launch_external_scheme_windows(url: &str) -> bool {
    use windows::core::{w, HSTRING};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    if url.len() > 2048 || url.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return false;
    }

    let lower = url.to_ascii_lowercase();
    let is_safe_external =
        lower.starts_with("mailto:") || lower.starts_with("tel:") || lower.starts_with("sms:");

    if !is_safe_external {
        return false;
    }

    let target = HSTRING::from(url);
    unsafe {
        let res = ShellExecuteW(
            Some(HWND::default()),
            w!("open"),
            &target,
            None,
            None,
            SW_SHOWNORMAL,
        );
        (res.0 as isize) > 32
    }
}

#[cfg(target_os = "windows")]
pub fn open_folder_in_explorer(path: &std::path::Path) {
    use windows::core::{w, HSTRING};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let _ = std::fs::create_dir_all(path);
    let target = HSTRING::from(path.as_os_str());
    unsafe {
        let _ = ShellExecuteW(
            Some(HWND::default()),
            w!("open"),
            &target,
            None,
            None,
            SW_SHOWNORMAL,
        );
    }
}

/// Returns `true` when the filename lacks a safe extension or looks like an executable/script/shortcut.
pub fn is_dangerous_download_filename(raw_name: &str) -> bool {
    let trimmed = raw_name.trim().trim_end_matches(['.', ' ']);
    let ext = std::path::Path::new(trimmed)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if ext.is_empty() {
        return true;
    }
    matches!(
        ext.as_str(),
        "exe"
            | "msi"
            | "msp"
            | "bat"
            | "cmd"
            | "com"
            | "scr"
            | "pif"
            | "ps1"
            | "psm1"
            | "vbs"
            | "vbe"
            | "js"
            | "jse"
            | "wsf"
            | "wsh"
            | "hta"
            | "lnk"
            | "url"
            | "reg"
            | "dll"
            | "cpl"
            | "inf"
            | "jar"
            | "py"
            | "sh"
            | "chm"
            | "scf"
    )
}

#[cfg(target_os = "windows")]
pub fn open_download_path(full: &std::path::Path) -> bool {
    use windows::core::{w, HSTRING};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    if !full.is_file() {
        return false;
    }
    let fname = full
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if is_dangerous_download_filename(fname) {
        let select_arg = HSTRING::from(format!("/select,\"{}\"", full.display()));
        unsafe {
            let res = ShellExecuteW(
                Some(HWND::default()),
                w!("open"),
                w!("explorer.exe"),
                &select_arg,
                None,
                SW_SHOWNORMAL,
            );
            return (res.0 as isize) > 32;
        }
    }
    let target = HSTRING::from(full.as_os_str());
    unsafe {
        let res = ShellExecuteW(
            Some(HWND::default()),
            w!("open"),
            &target,
            None,
            None,
            SW_SHOWNORMAL,
        );
        (res.0 as isize) > 32
    }
}

#[cfg(target_os = "windows")]
pub fn reveal_download_path(full: &std::path::Path, fallback_root: &std::path::Path) -> bool {
    use windows::core::{w, HSTRING};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    if !full.is_file() {
        open_folder_in_explorer(fallback_root);
        return false;
    }
    let select_arg = HSTRING::from(format!("/select,\"{}\"", full.display()));
    unsafe {
        let res = ShellExecuteW(
            Some(HWND::default()),
            w!("open"),
            w!("explorer.exe"),
            &select_arg,
            None,
            SW_SHOWNORMAL,
        );
        (res.0 as isize) > 32
    }
}

#[cfg(target_os = "windows")]
pub fn open_download_file(downloads_root: &std::path::Path, raw_name: &str) -> bool {
    let clean = raw_name.trim();
    if clean.is_empty()
        || clean.contains('/')
        || clean.contains('\\')
        || clean.contains("..")
    {
        return false;
    }
    let Some(file_only) = std::path::Path::new(clean).file_name() else {
        return false;
    };
    open_download_path(&downloads_root.join(file_only))
}

#[cfg(target_os = "windows")]
pub fn reveal_download_in_explorer(downloads_root: &std::path::Path, raw_name: &str) -> bool {
    let clean = raw_name.trim();
    if clean.is_empty()
        || clean.contains('/')
        || clean.contains('\\')
        || clean.contains("..")
    {
        return false;
    }
    let Some(file_only) = std::path::Path::new(clean).file_name() else {
        return false;
    };
    reveal_download_path(&downloads_root.join(file_only), downloads_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_entries_are_normalised() {
        assert_eq!(normalize_domain_entry("  HTTPS://WWW.Ads.Example.com/path?x=1 ").as_deref(), Some("ads.example.com"));
        assert_eq!(normalize_domain_entry("*.tracker.net").as_deref(), Some("tracker.net"));
        assert_eq!(normalize_domain_entry("host.io:8080/a").as_deref(), Some("host.io"));
        assert_eq!(normalize_domain_entry("*://cdn.example.org#frag").as_deref(), Some("cdn.example.org"));
    }

    #[test]
    fn invalid_domain_entries_are_rejected() {
        assert_eq!(normalize_domain_entry(""), None);
        assert_eq!(normalize_domain_entry("   "), None);
        assert_eq!(normalize_domain_entry("localhost"), None);
        assert_eq!(normalize_domain_entry("has space.com"), None);
        assert_eq!(normalize_domain_entry("bad..domain.com"), None);
        assert_eq!(normalize_domain_entry("-bad.example.com"), None);
        assert_eq!(normalize_domain_entry("bad\"quote.com"), None);
        assert_eq!(normalize_domain_entry("bad*wild.com"), None);
    }

    #[test]
    fn sanitize_dedups_and_cleans_lists() {
        let mut cfg = AdblockConfig {
            custom_blocked_domains: vec!["Ads.com".into(), "https://ads.com/x".into(), "bad".into()],
            whitelisted_domains: vec!["*.ok.org".into(), "ok.org".into()],
        };
        cfg.sanitize();
        assert_eq!(cfg.custom_blocked_domains, vec!["ads.com".to_string()]);
        assert_eq!(cfg.whitelisted_domains, vec!["ok.org".to_string()]);
    }

    #[test]
    fn dangerous_download_extensions_are_flagged() {
        for bad in [
            "setup.exe",
            "SCRIPT.BAT",
            "payload.exe.",
            "run.ps1 ",
            "helper.js",
            "shortcut.lnk",
            "installer.msi",
            "noext",
        ] {
            assert!(is_dangerous_download_filename(bad), "expected dangerous: {bad}");
        }
        for safe in [
            "report.pdf",
            "photo.png",
            "fynexo_rules.json",
            "notes.txt",
            "archive.zip",
            "clip.mp4",
        ] {
            assert!(!is_dangerous_download_filename(safe), "expected safe: {safe}");
        }
    }

    #[test]
    fn per_tab_whitelist_does_not_leak_across_tabs() {
        let cfg = AdblockConfig {
            custom_blocked_domains: vec!["tracker.custom.example".into()],
            whitelisted_domains: vec!["friendly.example".into()],
        };
        sync_dynamic_lists(&cfg);
        set_active_page_url("https://friendly.example/article");

        // Tab on the whitelisted host bypasses adblock
        assert!(!is_blocked_host_for_page(
            "ads.doubleclick.net",
            Some("friendly.example")
        ));
        // Concurrent tab on a different host MUST still block ads even while global active_page_host is whitelisted
        assert!(is_blocked_host_for_page(
            "ads.doubleclick.net",
            Some("other.example")
        ));
        assert!(is_blocked_host_for_page(
            "sub.tracker.custom.example",
            Some("other.example")
        ));
    }

    #[test]
    fn cdp_user_agent_override_syncs_client_hints_metadata() {
        let reset: serde_json::Value =
            serde_json::from_str(&cdp_user_agent_override_params(None)).unwrap();
        assert_eq!(reset["userAgent"], "");
        assert!(reset.get("userAgentMetadata").is_none());

        let mob: serde_json::Value = serde_json::from_str(&cdp_user_agent_override_params(Some(
            crate::config::MOBILE_CHROME_UA,
        )))
        .unwrap();
        assert_eq!(mob["platform"], "Linux armv8l");
        assert_eq!(mob["userAgentMetadata"]["mobile"], true);
        assert_eq!(mob["userAgentMetadata"]["platform"], "Android");
        assert_eq!(mob["userAgentMetadata"]["platformVersion"], "14.0.0");
        assert_eq!(mob["userAgentMetadata"]["model"], "Pixel 8");

        let desk: serde_json::Value = serde_json::from_str(&cdp_user_agent_override_params(Some(
            crate::config::DESKTOP_CHROME_UA,
        )))
        .unwrap();
        assert_eq!(desk["platform"], "Win32");
        assert_eq!(desk["userAgentMetadata"]["mobile"], false);
        assert_eq!(desk["userAgentMetadata"]["platform"], "Windows");
    }

    #[test]
    fn youtube_first_party_ad_urls_are_blocked_without_breaking_playback() {
        assert!(is_youtube_first_party_ad_url(
            "m.youtube.com",
            "https://m.youtube.com/pagead/adview?ai=1"
        ));
        assert!(is_youtube_first_party_ad_url(
            "www.youtube.com",
            "https://www.youtube.com/api/stats/ads?ns=yt"
        ));
        assert!(is_youtube_first_party_ad_url(
            "m.youtube.com",
            "https://m.youtube.com/get_midroll_info?v=abc"
        ));
        assert!(!is_youtube_first_party_ad_url(
            "m.youtube.com",
            "https://m.youtube.com/watch?v=dQw4w9WgXcQ"
        ));
        assert!(!is_youtube_first_party_ad_url(
            "m.youtube.com",
            "https://m.youtube.com/youtubei/v1/player"
        ));
        assert!(!is_youtube_first_party_ad_url(
            "rr1---sn.googlevideo.com",
            "https://rr1---sn.googlevideo.com/videoplayback?id=1"
        ));
    }
}
