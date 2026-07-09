// generic archive-source fetcher for runners (wine-ge, proton-ge, spritz, ...)
// and dll packs (dxvk, vkd3d-proton, dxvk-nvapi, d3d-extras). callers pass a dest_root; thats the only real difference between a runner and a dll pack install.
// adding a new source is a 5-line paste in settings.rs, no code change here. yayyyy =m=

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::collections::VecDeque;

use crate::components_config::ArchiveSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag: String,
    pub published_at: String,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_size: u64,
}

#[derive(Debug, Clone)]
pub enum ArchiveEvent {
    // category: "runners" | "dll_packs", routes to the right QML listener
    Started   { category: String, source: String, tag: String },
    Progress  { category: String, source: String, tag: String, phase: String, percent: f64 },
    Completed { category: String, source: String, tag: String, install_dir: String },
    Failed    { category: String, source: String, tag: String, error: String },
}

static EVENTS: OnceLock<Mutex<VecDeque<ArchiveEvent>>> = OnceLock::new();

fn queue() -> &'static Mutex<VecDeque<ArchiveEvent>> {
    EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn drain_events() -> Vec<ArchiveEvent> {
    queue().lock().unwrap().drain(..).collect()
}

fn push(ev: ArchiveEvent) {
    queue().lock().unwrap().push_back(ev);
}

pub async fn fetch_versions(source: &ArchiveSource) -> Result<Vec<ReleaseInfo>> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("omikuji/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let resp = client
        .get(&source.api_url)
        .query(&[("per_page", "100")])
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("release list ({}): {}", source.api_url, e))?;

    let releases: Vec<serde_json::Value> = resp.json().await?;

    let mut out = Vec::new();
    for r in releases {
        let tag = r
            .get("tag_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let published = r
            .get("published_at")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let empty_assets = vec![];
        let assets = r
            .get("assets")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_assets);

        let Some(asset) = assets.iter().find(|a| {
            a.get("name")
                .and_then(|v| v.as_str())
                .map(|n| n.contains(&source.asset_pattern))
                .unwrap_or(false)
        }) else {
            continue;
        };

        let asset_name = asset
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let asset_url = asset
            .get("browser_download_url")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let asset_size = asset.get("size").and_then(|v| v.as_u64()).unwrap_or(0);

        if tag.is_empty() || asset_url.is_empty() {
            continue;
        }

        out.push(ReleaseInfo {
            tag,
            published_at: published,
            asset_name,
            asset_url,
            asset_size,
        });
    }
    Ok(out)
}

pub async fn install_version(
    category: &str,
    source: &ArchiveSource,
    release: &ReleaseInfo,
    dest_root: &Path,
) -> Result<PathBuf> {
    push(ArchiveEvent::Started {
        category: category.into(),
        source: source.name.clone(),
        tag: release.tag.clone(),
    });

    match install_inner(category, source, release, dest_root).await {
        Ok(dir) => {
            push(ArchiveEvent::Completed {
                category: category.into(),
                source: source.name.clone(),
                tag: release.tag.clone(),
                install_dir: dir.to_string_lossy().into_owned(),
            });
            Ok(dir)
        }
        Err(e) => {
            let msg = format!("{:#}", e);
            push(ArchiveEvent::Failed {
                category: category.into(),
                source: source.name.clone(),
                tag: release.tag.clone(),
                error: msg.clone(),
            });
            Err(anyhow!(msg))
        }
    }
}

