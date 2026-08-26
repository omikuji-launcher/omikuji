use anyhow::{Result, anyhow, bail};
use futures_util::StreamExt;
use md5::{Digest, Md5};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;

use super::api::{PatchConfig, PatchIndexFile, ResourceFile, ResourceInfo};
use super::krpdiff::Krpdiff;
use super::source::sync_file;
use crate::downloads::limits::GachaLimits;
use crate::downloads::{ControlSignal, DownloadEntry, DownloadStatus, check_control, set_status};
use crate::gacha::file_sync::{SyncProgress, download_one, sanitize_rel};

pub(super) async fn run_patch_update(
    entry: &DownloadEntry,
    info: &ResourceInfo,
    patch: &PatchConfig,
    pidx: &PatchIndexFile,
) -> Result<bool> {
    if pidx.resource.is_empty() {
        bail!("patch indexFile returned zero resources");
    }
    if !pidx.zip_infos.is_empty() || !pidx.patch_infos.is_empty() {
        bail!(
            "patch indexFile carries {} zipInfos and {} patchInfos payloads, which are not implemented",
            pidx.zip_infos.len(),
            pidx.patch_infos.len()
        );
    }
    if pidx.group_infos.is_empty() {
        bail!("patch indexFile declares no groupInfos to apply");
    }

    let install_root = entry.install_path.clone();
    let staging = install_root.join(".omikuji-patch");
    let dl_root = staging.join("dl");
    let out_root = staging.join("out");
    std::fs::create_dir_all(&dl_root)?;
    std::fs::create_dir_all(&out_root)?;

    let total: u64 = pidx.resource.iter().map(|r| r.size).sum();
    let progress = SyncProgress::new(total);

    let id = entry.id.clone();
    let cdn = info.cdn_url.clone();
    let patch_base = format!("{}{}", info.cdn_url, patch.base_url_rel);
    let resources = pidx.resource.clone();
    let dl_for_workers = dl_root.clone();
    let progress_for_workers = progress.clone();

    let stream = futures_util::stream::iter(resources.into_iter().map(move |file| {
        let id = id.clone();
        let dl_root = dl_for_workers.clone();
        let progress = progress_for_workers.clone();
        let base = if file.from_folder.is_empty() {
            patch_base.clone()
        } else {
            format!("{}{}", cdn, file.from_folder)
        };
        async move {
            if check_control(&id) != ControlSignal::None {
                return Ok::<_, anyhow::Error>(());
            }
            download_one(&id, &sync_file(&file, &base), &dl_root, &progress).await
        }
    }))
    .buffer_unordered(GachaLimits::load().connections);

    tokio::pin!(stream);
    while let Some(res) = stream.next().await {
        res?;
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(false);
        }
    }

    set_status(&entry.id, DownloadStatus::Patching);
    let mut pending_src: HashMap<&str, usize> = HashMap::new();
    for g in &pidx.group_infos {
        for s in &g.src_files {
            *pending_src.entry(s.dest.as_str()).or_default() += 1;
        }
    }
    let mut staged: Vec<&str> = Vec::new();
    for group in &pidx.group_infos {
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(false);
        }
        let diff_path = dl_root.join(sanitize_rel(&group.dest));
        if diff_path.exists() {
            let apply = {
                let diff_path = diff_path.clone();
                let install_root = install_root.clone();
                let out_root = out_root.clone();
                let dst_files = group.dst_files.clone();
                tokio::task::spawn_blocking(move || {
                    apply_group(&diff_path, &install_root, &out_root, &dst_files)
                })
            };
            match apply.await? {
                Ok(()) => staged.extend(group.dst_files.iter().map(|f| f.dest.as_str())),
                Err(e) => {
                    tracing::warn!(
                        "krpdiff group {} failed, its files fall back to full download: {}",
                        group.dest,
                        e
                    );
                    for f in &group.dst_files {
                        let _ = std::fs::remove_file(out_root.join(sanitize_rel(&f.dest)));
                    }
                }
            }
            let _ = std::fs::remove_file(&diff_path);
        }
        for s in &group.src_files {
            if let Some(n) = pending_src.get_mut(s.dest.as_str()) {
                *n = n.saturating_sub(1);
            }
        }
        flush_staged(&mut staged, &pending_src, &out_root, &install_root);
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
                &sync_file(f, &info.base_url),
                &out_root,
                &progress,
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
        move_file(&src, &install_root.join(&rel))?;
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

fn move_file(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(from, to)?;
    Ok(())
}

fn flush_staged<'a>(
    staged: &mut Vec<&'a str>,
    pending_src: &HashMap<&'a str, usize>,
    out_root: &Path,
    install_root: &Path,
) {
    staged.retain(|dest| {
        if pending_src.get(*dest).copied().unwrap_or(0) > 0 {
            return true;
        }
        let rel = sanitize_rel(dest);
        let src = out_root.join(&rel);
        src.exists() && move_file(&src, &install_root.join(&rel)).is_err()
    });
}
