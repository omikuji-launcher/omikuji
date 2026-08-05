use anyhow::{Result, anyhow};
use futures_util::stream::{self, StreamExt};
use md5::{Digest, Md5};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::api::{DownloadInfo, SophonManifestEntry};
use super::manifest::fetch_build_manifest;
use super::patcher::{CancelFn, ProgressFn, ProgressReport, Stage};
use super::protos::{FileChunk, ManifestFile};

const PARALLEL_FILES: usize = 16;
const PARALLEL_CHUNKS_PER_FILE: usize = 4;

pub async fn apply_install(
    entries: &[SophonManifestEntry],
    target_dir: PathBuf,
    on_progress: ProgressFn,
    is_cancelled: CancelFn,
) -> Result<()> {
    if entries.is_empty() {
        return Err(anyhow!("no manifest entries for install"));
    }

    let mut all_files: Vec<(ManifestFile, Arc<DownloadInfo>)> = Vec::new();
    let mut total_bytes: u64 = 0;

    for entry in entries {
        let manifest = fetch_build_manifest(entry).await?;
        let download = Arc::new(entry.chunk_download.clone());
        for f in manifest.files {
            if f.r#type == 64 {
                continue;
            }
            total_bytes = total_bytes.saturating_add(f.size);
            all_files.push((f, download.clone()));
        }
    }

    if all_files.is_empty() {
        return Err(anyhow!("manifest contains zero files"));
    }

    std::fs::create_dir_all(&target_dir)?;

    let done_bytes = Arc::new(AtomicU64::new(0));
    let session_bytes = Arc::new(AtomicU64::new(0));
    let on_progress = Arc::new(on_progress);
    let is_cancelled = Arc::new(is_cancelled);
    let client = Arc::new(
        reqwest::Client::builder()
            .build()
            .map_err(|e| anyhow!("reqwest client: {}", e))?,
    );

    on_progress(ProgressReport {
        stage: Stage::Downloading,
        current: 0,
        total: all_files.len() as u64,
        bytes_done: 0,
        bytes_total: total_bytes,
        bytes_session: 0,
    });

    let target = target_dir.clone();
    let progress_total = total_bytes;
    let file_count = all_files.len() as u64;

    let results: Vec<Result<()>> = stream::iter(all_files.into_iter().enumerate().map(
        |(idx, (file, download))| {
            let target = target.clone();
            let done = done_bytes.clone();
            let session = session_bytes.clone();
            let on_progress = on_progress.clone();
            let is_cancelled = is_cancelled.clone();
            let client = client.clone();
            async move {
                if is_cancelled() {
                    return Ok(());
                }
                install_one_file(
                    &file,
                    &download,
                    &target,
                    &client,
                    &done,
                    &session,
                    progress_total,
                    file_count,
                    idx as u64 + 1,
                    &on_progress,
                    &is_cancelled,
                )
                .await
            }
        },
    ))
    .buffer_unordered(PARALLEL_FILES)
    .collect()
    .await;

    for r in results {
        r?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn install_one_file(
    file: &ManifestFile,
    download: &DownloadInfo,
    target_dir: &Path,
    client: &Arc<reqwest::Client>,
    done_bytes: &Arc<AtomicU64>,
    session_bytes: &Arc<AtomicU64>,
    total_bytes: u64,
    file_total: u64,
    file_index: u64,
    on_progress: &Arc<ProgressFn>,
    is_cancelled: &Arc<CancelFn>,
) -> Result<()> {
    let dest = target_dir.join(sanitize_rel(&file.name));
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let parts_p = parts_path(&dest);

    if let Ok(meta) = std::fs::metadata(&dest)
        && meta.len() == file.size
        && !parts_p.exists()
    {
        let new_done = done_bytes.fetch_add(file.size, Ordering::Relaxed) + file.size;
        let sess = session_bytes.load(Ordering::Relaxed);
        report(
            on_progress,
            file_index,
            file_total,
            new_done,
            total_bytes,
            sess,
        );
        return Ok(());
    }

    let _ = OpenOptions::new().create(true).append(true).open(&parts_p);

    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&dest)
        .map_err(|e| anyhow!("open {}: {}", dest.display(), e))?;
    f.set_len(file.size)?;
    let handle = Arc::new(f);

    let completed = Arc::new(Mutex::new(read_completed_chunks(&dest)));
    {
        let c = completed.lock().unwrap();
        let already_done: u64 = file
            .chunks
            .iter()
            .enumerate()
            .filter(|(i, _)| c.contains(i))
            .map(|(_, ch)| ch.chunk_decompressed_size)
            .sum();
        if already_done > 0 {
            let new_done = done_bytes.fetch_add(already_done, Ordering::Relaxed) + already_done;
            let sess = session_bytes.load(Ordering::Relaxed);
            report(
                on_progress,
                file_index,
                file_total,
                new_done,
                total_bytes,
                sess,
            );
        }
    }

    let pending: Vec<(usize, FileChunk)> = file
        .chunks
        .iter()
        .enumerate()
        .filter(|(i, _)| !completed.lock().unwrap().contains(i))
        .map(|(i, c)| (i, c.clone()))
        .collect();

    if !pending.is_empty() {
        let dest_for_workers = dest.clone();
        let results: Vec<Result<()>> = stream::iter(pending.into_iter().map(|(idx, chunk)| {
            let client = client.clone();
            let download_url_prefix = download.url_prefix.clone();
            let download_url_suffix = download.url_suffix.clone();
            let download_compression = download.compression;
            let handle = handle.clone();
            let done_bytes = done_bytes.clone();
            let session_bytes = session_bytes.clone();
            let on_progress = on_progress.clone();
            let is_cancelled = is_cancelled.clone();
            let completed = completed.clone();
            let dest = dest_for_workers.clone();
            async move {
                if is_cancelled() {
                    return Ok(());
                }
                let url = format!(
                    "{}{}/{}",
                    download_url_prefix, download_url_suffix, chunk.chunk_name
                );
                let resp = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| anyhow!("GET {} failed: {}", url, e))?;
                if !resp.status().is_success() {
                    anyhow::bail!("GET {} http {}", url, resp.status());
                }
                let raw = resp
                    .bytes()
                    .await
                    .map_err(|e| anyhow!("read chunk body: {}", e))?;
                let decompressed = if download_compression == 1 {
                    tokio::task::spawn_blocking(move || zstd::decode_all(&*raw))
                        .await
                        .map_err(|e| anyhow!("join zstd decode: {}", e))?
                        .map_err(|e| anyhow!("zstd decode chunk: {}", e))?
                } else {
                    raw.to_vec()
                };
                let h = handle.clone();
                let offset = chunk.chunk_on_file_offset;
                tokio::task::spawn_blocking(move || h.write_all_at(&decompressed, offset))
                    .await
                    .map_err(|e| anyhow!("join chunk write: {}", e))?
                    .map_err(|e| anyhow!("write chunk: {}", e))?;
                completed.lock().unwrap().insert(idx);
                let _ = mark_chunk_complete(&dest, idx);
                let new_done = done_bytes
                    .fetch_add(chunk.chunk_decompressed_size, Ordering::Relaxed)
                    + chunk.chunk_decompressed_size;
                let new_session = session_bytes
                    .fetch_add(chunk.chunk_decompressed_size, Ordering::Relaxed)
                    + chunk.chunk_decompressed_size;
                report(
                    &on_progress,
                    file_index,
                    file_total,
                    new_done,
                    total_bytes,
                    new_session,
                );
                Ok(())
            }
        }))
        .buffer_unordered(PARALLEL_CHUNKS_PER_FILE)
        .collect()
        .await;

        for r in results {
            r?;
        }
    }

    drop(handle);

    if is_cancelled() {
        return Ok(());
    }

    let dest_for_hash = dest.clone();
    let got = tokio::task::spawn_blocking(move || hash_file(&dest_for_hash))
        .await
        .map_err(|e| anyhow!("join md5 verify: {}", e))??;
    if got != file.md5 {
        anyhow::bail!(
            "md5 mismatch on {}: expected {}, got {}",
            file.name,
            file.md5,
            got
        );
    }

    clear_parts(&dest);
    Ok(())
}

