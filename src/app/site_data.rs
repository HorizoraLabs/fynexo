//! Native Windows WebView2 Site Data (per-origin Cookie Manager) & Page Screenshot (`CapturePreview`).

use serde::{Deserialize, Serialize};
use tao::event_loop::EventLoopProxy;

use super::events::UserEvent;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SiteCookieItem {
    pub name: String,
    pub domain: String,
    pub path: String,
    pub is_http_only: bool,
    pub is_secure: bool,
    pub is_session: bool,
}

/// Enumerate cookies for `uri` via WebView2's `ICoreWebView2CookieManager` (metadata only; values stay private).
#[cfg(target_os = "windows")]
pub fn query_origin_cookies(
    webview: &wry::WebView,
    uri: &str,
    win_id: usize,
    req_id: u64,
    proxy: EventLoopProxy<UserEvent>,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_2;
    use webview2_com::{take_pwstr, GetCookiesCompletedHandler};
    use windows::core::{Interface, HSTRING, PWSTR};
    use wry::WebViewExtWindows;

    let core = webview.webview();
    let Ok(wv2) = core.cast::<ICoreWebView2_2>() else {
        let _ = proxy.send_event(UserEvent::SiteCookiesResult {
            win_id,
            req_id,
            uri: uri.to_string(),
            cookies: Vec::new(),
        });
        return;
    };
    let Ok(mgr) = (unsafe { wv2.CookieManager() }) else {
        let _ = proxy.send_event(UserEvent::SiteCookiesResult {
            win_id,
            req_id,
            uri: uri.to_string(),
            cookies: Vec::new(),
        });
        return;
    };

    let uri_owned = uri.to_string();
    let uri_hs = HSTRING::from(uri);
    let handler = GetCookiesCompletedHandler::create(Box::new(move |hr, list_opt| {
        let mut cookies = Vec::new();
        if hr.is_ok() {
            if let Some(list) = list_opt {
                unsafe {
                    let mut count: u32 = 0;
                    if list.Count(&mut count).is_ok() {
                        let capped = count.min(120);
                        for idx in 0..capped {
                            if let Ok(c) = list.GetValueAtIndex(idx) {
                                let mut name_p = PWSTR::null();
                                let mut dom_p = PWSTR::null();
                                let mut path_p = PWSTR::null();
                                let name = if c.Name(&mut name_p).is_ok() {
                                    take_pwstr(name_p)
                                } else {
                                    String::new()
                                };
                                let domain = if c.Domain(&mut dom_p).is_ok() {
                                    take_pwstr(dom_p)
                                } else {
                                    String::new()
                                };
                                let path = if c.Path(&mut path_p).is_ok() {
                                    take_pwstr(path_p)
                                } else {
                                    "/".to_string()
                                };
                                let mut http_only = windows_core_bool_false();
                                let mut secure = windows_core_bool_false();
                                let mut session = windows_core_bool_false();
                                let _ = c.IsHttpOnly(&mut http_only);
                                let _ = c.IsSecure(&mut secure);
                                let _ = c.IsSession(&mut session);
                                if !name.is_empty() {
                                    cookies.push(SiteCookieItem {
                                        name,
                                        domain,
                                        path,
                                        is_http_only: http_only.as_bool(),
                                        is_secure: secure.as_bool(),
                                        is_session: session.as_bool(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        let _ = proxy.send_event(UserEvent::SiteCookiesResult {
            win_id,
            req_id,
            uri: uri_owned.clone(),
            cookies,
        });
        Ok(())
    }));

    unsafe {
        let _ = mgr.GetCookies(&uri_hs, &handler);
    }
}

#[cfg(target_os = "windows")]
#[inline]
fn windows_core_bool_false() -> windows::core::BOOL {
    windows::core::BOOL(0)
}

#[cfg(not(target_os = "windows"))]
pub fn query_origin_cookies(
    _webview: &wry::WebView,
    uri: &str,
    win_id: usize,
    req_id: u64,
    proxy: EventLoopProxy<UserEvent>,
) {
    let _ = proxy.send_event(UserEvent::SiteCookiesResult {
        win_id,
        req_id,
        uri: uri.to_string(),
        cookies: Vec::new(),
    });
}

/// Delete a single named cookie for `(name, domain, path)` via WebView2 CookieManager.
#[cfg(target_os = "windows")]
pub fn delete_site_cookie(webview: &wry::WebView, name: &str, domain: &str, path: &str) -> bool {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_2;
    use windows::core::{Interface, HSTRING};
    use wry::WebViewExtWindows;

    let core = webview.webview();
    let Ok(wv2) = core.cast::<ICoreWebView2_2>() else {
        return false;
    };
    let Ok(mgr) = (unsafe { wv2.CookieManager() }) else {
        return false;
    };
    let n = HSTRING::from(name);
    let d = HSTRING::from(domain);
    let p = HSTRING::from(if path.is_empty() { "/" } else { path });
    unsafe { mgr.DeleteCookiesWithDomainAndPath(&n, &d, &p).is_ok() }
}

#[cfg(not(target_os = "windows"))]
pub fn delete_site_cookie(_webview: &wry::WebView, _name: &str, _domain: &str, _path: &str) -> bool {
    false
}

/// Delete every WebView2 cookie (including HttpOnly cookies) matching `uri`.
#[cfg(target_os = "windows")]
pub fn clear_origin_cookies(
    webview: &wry::WebView,
    uri: &str,
    win_id: usize,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_2;
    use webview2_com::GetCookiesCompletedHandler;
    use windows::core::{Interface, HSTRING};
    use wry::WebViewExtWindows;

    let core = webview.webview();
    let Ok(wv2) = core.cast::<ICoreWebView2_2>() else {
        return;
    };
    let Ok(mgr) = (unsafe { wv2.CookieManager() }) else {
        return;
    };
    let mgr_for_cb = mgr.clone();
    let uri_owned = uri.to_string();
    let uri_hs = HSTRING::from(uri);
    let handler = GetCookiesCompletedHandler::create(Box::new(move |hr, list_opt| {
        let mut deleted = 0usize;
        if hr.is_ok() {
            if let Some(list) = list_opt {
                unsafe {
                    let mut count: u32 = 0;
                    if list.Count(&mut count).is_ok() {
                        for idx in 0..count {
                            if let Ok(c) = list.GetValueAtIndex(idx) {
                                if mgr_for_cb.DeleteCookie(&c).is_ok() {
                                    deleted += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        let _ = proxy.send_event(UserEvent::SiteCookiesResult {
            win_id,
            req_id: 0,
            uri: uri_owned.clone(),
            cookies: Vec::new(),
        });
        let msg = if zh {
            format!("已清除当前站点 {} 条 Cookie 与本地存储", deleted)
        } else {
            format!("Cleared {} site cookies & local storage", deleted)
        };
        let _ = proxy.send_event(UserEvent::Toast(msg));
        Ok(())
    }));

    unsafe {
        let _ = mgr.GetCookies(&uri_hs, &handler);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn clear_origin_cookies(
    _webview: &wry::WebView,
    _uri: &str,
    _win_id: usize,
    _proxy: EventLoopProxy<UserEvent>,
    _zh: bool,
) {
}

/// Capture the active WebView viewport as a PNG file saved into `downloads_dir`.
#[cfg(target_os = "windows")]
pub fn capture_page_screenshot(
    webview: &wry::WebView,
    downloads_dir: std::path::PathBuf,
    page_url: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_PNG;
    use webview2_com::CapturePreviewCompletedHandler;
    use windows::Win32::System::Com::STREAM_SEEK_SET;
    use windows::Win32::UI::Shell::SHCreateMemStream;
    use wry::WebViewExtWindows;

    let Some(stream) = (unsafe { SHCreateMemStream(None) }) else {
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "截图失败：无法创建内存流".to_string()
        } else {
            "Screenshot failed: unable to allocate memory stream".to_string()
        }));
        return;
    };

    let stream_for_cb = stream.clone();
    let handler = CapturePreviewCompletedHandler::create(Box::new(move |hr| {
        if hr.is_err() {
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "网页截图失败".to_string()
            } else {
                "Page screenshot failed".to_string()
            }));
            return Ok(());
        }

        let mut png_bytes: Vec<u8> = Vec::with_capacity(256 * 1024);
        unsafe {
            let mut new_pos: u64 = 0;
            let _ = stream_for_cb.Seek(0, STREAM_SEEK_SET, Some(&mut new_pos));
            let mut buf = [0u8; 16384];
            loop {
                let mut read: u32 = 0;
                let _ = stream_for_cb.Read(
                    buf.as_mut_ptr().cast(),
                    buf.len() as u32,
                    Some(&mut read),
                );
                if read == 0 {
                    break;
                }
                png_bytes.extend_from_slice(&buf[..read as usize]);
                if png_bytes.len() > 32 * 1024 * 1024 {
                    break;
                }
            }
        }

        if png_bytes.is_empty() {
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "截图为空，请稍后重试".to_string()
            } else {
                "Screenshot was empty".to_string()
            }));
            return Ok(());
        }

        let ts = crate::config::now_unix_secs();
        let suggested = format!("fynexo-screenshot-{}.png", ts);
        let dest = crate::config::unique_download_path_in(&downloads_dir, &suggested);
        let fname = dest
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("fynexo-screenshot.png")
            .to_string();

        match std::fs::write(&dest, &png_bytes) {
            Ok(()) => {
                let sz = png_bytes.len() as u64;
                let _ = proxy.send_event(UserEvent::DownloadUpdate {
                    id: 0,
                    name: fname.clone(),
                    status: "completed".to_string(),
                    bytes_received: sz,
                    total_bytes: sz,
                    speed_bps: 0,
                    saved_path: Some(dest.to_string_lossy().to_string()),
                    url: Some(page_url.clone()),
                });
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    format!("已保存截图：{}", fname)
                } else {
                    format!("Saved screenshot: {}", fname)
                }));
            }
            Err(e) => {
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    format!("保存截图失败：{}", e)
                } else {
                    format!("Failed to save screenshot: {}", e)
                }));
            }
        }
        Ok(())
    }));

    let core = webview.webview();
    unsafe {
        let _ = core.CapturePreview(
            COREWEBVIEW2_CAPTURE_PREVIEW_IMAGE_FORMAT_PNG,
            &stream,
            &handler,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn capture_page_screenshot(
    _webview: &wry::WebView,
    _downloads_dir: std::path::PathBuf,
    _page_url: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    let _ = proxy.send_event(UserEvent::Toast(if zh {
        "当前平台请使用系统截屏功能".to_string()
    } else {
        "Use system screenshot on this platform".to_string()
    }));
}

#[cfg(any(target_os = "windows", test))]
fn sanitize_pdf_filename(title: &str) -> String {
    let cleaned: String = title
        .trim()
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .take(72)
        .collect();
    let trimmed = cleaned.trim_matches(|c| c == '.' || c == '_' || c == ' ');
    if trimmed.is_empty() {
        format!("fynexo-page-{}.pdf", crate::config::now_unix_secs())
    } else {
        format!("{}.pdf", trimmed)
    }
}

/// Export the active page as a clean PDF via WebView2 `ICoreWebView2_7::PrintToPdf`.
#[cfg(target_os = "windows")]
pub fn print_page_to_pdf(
    webview: &wry::WebView,
    downloads_dir: std::path::PathBuf,
    page_title: String,
    page_url: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_7;
    use webview2_com::PrintToPdfCompletedHandler;
    use windows::core::{Interface, HSTRING};
    use wry::WebViewExtWindows;

    let core = webview.webview();
    let Ok(wv7) = core.cast::<ICoreWebView2_7>() else {
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "当前 WebView2 版本不支持直接导出 PDF".to_string()
        } else {
            "PrintToPdf is unavailable on this WebView2 version".to_string()
        }));
        return;
    };

    let suggested = sanitize_pdf_filename(&page_title);
    let dest = crate::config::unique_download_path_in(&downloads_dir, &suggested);
    let dest_str = dest.to_string_lossy().to_string();
    let fname = dest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("page.pdf")
        .to_string();
    let path_hs = HSTRING::from(dest_str.as_str());

    let handler = PrintToPdfCompletedHandler::create(Box::new(move |hr, ok| {
        if hr.is_ok() && ok {
            let sz = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
            let _ = proxy.send_event(UserEvent::DownloadUpdate {
                id: 0,
                name: fname.clone(),
                status: "completed".to_string(),
                bytes_received: sz,
                total_bytes: sz,
                speed_bps: 0,
                saved_path: Some(dest_str.clone()),
                url: Some(page_url.clone()),
            });
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                format!("已导出 PDF：{}", fname)
            } else {
                format!("Saved PDF: {}", fname)
            }));
        } else {
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "导出 PDF 失败".to_string()
            } else {
                "Failed to export PDF".to_string()
            }));
        }
        Ok(())
    }));

    unsafe {
        let _ = wv7.PrintToPdf(&path_hs, None, &handler);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn print_page_to_pdf(
    _webview: &wry::WebView,
    _downloads_dir: std::path::PathBuf,
    _page_title: String,
    _page_url: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    let _ = proxy.send_event(UserEvent::Toast(if zh {
        "请使用打印菜单保存为 PDF".to_string()
    } else {
        "Use Print menu to save as PDF on this platform".to_string()
    }));
}

/// Capture a full-page scrolling PNG screenshot via WebView2 CDP `Page.captureScreenshot`.
#[cfg(target_os = "windows")]
pub fn capture_full_page_screenshot(
    webview: &wry::WebView,
    downloads_dir: std::path::PathBuf,
    page_url: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use windows::core::{w, HSTRING};
    use wry::WebViewExtWindows;

    let core = webview.webview();
    let params = HSTRING::from(r#"{"format":"png","captureBeyondViewport":true}"#);
    let _ = proxy.send_event(UserEvent::Toast(if zh {
        "正在生成整页长截图…".to_string()
    } else {
        "Capturing full-page screenshot...".to_string()
    }));

    let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |hr, json_str| {
        if hr.is_err() || json_str.is_empty() {
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                "整页截图失败".to_string()
            } else {
                "Full-page screenshot failed".to_string()
            }));
            return Ok(());
        }
        let dl_dir = downloads_dir.clone();
        let url_clone = page_url.clone();
        let proxy_bg = proxy.clone();
        std::thread::spawn(move || {
            let decoded = serde_json::from_str::<serde_json::Value>(&json_str)
                .ok()
                .and_then(|v| v["data"].as_str().and_then(decode_base64_bytes));
            let Some(bytes) = decoded.filter(|b| !b.is_empty()) else {
                let _ = proxy_bg.send_event(UserEvent::Toast(if zh {
                    "解析整页截图数据失败".to_string()
                } else {
                    "Failed to decode full-page screenshot".to_string()
                }));
                return;
            };
            let suggested = format!("fynexo-fullpage-{}.png", crate::config::now_unix_secs());
            let dest = crate::config::unique_download_path_in(&dl_dir, &suggested);
            let fname = dest
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("fullpage.png")
                .to_string();
            match std::fs::write(&dest, &bytes) {
                Ok(()) => {
                    let sz = bytes.len() as u64;
                    let _ = proxy_bg.send_event(UserEvent::DownloadUpdate {
                        id: 0,
                        name: fname.clone(),
                        status: "completed".to_string(),
                        bytes_received: sz,
                        total_bytes: sz,
                        speed_bps: 0,
                        saved_path: Some(dest.to_string_lossy().to_string()),
                        url: Some(url_clone),
                    });
                    let _ = proxy_bg.send_event(UserEvent::Toast(if zh {
                        format!("已保存整页长图：{}", fname)
                    } else {
                        format!("Saved full-page screenshot: {}", fname)
                    }));
                }
                Err(e) => {
                    let _ = proxy_bg.send_event(UserEvent::Toast(if zh {
                        format!("保存整页长图失败：{}", e)
                    } else {
                        format!("Failed to save full-page screenshot: {}", e)
                    }));
                }
            }
        });
        Ok(())
    }));

    unsafe {
        let _ = core.CallDevToolsProtocolMethod(w!("Page.captureScreenshot"), &params, &handler);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn capture_full_page_screenshot(
    _webview: &wry::WebView,
    _downloads_dir: std::path::PathBuf,
    _page_url: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    let _ = proxy.send_event(UserEvent::Toast(if zh {
        "整页长截图暂仅支持 Windows 桌面端".to_string()
    } else {
        "Full-page screenshot is currently Windows-only".to_string()
    }));
}

pub fn sanitize_markdown_filename(title: &str) -> String {
    let cleaned: String = title
        .trim()
        .chars()
        .map(|c| {
            if c.is_control()
                || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            {
                '-'
            } else {
                c
            }
        })
        .take(64)
        .collect();
    let trimmed = cleaned.trim().trim_matches(['.', '-', ' ']);
    if trimmed.is_empty() {
        format!("fynexo-article-{}.md", crate::config::now_unix_secs())
    } else {
        format!("{}.md", trimmed)
    }
}

/// Save extracted article Markdown (`.md`) into `downloads_dir` and record it in Downloads.
pub fn save_markdown_file(
    downloads_dir: std::path::PathBuf,
    title: String,
    page_url: String,
    markdown: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    if markdown.trim().is_empty() || markdown.len() > 4 * 1024 * 1024 {
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "无可导出的正文内容".to_string()
        } else {
            "No article content to export".to_string()
        }));
        return;
    }
    std::thread::spawn(move || {
        let suggested = sanitize_markdown_filename(&title);
        let dest = crate::config::unique_download_path_in(&downloads_dir, &suggested);
        let fname = dest
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("article.md")
            .to_string();
        match std::fs::write(&dest, markdown.as_bytes()) {
            Ok(()) => {
                let sz = markdown.len() as u64;
                let _ = proxy.send_event(UserEvent::DownloadUpdate {
                    id: 0,
                    name: fname.clone(),
                    status: "completed".to_string(),
                    bytes_received: sz,
                    total_bytes: sz,
                    speed_bps: 0,
                    saved_path: Some(dest.to_string_lossy().to_string()),
                    url: Some(page_url),
                });
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    format!("已导出 Markdown：{}", fname)
                } else {
                    format!("Exported Markdown: {}", fname)
                }));
            }
            Err(e) => {
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    format!("导出 Markdown 失败：{}", e)
                } else {
                    format!("Failed to export Markdown: {}", e)
                }));
            }
        }
    });
}

