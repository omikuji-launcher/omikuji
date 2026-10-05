use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{LazyLock, Mutex};

use serde::Serialize;

const MAX_BYTES_PER_GAME: usize = 2 * 1024 * 1024;

#[derive(Default)]
struct Buffer {
    lines: VecDeque<String>,
    bytes: usize,
    first_seq: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct LogDelta {
    pub first_seq: u64,
    pub next_seq: u64,
    pub text: String,
}

impl Buffer {
    fn next_seq(&self) -> u64 {
        self.first_seq + self.lines.len() as u64
    }

    fn push(&mut self, line: String) {
        self.bytes += line.len() + 1;
        self.lines.push_back(line);
        while self.bytes > MAX_BYTES_PER_GAME {
            match self.lines.pop_front() {
                Some(dropped) => {
                    self.bytes = self.bytes.saturating_sub(dropped.len() + 1);
                    self.first_seq += 1;
                }
                None => break,
            }
        }
    }

    fn clear(&mut self) {
        self.first_seq = self.next_seq();
        self.lines.clear();
        self.bytes = 0;
    }

    fn since(&self, seq: u64) -> LogDelta {
        let skip = usize::try_from(seq.saturating_sub(self.first_seq)).unwrap_or(usize::MAX);
        let mut text = String::new();
        for line in self.lines.iter().skip(skip) {
            text.push_str(line);
            text.push('\n');
        }
        LogDelta {
            first_seq: self.first_seq,
            next_seq: self.next_seq(),
            text,
        }
    }
}

static BUFFERS: LazyLock<Mutex<HashMap<String, Buffer>>> = LazyLock::new(Default::default);
static DIRTY: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);

fn mark_dirty(game_id: &str) {
    DIRTY.lock().unwrap().insert(game_id.to_string());
}

pub fn append_lines(game_id: &str, lines: impl IntoIterator<Item = String>) {
    {
        let mut buffers = BUFFERS.lock().unwrap();
        let buf = buffers.entry(game_id.to_string()).or_default();
        for line in lines {
            buf.push(line);
        }
    }
    mark_dirty(game_id);
}

pub fn since(game_id: &str, seq: u64) -> LogDelta {
    BUFFERS
        .lock()
        .unwrap()
        .get(game_id)
        .map(|b| b.since(seq))
        .unwrap_or_default()
}

pub fn get_log(game_id: &str) -> String {
    since(game_id, 0).text
}

pub fn clear_log(game_id: &str) {
    if let Some(buf) = BUFFERS.lock().unwrap().get_mut(game_id) {
        buf.clear();
    }
    mark_dirty(game_id);
}

pub fn drain_dirty() -> Vec<String> {
    std::mem::take(&mut *DIRTY.lock().unwrap())
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_and_clear_keep_seq_monotonic() {
        let mut buf = Buffer::default();
        let line = "x".repeat(1024 * 1024);
        buf.push(line.clone());
        buf.push(line.clone());
        buf.push(line);
        assert_eq!(buf.first_seq, 2);
        assert_eq!(buf.since(1).text.len(), 1024 * 1024 + 1);
        assert!(buf.since(3).text.is_empty());

        buf.clear();
        assert_eq!((buf.first_seq, buf.next_seq()), (3, 3));
        buf.push("a".into());
        let delta = buf.since(3);
        assert_eq!(
            (delta.first_seq, delta.next_seq, delta.text.as_str()),
            (3, 4, "a\n")
        );
    }
}
