use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use super::{check_control, report_progress, ControlSignal, DownloadEntry, DownloadSource};

async fn shutdown(child: &mut Child) {
    use nix::sys::signal::{kill, killpg, Signal};
    use nix::unistd::Pid;

    let pid = child.id();
    if let Some(pid) = pid {
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }
    let _ = tokio::time::timeout(std::time::Duration::from_secs(8), child.wait()).await;
    if let Some(pid) = pid {
        let pgid = Pid::from_raw(pid as i32);
        let _ = killpg(pgid, Signal::SIGKILL);
    }
    if matches!(child.try_wait(), Ok(None)) {
        let _ = child.wait().await;
    }
}

pub struct GogdlSource;

fn gogdl_bin() -> Result<PathBuf> {
    crate::gog::find_gogdl().ok_or_else(|| {
        anyhow!(
            "gogdl not found — install via first-run components or place at {}",
            crate::runtime_dir().join("gogdl").display()
        )
    })
}

#[async_trait]
impl DownloadSource for GogdlSource {
    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        let gogdl = gogdl_bin()?;

        // ghost-state: if the registry still has this app but files are gone
        // (user wiped the dir, or a prior install didn't finish), drop the stale entry before re-running gogdl, stops a "Completed" flash over an empty dir
        if let Some(info) = crate::gog::find_installed_info(&entry.app_id) {
            let has_marker = info.install_path.exists()
                && dir_has_info_marker(&info.install_path, &entry.app_id);
            if !has_marker {
                eprintln!(
                    "[gogdl] stale registry entry for {} (path={} marker_missing) → clearing before install",
                    entry.app_id,
                    info.install_path.display()
                );
                let _ = crate::gog::remove_install(&entry.app_id);
            }
        }

        // without this, gogdl reads its cached manifest and can decide "Nothing to do"
        // against an empty install dir. fresh install = fresh manifest. fuck you gogdl ngl
        crate::gog::wipe_gogdl_manifest_for(&entry.app_id);

        if let Err(e) = std::fs::create_dir_all(&entry.install_path) {
            return Err(anyhow!(
                "failed to create install dir {}: {e}",
                entry.install_path.display()
            ));
        }

        let child = spawn_download(&gogdl, entry)?;
        run_with_progress(child, entry).await?;

        // heroic treats a clean gogdl exit as the install signal, we do too
        // resolve_install_root BFS handles games where gogdl unpacks into a folder_name subdir of --path, which happens with names containing ™.
        let final_root = resolve_install_root(&entry.install_path, &entry.app_id)
            .unwrap_or_else(|| entry.install_path.clone());
        let bytes = dir_size_bytes(&final_root);
        eprintln!(
            "[gogdl] install recorded at {} ({} MB on disk)",
            final_root.display(),
            bytes / (1024 * 1024)
        );
        if !dir_has_info_marker(&final_root, &entry.app_id) {
            log_dir_listing(&entry.install_path);
        }

        let title = entry.display_name.clone();
        let exe = find_game_exe(&final_root, &entry.app_id).unwrap_or_default();
        if exe.is_empty() {
            eprintln!(
                "[gogdl] no launchable exe found for {} under {} — the play button will need a manual exe path",
                entry.app_id,
                final_root.display()
            );
        } else {
            eprintln!("[gogdl] resolved exe for {}: {}", entry.app_id, exe);
        }
        if let Err(e) = crate::gog::record_install(
            &entry.app_id,
            &final_root,
            &exe,
            &title,
        ) {
            eprintln!("[gogdl] failed to record install: {}", e);
        }

        Ok(())
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        let gogdl = gogdl_bin()?;
        // wipe stale manifest so gogdl sees the latest build before deciding whats to patch
        crate::gog::wipe_gogdl_manifest_for(&entry.app_id);
        let child = spawn_download(&gogdl, entry)?;
        run_with_progress(child, entry).await
    }
}

fn spawn_download(gogdl: &std::path::Path, entry: &DownloadEntry) -> Result<Child> {
    let support_dir = crate::data_dir()
        .join("gog")
        .join("support")
        .join(&entry.app_id);
    let _ = std::fs::create_dir_all(&support_dir);

    let auth = crate::gog::gog_auth_path();
    let gogdl_cfg = crate::gog::gogdl_config_dir();
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
        .arg("--skip-dlcs")
        .arg("--lang")
        .arg("en-US")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);

    cmd.spawn()
        .map_err(|e| anyhow!("failed to spawn gogdl: {}", e))
}