fn decode_base64_bytes(input: &str) -> Option<Vec<u8>> {
    fn val(b: u8) -> Option<u8> {
        match b {
            b'A'..=b'Z' => Some(b - b'A'),
            b'a'..=b'z' => Some(b - b'a' + 26),
            b'0'..=b'9' => Some(b - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let cleaned: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if cleaned.is_empty() || !cleaned.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity((cleaned.len() / 4) * 3);
    for chunk in cleaned.as_chunks::<4>().0 {
        let a = val(chunk[0])?;
        let b = val(chunk[1])?;
        out.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            let c = val(chunk[2])?;
            out.push((b << 4) | (c >> 2));
            if chunk[3] != b'=' {
                let d = val(chunk[3])?;
                out.push((c << 6) | d);
            }
        }
    }
    Some(out)
}

fn decode_data_image_uri(uri: &str) -> Option<(&'static str, Vec<u8>)> {
    let rest = uri.strip_prefix("data:image/")?;
    let (meta, payload) = rest.split_once(',')?;
    if !meta.to_ascii_lowercase().contains(";base64") {
        return None;
    }
    let subtype = meta.split(';').next().unwrap_or("png").to_ascii_lowercase();
    let ext = match subtype.as_str() {
        "jpeg" | "jpg" => "jpg",
        "webp" => "webp",
        "gif" => "gif",
        "svg+xml" => "svg",
        "bmp" => "bmp",
        "avif" => "avif",
        _ => "png",
    };
    let bytes = decode_base64_bytes(payload)?;
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 {
        return None;
    }
    Some((ext, bytes))
}

#[cfg(any(target_os = "windows", test))]
fn infer_image_filename(url: &str, content_type: &str) -> String {
    let base_no_q = url.split(['?', '#']).next().unwrap_or("");
    let seg = base_no_q.rsplit('/').next().unwrap_or("");
    let clean_seg: String = seg
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
        .take(64)
        .collect();
    let ext_from_ct = match content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        "image/svg+xml" => Some("svg"),
        "image/avif" => Some("avif"),
        "image/bmp" => Some("bmp"),
        _ => None,
    };
    if !clean_seg.is_empty() && clean_seg.contains('.') {
        clean_seg
    } else {
        let ext = ext_from_ct.unwrap_or("png");
        if clean_seg.is_empty() {
            format!("image-{}.{}", crate::config::now_unix_secs(), ext)
        } else {
            format!("{}.{}", clean_seg, ext)
        }
    }
}

/// Download an image (`http(s)://` or `data:image/...;base64,...`) directly into `downloads_dir`.
pub fn save_image_url(
    url: String,
    downloads_dir: std::path::PathBuf,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    std::thread::spawn(move || {
        let fetched: Result<(String, Vec<u8>), String> = if url.starts_with("data:image/") {
            decode_data_image_uri(&url)
                .map(|(ext, bytes)| (format!("image-{}.{}", crate::config::now_unix_secs(), ext), bytes))
                .ok_or_else(|| "Unsupported data image".to_string())
        } else if url.starts_with("http://") || url.starts_with("https://") {
            fetch_image_bytes(&url)
        } else {
            Err("Unsupported image URL".to_string())
        };

        match fetched {
            Ok((suggested, bytes)) => {
                let dest = crate::config::unique_download_path_in(&downloads_dir, &suggested);
                let fname = dest
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image.png")
                    .to_string();
                match std::fs::write(&dest, &bytes) {
                    Ok(()) => {
                        let sz = bytes.len() as u64;
                        let _ = proxy.send_event(UserEvent::DownloadUpdate {
                            id: 0,
                            name: fname.clone(),
                            status: "completed".to_string(),
                            bytes_received: sz,
                            total_bytes: sz,
                            speed_bps: 0,
                            saved_path: Some(dest.to_string_lossy().to_string()),
                            url: Some(url),
                        });
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("已保存图片：{}", fname)
                        } else {
                            format!("Saved image: {}", fname)
                        }));
                    }
                    Err(e) => {
                        let _ = proxy.send_event(UserEvent::Toast(if zh {
                            format!("保存图片失败：{}", e)
                        } else {
                            format!("Failed to save image: {}", e)
                        }));
                    }
                }
            }
            Err(e) => {
                let _ = proxy.send_event(UserEvent::Toast(if zh {
                    format!("下载图片失败：{}", e)
                } else {
                    format!("Failed to download image: {}", e)
                }));
            }
        }
    });
}

