//! Native multi-threaded HTTP Range segmented download accelerator (IDM-style).
//!
//! Automatically probes `Range: bytes=0-0` (forwarding WebView2 cookies when needed).
//! If the server responds `206 Partial Content` with a valid total size >= 2 MiB,
//! Fynexo splits the file into `N` (2..=16) parallel WinInet byte-range workers
//! writing lock-free at exact offsets, with full Pause / Resume / Cancel support.

#[cfg(target_os = "windows")]
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex, OnceLock};
use tao::event_loop::EventLoopProxy;

use super::events::UserEvent;

#[cfg(target_os = "windows")]
pub const MIN_ACCEL_BYTES: u64 = 2 * 1024 * 1024; // 2 MiB threshold
#[cfg(any(target_os = "windows", test))]
const MIN_CHUNK_BYTES: u64 = 512 * 1024; // 512 KiB minimum per thread

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct AccelTaskControl {
    pub paused: AtomicBool,
    pub cancelled: AtomicBool,
    pub bytes_received: AtomicU64,
}

#[cfg(target_os = "windows")]
impl AccelTaskControl {
    fn new() -> Self {
        Self {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            bytes_received: AtomicU64::new(0),
        }
    }
}

#[cfg(target_os = "windows")]
fn active_map() -> &'static Mutex<HashMap<u64, Arc<AccelTaskControl>>> {
    static MAP: OnceLock<Mutex<HashMap<u64, Arc<AccelTaskControl>>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(target_os = "windows")]
pub fn pause_accel(id: u64) -> bool {
    if let Ok(guard) = active_map().lock() {
        if let Some(ctrl) = guard.get(&id) {
            ctrl.paused.store(true, Ordering::Release);
            return true;
        }
    }
    false
}

#[cfg(target_os = "windows")]
pub fn resume_accel(id: u64) -> bool {
    if let Ok(guard) = active_map().lock() {
        if let Some(ctrl) = guard.get(&id) {
            ctrl.paused.store(false, Ordering::Release);
            return true;
        }
    }
    false
}

#[cfg(target_os = "windows")]
pub fn cancel_accel(id: u64) -> bool {
    if let Ok(guard) = active_map().lock() {
        if let Some(ctrl) = guard.get(&id) {
            ctrl.cancelled.store(true, Ordering::Release);
            ctrl.paused.store(false, Ordering::Release);
            return true;
        }
    }
    false
}

/// Parse total file length from an HTTP `Content-Range` header such as `bytes 0-0/10485760`.
#[cfg(any(target_os = "windows", test))]
pub fn parse_content_range_total(header_val: &str) -> Option<u64> {
    let trimmed = header_val.trim();
    let (_, total_str) = trimmed.rsplit_once('/')?;
    let t = total_str.trim();
    if t == "*" {
        return None;
    }
    t.parse::<u64>().ok().filter(|&n| n > 0)
}

/// Split `[0, total_bytes)` into `1..=16` inclusive byte ranges `(start, end)`.
#[cfg(any(target_os = "windows", test))]
pub fn compute_segments(total_bytes: u64, requested_threads: u8) -> Vec<(u64, u64)> {
    if total_bytes == 0 {
        return Vec::new();
    }
    let max_by_size = (total_bytes / MIN_CHUNK_BYTES).clamp(1, 16) as u8;
    let n = requested_threads.clamp(1, 16).min(max_by_size).max(1) as u64;
    let base = total_bytes / n;
    let mut out = Vec::with_capacity(n as usize);
    let mut offset = 0u64;
    for i in 0..n {
        let end = if i + 1 == n {
            total_bytes - 1
        } else {
            offset + base - 1
        };
        out.push((offset, end));
        offset = end + 1;
    }
    out
}

#[cfg(any(target_os = "windows", test))]
fn infer_filename_from_url(url: &str) -> String {
    let no_q = url.split(['?', '#']).next().unwrap_or("");
    let raw = no_q.rsplit('/').next().unwrap_or("");
    let cleaned: String = raw
        .chars()
        .filter(|c| {
            !c.is_control()
                && !matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
        })
        .take(96)
        .collect();
    let t = cleaned.trim_matches(|c| c == '.' || c == ' ');
    if t.is_empty() {
        format!("download-{}.bin", crate::config::now_unix_secs())
    } else {
        t.to_string()
    }
}

