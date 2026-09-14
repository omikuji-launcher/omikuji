use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::downloads::limits::StoreLimits;
use crate::downloads::proc_tree::shutdown;
use crate::downloads::proxy;
use crate::downloads::rate::{RateMeter, seeded_update};
use crate::downloads::session::SessionTally;
use crate::downloads::{
    ControlSignal, DownloadEntry, DownloadSource, check_control, report_progress,
};

pub struct GogdlSource;

fn gogdl_bin() -> Result<PathBuf> {
    crate::store::gog::find_gogdl().ok_or_else(|| {
        anyhow!(
            "gogdl not found — install via first-run components or place at {}",
            crate::runtime_dir().join("gogdl").display()
        )
    })
}

#[async_trait]
impl DownloadSource for GogdlSource {
    // destructive_cleanup on Install already rm -rf's install_path, so this is a no-op there
    fn cleanup_state(&self, entry: &DownloadEntry) {
        let support = crate::store::gog::gog_dir()
            .join("support")
            .join(&entry.app_id);
        if support.exists() {
            let _ = std::fs::remove_dir_all(&support);
        }
    }

    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        let gogdl = gogdl_bin()?;

        // drop stale registry entries whose files are gone, else you get a "Completed" flash over an empty dir
        let mut live_install = false;
        if let Some(info) = crate::store::gog::find_installed_info(&entry.app_id) {
            let has_marker = info.install_path.exists()
                && dir_has_info_marker(&info.install_path, &entry.app_id);
            if has_marker {
                live_install = true;
            } else {
                tracing::warn!(
                    "stale registry entry for {} (path={} marker_missing) - clearing before install",
                    entry.app_id,
                    info.install_path.display()
                );
                let _ = crate::store::gog::remove_install(&entry.app_id);
            }
        }

        // fresh install = fresh manifest, else gogdl says "Nothing to do" over an empty dir. fuck you gogdl ngl
        if !live_install {
            crate::store::gog::wipe_gogdl_manifest_for(&entry.app_id);
        }

        if let Err(e) = std::fs::create_dir_all(&entry.install_path) {
            return Err(anyhow!(
                "failed to create install dir {}: {e}",
                entry.install_path.display()
            ));
        }

        let child = spawn_download(&gogdl, entry).await?;
        run_with_progress(child, entry).await?;

        // clean gogdl exit is the install signal, same as heroic
        let final_root = resolve_install_root(&entry.install_path, &entry.app_id)
            .unwrap_or_else(|| entry.install_path.clone());
        let bytes = crate::fs_util::dir_size(&final_root);
        tracing::info!(
            "install recorded at {} ({} MB on disk)",
            final_root.display(),
            bytes / (1024 * 1024)
        );
        if !dir_has_info_marker(&final_root, &entry.app_id) {
            log_dir_listing(&entry.install_path);
        }

        let title = entry.display_name.clone();
        let exe = find_game_exe(&final_root, &entry.app_id).unwrap_or_default();
        if exe.is_empty() {
            tracing::warn!(
                "no launchable exe found for {} under {} - the play button will need a manual exe path",
                entry.app_id,
                final_root.display()
            );
        } else {
            tracing::info!("resolved exe for {}: {}", entry.app_id, exe);
        }
        if let Err(e) = crate::store::gog::record_install(&entry.app_id, &final_root, &exe, &title)
        {
            tracing::error!("failed to record install: {}", e);
        }

        Ok(())
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        let gogdl = gogdl_bin()?;
        // wipe stale manifest so gogdl sees the latest build before deciding whats to patch
        crate::store::gog::wipe_gogdl_manifest_for(&entry.app_id);
        let child = spawn_download(&gogdl, entry).await?;
        run_with_progress(child, entry).await
    }

    async fn import_existing(&self, entry: &DownloadEntry) -> Result<()> {
        let root = resolve_install_root(&entry.install_path, &entry.app_id)
            .unwrap_or_else(|| entry.install_path.clone());
        if !dir_has_info_marker(&root, &entry.app_id) {
            anyhow::bail!(
                "no GOG install for this game found at {}",
                entry.install_path.display()
            );
        }
        let exe = find_game_exe(&root, &entry.app_id).unwrap_or_default();
        if exe.is_empty() {
            tracing::warn!(
                "no launchable exe found for {} under {} - the play button will need a manual exe path",
                entry.app_id,
                root.display()
            );
        }
        crate::store::gog::record_install(&entry.app_id, &root, &exe, &entry.display_name)?;
        Ok(())
    }
}