async fn run_with_progress(mut child: Child, entry: &DownloadEntry) -> Result<()> {
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let mut out_lines = BufReader::new(stdout).lines();
    let mut err_lines = BufReader::new(stderr).lines();

    let mut pct: f64 = 0.0;
    let mut speed_bps: u64 = 0;
    let mut dl_bytes: u64 = 0;
    let mut total_bytes: u64 = entry.bytes_total;

    let mut control_tick = tokio::time::interval(std::time::Duration::from_millis(250));
    control_tick.tick().await;

    loop {
        tokio::select! {
            line = out_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        if parse_into(&l, &mut pct, &mut speed_bps, &mut dl_bytes, &mut total_bytes) {
                            report_progress(&entry.id, pct, dl_bytes, total_bytes, speed_bps);
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
            line = err_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        if parse_into(&l, &mut pct, &mut speed_bps, &mut dl_bytes, &mut total_bytes) {
                            report_progress(&entry.id, pct, dl_bytes, total_bytes, speed_bps);
                        } else {
                            eprintln!("[gogdl] {}", l);
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

// called post-install when we couldnt find a goggame-*.info marker, to see where gogdl actually dropped teh game. only logs top leevel + immediate subdirs.
fn log_dir_listing(dir: &std::path::Path) {
    eprintln!("[gogdl] listing {} (diagnostic — no info marker found):", dir.display());
    let Ok(entries) = std::fs::read_dir(dir) else {
        eprintln!("[gogdl]   <unreadable>");
        return;
    };
    for e in entries.flatten() {
        let is_dir = e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false);
        eprintln!(
            "[gogdl]   {}{}",
            e.file_name().to_string_lossy(),
            if is_dir { "/" } else { "" }
        );
        if is_dir
            && let Ok(sub) = std::fs::read_dir(e.path()) {
                for se in sub.flatten().take(8) {
                    eprintln!("[gogdl]     {}", se.file_name().to_string_lossy());
                }
            }
    }
}

fn dir_size_bytes(dir: &std::path::Path) -> u64 {
    fn walk(dir: &std::path::Path, depth: usize) -> u64 {
        if depth == 0 {
            return 0;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return 0;
        };
        let mut total = 0u64;
        for e in entries.flatten() {
            let Ok(md) = e.metadata() else { continue };
            if md.is_dir() {
                total = total.saturating_add(walk(&e.path(), depth - 1));
            } else if md.is_file() {
                total = total.saturating_add(md.len());
            }
        }
        total
    }
    walk(dir, 8)
}

fn dir_has_info_marker(dir: &std::path::Path, app_id: &str) -> bool {
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

// gogdl sometimes creates a folder_name subdir inside --path (especially when the path has chars gogdl rewrites, like ™)
// so the marker can land a few levels deep. BFS up to depth 3, past the two-subfolder cases seen so far
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

// mirrors heroic storeManagers/gog/library.ts::getExecutable:
//   1. goggame-{app_id}.info in install root
//   2. playTasks, isPrimary=true (or first FileTask)
//   3. workingDir/path
// falls back to scanning a one-level subdir (gogdl subdir rewrite), then any non-installer .exe as a last resort.
fn find_game_exe(install_path: &std::path::Path, app_id: &str) -> Option<String> {
    let preferred = install_path.join(format!("goggame-{}.info", app_id));
    if preferred.exists()
        && let Some(exe) = parse_info_for_exe(&preferred) {
            return Some(exe);
        }

    if let Some(exe) = scan_dir_for_info(install_path) {
        return Some(exe);
    }
    if let Ok(entries) = std::fs::read_dir(install_path) {
        for e in entries.flatten() {
            if e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false)
                && let Some(exe) = scan_dir_for_info(&e.path()) {
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
                    && let Some(exe) = scan_dir_for_exe(&e.path()) {
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
        if name.starts_with("goggame-") && name.ends_with(".info")
            && let Some(exe) = parse_info_for_exe(&e.path()) {
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
        .or_else(|| tasks.iter().find(|t| t.get("type").and_then(|x| x.as_str()) == Some("FileTask")))
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

// gogdl's current progress format:
//   [gogdl] [PROGRESS] INFO: = Progress: 68.61 15271662900/22258050474,
//     Running for: 00:01:57, ETA: 00:00:53
// heroic also handles an older "Downloaded: N MiB / Download\t- N MiB" format, kept here so we dont regress when gogld updates.
fn parse_into(
    line: &str,
    pct: &mut f64,
    speed_bps: &mut u64,
    dl_bytes: &mut u64,
    total_bytes: &mut u64,
) -> bool {
    let mut changed = false;

    if let Some((p, dl, total)) = parse_progress_line(line) {
        *pct = p;
        if dl > 0 {
            *dl_bytes = dl;
        }
        if total > 0 {
            *total_bytes = total;
        }
        changed = true;
    }
    if let Some(s) = parse_speed(line) {
        *speed_bps = s;
        changed = true;
    }
    if let Some(b) = parse_downloaded(line) {
        *dl_bytes = b;
        changed = true;
    }
    if let Some(t) = parse_total(line)
        && *total_bytes == 0 {
            *total_bytes = t;
            changed = true;
        }

    changed
}

fn parse_progress_line(line: &str) -> Option<(f64, u64, u64)> {
    let idx = line.find("Progress:")?;
    let rest = &line[idx + "Progress:".len()..].trim_start();
    let first_end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    let pct: f64 = rest[..first_end].parse().ok()?;
    let after = rest[first_end..].trim_start().trim_start_matches('%').trim_start();
    if let Some(slash_idx) = after.find('/') {
        let dl_str: String = after[..slash_idx]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let total_str: String = after[slash_idx + 1..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let dl = dl_str.parse::<u64>().unwrap_or(0);
        let total = total_str.parse::<u64>().unwrap_or(0);
        return Some((pct, dl, total));
    }
    Some((pct, 0, 0))
}

fn parse_speed(line: &str) -> Option<u64> {
    let marker = "Download\t- ";
    if let Some(idx) = line.find(marker) {
        let rest = &line[idx + marker.len()..];
        let num_end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(rest.len());
        if let Ok(v) = rest[..num_end].parse::<f64>() {
            return Some((v * 1024.0 * 1024.0) as u64);
        }
    }
    for (unit, mult) in &[("MiB/s", 1024.0 * 1024.0), ("MB/s", 1_000_000.0)] {
        if let Some(idx) = line.find(unit) {
            let prefix = line[..idx].trim_end();
            let num_start = prefix
                .rfind(|c: char| !c.is_ascii_digit() && c != '.')
                .map(|i| i + 1)
                .unwrap_or(0);
            if let Ok(v) = prefix[num_start..].parse::<f64>() {
                return Some((v * mult) as u64);
            }
        }
    }
    None
}

fn parse_downloaded(line: &str) -> Option<u64> {
    let idx = line.find("Downloaded:")?;
    let rest = &line[idx + "Downloaded:".len()..].trim_start();
    let num_end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    let num: f64 = rest[..num_end].parse().ok()?;
    let after = rest[num_end..].trim_start();
    let mult = if after.starts_with("GiB") {
        1024.0_f64.powi(3)
    } else if after.starts_with("MiB") {
        1024.0_f64.powi(2)
    } else if after.starts_with("KiB") {
        1024.0
    } else {
        return None;
    };
    Some((num * mult) as u64)
}

fn parse_total(line: &str) -> Option<u64> {
    let slash_idx = line.find('/')?;
    let after = line[slash_idx + 1..].trim_start();
    let num_end = after
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(after.len());
    let num: f64 = after[..num_end].parse().ok()?;
    let unit = after[num_end..].trim_start();
    let mult = if unit.starts_with("GiB") {
        1024.0_f64.powi(3)
    } else if unit.starts_with("MiB") {
        1024.0_f64.powi(2)
    } else {
        return None;
    };
    Some((num * mult) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pct_legacy_with_percent() {
        let line = "[INFO] Progress: 42.3 % | ETA: 00:05:12";
        let (p, dl, total) = parse_progress_line(line).unwrap();
        assert!((p - 42.3).abs() < 0.01);
        assert_eq!(dl, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn parse_pct_modern_with_bytes() {
        let line = "[gogdl] [PROGRESS] INFO: = Progress: 68.61 15271662900/22258050474, Running for: 00:01:57, ETA: 00:00:53";
        let (p, dl, total) = parse_progress_line(line).unwrap();
        assert!((p - 68.61).abs() < 0.01);
        assert_eq!(dl, 15271662900);
        assert_eq!(total, 22258050474);
    }

    #[test]
    fn parse_speed_mib_tab() {
        let line = "Download\t- 23.45 MiB | Disk\t- 19.23 MiB";
        assert_eq!(parse_speed(line), Some((23.45 * 1024.0 * 1024.0) as u64));
    }

    #[test]
    fn parse_downloaded_mib() {
        let line = "Downloaded: 512.5 MiB";
        assert_eq!(parse_downloaded(line), Some((512.5 * 1024.0 * 1024.0) as u64));
    }
}
