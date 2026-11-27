#[cfg(target_os = "windows")]
use std::cell::RefCell;
#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "windows")]
use std::time::Instant;

#[cfg(target_os = "windows")]
use tao::event_loop::EventLoopProxy;
#[cfg(target_os = "windows")]
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Deferral, ICoreWebView2DownloadOperation, ICoreWebView2DownloadStartingEventArgs,
    ICoreWebView2_2, ICoreWebView2_4, COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_CANCELED,
    COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_PAUSED, COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED,
    COREWEBVIEW2_DOWNLOAD_STATE_IN_PROGRESS, COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED,
    COREWEBVIEW2_PERMISSION_KIND_CAMERA, COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ,
    COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
    COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS, COREWEBVIEW2_PERMISSION_STATE_ALLOW,
    COREWEBVIEW2_PERMISSION_STATE_DEFAULT, COREWEBVIEW2_PERMISSION_STATE_DENY,
};
#[cfg(target_os = "windows")]
use webview2_com::{
    take_pwstr, BytesReceivedChangedEventHandler, ContainsFullScreenElementChangedEventHandler,
    DownloadStartingEventHandler, GetCookiesCompletedHandler, PermissionRequestedEventHandler,
    StateChangedEventHandler,
};
#[cfg(target_os = "windows")]
use windows::core::{Interface, HSTRING, PWSTR};
#[cfg(target_os = "windows")]
use wry::WebViewExtWindows;

#[cfg(target_os = "windows")]
use crate::config::{resolved_downloads_dir, unique_download_path_in};
#[cfg(target_os = "windows")]
use crate::util::LockExt;

#[cfg(target_os = "windows")]
use super::events::UserEvent;
#[cfg(target_os = "windows")]
use super::state::AppState;

static NEXT_DOWNLOAD_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub fn next_download_id() -> u64 {
    NEXT_DOWNLOAD_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[cfg(target_os = "windows")]
static NEXT_PROBE_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(target_os = "windows")]
struct PendingDownloadProbe {
    args: ICoreWebView2DownloadStartingEventArgs,
    deferral: ICoreWebView2Deferral,
    op: ICoreWebView2DownloadOperation,
    dest_path: PathBuf,
    display_name: String,
    saved_path_str: String,
    source_url: String,
    total_u64: u64,
    threads: u8,
}

#[cfg(target_os = "windows")]
thread_local! {
    static ACTIVE_DOWNLOADS: RefCell<HashMap<u64, ICoreWebView2DownloadOperation>> =
        RefCell::new(HashMap::new());
    static PENDING_PROBES: RefCell<HashMap<u64, PendingDownloadProbe>> =
        RefCell::new(HashMap::new());
}

/// Cancel an active download (multi-threaded or WebView2 native) by ID.
#[cfg(target_os = "windows")]
pub fn cancel_download(id: u64) -> bool {
    if super::dl_accel::cancel_accel(id) {
        return true;
    }
    ACTIVE_DOWNLOADS.with(|map| {
        if let Some(op) = map.borrow().get(&id) {
            unsafe { op.Cancel().is_ok() }
        } else {
            false
        }
    })
}

/// Pause an active download (multi-threaded or WebView2 native) by ID.
#[cfg(target_os = "windows")]
pub fn pause_download(id: u64) -> bool {
    if super::dl_accel::pause_accel(id) {
        return true;
    }
    ACTIVE_DOWNLOADS.with(|map| {
        if let Some(op) = map.borrow().get(&id) {
            unsafe { op.Pause().is_ok() }
        } else {
            false
        }
    })
}

/// Resume a paused download (multi-threaded or WebView2 native) by ID.
#[cfg(target_os = "windows")]
pub fn resume_download(id: u64) -> bool {
    if super::dl_accel::resume_accel(id) {
        return true;
    }
    ACTIVE_DOWNLOADS.with(|map| {
        if let Some(op) = map.borrow().get(&id) {
            unsafe { op.Resume().is_ok() }
        } else {
            false
        }
    })
}

