//! OS integration: per-user Windows Default Browser registration (HKCU, zero UAC),
//! per-profile single-instance Named Pipe hand-off, local HTML/PDF CLI URL resolution,
//! and procedural branded window icon generation.

use std::path::Path;
use tao::event_loop::EventLoopProxy;

use super::events::UserEvent;
use super::security;
use super::url::normalize_logical_url;

/// True when `url` is a local `file://` document URL pointing to a safe static document (`.htm`, `.html`, `.pdf`).
pub fn is_safe_local_document_file_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    let Some(rest) = lower.strip_prefix("file://") else {
        return false;
    };
    let clean = rest.split(['?', '#']).next().unwrap_or("");
    clean.ends_with(".html") || clean.ends_with(".htm") || clean.ends_with(".pdf")
}

/// Resolve a CLI argument (HTTP/HTTPS/fynexo:// URL or existing local `.htm`/`.html`/`.pdf` file path)
/// into a launchable URL string.
pub fn parse_launch_target(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() {
        return None;
    }
    if security::is_safe_navigation_target(trimmed) {
        return Some(normalize_logical_url(trimmed));
    }
    if is_safe_local_document_file_url(trimmed) {
        return Some(trimmed.to_string());
    }
    let path = Path::new(trimmed);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if matches!(ext.as_str(), "htm" | "html" | "pdf") && path.is_file() {
        let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let mut s = abs.to_string_lossy().replace('\\', "/");
        if let Some(stripped) = s.strip_prefix("//?/") {
            s = stripped.to_string();
        }
        let encoded = s
            .replace('%', "%25")
            .replace(' ', "%20")
            .replace('#', "%23")
            .replace('?', "%3F");
        let file_url = if encoded.starts_with('/') {
            format!("file://{}", encoded)
        } else {
            format!("file:///{}", encoded)
        };
        return Some(file_url);
    }
    None
}

#[cfg(target_os = "windows")]
fn single_instance_pipe_name(profile_id: &str) -> String {
    let clean: String = profile_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .take(32)
        .collect();
    let suffix = if clean.is_empty() { "default" } else { &clean };
    format!(r"\\.\pipe\FynexoSingleInstance_{}", suffix)
}

/// If another Fynexo instance for the same profile is already running, forward the optional
/// launch URL (or focus ping) over the Named Pipe and return `true` so this process can exit cleanly.
#[cfg(target_os = "windows")]
pub fn try_forward_to_running_instance(profile_id: &str, url: Option<&str>) -> bool {
    use std::io::Write;
    let pipe = single_instance_pipe_name(profile_id);
    if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open(&pipe) {
        let payload = format!("{}\n", url.unwrap_or("__FOCUS__"));
        let _ = f.write_all(payload.as_bytes());
        let _ = f.flush();
        true
    } else {
        false
    }
}

#[cfg(not(target_os = "windows"))]
pub fn try_forward_to_running_instance(_profile_id: &str, _url: Option<&str>) -> bool {
    false
}

