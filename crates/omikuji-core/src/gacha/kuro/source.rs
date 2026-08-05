use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use super::api;
use crate::downloads::{
    ControlSignal, DownloadEntry, DownloadKind, DownloadSource, check_control, report_progress,
};

pub(super) const PARALLEL_FILES: usize = 8;

pub struct KuroSource;

#[async_trait]
impl DownloadSource for KuroSource {
    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        run_install_or_update(entry).await
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        run_install_or_update(entry).await
    }
}

async fn run_install_or_update(entry: &DownloadEntry) -> Result<()> {
    if !matches!(
        entry.kind,
        DownloadKind::Install | DownloadKind::Update { .. }
    ) {
        return Err(anyhow!("KuroSource: unexpected DownloadKind"));
    }

    let (manifest, edition_id, _) = crate::gacha::strategies::find_for_app_id(&entry.app_id)
        .ok_or_else(|| anyhow!("no manifest for app_id {}", entry.app_id))?;
    let game_slug = manifest.game_slug.clone();

    let info = api::fetch_resource_info(&manifest, &edition_id).await?;

    if let DownloadKind::Update { from_version } = &entry.kind
        && let Some(pc) = info.matching_patch(from_version)
    {
        match super::patcher::run_patch_update(entry, &info, pc).await {
            Ok(true) => {
                super::set_installed_version(&game_slug, &edition_id, &info.version);
                return Ok(());
            }
            Ok(false) => return Ok(()),
            Err(e) => tracing::warn!("kuro delta update failed, falling back to full sync: {}", e),
        }
    }

    let index = api::fetch_index_file(&info.index_file_url).await?;
    if index.resource.is_empty() {
        return Err(anyhow!("indexFile returned zero resources"));
    }

    let total_bytes: u64 = index.resource.iter().map(|r| r.size).sum();
    let downloaded = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let install_root = entry.install_path.clone();
    std::fs::create_dir_all(&install_root)?;

    let id = entry.id.clone();
    let base_url = info.base_url.clone();
    let resources = index.resource.clone();
    let install_root_for_workers = install_root.clone();

    let stream = futures_util::stream::iter(resources.into_iter().map(move |file| {
        let id = id.clone();
        let downloaded = downloaded.clone();
        let install_root = install_root_for_workers.clone();
        let base_url = base_url.clone();
        let start = start;
        let total = total_bytes;
        async move {
            if check_control(&id) != ControlSignal::None {
                return Ok::<_, anyhow::Error>(());
            }
            download_one(
                &id,
                &file,
                &base_url,
                &install_root,
                &downloaded,
                total,
                start,
            )
            .await
        }
    }))
    .buffer_unordered(PARALLEL_FILES);

    tokio::pin!(stream);
    while let Some(res) = stream.next().await {
        res?;
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }
    }

    for stale in &index.delete_files {
        let p = install_root.join(stale);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }
    super::set_installed_version(&game_slug, &edition_id, &info.version);
    Ok(())
}

pub(super) async fn download_one(
    id: &str,
    file: &api::ResourceFile,
    base_url: &str,
    install_root: &Path,
    downloaded: &Arc<AtomicU64>,
    total: u64,
    start: Instant,
) -> Result<()> {
    let rel = sanitize_rel(&file.dest);
    let dest_path = install_root.join(&rel);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // size-gate instead of md5, hot path; matches aag's integration approach.
    if matches!(std::fs::metadata(&dest_path), Ok(m) if m.len() == file.size) {
        downloaded.fetch_add(file.size, Ordering::Relaxed);
        tick_progress(id, downloaded, total, start);
        return Ok(());
    }

    let existing = std::fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
    let resume = existing > 0 && existing < file.size;
    // pgr ships resource paths with a leading slash; wuwa doesnt, strip both
    // so the filesystem join and url prefix join dont produce double slashes
    let dest_clean = file.dest.trim_start_matches(['/', '\\']);
    let url = format!("{}{}", base_url, urlencode_path(dest_clean));

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| anyhow!("reqwest client: {}", e))?;
    let mut req = client.get(&url);
    if resume {
        req = req.header("Range", format!("bytes={}-", existing));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| anyhow!("GET {}: {}", url, e))?;
    let status = resp.status();
    if !(status.is_success() || status.as_u16() == 206) {
        anyhow::bail!("GET {}: http {}", url, status);
    }

    // only treat as resumed if the server actually returned 206; full body means overwrite
    let append = resp.status().as_u16() == 206 && resume;
    let mut writer: Box<dyn std::io::Write + Send> = if append {
        Box::new(std::fs::OpenOptions::new().append(true).open(&dest_path)?)
    } else {
        downloaded.fetch_sub(
            existing.min(downloaded.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
        Box::new(std::fs::File::create(&dest_path)?)
    };
    if append {
        downloaded.fetch_add(existing, Ordering::Relaxed);
    }

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if check_control(id) != ControlSignal::None {
            return Ok(());
        }
        let bytes = chunk.map_err(|e| anyhow!("network: {}", e))?;
        use std::io::Write;
        writer
            .write_all(&bytes)
            .map_err(|e| anyhow!("write: {}", e))?;
        downloaded.fetch_add(bytes.len() as u64, Ordering::Relaxed);
        tick_progress(id, downloaded, total, start);
    }
    writer.flush()?;
    Ok(())
}

fn tick_progress(id: &str, downloaded: &AtomicU64, total: u64, start: Instant) {
    let done = downloaded.load(Ordering::Relaxed);
    let pct = if total > 0 {
        (done as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let elapsed = start.elapsed().as_secs_f64().max(0.001);
    let bps = (done as f64 / elapsed) as u64;
    report_progress(id, pct, done, total, bps);
}

// drop leading slashes and .. to prevent path traversal outside install_root
pub(super) fn sanitize_rel(dest: &str) -> PathBuf {
    let cleaned: Vec<&str> = dest
        .split(['/', '\\'])
        .filter(|p| !p.is_empty() && *p != "." && *p != "..")
        .collect();
    cleaned.iter().collect()
}

fn urlencode_path(dest: &str) -> String {
    dest.replace(' ', "%20")
}
