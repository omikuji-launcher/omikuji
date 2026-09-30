// voice packs ride in runner_version as a comma-separated locale list

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures_util::StreamExt;
use nix::fcntl::{PosixFadviseAdvice, posix_fadvise};
use reqwest::header::CONTENT_RANGE;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::sophon;
use super::{HoyoEdition, VoiceLocale};
use crate::downloads::io_stats::track_child;
use crate::downloads::limits::GachaLimits;
use crate::downloads::rate::RateMeter;
use crate::downloads::throttle;
use crate::downloads::{
    ControlSignal, DownloadEntry, DownloadKind, DownloadSource, DownloadStatus, check_control,
    report_progress, set_status,
};
use crate::gacha::{state, strategies};

struct ParsedHoyoApp {
    biz_id: String,
    game_slug: String,
    display_name: String,
    edition: HoyoEdition,
}

pub struct HoyoSource;

#[async_trait]
impl DownloadSource for HoyoSource {
    fn cleanup_state(&self, entry: &DownloadEntry) {
        cleanup_hoyo_state(
            &entry.app_id,
            &entry.install_path,
            entry.temp_dir.as_deref(),
        );
    }

    // segments are suspect after a failure, dont re-extract the same corrupt archive
    fn reset_for_retry(&self, entry: &DownloadEntry) {
        self.cleanup_state(entry);
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        let DownloadKind::Update { from_version } = &entry.kind else {
            return Err(anyhow!("update() called on a non-update entry"));
        };

        let job = plan_patch(entry, from_version, PackageChannel::Main).await?;
        let Some(diff_key) = &job.diff_key else {
            tracing::warn!(
                "no diff path from {} to {} for {}, falling back to full reinstall",
                from_version,
                job.package.tag,
                job.parsed.display_name
            );
            return self.install(entry).await;
        };

        run_patch(entry, &job, diff_key, PatchMode::Apply).await?;
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }

        state::write_installed_version(
            &job.parsed.game_slug,
            job.parsed.edition.id(),
            &job.package.tag,
        );
        let _ = fs_err::remove_dir_all(&job.temp_root);
        Ok(())
    }

    async fn pre_download(&self, entry: &DownloadEntry) -> Result<()> {
        let DownloadKind::PreDownload { from_version, .. } = &entry.kind else {
            return Err(anyhow!("pre_download() called on a non-pre-download entry"));
        };

        let job = plan_patch(entry, from_version, PackageChannel::PreDownload).await?;
        let Some(diff_key) = &job.diff_key else {
            return Err(anyhow!(
                "no pre-download patch from {} to {} for {}",
                from_version,
                job.package.tag,
                job.parsed.display_name
            ));
        };

        run_patch(entry, &job, diff_key, PatchMode::DownloadOnly).await?;
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }

        fs_err::write(job.temp_root.join(PREDOWNLOAD_MARKER), &job.package.tag)?;
        Ok(())
    }

    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        let parsed = parse_app_id(&entry.app_id)?;
        let main = fetch_package(&parsed, PackageChannel::Main).await?;
        let target_version = main.tag.clone();

        let build = sophon::api::fetch_build(parsed.edition, &main).await?;

        let game_entry = build
            .get_for("game")
            .ok_or_else(|| anyhow!("no 'game' category in sophon build"))?
            .clone();
        let mut entries = vec![game_entry];
        for locale in voice_locales_for(&entry.app_id) {
            if let Some(audio_entry) = build.get_for(locale.api_name()) {
                entries.push(audio_entry.clone());
            }
        }

        fs_err::create_dir_all(&entry.install_path)?;

        set_status(&entry.id, DownloadStatus::Downloading);

        let callbacks = sophon_callbacks(&entry.id);
        sophon::installer::apply_install(
            &entries,
            entry.install_path.clone(),
            callbacks.on_progress,
            callbacks.is_cancelled,
        )
        .await?;

        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }

        state::write_installed_version(&parsed.game_slug, parsed.edition.id(), &target_version);
        let total = callbacks.total_bytes.load(Ordering::SeqCst);
        report_progress(&entry.id, 100.0, total, total, 0);
        tracing::info!(
            "installed {} {} v{}",
            parsed.display_name,
            parsed.edition.display_name(),
            target_version
        );
        Ok(())
    }

    fn supports_repair(&self) -> bool {
        true
    }

    // Rinphon crate when?
    async fn repair(&self, entry: &DownloadEntry) -> Result<()> {
        self.install(entry).await
    }
}

