pub mod api;
pub mod source;
pub mod update;

use std::path::PathBuf;

const PUBLISHER_SLUG: &str = "hypergryph";

pub fn version_file(game_slug: &str, edition_id: &str) -> PathBuf {
    crate::gacha::state::version_file(PUBLISHER_SLUG, game_slug, edition_id)
}

pub fn installed_version(game_slug: &str, edition_id: &str) -> Option<String> {
    crate::gacha::state::read_installed_version(PUBLISHER_SLUG, game_slug, edition_id)
}

pub fn set_installed_version(game_slug: &str, edition_id: &str, version: &str) {
    crate::gacha::state::write_installed_version(PUBLISHER_SLUG, game_slug, edition_id, version);
}

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
