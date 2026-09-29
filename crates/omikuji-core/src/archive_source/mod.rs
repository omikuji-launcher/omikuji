// generic archive-source fetcher for runners (wine-ge, proton-ge, spritz, ...)
// and dll packs (dxvk, vkd3d-proton, dxvk-nvapi, d3d-extras). callers pass a dest_root; thats the only real difference between a runner and a dll pack install.
// adding a new source is a 5-line paste in settings.rs, no code change here. yayyyy =m=

use anyhow::{Result, anyhow};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::archive::{self, ArchiveKind};
use crate::components_config::{ArchiveSource, SourceCategory};
use crate::event_queue::EventQueue;
use crate::http;

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

const GITHUB_HOST: &str = "github.com";
const GITHUB_API_HOST: &str = "api.github.com";
const GITHUB_API_PREFIX: &[&str] = &["repos"];
const FORGE_API_PREFIX: &[&str] = &["api", "v1", "repos"];

#[derive(Debug, Clone)]
pub struct RepoLink {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub tag: Option<String>,
}

impl RepoLink {
    pub fn parse(link: &str) -> Option<Self> {
        let url = Url::parse(link.trim()).ok()?;
        let path: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
        let (host, rest) = match url.host_str()? {
            GITHUB_API_HOST => (GITHUB_HOST, path.strip_prefix(GITHUB_API_PREFIX)?),
            host => (host, path.strip_prefix(FORGE_API_PREFIX).unwrap_or(&path)),
        };
        let [owner, repo, tail @ ..] = rest else {
            return None;
        };
        let tag = match tail {
            ["releases", "tag", tag, ..] => Some(tag.to_string()),
            [] | ["releases" | "tags", ..] => None,
            _ => return None,
        };
        Some(Self {
            host: host.to_string(),
            owner: owner.to_string(),
            repo: repo.trim_end_matches(".git").to_string(),
            tag,
        })
    }

    pub fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    pub fn releases_api_url(&self) -> String {
        match self.host.as_str() {
            GITHUB_HOST => format!("https://{GITHUB_API_HOST}/repos/{}/releases", self.slug()),
            host => format!("https://{host}/api/v1/repos/{}/releases", self.slug()),
        }
    }
}