const PIECE_SIZE: u64 = 256 * 1024 * 1024;

pub async fn download_file(
    url: &str,
    dest: &Path,
    entry_id: &str,
    base_offset: u64,
    total_bytes: u64,
) -> Result<()> {
    download_file_conn(
        url,
        dest,
        entry_id,
        base_offset,
        total_bytes,
        GachaLimits::load().connections,
    )
    .await
}

async fn download_file_conn(
    url: &str,
    dest: &Path,
    entry_id: &str,
    base_offset: u64,
    total_bytes: u64,
    max_connections: usize,
) -> Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::io::{AsyncSeekExt, AsyncWriteExt};

    if let Some(parent) = dest.parent() {
        fs_err::create_dir_all(parent)?;
    }

    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
        .build()
        .unwrap_or_default();

    if max_connections <= 1 {
        return download_file_simple(url, dest, 0, entry_id, base_offset, total_bytes, &client)
            .await;
    }

    let probe = client
        .get(url)
        .header("Range", "bytes=0-0")
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .map_err(|e| anyhow!("size probe failed: {e}"))?;

    let probed_size = if probe.status() == reqwest::StatusCode::PARTIAL_CONTENT {
        probe
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.rsplit('/').next())
            .and_then(|s| s.parse::<u64>().ok())
    } else {
        None
    };
    drop(probe);

    let file_size = match probed_size {
        Some(s) if s > 0 => s,
        _ => {
            tracing::debug!("range probe returned no size, falling back to single stream");
            return download_file_simple(url, dest, 0, entry_id, base_offset, total_bytes, &client)
                .await;
        }
    };

    if file_size < 10 * 1024 * 1024 {
        let _ = fs_err::remove_file(parts_path(dest));
        return download_file_simple(
            url,
            dest,
            file_size,
            entry_id,
            base_offset,
            total_bytes,
            &client,
        )
        .await;
    }

    let mut pieces: Vec<(u64, u64)> = Vec::new();
    let mut offset: u64 = 0;
    while offset < file_size {
        let end = (offset + PIECE_SIZE).min(file_size) - 1;
        pieces.push((offset, end));
        offset = end + 1;
    }

    let completed = read_completed_parts(dest);

    if completed.len() == pieces.len()
        && let Ok(meta) = fs_err::metadata(dest)
        && meta.len() == file_size
    {
        tracing::debug!("already downloaded: {}", dest.display());
        return Ok(());
    }

    if completed.is_empty()
        && !parts_path(dest).exists()
        && let Ok(meta) = fs_err::metadata(dest)
        && meta.len() == file_size
    {
        tracing::debug!(
            "already downloaded (no journal, size matches): {}",
            dest.display()
        );
        return Ok(());
    }

    {
        let f = fs_err::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dest)?;
        f.set_len(file_size)?;
    }

    let _ = fs_err::OpenOptions::new()
        .create(true)
        .append(true)
        .open(parts_path(dest));

    let resumed_bytes: u64 = pieces
        .iter()
        .enumerate()
        .filter(|(i, _)| completed.contains(i))
        .map(|(_, (s, e))| e - s + 1)
        .sum();

    let pieces = std::sync::Arc::new(pieces);
    let completed = std::sync::Arc::new(completed);
    let cursor = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let downloaded = std::sync::Arc::new(AtomicU64::new(resumed_bytes));
    let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let worker_count = max_connections.min(pieces.len());
    let mut tasks = Vec::new();
    for worker_id in 0..worker_count {
        let client = client.clone();
        let url = url.to_string();
        let dest = dest.to_path_buf();
        let pieces = pieces.clone();
        let completed = completed.clone();
        let cursor = cursor.clone();
        let downloaded = downloaded.clone();
        let cancelled = cancelled.clone();

        tasks.push(tokio::spawn(async move {
            loop {
                if cancelled.load(Ordering::Relaxed) {
                    return Ok::<(), anyhow::Error>(());
                }
                let idx = cursor.fetch_add(1, Ordering::Relaxed);
                if idx >= pieces.len() {
                    return Ok(());
                }
                if completed.contains(&idx) {
                    continue;
                }
                let (start, end) = pieces[idx];

                let resp = client
                    .get(&url)
                    .header("Range", format!("bytes={start}-{end}"))
                    .header("Accept-Encoding", "identity")
                    .send()
                    .await
                    .map_err(|e| anyhow!("worker {worker_id} piece {idx} request failed: {e}"))?;

                if resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                    return Err(anyhow!(
                        "worker {worker_id} piece {idx}: expected 206, got {}",
                        resp.status()
                    ));
                }

                let file = fs_err::tokio::OpenOptions::new()
                    .write(true)
                    .open(&dest)
                    .await?;
                let mut file = tokio::io::BufWriter::with_capacity(256 * 1024, file);
                file.seek(std::io::SeekFrom::Start(start)).await?;

                let mut stream = resp.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    if cancelled.load(Ordering::Relaxed) {
                        file.flush().await?;
                        return Ok(());
                    }
                    let chunk = chunk
                        .map_err(|e| anyhow!("worker {worker_id} piece {idx} stream error: {e}"))?;
                    file.write_all(&chunk).await?;
                    downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                    throttle::global().take(chunk.len()).await;
                }
                file.flush().await?;
                {
                    use std::os::unix::io::AsRawFd;
                    let fd = file.get_ref().as_raw_fd();
                    let len = (end - start + 1) as libc::off_t;
                    let _ = posix_fadvise(
                        fd,
                        start as libc::off_t,
                        len,
                        PosixFadviseAdvice::POSIX_FADV_DONTNEED,
                    );
                }
                if let Err(e) = mark_part_complete(&dest, idx) {
                    tracing::warn!(
                        "failed to update parts journal for {}: {}",
                        dest.display(),
                        e
                    );
                }
            }
        }));
    }

    let progress_entry_id = entry_id.to_string();
    let progress_downloaded = downloaded.clone();
    let progress_cancelled = cancelled.clone();

    let reporter = tokio::spawn(async move {
        let mut meter = RateMeter::new(0);
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            let dl = progress_downloaded.load(Ordering::Relaxed);
            let speed = meter.update(dl);
            let overall = base_offset + dl;
            let pct = (overall as f64 / total_bytes as f64) * 100.0;
            report_progress(&progress_entry_id, pct, overall, total_bytes, speed);

            if check_control(&progress_entry_id) != ControlSignal::None {
                progress_cancelled.store(true, Ordering::Relaxed);
                return;
            }
            if dl >= file_size {
                return;
            }
        }
    });

    let mut errors = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => errors.push(e),
            Err(e) => errors.push(anyhow!("task panicked: {e}")),
        }
    }

    reporter.abort();
    let _ = reporter.await;

    if !errors.is_empty() {
        return Err(anyhow!("download failed: {}", errors[0]));
    }

    if !cancelled.load(Ordering::Relaxed) {
        let _ = fs_err::remove_file(parts_path(dest));
    }

    Ok(())
}

