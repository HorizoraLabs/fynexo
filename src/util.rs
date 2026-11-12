use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

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
}
