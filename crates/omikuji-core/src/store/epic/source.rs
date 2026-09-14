// epic installs via the legendary cli, https://github.com/derrod/legendary

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

pub fn require_legendary() -> Result<PathBuf> {
    find_legendary()
        .ok_or_else(|| anyhow!("Legendary not found - reinstall it from Settings > Components"))
}

#[async_trait]
impl DownloadSource for LegendarySource {
    fn cleanup_state(&self, entry: &DownloadEntry) {
        let Some(cfg) = dirs::config_dir() else {
            return;
        };
        let resume = cfg
            .join("legendary")
            .join("tmp")
            .join(format!("{}.resume", entry.app_id));
        if !resume.exists() {
            return;
        }
        if let Err(e) = std::fs::remove_file(&resume) {
            tracing::error!("failed to clear resume state {}: {}", resume.display(), e);
        } else {
            tracing::debug!("cleared resume state for {}", entry.app_id);
        }
    }

    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        let legendary = require_legendary()?;

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

        if let Some(info) = crate::store::epic::find_installed_info(&entry.app_id)
            && !info.install_path.exists()
        {
            tracing::warn!(
                "stale installed.json entry for {} - clearing before reinstall",
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

        run_install(
            &legendary,
            &entry.app_id,
            &base_path_str,
            &game_folder,
            entry,
        )
        .await?;

        // legendary can exit 0 without writing installed.json, and then completion imports a game that isnt installed
        if crate::store::epic::find_installed_info(&entry.app_id).is_none() {
            return Err(anyhow!(
                "legendary exited cleanly but installed.json has no record for {} \u{2014} try cancelling and starting again",
                entry.app_id
            ));
        }

        let base_label = entry.display_name.clone();
        for (i, dlc) in entry.dlcs.iter().enumerate() {
            crate::downloads::set_display_name(
                &entry.id,
                &format!("{} · DLC {}/{}", base_label, i + 1, entry.dlcs.len()),
            );
            if let Err(e) = run_install(&legendary, dlc, &base_path_str, &game_folder, entry).await
            {
                crate::downloads::set_display_name(&entry.id, &base_label);
                return Err(anyhow!("dlc {} failed to install: {}", dlc, e));
            }
        }
        crate::downloads::set_display_name(&entry.id, &base_label);

        Ok(())
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        let legendary = require_legendary()?;

        let mut cmd = Command::new(&legendary);
        cmd.arg("update")
            .arg(&entry.app_id)
            .arg("-y")
            .arg("--skip-sdl")
            .args(StoreLimits::epic().args())
            .envs(proxy::env_vars().await)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| anyhow!("failed to spawn legendary update: {}", e))?;

        run_with_progress(child, entry).await
    }

    async fn import_existing(&self, entry: &DownloadEntry) -> Result<()> {
        let legendary = require_legendary()?;

        // lets a game living somewhere else than installed.json says stilll import at the new path ig
        if crate::store::epic::find_installed_info(&entry.app_id).is_some() {
            let _ = Command::new(&legendary)
                .arg("-y")
                .arg("uninstall")
                .arg("--keep-files")
                .arg("--skip-uninstaller")
                .arg(&entry.app_id)
                .output()
                .await;
        }

        let output = Command::new(&legendary)
            .arg("-y")
            .arg("import")
            .arg("--platform")
            .arg("Windows")
            .arg(&entry.app_id)
            .arg(&entry.install_path)
            .output()
            .await
            .map_err(|e| anyhow!("failed to run legendary import: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            let line = err
                .lines()
                .rev()
                .find(|l| l.contains("ERROR"))
                .or_else(|| err.lines().rev().find(|l| !l.trim().is_empty()))
                .unwrap_or("no output");
            let msg = line.splitn(2, "ERROR: ").last().unwrap_or(line).trim();
            anyhow::bail!("import failed: {}", msg);
        }

        if crate::store::epic::find_installed_info(&entry.app_id).is_none() {
            anyhow::bail!(
                "legendary import exited cleanly but installed.json has no record for {}",
                entry.app_id
            );
        }

        Ok(())
    }
}

async fn run_install(
    legendary: &std::path::Path,
    app_name: &str,
    base_path: &str,
    game_folder: &str,
    entry: &DownloadEntry,
) -> Result<()> {
    let mut cmd = Command::new(legendary);
    cmd.arg("install")
        .arg(app_name)
        .arg("-y")
        .arg("--skip-sdl")
        .arg("--skip-dlcs")
        .arg("--platform")
        .arg("Windows")
        .arg("--base-path")
        .arg(base_path)
        .arg("--game-folder")
        .arg(game_folder)
        .args(StoreLimits::epic().args())
        .envs(proxy::env_vars().await)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);

    let child = cmd
        .spawn()
        .map_err(|e| anyhow!("failed to spawn legendary: {}", e))?;

    run_with_progress(child, entry).await
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
        let parsed = feed_line(&mut tally, line);
        if parsed && let Some((pct, done, total)) = tally.snapshot() {
            let speed = seeded_update(&mut meter, tally.session_done());
            report_progress(&entry.id, pct, done, total, speed);
        }
        parsed
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
        .map_err(|e| anyhow!("legendary wait failed: {}", e))?;

    if !status.success() {
        return Err(anyhow!("legendary exited with status: {}", status));
    }

    Ok(())
}

fn feed_line(tally: &mut SessionTally, line: &str) -> bool {
    if let Some(bytes) = parse_labeled_size(line, "Download size:") {
        tally.set_session_total(bytes);
        return true;
    }
    if let Some(bytes) = parse_labeled_size(line, "Downloaded:") {
        tally.set_session_done(bytes);
        return true;
    }
    false
}

fn parse_labeled_size(line: &str, label: &str) -> Option<u64> {
    let idx = line.find(label)?;
    parse_size(&line[idx + label.len()..])
}

fn unit_multiplier(s: &str) -> Option<f64> {
    // longest prefix first so "GiB" beats "B"
    if s.starts_with("GiB") {
        Some(1024.0 * 1024.0 * 1024.0)
    } else if s.starts_with("MiB") {
        Some(1024.0 * 1024.0)
    } else if s.starts_with("KiB") {
        Some(1024.0)
    } else if s.starts_with("GB") {
        Some(1_000_000_000.0)
    } else if s.starts_with("MB") {
        Some(1_000_000.0)
    } else if s.starts_with("KB") {
        Some(1000.0)
    } else if s.starts_with('B') {
        Some(1.0)
    } else {
        None
    }
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

    const MIB: u64 = 1024 * 1024;

    #[test]
    fn legendary_resume_lines_feed_the_tally() {
        let size = "[cli] INFO: Download size: 600.00 MiB (Compression savings: 12.0%)";
        let downloaded = "[DLManager] INFO: - Downloaded: 100.00 MiB, Written: 120.00 MiB";
        let install = "[cli] INFO: Install size: 1400.00 MiB";
        let mut tally = SessionTally::new(1000 * MIB);
        assert!(feed_line(&mut tally, size));
        assert!(feed_line(&mut tally, downloaded));
        assert!(!feed_line(&mut tally, install));
        assert_eq!(tally.snapshot(), Some((50.0, 500 * MIB, 1000 * MIB)));
    }
}
