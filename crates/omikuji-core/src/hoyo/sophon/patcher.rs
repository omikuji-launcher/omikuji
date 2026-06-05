// sophon diff-apply loop. ported from aag-core's SophonPatcher into tokio + spawn_blocking.
//
// per-file algorithm:
//   1. target already has right size+md5 => skip
//   2. patch_chunk has OriginalFileName => hpatchz path:
//      verify original md5 => copy to tmp => hpatchz(tmp, artifact) => md5 check => move into place
//   3. else copy-over path:
//      artifact bytes are the new file (possibly wrapped in HDIFF13 envelope) => md5 check => move
//
// "globalgamemanagers" is held until the very end so the intall stays atomically "old"
// until all other files are patched. a crash mid-update leaves the game in a consistent state.

use anyhow::{anyhow, Result};
use futures_util::stream::{self, StreamExt};
use md5::{Digest, Md5};
use reqwest::header::RANGE;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::api::{DownloadInfo, SophonDiff};
use super::manifest::fetch_patch_manifest;
use super::protos::{SophonPatchAssetChunk, SophonPatchAssetProperty, SophonPatchProto};
use crate::external::hpatchz;

const HOLD_LAST_FILE_SUFFIX: &str = "globalgamemanagers";
const DEFAULT_RETRIES: u8 = 4;
const PARALLEL_DOWNLOADS: usize = 4;
const PARALLEL_PATCHES: usize = 4;

#[derive(Debug, Clone, Copy)]
pub enum Stage {
    Downloading,
    Patching,
    Deleting,
}

#[derive(Debug, Clone, Copy)]
pub struct ProgressReport {
    pub stage: Stage,
    pub current: u64,
    pub total: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub bytes_session: u64,
}

pub type ProgressFn = Arc<dyn Fn(ProgressReport) + Send + Sync + 'static>;
pub type CancelFn = Arc<dyn Fn() -> bool + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct PatchOutcome {
    pub files_patched: u64,
    pub files_deleted: u64,
}

#[derive(Clone)]
struct FileTask {
    seq: usize,
    asset_name: String,
    asset_size: u64,
    asset_hash_md5: String,
    patch_name: String,
    patch_offset: u64,
    patch_length: u64,
    original_file_name: String,
    original_file_length: u64,
    original_file_md5: String,
    download_info: Arc<DownloadInfo>,
}

impl FileTask {
    fn is_patch(&self) -> bool {
        !self.original_file_name.is_empty()
    }

    fn target_path(&self, game_dir: &Path) -> PathBuf {
        game_dir.join(&self.asset_name)
    }

    fn original_path(&self, game_dir: &Path) -> Option<PathBuf> {
        if self.is_patch() {
            Some(game_dir.join(&self.original_file_name))
        } else {
            None
        }
    }

    fn artifact_filename(&self) -> String {
        if self.is_patch() {
            format!("{}-{}.hdiff", self.patch_name, self.asset_hash_md5)
        } else {
            format!("{}.bin", self.asset_hash_md5)
        }
    }

    fn tmp_src_filename(&self) -> String {
        format!("{}-{}.tmp", self.seq, self.asset_hash_md5)
    }

    fn tmp_out_filename(&self) -> String {
        format!("{}-{}.tmp.out", self.seq, self.asset_hash_md5)
    }

    fn download_url(&self) -> String {
        self.download_info.url_for(&self.patch_name)
    }

    fn range_header(&self) -> String {
        format!(
            "bytes={}-{}",
            self.patch_offset,
            self.patch_offset + self.patch_length.saturating_sub(1)
        )
    }
}