async fn install_inner(
    category: &str,
    source: &ArchiveSource,
    release: &ReleaseInfo,
    dest_root: &Path,
) -> Result<PathBuf> {
    fs::create_dir_all(dest_root)?;

    let bytes = download_bytes(category, source, release).await?;

    push(ArchiveEvent::Progress {
        category: category.into(),
        source: source.name.clone(),
        tag: release.tag.clone(),
        phase: "extracting".into(),
        percent: 0.0,
    });

    let staging = dest_root.join(format!(".staging-{}-{}", source.name, release.tag));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;

    match source.extract.as_str() {
        "tar_gz" => {
            let reader = flate2::read::GzDecoder::new(std::io::Cursor::new(&bytes));
            tar::Archive::new(reader).unpack(&staging)?;
        }
        "tar_xz" => {
            let reader = xz2::read::XzDecoder::new(std::io::Cursor::new(&bytes));
            tar::Archive::new(reader).unpack(&staging)?;
        }
        "tar_zst" => {
            let reader = zstd::stream::read::Decoder::new(std::io::Cursor::new(&bytes))?;
            tar::Archive::new(reader).unpack(&staging)?;
        }
        "zip" => {
            zip::ZipArchive::new(std::io::Cursor::new(&bytes))?.extract(&staging)?;
        }
        other => {
            let _ = fs::remove_dir_all(&staging);
            return Err(anyhow!("unknown extract strategy: {}", other));
        }
    }

    let final_dir = dest_root.join(&release.tag);
    let _ = fs::remove_dir_all(&final_dir);

    let entries: Vec<PathBuf> = fs::read_dir(&staging)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();

    if entries.len() == 1 && entries[0].is_dir() {
        fs::rename(&entries[0], &final_dir)?;
        let _ = fs::remove_dir_all(&staging);
    } else {
        fs::rename(&staging, &final_dir)?;
    }

    write_sidecar(&final_dir, &source.name, &release.tag)?;

    Ok(final_dir)
}

const SIDECAR_FILENAME: &str = ".omikuji.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallSidecar {
    source: String,
    tag: String,
}

fn write_sidecar(dir: &Path, source_name: &str, tag: &str) -> Result<()> {
    let sc = InstallSidecar {
        source: source_name.to_string(),
        tag: tag.to_string(),
    };
    let path = dir.join(SIDECAR_FILENAME);
    fs::write(path, serde_json::to_string(&sc)?)?;
    Ok(())
}

fn read_sidecar(dir: &Path) -> Option<InstallSidecar> {
    let path = dir.join(SIDECAR_FILENAME);
    let body = fs::read_to_string(path).ok()?;
    serde_json::from_str::<InstallSidecar>(&body).ok()
}

pub fn installed_source_tag(dir: &Path) -> Option<(String, String)> {
    read_sidecar(dir).map(|sc| (sc.source, sc.tag))
}

async fn download_bytes(
    category: &str,
    source: &ArchiveSource,
    release: &ReleaseInfo,
) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("omikuji/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let resp = client
        .get(&release.asset_url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("download {}: {}", release.asset_url, e))?;

    let total = resp.content_length().unwrap_or(release.asset_size);
    let mut buf: Vec<u8> = if total > 0 {
        Vec::with_capacity(total as usize)
    } else {
        Vec::new()
    };

    let mut stream = resp.bytes_stream();
    let mut last_pct = -1.0_f64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buf.extend_from_slice(&chunk);
        if total > 0 {
            let pct = (buf.len() as f64 / total as f64) * 100.0;
            if pct - last_pct >= 1.0 {
                push(ArchiveEvent::Progress {
                    category: category.into(),
                    source: source.name.clone(),
                    tag: release.tag.clone(),
                    phase: "downloading".into(),
                    percent: pct,
                });
                last_pct = pct;
            }
        }
    }
    Ok(buf)
}

pub fn list_installed(source: &ArchiveSource, dest_root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dest_root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Some(sc) = read_sidecar(&path)
            && sc.source == source.name && !sc.tag.is_empty() {
                out.push(sc.tag);
            }
    }
    out.sort();
    out
}

pub fn delete_version(source: &ArchiveSource, dest_root: &Path, tag: &str) -> Result<()> {
    let dir = dest_root.join(tag);
    if !dir.exists() {
        return Ok(());
    }
    match read_sidecar(&dir) {
        Some(sc) if sc.source == source.name => {
            fs::remove_dir_all(&dir)?;
            Ok(())
        }
        Some(sc) => Err(anyhow!(
            "refusing to delete {}: sidecar claims source '{}', requested '{}'",
            dir.display(),
            sc.source,
            source.name
        )),
        None => Err(anyhow!(
            "refusing to delete {}: no sidecar (manually placed?)",
            dir.display()
        )),
    }
}