#[cfg(target_os = "windows")]
fn fetch_image_bytes(url: &str) -> Result<(String, Vec<u8>), String> {
    use core::ffi::c_void;
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
        InternetSetOptionW, HTTP_QUERY_CONTENT_TYPE, INTERNET_FLAG_NO_CACHE_WRITE,
        INTERNET_FLAG_RELOAD, INTERNET_OPEN_TYPE_PRECONFIG, INTERNET_OPTION_CONNECT_TIMEOUT,
        INTERNET_OPTION_RECEIVE_TIMEOUT,
    };

    unsafe {
        let h_net = InternetOpenW(
            w!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/134.0.0.0 Safari/537.36"),
            INTERNET_OPEN_TYPE_PRECONFIG.0,
            None,
            None,
            0,
        );
        if h_net.is_null() {
            return Err("WinInet unavailable".to_string());
        }
        let timeout_ms: u32 = 15_000;
        let ptr = (&timeout_ms as *const u32).cast::<c_void>();
        let len = std::mem::size_of::<u32>() as u32;
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_CONNECT_TIMEOUT, Some(ptr), len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_RECEIVE_TIMEOUT, Some(ptr), len);

        let url_hs = HSTRING::from(url);
        let h_url = InternetOpenUrlW(
            h_net,
            &url_hs,
            None,
            INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE,
            None,
        );
        if h_url.is_null() {
            let _ = InternetCloseHandle(h_net);
            return Err("Unable to open image URL".to_string());
        }

        let mut ct_buf = [0u16; 128];
        let mut ct_len = (ct_buf.len() * 2) as u32;
        let content_type = if HttpQueryInfoW(
            h_url,
            HTTP_QUERY_CONTENT_TYPE,
            Some(ct_buf.as_mut_ptr().cast::<c_void>()),
            &mut ct_len,
            None,
        )
        .is_ok()
        {
            String::from_utf16_lossy(&ct_buf[..(ct_len as usize / 2)])
        } else {
            String::new()
        };

        let mut bytes: Vec<u8> = Vec::with_capacity(64 * 1024);
        let mut chunk = [0u8; 16384];
        loop {
            let mut read: u32 = 0;
            if InternetReadFile(
                h_url,
                chunk.as_mut_ptr().cast::<c_void>(),
                chunk.len() as u32,
                &mut read,
            )
            .is_err()
                || read == 0
            {
                break;
            }
            bytes.extend_from_slice(&chunk[..read as usize]);
            if bytes.len() > 25 * 1024 * 1024 {
                break;
            }
        }
        let _ = InternetCloseHandle(h_url);
        let _ = InternetCloseHandle(h_net);

        if bytes.is_empty() {
            return Err("Empty image response".to_string());
        }
        Ok((infer_image_filename(url, &content_type), bytes))
    }
}