fn build_tasks(
    manifest: &SophonPatchProto,
    download_info: &DownloadInfo,
    from_version: &str,
) -> Vec<FileTask> {
    let di = Arc::new(download_info.clone());
    manifest
        .patch_assets
        .iter()
        .enumerate()
        .filter_map(|(seq, asset)| {
            let chunk = pick_chunk(asset, from_version)?;
            Some(FileTask {
                seq,
                asset_name: asset.asset_name.clone(),
                asset_size: asset.asset_size,
                asset_hash_md5: asset.asset_hash_md5.clone(),
                patch_name: chunk.patch_name.clone(),
                patch_offset: chunk.patch_offset,
                patch_length: chunk.patch_length,
                original_file_name: chunk.original_file_name.clone(),
                original_file_length: chunk.original_file_length,
                original_file_md5: chunk.original_file_md5.clone(),
                download_info: di.clone(),
            })
        })
        .collect()
}

fn pick_chunk<'a>(
    asset: &'a SophonPatchAssetProperty,
    from_version: &str,
) -> Option<&'a SophonPatchAssetChunk> {
    asset.asset_patch_chunks.get(from_version)
}

fn files_temp(temp_root: &Path, matching_field: &str) -> PathBuf {
    temp_root.join(format!("updating-{}", matching_field))
}

fn patches_temp(temp_root: &Path, matching_field: &str) -> PathBuf {
    files_temp(temp_root, matching_field).join("patches")
}

fn ensure_temp_dirs(temp_root: &Path, matching_field: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(files_temp(temp_root, matching_field))?;
    std::fs::create_dir_all(patches_temp(temp_root, matching_field))?;
    Ok(())
}

