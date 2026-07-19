// app_id format: "{game}:{edition}" e.g. "genshin:global", "star-rail:china"
// voice packs encoded in runner_version as comma-separated locale names:
// "en-us,ja-jp" (empty = no voice packs, just game files)

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures_util::StreamExt;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::sophon;
use super::{HoyoEdition, VoiceLocale};

struct ParsedHoyoApp {
    biz_id: String,
    game_slug: String,
    display_name: String,
    edition: HoyoEdition,
}
use crate::downloads::{
    ControlSignal, DownloadEntry, DownloadKind, DownloadSource, DownloadStatus, check_control,
    report_progress, set_status,
};

pub struct HoyoSource;

#[async_trait]
impl DownloadSource for HoyoSource {
    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        let from_version = match &entry.kind {
            DownloadKind::Update { from_version } => from_version.clone(),
            _ => return Err(anyhow!("update() called on a non-update entry")),
        };

        let parsed = parse_app_id(&entry.app_id)?;
        let app_parts: Vec<&str> = entry.app_id.splitn(3, ':').collect();
        let voice_str = app_parts.get(2).unwrap_or(&"");
        let voice_locales = parse_voice_locales(voice_str);

        let safe_id = entry.app_id.replace(':', "-");
        let temp_root = entry
            .install_path
            .parent()
            .unwrap_or(&entry.install_path)
            .join(format!(".omikuji-update-{}", safe_id));
        let _ = std::fs::create_dir_all(&temp_root);

        let branches = sophon::api::fetch_game_branches(parsed.edition).await?;
        let branch = branches
            .find_for(&parsed.biz_id)
            .ok_or_else(|| anyhow!("game branch not found in api response"))?;
        let main = branch
            .main
            .as_ref()
            .ok_or_else(|| anyhow!("no main package info for {}", parsed.display_name))?;
        let target_version = main.tag.clone();

        let target = crate::gachas::strategies::normalize_version(&from_version);
        let matched_tag = main
            .diff_tags
            .iter()
            .find(|t| crate::gachas::strategies::normalize_version(t) == target)
            .cloned();
        let Some(diff_key) = matched_tag else {
            tracing::warn!(
                "no diff path from {} to {} for {}, falling back to full reinstall",
                from_version,
                target_version,
                parsed.display_name
            );
            return self.install(entry).await;
        };

        let diffs = sophon::api::fetch_patch_build(parsed.edition, main).await?;

        let id = entry.id.clone();
        let total_bytes_arc = Arc::new(AtomicU64::new(0));
        let total_bytes_arc_cb = total_bytes_arc.clone();
        let last_stage = Arc::new(std::sync::Mutex::new(None::<sophon::patcher::Stage>));
        let last_stage_cb = last_stage.clone();
        let id_cb = id.clone();

