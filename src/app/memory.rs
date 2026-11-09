#[cfg(target_os = "windows")]
pub fn get_current_process_memory_mb() -> (f64, f64) {
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let mut pmc = PROCESS_MEMORY_COUNTERS::default();
        let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, size).is_ok() {
            let working_set_mb = (pmc.WorkingSetSize as f64) / (1024.0 * 1024.0);
            let pagefile_mb = (pmc.PagefileUsage as f64) / (1024.0 * 1024.0);
            (working_set_mb, pagefile_mb)
        } else {
            (0.0, 0.0)
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_current_process_memory_mb() -> (f64, f64) {
    (0.0, 0.0)
}

#[cfg(target_os = "windows")]
pub fn force_empty_working_set() -> f64 {
    use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
    use windows::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
    let (ws, _) = get_current_process_memory_mb();
    ws
}

#[cfg(not(target_os = "windows"))]
pub fn force_empty_working_set() -> f64 {
    0.0
}
