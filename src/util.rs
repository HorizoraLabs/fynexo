use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

/// Poison-tolerant locking: a panic on another thread must not cascade into
/// every later `lock()` call. The inner data is still valid for our use cases
/// (plain config / tab metadata), so we simply recover the guard.
pub trait LockExt<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T>;
}

impl<T> LockExt<T> for Mutex<T> {
    #[inline]
    fn lock_safe(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Crash-safe file write: write to a sibling `.tmp` file, then atomically
/// rename over the destination. A crash mid-write leaves the old file intact
/// instead of a truncated / corrupt JSON document.
pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let path = path.as_ref();
    let mut tmp_name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(tmp_name);

    let result = fs::write(&tmp, contents.as_ref()).and_then(|_| fs::rename(&tmp, path));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
        // Last-resort fallback (e.g. destination on a filesystem refusing rename-over).
        return fs::write(path, contents.as_ref());
    }
    result
}

/// Serialize `value` as JSON safe to embed inline inside an HTML `<script>` element.
/// Escapes `<`, `>`, `&`, U+2028, and U+2029 so untrusted strings (such as a visited
/// page's `<title>`) cannot break out of `</script>` and cause stored XSS.
pub fn json_for_html_script<T: serde::Serialize>(value: &T) -> String {
    let raw = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            _ => out.push(ch),
        }
    }
    out
}

/// Split a static template once around `placeholder` so callers only splice middle data
/// without re-scanning the entire template on every render.
#[inline]
pub fn split_template_once(template: &'static str, placeholder: &str) -> (&'static str, &'static str) {
    template.split_once(placeholder).unwrap_or((template, ""))
}

/// Concatenate a pre-split `(prefix, suffix)` around `middle` with exact pre-allocation.
#[inline]
pub fn join_split_template(prefix: &str, middle: &str, suffix: &str) -> String {
    let mut out = String::with_capacity(prefix.len() + middle.len() + suffix.len());
    out.push_str(prefix);
    out.push_str(middle);
    out.push_str(suffix);
    out
}

struct PendingEntry {
    bytes: Vec<u8>,
    due: Instant,
}

#[derive(Default)]
struct PendingWriteQueue {
    entries: HashMap<PathBuf, PendingEntry>,
    in_flight: bool,
    flush_requested: bool,
}

struct PendingWriterHub {
    queue: Mutex<PendingWriteQueue>,
    cv: Condvar,
}

static PENDING_WRITER: OnceLock<PendingWriterHub> = OnceLock::new();

fn writer_hub() -> &'static PendingWriterHub {
    PENDING_WRITER.get_or_init(|| {
        let hub = PendingWriterHub {
            queue: Mutex::new(PendingWriteQueue::default()),
            cv: Condvar::new(),
        };
        let _ = std::thread::Builder::new()
            .name("fnx-disk-writer".to_string())
            .spawn(writer_worker_loop);
        hub
    })
}

fn writer_worker_loop() {
    let hub = PENDING_WRITER
        .get()
        .expect("writer hub initialized before worker starts");
    loop {
        let batch: Vec<(PathBuf, Vec<u8>)> = {
            let mut guard = hub.queue.lock_safe();
            while guard.entries.is_empty() {
                guard.flush_requested = false;
                hub.cv.notify_all();
                guard = hub.cv.wait(guard).unwrap_or_else(|p| p.into_inner());
            }

            if !guard.flush_requested {
                if let Some(earliest) = guard.entries.values().map(|e| e.due).min() {
                    let now = Instant::now();
                    if earliest > now {
                        let (g, _) = hub
                            .cv
                            .wait_timeout(guard, earliest.saturating_duration_since(now))
                            .unwrap_or_else(|p| p.into_inner());
                        guard = g;
                    }
                }
            }

            let now = Instant::now();
            let drain_all = guard.flush_requested;
            let ready_paths: Vec<PathBuf> = guard
                .entries
                .iter()
                .filter(|(_, entry)| drain_all || entry.due <= now)
                .map(|(p, _)| p.clone())
                .collect();
            if ready_paths.is_empty() {
                continue;
            }
            let mut batch = Vec::with_capacity(ready_paths.len());
            for path in ready_paths {
                if let Some(entry) = guard.entries.remove(&path) {
                    batch.push((path, entry.bytes));
                }
            }
            guard.in_flight = true;
            batch
        };

        for (path, bytes) in batch {
            let _ = atomic_write(path, bytes);
        }

        let mut guard = hub.queue.lock_safe();
        guard.in_flight = false;
        if guard.entries.is_empty() {
            guard.flush_requested = false;
        }
        hub.cv.notify_all();
    }
}