#[cfg(not(target_os = "windows"))]
fn fetch_image_bytes(_url: &str) -> Result<(String, Vec<u8>), String> {
    Err("Image download requires Windows desktop".to_string())
}

/// Clear WebView2 cookies and/or disk cache within a specific time range (`range_secs == 0` means all time).
#[cfg(target_os = "windows")]
pub fn clear_browsing_data_in_range(
    webview: &wry::WebView,
    clear_cookies: bool,
    clear_cache: bool,
    range_secs: u64,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Profile2, ICoreWebView2_13, COREWEBVIEW2_BROWSING_DATA_KINDS,
        COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_DOM_STORAGE, COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES,
        COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE,
    };
    use webview2_com::ClearBrowsingDataCompletedHandler;
    use windows::core::Interface;
    use wry::WebViewExtWindows;

    if !clear_cookies && !clear_cache {
        return;
    }

    let mut kinds_bits: i32 = 0;
    if clear_cookies {
        kinds_bits |= COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES.0
            | COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_DOM_STORAGE.0;
    }
    if clear_cache {
        kinds_bits |= COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE.0;
    }
    let kinds = COREWEBVIEW2_BROWSING_DATA_KINDS(kinds_bits);

    let now_f = crate::config::now_unix_secs() as f64;
    let start_f = if range_secs == 0 {
        0.0
    } else {
        (now_f - range_secs as f64).max(0.0)
    };
    let end_f = now_f + 3600.0;

    let core = webview.webview();
    let handled = (|| -> Option<()> {
        let wv13 = core.cast::<ICoreWebView2_13>().ok()?;
        let prof = unsafe { wv13.Profile().ok()? };
        let prof2 = prof.cast::<ICoreWebView2Profile2>().ok()?;
        let proxy_cb = proxy.clone();
        let handler = ClearBrowsingDataCompletedHandler::create(Box::new(move |hr| {
            if hr.is_ok() {
                let _ = proxy_cb.send_event(UserEvent::Toast(if zh {
                    "已按选定范围清理浏览数据".to_string()
                } else {
                    "Cleared selected browsing data".to_string()
                }));
            }
            Ok(())
        }));
        unsafe {
            prof2
                .ClearBrowsingDataInTimeRange(kinds, start_f, end_f, &handler)
                .ok()?;
        }
        Some(())
    })()
    .is_some();

    if !handled {
        let _ = webview.clear_all_browsing_data();
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "已清理浏览数据".to_string()
        } else {
            "Cleared browsing data".to_string()
        }));
    }
}

