use anyhow::{Result, anyhow, bail};
use futures_util::{StreamExt, TryStreamExt, stream};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::api::{Pack, PatchConfig, PatchIndexFile, ResourceFile};
use super::krpdiff::Krpdiff;
use super::source::sync_file;
use crate::downloads::limits::GachaLimits;
use crate::downloads::{ControlSignal, DownloadEntry, DownloadStatus, check_control, set_status};
use crate::fs_util;
use crate::gacha::file_sync::{
    Skip, SyncProgress, download_one, expected_md5, file_md5, is_stale, sanitize_rel,
};

// interrupt keeps staging so a resume reuses the pulled diffs
pub(super) async fn run_patch_update(
    entry: &DownloadEntry,
    pack_id: &str,
    pack: &Pack,
    patch: &PatchConfig,
    pidx: &PatchIndexFile,
) -> Result<()> {
    let staging = staging_root(&entry.install_path).join(pack_id);
    let result = patch_into_staging(entry, &staging, pack, patch, pidx).await;
    if result.is_err() {
        let _ = fs_err::remove_dir_all(&staging);
    }
    result
}

const STAGING_DIR: &str = ".omikuji-patch";

fn staging_root(install_root: &Path) -> PathBuf {
    install_root.join(STAGING_DIR)
}

fn done_marker(install_root: &Path, pack_id: &str) -> PathBuf {
    staging_root(install_root).join(format!("{pack_id}.done"))
}

pub(super) fn discard_staging(install_root: &Path) {
    let _ = fs_err::remove_dir_all(staging_root(install_root));
}

pub(super) fn is_pack_done(install_root: &Path, pack_id: &str, stamp: &str) -> bool {
    fs_err::read_to_string(done_marker(install_root, pack_id)).is_ok_and(|s| s == stamp)
}

pub(super) fn mark_pack_done(install_root: &Path, pack_id: &str, stamp: &str) -> Result<()> {
    Ok(fs_util::write_atomic(
        &done_marker(install_root, pack_id),
        stamp,
    )?)
}

#[derive(Debug)]
struct Interrupted;

impl std::fmt::Display for Interrupted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("patching interrupted")
    }
}

impl std::error::Error for Interrupted {}

enum GroupOutcome {
    Applied(Result<()>),
    NoDiff,
    Skipped,
}

fn patch_workers() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get().min(4))
}