#[cfg(not(target_os = "windows"))]
pub fn cancel_download(_id: u64) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn pause_download(_id: u64) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn resume_download(_id: u64) -> bool {
    false
}

/// Open the native Windows Save As dialog (`GetSaveFileNameW`) pre-filled with `initial_dir` & `suggested_name`.
#[cfg(target_os = "windows")]
fn show_windows_save_dialog(
    owner_hwnd: isize,
    initial_dir: &Path,
    suggested_name: &str,
) -> Option<PathBuf> {
    #[repr(C)]
    struct OpenFileNameW {
        l_struct_size: u32,
        hwnd_owner: isize,
        h_instance: isize,
        lpstr_filter: *const u16,
        lpstr_custom_filter: *mut u16,
        n_max_cust_filter: u32,
        n_filter_index: u32,
        lpstr_file: *mut u16,
        n_max_file: u32,
        lpstr_file_title: *mut u16,
        n_max_file_title: u32,
        lpstr_initial_dir: *const u16,
        lpstr_title: *const u16,
        flags: u32,
        n_file_offset: u16,
        n_file_extension: u16,
        lpstr_def_ext: *const u16,
        l_cust_data: isize,
        lpfn_hook: isize,
        lp_template_name: *const u16,
        pv_reserved: *mut core::ffi::c_void,
        dw_reserved: u32,
        flags_ex: u32,
    }

    #[link(name = "comdlg32")]
    extern "system" {
        fn GetSaveFileNameW(lpofn: *mut OpenFileNameW) -> i32;
    }

    const OFN_OVERWRITEPROMPT: u32 = 0x0000_0002;
    const OFN_NOCHANGEDIR: u32 = 0x0000_0008;
    const OFN_PATHMUSTEXIST: u32 = 0x0000_0800;

    let mut file_buf = vec![0u16; 4096];
    let safe_name: Vec<u16> = suggested_name.encode_utf16().collect();
    let copy_len = safe_name.len().min(file_buf.len() - 1);
    file_buf[..copy_len].copy_from_slice(&safe_name[..copy_len]);

    let init_dir_w: Vec<u16> = initial_dir
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let filter_w: Vec<u16> = "All Files (*.*)\0*.*\0\0".encode_utf16().collect();
    let title_w: Vec<u16> = "Save File As — Fynexo\0".encode_utf16().collect();

    let mut ofn = OpenFileNameW {
        l_struct_size: std::mem::size_of::<OpenFileNameW>() as u32,
        hwnd_owner: owner_hwnd,
        h_instance: 0,
        lpstr_filter: filter_w.as_ptr(),
        lpstr_custom_filter: std::ptr::null_mut(),
        n_max_cust_filter: 0,
        n_filter_index: 1,
        lpstr_file: file_buf.as_mut_ptr(),
        n_max_file: file_buf.len() as u32,
        lpstr_file_title: std::ptr::null_mut(),
        n_max_file_title: 0,
        lpstr_initial_dir: init_dir_w.as_ptr(),
        lpstr_title: title_w.as_ptr(),
        flags: OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
        n_file_offset: 0,
        n_file_extension: 0,
        lpstr_def_ext: std::ptr::null(),
        l_cust_data: 0,
        lpfn_hook: 0,
        lp_template_name: std::ptr::null(),
        pv_reserved: std::ptr::null_mut(),
        dw_reserved: 0,
        flags_ex: 0,
    };

    let ok = unsafe { GetSaveFileNameW(&mut ofn) != 0 };
    if !ok {
        return None;
    }
    let nul = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
    let chosen = String::from_utf16_lossy(&file_buf[..nul]);
    if chosen.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(chosen))
    }
}

