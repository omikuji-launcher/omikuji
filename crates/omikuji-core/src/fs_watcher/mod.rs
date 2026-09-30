// watch the parent dir not the file itself, editors atomic-rename on save and inode watches go stale

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const DEBOUNCE_MS: u64 = 150;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    _thread: thread::JoinHandle<()>,
}

impl FileWatcher {
    // callback runs on the bg thread, hop to the ui thread inside it if needed
    pub fn watch<F>(file_path: PathBuf, on_change: F) -> notify::Result<Self>
    where
        F: Fn() + Send + 'static,
    {
        let parent = file_path
            .parent()
            .ok_or_else(|| notify::Error::generic("watched path has no parent directory"))?
            .to_path_buf();
        let target_name = file_path
            .file_name()
            .map(|s| s.to_os_string())
            .ok_or_else(|| notify::Error::generic("watched path has no filename"))?;

        fs_err::create_dir_all(&parent).ok();

        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        watcher.watch(&parent, RecursiveMode::NonRecursive)?;

        let handle = thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(Ok(event)) => {
                        if !matches_target(&event, &target_name) || !interesting(&event.kind) {
                            continue;
                        }
                        // collapse the burst then fire once
                        let deadline = Instant::now() + Duration::from_millis(DEBOUNCE_MS);
                        loop {
                            let remaining = deadline.saturating_duration_since(Instant::now());
                            if remaining.is_zero() {
                                break;
                            }
                            match rx.recv_timeout(remaining) {
                                Ok(_) => {}
                                Err(mpsc::RecvTimeoutError::Timeout) => break,
                                Err(mpsc::RecvTimeoutError::Disconnected) => return,
                            }
                        }
                        on_change();
                    }
                    Ok(Err(_)) => {}
                    Err(_) => return,
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            _thread: handle,
        })
    }
}

fn matches_target(event: &notify::Event, target: &std::ffi::OsStr) -> bool {
    event.paths.iter().any(|p| p.file_name() == Some(target))
}

// access events would spam from reads, skip them
fn interesting(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

pub struct DirWatcher {
    _watcher: RecommendedWatcher,
    _thread: thread::JoinHandle<()>,
}

impl DirWatcher {
    pub fn watch<F, G>(dir: PathBuf, filter: F, on_change: G) -> notify::Result<Self>
    where
        F: Fn(&Path) -> bool + Send + 'static,
        G: Fn() + Send + 'static,
    {
        fs_err::create_dir_all(&dir).ok();

        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        watcher.watch(&dir, RecursiveMode::NonRecursive)?;

        let handle = thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(Ok(event)) => {
                        if !interesting(&event.kind) {
                            continue;
                        }
                        if !event.paths.iter().any(|p| filter(p)) {
                            continue;
                        }
                        let deadline = Instant::now() + Duration::from_millis(DEBOUNCE_MS);
                        loop {
                            let remaining = deadline.saturating_duration_since(Instant::now());
                            if remaining.is_zero() {
                                break;
                            }
                            match rx.recv_timeout(remaining) {
                                Ok(_) => {}
                                Err(mpsc::RecvTimeoutError::Timeout) => break,
                                Err(mpsc::RecvTimeoutError::Disconnected) => return,
                            }
                        }
                        on_change();
                    }
                    Ok(Err(_)) => {}
                    Err(_) => return,
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            _thread: handle,
        })
    }
}
