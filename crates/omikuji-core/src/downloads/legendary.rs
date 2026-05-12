// epic games installs via the legendary cli
// https://github.com/derrod/legendary

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use super::{check_control, report_progress, ControlSignal, DownloadEntry, DownloadSource};

// two-phase shutdown for legendary downloads.
// phase 1: SIGTERM the parent only. legendary catches it, flushes the .resume
// manifest in ~/.config/legendary/tmp/, then coordinates worker shutdown (8s)
// phase 2: SIGKILL the entire process group. legendary uses python ultiprocessing and workers inherit the lock fd via fcntl.flock().
// any worker surviving phase 1 holds the lock, causing "Failed to acquire installed data lock" on the next install. killpg ensures the kernel releases it.

async fn shutdown(child: &mut Child) {
    use nix::sys::signal::{kill, killpg, Signal};
    use nix::unistd::Pid;

    // grab pid before wait() reaps the child and clears it
    let pid = child.id();

    if let Some(pid) = pid {
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }
    let _ = tokio::time::timeout(std::time::Duration::from_secs(8), child.wait()).await;

    // kill the group even if parent exited cleanly; orphaned workers may still hold the lock
    if let Some(pid) = pid {
        let pgid = Pid::from_raw(pid as i32);
        let _ = killpg(pgid, Signal::SIGKILL);
    }

    if matches!(child.try_wait(), Ok(None)) {
        let _ = child.wait().await;
    }
}

pub struct LegendarySource;

pub fn find_legendary() -> Option<PathBuf> {
    let bundled = crate::runtime_dir().join("legendary");
    if bundled.exists() {
        return Some(bundled);
    }
    if let Ok(p) = which::which("legendary") {
        return Some(p);
    }
    let candidates = [
        dirs::home_dir().map(|h| h.join(".local/bin/legendary")),
        Some(PathBuf::from("/usr/local/bin/legendary")),
        Some(PathBuf::from("/usr/bin/legendary")),
        // pipx default
        dirs::home_dir().map(|h| h.join(".local/share/pipx/venvs/legendary-gl/bin/legendary")),
    ];
    candidates.into_iter().flatten().find(|p| p.exists())
}

#[async_trait]
impl DownloadSource for LegendarySource {
    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        let legendary = find_legendary().ok_or_else(|| anyhow!(
            "legendary not found — install it with `pipx install legendary-gl` or `pip install --user legendary-gl`, then restart omikuji"
        ))?;