/// Queue a coalesced crash-safe write on the background writer thread. Repeated updates
/// to the same `path` within `delay` collapse into a single write with the latest bytes.
pub fn schedule_atomic_write<P: Into<PathBuf>, C: Into<Vec<u8>>>(
    path: P,
    contents: C,
    delay: Duration,
) {
    let hub = writer_hub();
    let due = Instant::now() + delay;
    let mut guard = hub.queue.lock_safe();
    guard.entries.insert(
        path.into(),
        PendingEntry {
            bytes: contents.into(),
            due,
        },
    );
    hub.cv.notify_all();
}

/// Block until all previously scheduled background writes have finished flushing to disk.
pub fn flush_pending_writes() {
    let Some(hub) = PENDING_WRITER.get() else {
        return;
    };
    let mut guard = hub.queue.lock_safe();
    if guard.entries.is_empty() && !guard.in_flight {
        return;
    }
    guard.flush_requested = true;
    hub.cv.notify_all();
    while !guard.entries.is_empty() || guard.in_flight {
        guard = hub.cv.wait(guard).unwrap_or_else(|p| p.into_inner());
    }
}

#[cfg(test)]
pub fn isolate_data_dir() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let dir = std::env::temp_dir().join(format!("fynexo_test_data_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        // Must run before the first `data_dir()` call so tests never touch real user data.
        std::env::set_var("FYNEXO_DATA_DIR", &dir);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_content_and_leaves_no_tmp() {
        let dir = std::env::temp_dir().join(format!("fynexo_test_atomic_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("sample.json");

        atomic_write(&file, "{\"a\":1}").unwrap();
        atomic_write(&file, "{\"a\":2}").unwrap();

        assert_eq!(fs::read_to_string(&file).unwrap(), "{\"a\":2}");
        assert!(!dir.join("sample.json.tmp").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lock_safe_recovers_from_poison() {
        let m = std::sync::Arc::new(Mutex::new(5));
        let m2 = std::sync::Arc::clone(&m);
        let _ = std::thread::spawn(move || {
            let _g = m2.lock().unwrap();
            panic!("poison it");
        })
        .join();
        assert_eq!(*m.lock_safe(), 5);
    }

    #[test]
    fn json_for_html_script_escapes_script_breakouts() {
        let evil = serde_json::json!({
            "title": "</script><img src=x onerror=alert(1)>&\u{2028}\u{2029}"
        });
        let safe = json_for_html_script(&evil);
        assert!(!safe.contains('<'));
        assert!(!safe.contains('>'));
        assert!(!safe.contains('&'));
        assert!(!safe.contains('\u{2028}'));
        assert!(!safe.contains('\u{2029}'));
        let decoded: serde_json::Value = serde_json::from_str(&safe).unwrap();
        assert_eq!(decoded, evil);
    }

    #[test]
    fn scheduled_writes_coalesce_and_flush_cleanly() {
        let dir = std::env::temp_dir().join(format!("fynexo_test_coalesce_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("debounced.json");

        schedule_atomic_write(&file, b"{\"v\":1}".to_vec(), Duration::from_secs(30));
        schedule_atomic_write(&file, b"{\"v\":2}".to_vec(), Duration::from_secs(30));
        flush_pending_writes();

        assert_eq!(fs::read_to_string(&file).unwrap(), "{\"v\":2}");
        assert!(!dir.join("debounced.json.tmp").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn split_and_join_template_splices_once() {
        let tpl = "<html><script>const x = __BOOT__;</script></html>";
        let (pre, suf) = split_template_once(tpl, "__BOOT__");
        assert_eq!(join_split_template(pre, "{\"ok\":true}", suf), "<html><script>const x = {\"ok\":true};</script></html>");
    }
}
