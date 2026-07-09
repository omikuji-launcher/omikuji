pub mod gogdl;
pub mod legendary;
pub mod source;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub use source::DownloadSource;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Starting,
    Downloading,
    Extracting,
    Patching,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

impl DownloadStatus {
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Queued
                | Self::Starting
                | Self::Downloading
                | Self::Extracting
                | Self::Patching
                | Self::Paused
        )
    }

    // failure detail is dropped here on purpose; callers that need it read
    // the full status variant
    pub fn short(&self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Starting => "Starting",
            Self::Downloading => "Downloading",
            Self::Extracting => "Extracting",
            Self::Patching => "Patching",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed(_) => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}

// install and update have very different plumbing (full archive vs delta patch), so we keep them distinct rather than using a flag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum DownloadKind {
    #[default]
    Install,
    Update { from_version: String },
    Repair,
}


#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub source: String,
    pub app_id: String,
    pub display_name: String,
    pub banner_url: Option<String>,
    pub install_path: PathBuf,
    pub prefix_path: Option<PathBuf>,
    pub runner_version: String,
    pub temp_dir: Option<PathBuf>,
    pub kind: DownloadKind,
    pub destructive_cleanup: bool,
    pub start_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEntry {
    pub id: String,
    pub source: String,
    pub app_id: String,
    pub display_name: String,
    pub banner_url: Option<String>,
    pub install_path: PathBuf,
    pub prefix_path: Option<PathBuf>,
    pub runner_version: String,
    #[serde(default)]
    pub temp_dir: Option<PathBuf>,
    #[serde(default)]
    pub kind: DownloadKind,
    #[serde(default = "default_destructive_cleanup")]
    pub destructive_cleanup: bool,
    pub status: DownloadStatus,
    pub progress: f64,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub speed_bps: u64,
}

fn default_destructive_cleanup() -> bool {
    true
}

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Added(String),
    StatusChanged(String, DownloadStatus),
    Progress {
        id: String,
        progress: f64,
        bytes_downloaded: u64,
        bytes_total: u64,
        speed_bps: u64,
    },
    Completed {
        id: String,
        source: String,
        app_id: String,
        display_name: String,
        install_path: PathBuf,
        prefix_path: Option<PathBuf>,
        runner_version: String,
    },
    Failed(String, String),
    Removed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSignal {
    None,
    Pause,
    Cancel,
}

struct Inner {
    entries: Vec<DownloadEntry>,
    events: VecDeque<DownloadEvent>,
    control: HashMap<String, ControlSignal>,
    sources: HashMap<String, Arc<dyn DownloadSource>>,
    worker_started: bool,
}

pub struct DownloadManager {
    inner: Mutex<Inner>,
    notify: Notify,
}

lazy_static! {
    static ref MANAGER: Arc<DownloadManager> = {
        let mut sources: HashMap<String, Arc<dyn DownloadSource>> = HashMap::new();
        sources.insert("epic".to_string(), Arc::new(legendary::LegendarySource));
        sources.insert("gog".to_string(), Arc::new(gogdl::GogdlSource));
        sources.insert("hoyo".to_string(), Arc::new(crate::hoyo::source::HoyoSource));
        sources.insert("endfield".to_string(), Arc::new(crate::endfield::source::EndfieldSource));
        sources.insert("kuro".to_string(), Arc::new(crate::kuro::source::KuroSource));

        let restored = load_queue();
        if !restored.is_empty() {
            tracing::info!("restored {} paused entries from previous session", restored.len());
        }

        Arc::new(DownloadManager {
            inner: Mutex::new(Inner {
                entries: restored,
                events: VecDeque::new(),
                control: HashMap::new(),
                sources,
                worker_started: false,
            }),
            notify: Notify::new(),
        })
    };
}

lazy_static! {
    static ref MULTI: MultiProgress = MultiProgress::new();
    static ref BARS: Mutex<HashMap<String, ProgressBar>> = Mutex::new(HashMap::new());
}

