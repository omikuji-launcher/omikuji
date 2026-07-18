// generic archive-source fetcher for runners (wine-ge, proton-ge, spritz, ...)
// and dll packs (dxvk, vkd3d-proton, dxvk-nvapi, d3d-extras). callers pass a dest_root; thats the only real difference between a runner and a dll pack install.
// adding a new source is a 5-line paste in settings.rs, no code change here. yayyyy =m=

use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::components_config::ArchiveSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag: String,
    pub published_at: String,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_size: u64,
    #[serde(default)]
    pub assets: Vec<AssetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub enum ArchiveEvent {
    // category: "runners" | "dll_packs", routes to the right QML listener
    Started {
        category: String,
        source: String,
        tag: String,
    },
    Progress {
        category: String,
        source: String,
        tag: String,
        phase: String,
        percent: f64,
    },
    Completed {
        category: String,
        source: String,
        tag: String,
        install_dir: String,
    },
    Failed {
        category: String,
        source: String,
        tag: String,
        error: String,
    },
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

const ARCHIVE_EXTS: &[(&str, &str)] = &[
    (".tar.gz", "tar_gz"),
    (".tar.xz", "tar_xz"),
    (".tar.zst", "tar_zst"),
    (".zip", "zip"),
];

fn asset_stem(name: &str) -> &str {
    ARCHIVE_EXTS
        .iter()
        .find_map(|(ext, _)| name.strip_suffix(ext))
        .unwrap_or(name)
}

fn extract_strategy(name: &str) -> Option<&'static str> {
    ARCHIVE_EXTS
        .iter()
        .find(|(ext, _)| name.ends_with(ext))
        .map(|(_, strategy)| *strategy)
}

fn installable_assets(assets: &[serde_json::Value]) -> Vec<AssetInfo> {
    assets
        .iter()
        .filter_map(|a| {
            let name = a.get("name").and_then(|v| v.as_str())?;
            extract_strategy(name)?;
            Some(AssetInfo {
                name: name.to_string(),
                url: a
                    .get("browser_download_url")
                    .and_then(|v| v.as_str())?
                    .to_string(),
                size: a.get("size").and_then(|v| v.as_u64()).unwrap_or(0),
            })
        })
        .collect()
}

fn default_asset(assets: &[AssetInfo]) -> Option<AssetInfo> {
    let native = |a: &&AssetInfo| !a.name.contains("arm64") && !a.name.contains("aarch64");
    assets
        .iter()
        .find(native)
        .or_else(|| assets.first())
        .cloned()
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

        let assets = installable_assets(assets);
        let Some(default) = default_asset(&assets) else {
            continue;
        };

        if tag.is_empty() {
            continue;
        }

        out.push(ReleaseInfo {
            tag,
            published_at: published,
            asset_name: default.name,
            asset_url: default.url,
            asset_size: default.size,
            assets,
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

    match extract_strategy(&release.asset_name).unwrap_or_default() {
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
        _ => {
            let _ = fs::remove_dir_all(&staging);
            return Err(anyhow!("unknown archive type: {}", release.asset_name));
        }
    }

    let stem = asset_stem(&release.asset_name);
    let final_dir = dest_root.join(if stem.is_empty() { &release.tag } else { stem });
    if let Some(old) = installed_dir(&source.name, dest_root, &release.tag)
        && old.file_name().and_then(|n| n.to_str()) == Some(release.tag.as_str())
    {
        let _ = fs::remove_dir_all(&old);
    }
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
            && sc.source == source.name
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            out.push(name.to_string());
        }
    }
    out.sort();
    out
}

pub fn installed_dir(source_name: &str, dest_root: &Path, tag: &str) -> Option<PathBuf> {
    fs::read_dir(dest_root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.is_dir()
                && read_sidecar(p).is_some_and(|sc| sc.source == source_name && sc.tag == tag)
        })
}

pub fn delete_version(source: &ArchiveSource, dest_root: &Path, name: &str) -> Result<()> {
    let dir = dest_root.join(name);
    if !dir.exists() {
        return Ok(());
    }
    match read_sidecar(&dir) {
        Some(sc) if sc.source == source.name => {
            fs::remove_dir_all(&dir)?;
            Ok(())
        }
        _ => Err(anyhow!(
            "refusing to delete {}: not installed by {}",
            dir.display(),
            source.name
        )),
    }
}