/// Spawn a lightweight background Named Pipe server thread that forwards external URL clicks
/// into this running Fynexo process as `UserEvent::ExternalLaunch`.
#[cfg(target_os = "windows")]
pub fn start_single_instance_listener(profile_id: &str, proxy: EventLoopProxy<UserEvent>) {
    let pipe_name = single_instance_pipe_name(profile_id);
    let _ = std::thread::Builder::new()
        .name("fynexo-single-instance".to_string())
        .spawn(move || {
            #[link(name = "kernel32")]
            extern "system" {
                fn CreateNamedPipeW(
                    lp_name: *const u16,
                    dw_open_mode: u32,
                    dw_pipe_mode: u32,
                    n_max_instances: u32,
                    n_out_buffer_size: u32,
                    n_in_buffer_size: u32,
                    n_default_time_out: u32,
                    lp_security_attributes: *mut core::ffi::c_void,
                ) -> isize;
                fn ConnectNamedPipe(
                    h_named_pipe: isize,
                    lp_overlapped: *mut core::ffi::c_void,
                ) -> i32;
                fn ReadFile(
                    h_file: isize,
                    lp_buffer: *mut u8,
                    n_number_of_bytes_to_read: u32,
                    lp_number_of_bytes_read: *mut u32,
                    lp_overlapped: *mut core::ffi::c_void,
                ) -> i32;
                fn DisconnectNamedPipe(h_named_pipe: isize) -> i32;
                fn CloseHandle(h_object: isize) -> i32;
                fn GetLastError() -> u32;
            }

            const INVALID_HANDLE_VALUE: isize = -1;
            const PIPE_ACCESS_INBOUND: u32 = 0x0000_0001;
            const PIPE_TYPE_BYTE: u32 = 0x0000_0000;
            const PIPE_READMODE_BYTE: u32 = 0x0000_0000;
            const PIPE_WAIT: u32 = 0x0000_0000;
            const ERROR_PIPE_CONNECTED: u32 = 535;

            let name_w: Vec<u16> = pipe_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            loop {
                let handle = unsafe {
                    CreateNamedPipeW(
                        name_w.as_ptr(),
                        PIPE_ACCESS_INBOUND,
                        PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                        8,
                        0,
                        4096,
                        0,
                        std::ptr::null_mut(),
                    )
                };
                if handle == INVALID_HANDLE_VALUE {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    continue;
                }

                let connected = unsafe {
                    ConnectNamedPipe(handle, std::ptr::null_mut()) != 0
                        || GetLastError() == ERROR_PIPE_CONNECTED
                };
                if connected {
                    let mut buf = [0u8; 4096];
                    let mut bytes_read: u32 = 0;
                    let ok = unsafe {
                        ReadFile(
                            handle,
                            buf.as_mut_ptr(),
                            buf.len() as u32,
                            &mut bytes_read,
                            std::ptr::null_mut(),
                        ) != 0
                    };
                    if ok && bytes_read > 0 {
                        let raw = String::from_utf8_lossy(&buf[..bytes_read as usize]);
                        let line = raw.lines().next().unwrap_or("").trim();
                        if line == "__FOCUS__" || line.is_empty() {
                            let _ = proxy.send_event(UserEvent::ExternalLaunch { url: None });
                        } else if let Some(target) = parse_launch_target(line) {
                            let _ = proxy.send_event(UserEvent::ExternalLaunch {
                                url: Some(target),
                            });
                        }
                    }
                    unsafe {
                        DisconnectNamedPipe(handle);
                    }
                }
                unsafe {
                    CloseHandle(handle);
                }
            }
        });
}

#[cfg(not(target_os = "windows"))]
pub fn start_single_instance_listener(_profile_id: &str, _proxy: EventLoopProxy<UserEvent>) {}