fn report(
    cb: &Arc<ProgressFn>,
    current: u64,
    total: u64,
    bytes_done: u64,
    bytes_total: u64,
    bytes_session: u64,
) {
    cb(ProgressReport {
        stage: Stage::Downloading,
        current,
        total,
        bytes_done,
        bytes_total,
        bytes_session,
    });
}

fn hash_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sanitize_rel(name: &str) -> PathBuf {
    let cleaned: Vec<&str> = name
        .split(['/', '\\'])
        .filter(|p| !p.is_empty() && *p != "." && *p != "..")
        .collect();
    cleaned.iter().collect()
}

fn parts_path(dest: &Path) -> PathBuf {
    let mut p = dest.to_path_buf();
    let mut name = p.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(".omikuji-parts");
    p.set_file_name(name);
    p
}

fn read_completed_chunks(dest: &Path) -> HashSet<usize> {
    let Ok(content) = std::fs::read_to_string(parts_path(dest)) else {
        return HashSet::new();
    };
    content
        .lines()
        .filter_map(|l| l.trim().parse::<usize>().ok())
        .collect()
}

fn mark_chunk_complete(dest: &Path, idx: usize) -> std::io::Result<()> {
    let p = parts_path(dest);
    let mut f = OpenOptions::new().create(true).append(true).open(&p)?;
    writeln!(f, "{}", idx)
}

fn clear_parts(dest: &Path) {
    let _ = std::fs::remove_file(parts_path(dest));
}
