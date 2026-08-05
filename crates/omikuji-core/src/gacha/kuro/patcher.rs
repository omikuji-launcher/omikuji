use anyhow::{Result, anyhow, bail};
use futures_util::StreamExt;
use md5::{Digest, Md5};
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Instant;

use super::api::{self, PatchConfig, ResourceFile, ResourceInfo};
use super::krpdiff::Krpdiff;
use super::source::{PARALLEL_FILES, download_one, sanitize_rel};
use crate::downloads::{ControlSignal, DownloadEntry, DownloadStatus, check_control, set_status};

pub(super) async fn run_patch_update(
    entry: &DownloadEntry,
    info: &ResourceInfo,
    patch: &PatchConfig,
) -> Result<bool> {
    let index_url = format!("{}{}", info.cdn_url, patch.index_file_rel);
    let pidx = api::fetch_patch_index(&index_url).await?;
    if pidx.resource.is_empty() {
        bail!("patch indexFile returned zero resources");
    }

    let install_root = entry.install_path.clone();
    let staging = install_root.join(".omikuji-patch");
    let dl_root = staging.join("dl");
    let out_root = staging.join("out");
    std::fs::create_dir_all(&dl_root)?;
    std::fs::create_dir_all(&out_root)?;

    let total: u64 = pidx.resource.iter().map(|r| r.size).sum();
    let downloaded = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let id = entry.id.clone();
    let cdn = info.cdn_url.clone();
    let patch_base = format!("{}{}", info.cdn_url, patch.base_url_rel);
    let resources = pidx.resource.clone();
    let dl_for_workers = dl_root.clone();
    let downloaded_for_workers = downloaded.clone();

    let stream = futures_util::stream::iter(resources.into_iter().map(move |file| {
        let id = id.clone();
        let downloaded = downloaded_for_workers.clone();
        let dl_root = dl_for_workers.clone();
        let base = if file.from_folder.is_empty() {
            patch_base.clone()
        } else {
            format!("{}{}", cdn, file.from_folder)
        };
        async move {
            if check_control(&id) != ControlSignal::None {
                return Ok::<_, anyhow::Error>(());
            }
            download_one(&id, &file, &base, &dl_root, &downloaded, total, start).await
        }
    }))
    .buffer_unordered(PARALLEL_FILES);

    tokio::pin!(stream);
    while let Some(res) = stream.next().await {
        res?;
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(false);
        }
    }

    set_status(&entry.id, DownloadStatus::Patching);
    for group in &pidx.group_infos {
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(false);
        }
        let diff_path = dl_root.join(sanitize_rel(&group.dest));
        if !diff_path.exists() {
            continue;
        }
        let apply = {
            let diff_path = diff_path.clone();
            let install_root = install_root.clone();
            let out_root = out_root.clone();
            let dst_files = group.dst_files.clone();
            tokio::task::spawn_blocking(move || {
                apply_group(&diff_path, &install_root, &out_root, &dst_files)
            })
        };
        if let Err(e) = apply.await? {
            tracing::warn!(
                "krpdiff group {} failed, its files fall back to full download: {}",
                group.dest,
                e
            );
            for f in &group.dst_files {
                let _ = std::fs::remove_file(out_root.join(sanitize_rel(&f.dest)));
            }
        }
        let _ = std::fs::remove_file(&diff_path);
    }

    let mut fallback: Vec<ResourceFile> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for f in pidx.group_infos.iter().flat_map(|g| g.dst_files.iter()) {
        if !seen.insert(f.dest.as_str()) {
            continue;
        }
        let rel = sanitize_rel(&f.dest);
        let in_out = matches!(std::fs::metadata(out_root.join(&rel)), Ok(m) if m.len() == f.size);
        let in_place =
            matches!(std::fs::metadata(install_root.join(&rel)), Ok(m) if m.len() == f.size);
        if !in_out && !in_place {
            fallback.push(f.clone());
        }
    }
    if !fallback.is_empty() {
        tracing::warn!("kuro patch: {} files need a full download", fallback.len());
        set_status(&entry.id, DownloadStatus::Downloading);
        for f in &fallback {
            if check_control(&entry.id) != ControlSignal::None {
                return Ok(false);
            }
            download_one(
                &entry.id,
                f,
                &info.base_url,
                &out_root,
                &downloaded,
                total,
                start,
            )
            .await?;
        }
        set_status(&entry.id, DownloadStatus::Patching);
    }

    let group_names: HashSet<&str> = pidx.group_infos.iter().map(|g| g.dest.as_str()).collect();
    for file in &pidx.resource {
        if group_names.contains(file.dest.as_str()) {
            continue;
        }
        let rel = sanitize_rel(&file.dest);
        let src = dl_root.join(&rel);
        if !src.exists() {
            continue;
        }
        let dst = install_root.join(&rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&src, &dst)?;
    }
    move_tree(&out_root, &install_root)?;

    for stale in &pidx.delete_files {
        let p = install_root.join(sanitize_rel(stale));
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }
    let _ = std::fs::remove_dir_all(&staging);
    Ok(true)
}

fn apply_group(
    diff: &Path,
    old_root: &Path,
    out_root: &Path,
    dst_files: &[ResourceFile],
) -> Result<()> {
    let kr = Krpdiff::open(diff)?;
    kr.apply(old_root, out_root, |_| {})?;
    for f in dst_files {
        let path = out_root.join(sanitize_rel(&f.dest));
        let size = std::fs::metadata(&path)
            .map_err(|e| anyhow!("patched output missing {}: {}", f.dest, e))?
            .len();
        if size != f.size {
            bail!(
                "patched output {} size mismatch: expected {}, got {}",
                f.dest,
                f.size,
                size
            );
        }
        if !f.md5.is_empty() && file_md5(&path)? != f.md5.to_lowercase() {
            bail!("patched output {} md5 mismatch", f.dest);
        }
    }
    Ok(())
}

fn file_md5(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn move_tree(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            std::fs::create_dir_all(&target)?;
            move_tree(&entry.path(), &target)?;
        } else {
            std::fs::rename(entry.path(), &target)?;
        }
    }
    Ok(())
}