fn bar_style() -> ProgressStyle {
    ProgressStyle::with_template("\n{msg}\n  {prefix}  [{bar:30}]  {decimal_bytes}/{decimal_total_bytes}  {decimal_bytes_per_sec}\n")
        .unwrap()
        .progress_chars("▰▰▱")
}

fn bar_style_no_total() -> ProgressStyle {
    ProgressStyle::with_template("\n{msg}\n  {prefix}  [{bar:30}]  {decimal_bytes}  {decimal_bytes_per_sec}\n")
        .unwrap()
        .progress_chars("▰▰▱")
}

fn finished_style() -> ProgressStyle {
    ProgressStyle::with_template("\n{msg}\n")
        .unwrap()
}

fn bold(s: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", s)
}

fn get_or_create_bar(id: &str, display_name: &str, total: u64) -> ProgressBar {
    let mut bars = BARS.lock().unwrap();
    if let Some(bar) = bars.get(id) {
        if total > 0 && bar.length() != Some(total) {
            bar.set_length(total);
            bar.set_style(bar_style());
        }
        return bar.clone();
    }
    let bar = if total > 0 {
        MULTI.add(ProgressBar::new(total).with_style(bar_style()))
    } else {
        MULTI.add(ProgressBar::new(0).with_style(bar_style_no_total()))
    };
    bar.set_message(bold(display_name));
    bar.set_prefix("Downloading");
    bars.insert(id.to_string(), bar.clone());
    bar
}

fn finish_bar(id: &str, name: &str, status: &str) {
    let mut bars = BARS.lock().unwrap();
    if let Some(bar) = bars.remove(id) {
        bar.set_style(finished_style());
        bar.finish_with_message(format!("{} - {}", bold(name), status));
    }
}

pub fn manager() -> Arc<DownloadManager> {
    MANAGER.clone()
}

impl DownloadManager {
    fn next_id() -> String {
        crate::library::generate_id()
    }