pub fn normalize_releases_url(link: &str) -> String {
    RepoLink::parse(link).map_or_else(|| link.trim().to_string(), |repo| repo.releases_api_url()) // uhm
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallChannel {
    Runners,
    Layers,
    RunnersLatest,
}

impl InstallChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runners => SourceCategory::Runners.as_str(),
            Self::Layers => SourceCategory::Layers.as_str(),
            Self::RunnersLatest => "runners_latest",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArchiveEvent {
    Started {
        channel: InstallChannel,
        source: String,
        tag: String,
    },
    Progress {
        channel: InstallChannel,
        source: String,
        tag: String,
        phase: String,
        percent: f64,
    },
    Completed {
        channel: InstallChannel,
        source: String,
        tag: String,
        install_dir: String,
    },
    Failed {
        channel: InstallChannel,
        source: String,
        tag: String,
        error: String,
    },
}

static EVENTS: EventQueue<ArchiveEvent> = EventQueue::new(0);

pub fn drain_events() -> Vec<ArchiveEvent> {
    EVENTS.drain()
}

fn push(ev: ArchiveEvent) {
    EVENTS.push(ev);
}

fn installable_assets(assets: &[serde_json::Value]) -> Vec<AssetInfo> {
    assets
        .iter()
        .filter_map(|a| {
            let name = a.get("name").and_then(|v| v.as_str())?;
            ArchiveKind::from_name(name)?;
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

pub fn matches_priority(asset_name: &str, priority: &[String]) -> bool {
    priority
        .iter()
        .any(|want| asset_name.contains(want.as_str()))
}

fn pick_asset(assets: &[AssetInfo], priority: &[String]) -> Option<AssetInfo> {
    priority
        .iter()
        .find_map(|want| {
            assets
                .iter()
                .find(|a| a.name.contains(want.as_str()))
                .cloned()
        })
        .or_else(|| default_asset(assets))
}

fn default_asset(assets: &[AssetInfo]) -> Option<AssetInfo> {
    let native = |a: &&AssetInfo| !a.name.contains("arm64") && !a.name.contains("aarch64");
    assets
        .iter()
        .find(native)
        .or_else(|| assets.first())
        .cloned()
}

async fn fetch_releases(api_url: &str) -> Result<Vec<serde_json::Value>> {
    let resp = http::client()
        .get(api_url)
        .query(&[("per_page", "100")])
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("release list ({}): {}", api_url, e))?;
    Ok(resp.json().await?)
}

pub async fn fetch_versions(source: &ArchiveSource) -> Result<Vec<ReleaseInfo>> {
    let releases = fetch_releases(&source.api_url).await?;

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
        let Some(default) = pick_asset(&assets, &source.asset_priority) else {
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

pub async fn install_asset(api_url: &str, asset_name: &str, dest_dir: &Path) -> Result<PathBuf> {
    let releases = fetch_releases(api_url).await?;
    let asset = releases
        .iter()
        .filter_map(|r| r.get("assets").and_then(|v| v.as_array()))
        .flatten()
        .find(|a| a.get("name").and_then(|v| v.as_str()) == Some(asset_name))
        .ok_or_else(|| anyhow!("no asset named {} at {}", asset_name, api_url))?;
    let url = asset
        .get("browser_download_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("asset {} has no download url", asset_name))?;

    let bytes = http::client()
        .get(url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("download {}: {}", url, e))?
        .bytes()
        .await?;

    fs::create_dir_all(dest_dir)?;
    let dest = dest_dir.join(asset_name);
    fs::write(&dest, &bytes)?;
    Ok(dest)
}

pub async fn install_version(
    channel: InstallChannel,
    source: &ArchiveSource,
    release: &ReleaseInfo,
    dest_root: &Path,
) -> Result<PathBuf> {
    install_named(channel, source, release, dest_root, None).await
}

pub async fn install_version_named(
    channel: InstallChannel,
    source: &ArchiveSource,
    release: &ReleaseInfo,
    dest_root: &Path,
    dir_name: &str,
) -> Result<PathBuf> {
    install_named(channel, source, release, dest_root, Some(dir_name)).await
}

async fn install_named(
    channel: InstallChannel,
    source: &ArchiveSource,
    release: &ReleaseInfo,
    dest_root: &Path,
    dir_name: Option<&str>,
) -> Result<PathBuf> {
    push(ArchiveEvent::Started {
        channel,
        source: source.name.clone(),
        tag: release.tag.clone(),
    });

    match install_inner(channel, source, release, dest_root, dir_name).await {
        Ok(dir) => {
            push(ArchiveEvent::Completed {
                channel,
                source: source.name.clone(),
                tag: release.tag.clone(),
                install_dir: dir.to_string_lossy().into_owned(),
            });
            Ok(dir)
        }
        Err(e) => {
            let msg = format!("{:#}", e);
            push(ArchiveEvent::Failed {
                channel,
                source: source.name.clone(),
                tag: release.tag.clone(),
                error: msg.clone(),
            });
            Err(anyhow!(msg))
        }
    }
}

struct ExtractProgress<'a> {
    inner: Cursor<&'a [u8]>,
    total: u64,
    last_pct: f64,
    channel: InstallChannel,
    source: String,
    tag: String,
}

impl<'a> ExtractProgress<'a> {
    fn new(
        bytes: &'a [u8],
        channel: InstallChannel,
        source: &ArchiveSource,
        release: &ReleaseInfo,
    ) -> Self {
        Self {
            inner: Cursor::new(bytes),
            total: bytes.len() as u64,
            last_pct: -1.0,
            channel,
            source: source.name.clone(),
            tag: release.tag.clone(),
        }
    }
}

impl Seek for ExtractProgress<'_> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl Read for ExtractProgress<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if self.total == 0 {
            return Ok(n);
        }
        let pct = (self.inner.position() as f64 / self.total as f64) * 100.0;
        if pct - self.last_pct >= 1.0 {
            push(ArchiveEvent::Progress {
                channel: self.channel,
                source: self.source.clone(),
                tag: self.tag.clone(),
                phase: "extracting".into(),
                percent: pct,
            });
            self.last_pct = pct;
        }
        Ok(n)
    }
}

async fn install_inner(
    channel: InstallChannel,
    source: &ArchiveSource,
    release: &ReleaseInfo,
    dest_root: &Path,
    dir_name: Option<&str>,
) -> Result<PathBuf> {
    fs::create_dir_all(dest_root)?;

    let bytes = download_bytes(channel, source, release).await?;

    push(ArchiveEvent::Progress {
        channel,
        source: source.name.clone(),
        tag: release.tag.clone(),
        phase: "extracting".into(),
        percent: 0.0,
    });

    let kind = ArchiveKind::from_name(&release.asset_name)
        .ok_or_else(|| anyhow!("unknown archive type: {}", release.asset_name))?;
    let staging = dest_root.join(format!(".staging-{}-{}", source.name, release.tag));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;
    kind.unpack(
        ExtractProgress::new(&bytes, channel, source, release),
        &staging,
    )?;

    let stem = archive::stem(&release.asset_name);
    let default_name = if stem.is_empty() {
        release.tag.as_str()
    } else {
        stem
    };
    let final_dir = dest_root.join(dir_name.unwrap_or(default_name));
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
    channel: InstallChannel,
    source: &ArchiveSource,
    release: &ReleaseInfo,
) -> Result<Vec<u8>> {
    http::download_with_progress(&release.asset_url, release.asset_size, |pct| {
        push(ArchiveEvent::Progress {
            channel,
            source: source.name.clone(),
            tag: release.tag.clone(),
            phase: "downloading".into(),
            percent: pct,
        });
    })
    .await
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