        let on_progress: sophon::patcher::ProgressFn = Arc::new(move |rep| {
            use sophon::patcher::Stage;
            let mut last = last_stage_cb.lock().unwrap();
            let transitioned = !matches!((&*last, &rep.stage), (Some(s), s2) if std::mem::discriminant(s) == std::mem::discriminant(s2));
            *last = Some(rep.stage);
            drop(last);

            if transitioned {
                match rep.stage {
                    Stage::Downloading => set_status(&id_cb, DownloadStatus::Downloading),
                    Stage::Patching => set_status(&id_cb, DownloadStatus::Patching),
                    Stage::Deleting => set_status(&id_cb, DownloadStatus::Patching),
                }
            }

            // prefer byte progress when we have it; fall back to file counter
            let (done, total) = if rep.bytes_total > 0 {
                (rep.bytes_done, rep.bytes_total)
            } else {
                (rep.current, rep.total.max(1))
            };
            total_bytes_arc_cb.store(total, Ordering::SeqCst);
            let pct = if total > 0 {
                (done as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            report_progress(&id_cb, pct, done, total, 0);
        });

        let id_cancel = id.clone();
        let is_cancelled: sophon::patcher::CancelFn =
            Arc::new(move || !matches!(check_control(&id_cancel), ControlSignal::None));

        let game_diff = diffs
            .get_for("game")
            .ok_or_else(|| anyhow!("no 'game' diff in sophon response"))?;
        sophon::patcher::apply_update(
            game_diff,
            entry.install_path.clone(),
            temp_root.clone(),
            diff_key.clone(),
            on_progress.clone(),
            is_cancelled.clone(),
        )
        .await?;

        if check_control(&id) != ControlSignal::None {
            return Ok(());
        }

        for locale in &voice_locales {
            let field = locale.api_name();
            let Some(voice_diff) = diffs.get_for(field) else {
                continue;
            };
            if !voice_diff.stats.contains_key(&diff_key) {
                continue;
            }
            sophon::patcher::apply_update(
                voice_diff,
                entry.install_path.clone(),
                temp_root.clone(),
                diff_key.clone(),
                on_progress.clone(),
                is_cancelled.clone(),
            )
            .await?;

            if check_control(&id) != ControlSignal::None {
                return Ok(());
            }
        }

        super::set_installed_version(&parsed.game_slug, parsed.edition, &target_version);

        let _ = std::fs::remove_dir_all(&temp_root);

        Ok(())
    }

    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        let parsed = parse_app_id(&entry.app_id)?;
        let app_parts: Vec<&str> = entry.app_id.splitn(3, ':').collect();
        let voice_str = app_parts.get(2).unwrap_or(&"");
        let voice_locales = parse_voice_locales(voice_str);

        let branches = sophon::api::fetch_game_branches(parsed.edition).await?;
        let branch = branches
            .find_for(&parsed.biz_id)
            .ok_or_else(|| anyhow!("game branch not found for biz_id {}", parsed.biz_id))?;
        let main = branch
            .main
            .as_ref()
            .ok_or_else(|| anyhow!("no main package info for {}", parsed.display_name))?;
        let target_version = main.tag.clone();

        let build = sophon::api::fetch_build(parsed.edition, main).await?;

        let game_entry = build
            .get_for("game")
            .ok_or_else(|| anyhow!("no 'game' category in sophon build"))?
            .clone();
        let mut entries = vec![game_entry];
        for locale in &voice_locales {
            if let Some(audio_entry) = build.get_for(locale.api_name()) {
                entries.push(audio_entry.clone());
            }
        }

        std::fs::create_dir_all(&entry.install_path)?;

        let id = entry.id.clone();
        let total_bytes_arc = Arc::new(AtomicU64::new(0));
        let total_bytes_arc_cb = total_bytes_arc.clone();
        let id_cb = id.clone();
        let start = std::time::Instant::now();

        set_status(&entry.id, DownloadStatus::Downloading);

        let on_progress: sophon::patcher::ProgressFn = Arc::new(move |rep| {
            let (done, total) = if rep.bytes_total > 0 {
                (rep.bytes_done, rep.bytes_total)
            } else {
                (rep.current, rep.total.max(1))
            };
            total_bytes_arc_cb.store(total, Ordering::SeqCst);
            let pct = if total > 0 {
                (done as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            let elapsed = start.elapsed().as_secs_f64().max(0.001);
            let bps_basis = if rep.bytes_session > 0 {
                rep.bytes_session
            } else {
                done
            };
            let bps = (bps_basis as f64 / elapsed) as u64;
            report_progress(&id_cb, pct, done, total, bps);
        });

        let id_cancel = id.clone();
        let is_cancelled: sophon::patcher::CancelFn =
            Arc::new(move || !matches!(check_control(&id_cancel), ControlSignal::None));

        sophon::installer::apply_install(
            &entries,
            entry.install_path.clone(),
            on_progress,
            is_cancelled,
        )
        .await?;

        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }

        super::set_installed_version(&parsed.game_slug, parsed.edition, &target_version);
        let total = total_bytes_arc.load(Ordering::SeqCst);
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

const NUM_CONNECTIONS: usize = 8;
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
        NUM_CONNECTIONS,
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
        std::fs::create_dir_all(parent)?;
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
            .get(reqwest::header::CONTENT_RANGE)
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
        let _ = std::fs::remove_file(parts_path(dest));
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
        && let Ok(meta) = std::fs::metadata(dest)
        && meta.len() == file_size
    {
        tracing::debug!("already downloaded: {}", dest.display());
        return Ok(());
    }

    if completed.is_empty()
        && !parts_path(dest).exists()
        && let Ok(meta) = std::fs::metadata(dest)
        && meta.len() == file_size
    {
        tracing::debug!(
            "already downloaded (no journal, size matches): {}",
            dest.display()
        );
        return Ok(());
    }

    {
        let f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dest)?;
        f.set_len(file_size)?;
    }

    let _ = std::fs::OpenOptions::new()
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

                let file = tokio::fs::OpenOptions::new()
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
                }
                file.flush().await?;
                {
                    use std::os::unix::io::AsRawFd;
                    let fd = file.get_ref().as_raw_fd();
                    let len = (end - start + 1) as libc::off_t;
                    let _ = nix::fcntl::posix_fadvise(
                        fd,
                        start as libc::off_t,
                        len,
                        nix::fcntl::PosixFadviseAdvice::POSIX_FADV_DONTNEED,
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
        let mut last_bytes: u64 = 0;
        let mut last_time = std::time::Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            let dl = progress_downloaded.load(Ordering::Relaxed);
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_time).as_secs_f64();
            let speed = if elapsed > 0.0 {
                (dl.saturating_sub(last_bytes) as f64 / elapsed) as u64
            } else {
                0
            };
            let overall = base_offset + dl;
            let pct = (overall as f64 / total_bytes as f64) * 100.0;
            report_progress(&progress_entry_id, pct, overall, total_bytes, speed);
            last_bytes = dl;
            last_time = now;

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
        let _ = std::fs::remove_file(parts_path(dest));
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
    let existing = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
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
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(dest)
            .await?
    } else {
        tokio::fs::File::create(dest).await?
    };