/// Procedurally render a crisp 64×64 anti-aliased RGBA Fynexo brand icon
/// (dark obsidian squircle badge with Fynexo's signature indigo-violet 4-point spark emblem).
pub fn render_app_icon_rgba(size: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let inv = 1.0 / (size as f32);

    let sample = |px: f32, py: f32| -> [f32; 4] {
        // Normalized coords in [-1, 1]
        let nx = px * inv * 2.0 - 1.0;
        let ny = py * inv * 2.0 - 1.0;
        let ax = nx.abs();
        let ay = ny.abs();

        // Rounded-square SDF (squircle-like box radius)
        let r = 0.28f32;
        let b = 0.90 - r;
        let qx = (ax - b).max(0.0);
        let qy = (ay - b).max(0.0);
        let sdf = (qx * qx + qy * qy).sqrt() - r;
        if sdf > 0.02 {
            return [0.0, 0.0, 0.0, 0.0];
        }
        let alpha = ((-sdf + 0.02) / 0.04).clamp(0.0, 1.0);

        // Background vertical gradient: obsidian (#151824) -> deep slate (#0B0D14)
        let t_bg = ((ny + 1.0) * 0.5).clamp(0.0, 1.0);
        let mut cr = 23.0 + (11.0 - 23.0) * t_bg;
        let mut cg = 26.0 + (13.0 - 26.0) * t_bg;
        let mut cb = 38.0 + (20.0 - 38.0) * t_bg;

        // Subtle radial glow behind the center spark
        let dist2 = nx * nx + ny * ny;
        let glow = (-dist2 * 3.8).exp() * 0.28;
        cr += 124.0 * glow;
        cg += 134.0 * glow;
        cb += 255.0 * glow;

        // Crisp subtle rim highlight inside border
        if sdf > -0.055 {
            let rim = 0.34;
            cr = cr * (1.0 - rim) + 92.0 * rim;
            cg = cg * (1.0 - rim) + 104.0 * rim;
            cb = cb * (1.0 - rim) + 158.0 * rim;
        }

        // 4-point geometric spark emblem: (|x|/s)^0.58 + (|y|/s)^0.58 <= 1
        let s = 0.64f32;
        let star_val = (ax / s).powf(0.58) + (ay / s).powf(0.58);
        if star_val < 1.04 {
            let star_a = ((1.04 - star_val) / 0.08).clamp(0.0, 1.0);
            let t_st = ((nx + ny + 1.2) / 2.4).clamp(0.0, 1.0);
            // Indigo (#7C86FF) -> Soft Periwinkle (#C4B5FD)
            let sr = 124.0 + (196.0 - 124.0) * t_st;
            let sg = 134.0 + (181.0 - 134.0) * t_st;
            let sb = 255.0 + (253.0 - 255.0) * t_st;
            cr = cr * (1.0 - star_a) + sr * star_a;
            cg = cg * (1.0 - star_a) + sg * star_a;
            cb = cb * (1.0 - star_a) + sb * star_a;
        }

        [
            cr.clamp(0.0, 255.0),
            cg.clamp(0.0, 255.0),
            cb.clamp(0.0, 255.0),
            alpha * 255.0,
        ]
    };

    for y in 0..size {
        for x in 0..size {
            // 2x2 supersampling for crisp edges
            let mut acc = [0.0f32; 4];
            for sy in [0.25f32, 0.75f32] {
                for sx in [0.25f32, 0.75f32] {
                    let c = sample(x as f32 + sx, y as f32 + sy);
                    for i in 0..4 {
                        acc[i] += c[i] * 0.25;
                    }
                }
            }
            let idx = ((y * size + x) * 4) as usize;
            rgba[idx] = acc[0].round() as u8;
            rgba[idx + 1] = acc[1].round() as u8;
            rgba[idx + 2] = acc[2].round() as u8;
            rgba[idx + 3] = acc[3].round() as u8;
        }
    }
    rgba
}

pub fn create_app_window_icon() -> Option<tao::window::Icon> {
    let size = 64u32;
    let rgba = render_app_icon_rgba(size);
    tao::window::Icon::from_rgba(rgba, size, size).ok()
}

#[cfg(target_os = "windows")]
fn set_reg_sz(subkey: &str, value_name: Option<&str>, data: &str) -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE,
        REG_OPTION_NON_VOLATILE, REG_SZ,
    };

    let subkey_h = HSTRING::from(subkey);
    let mut hkey = HKEY::default();
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            &subkey_h,
            Some(0),
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )
    };
    if status.is_err() {
        return Err(format!("RegCreateKeyExW failed for {}", subkey));
    }

    let utf16: Vec<u16> = data.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = unsafe {
        std::slice::from_raw_parts(utf16.as_ptr() as *const u8, utf16.len() * 2)
    };
    let val_h = value_name.map(HSTRING::from);
    let set_status = unsafe {
        match &val_h {
            Some(name) => RegSetValueExW(hkey, name, Some(0), REG_SZ, Some(bytes)),
            None => RegSetValueExW(hkey, None, Some(0), REG_SZ, Some(bytes)),
        }
    };
    unsafe {
        let _ = RegCloseKey(hkey);
    }
    if set_status.is_err() {
        return Err(format!("RegSetValueExW failed for {}", subkey));
    }
    Ok(())
}