async fn download_file_simple(
    url: &str,
    dest: &Path,
    _expected_size: u64,
    entry_id: &str,
    base_offset: u64,
    total_bytes: u64,
    _parent_client: &reqwest::Client,
) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::builder()
        .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
        .build()
        .unwrap_or_default();

    // resume support: server responds 206 (resume) or 200 (full, ignoring Ragne)
    let existing = fs_err::metadata(dest).map(|m| m.len()).unwrap_or(0);
    let mut req = client.get(url).header("Accept-Encoding", "identity");
    if existing > 0 {
        tracing::debug!("resuming single-stream from {}", format_bytes(existing));
        req = req.header("Range", format!("bytes={}-", existing));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| anyhow!("download failed: {e}"))?;
    let status = resp.status();
    let content_len = resp.content_length();
    tracing::debug!(
        "simple download: status={}, content-length={:?}, url={}",
        status,
        content_len,
        url
    );

    if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE && existing > 0 {
        tracing::debug!("already fully downloaded: {}", dest.display());
        return Ok(());
    }

    let resumed = status == reqwest::StatusCode::PARTIAL_CONTENT;
    if !status.is_success() && !resumed {
        return Err(anyhow!("download failed: status {}", status));
    }

    let raw_file = if resumed {
        fs_err::tokio::OpenOptions::new()
            .append(true)
            .open(dest)
            .await?
    } else {
        fs_err::tokio::File::create(dest).await?
    };

    let mut file = tokio::io::BufWriter::with_capacity(512 * 1024, raw_file);
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = if resumed { existing } else { 0 };
    let mut last_report = std::time::Instant::now();
    let mut meter = RateMeter::new(downloaded);

    let mut chunk_count: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| anyhow!("stream error: {e}"))?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        chunk_count += 1;
        throttle::global().take(chunk.len()).await;

        let now = std::time::Instant::now();
        if now.duration_since(last_report).as_millis() >= 250 {
            let speed = meter.update(downloaded);
            let overall = base_offset + downloaded;
            let pct = (overall as f64 / total_bytes as f64) * 100.0;
            report_progress(entry_id, pct, overall, total_bytes, speed);
            last_report = now;
        }

        if check_control(entry_id) != ControlSignal::None {
            file.flush().await?;
            return Ok(());
        }
    }

    file.flush().await?;
    tracing::debug!(
        "simple download done: {} chunks, {} bytes written",
        chunk_count,
        downloaded
    );
    Ok(())
}

