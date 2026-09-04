use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::downloads::proc_tree::shutdown;
use crate::downloads::proxy;
use crate::downloads::rate::{RateMeter, seeded_update};
use crate::downloads::{
    ControlSignal, DownloadEntry, DownloadSource, check_control, report_progress,
};

pub struct NileSource;

#[derive(Debug, Clone, Copy)]
enum Verb {
    Install,
    Update,
    Verify,
}

impl Verb {
    fn as_arg(self) -> &'static str {
        match self {
            Verb::Install => "install",
            Verb::Update => "update",
            Verb::Verify => "verify",
        }
    }
}

#[async_trait]
impl DownloadSource for NileSource {
    fn supports_repair(&self) -> bool {
        true
    }

    fn supports_import(&self) -> bool {
        true
    }

    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        run(entry, Verb::Install).await
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        run(entry, Verb::Update).await
    }

    async fn repair(&self, entry: &DownloadEntry) -> Result<()> {
        run(entry, Verb::Verify).await
    }

    async fn import_existing(&self, entry: &DownloadEntry) -> Result<()> {
        let output = super::command()?
            .arg("import")
            .arg("--path")
            .arg(&entry.install_path)
            .arg(&entry.app_id)
            .output()
            .await?;

        if super::find_installed_info(&entry.app_id).is_none() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("nile import failed: {}", err.trim()));
        }
        Ok(())
    }
}

async fn run(entry: &DownloadEntry, verb: Verb) -> Result<()> {
    let child = super::command()?
        .arg(verb.as_arg())
        .arg("--path")
        .arg(&entry.install_path)
        .arg(&entry.app_id)
        .envs(proxy::env_vars().await)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| anyhow!("failed to spawn nile: {}", e))?;

    run_with_progress(child, entry, verb).await
}

async fn run_with_progress(mut child: Child, entry: &DownloadEntry, verb: Verb) -> Result<()> {
    if let Some(pid) = child.id() {
        crate::downloads::io_stats::track_child(pid);
    }
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let mut out_lines = BufReader::new(stdout).lines();
    let mut err_lines = BufReader::new(stderr).lines();

    let mut total_bytes: u64 = entry.bytes_total;
    let mut meter: Option<RateMeter> = None;
    let mut corrupt: Vec<String> = Vec::new();

    let mut control_tick = tokio::time::interval(std::time::Duration::from_millis(250));
    control_tick.tick().await;

    let mut handle = |line: &str, total: &mut u64, meter: &mut Option<RateMeter>| {
        if let Some(path) = parse_checksum_error(line) {
            tracing::error!("nile checksum error: {}", path);
            corrupt.push(path);
            return;
        }
        let Some((downloaded, line_total)) = parse_progress(line) else {
            tracing::debug!("{}", line);
            return;
        };
        if line_total > 0 {
            *total = line_total;
        }
        let pct = if *total > 0 {
            downloaded as f64 / *total as f64 * 100.0
        } else {
            0.0
        };
        report_progress(
            &entry.id,
            pct,
            downloaded,
            *total,
            seeded_update(meter, downloaded),
        );
    };

    loop {
        tokio::select! {
            line = out_lines.next_line() => {
                match line {
                    Ok(Some(l)) => handle(&l, &mut total_bytes, &mut meter),
                    Ok(None) | Err(_) => break,
                }
            }
            line = err_lines.next_line() => {
                match line {
                    Ok(Some(l)) => handle(&l, &mut total_bytes, &mut meter),
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
        .map_err(|e| anyhow!("nile wait failed: {}", e))?;

    if !status.success() {
        return Err(anyhow!("nile exited with status: {}", status));
    }

    // nile prints checksum mismatches and exits 0
    if !corrupt.is_empty() {
        return Err(anyhow!(
            "nile reported {} corrupt file(s) during {}: {}",
            corrupt.len(),
            verb.as_arg(),
            corrupt.join(", ")
        ));
    }

    if let Some(deps) = super::post_install_summary(&entry.install_path) {
        tracing::info!(
            "{} declares PostInstall deps we do not run: {}. install them via the prefix tools if the game refuses to start",
            entry.app_id,
            deps
        );
    }

    Ok(())
}

fn parse_progress(line: &str) -> Option<(u64, u64)> {
    let rest = line.split("Progress:").nth(1)?;
    let pair = rest.split_whitespace().find(|t| t.contains('/'))?;
    let (downloaded, total) = pair.trim_end_matches(',').split_once('/')?;
    Some((downloaded.parse().ok()?, total.parse().ok()?))
}

fn parse_checksum_error(line: &str) -> Option<String> {
    Some(line.split("Checksum error for ").nth(1)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_byte_pair_not_the_lagging_percent() {
        let line = "INFO [PROGRESS]:\t = Progress: 0.00 708588/89667154, Running for: 00:00:01, ETA: 00:00:00";
        assert_eq!(parse_progress(line), Some((708588, 89667154)));
    }

    #[test]
    fn ignores_lines_without_progress() {
        assert_eq!(parse_progress("INFO [CLI]:\t Found: Metal Slug 4"), None);
        assert_eq!(
            parse_progress("INFO [PROGRESS]:\t = Downloaded: 0.68 MiB"),
            None
        );
    }

    #[test]
    fn catches_checksum_errors() {
        assert_eq!(
            parse_checksum_error("Checksum error for /games/ms4/Data/x.bin"),
            Some("/games/ms4/Data/x.bin".to_string())
        );
        assert_eq!(parse_checksum_error("all good"), None);
    }
}