fn dlc_args(dlcs: &[String]) -> Vec<String> {
    if dlcs.is_empty() {
        return vec!["--skip-dlcs".to_string()];
    }
    vec![
        "--with-dlcs".to_string(),
        "--dlcs".to_string(),
        dlcs.join(","),
    ]
}

async fn spawn_download(gogdl: &std::path::Path, entry: &DownloadEntry) -> Result<Child> {
    let support_dir = crate::store::gog::gog_dir()
        .join("support")
        .join(&entry.app_id);
    let _ = std::fs::create_dir_all(&support_dir);

    let auth = crate::store::gog::gog_auth_path();
    let gogdl_cfg = crate::store::gog::gogdl_config_dir();
    let _ = std::fs::create_dir_all(&gogdl_cfg);

    let mut cmd = Command::new(gogdl);
    cmd.env("GOGDL_CONFIG_PATH", &gogdl_cfg)
        .arg("--auth-config-path")
        .arg(&auth)
        .arg("download")
        .arg(&entry.app_id)
        .arg("--platform")
        .arg("windows")
        .arg("--path")
        .arg(&entry.install_path)
        .arg("--support")
        .arg(&support_dir)
        .arg("--lang")
        .arg("en-US")
        .args(dlc_args(&entry.dlcs))
        .args(StoreLimits::gog().args())
        .envs(proxy::env_vars().await)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);

    cmd.spawn()
        .map_err(|e| anyhow!("failed to spawn gogdl: {}", e))
}

async fn run_with_progress(mut child: Child, entry: &DownloadEntry) -> Result<()> {
    if let Some(pid) = child.id() {
        crate::downloads::io_stats::track_child(pid);
    }
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let mut out_lines = BufReader::new(stdout).lines();
    let mut err_lines = BufReader::new(stderr).lines();

    let mut tally = SessionTally::new(entry.bytes_total);
    let mut meter: Option<RateMeter> = None;

    let mut control_tick = tokio::time::interval(std::time::Duration::from_millis(250));
    control_tick.tick().await;

    let mut handle = |line: &str| -> bool {
        let Some((written, total)) = parse_progress_bytes(line) else {
            return false;
        };
        tally.set_session_total(total);
        tally.set_session_done(written);
        if let Some((pct, done, total)) = tally.snapshot() {
            let speed = seeded_update(&mut meter, tally.session_done());
            report_progress(&entry.id, pct, done, total, speed);
        }
        true
    };

    loop {
        tokio::select! {
            line = out_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        handle(&l);
                    }
                    Ok(None) | Err(_) => break,
                }
            }
            line = err_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        if !handle(&l) {
                            tracing::debug!("{}", l);
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
            _ = control_tick.tick() => {
                if check_control(&entry.id) != ControlSignal::None {
                    shutdown(&mut child).await;
                    return Ok(());
                }
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| anyhow!("gogdl wait failed: {}", e))?;

    if !status.success() {
        return Err(anyhow!("gogdl exited with status: {}", status));
    }

    Ok(())
}

// post-install fallback to find where gogdl actually dropped the game, logs top level + immediate subdirs
fn log_dir_listing(dir: &std::path::Path) {
    tracing::debug!(
        "listing {} (diagnostic - no info marker found):",
        dir.display()
    );
    let Ok(entries) = std::fs::read_dir(dir) else {
        tracing::debug!("  <unreadable>");
        return;
    };
    for e in entries.flatten() {
        let is_dir = e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false);
        tracing::debug!(
            "  {}{}",
            e.file_name().to_string_lossy(),
            if is_dir { "/" } else { "" }
        );
        if is_dir && let Ok(sub) = std::fs::read_dir(e.path()) {
            for se in sub.flatten().take(8) {
                tracing::debug!("    {}", se.file_name().to_string_lossy());
            }
        }
    }
}

pub fn dir_has_info_marker(dir: &std::path::Path, app_id: &str) -> bool {
    if dir.join(format!("goggame-{}.info", app_id)).exists() {
        return true;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("goggame-") && name.ends_with(".info") {
                return true;
            }
        }
    }
    false
}

// gogdl can nest a folder_name subdir inside --path (paths with ™ etc), so BFS to depth 3 for the marker
fn resolve_install_root(dir: &std::path::Path, app_id: &str) -> Option<std::path::PathBuf> {
    let mut queue: std::collections::VecDeque<(std::path::PathBuf, usize)> =
        std::collections::VecDeque::new();
    queue.push_back((dir.to_path_buf(), 0));
    while let Some((d, depth)) = queue.pop_front() {
        if dir_has_info_marker(&d, app_id) {
            return Some(d);
        }
        if depth >= 3 {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&d) {
            for e in entries.flatten() {
                if e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false) {
                    queue.push_back((e.path(), depth + 1));
                }
            }
        }
    }
    None
}