        let base_path = entry.install_path.parent().ok_or_else(|| {
            anyhow!(
                "install path has no parent directory: {}",
                entry.install_path.display()
            )
        })?;
        let game_folder = entry
            .install_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                anyhow!(
                    "install path has no final component: {}",
                    entry.install_path.display()
                )
            })?
            .to_string();
        let base_path_str = base_path.to_string_lossy().to_string();

        if let Some(info) = crate::epic::find_installed_info(&entry.app_id)
            && !info.install_path.exists() {
                eprintln!(
                    "[legendary] stale installed.json entry for {} → clearing before reinstall",
                    entry.app_id
                );
                let _ = Command::new(&legendary)
                    .arg("-y")
                    .arg("uninstall")
                    .arg(&entry.app_id)
                    .arg("--keep-files")
                    .output()
                    .await;
            }

        if let Err(e) = std::fs::create_dir_all(&entry.install_path) {
            return Err(anyhow!(
                "failed to create install dir {}: {e}",
                entry.install_path.display()
            ));
        }

        let mut cmd = Command::new(&legendary);
        cmd.arg("install")
            .arg(&entry.app_id)
            .arg("-y")
            .arg("--skip-sdl")
            .arg("--skip-dlcs")
            .arg("--platform")
            .arg("Windows")
            .arg("--base-path")
            .arg(&base_path_str)
            .arg("--game-folder")
            .arg(&game_folder)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| anyhow!("failed to spawn legendary: {}", e))?;

        run_with_progress(child, entry).await?;

        // legendary occasionally exits 0 without writing installed.json (stale
        // lock files, interrupted prior state). without this guard the completion handler tries to import a game that isnt really installed.
        if crate::epic::find_installed_info(&entry.app_id).is_none() {
            return Err(anyhow!(
                "legendary exited cleanly but installed.json has no record for {} \u{2014} try cancelling and starting again",
                entry.app_id
            ));
        }

        Ok(())
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        let legendary = find_legendary().ok_or_else(|| anyhow!(
            "legendary not found — install it with `pipx install legendary-gl` or `pip install --user legendary-gl`, then restart omikuji"
        ))?;

        let mut cmd = Command::new(&legendary);
        cmd.arg("update")
            .arg(&entry.app_id)
            .arg("-y")
            .arg("--skip-sdl")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| anyhow!("failed to spawn legendary update: {}", e))?;

        run_with_progress(child, entry).await
    }
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
    let mut reusable_bytes: u64 = 0;

    let mut control_tick = tokio::time::interval(std::time::Duration::from_millis(250));
    control_tick.tick().await;

    let adjusted = |pct: f64, dl: u64, total: u64, reusable: u64, speed: u64| -> (f64, u64, u64, u64) {
        if reusable > 0 && total > 0 {
            let base = reusable as f64 / total as f64;
            let adj_pct = (base + (1.0 - base) * pct / 100.0) * 100.0;
            let adj_dl = (adj_pct / 100.0 * total as f64) as u64;
            (adj_pct, adj_dl, total, speed)
        } else {
            (pct, dl, total, speed)
        }
    };

    loop {
        tokio::select! {
            line = out_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        parse_reusable(&l, &mut reusable_bytes);
                        if parse_into(&l, &mut pct, &mut speed_bps, &mut dl_bytes, &mut total_bytes) {
                            let (p, d, t, s) = adjusted(pct, dl_bytes, total_bytes, reusable_bytes, speed_bps);
                            report_progress(&entry.id, p, d, t, s);
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
            line = err_lines.next_line() => {
                match line {
                    Ok(Some(l)) => {
                        parse_reusable(&l, &mut reusable_bytes);
                        if parse_into(&l, &mut pct, &mut speed_bps, &mut dl_bytes, &mut total_bytes) {
                            let (p, d, t, s) = adjusted(pct, dl_bytes, total_bytes, reusable_bytes, speed_bps);
                            report_progress(&entry.id, p, d, t, s);
                        } else {
                            eprintln!("[legendary] {}", l);
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
        .map_err(|e| anyhow!("legendary wait failed: {}", e))?;

    if !status.success() {
        return Err(anyhow!("legendary exited with status: {}", status));
    }

    Ok(())
}

// legendary's progress output looks like:
//   [DLManager][Progress] - Downloaded: 152 MiB, Written: 128 MiB
//   [DLManager][Progress] + Download   - 23.45 MiB/s (raw) / 19.23 MiB/s (decompressed)
//   [DLManager][Progress] - Completed: 1234/5678 chunks (21.74%)
fn parse_into(
    line: &str,
    pct: &mut f64,
    speed_bps: &mut u64,
    dl_bytes: &mut u64,
    total_bytes: &mut u64,
) -> bool {
    let mut changed = false;

    if let Some(total) = parse_total_size(line)
        && total > 0 && *total_bytes == 0 {
            *total_bytes = total;
            changed = true;
        }

    if let Some(p) = parse_percent(line) {
        *pct = p;
        changed = true;
    }
    if let Some(s) = parse_speed(line) {
        *speed_bps = s;
        changed = true;
    }
    if let Some((dl, total)) = parse_downloaded(line) {
        *dl_bytes = dl;
        if total > 0 {
            *total_bytes = total;
        }
        changed = true;
    }

    changed
}

// "Reusable size: A MiB (chunks) / B MiB (unchanged / skipped)", sum both parts, they're all already-done work used as the base offset for resume
fn parse_reusable(line: &str, reusable_bytes: &mut u64) {
    let marker = "Reusable size:";
    let Some(idx) = line.find(marker) else { return };
    let rest = &line[idx + marker.len()..];
    let Some(slash) = rest.find('/') else { return };
    let chunks = parse_size(rest[..slash].trim()).unwrap_or(0);
    let skipped = parse_size(rest[slash + 1..].trim()).unwrap_or(0);
    let total = chunks + skipped;
    if total > 0 {
        *reusable_bytes = total;
    }
}

fn parse_total_size(line: &str) -> Option<u64> {
    for marker in &["Install size:", "Download size:"] {
        if let Some(idx) = line.find(marker) {
            let rest = &line[idx + marker.len()..];
            if let Some(size) = parse_size(rest) {
                return Some(size);
            }
        }
    }
    None
}

fn parse_percent(line: &str) -> Option<f64> {
    let pct_idx = line.find('%')?;
    let prefix = &line[..pct_idx];
    let num_start = prefix
        .rfind(|c: char| !c.is_ascii_digit() && c != '.')
        .map(|i| i + 1)
        .unwrap_or(0);
    prefix[num_start..].trim().parse::<f64>().ok()
}

// first match wins (raw rate)
fn parse_speed(line: &str) -> Option<u64> {
    for (unit, mult) in &[("MiB/s", 1024.0 * 1024.0), ("MB/s", 1_000_000.0), ("KiB/s", 1024.0), ("KB/s", 1000.0)] {
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

fn parse_downloaded(line: &str) -> Option<(u64, u64)> {
    let idx = line.find("Downloaded:")?;
    let rest = &line[idx + "Downloaded:".len()..];

    // X / Y form: shared unit comes after Y
    if let Some(slash_idx) = rest.find('/') {
        let total = parse_size(&rest[slash_idx + 1..])?;
        let total_part = rest[slash_idx + 1..].trim_start();
        let unit_mult = unit_multiplier_from_text(total_part)?;
        let dl_num: f64 = rest[..slash_idx].trim().parse().ok()?;
        return Some(((dl_num * unit_mult) as u64, total));
    }

    let dl = parse_size(rest)?;
    Some((dl, 0))
}

fn unit_multiplier(s: &str) -> Option<f64> {
    // longest prefix first so "GiB" beats "B"
    if s.starts_with("GiB") { Some(1024.0 * 1024.0 * 1024.0) }
    else if s.starts_with("MiB") { Some(1024.0 * 1024.0) }
    else if s.starts_with("KiB") { Some(1024.0) }
    else if s.starts_with("GB") { Some(1_000_000_000.0) }
    else if s.starts_with("MB") { Some(1_000_000.0) }
    else if s.starts_with("KB") { Some(1000.0) }
    else if s.starts_with('B') { Some(1.0) }
    else { None }
}

fn unit_multiplier_from_text(s: &str) -> Option<f64> {
    let trimmed = s.trim_start();
    let num_end = trimmed
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(trimmed.len());
    let after = trimmed[num_end..].trim_start();
    unit_multiplier(after)
}

fn parse_size(s: &str) -> Option<u64> {
    let trimmed = s.trim_start();
    let num_end = trimmed
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(trimmed.len());
    let num: f64 = trimmed[..num_end].parse().ok()?;
    let rest = trimmed[num_end..].trim_start();
    let mult = unit_multiplier(rest)?;
    Some((num * mult) as u64)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_percent_from_chunks_line() {
        let line = "[DLManager][Progress] - Completed: 1234/5678 chunks (21.74%)";
        assert_eq!(parse_percent(line), Some(21.74));
    }

    #[test]
    fn parse_speed_mib() {
        let line = "[DLManager][Progress] + Download   - 23.45 MiB/s (raw)";
        assert_eq!(parse_speed(line), Some((23.45 * 1024.0 * 1024.0) as u64));
    }

    #[test]
    fn parse_downloaded_with_total() {
        let line = "[Progress] Downloaded: 1.5 / 4.0 GiB";
        let (dl, total) = parse_downloaded(line).unwrap();
        assert_eq!(dl, (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
        assert_eq!(total, (4.0 * 1024.0 * 1024.0 * 1024.0) as u64);
    }

    #[test]
    fn parse_downloaded_no_total() {
        let line = "[DLManager][Progress] - Downloaded: 152 MiB, Written: 128 MiB";
        let (dl, total) = parse_downloaded(line).unwrap();
        assert_eq!(dl, 152 * 1024 * 1024);
        assert_eq!(total, 0);
    }

    #[test]
    fn parse_reusable_size_on_resume() {
        let line = "[cli] INFO: Reusable size: 0.00 MiB (chunks) / 3882.77 MiB (unchanged / skipped)";
        let mut reusable: u64 = 0;
        parse_reusable(line, &mut reusable);
        assert_eq!(reusable, (3882.77 * 1024.0 * 1024.0) as u64);
    }

    #[test]
    fn parse_reusable_size_both_parts() {
        let line = "[cli] INFO: Reusable size: 512.00 MiB (chunks) / 1024.00 MiB (unchanged / skipped)";
        let mut reusable: u64 = 0;
        parse_reusable(line, &mut reusable);
        assert_eq!(reusable, (512.0 + 1024.0) as u64 * 1024 * 1024);
    }

    #[test]
    fn parse_reusable_size_zero() {
        let line = "[cli] INFO: Reusable size: 0.00 MiB (chunks) / 0.00 MiB (unchanged / skipped)";
        let mut reusable: u64 = 42;
        parse_reusable(line, &mut reusable);
        // stays at previous value when both are zero
        assert_eq!(reusable, 42);
    }

    #[test]
    fn parse_install_size_at_start() {
        let line = "[Core] INFO: Install size: 5.86 GiB";
        let bytes = parse_total_size(line).unwrap();
        assert_eq!(bytes, (5.86 * 1024.0 * 1024.0 * 1024.0) as u64);
    }

    #[test]
    fn parse_download_size_form() {
        let line = "[Core] INFO: Download size: 4.32 GiB";
        let bytes = parse_total_size(line).unwrap();
        assert_eq!(bytes, (4.32 * 1024.0 * 1024.0 * 1024.0) as u64);
    }
}
