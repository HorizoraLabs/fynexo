#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct MemorySnapshot {
    pub host_mb: f64,
    pub webview_mb: f64,
    pub total_mb: f64,
    pub commit_mb: f64,
    pub process_count: usize,
}

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

#[cfg(target_os = "windows")]
pub fn get_process_tree_memory_with_roots(extra_roots: &[u32]) -> MemorySnapshot {
    use std::collections::{HashMap, HashSet, VecDeque};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::{
        GetCurrentProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    let (host_mb, host_commit) = get_current_process_memory_mb();
    let root_pid = unsafe { GetCurrentProcessId() };

    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut is_wv2: HashSet<u32> = HashSet::new();
    unsafe {
        if let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            let mut pe = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            if Process32FirstW(snap, &mut pe).is_ok() {
                loop {
                    let pid = pe.th32ProcessID;
                    let ppid = pe.th32ParentProcessID;
                    if pid != 0 && pid != ppid {
                        children.entry(ppid).or_default().push(pid);
                    }
                    let name_len = pe
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(pe.szExeFile.len());
                    let exe_name = String::from_utf16_lossy(&pe.szExeFile[..name_len]);
                    if exe_name.eq_ignore_ascii_case("msedgewebview2.exe") {
                        is_wv2.insert(pid);
                    }
                    if Process32NextW(snap, &mut pe).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);
        }
    }

    let mut visited: HashSet<u32> = HashSet::new();
    let mut queue: VecDeque<u32> = VecDeque::new();
    visited.insert(root_pid);
    queue.push_back(root_pid);
    for &pid in extra_roots {
        if pid != 0 && visited.insert(pid) {
            queue.push_back(pid);
        }
    }

    let mut webview_ws_bytes: u64 = 0;
    let mut webview_commit_bytes: u64 = 0;
    let mut wv_procs: usize = 0;

    while let Some(pid) = queue.pop_front() {
        if let Some(kids) = children.get(&pid) {
            for &kid in kids {
                if visited.insert(kid) {
                    queue.push_back(kid);
                }
            }
        }
        if pid == root_pid || !is_wv2.contains(&pid) {
            continue;
        }
        unsafe {
            if let Ok(handle) = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            ) {
                let mut pmc = PROCESS_MEMORY_COUNTERS::default();
                let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
                if GetProcessMemoryInfo(handle, &mut pmc, size).is_ok() {
                    webview_ws_bytes = webview_ws_bytes.saturating_add(pmc.WorkingSetSize as u64);
                    webview_commit_bytes =
                        webview_commit_bytes.saturating_add(pmc.PagefileUsage as u64);
                    wv_procs += 1;
                }
                let _ = CloseHandle(handle);
            }
        }
    }

    let webview_mb = (webview_ws_bytes as f64) / (1024.0 * 1024.0);
    let webview_commit_mb = (webview_commit_bytes as f64) / (1024.0 * 1024.0);

    MemorySnapshot {
        host_mb,
        webview_mb,
        total_mb: host_mb + webview_mb,
        commit_mb: host_commit + webview_commit_mb,
        process_count: 1 + wv_procs,
    }
}

#[cfg(target_os = "windows")]
pub fn get_process_tree_memory() -> MemorySnapshot {
    get_process_tree_memory_with_roots(&[])
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub fn get_current_process_memory_mb() -> (f64, f64) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (0.0, 0.0);
    };
    let mut rss_kb = 0.0f64;
    let mut vm_kb = 0.0f64;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            rss_kb = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
        } else if let Some(rest) = line.strip_prefix("VmSize:") {
            vm_kb = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
        }
    }
    (rss_kb / 1024.0, vm_kb / 1024.0)
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub fn get_process_tree_memory_with_roots(_extra_roots: &[u32]) -> MemorySnapshot {
    let (host_mb, commit_mb) = get_current_process_memory_mb();
    MemorySnapshot {
        host_mb,
        webview_mb: 0.0,
        total_mb: host_mb,
        commit_mb,
        process_count: 1,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub fn get_process_tree_memory() -> MemorySnapshot {
    get_process_tree_memory_with_roots(&[])
}

#[cfg(not(any(target_os = "windows", target_os = "android", target_os = "linux")))]
pub fn get_current_process_memory_mb() -> (f64, f64) {
    (0.0, 0.0)
}

#[cfg(not(any(target_os = "windows", target_os = "android", target_os = "linux")))]
pub fn get_process_tree_memory_with_roots(_extra_roots: &[u32]) -> MemorySnapshot {
    MemorySnapshot::default()
}

#[cfg(not(any(target_os = "windows", target_os = "android", target_os = "linux")))]
pub fn get_process_tree_memory() -> MemorySnapshot {
    MemorySnapshot::default()
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