/// Probe whether `url` supports HTTP `Range` requests (`206 Partial Content`) and return exact total bytes.
#[cfg(target_os = "windows")]
pub fn probe_range_support(url: &str, cookie_header: &str) -> Option<u64> {
    use core::ffi::c_void;
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetSetOptionW,
        HTTP_QUERY_CUSTOM, HTTP_QUERY_FLAG_NUMBER, HTTP_QUERY_STATUS_CODE,
        INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_NO_COOKIES, INTERNET_FLAG_RELOAD,
        INTERNET_OPEN_TYPE_PRECONFIG, INTERNET_OPTION_CONNECT_TIMEOUT,
        INTERNET_OPTION_RECEIVE_TIMEOUT,
    };

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }

    unsafe {
        let h_net = InternetOpenW(
            w!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"),
            INTERNET_OPEN_TYPE_PRECONFIG.0,
            None,
            None,
            0,
        );
        if h_net.is_null() {
            return None;
        }
        let timeout_ms: u32 = 3000;
        let ptr = (&timeout_ms as *const u32).cast::<c_void>();
        let len = std::mem::size_of::<u32>() as u32;
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_CONNECT_TIMEOUT, Some(ptr), len);
        let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_RECEIVE_TIMEOUT, Some(ptr), len);

        let mut hdrs = String::from("Range: bytes=0-0\r\n");
        if !cookie_header.is_empty() {
            hdrs.push_str("Cookie: ");
            hdrs.push_str(cookie_header);
            hdrs.push_str("\r\n");
        }
        let hdr_u16: Vec<u16> = hdrs.encode_utf16().collect();
        let url_hs = HSTRING::from(url);
        let h_url = InternetOpenUrlW(
            h_net,
            &url_hs,
            Some(hdr_u16.as_slice()),
            INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE | INTERNET_FLAG_NO_COOKIES,
            None,
        );
        if h_url.is_null() {
            let _ = InternetCloseHandle(h_net);
            return None;
        }

        let mut status: u32 = 0;
        let mut status_len = std::mem::size_of::<u32>() as u32;
        let ok_status = HttpQueryInfoW(
            h_url,
            HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
            Some((&mut status as *mut u32).cast::<c_void>()),
            &mut status_len,
            None,
        )
        .is_ok();

        let mut total_opt = None;
        if ok_status && status == 206 {
            let mut buf = [0u16; 128];
            let name: Vec<u16> = "Content-Range\0".encode_utf16().collect();
            buf[..name.len()].copy_from_slice(&name);
            let mut buf_len = (buf.len() * 2) as u32;
            if HttpQueryInfoW(
                h_url,
                HTTP_QUERY_CUSTOM,
                Some(buf.as_mut_ptr().cast::<c_void>()),
                &mut buf_len,
                None,
            )
            .is_ok()
            {
                let cr = String::from_utf16_lossy(&buf[..(buf_len as usize / 2)]);
                total_opt = parse_content_range_total(&cr);
            }
        }

        let _ = InternetCloseHandle(h_url);
        let _ = InternetCloseHandle(h_net);
        total_opt
    }
}

#[cfg(target_os = "windows")]
fn ensure_wininet_high_concurrency() {
    static CONFIGURED: OnceLock<()> = OnceLock::new();
    CONFIGURED.get_or_init(|| {
        use core::ffi::c_void;
        use windows::Win32::Networking::WinInet::{
            InternetSetOptionW, INTERNET_OPTION_MAX_CONNS_PER_1_0_SERVER,
            INTERNET_OPTION_MAX_CONNS_PER_SERVER,
        };
        let max_conns: u32 = 32;
        let ptr = (&max_conns as *const u32).cast::<c_void>();
        let len = std::mem::size_of::<u32>() as u32;
        unsafe {
            let _ = InternetSetOptionW(None, INTERNET_OPTION_MAX_CONNS_PER_SERVER, Some(ptr), len);
            let _ = InternetSetOptionW(
                None,
                INTERNET_OPTION_MAX_CONNS_PER_1_0_SERVER,
                Some(ptr),
                len,
            );
        }
    });
}

