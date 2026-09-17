pub mod api;
pub mod source;
pub mod update;

pub fn read_install_version(install_path: &std::path::Path, data_folder: &str) -> Option<String> {
    if let Some(v) = crate::gacha::state::read_install_dotversion(install_path) {
        return Some(v);
    }
    crate::gacha::state::scan_globalgamemanagers(install_path, data_folder, 0)
}

// gryphline wants a rand_str on every request; not validated server-side per traces
pub fn rand_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let mut hasher = md5::Md5::new();
    use md5::Digest;
    hasher.update(format!("{}-{}-{}", nanos, pid, rand_counter()).as_bytes());
    format!("{:x}", hasher.finalize())
}

fn rand_counter() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