async fn patch_into_staging(
    entry: &DownloadEntry,
    staging: &Path,
    pack: &Pack,
    patch: &PatchConfig,
    pidx: &PatchIndexFile,
) -> Result<()> {
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
    let dl_root = staging.join("dl");
    let out_root = staging.join("out");
    fs_err::create_dir_all(&dl_root)?;
    fs_err::create_dir_all(&out_root)?;

    let total: u64 = pidx.resource.iter().map(|r| r.size).sum();
    let progress = SyncProgress::new(total);

    let id = entry.id.clone();
    let cdn = pack.cdn_url.clone();
    let patch_base = format!("{}{}", pack.cdn_url, patch.base_url_rel);
    let resources = pidx.resource.clone();
    let dl_for_workers = dl_root.clone();
    let progress_for_workers = progress.clone();

    let stream = stream::iter(resources.into_iter().map(move |file| {
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
            download_one(
                &id,
                &sync_file(&file, &base),
                &dl_root,
                &progress,
                Skip::SameSize,
            )
            .await
        }
    }))
    .buffer_unordered(GachaLimits::load().connections);

    tokio::pin!(stream);
    while let Some(res) = stream.next().await {
        res?;
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
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
    let patch_progress = SyncProgress::new(
        pidx.group_infos
            .iter()
            .filter(|g| dl_root.join(sanitize_rel(&g.dest)).exists())
            .flat_map(|g| &g.dst_files)
            .map(|f| f.size)
            .sum(),
    );
    let mut groups = stream::iter(0..pidx.group_infos.len())
        .map(|i| {
            let group = &pidx.group_infos[i];
            let id = entry.id.clone();
            let diff_path = dl_root.join(sanitize_rel(&group.dest));
            let install_root = install_root.clone();
            let out_root = out_root.clone();
            let dst_files = group.dst_files.clone();
            let progress = patch_progress.clone();
            async move {
                if check_control(&id) != ControlSignal::None {
                    return Ok((i, GroupOutcome::Skipped));
                }
                if !diff_path.exists() {
                    return Ok((i, GroupOutcome::NoDiff));
                }
                let outcome = tokio::task::spawn_blocking(move || {
                    let on_bytes = |n| {
                        progress.advance(&id, n);
                        match check_control(&id) {
                            ControlSignal::None => Ok(()),
                            _ => Err(Interrupted.into()),
                        }
                    };
                    match apply_group(&diff_path, &install_root, &out_root, &dst_files, on_bytes) {
                        Err(e) if e.is::<Interrupted>() => GroupOutcome::Skipped,
                        applied => {
                            let _ = fs_err::remove_file(&diff_path);
                            GroupOutcome::Applied(applied)
                        }
                    }
                })
                .await?;
                Ok::<_, anyhow::Error>((i, outcome))
            }
        })
        .buffer_unordered(patch_workers());

    let mut settled: HashSet<&str> = HashSet::new();
    let mut interrupted = false;
    while let Some(res) = groups.next().await {
        let (i, outcome) = res?;
        let group = &pidx.group_infos[i];
        match outcome {
            GroupOutcome::Skipped => {
                interrupted = true;
                continue;
            }
            GroupOutcome::NoDiff => {}
            GroupOutcome::Applied(Ok(())) => {
                let dests = group.dst_files.iter().map(|f| f.dest.as_str());
                staged.extend(dests.clone());
                settled.extend(dests);
            }
            GroupOutcome::Applied(Err(e)) => {
                tracing::warn!(
                    "krpdiff group {} failed, its files fall back to full download: {}",
                    group.dest,
                    e
                );
                for f in &group.dst_files {
                    let _ = fs_err::remove_file(out_root.join(sanitize_rel(&f.dest)));
                }
            }
        }
        for s in &group.src_files {
            if let Some(n) = pending_src.get_mut(s.dest.as_str()) {
                *n = n.saturating_sub(1);
            }
        }
        flush_staged(&mut staged, &pending_src, &out_root, &install_root);
    }
    drop(groups);
    if interrupted {
        return Ok(());
    }

    let candidates: Vec<ResourceFile> = pidx
        .group_infos
        .iter()
        .flat_map(|g| g.dst_files.iter())
        .filter(|f| settled.insert(f.dest.as_str()))
        .cloned()
        .collect();

    let fallback: Vec<ResourceFile> = stream::iter(candidates)
        .map(|f| {
            let id = entry.id.clone();
            let path = install_root.join(sanitize_rel(&f.dest));
            tokio::task::spawn_blocking(move || {
                if check_control(&id) != ControlSignal::None {
                    return None;
                }
                let want = expected_md5(&f.md5);
                is_stale(&path, f.size, want.as_deref()).then_some(f)
            })
        })
        .buffer_unordered(patch_workers())
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .flatten()
        .collect();
    if check_control(&entry.id) != ControlSignal::None {
        return Ok(());
    }
    if !fallback.is_empty() {
        tracing::warn!("kuro patch: {} files need a full download", fallback.len());
        set_status(&entry.id, DownloadStatus::Downloading);
        for f in &fallback {
            if check_control(&entry.id) != ControlSignal::None {
                return Ok(());
            }
            download_one(
                &entry.id,
                &sync_file(f, &pack.base_url),
                &out_root,
                &progress,
                Skip::SameSize,
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
            let _ = fs_err::remove_file(&p);
        }
    }
    let _ = fs_err::remove_dir_all(staging);
    Ok(())
}

fn apply_group(
    diff: &Path,
    old_root: &Path,
    out_root: &Path,
    dst_files: &[ResourceFile],
    on_bytes: impl FnMut(u64) -> Result<()>,
) -> Result<()> {
    let kr = Krpdiff::open(diff)?;
    kr.apply(old_root, out_root, on_bytes)?;
    for f in dst_files {
        let path = out_root.join(sanitize_rel(&f.dest));
        let size = fs_err::metadata(&path)
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
        if let Some(want) = expected_md5(&f.md5)
            && file_md5(&path)? != want
        {
            bail!("patched output {} md5 mismatch", f.dest);
        }
    }
    Ok(())
}

fn move_tree(from: &Path, to: &Path) -> Result<()> {
    for entry in fs_err::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs_err::create_dir_all(&target)?;
            move_tree(&entry.path(), &target)?;
        } else {
            fs_err::rename(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn move_file(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        fs_err::create_dir_all(parent)?;
    }
    fs_err::rename(from, to)?;
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
