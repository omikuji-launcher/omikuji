use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub fn cache_dir(store: &str) -> PathBuf {
    crate::cache_dir().join(store)
}

pub fn image_path(store: &str, app_name: &str, kind: &str) -> PathBuf {
    cache_dir(store).join(format!("{}_{}.img", app_name, kind))
}

pub fn library_path(store: &str) -> PathBuf {
    cache_dir(store).join("library.json")
}

pub fn load_library<T: DeserializeOwned>(store: &str) -> Vec<T> {
    let Ok(data) = std::fs::read_to_string(library_path(store)) else {
        return Vec::new();
    };
    match serde_json::from_str::<Vec<T>>(&data) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("{} library cache parse failed: {}", store, e);
            Vec::new()
        }
    }
}

pub fn save_library<T: Serialize>(store: &str, games: &[T]) {
    let body = match serde_json::to_string(games) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("{} library cache serialize failed: {}", store, e);
            return;
        }
    };
    if let Err(e) = crate::fs_util::write_atomic(&library_path(store), body) {
        tracing::error!("{} library cache write failed: {}", store, e);
    }
}

// transform is the vendor's url rewrite, epic thumbnails its cdn urls and gog takes them raw
pub fn resolve_image(
    store: &str,
    app_name: &str,
    kind: &str,
    cdn_url: Option<&str>,
    transform: impl Fn(&str) -> String,
) -> Option<String> {
    let url = cdn_url?;
    if url.is_empty() {
        return None;
    }
    crate::media::fetch_cached_image(
        &image_path(store, app_name, kind),
        &transform(url),
        format!("{}_{}_{}", store, app_name, kind),
    )
}