    let mut file = tokio::io::BufWriter::with_capacity(512 * 1024, raw_file);
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = if resumed { existing } else { 0 };
    let mut last_report = std::time::Instant::now();
    let mut last_bytes: u64 = downloaded;

    let mut chunk_count: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| anyhow!("stream error: {e}"))?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        chunk_count += 1;

        let now = std::time::Instant::now();
        if now.duration_since(last_report).as_millis() >= 250 {
            let elapsed = now.duration_since(last_report).as_secs_f64();
            let speed = (downloaded.saturating_sub(last_bytes) as f64 / elapsed) as u64;
            let overall = base_offset + downloaded;
            let pct = (overall as f64 / total_bytes as f64) * 100.0;
            report_progress(entry_id, pct, overall, total_bytes, speed);
            last_report = now;
            last_bytes = downloaded;
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
    std::fs::create_dir_all(dest)?;

    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if ext == "zip"
        && let Ok(bin) = which::which("unzip")
    {
        let output = std::process::Command::new(&bin)
            .arg("-o")
            .arg("-q")
            .arg(archive_path)
            .arg("-d")
            .arg(dest)
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

    let mut child = std::process::Command::new(&bin)
        .arg("x")
        .arg(archive_path)
        .arg(format!("-o{}", dest.display()))
        .arg("-aoa")
        .arg("-bso0") // suppress file listing
        .arg("-bsp1") // enable progress to stdout
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("failed to run 7z: {}", e))?;
    crate::downloads::io_stats::track_child(child.id());

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
    let (manifest, edition_id, _) = crate::gachas::strategies::find_for_app_id(app_id)
        .ok_or_else(|| anyhow!("no manifest found for app_id: {}", app_id))?;

    let edition = match edition_id.as_str() {
        "global" => HoyoEdition::Global,
        "china" => HoyoEdition::China,
        other => return Err(anyhow!("unknown hoyo edition: {}", other)),
    };

    let biz_id = manifest
        .editions
        .iter()
        .find(|e| e.id == edition_id)
        .and_then(|e| e.strategy_config.get("biz_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            anyhow!(
                "no biz_id in manifest for {} edition {}",
                manifest.id,
                edition_id
            )
        })?
        .to_string();

    Ok(ParsedHoyoApp {
        biz_id,
        game_slug: manifest.game_slug.clone(),
        display_name: manifest.display_name.clone(),
        edition,
    })
}

fn parse_voice_locales(s: &str) -> Vec<VoiceLocale> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',')
        .filter_map(|name| {
            VoiceLocale::all()
                .iter()
                .find(|vl| vl.api_name() == name)
                .copied()
        })
        .collect()
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
    std::fs::read_to_string(parts_path(dest))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.trim().parse::<usize>().ok())
        .collect()
}

fn mark_part_complete(dest: &Path, idx: usize) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
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
    let Ok(entries) = std::fs::read_dir(&parent) else {
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
        let Ok(children) = std::fs::read_dir(&dir) else {
            continue;
        };
        for c in children.flatten() {
            let p = c.path();
            if p.extension().and_then(|s| s.to_str()) == Some("parts") {
                continue;
            }
            if let Ok(meta) = std::fs::metadata(&p)
                && meta.is_file()
            {
                bytes += meta.len();
                segments += 1;
            }
        }
    }
    (bytes, segments)
}

pub fn cleanup_hoyo_state(app_id: &str, install_path: &Path, temp_dir: Option<&Path>) {
    let dir = scratch_dir_for(app_id, install_path, temp_dir);
    if dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&dir) {
            tracing::warn!("failed to clean temp dir {}: {}", dir.display(), e);
        } else {
            tracing::debug!("cleaned temp dir {}", dir.display());
        }
    }

    let safe_id = app_id.replace(':', "-");
    let update_scratch = install_path
        .parent()
        .unwrap_or(install_path)
        .join(format!(".omikuji-update-{}", safe_id));
    if update_scratch.exists() {
        if let Err(e) = std::fs::remove_dir_all(&update_scratch) {
            tracing::warn!(
                "failed to clean update scratch {}: {}",
                update_scratch.display(),
                e
            );
        } else {
            tracing::debug!("cleaned update scratch {}", update_scratch.display());
        }
    }
}