    pub fn source_supports_repair(&self, key: &str) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.sources.get(key).is_some_and(|s| s.supports_repair())
    }

    pub fn enqueue(&self, req: DownloadRequest) -> String {
        let id = Self::next_id();
        let start_paused = req.start_paused;
        let initial_status = if start_paused {
            DownloadStatus::Paused
        } else {
            DownloadStatus::Queued
        };
        let entry = DownloadEntry {
            id: id.clone(),
            source: req.source,
            app_id: req.app_id,
            display_name: req.display_name,
            banner_url: req.banner_url,
            install_path: req.install_path,
            prefix_path: req.prefix_path,
            runner_version: req.runner_version,
            temp_dir: req.temp_dir,
            kind: req.kind,
            destructive_cleanup: req.destructive_cleanup,
            status: initial_status,
            progress: 0.0,
            bytes_downloaded: 0,
            bytes_total: 0,
            speed_bps: 0,
        };

        let need_worker = {
            let mut inner = self.inner.lock().unwrap();
            inner.entries.push(entry);
            inner.events.push_back(DownloadEvent::Added(id.clone()));
            save_queue(&inner.entries);
            if start_paused {
                false
            } else {
                let start = !inner.worker_started;
                if start {
                    inner.worker_started = true;
                }
                start
            }
        };

        if need_worker {
            Self::spawn_worker();
        }
        if !start_paused {
            self.notify.notify_one();
        }
        id
    }

    pub fn pause(&self, id: &str) {
        let mut inner = self.inner.lock().unwrap();
        let Some(e) = inner.entries.iter_mut().find(|e| e.id == id) else { return };
        match e.status {
            DownloadStatus::Downloading | DownloadStatus::Starting => {
                inner.control.insert(id.to_string(), ControlSignal::Pause);
            }
            DownloadStatus::Queued => {
                e.status = DownloadStatus::Paused;
                inner
                    .events
                    .push_back(DownloadEvent::StatusChanged(id.to_string(), DownloadStatus::Paused));
                save_queue(&inner.entries);
            }
            _ => {}
        }
    }

    pub fn resume(&self, id: &str) {
        let (should_notify, need_worker) = {
            let mut inner = self.inner.lock().unwrap();
            let Some(e) = inner.entries.iter_mut().find(|e| e.id == id) else { return };
            if e.status == DownloadStatus::Paused {
                e.status = DownloadStatus::Queued;
                inner
                    .events
                    .push_back(DownloadEvent::StatusChanged(id.to_string(), DownloadStatus::Queued));
                save_queue(&inner.entries);
                let start = !inner.worker_started;
                if start {
                    inner.worker_started = true;
                }
                (true, start)
            } else {
                (false, false)
            }
        };
        if need_worker {
            Self::spawn_worker();
        }
        if should_notify {
            self.notify.notify_one();
        }
    }

    pub fn cancel(&self, id: &str) {
        let mut inner = self.inner.lock().unwrap();
        let Some(idx) = inner.entries.iter().position(|e| e.id == id) else { return };
        let status = inner.entries[idx].status.clone();
        let entry_snapshot = inner.entries[idx].clone();
        match status {
            DownloadStatus::Downloading | DownloadStatus::Starting => {
                inner.control.insert(id.to_string(), ControlSignal::Cancel);
            }
            DownloadStatus::Paused => {
                // not running but partial files are on disk; drop them now
                // for Update entries install_path holds the user's exsisting game, dont wipe it
                // same for import flows (destructive_cleanup == false)
                inner.entries.remove(idx);
                inner.events.push_back(DownloadEvent::Removed(id.to_string()));
                save_queue(&inner.entries);
                drop(inner);
                finish_bar(id, &entry_snapshot.display_name, "cancelled");
                if matches!(entry_snapshot.kind, DownloadKind::Install)
                    && entry_snapshot.destructive_cleanup
                {
                    cleanup_install_dir_blocking(&entry_snapshot.install_path);
                }
                cleanup_source_state(&entry_snapshot);
            }
            _ => {
                inner.entries.remove(idx);
                inner.events.push_back(DownloadEvent::Removed(id.to_string()));
                save_queue(&inner.entries);
                drop(inner);
                finish_bar(id, &entry_snapshot.display_name, "removed");
            }
        }
    }

    pub fn retry(&self, id: &str) {
        let entry_copy = {
            let inner = self.inner.lock().unwrap();
            inner.entries.iter().find(|e| e.id == id).cloned()
        };
        let Some(entry) = entry_copy else { return };
        if !matches!(entry.status, DownloadStatus::Failed(_)) {
            return;
        }

        // hoyo keeps segments + a per-piece journal in a temp dir
        // if extraction failed those bytes are suspect and we want a truly fresh attempt rather than silently re-extracting teh same corrupt archive.
        if entry.source == "hoyo" {
            crate::hoyo::source::cleanup_hoyo_state(
                &entry.app_id,
                &entry.install_path,
                entry.temp_dir.as_deref(),
            );
        }

        let need_worker = {
            let mut inner = self.inner.lock().unwrap();
            if let Some(e) = inner.entries.iter_mut().find(|e| e.id == id) {
                e.status = DownloadStatus::Queued;
                e.progress = 0.0;
                e.bytes_downloaded = 0;
                e.speed_bps = 0;
                inner.events.push_back(DownloadEvent::StatusChanged(
                    id.to_string(),
                    DownloadStatus::Queued,
                ));
                save_queue(&inner.entries);
            }
            let start = !inner.worker_started;
            if start {
                inner.worker_started = true;
            }
            start
        };

        if need_worker {
            Self::spawn_worker();
        }
        self.notify.notify_one();
    }

    pub fn dismiss(&self, id: &str) {
        let mut inner = self.inner.lock().unwrap();
        let Some(idx) = inner.entries.iter().position(|e| e.id == id) else { return };
        match inner.entries[idx].status {
            DownloadStatus::Completed | DownloadStatus::Failed(_) | DownloadStatus::Cancelled => {
                inner.entries.remove(idx);
                inner.events.push_back(DownloadEvent::Removed(id.to_string()));
            }
            _ => {}
        }
    }

    pub fn list(&self) -> Vec<DownloadEntry> {
        self.inner.lock().unwrap().entries.clone()
    }

    pub fn get(&self, id: &str) -> Option<DownloadEntry> {
        self.inner.lock().unwrap().entries.iter().find(|e| e.id == id).cloned()
    }

    pub fn take_events(&self) -> Vec<DownloadEvent> {
        let mut inner = self.inner.lock().unwrap();
        inner.events.drain(..).collect()
    }

    fn spawn_worker() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("downloads worker runtime");
            rt.block_on(Self::worker_loop());
        });
    }

    async fn worker_loop() {
        let mgr = MANAGER.clone();
        loop {
            let next = {
                let inner = mgr.inner.lock().unwrap();
                inner
                    .entries
                    .iter()
                    .find(|e| e.status == DownloadStatus::Queued)
                    .cloned()
            };

            let Some(entry) = next else {
                mgr.notify.notified().await;
                continue;
            };

            let source = {
                let inner = mgr.inner.lock().unwrap();
                inner.sources.get(&entry.source).cloned()
            };

            let Some(source) = source else {
                set_failed(&entry.id, format!("unknown source: {}", entry.source));
                continue;
            };

            set_status(&entry.id, DownloadStatus::Starting);

            let result = match &entry.kind {
                DownloadKind::Install => source.install(&entry).await,
                DownloadKind::Update { .. } => source.update(&entry).await,
                DownloadKind::Repair => source.repair(&entry).await,
            };

            let final_signal = {
                let mut inner = mgr.inner.lock().unwrap();
                inner.control.remove(&entry.id).unwrap_or(ControlSignal::None)
            };

            match (result, final_signal) {
                (_, ControlSignal::Pause) => set_status(&entry.id, DownloadStatus::Paused),
                (_, ControlSignal::Cancel) => {
                    if matches!(entry.kind, DownloadKind::Install) && entry.destructive_cleanup {
                        cleanup_install_dir_blocking(&entry.install_path);
                    }
                    cleanup_source_state(&entry);
                    finish_bar(&entry.id, &entry.display_name, "cancelled");
                    let mgr = MANAGER.clone();
                    let mut inner = mgr.inner.lock().unwrap();
                    if let Some(idx) = inner.entries.iter().position(|e| e.id == entry.id) {
                        inner.entries.remove(idx);
                        inner.events.push_back(DownloadEvent::Removed(entry.id.clone()));
                        save_queue(&inner.entries);
                    }
                }
                (Ok(()), ControlSignal::None) => complete(&entry),
                (Err(e), ControlSignal::None) => set_failed(&entry.id, e.to_string()),
            }
        }
    }
}