#[cfg(target_os = "windows")]
fn download_segment_worker(
    url: &str,
    cookie_header: &str,
    part_path: &PathBuf,
    range_start: u64,
    range_end: u64,
    ctrl: &Arc<AccelTaskControl>,
    failed_flag: &Arc<AtomicBool>,
) -> Result<(), String> {
    use core::ffi::c_void;
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
        InternetSetOptionW, HTTP_QUERY_FLAG_NUMBER, HTTP_QUERY_STATUS_CODE,
        INTERNET_FLAG_KEEP_CONNECTION, INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_NO_COOKIES,
        INTERNET_FLAG_RELOAD, INTERNET_OPEN_TYPE_PRECONFIG, INTERNET_OPTION_CONNECT_TIMEOUT,
        INTERNET_OPTION_RECEIVE_TIMEOUT,
    };

    ensure_wininet_high_concurrency();
    let mut file = OpenOptions::new()
        .write(true)
        .open(part_path)
        .map_err(|e| format!("open part file: {}", e))?;

    let mut cur = range_start;
    let mut retries = 0u32;

    while cur <= range_end {
        if ctrl.cancelled.load(Ordering::Acquire) || failed_flag.load(Ordering::Acquire) {
            return Err("cancelled".to_string());
        }
        while ctrl.paused.load(Ordering::Acquire) {
            if ctrl.cancelled.load(Ordering::Acquire) || failed_flag.load(Ordering::Acquire) {
                return Err("cancelled".to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(120));
        }

        let seg_res: Result<(), String> = unsafe {
            let h_net = InternetOpenW(
                w!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"),
                INTERNET_OPEN_TYPE_PRECONFIG.0,
                None,
                None,
                0,
            );
            if h_net.is_null() {
                return Err("WinInet unavailable".to_string());
            }
            let timeout_ms: u32 = 20_000;
            let ptr = (&timeout_ms as *const u32).cast::<c_void>();
            let len = std::mem::size_of::<u32>() as u32;
            let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_CONNECT_TIMEOUT, Some(ptr), len);
            let _ = InternetSetOptionW(Some(h_net), INTERNET_OPTION_RECEIVE_TIMEOUT, Some(ptr), len);

            let mut hdrs = format!("Range: bytes={}-{}\r\n", cur, range_end);
            if !cookie_header.is_empty() {
                hdrs.push_str("Cookie: ");
                hdrs.push_str(cookie_header);
                hdrs.push_str("\r\n");
            }
            let hdr_u16: Vec<u16> = hdrs.encode_utf16().collect();
            let url_hs = HSTRING::from(url);
            let h_url = InternetOpenUrlW(
                h_net,
                &url_hs,
                Some(hdr_u16.as_slice()),
                INTERNET_FLAG_RELOAD
                    | INTERNET_FLAG_NO_CACHE_WRITE
                    | INTERNET_FLAG_NO_COOKIES
                    | INTERNET_FLAG_KEEP_CONNECTION,
                None,
            );
            if h_url.is_null() {
                let _ = InternetCloseHandle(h_net);
                Err("connect failed".to_string())
            } else {
                let mut status: u32 = 0;
                let mut status_len = std::mem::size_of::<u32>() as u32;
                let st_ok = HttpQueryInfoW(
                    h_url,
                    HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
                    Some((&mut status as *mut u32).cast::<c_void>()),
                    &mut status_len,
                    None,
                )
                .is_ok();

                let mut inner_res = Ok(());
                if !st_ok || (status != 206 && !(cur == 0 && status == 200)) {
                    inner_res = Err(format!("HTTP {}", status));
                } else {
                    if let Err(e) = file.seek(SeekFrom::Start(cur)) {
                        inner_res = Err(format!("seek: {}", e));
                    } else {
                        let mut buf = vec![0u8; 65536];
                        while cur <= range_end {
                            if ctrl.cancelled.load(Ordering::Acquire)
                                || failed_flag.load(Ordering::Acquire)
                            {
                                inner_res = Err("cancelled".to_string());
                                break;
                            }
                            if ctrl.paused.load(Ordering::Acquire) {
                                // Break cleanly to close socket; outer loop sleeps & resumes at `cur`.
                                break;
                            }
                            let remaining = (range_end - cur + 1) as usize;
                            let to_read = buf.len().min(remaining) as u32;
                            let mut read: u32 = 0;
                            if InternetReadFile(
                                h_url,
                                buf.as_mut_ptr().cast::<c_void>(),
                                to_read,
                                &mut read,
                            )
                            .is_err()
                            {
                                inner_res = Err("read error".to_string());
                                break;
                            }
                            if read == 0 {
                                break;
                            }
                            let n = read as usize;
                            if let Err(e) = file.write_all(&buf[..n]) {
                                inner_res = Err(format!("write error: {}", e));
                                break;
                            }
                            cur += n as u64;
                            ctrl.bytes_received.fetch_add(n as u64, Ordering::Relaxed);
                        }
                    }
                }
                let _ = InternetCloseHandle(h_url);
                let _ = InternetCloseHandle(h_net);
                inner_res
            }
        };

        if ctrl.cancelled.load(Ordering::Acquire) {
            return Err("cancelled".to_string());
        }
        if ctrl.paused.load(Ordering::Acquire) {
            continue;
        }
        match seg_res {
            Ok(()) => {
                if cur <= range_end {
                    retries += 1;
                    if retries > 3 {
                        return Err("premature EOF".to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(350));
                }
            }
            Err(e) => {
                retries += 1;
                if retries > 3 {
                    return Err(e);
                }
                std::thread::sleep(std::time::Duration::from_millis(450));
            }
        }
    }

    let _ = file.flush();
    Ok(())
}