#[cfg(not(target_os = "windows"))]
pub fn clear_browsing_data_in_range(
    webview: &wry::WebView,
    clear_cookies: bool,
    clear_cache: bool,
    _range_secs: u64,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    if clear_cookies || clear_cache {
        let _ = webview.clear_all_browsing_data();
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "已清理浏览数据".to_string()
        } else {
            "Cleared browsing data".to_string()
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_pdf_filename_strips_invalid_chars() {
        assert_eq!(sanitize_pdf_filename("Rust: Book / Ch?1"), "Rust_ Book _ Ch_1.pdf");
        assert!(sanitize_pdf_filename("   ").starts_with("fynexo-page-"));
    }

    #[test]
    fn sanitize_markdown_filename_strips_invalid_chars() {
        assert_eq!(
            sanitize_markdown_filename("Guide: Async/Await? *2026*"),
            "Guide- Async-Await- -2026.md"
        );
        assert!(sanitize_markdown_filename("   ").starts_with("fynexo-article-"));
    }

    #[test]
    fn decode_data_image_uri_parses_base64_png() {
        let (ext, bytes) =
            decode_data_image_uri("data:image/png;base64,iVBORw0K").expect("valid data uri");
        assert_eq!(ext, "png");
        assert_eq!(bytes, vec![0x89, b'P', b'N', b'G', b'\r', b'\n']);
        assert!(decode_data_image_uri("data:text/plain;base64,SGVsbG8=").is_none());
    }

    #[test]
    fn infer_image_filename_uses_path_or_content_type() {
        assert_eq!(
            infer_image_filename("https://example.com/assets/hero.webp?v=2", "image/webp"),
            "hero.webp"
        );
        let fallback = infer_image_filename("https://example.com/img?id=9", "image/jpeg");
        assert!(fallback.ends_with(".jpg"));
    }
}