// usable from both the tokio worker and the Qt thread (which has no tokio runtime)
pub fn cleanup_install_dir_blocking(path: &std::path::Path) {
    if !path.exists() {
        return;
    }
    if !is_safe_to_wipe(path) {
        tracing::error!("refusing to wipe suspicious path {} (defense-in-depth sanity check)", path.display());
        return;
    }
    if let Err(e) = std::fs::remove_dir_all(path) {
        tracing::error!("failed to clean up {}: {}", path.display(), e);
    } else {
        tracing::info!("cleaned up {}", path.display());
    }
}

// defense-in-depth guard, not a substitute for passing the right path, just a last line before remove_dir_all
fn is_safe_to_wipe(path: &std::path::Path) -> bool {
    use std::path::{Component, Path};

    // canonicalize strips `..`, symlinks, relative segments. if it fails (broken symlink, missing parent) refuse, we cant reason about it
    let canon = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };

    if canon.parent().is_none() {
        return false;
    }

    let comp_count = canon.components().count();
    let first_named = canon
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .next()
        .unwrap_or_default();
    let min_components = if matches!(first_named.as_str(), "mnt" | "media" | "run") {
        4
    } else {
        5
    };
    if comp_count < min_components {
        return false;
    }

    if let Some(home) = dirs::home_dir() {
        if canon == home {
            return false;
        }

        if canon.parent() == Some(home.as_path()) {
            return false;
        }
    }

    let blacklist: &[&str] = &[
        "/",
        "/home",
        "/root",
        "/usr",
        "/etc",
        "/var",
        "/opt",
        "/bin",
        "/sbin",
        "/lib",
        "/lib64",
        "/boot",
        "/dev",
        "/proc",
        "/sys",
        "/mnt",
        "/media",
        "/run",
        "/tmp",
        "/srv",
    ]; // may someone want to download stuff in weird places. 
    for entry in blacklist {
        if canon == Path::new(entry) {
            return false;
        }
    }

    let allowed_roots: &[&str] = &[
        "home", "mnt", "media", "run", "tmp", "opt", "var", "srv", "data",
    ];
    let mut comps = canon.components();
    if !matches!(comps.next(), Some(Component::RootDir)) {
        return false;
    }
    let first = match comps.next() {
        Some(Component::Normal(s)) => s.to_string_lossy().into_owned(),
        _ => return false,
    };
    if !allowed_roots.iter().any(|r| *r == first) {
        return false;
    }

    true
}