/// Launch a multi-threaded segmented download into `dest_path`, emitting live `DownloadUpdate` events.
#[cfg(target_os = "windows")]
#[allow(clippy::too_many_arguments)]
pub fn spawn_accelerated_download(
    dl_id: u64,
    url: String,
    dest_path: PathBuf,
    total_bytes: u64,
    threads: u8,
    cookie_header: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    let display_name = dest_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download.bin")
        .to_string();
    let saved_path_str = dest_path.to_string_lossy().to_string();
    let segments = compute_segments(total_bytes, threads);
    let thread_count = segments.len();

    let ctrl = Arc::new(AccelTaskControl::new());
    if let Ok(mut guard) = active_map().lock() {
        guard.insert(dl_id, Arc::clone(&ctrl));
    }

    let _ = proxy.send_event(UserEvent::DownloadUpdate {
        id: dl_id,
        name: display_name.clone(),
        status: "downloading".to_string(),
        bytes_received: 0,
        total_bytes,
        speed_bps: 0,
        saved_path: Some(saved_path_str.clone()),
        url: Some(url.clone()),
    });
    let _ = proxy.send_event(UserEvent::Toast(if zh {
        format!("已启用 {} 线程分片加速下载：{}", thread_count, display_name)
    } else {
        format!("Turbo {}-thread download started: {}", thread_count, display_name)
    }));

    std::thread::spawn(move || {
        let mut part_os = dest_path.as_os_str().to_os_string();
        part_os.push(".fnxpart");
        let part_path = PathBuf::from(part_os);

        let prep = (|| -> Result<(), String> {
            if let Some(parent) = dest_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let f = std::fs::File::create(&part_path).map_err(|e| e.to_string())?;
            f.set_len(total_bytes).map_err(|e| e.to_string())?;
            Ok(())
        })();

        if let Err(e) = prep {
            if let Ok(mut g) = active_map().lock() {
                g.remove(&dl_id);
            }
            let _ = std::fs::remove_file(&part_path);
            let _ = proxy.send_event(UserEvent::DownloadUpdate {
                id: dl_id,
                name: display_name,
                status: "failed".to_string(),
                bytes_received: 0,
                total_bytes,
                speed_bps: 0,
                saved_path: None,
                url: Some(url),
            });
            let _ = proxy.send_event(UserEvent::Toast(if zh {
                format!("创建下载文件失败：{}", e)
            } else {
                format!("Download file error: {}", e)
            }));
            return;
        }

        let failed_flag = Arc::new(AtomicBool::new(false));
        let done_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(segments.len());

        for (seg_start, seg_end) in segments {
            let u = url.clone();
            let ck = cookie_header.clone();
            let pp = part_path.clone();
            let c = Arc::clone(&ctrl);
            let ff = Arc::clone(&failed_flag);
            let dc = Arc::clone(&done_count);
            handles.push(std::thread::spawn(move || {
                if download_segment_worker(&u, &ck, &pp, seg_start, seg_end, &c, &ff).is_err() {
                    ff.store(true, Ordering::Release);
                }
                dc.fetch_add(1, Ordering::Release);
            }));
        }

        // Progress ticker loop until all workers finish
        let mut last_bytes = 0u64;
        let mut last_tick = std::time::Instant::now();
        let mut was_paused = false;

        while done_count.load(Ordering::Acquire) < thread_count {
            std::thread::sleep(std::time::Duration::from_millis(240));
            let is_paused = ctrl.paused.load(Ordering::Acquire);
            let recv = ctrl.bytes_received.load(Ordering::Relaxed);
            let elapsed = last_tick.elapsed().as_secs_f64().max(0.05);
            let delta = recv.saturating_sub(last_bytes);
            let speed_bps = if is_paused {
                0
            } else {
                (delta as f64 / elapsed).round() as u64
            };
            last_bytes = recv;
            last_tick = std::time::Instant::now();

            if is_paused != was_paused || !is_paused {
                was_paused = is_paused;
                let _ = proxy.send_event(UserEvent::DownloadUpdate {
                    id: dl_id,
                    name: display_name.clone(),
                    status: if is_paused {
                        "paused".to_string()
                    } else {
                        "downloading".to_string()
                    },
                    bytes_received: recv.min(total_bytes),
                    total_bytes,
                    speed_bps,
                    saved_path: None,
                    url: None,
                });
            }
        }

        for h in handles {
            let _ = h.join();
        }
        if let Ok(mut g) = active_map().lock() {
            g.remove(&dl_id);
        }

        let cancelled = ctrl.cancelled.load(Ordering::Acquire);
        let failed = failed_flag.load(Ordering::Acquire);
        let final_recv = ctrl.bytes_received.load(Ordering::Relaxed);

        if cancelled {
            let _ = std::fs::remove_file(&part_path);
            let _ = proxy.send_event(UserEvent::DownloadUpdate {
                id: dl_id,
                name: display_name,
                status: "cancelled".to_string(),
                bytes_received: final_recv,
                total_bytes,
                speed_bps: 0,
                saved_path: None,
                url: Some(url),
            });
        } else if failed || final_recv < total_bytes {
            let _ = std::fs::remove_file(&part_path);
            let _ = proxy.send_event(UserEvent::DownloadUpdate {
                id: dl_id,
                name: display_name,
                status: "failed".to_string(),
                bytes_received: final_recv,
                total_bytes,
                speed_bps: 0,
                saved_path: None,
                url: Some(url),
            });
        } else {
            let _ = std::fs::remove_file(&dest_path);
            if std::fs::rename(&part_path, &dest_path).is_ok() {
                let _ = proxy.send_event(UserEvent::DownloadUpdate {
                    id: dl_id,
                    name: display_name,
                    status: "completed".to_string(),
                    bytes_received: total_bytes,
                    total_bytes,
                    speed_bps: 0,
                    saved_path: Some(saved_path_str),
                    url: Some(url),
                });
            } else {
                let _ = std::fs::remove_file(&part_path);
                let _ = proxy.send_event(UserEvent::DownloadUpdate {
                    id: dl_id,
                    name: display_name,
                    status: "failed".to_string(),
                    bytes_received: final_recv,
                    total_bytes,
                    speed_bps: 0,
                    saved_path: None,
                    url: Some(url),
                });
            }
        }
    });
}

