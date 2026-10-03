use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use fs_err as fs;

use super::GAME_ID_VAR;
use crate::library::Library;

static MARKED_IDS: LazyLock<Mutex<(Option<Instant>, HashSet<String>)>> =
    LazyLock::new(Default::default);
const MARKED_IDS_TTL: Duration = Duration::from_millis(400);

#[cfg(target_os = "linux")]
fn proc_pids() -> impl Iterator<Item = u32> {
    let my_pid = std::process::id();
    fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
        .filter(move |pid| *pid != my_pid)
}

#[cfg(target_os = "linux")]
pub(super) fn session_pids(sid: u32) -> Vec<u32> {
    proc_pids()
        .filter(|pid| session_of(*pid) == Some(sid))
        .collect()
}

#[cfg(target_os = "linux")]
fn marked_processes() -> Vec<(u32, String)> {
    let key = format!("{GAME_ID_VAR}=");
    proc_pids()
        .filter_map(|pid| {
            let environ = fs::read(format!("/proc/{pid}/environ")).ok()?;
            let id = environ
                .split(|b| *b == 0)
                .find_map(|var| var.strip_prefix(key.as_bytes()))?;
            Some((pid, String::from_utf8_lossy(id).into_owned()))
        })
        .collect()
}

pub(super) fn marker_pids(game_id: &str) -> Vec<u32> {
    marked_processes()
        .into_iter()
        .filter(|(_, id)| id == game_id)
        .map(|(pid, _)| pid)
        .collect()
}

// i mean the launcher isnt even meant at all outside linux but well, do it now and forget it forever
#[cfg(not(target_os = "linux"))]
pub(super) fn session_pids(_sid: u32) -> Vec<u32> {
    Vec::new()
}

#[cfg(not(target_os = "linux"))]
pub(super) fn marker_pids(_game_id: &str) -> Vec<u32> {
    Vec::new()
}

#[cfg(not(target_os = "linux"))]
fn marked_processes() -> Vec<(u32, String)> {
    Vec::new()
}

// comm can contain spaces and parens so we parse from the last ')' as the reliable field delimiter
fn session_of(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rparen = stat.rfind(')')?;
    stat[rparen + 1..].split_whitespace().nth(3)?.parse().ok()
}

fn runs_exe(pid: u32, exe_name: &str) -> bool {
    let Ok(comm) = fs::read_to_string(format!("/proc/{pid}/comm")) else {
        return false;
    };
    let comm = comm.trim().to_lowercase();
    !comm.is_empty() && exe_name.to_lowercase().starts_with(&comm)
}

pub(super) fn running_marked_ids() -> HashSet<String> {
    let Ok(mut cache) = MARKED_IDS.lock() else {
        return HashSet::new();
    };
    if let Some(taken_at) = cache.0
        && taken_at.elapsed() < MARKED_IDS_TTL
    {
        return cache.1.clone();
    }
    let mut by_game: HashMap<String, Vec<u32>> = HashMap::new();
    for (pid, game_id) in marked_processes() {
        by_game.entry(game_id).or_default().push(pid);
    }

    let ids: HashSet<String> = by_game
        .into_iter()
        .filter(|(game_id, pids)| {
            Library::load_game_by_id(game_id)
                .ok()
                .flatten()
                .and_then(|g| {
                    g.metadata
                        .exe
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                })
                .is_some_and(|exe| pids.iter().any(|pid| runs_exe(*pid, &exe)))
        })
        .map(|(game_id, _)| game_id)
        .collect();
    *cache = (Some(Instant::now()), ids.clone());
    ids
}

pub(super) fn session_has_live_process(sid: u32) -> bool {
    !session_pids(sid).is_empty()
}
