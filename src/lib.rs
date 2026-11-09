pub mod adblock;
pub mod app;
pub mod config;
pub mod hud;
pub mod pages;
pub mod tabs;

#[cfg(target_os = "android")]
fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
            std::process::abort()
        }
    }
}

#[cfg(target_os = "android")]
fn _start_app() {
    stop_unwind(|| {
        let _ = app::run();
    });
}

#[no_mangle]
#[inline(never)]
#[cfg(target_os = "android")]
pub extern "C" fn start_app() {
    tao::android_binding!(
        com_horizoralabs,
        fynexo,
        Rust,
        wry::android_setup,
        _start_app
    );
    wry::android_binding!(com_horizoralabs, fynexo);
}