/// Register Fynexo as a per-user Windows browser (`HKCU`, no administrator privileges required)
/// and launch Windows Settings -> Default Apps so the user can select Fynexo for HTTP/HTTPS.
#[cfg(target_os = "windows")]
pub fn register_default_browser_and_open_settings() -> Result<(), String> {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe().map_err(|e| format!("Cannot resolve exe path: {}", e))?;
    let exe_str = exe.to_string_lossy().to_string();
    let open_cmd = format!("\"{}\" \"%1\"", exe_str);
    let launch_cmd = format!("\"{}\"", exe_str);
    let icon_val = format!("{},0", exe_str);

    // 1. ProgID for FynexoHTML
    set_reg_sz(r"Software\Classes\FynexoHTML", None, "Fynexo HTML Document")?;
    set_reg_sz(r"Software\Classes\FynexoHTML", Some("URL Protocol"), "")?;
    set_reg_sz(r"Software\Classes\FynexoHTML\DefaultIcon", None, &icon_val)?;
    set_reg_sz(r"Software\Classes\FynexoHTML\shell\open\command", None, &open_cmd)?;

    // 2. StartMenuInternet browser capabilities
    set_reg_sz(r"Software\Clients\StartMenuInternet\Fynexo", None, "Fynexo")?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\DefaultIcon",
        None,
        &icon_val,
    )?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\shell\open\command",
        None,
        &launch_cmd,
    )?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\Capabilities",
        Some("ApplicationName"),
        "Fynexo",
    )?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\Capabilities",
        Some("ApplicationDescription"),
        "Fynexo Studio Browser",
    )?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\Capabilities\URLAssociations",
        Some("http"),
        "FynexoHTML",
    )?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\Capabilities\URLAssociations",
        Some("https"),
        "FynexoHTML",
    )?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\Capabilities\FileAssociations",
        Some(".htm"),
        "FynexoHTML",
    )?;
    set_reg_sz(
        r"Software\Clients\StartMenuInternet\Fynexo\Capabilities\FileAssociations",
        Some(".html"),
        "FynexoHTML",
    )?;

    // 3. RegisteredApplications entry
    set_reg_sz(
        r"Software\RegisteredApplications",
        Some("Fynexo"),
        r"Software\Clients\StartMenuInternet\Fynexo\Capabilities",
    )?;

    // 4. Open Windows Settings -> Default Apps
    let verb = HSTRING::from("open");
    let target = HSTRING::from("ms-settings:defaultapps");
    unsafe {
        ShellExecuteW(None, &verb, &target, None, None, SW_SHOWNORMAL);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn register_default_browser_and_open_settings() -> Result<(), String> {
    Err("Default browser registration is currently supported on Windows.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_local_document_urls_only_allow_html_and_pdf() {
        assert!(is_safe_local_document_file_url("file:///C:/docs/index.html"));
        assert!(is_safe_local_document_file_url("file:///C:/docs/guide.htm#top"));
        assert!(is_safe_local_document_file_url("file:///C:/docs/manual.pdf?page=2"));
        assert!(!is_safe_local_document_file_url("file:///C:/Windows/win.ini"));
        assert!(!is_safe_local_document_file_url("file:///C:/Users/me/secret.txt"));
        assert!(!is_safe_local_document_file_url("https://example.com/a.html"));
    }

    #[test]
    fn procedural_window_icon_renders_valid_rgba() {
        let rgba = render_app_icon_rgba(64);
        assert_eq!(rgba.len(), 64 * 64 * 4);
        // Center pixel should be opaque bright spark color
        let c = ((32 * 64 + 32) * 4) as usize;
        assert_eq!(rgba[c + 3], 255);
        assert!(rgba[c + 2] > 180);
        // Corner pixel (0, 0) outside squircle should be transparent
        assert_eq!(rgba[3], 0);
    }
}