pub fn find_game_exe_pub(install_path: &std::path::Path, app_id: &str) -> Option<String> {
    find_game_exe(install_path, app_id)
}

// mirrors heroic's getExecutable: goggame-{app_id}.info -> playTasks isPrimary -> workingDir/path, then a one-level subdir scan, then any non-installer .exe
fn find_game_exe(install_path: &std::path::Path, app_id: &str) -> Option<String> {
    let preferred = install_path.join(format!("goggame-{}.info", app_id));
    if preferred.exists()
        && let Some(exe) = parse_info_for_exe(&preferred)
    {
        return Some(exe);
    }

    if let Some(exe) = scan_dir_for_info(install_path) {
        return Some(exe);
    }
    if let Ok(entries) = std::fs::read_dir(install_path) {
        for e in entries.flatten() {
            if e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false)
                && let Some(exe) = scan_dir_for_info(&e.path())
            {
                let sub = e.file_name().to_string_lossy().to_string();
                return Some(format!("{}/{}", sub, exe));
            }
        }
    }

    // last resort: first plausible .exe, skipping common installer prefixes
    scan_dir_for_exe(install_path).or_else(|| {
        std::fs::read_dir(install_path).ok().and_then(|entries| {
            for e in entries.flatten() {
                if e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false)
                    && let Some(exe) = scan_dir_for_exe(&e.path())
                {
                    let sub = e.file_name().to_string_lossy().to_string();
                    return Some(format!("{}/{}", sub, exe));
                }
            }
            None
        })
    })
}

fn scan_dir_for_info(dir: &std::path::Path) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("goggame-")
            && name.ends_with(".info")
            && let Some(exe) = parse_info_for_exe(&e.path())
        {
            return Some(exe);
        }
    }
    None
}

fn parse_info_for_exe(info_path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(info_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let tasks = v.get("playTasks").and_then(|t| t.as_array())?;

    // prefer isPrimary: true, fall back to first FileTask
    let primary = tasks
        .iter()
        .find(|t| t.get("isPrimary").and_then(|b| b.as_bool()) == Some(true))
        .or_else(|| {
            tasks
                .iter()
                .find(|t| t.get("type").and_then(|x| x.as_str()) == Some("FileTask"))
        })
        .or_else(|| tasks.first())?;

    if primary.get("type").and_then(|t| t.as_str()) == Some("URLTask") {
        return None;
    }

    let path = primary.get("path").and_then(|p| p.as_str()).unwrap_or("");
    if path.is_empty() {
        return None;
    }
    let working_dir = primary
        .get("workingDir")
        .and_then(|w| w.as_str())
        .unwrap_or("");
    if working_dir.is_empty() {
        Some(path.to_string())
    } else {
        Some(format!("{}/{}", working_dir.trim_end_matches('/'), path))
    }
}

fn scan_dir_for_exe(dir: &std::path::Path) -> Option<String> {
    let skip_prefixes = [
        "setup", "install", "unins", "redist", "dxsetup", "vcredist", "directx",
    ];
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            if !name.ends_with(".exe") {
                return None;
            }
            if skip_prefixes.iter().any(|p| name.starts_with(p)) {
                return None;
            }
            Some(e.file_name().to_string_lossy().to_string())
        })
        .next()
}

fn parse_progress_bytes(line: &str) -> Option<(u64, u64)> {
    let idx = line.find("Progress:")?;
    let (_pct, counts) = line[idx + "Progress:".len()..]
        .trim_start()
        .split_once(' ')?;
    let (written, total) = counts.split_once('/')?;
    let total = total.split(|c: char| !c.is_ascii_digit()).next()?;
    Some((written.parse().ok()?, total.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_line_gives_written_and_total() {
        let progress = "[gogdl] [PROGRESS] INFO: = Progress: 68.61 15271662900/22258050474, Running for: 00:01:57, ETA: 00:00:53";
        let downloaded = "[gogdl] [PROGRESS] INFO: = Downloaded: 512.50 MiB, Written: 480.00 MiB";
        assert_eq!(
            parse_progress_bytes(progress),
            Some((15271662900, 22258050474))
        );
        assert_eq!(parse_progress_bytes(downloaded), None);
    }
}
