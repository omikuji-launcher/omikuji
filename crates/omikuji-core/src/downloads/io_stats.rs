use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{LazyLock, Mutex};

const HISTORY_LEN: usize = 60;

#[derive(Default)]
struct Inner {
    roots: HashMap<u32, u64>,
    seen: HashMap<u32, u64>,
    retired: u64,
    last_total: Option<u64>,
    history: VecDeque<(u64, u64)>,
}

static STATE: LazyLock<Mutex<Inner>> = LazyLock::new(|| Mutex::new(Inner::default()));

pub fn track_child(pid: u32) {
    if let Some(started) = super::proc_tree::start_time(pid) {
        STATE.lock().unwrap().roots.insert(pid, started);
    }
}

fn read_write_bytes(path: &str) -> Option<u64> {
    let data = fs_err::read_to_string(path).ok()?;
    data.lines()
        .find_map(|l| l.strip_prefix("write_bytes: "))
        .and_then(|v| v.trim().parse().ok())
}

use super::proc_tree::descendants;

fn disk_total(inner: &mut Inner) -> u64 {
    let mut live = HashSet::new();
    inner.roots.retain(|&root, started| {
        let alive = super::proc_tree::start_time(root) == Some(*started);
        if alive {
            descendants(root, &mut live);
        }
        alive
    });

    let Inner { seen, retired, .. } = inner;
    for &pid in &live {
        if let Some(now) = read_write_bytes(&format!("/proc/{}/io", pid)) {
            let entry = seen.entry(pid).or_insert(0);
            if now >= *entry {
                *entry = now;
            } else {
                *retired += *entry;
                *entry = now;
            }
        }
    }
    seen.retain(|pid, last| {
        if live.contains(pid) {
            true
        } else {
            *retired += *last;
            false
        }
    });

    *retired + read_write_bytes("/proc/self/io").unwrap_or(0) + seen.values().sum::<u64>()
}

pub fn tick(net_bps: u64) {
    let mut inner = STATE.lock().unwrap();
    let total = disk_total(&mut inner);
    let disk_bps = inner
        .last_total
        .map(|prev| total.saturating_sub(prev))
        .unwrap_or(0);
    inner.last_total = Some(total);
    if inner.history.len() == HISTORY_LEN {
        inner.history.pop_front();
    }
    inner.history.push_back((net_bps, disk_bps));
}

pub fn reset_history() {
    let mut inner = STATE.lock().unwrap();
    inner.history.clear();
    inner.last_total = None;
}

pub fn history_json() -> String {
    let inner = STATE.lock().unwrap();
    let net: Vec<u64> = inner.history.iter().map(|(n, _)| *n).collect();
    let disk: Vec<u64> = inner.history.iter().map(|(_, d)| *d).collect();
    serde_json::json!({ "net": net, "disk": disk }).to_string()
}