pub fn extract_archive(archive_path: &Path, dest: &Path, entry_id: Option<&str>) -> Result<()> {
    extract_archive_with_password(archive_path, dest, entry_id, None)
}

pub fn extract_archive_with_password(
    archive_path: &Path,
    dest: &Path,
    entry_id: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    fs_err::create_dir_all(dest)?;

    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext == "zip"
        && let Ok(bin) = which::which("unzip")
    {
        let mut cmd = std::process::Command::new(&bin);
        cmd.arg("-o").arg("-q");
        if let Some(pw) = password {
            cmd.arg("-P").arg(pw);
        }
        let output = cmd
            .arg(archive_path)
            .arg("-d")
            .arg(dest)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| anyhow!("failed to run unzip: {}", e))?;

        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("unzip failed, falling back to 7z: {}", stderr.trim());
    }

    let bin = which::which("7z")
        .or_else(|_| which::which("7za"))
        .map_err(|_| {
            anyhow!("7z not found — install p7zip-full (apt), 7zip (pacman), or p7zip (dnf)")
        })?;

    let mut cmd = std::process::Command::new(&bin);
    cmd.arg("x")
        .arg(archive_path)
        .arg(format!("-o{}", dest.display()))
        .arg("-aoa")
        .arg("-bso0") // suppress file listing
        .arg("-bsp1"); // enable progress to stdout
    if let Some(pw) = password {
        cmd.arg(format!("-p{}", pw));
    }
    let mut child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("failed to run 7z: {}", e))?;
    track_child(child.id());

    if let Some(stdout) = child.stdout.take() {
        use std::io::Read;
        let mut reader = std::io::BufReader::new(stdout);
        let mut buf = [0u8; 4096];
        let mut last_pct: f64 = -1.0;

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    for cap in text.split('%') {
                        let num_str = cap
                            .trim_end()
                            .chars()
                            .rev()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect::<String>()
                            .chars()
                            .rev()
                            .collect::<String>();
                        if let Ok(pct) = num_str.parse::<f64>()
                            && pct != last_pct
                            && (0.0..=100.0).contains(&pct)
                        {
                            last_pct = pct;
                            if let Some(id) = entry_id {
                                report_progress(id, pct, 0, 0, 0);
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    let status = child.wait().map_err(|e| anyhow!("7z wait failed: {}", e))?;
    if !status.success() {
        return Err(anyhow!("7z extraction failed with status {}", status));
    }

    Ok(())
}

fn parse_app_id(app_id: &str) -> Result<ParsedHoyoApp> {
    let (manifest, edition_id, _) = strategies::find_for_app_id(app_id)
        .ok_or_else(|| anyhow!("no manifest found for app_id: {}", app_id))?;

    Ok(ParsedHoyoApp {
        biz_id: super::biz_id(&manifest, &edition_id)?,
        game_slug: manifest.game_slug.clone(),
        display_name: manifest.display_name.clone(),
        edition: HoyoEdition::from_id(&edition_id)?,
    })
}

fn parse_voice_locales(s: &str) -> Vec<VoiceLocale> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',')
        .filter_map(VoiceLocale::from_api_name)
        .collect()
}

fn voice_locales_for(app_id: &str) -> Vec<VoiceLocale> {
    parse_voice_locales(app_id.splitn(3, ':').nth(2).unwrap_or(""))
}

#[derive(Clone, Copy)]
enum PackageChannel {
    Main,
    PreDownload,
}

impl PackageChannel {
    fn pick(self, branch: &sophon::api::GameBranchInfo) -> Option<&sophon::api::PackageInfo> {
        match self {
            Self::Main => branch.main.as_ref(),
            Self::PreDownload => branch.pre_download.as_ref(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::PreDownload => "pre-download",
        }
    }
}

async fn fetch_package(
    parsed: &ParsedHoyoApp,
    channel: PackageChannel,
) -> Result<sophon::api::PackageInfo> {
    let branches = sophon::api::fetch_game_branches(parsed.edition).await?;
    let branch = branches
        .find_for(&parsed.biz_id)
        .ok_or_else(|| anyhow!("game branch not found for biz_id {}", parsed.biz_id))?;
    channel.pick(branch).cloned().ok_or_else(|| {
        anyhow!(
            "no {} package info for {}",
            channel.label(),
            parsed.display_name
        )
    })
}

struct SophonCallbacks {
    on_progress: sophon::patcher::ProgressFn,
    is_cancelled: sophon::patcher::CancelFn,
    total_bytes: Arc<AtomicU64>,
}

fn sophon_callbacks(id: &str) -> SophonCallbacks {
    use sophon::patcher::Stage;

    let total_bytes = Arc::new(AtomicU64::new(0));
    let total_bytes_cb = total_bytes.clone();
    let last_stage = std::sync::Mutex::new(None::<Stage>);
    let id_cb = id.to_string();

    let on_progress: sophon::patcher::ProgressFn = Arc::new(move |rep| {
        let mut last = last_stage.lock().unwrap();
        let transitioned = !matches!((&*last, &rep.stage), (Some(s), s2) if std::mem::discriminant(s) == std::mem::discriminant(s2));
        *last = Some(rep.stage);
        drop(last);

        if transitioned {
            match rep.stage {
                Stage::Downloading => set_status(&id_cb, DownloadStatus::Downloading),
                Stage::Patching | Stage::Deleting => set_status(&id_cb, DownloadStatus::Patching),
            }
        }

        let (done, total) = if rep.bytes_total > 0 {
            (rep.bytes_done, rep.bytes_total)
        } else {
            (rep.current, rep.total.max(1))
        };
        total_bytes_cb.store(total, Ordering::SeqCst);
        let pct = if total > 0 {
            (done as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        report_progress(&id_cb, pct, done, total, throttle::global().speed_bps());
    });

    let id_cancel = id.to_string();
    let is_cancelled: sophon::patcher::CancelFn =
        Arc::new(move || !matches!(check_control(&id_cancel), ControlSignal::None));

    SophonCallbacks {
        on_progress,
        is_cancelled,
        total_bytes,
    }
}

#[derive(Clone, Copy)]
enum PatchMode {
    Apply,
    DownloadOnly,
}

struct PatchJob {
    parsed: ParsedHoyoApp,
    voice_locales: Vec<VoiceLocale>,
    temp_root: PathBuf,
    package: sophon::api::PackageInfo,
    diff_key: Option<String>,
}

async fn plan_patch(
    entry: &DownloadEntry,
    from_version: &str,
    channel: PackageChannel,
) -> Result<PatchJob> {
    let parsed = parse_app_id(&entry.app_id)?;
    let temp_root = update_scratch_dir(&entry.app_id, &entry.install_path);
    let _ = fs_err::create_dir_all(&temp_root);

    let package = fetch_package(&parsed, channel).await?;
    let target = strategies::normalize_version(from_version);
    let diff_key = package
        .diff_tags
        .iter()
        .find(|t| strategies::normalize_version(t) == target)
        .cloned();

    Ok(PatchJob {
        voice_locales: voice_locales_for(&entry.app_id),
        parsed,
        temp_root,
        package,
        diff_key,
    })
}

async fn run_patch(
    entry: &DownloadEntry,
    job: &PatchJob,
    diff_key: &str,
    mode: PatchMode,
) -> Result<()> {
    let diffs = sophon::api::fetch_patch_build(job.parsed.edition, &job.package).await?;
    let game_diff = diffs
        .get_for("game")
        .ok_or_else(|| anyhow!("no 'game' diff in sophon response"))?;
    let voice_diffs = job
        .voice_locales
        .iter()
        .filter_map(|locale| diffs.get_for(locale.api_name()))
        .filter(|diff| diff.stats.contains_key(diff_key));

    let callbacks = sophon_callbacks(&entry.id);
    for diff in std::iter::once(game_diff).chain(voice_diffs) {
        match mode {
            PatchMode::Apply => {
                sophon::patcher::apply_update(
                    diff,
                    entry.install_path.clone(),
                    job.temp_root.clone(),
                    diff_key.to_string(),
                    callbacks.on_progress.clone(),
                    callbacks.is_cancelled.clone(),
                )
                .await?;
            }
            PatchMode::DownloadOnly => {
                sophon::patcher::download_update(
                    diff,
                    &entry.install_path,
                    &job.temp_root,
                    diff_key,
                    &callbacks.on_progress,
                    &callbacks.is_cancelled,
                )
                .await?;
            }
        }
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }
    }
    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.0} KiB", bytes as f64 / 1024.0)
    }
}

fn parts_path(dest: &Path) -> std::path::PathBuf {
    let mut p = dest.as_os_str().to_os_string();
    p.push(".parts");
    std::path::PathBuf::from(p)
}

fn read_completed_parts(dest: &Path) -> std::collections::HashSet<usize> {
    fs_err::read_to_string(parts_path(dest))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.trim().parse::<usize>().ok())
        .collect()
}