#[cfg(target_os = "windows")]
fn attach_webview_op_handlers(
    op: ICoreWebView2DownloadOperation,
    dl_id: u64,
    display_name: String,
    saved_path_str: String,
    source_url: String,
    total_u64: u64,
    proxy: EventLoopProxy<UserEvent>,
) {
    ACTIVE_DOWNLOADS.with(|map| {
        map.borrow_mut().insert(dl_id, op.clone());
    });

    let _ = proxy.send_event(UserEvent::DownloadUpdate {
        id: dl_id,
        name: display_name.clone(),
        status: "downloading".to_string(),
        bytes_received: 0,
        total_bytes: total_u64,
        speed_bps: 0,
        saved_path: Some(saved_path_str.clone()),
        url: if source_url.is_empty() {
            None
        } else {
            Some(source_url.clone())
        },
    });

    let last_tick = Arc::new(Mutex::new((Instant::now(), 0u64)));
    let name_for_prog = display_name.clone();
    let proxy_for_prog = proxy.clone();
    let mut prog_tok: i64 = 0;
    unsafe {
        let _ = op.add_BytesReceivedChanged(
            &BytesReceivedChangedEventHandler::create(Box::new(move |sender, _| {
                let Some(op_ref) = sender else {
                    return Ok(());
                };
                let mut recv: i64 = 0;
                let mut total: i64 = 0;
                let _ = op_ref.BytesReceived(&mut recv);
                let _ = op_ref.TotalBytesToReceive(&mut total);
                let recv_u64 = recv.max(0) as u64;
                let total_u64 = total.max(0) as u64;

                let speed_bps = {
                    let mut guard = last_tick.lock_safe();
                    let elapsed = guard.0.elapsed();
                    if elapsed.as_millis() < 200 && (total_u64 == 0 || recv_u64 < total_u64) {
                        return Ok(());
                    }
                    let delta_bytes = recv_u64.saturating_sub(guard.1);
                    let secs = elapsed.as_secs_f64().max(0.05);
                    let spd = (delta_bytes as f64 / secs).round() as u64;
                    *guard = (Instant::now(), recv_u64);
                    spd
                };

                let _ = proxy_for_prog.send_event(UserEvent::DownloadUpdate {
                    id: dl_id,
                    name: name_for_prog.clone(),
                    status: "downloading".to_string(),
                    bytes_received: recv_u64,
                    total_bytes: total_u64,
                    speed_bps,
                    saved_path: None,
                    url: None,
                });
                Ok(())
            })),
            &mut prog_tok,
        );
    }

    let name_for_state = display_name;
    let saved_for_state = saved_path_str;
    let url_for_state = source_url;
    let proxy_for_state = proxy;
    let mut state_tok: i64 = 0;
    unsafe {
        let _ = op.add_StateChanged(
            &StateChangedEventHandler::create(Box::new(move |sender, _| {
                let Some(op_ref) = sender else {
                    return Ok(());
                };
                let mut state = Default::default();
                if op_ref.State(&mut state).is_err() {
                    return Ok(());
                }
                let mut recv: i64 = 0;
                let mut total: i64 = 0;
                let _ = op_ref.BytesReceived(&mut recv);
                let _ = op_ref.TotalBytesToReceive(&mut total);
                let recv_u64 = recv.max(0) as u64;
                let total_u64 = total.max(0) as u64;

                if state == COREWEBVIEW2_DOWNLOAD_STATE_IN_PROGRESS {
                    let _ = proxy_for_state.send_event(UserEvent::DownloadUpdate {
                        id: dl_id,
                        name: name_for_state.clone(),
                        status: "downloading".to_string(),
                        bytes_received: recv_u64,
                        total_bytes: total_u64,
                        speed_bps: 0,
                        saved_path: None,
                        url: None,
                    });
                } else if state == COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED {
                    ACTIVE_DOWNLOADS.with(|map| {
                        map.borrow_mut().remove(&dl_id);
                    });
                    let _ = proxy_for_state.send_event(UserEvent::DownloadUpdate {
                        id: dl_id,
                        name: name_for_state.clone(),
                        status: "completed".to_string(),
                        bytes_received: recv_u64.max(total_u64),
                        total_bytes: total_u64.max(recv_u64),
                        speed_bps: 0,
                        saved_path: Some(saved_for_state.clone()),
                        url: if url_for_state.is_empty() {
                            None
                        } else {
                            Some(url_for_state.clone())
                        },
                    });
                } else if state == COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED {
                    let mut reason = Default::default();
                    let _ = op_ref.InterruptReason(&mut reason);
                    if reason == COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_PAUSED {
                        let _ = proxy_for_state.send_event(UserEvent::DownloadUpdate {
                            id: dl_id,
                            name: name_for_state.clone(),
                            status: "paused".to_string(),
                            bytes_received: recv_u64,
                            total_bytes: total_u64,
                            speed_bps: 0,
                            saved_path: None,
                            url: None,
                        });
                    } else {
                        ACTIVE_DOWNLOADS.with(|map| {
                            map.borrow_mut().remove(&dl_id);
                        });
                        let status_str =
                            if reason == COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_CANCELED {
                                "cancelled"
                            } else {
                                "failed"
                            };
                        let _ = proxy_for_state.send_event(UserEvent::DownloadUpdate {
                            id: dl_id,
                            name: name_for_state.clone(),
                            status: status_str.to_string(),
                            bytes_received: recv_u64,
                            total_bytes: total_u64,
                            speed_bps: 0,
                            saved_path: None,
                            url: if url_for_state.is_empty() {
                                None
                            } else {
                                Some(url_for_state.clone())
                            },
                        });
                    }
                }
                Ok(())
            })),
            &mut state_tok,
        );
    }
}

