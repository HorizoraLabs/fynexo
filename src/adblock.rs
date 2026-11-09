use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{LazyLock, RwLock};
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
    if host.is_empty() || !host.contains('.') || host.contains(char::is_whitespace) {
        None
    } else {
        Some(host.to_string())
    }
}

impl AdblockConfig {
    pub fn path() -> PathBuf {
        synced_json_path("fynexo_adblock.json")
    }

    pub fn path_display() -> String {
        display_path_preferring_local("fynexo_adblock.json")
    }

    pub fn mtime() -> Option<SystemTime> {
        synced_json_mtime("fynexo_adblock.json")
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
            synced_json_save("fynexo_adblock.json", &json);
        }
    }
}

#[derive(Default)]
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

static DYNAMIC_ADBLOCK: LazyLock<RwLock<DynamicAdblockState>> =
    LazyLock::new(|| RwLock::new(DynamicAdblockState::default()));

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
        guard.custom_blocked = cfg
            .custom_blocked_domains
            .iter()
            .filter_map(|d| normalize_domain_entry(d))
            .collect();
        guard.whitelisted = cfg
            .whitelisted_domains
            .iter()
            .filter_map(|d| normalize_domain_entry(d))
            .collect();
    }
}

pub fn set_active_page_url(url: &str) {
    let host = extract_host(url)
        .map(|h| h.to_ascii_lowercase())
        .unwrap_or_default();
    if let Ok(mut guard) = DYNAMIC_ADBLOCK.write() {
        guard.active_page_host = host;
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
pub fn is_blocked_host(host: &str) -> bool {
    if host.is_empty() || host.starts_with("fynexo.") || host == "localhost" {
        return false;
    }

    let lower_cow: std::borrow::Cow<'_, str> = if host.bytes().all(|b| !b.is_ascii_uppercase()) {
        std::borrow::Cow::Borrowed(host)
    } else {
        std::borrow::Cow::Owned(host.to_ascii_lowercase())
    };
    let lower = lower_cow.as_ref();

    let custom_matched = if let Ok(guard) = DYNAMIC_ADBLOCK.read() {
        if !guard.whitelisted.is_empty()
            && (matches_domain_set(&guard.active_page_host, &guard.whitelisted)
                || matches_domain_set(lower, &guard.whitelisted))
        {
            return false;
        }
        matches_domain_set(lower, &guard.custom_blocked)
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

#[inline]
pub fn should_block_url(url: &str) -> bool {
    if !is_enabled() {
        return false;
    }
    match extract_host(url) {
        Some(host) => is_blocked_host(host),
        None => false,
    }
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
pub fn attach_windows_adblock(webview: &wry::WebView, cfg: &AdblockConfig) {
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
            &WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
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

                if should_block_url(&uri) {
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

#[cfg(target_os = "windows")]
pub fn apply_windows_user_agent(webview: &wry::WebView, custom_ua: Option<&str>) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings2;
    use windows::core::{Interface, HSTRING};
    use wry::WebViewExtWindows;

    static DEFAULT_UA: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));

    let core_wv = webview.webview();
    unsafe {
        if let Ok(settings) = core_wv.Settings() {
            if let Ok(settings2) = settings.cast::<ICoreWebView2Settings2>() {
                // Capture system default User-Agent once before overriding
                if DEFAULT_UA.read().ok().and_then(|g| g.clone()).is_none() {
                    let mut pw = windows::core::PWSTR::null();
                    if settings2.UserAgent(&mut pw).is_ok() {
                        let orig = webview2_com::take_pwstr(pw);
                        if !orig.is_empty() {
                            if let Ok(mut g) = DEFAULT_UA.write() {
                                *g = Some(orig);
                            }
                        }
                    }
                }

                let target_ua = match custom_ua {
                    Some(ua) if !ua.trim().is_empty() => Some(ua.to_string()),
                    _ => DEFAULT_UA.read().ok().and_then(|g| g.clone()),
                };

                if let Some(ua) = target_ua {
                    let h = HSTRING::from(ua);
                    let _ = settings2.SetUserAgent(&h);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn launch_external_scheme_windows(url: &str) -> bool {
    use windows::core::{w, HSTRING};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

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