fn mark_part_complete(dest: &Path, idx: usize) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = fs_err::OpenOptions::new()
        .create(true)
        .append(true)
        .open(parts_path(dest))?;
    writeln!(f, "{}", idx)?;
    f.flush()
}

fn scratch_dir_for(
    app_id: &str,
    install_path: &Path,
    temp_dir: Option<&Path>,
) -> std::path::PathBuf {
    let safe_id = app_id.replace(':', "-");
    match temp_dir {
        Some(p) => p.join(format!(".omikuji-dl-{}", safe_id)),
        None => install_path
            .parent()
            .unwrap_or(install_path)
            .join(format!(".omikuji-dl-{}", safe_id)),
    }
}

pub fn inspect_hoyo_temp(app_id: &str, install_path: &Path, temp_dir: Option<&Path>) -> (u64, u32) {
    let prefix = format!(".omikuji-dl-{}", app_id.replace(':', "-"));
    let parent = match temp_dir {
        Some(p) => p.to_path_buf(),
        None => install_path.parent().unwrap_or(install_path).to_path_buf(),
    };
    if !parent.exists() {
        return (0, 0);
    }

    let mut bytes: u64 = 0;
    let mut segments: u32 = 0;
    let Ok(entries) = fs_err::read_dir(&parent) else {
        return (0, 0);
    };
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !name.starts_with(&prefix) {
            continue;
        }
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let Ok(children) = fs_err::read_dir(&dir) else {
            continue;
        };
        for c in children.flatten() {
            let p = c.path();
            if p.extension().and_then(|s| s.to_str()) == Some("parts") {
                continue;
            }
            if let Ok(meta) = fs_err::metadata(&p)
                && meta.is_file()
            {
                bytes += meta.len();
                segments += 1;
            }
        }
    }
    (bytes, segments)
}

const PREDOWNLOAD_MARKER: &str = ".predownload";

fn update_scratch_dir(app_id: &str, install_path: &Path) -> PathBuf {
    install_path
        .parent()
        .unwrap_or(install_path)
        .join(format!(".omikuji-update-{}", app_id.replace(':', "-")))
}

pub fn predownloaded_version(app_id: &str, install_path: &Path) -> Option<String> {
    let marker = update_scratch_dir(app_id, install_path).join(PREDOWNLOAD_MARKER);
    fs_err::read_to_string(marker)
        .ok()
        .map(|tag| tag.trim().to_string())
}

fn remove_scratch(dir: &Path) {
    if !dir.exists() {
        return;
    }
    match fs_err::remove_dir_all(dir) {
        Ok(()) => tracing::debug!("cleaned {}", dir.display()),
        Err(e) => tracing::warn!("failed to clean {}: {}", dir.display(), e),
    }
}

pub fn cleanup_hoyo_state(app_id: &str, install_path: &Path, temp_dir: Option<&Path>) {
    remove_scratch(&scratch_dir_for(app_id, install_path, temp_dir));
    remove_scratch(&update_scratch_dir(app_id, install_path));
}
