use crate::fs_util::write_atomic;
use crate::media;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

pub fn cache_dir(store: &str) -> PathBuf {
    crate::cache_dir().join(store)
}

fn image_stem(app_name: &str, kind: &str) -> String {
    format!("{app_name}_{kind}")
}

fn image_path(store: &str, app_name: &str, kind: &str, url: &str) -> PathBuf {
    let hash = crc32fast::hash(url.as_bytes());
    cache_dir(store).join(format!("{}_{hash:08x}.img", image_stem(app_name, kind)))
}

fn remove_stale_images(store: &str, app_name: &str, kind: &str, keep: &Path) {
    let stem = image_stem(app_name, kind);
    let Ok(entries) = fs_err::read_dir(cache_dir(store)) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(rest) = name.strip_prefix(&stem) else {
            continue;
        };
        let ours = rest == ".img" || (rest.starts_with('_') && rest.ends_with(".img"));
        if ours && entry.path() != keep {
            let _ = fs_err::remove_file(entry.path());
        }
    }
}

pub fn library_path(store: &str) -> PathBuf {
    cache_dir(store).join("library.json")
}

pub fn load_library<T: DeserializeOwned>(store: &str) -> Vec<T> {
    let Ok(data) = fs_err::read_to_string(library_path(store)) else {
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
    if let Err(e) = write_atomic(&library_path(store), body) {
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
    let url = transform(url);
    let path = image_path(store, app_name, kind, &url);
    if !path.exists() {
        remove_stale_images(store, app_name, kind, &path);
    }
    media::fetch_cached_image(
        &path,
        &url,
        format!("{store}_{}", image_stem(app_name, kind)),
    )
}