/// Manual "+ Turbo Download" launcher from `fynexo://downloads`: probes Range support
/// and either launches N-thread segmented download or streams cleanly in 1 thread.
#[cfg(target_os = "windows")]
pub fn start_manual_turbo_download(
    dl_id: u64,
    url: String,
    downloads_dir: PathBuf,
    threads: u8,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    std::thread::spawn(move || {
        let suggested = infer_filename_from_url(&url);
        let dest = crate::config::unique_download_path_in(&downloads_dir, &suggested);
        if let Some(total) = probe_range_support(&url, "") {
            if total >= MIN_ACCEL_BYTES && threads >= 2 {
                spawn_accelerated_download(
                    dl_id,
                    url,
                    dest,
                    total,
                    threads,
                    String::new(),
                    proxy,
                    zh,
                );
                return;
            }
        }
        // Fallback: single-stream direct download
        single_stream_download(dl_id, url, dest, proxy, zh);
    });
}

#[cfg(target_os = "windows")]
fn single_stream_download(
    dl_id: u64,
    url: String,
    dest_path: PathBuf,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    use core::ffi::c_void;
    use std::io::Write;
    use windows::core::{w, HSTRING};
    use windows::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
        HTTP_QUERY_CONTENT_LENGTH, HTTP_QUERY_FLAG_NUMBER, HTTP_QUERY_STATUS_CODE,
        INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_RELOAD, INTERNET_OPEN_TYPE_PRECONFIG,
    };

    let display_name = dest_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download.bin")
        .to_string();
    let saved_str = dest_path.to_string_lossy().to_string();

    let ctrl = Arc::new(AccelTaskControl::new());
    if let Ok(mut g) = active_map().lock() {
        g.insert(dl_id, Arc::clone(&ctrl));
    }

    let _ = proxy.send_event(UserEvent::DownloadUpdate {
        id: dl_id,
        name: display_name.clone(),
        status: "downloading".to_string(),
        bytes_received: 0,
        total_bytes: 0,
        speed_bps: 0,
        saved_path: Some(saved_str.clone()),
        url: Some(url.clone()),
    });

    let mut total_bytes: u64 = 0;
    let mut recv: u64 = 0;
    let mut ok = false;

    unsafe {
        let h_net = InternetOpenW(
            w!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/134.0.0.0 Safari/537.36"),
            INTERNET_OPEN_TYPE_PRECONFIG.0,
            None,
            None,
            0,
        );
        if !h_net.is_null() {
            let url_hs = HSTRING::from(url.as_str());
            let h_url = InternetOpenUrlW(
                h_net,
                &url_hs,
                None,
                INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE,
                None,
            );
            if !h_url.is_null() {
                let mut st: u32 = 0;
                let mut st_len = std::mem::size_of::<u32>() as u32;
                let _ = HttpQueryInfoW(
                    h_url,
                    HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
                    Some((&mut st as *mut u32).cast::<c_void>()),
                    &mut st_len,
                    None,
                );
                let mut cl_buf = [0u16; 32];
                let mut cl_len = (cl_buf.len() * 2) as u32;
                if HttpQueryInfoW(
                    h_url,
                    HTTP_QUERY_CONTENT_LENGTH,
                    Some(cl_buf.as_mut_ptr().cast::<c_void>()),
                    &mut cl_len,
                    None,
                )
                .is_ok()
                {
                    let cl_str = String::from_utf16_lossy(&cl_buf[..(cl_len as usize / 2)]);
                    if let Ok(parsed) = cl_str.trim().parse::<u64>() {
                        total_bytes = parsed;
                    }
                }

                if (200..300).contains(&st) {
                    if let Ok(mut file) = std::fs::File::create(&dest_path) {
                        let mut buf = vec![0u8; 32768];
                        let mut last_tick = std::time::Instant::now();
                        let mut last_b = 0u64;
                        ok = true;
                        loop {
                            if ctrl.cancelled.load(Ordering::Acquire) {
                                ok = false;
                                break;
                            }
                            let mut n: u32 = 0;
                            if InternetReadFile(
                                h_url,
                                buf.as_mut_ptr().cast::<c_void>(),
                                buf.len() as u32,
                                &mut n,
                            )
                            .is_err()
                            {
                                ok = false;
                                break;
                            }
                            if n == 0 {
                                break;
                            }
                            if file.write_all(&buf[..n as usize]).is_err() {
                                ok = false;
                                break;
                            }
                            recv += n as u64;
                            if last_tick.elapsed().as_millis() >= 250 {
                                let spd = ((recv - last_b) as f64
                                    / last_tick.elapsed().as_secs_f64().max(0.05))
                                .round() as u64;
                                last_b = recv;
                                last_tick = std::time::Instant::now();
                                let _ = proxy.send_event(UserEvent::DownloadUpdate {
                                    id: dl_id,
                                    name: display_name.clone(),
                                    status: "downloading".to_string(),
                                    bytes_received: recv,
                                    total_bytes,
                                    speed_bps: spd,
                                    saved_path: None,
                                    url: None,
                                });
                            }
                        }
                    }
                }
                let _ = InternetCloseHandle(h_url);
            }
            let _ = InternetCloseHandle(h_net);
        }
    }

    if let Ok(mut g) = active_map().lock() {
        g.remove(&dl_id);
    }

    if ctrl.cancelled.load(Ordering::Acquire) {
        let _ = std::fs::remove_file(&dest_path);
        let _ = proxy.send_event(UserEvent::DownloadUpdate {
            id: dl_id,
            name: display_name,
            status: "cancelled".to_string(),
            bytes_received: recv,
            total_bytes,
            speed_bps: 0,
            saved_path: None,
            url: Some(url),
        });
    } else if ok {
        let _ = proxy.send_event(UserEvent::DownloadUpdate {
            id: dl_id,
            name: display_name,
            status: "completed".to_string(),
            bytes_received: recv,
            total_bytes: total_bytes.max(recv),
            speed_bps: 0,
            saved_path: Some(saved_str),
            url: Some(url),
        });
    } else {
        let _ = std::fs::remove_file(&dest_path);
        let _ = proxy.send_event(UserEvent::DownloadUpdate {
            id: dl_id,
            name: display_name,
            status: "failed".to_string(),
            bytes_received: recv,
            total_bytes,
            speed_bps: 0,
            saved_path: None,
            url: Some(url),
        });
        let _ = proxy.send_event(UserEvent::Toast(if zh {
            "下载失败，请检查链接是否可直接访问".to_string()
        } else {
            "Download failed — check that the URL is directly accessible".to_string()
        }));
    }
}