fn file_md5(path: &Path) -> std::io::Result<String> {
    let mut f = File::open(path)?;
    let mut hasher = Md5::new();
    std::io::copy(&mut f, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn check_file(path: &Path, expected_size: u64, expected_md5: &str) -> std::io::Result<bool> {
    let Ok(meta) = std::fs::metadata(path) else { return Ok(false) };
    if meta.len() != expected_size {
        return Ok(false);
    }
    Ok(file_md5(path)? == expected_md5)
}

fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    Ok(())
}

fn add_user_write_perm(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut perms = std::fs::metadata(path)?.permissions();
    if perms.readonly() {
        let mode = perms.mode() | 0o200;
        perms.set_mode(mode);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn finalize_file(tmp: &Path, target: &Path, size: u64, md5: &str) -> Result<()> {
    if !check_file(tmp, size, md5)? {
        let got = file_md5(tmp).unwrap_or_else(|_| "<unreadable>".into());
        return Err(anyhow!(
            "file hash mismatch for {}: expected {}, got {}",
            tmp.display(),
            md5,
            got
        ));
    }
    ensure_parent(target)?;
    add_user_write_perm(target)?;
    std::fs::copy(tmp, target)?;
    Ok(())
}

// some copy-over artifacts are HDIFF13-wrapped. detect via 7-byte magic,
// skip the variable-length header, extract the trailing blob (optionally zstd-compressed).
// ported verbatim from aag-core/updater.rs:1146-1190.

fn parse_hdiff13_header<R: Read + Seek>(reader: &mut R) -> std::io::Result<(bool, u64)> {
    let mut buf = [0_u8; 128];
    reader.read_exact(&mut buf)?;
    if !buf.starts_with(b"HDIFF13") {
        return Err(std::io::Error::other("not HDIFF13"));
    }
    // header ends at the first 0x00 byte
    let header_start = buf
        .iter()
        .position(|b| *b == 0)
        .ok_or_else(|| std::io::Error::other("HDIFF13: no header terminator"))? as u64;
    let mut cursor = Cursor::new(&buf[..]);
    cursor.seek(SeekFrom::Start(header_start))?;
    // skip 10 varints
    for _ in 0..10 {
        let _ = read_varint(&mut cursor);
    }
    let new_size = read_varint(&mut cursor)?;
    let compressed_size = read_varint(&mut cursor)?;
    if compressed_size == 0 {
        Ok((false, new_size))
    } else {
        Ok((true, compressed_size))
    }
}

fn read_varint<R: Read>(reader: &mut R) -> std::io::Result<u64> {
    const CONTINUE: u8 = 0b1000_0000;
    const MASK: u8 = !CONTINUE;
    let mut byte = read_u8(reader)?;
    let mut out = (byte & MASK) as u64;
    while byte & CONTINUE != 0 {
        byte = read_u8(reader)?;
        out <<= 7;
        out |= (byte & MASK) as u64;
    }
    Ok(out)
}

fn read_u8<R: Read>(reader: &mut R) -> std::io::Result<u8> {
    let mut b = [0_u8];
    reader.read_exact(&mut b)?;
    Ok(b[0])
}

fn unwrap_hdiff13(
    artifact: &Path,
    is_compressed: bool,
    inner_size: u64,
) -> std::io::Result<PathBuf> {
    let mut src = File::open(artifact)?;
    let file_size = src.metadata()?.len();
    src.seek(SeekFrom::Start(file_size - inner_size))?;

    let mut tmp_name = artifact.file_name().unwrap().to_owned();
    tmp_name.push(".unwrapped");
    let tmp_path = artifact.parent().unwrap().join(tmp_name);

    let mut out = File::create(&tmp_path)?;
    out.set_len(inner_size)?;
    if is_compressed {
        let mut dec = zstd::Decoder::new(&mut src)?;
        std::io::copy(&mut dec, &mut out)?;
    } else {
        std::io::copy(&mut src, &mut out)?;
    }
    out.flush()?;
    Ok(tmp_path)
}

async fn download_artifact(
    task: &FileTask,
    patches_dir: &Path,
    bytes_done: &AtomicU64,
    on_progress: &ProgressFn,
    bytes_total: u64,
) -> Result<()> {
    let artifact_path = patches_dir.join(task.artifact_filename());
    if let Ok(meta) = std::fs::metadata(&artifact_path) {
        if meta.len() == task.patch_length {
            bytes_done.fetch_add(task.patch_length, Ordering::SeqCst);
            on_progress(ProgressReport {
                stage: Stage::Downloading,
                current: 0,
                total: 0,
                bytes_done: bytes_done.load(Ordering::SeqCst),
                bytes_total,
                bytes_session: 0,
            });
            return Ok(());
        }
        let _ = std::fs::remove_file(&artifact_path);
    }

    let mut last_err: Option<anyhow::Error> = None;
    for _ in 0..DEFAULT_RETRIES {
        match try_download_once(task, &artifact_path).await {
            Ok(()) => {
                bytes_done.fetch_add(task.patch_length, Ordering::SeqCst);
                on_progress(ProgressReport {
                    stage: Stage::Downloading,
                    current: 0,
                    total: 0,
                    bytes_done: bytes_done.load(Ordering::SeqCst),
                    bytes_total,
                    bytes_session: 0,
                });
                return Ok(());
            }
            Err(e) => {
                let _ = std::fs::remove_file(&artifact_path);
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("download failed after retries")))
}

async fn try_download_once(task: &FileTask, out: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let url = task.download_url();
    let resp = client
        .get(&url)
        .header(RANGE, task.range_header())
        .send()
        .await
        .map_err(|e| anyhow!("GET {} failed: {}", url, e))?
        .error_for_status()
        .map_err(|e| anyhow!("GET {} http error: {}", url, e))?;

    if let Some(len) = resp.content_length()
        && len != task.patch_length {
            return Err(anyhow!(
                "content-length {} != expected {}",
                len,
                task.patch_length
            ));
        }

    let mut file = tokio::fs::File::create(out)
        .await
        .map_err(|e| anyhow!("create {}: {}", out.display(), e))?;
    let mut stream = resp.bytes_stream();
    let mut written: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| anyhow!("stream error: {}", e))?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk).await.map_err(|e| anyhow!("write: {}", e))?;
        written += chunk.len() as u64;
    }
    use tokio::io::AsyncWriteExt;
    file.flush().await.map_err(|e| anyhow!("flush: {}", e))?;

    if written != task.patch_length {
        return Err(anyhow!(
            "wrote {} bytes, expected {}",
            written,
            task.patch_length
        ));
    }
    Ok(())
}

fn apply_file_task_blocking(
    task: &FileTask,
    game_dir: &Path,
    files_dir: &Path,
    patches_dir: &Path,
    hold_for_last: bool,
) -> Result<()> {
    let target = task.target_path(game_dir);
    if check_file(&target, task.asset_size, &task.asset_hash_md5).unwrap_or(false) {
        return Ok(());
    }

    let artifact = patches_dir.join(task.artifact_filename());
    let effective_target = if hold_for_last {
        files_dir.join("last_file.tmp")
    } else {
        target.clone()
    };

    if let Some(orig) = task.original_path(game_dir) {
        if !check_file(&orig, task.original_file_length, &task.original_file_md5)? {
            let got = file_md5(&orig).unwrap_or_else(|_| "<unreadable>".into());
            return Err(anyhow!(
                "original file {} md5 mismatch (expected {}, got {})",
                orig.display(),
                task.original_file_md5,
                got
            ));
        }

        let tmp_src = files_dir.join(task.tmp_src_filename());
        let tmp_out = files_dir.join(task.tmp_out_filename());

        let _ = std::fs::copy(&orig, &tmp_src).map_err(|e| anyhow!("copy orig to tmp: {}", e))?;
        hpatchz::patch(&tmp_src, &artifact, &tmp_out).map_err(|e| anyhow!("hpatchz: {}", e))?;

        finalize_file(&tmp_out, &effective_target, task.asset_size, &task.asset_hash_md5)?;

        let _ = std::fs::remove_file(&tmp_src);
        let _ = std::fs::remove_file(&tmp_out);
    } else {
        let mut f = File::open(&artifact).map_err(|e| anyhow!("open artifact: {}", e))?;
        let blob_path = match parse_hdiff13_header(&mut f) {
            Ok((is_compressed, inner_size)) => {
                unwrap_hdiff13(&artifact, is_compressed, inner_size)
                    .map_err(|e| anyhow!("unwrap HDIFF13: {}", e))?
            }
            Err(_) => artifact.clone(),
        };
        finalize_file(&blob_path, &effective_target, task.asset_size, &task.asset_hash_md5)?;
        if blob_path != artifact {
            let _ = std::fs::remove_file(&blob_path);
        }
    }
    Ok(())
}

pub async fn apply_update(
    diff: &SophonDiff,
    game_dir: PathBuf,
    temp_root: PathBuf,
    from_version: String,
    on_progress: ProgressFn,
    is_cancelled: CancelFn,
) -> Result<PatchOutcome> {
    let manifest = fetch_patch_manifest(diff).await?;
    let matching_field = diff.matching_field.clone();
    ensure_temp_dirs(&temp_root, &matching_field)?;

    let files_dir = files_temp(&temp_root, &matching_field);
    let patches_dir = patches_temp(&temp_root, &matching_field);

    let all_tasks = build_tasks(&manifest, &diff.diff_download, &from_version);
    let total_before = all_tasks.len();
    // oh wow i just spent 1 hour trying to figure out why it'd fail the update. Oh wow, i just realized it was resources management clean-up in game. I'm so very happy right now. Im genuinely blistering happiness from all my pores.
    let tasks: Vec<FileTask> = all_tasks
        .into_iter()
        .filter(|t| t.original_path(&game_dir).map_or(true, |p| p.exists()))
        .collect();
    if total_before > tasks.len() {
        tracing::info!(
            "skipped {} patch(es) with absent originals (in-game resource cleanup)",
            total_before - tasks.len()
        );
    }
    if tasks.is_empty() {
        return Ok(PatchOutcome { files_patched: 0, files_deleted: 0 });
    }

    let bytes_total: u64 = tasks.iter().map(|t| t.patch_length).sum();

    let bytes_done = Arc::new(AtomicU64::new(0));
    on_progress(ProgressReport {
        stage: Stage::Downloading,
        current: 0,
        total: 0,
        bytes_done: 0,
        bytes_total,
        bytes_session: 0,
    });

    {
        let patches_dir = patches_dir.clone();
        let bytes_done = bytes_done.clone();
        let on_progress = on_progress.clone();
        let is_cancelled = is_cancelled.clone();
        let dl_results: Vec<Result<()>> = stream::iter(tasks.clone())
            .map(|task| {
                let patches_dir = patches_dir.clone();
                let bytes_done = bytes_done.clone();
                let on_progress = on_progress.clone();
                let is_cancelled = is_cancelled.clone();
                async move {
                    if is_cancelled() {
                        return Err(anyhow!("cancelled"));
                    }
                    download_artifact(&task, &patches_dir, &bytes_done, &on_progress, bytes_total).await
                }
            })
            .buffer_unordered(PARALLEL_DOWNLOADS)
            .collect()
            .await;

        for r in dl_results {
            r?;
        }
    }

    if is_cancelled() {
        return Err(anyhow!("cancelled"));
    }

    let total_files = tasks.len() as u64;
    let patched = Arc::new(AtomicU64::new(0));
    let last_task = tasks
        .iter()
        .find(|t| t.asset_name.ends_with(HOLD_LAST_FILE_SUFFIX))
        .cloned();

    let patch_results: Vec<Result<()>> = stream::iter(tasks.clone())
        .map(|task| {
            let game_dir = game_dir.clone();
            let files_dir = files_dir.clone();
            let patches_dir = patches_dir.clone();
            let on_progress = on_progress.clone();
            let is_cancelled = is_cancelled.clone();
            let bytes_done = bytes_done.clone();
            let patched = patched.clone();
            async move {
                if is_cancelled() {
                    return Err(anyhow!("cancelled"));
                }
                let hold = task.asset_name.ends_with(HOLD_LAST_FILE_SUFFIX);
                tokio::task::spawn_blocking(move || {
                    apply_file_task_blocking(&task, &game_dir, &files_dir, &patches_dir, hold)
                })
                .await
                .map_err(|e| anyhow!("patch task panicked: {}", e))??;

                let done = patched.fetch_add(1, Ordering::SeqCst) + 1;
                on_progress(ProgressReport {
                    stage: Stage::Patching,
                    current: done,
                    total: total_files,
                    bytes_done: bytes_done.load(Ordering::SeqCst),
                    bytes_total,
                    bytes_session: 0,
                });
                Ok(())
            }
        })
        .buffer_unordered(PARALLEL_PATCHES)
        .collect()
        .await;

    for r in patch_results {
        r?;
    }

    if is_cancelled() {
        return Err(anyhow!("cancelled"));
    }

    if let Some(last) = last_task {
        let tmp = files_dir.join("last_file.tmp");
        if tmp.exists() {
            let target = last.target_path(&game_dir);
            let last_c = last.clone();
            tokio::task::spawn_blocking(move || -> Result<()> {
                finalize_file(&tmp, &target, last_c.asset_size, &last_c.asset_hash_md5)
            })
            .await
            .map_err(|e| anyhow!("last-file finalize panicked: {}", e))??;
        }
    }

    let mut deleted: u64 = 0;
    if let Some(unused) = manifest.unused_assets.get(&from_version) {
        let total_del = unused.assets.len() as u64;
        for asset in &unused.assets {
            let _ = std::fs::remove_file(game_dir.join(&asset.file_name));
            deleted += 1;
            on_progress(ProgressReport {
                stage: Stage::Deleting,
                current: deleted,
                total: total_del,
                bytes_done: bytes_done.load(Ordering::SeqCst),
                bytes_total,
                bytes_session: 0,
            });
        }
    }

    let _ = std::fs::remove_dir_all(&files_dir);

    Ok(PatchOutcome { files_patched: patched.load(Ordering::SeqCst), files_deleted: deleted })
}
