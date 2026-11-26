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

fn poly_sdf(px: f32, py: f32, pts: &[(f32, f32)]) -> f32 {
    let mut d = f32::INFINITY;
    let mut inside = false;
    let n = pts.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = pts[i];
        let (xj, yj) = pts[j];
        let ex = xj - xi;
        let ey = yj - yi;
        let wx = px - xi;
        let wy = py - yi;
        let t = ((wx * ex + wy * ey) / (ex * ex + ey * ey + 1e-12)).clamp(0.0, 1.0);
        let dx = wx - ex * t;
        let dy = wy - ey * t;
        let dist2 = dx * dx + dy * dy;
        if dist2 < d {
            d = dist2;
        }
        let cond = (yi > py) != (yj > py) && px < ((xj - xi) * (py - yi)) / (yj - yi + 1e-12) + xi;
        if cond {
            inside = !inside;
        }
        j = i;
    }
    let dist = d.sqrt();
    if inside {
        -dist
    } else {
        dist
    }
}

/// Procedurally render a crisp anti-aliased RGBA Fynexo brand icon
/// (dark obsidian squircle badge with Fynexo's sculpted 2-blade geometric "F" monogram).
pub fn render_app_icon_rgba(size: u32) -> Vec<u8> {
    const TOP_BLADE: [(f32, f32); 4] = [(33.5, 27.5), (78.5, 27.5), (71.5, 40.5), (33.5, 40.5)];
    const MID_BLADE: [(f32, f32); 4] = [(33.5, 46.5), (68.5, 46.5), (61.8, 59.0), (33.5, 59.0)];
    const STEM_BLADE: [(f32, f32); 4] = [(33.5, 46.5), (47.2, 46.5), (47.2, 74.2), (33.5, 81.0)];

    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let scale = 108.0 / (size as f32);
    let edge_aa = (64.0 / (size as f32)).max(0.35);

    let sample = |px: f32, py: f32| -> [f32; 4] {
        let u = px * scale;
        let v = py * scale;

        // Rounded squircle container centered at (54, 54)
        let nx = (u - 54.0).abs();
        let ny = (v - 54.0).abs();
        let r = 21.5f32;
        let b = 51.0 - r;
        let qx = (nx - b).max(0.0);
        let qy = (ny - b).max(0.0);
        let sdf = (qx * qx + qy * qy).sqrt() - r;
        if sdf > edge_aa {
            return [0.0, 0.0, 0.0, 0.0];
        }
        let alpha = ((edge_aa - sdf) / (2.0 * edge_aa)).clamp(0.0, 1.0);

        // Obsidian-indigo studio backdrop
        let t_diag = ((u * 0.42 + v * 0.58) / 108.0).clamp(0.0, 1.0);
        let mut cr = 22.0 + (9.0 - 22.0) * t_diag;
        let mut cg = 26.0 + (11.0 - 26.0) * t_diag;
        let mut cb = 43.0 + (18.0 - 43.0) * t_diag;

        // Ambient indigo aura behind the F mark
        let gx = (u - 50.0) / 38.0;
        let gy = (v - 44.0) / 38.0;
        let glow = (-(gx * gx + gy * gy) * 1.45).exp() * 0.36;
        cr += 96.0 * glow;
        cg += 108.0 * glow;
        cb += 255.0 * glow;

        // Crisp hairline glass rim
        if sdf > -1.4 && sdf <= 0.1 {
            let top_bias = (1.0 - (v - 4.0) / 86.0).clamp(0.2, 1.0);
            let rim = ((sdf + 1.4) / 1.4).clamp(0.0, 1.0) * 0.28 * top_bias;
            cr = cr * (1.0 - rim) + 158.0 * rim;
            cg = cg * (1.0 - rim) + 170.0 * rim;
            cb = cb * (1.0 - rim) + 255.0 * rim;
        }

        // Subtle drop shadow
        let sh_top = poly_sdf(u - 0.8, v - 2.4, &TOP_BLADE) - 1.2;
        let sh_low = poly_sdf(u - 0.8, v - 2.4, &MID_BLADE)
            .min(poly_sdf(u - 0.8, v - 2.4, &STEM_BLADE))
            - 1.2;
        let sh_min = sh_top.min(sh_low);
        if sh_min < 5.5 {
            let sh_a = ((5.5 - sh_min) / 6.5).clamp(0.0, 1.0) * 0.38;
            cr *= 1.0 - sh_a;
            cg *= 1.0 - sh_a;
            cb *= 1.0 - sh_a;
        }

        // Lower Stem + Mid Blade (electric indigo -> violet-blue)
        let d_low = poly_sdf(u, v, &MID_BLADE).min(poly_sdf(u, v, &STEM_BLADE)) - 1.1;
        if d_low < edge_aa {
            let a = ((edge_aa - d_low) / (2.0 * edge_aa)).clamp(0.0, 1.0);
            let t = (((u - 33.5) * 0.65 + (v - 46.5) * 0.85) / 52.0).clamp(0.0, 1.0);
            let lr = 148.0 + (92.0 - 148.0) * t;
            let lg = 160.0 + (102.0 - 160.0) * t;
            let lb = 255.0 + (245.0 - 255.0) * t;
            cr = cr * (1.0 - a) + lr * a;
            cg = cg * (1.0 - a) + lg * a;
            cb = cb * (1.0 - a) + lb * a;
        }

        // Upper Sweeping Crown (crisp white -> soft periwinkle)
        let d_top = poly_sdf(u, v, &TOP_BLADE) - 1.2;
        if d_top < edge_aa {
            let a = ((edge_aa - d_top) / (2.0 * edge_aa)).clamp(0.0, 1.0);
            let t = ((u - 33.5) / 46.0).clamp(0.0, 1.0);
            let tr = 255.0 + (214.0 - 255.0) * t;
            let tg = 255.0 + (222.0 - 255.0) * t;
            let tb = 255.0;
            cr = cr * (1.0 - a) + tr * a;
            cg = cg * (1.0 - a) + tg * a;
            cb = cb * (1.0 - a) + tb * a;
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