/// Complete a deferred `DownloadStarting` Range probe on the Windows UI thread.
#[cfg(target_os = "windows")]
pub fn handle_download_probe_finished(
    probe_id: u64,
    supports_range: bool,
    total_bytes: u64,
    cookie_header: String,
    proxy: EventLoopProxy<UserEvent>,
    zh: bool,
) {
    let Some(p) = PENDING_PROBES.with(|map| map.borrow_mut().remove(&probe_id)) else {
        return;
    };
    let dl_id = next_download_id();
    if supports_range && total_bytes >= super::dl_accel::MIN_ACCEL_BYTES && p.threads >= 2 {
        unsafe {
            let _ = p.args.SetCancel(true);
            let _ = p.deferral.Complete();
        }
        super::dl_accel::spawn_accelerated_download(
            dl_id,
            p.source_url,
            p.dest_path,
            total_bytes,
            p.threads,
            cookie_header,
            proxy,
            zh,
        );
    } else {
        attach_webview_op_handlers(
            p.op,
            dl_id,
            p.display_name,
            p.saved_path_str,
            p.source_url,
            p.total_u64,
            proxy,
        );
        unsafe {
            let _ = p.deferral.Complete();
        }
    }
}

/// Attach Windows WebView2 `ICoreWebView2_4` download tracking with real-time progress,
/// multi-threaded Range acceleration, speed calculation, pause/resume/cancel, and Save As dialog.
#[cfg(target_os = "windows")]
pub fn attach_windows_downloads(
    webview: &wry::WebView,
    app_state: Arc<Mutex<AppState>>,
    proxy: EventLoopProxy<UserEvent>,
) {
    let core_wv = webview.webview();
    let Ok(wv4) = core_wv.cast::<ICoreWebView2_4>() else {
        return;
    };
    let cookie_mgr_opt = core_wv
        .cast::<ICoreWebView2_2>()
        .ok()
        .and_then(|wv2| unsafe { wv2.CookieManager().ok() });
    let owner_hwnd: isize = {
        use windows::Win32::UI::WindowsAndMessaging::{GetAncestor, GA_ROOT};
        let raw = webview.hwnd();
        let root = unsafe { GetAncestor(raw, GA_ROOT) };
        if !root.is_invalid() {
            root.0 as isize
        } else {
            raw.0 as isize
        }
    };

    unsafe {
        let mut token: i64 = 0;
        let _ = wv4.add_DownloadStarting(
            &DownloadStartingEventHandler::create(Box::new(move |_sender, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                let _ = args.SetHandled(true);

                let mut default_path_pwstr = PWSTR::null();
                let default_path_str = if args.ResultFilePath(&mut default_path_pwstr).is_ok() {
                    take_pwstr(default_path_pwstr)
                } else {
                    String::new()
                };
                let suggested = Path::new(&default_path_str)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("download.bin")
                    .to_string();

                let (custom_dir, ask_location, parallel_on, dl_threads) = {
                    let st = app_state.lock_safe();
                    (
                        st.config.downloads_dir.clone(),
                        st.config.ask_download_location,
                        st.config.parallel_downloads,
                        st.config.download_threads.clamp(1, 16),
                    )
                };
                let base_dir = resolved_downloads_dir(&custom_dir);

                let final_path = if ask_location {
                    match show_windows_save_dialog(owner_hwnd, &base_dir, &suggested) {
                        Some(p) => p,
                        None => {
                            let _ = args.SetCancel(true);
                            return Ok(());
                        }
                    }
                } else {
                    unique_download_path_in(&base_dir, &suggested)
                };

                let display_name = final_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("file")
                    .to_string();

                let saved_path_str = final_path.to_string_lossy().to_string();
                let path_h = HSTRING::from(saved_path_str.as_str());
                let _ = args.SetResultFilePath(&path_h);

                let Ok(op) = args.DownloadOperation() else {
                    return Ok(());
                };

                let mut init_total: i64 = 0;
                let _ = op.TotalBytesToReceive(&mut init_total);
                let total_u64 = if init_total > 0 { init_total as u64 } else { 0 };
                let mut uri_pwstr = PWSTR::null();
                let source_url = if op.Uri(&mut uri_pwstr).is_ok() {
                    take_pwstr(uri_pwstr)
                } else {
                    String::new()
                };

                let can_try_accel = parallel_on
                    && dl_threads >= 2
                    && (source_url.starts_with("http://") || source_url.starts_with("https://"))
                    && (total_u64 == 0 || total_u64 >= super::dl_accel::MIN_ACCEL_BYTES);

                if can_try_accel {
                    if let Ok(deferral) = args.GetDeferral() {
                        let probe_id = NEXT_PROBE_ID.fetch_add(1, Ordering::Relaxed);
                        let url_for_probe = source_url.clone();
                        PENDING_PROBES.with(|map| {
                            map.borrow_mut().insert(
                                probe_id,
                                PendingDownloadProbe {
                                    args: args.clone(),
                                    deferral,
                                    op,
                                    dest_path: final_path,
                                    display_name,
                                    saved_path_str,
                                    source_url,
                                    total_u64,
                                    threads: dl_threads,
                                },
                            );
                        });
                        let proxy_for_probe = proxy.clone();
                        if let Some(ref mgr) = cookie_mgr_opt {
                            let uri_hs = HSTRING::from(url_for_probe.as_str());
                            let handler =
                                GetCookiesCompletedHandler::create(Box::new(move |hr, list_opt| {
                                    let mut pairs: Vec<String> = Vec::new();
                                    if hr.is_ok() {
                                        if let Some(list) = list_opt {
                                            let mut count: u32 = 0;
                                            if list.Count(&mut count).is_ok() {
                                                for i in 0..count.min(64) {
                                                    if let Ok(c) = list.GetValueAtIndex(i) {
                                                        let mut np = PWSTR::null();
                                                        let mut vp = PWSTR::null();
                                                        let n = if c.Name(&mut np).is_ok() {
                                                            take_pwstr(np)
                                                        } else {
                                                            String::new()
                                                        };
                                                        let v = if c.Value(&mut vp).is_ok() {
                                                            take_pwstr(vp)
                                                        } else {
                                                            String::new()
                                                        };
                                                        if !n.is_empty() {
                                                            pairs.push(format!("{}={}", n, v));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let ck_hdr = pairs.join("; ");
                                    let u = url_for_probe.clone();
                                    let pr = proxy_for_probe.clone();
                                    std::thread::spawn(move || {
                                        let total_opt =
                                            super::dl_accel::probe_range_support(&u, &ck_hdr);
                                        let _ = pr.send_event(UserEvent::DownloadProbeFinished {
                                            probe_id,
                                            supports_range: total_opt.is_some(),
                                            total_bytes: total_opt.unwrap_or(0),
                                            cookie_header: ck_hdr,
                                        });
                                    });
                                    Ok(())
                                }));
                            let _ = mgr.GetCookies(&uri_hs, &handler);
                        } else {
                            std::thread::spawn(move || {
                                let total_opt =
                                    super::dl_accel::probe_range_support(&url_for_probe, "");
                                let _ = proxy_for_probe.send_event(
                                    UserEvent::DownloadProbeFinished {
                                        probe_id,
                                        supports_range: total_opt.is_some(),
                                        total_bytes: total_opt.unwrap_or(0),
                                        cookie_header: String::new(),
                                    },
                                );
                            });
                        }
                        return Ok(());
                    }
                }

                let dl_id = next_download_id();
                attach_webview_op_handlers(
                    op,
                    dl_id,
                    display_name,
                    saved_path_str,
                    source_url,
                    total_u64,
                    proxy.clone(),
                );
                Ok(())
            })),
            &mut token,
        );
    }
}

/// Attach Windows WebView2 `PermissionRequested` handler honoring Fynexo's persistent per-site rules.
#[cfg(target_os = "windows")]
pub fn attach_windows_permissions(webview: &wry::WebView, app_state: Arc<Mutex<AppState>>) {
    let core_wv = webview.webview();
    unsafe {
        let mut token: i64 = 0;
        let _ = core_wv.add_PermissionRequested(
            &PermissionRequestedEventHandler::create(Box::new(move |_sender, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                let mut kind = Default::default();
                if args.PermissionKind(&mut kind).is_err() {
                    return Ok(());
                }
                let kind_str = if kind == COREWEBVIEW2_PERMISSION_KIND_CAMERA {
                    "camera"
                } else if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE {
                    "microphone"
                } else if kind == COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION {
                    "geolocation"
                } else if kind == COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS {
                    "notifications"
                } else if kind == COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ {
                    "clipboard"
                } else {
                    return Ok(());
                };

                let mut uri_pwstr = PWSTR::null();
                if args.Uri(&mut uri_pwstr).is_err() {
                    return Ok(());
                }
                let uri = take_pwstr(uri_pwstr);
                let decision = {
                    let st = app_state.lock_safe();
                    st.site_permissions.get(&uri, kind_str).map(|s| s.to_string())
                };
                match decision.as_deref() {
                    Some("allow") => {
                        let _ = args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW);
                    }
                    Some("deny") => {
                        let _ = args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY);
                    }
                    _ => {
                        let _ = args.SetState(COREWEBVIEW2_PERMISSION_STATE_DEFAULT);
                    }
                }
                Ok(())
            })),
            &mut token,
        );
    }
}

/// Sync HTML5 element fullscreen (e.g., YouTube/Bilibili video player fullscreen)
/// with the native browser window fullscreen state so the HUD topbar and OS taskbar hide cleanly.
#[cfg(target_os = "windows")]
pub fn attach_windows_fullscreen(
    webview: &wry::WebView,
    win_id: usize,
    proxy: EventLoopProxy<UserEvent>,
) {
    let core_wv = webview.webview();
    unsafe {
        let mut token: i64 = 0;
        let _ = core_wv.add_ContainsFullScreenElementChanged(
            &ContainsFullScreenElementChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(wv_ref) = sender {
                    let mut is_full = windows::core::BOOL(0);
                    if wv_ref.ContainsFullScreenElement(&mut is_full).is_ok() {
                        let _ = proxy.send_event(UserEvent::ToggleFullscreen {
                            win_id,
                            explicit: Some(is_full.as_bool()),
                        });
                    }
                }
                Ok(())
            })),
            &mut token,
        );
    }
}
