use tao::event_loop::EventLoopProxy;

use crate::app::events::UserEvent;

use super::ChromeExtensionEntry;

#[cfg(target_os = "windows")]
fn profile7(
    webview: &wry::WebView,
) -> Result<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Profile7, String> {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Profile7, ICoreWebView2_13,
    };
    use windows::core::Interface;
    use wry::WebViewExtWindows;

    unsafe {
        let core = webview.webview();
        let wv13: ICoreWebView2_13 = core
            .cast()
            .map_err(|e| format!("WebView2 ICoreWebView2_13 unsupported: {}", e))?;
        let profile = wv13
            .Profile()
            .map_err(|e| format!("Failed accessing WebView2 Profile: {}", e))?;
        profile
            .cast::<ICoreWebView2Profile7>()
            .map_err(|_| "Current WebView2 Runtime does not support browser extensions (requires >= 119)".to_string())
    }
}

#[cfg(target_os = "windows")]
pub fn add_extension(
    webview: &wry::WebView,
    entry_id: String,
    dir_path: String,
    desired_enabled: bool,
    proxy: EventLoopProxy<UserEvent>,
) {
    use webview2_com::{
        take_pwstr, BrowserExtensionEnableCompletedHandler,
        ProfileAddBrowserExtensionCompletedHandler,
    };
    use windows::core::{HSTRING, PWSTR};

    let prof = match profile7(webview) {
        Ok(p) => p,
        Err(err) => {
            let _ = proxy.send_event(UserEvent::ChromeExtRuntimeReady {
                entry_id,
                runtime_id: String::new(),
                enabled: desired_enabled,
                error: Some(err),
            });
            return;
        }
    };

    let path_hs = HSTRING::from(dir_path.as_str());
    let eid = entry_id.clone();
    let handler = ProfileAddBrowserExtensionCompletedHandler::create(Box::new(move |hr, ext_opt| {
        if let Err(err) = hr {
            let _ = proxy.send_event(UserEvent::ChromeExtRuntimeReady {
                entry_id: eid.clone(),
                runtime_id: String::new(),
                enabled: desired_enabled,
                error: Some(format!("WebView2 rejected extension: {}", err)),
            });
            return Ok(());
        }
        let mut rid = String::new();
        if let Some(ext) = ext_opt {
            unsafe {
                let mut pw = PWSTR::null();
                if ext.Id(&mut pw).is_ok() {
                    rid = take_pwstr(pw);
                }
                if !desired_enabled {
                    let h_en = BrowserExtensionEnableCompletedHandler::create(Box::new(|_| Ok(())));
                    let _ = ext.Enable(false, &h_en);
                }
            }
        }
        let _ = proxy.send_event(UserEvent::ChromeExtRuntimeReady {
            entry_id: eid.clone(),
            runtime_id: rid,
            enabled: desired_enabled,
            error: None,
        });
        Ok(())
    }));

    unsafe {
        if let Err(e) = prof.AddBrowserExtension(&path_hs, &handler) {
            let _ = e;
        }
    }
}

#[cfg(target_os = "windows")]
pub fn sync_all_extensions(
    webview: &wry::WebView,
    entries: Vec<ChromeExtensionEntry>,
    proxy: EventLoopProxy<UserEvent>,
) {
    use webview2_com::{
        take_pwstr, BrowserExtensionEnableCompletedHandler,
        ProfileAddBrowserExtensionCompletedHandler, ProfileGetBrowserExtensionsCompletedHandler,
    };
    use windows::core::{BOOL, HSTRING, PWSTR};

    let Ok(prof) = profile7(webview) else {
        return;
    };
    let prof_for_cb = prof.clone();
    let handler =
        ProfileGetBrowserExtensionsCompletedHandler::create(Box::new(move |hr, list_opt| {
            if hr.is_err() {
                return Ok(());
            }
            let mut loaded: Vec<(String, String, bool)> = Vec::new();
            let mut live_objs = Vec::new();
            if let Some(list) = list_opt {
                unsafe {
                    let mut count: u32 = 0;
                    if list.Count(&mut count).is_ok() {
                        for i in 0..count {
                            if let Ok(ext) = list.GetValueAtIndex(i) {
                                let mut id_pw = PWSTR::null();
                                let mut name_pw = PWSTR::null();
                                let mut en_b = BOOL::default();
                                let id = if ext.Id(&mut id_pw).is_ok() {
                                    take_pwstr(id_pw)
                                } else {
                                    String::new()
                                };
                                let name = if ext.Name(&mut name_pw).is_ok() {
                                    take_pwstr(name_pw)
                                } else {
                                    String::new()
                                };
                                let _ = ext.IsEnabled(&mut en_b);
                                loaded.push((id, name, en_b.as_bool()));
                                live_objs.push(ext);
                            }
                        }
                    }
                }
            }

            for entry in &entries {
                let matched_idx = loaded.iter().position(|(rid, nm, _)| {
                    (!entry.runtime_id.is_empty() && rid == &entry.runtime_id)
                        || (entry.runtime_id.is_empty() && nm.eq_ignore_ascii_case(&entry.name))
                });
                if let Some(idx) = matched_idx {
                    let (_, _, is_en) = loaded[idx];
                    if is_en != entry.enabled {
                        let en_h = BrowserExtensionEnableCompletedHandler::create(Box::new(|_| Ok(())));
                        unsafe {
                            let _ = live_objs[idx].Enable(entry.enabled, &en_h);
                        }
                    }
                } else if std::path::Path::new(&entry.dir_path).is_dir() {
                    let eid = entry.id.clone();
                    let desired = entry.enabled;
                    let p_cb = proxy.clone();
                    let add_h = ProfileAddBrowserExtensionCompletedHandler::create(Box::new(
                        move |ahr, ext_opt| {
                            if ahr.is_err() {
                                return Ok(());
                            }
                            let mut rid = String::new();
                            if let Some(ext) = ext_opt {
                                unsafe {
                                    let mut pw = PWSTR::null();
                                    if ext.Id(&mut pw).is_ok() {
                                        rid = take_pwstr(pw);
                                    }
                                    if !desired {
                                        let h_en = BrowserExtensionEnableCompletedHandler::create(
                                            Box::new(|_| Ok(())),
                                        );
                                        let _ = ext.Enable(false, &h_en);
                                    }
                                }
                            }
                            let _ = p_cb.send_event(UserEvent::ChromeExtRuntimeReady {
                                entry_id: eid.clone(),
                                runtime_id: rid,
                                enabled: desired,
                                error: None,
                            });
                            Ok(())
                        },
                    ));
                    unsafe {
                        let _ = prof_for_cb
                            .AddBrowserExtension(&HSTRING::from(entry.dir_path.as_str()), &add_h);
                    }
                }
            }

            let _ = proxy.send_event(UserEvent::ChromeExtSynced(loaded));
            Ok(())
        }));

    unsafe {
        let _ = prof.GetBrowserExtensions(&handler);
    }
}