#[cfg(not(target_os = "windows"))]
pub fn start_manual_turbo_download(
    _dl_id: u64,
    _url: String,
    _downloads_dir: PathBuf,
    _threads: u8,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    let _ = proxy.send_event(UserEvent::Toast(if zh {
        "多线程分片下载加速目前仅支持 Windows 桌面端".to_string()
    } else {
        "Multi-threaded turbo download is supported on Windows desktop".to_string()
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_content_range_total_extracts_exact_size() {
        assert_eq!(parse_content_range_total("bytes 0-0/10485760"), Some(10_485_760));
        assert_eq!(parse_content_range_total("bytes 0-1023/*"), None);
        assert_eq!(parse_content_range_total("invalid"), None);
    }

    #[test]
    fn compute_segments_covers_entire_file_without_gaps_or_overlap() {
        let total = 10_000_003u64;
        let segs = compute_segments(total, 8);
        assert_eq!(segs.len(), 8);
        assert_eq!(segs.first().unwrap().0, 0);
        assert_eq!(segs.last().unwrap().1, total - 1);
        for w in segs.windows(2) {
            assert_eq!(w[0].1 + 1, w[1].0);
        }
    }

    #[test]
    fn compute_segments_clamps_small_files_gracefully() {
        let segs = compute_segments(600_000, 16);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], (0, 599_999));
    }

    #[test]
    fn infer_filename_from_url_strips_query_and_invalid_chars() {
        assert_eq!(
            infer_filename_from_url("https://cdn.example.com/files/setup-v2.exe?token=abc"),
            "setup-v2.exe"
        );
        assert!(infer_filename_from_url("https://example.com/").starts_with("download-"));
    }
}