// epic: clear legendary's .resume chunk manifest
// hoyo: wipe the scratch dir (segments + .parts journals); otherwise a cancel leaves ~60 GB of dead split archives behind)
fn cleanup_source_state(entry: &DownloadEntry) {
    match entry.source.as_str() {
        "epic" => {
            if let Some(cfg) = dirs::config_dir() {
                let resume = cfg
                    .join("legendary")
                    .join("tmp")
                    .join(format!("{}.resume", entry.app_id));
                if resume.exists() {
                    if let Err(e) = std::fs::remove_file(&resume) {
                        tracing::error!("failed to clear resume state {}: {}", resume.display(), e);
                    } else {
                        tracing::debug!("cleared resume state for {}", entry.app_id);
                    }
                }
            }
        }
        "gog" => {
            // destructive_cleanup on Install kind already rm -rf's install_path
            // for us, so this is a no-op there. left explicit for symmetry
            let support = crate::gog::gog_dir()
                .join("support")
                .join(&entry.app_id);
            if support.exists() {
                let _ = std::fs::remove_dir_all(&support);
            }
        }
        "hoyo" => {
            crate::hoyo::source::cleanup_hoyo_state(
                &entry.app_id,
                &entry.install_path,
                entry.temp_dir.as_deref(),
            );
        }
        "endfield" => {
            crate::endfield::source::cleanup_endfield_state(
                &entry.app_id,
                &entry.install_path,
                entry.temp_dir.as_deref(),
            );
        }
        "kuro" => {
            crate::kuro::cleanup_kuro_state(
                &entry.app_id,
                &entry.install_path,
                entry.temp_dir.as_deref(),
            );
        }
        _ => {}
    }
}

pub fn report_progress(id: &str, progress: f64, bytes_downloaded: u64, bytes_total: u64, speed_bps: u64) {
    let mgr = MANAGER.clone();
    let mut inner = mgr.inner.lock().unwrap();

    let mut became_downloading = false;
    let display_name = inner.entries.iter().find(|e| e.id == id).map(|e| e.display_name.clone());
    if let Some(e) = inner.entries.iter_mut().find(|e| e.id == id) {
        if e.status == DownloadStatus::Starting {
            e.status = DownloadStatus::Downloading;
            became_downloading = true;
        }
        e.progress = progress;
        e.bytes_downloaded = bytes_downloaded;
        if bytes_total > 0 {
            e.bytes_total = bytes_total;
        }
        e.speed_bps = speed_bps;
    }

    if became_downloading {
        inner
            .events
            .push_back(DownloadEvent::StatusChanged(id.to_string(), DownloadStatus::Downloading));
    }
    inner.events.push_back(DownloadEvent::Progress {
        id: id.to_string(),
        progress,
        bytes_downloaded,
        bytes_total,
        speed_bps,
    });

    drop(inner);
    let name = display_name.unwrap_or_default();
    let bar = get_or_create_bar(id, &name, bytes_total);
    bar.set_position(bytes_downloaded);
}