#[cfg(target_os = "windows")]
pub fn set_extension_enabled(
    webview: &wry::WebView,
    entry_id: String,
    runtime_id: String,
    enabled: bool,
    proxy: EventLoopProxy<UserEvent>,
) {
    use webview2_com::{
        take_pwstr, BrowserExtensionEnableCompletedHandler,
        ProfileGetBrowserExtensionsCompletedHandler,
    };
    use windows::core::PWSTR;

    let Ok(prof) = profile7(webview) else {
        return;
    };
    let handler =
        ProfileGetBrowserExtensionsCompletedHandler::create(Box::new(move |hr, list_opt| {
            if hr.is_err() {
                return Ok(());
            }
            if let Some(list) = list_opt {
                unsafe {
                    let mut count: u32 = 0;
                    if list.Count(&mut count).is_ok() {
                        for i in 0..count {
                            if let Ok(ext) = list.GetValueAtIndex(i) {
                                let mut id_pw = PWSTR::null();
                                let rid = if ext.Id(&mut id_pw).is_ok() {
                                    take_pwstr(id_pw)
                                } else {
                                    String::new()
                                };
                                if rid == runtime_id || (runtime_id.is_empty() && rid == entry_id) {
                                    let p = proxy.clone();
                                    let eid = entry_id.clone();
                                    let r = rid.clone();
                                    let h = BrowserExtensionEnableCompletedHandler::create(Box::new(
                                        move |ehr| {
                                            if ehr.is_ok() {
                                                let _ = p.send_event(
                                                    UserEvent::ChromeExtRuntimeReady {
                                                        entry_id: eid.clone(),
                                                        runtime_id: r.clone(),
                                                        enabled,
                                                        error: None,
                                                    },
                                                );
                                            }
                                            Ok(())
                                        },
                                    ));
                                    let _ = ext.Enable(enabled, &h);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }));

    unsafe {
        let _ = prof.GetBrowserExtensions(&handler);
    }
}

#[cfg(target_os = "windows")]
pub fn remove_extension(
    webview: &wry::WebView,
    entry_id: String,
    runtime_id: String,
    proxy: EventLoopProxy<UserEvent>,
) {
    use webview2_com::{
        take_pwstr, BrowserExtensionRemoveCompletedHandler,
        ProfileGetBrowserExtensionsCompletedHandler,
    };
    use windows::core::PWSTR;

    let Ok(prof) = profile7(webview) else {
        let _ = proxy.send_event(UserEvent::SyncConfig);
        return;
    };
    let handler =
        ProfileGetBrowserExtensionsCompletedHandler::create(Box::new(move |hr, list_opt| {
            if hr.is_ok() {
                if let Some(list) = list_opt {
                    unsafe {
                        let mut count: u32 = 0;
                        if list.Count(&mut count).is_ok() {
                            for i in 0..count {
                                if let Ok(ext) = list.GetValueAtIndex(i) {
                                    let mut id_pw = PWSTR::null();
                                    let rid = if ext.Id(&mut id_pw).is_ok() {
                                        take_pwstr(id_pw)
                                    } else {
                                        String::new()
                                    };
                                    if rid == runtime_id
                                        || (runtime_id.is_empty() && rid == entry_id)
                                    {
                                        let h = BrowserExtensionRemoveCompletedHandler::create(
                                            Box::new(|_| Ok(())),
                                        );
                                        let _ = ext.Remove(&h);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let _ = proxy.send_event(UserEvent::SyncConfig);
            Ok(())
        }));

    unsafe {
        let _ = prof.GetBrowserExtensions(&handler);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn add_extension(
    _webview: &wry::WebView,
    _entry_id: String,
    _dir_path: String,
    _desired_enabled: bool,
    _proxy: EventLoopProxy<UserEvent>,
) {
}

#[cfg(not(target_os = "windows"))]
pub fn sync_all_extensions(
    _webview: &wry::WebView,
    _entries: Vec<ChromeExtensionEntry>,
    _proxy: EventLoopProxy<UserEvent>,
) {
}

#[cfg(not(target_os = "windows"))]
pub fn set_extension_enabled(
    _webview: &wry::WebView,
    _entry_id: String,
    _runtime_id: String,
    _enabled: bool,
    _proxy: EventLoopProxy<UserEvent>,
) {
}

#[cfg(not(target_os = "windows"))]
pub fn remove_extension(
    _webview: &wry::WebView,
    _entry_id: String,
    _runtime_id: String,
    _proxy: EventLoopProxy<UserEvent>,
) {
}
