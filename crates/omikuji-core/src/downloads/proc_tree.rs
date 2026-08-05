use std::collections::HashSet;
use std::path::PathBuf;
use tokio::process::Child;

pub(super) fn start_time(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    stat.rsplit(')')
        .next()?
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}

pub(super) fn descendants(root: u32, out: &mut HashSet<u32>) {
    let mut queue = vec![root];
    while let Some(pid) = queue.pop() {
        if !out.insert(pid) {
            continue;
        }
        let Ok(tasks) = std::fs::read_dir(format!("/proc/{}/task", pid)) else {
            continue;
        };
        for task in tasks.flatten() {
            let Ok(kids) = std::fs::read_to_string(task.path().join("children")) else {
                continue;
            };
            queue.extend(
                kids.split_whitespace()
                    .filter_map(|s| s.parse::<u32>().ok()),
            );
        }
    }
}

fn shm_mapped_paths(pids: &HashSet<u32>) -> HashSet<PathBuf> {
    let mut out = HashSet::new();
    for pid in pids {
        let Ok(maps) = std::fs::read_to_string(format!("/proc/{}/maps", pid)) else {
            continue;
        };
        for line in maps.lines() {
            let Some(idx) = line.find("/dev/shm/") else {
                continue;
            };
            let path = &line[idx..];
            if path.ends_with(" (deleted)") {
                continue;
            }
            if PathBuf::from(path)
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("psm_"))
            {
                out.insert(PathBuf::from(path));
            }
        }
    }
    out
}

// two-phase kill: SIGTERM lets legendary flush its .resume, then killpg the group since a surviving worker keeps the flock'd install lock and bricks the next install.
// killpg also takes out python's resource_tracker, so the 2GiB shm segment would leak on every cancel; snapshot the mappings while /proc is alive, unlink leftovers after.
// ALso i got this because after testing the new download page look i cancelled so many downloads my whole session crashed, and everytime i tried to come back in it'd crash again until reboot. You gotta love some stuff man.
pub(crate) async fn shutdown(child: &mut Child) {
    use nix::sys::signal::{Signal, kill, killpg};
    use nix::unistd::Pid;

    let pid = child.id();

    let shm = pid
        .map(|p| {
            let mut pids = HashSet::new();
            descendants(p, &mut pids);
            shm_mapped_paths(&pids)
        })
        .unwrap_or_default();

    if let Some(pid) = pid {
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
    }
    let _ = tokio::time::timeout(std::time::Duration::from_secs(8), child.wait()).await;

    if let Some(pid) = pid {
        let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
    }

    if matches!(child.try_wait(), Ok(None)) {
        let _ = child.wait().await;
    }

    for p in shm {
        if !p.exists() {
            continue;
        }
        match std::fs::remove_file(&p) {
            Ok(()) => tracing::info!("removed orphaned shm segment {}", p.display()),
            Err(e) => tracing::warn!("failed to remove shm segment {}: {}", p.display(), e),
        }
    }
}