pub fn check_control(id: &str) -> ControlSignal {
    let mgr = MANAGER.clone();
    let inner = mgr.inner.lock().unwrap();
    inner.control.get(id).copied().unwrap_or(ControlSignal::None)
}

pub fn set_status(id: &str, status: DownloadStatus) {
    let mgr = MANAGER.clone();
    let mut inner = mgr.inner.lock().unwrap();
    if let Some(e) = inner.entries.iter_mut().find(|e| e.id == id) {
        e.status = status.clone();
    }
    inner.events.push_back(DownloadEvent::StatusChanged(id.to_string(), status.clone()));
    save_queue(&inner.entries);
    drop(inner);

    let prefix = match status {
        DownloadStatus::Downloading => "Downloading",
        DownloadStatus::Extracting => "Extracting",
        DownloadStatus::Patching => "Patching",
        DownloadStatus::Starting => "Starting",
        DownloadStatus::Paused => "Paused",
        DownloadStatus::Queued => "Queued",
        _ => return,
    };
    let bars = BARS.lock().unwrap();
    if let Some(bar) = bars.get(id) {
        bar.set_prefix(prefix);
    }
}

fn set_failed(id: &str, err: String) {
    let mgr = MANAGER.clone();
    let mut inner = mgr.inner.lock().unwrap();
    let name = inner.entries.iter().find(|e| e.id == id).map(|e| e.display_name.clone());
    if let Some(e) = inner.entries.iter_mut().find(|e| e.id == id) {
        e.status = DownloadStatus::Failed(err.clone());
    }
    inner.events.push_back(DownloadEvent::Failed(id.to_string(), err));
    save_queue(&inner.entries);
    drop(inner);
    let label = name.unwrap_or_default();
    finish_bar(id, &label, "failed");
}

fn complete(entry: &DownloadEntry) {
    let mgr = MANAGER.clone();
    let mut inner = mgr.inner.lock().unwrap();
    if let Some(e) = inner.entries.iter_mut().find(|e| e.id == entry.id) {
        e.status = DownloadStatus::Completed;
        e.progress = 100.0;
    }
    inner.events.push_back(DownloadEvent::Completed {
        id: entry.id.clone(),
        source: entry.source.clone(),
        app_id: entry.app_id.clone(),
        display_name: entry.display_name.clone(),
        install_path: entry.install_path.clone(),
        prefix_path: entry.prefix_path.clone(),
        runner_version: entry.runner_version.clone(),
    });
    save_queue(&inner.entries);
    drop(inner);
    finish_bar(&entry.id, &entry.display_name, "done");
}

// active entries are saved to cache/downloads/queue.json so paused/queued
// downloads survive app restarts. legendary's .resume files handle chunk-level state; we just need to remember what was in 

fn queue_path() -> PathBuf {
    crate::cache_dir().join("downloads").join("queue.json")
}

fn save_queue(entries: &[DownloadEntry]) {
    let active: Vec<&DownloadEntry> = entries
        .iter()
        .filter(|e| e.status.is_active())
        .collect();

    let path = queue_path();
    if active.is_empty() {
        let _ = std::fs::remove_file(&path);
        return;
    }

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&active) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                tracing::error!("failed to save queue: {}", e);
            }
        }
        Err(e) => tracing::error!("failed to serialize queue: {}", e),
    }
}

fn load_queue() -> Vec<DownloadEntry> {
    let path = queue_path();
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let mut entries: Vec<DownloadEntry> = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("failed to parse queue.json: {}", e);
            return Vec::new();
        }
    };
    for e in &mut entries {
        e.status = DownloadStatus::Paused;
        e.speed_bps = 0;
    }
    entries
}
