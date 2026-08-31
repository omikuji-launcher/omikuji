use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const GAME_ID_VAR: &str = "OMIKUJI_GAME_ID";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(pub u64);

static ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_id() -> ProcessId {
    ProcessId(ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
}

#[derive(Debug, Clone)]
pub enum ProcessState {
    Running {
        pid: u32,
        started_at: Instant,
    },
    Exited {
        code: Option<i32>,
        playtime_secs: u64,
    },
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub id: ProcessId,
    pub game_id: String,
    pub game_name: String,
    pub state: ProcessState,
    pub log_path: PathBuf,
}

pub struct ProcessManager {
    sessions: Arc<Mutex<HashMap<ProcessId, GameSession>>>,
    logs_dir: PathBuf,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            logs_dir: crate::logs_dir(),
        }
    }

    pub async fn launch(&self, config: &crate::launch::LaunchConfig) -> Result<ProcessId> {
        let game = crate::library::Library::load_game_by_id(&config.game_id)?
            .ok_or_else(|| anyhow::anyhow!("game not found in library"))?;

        if game.is_epic() {
            let wine_exe = config
                .env
                .get("WINE")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("wine"));
            let _ = crate::launch::prepare_epic_prefix(&game, &wine_exe, &config.env);
        }

        // steam manages its own prefix, skip dll injection for it
        if game.runner.runner_type != "steam"
            && let Err(e) = crate::dll_packs::inject_all(&game, &config.env)
        {
            tracing::warn!("dll pack injection failed: {} (launching anyway)", e);
        }

        if game.runner.runner_type != "steam"
            && let Some(runner_dir) = crate::runners::runner_dir(&game.wine.version)
            && !crate::store::steam::local::under_steamapps_common(&runner_dir)
            && let Err(e) =
                crate::runners::dll_override::apply_for_launch(&runner_dir, &game, &config.env)
        {
            tracing::warn!("runner dll sync failed: {} (launching anyway)", e);
        }

        // download saves before the game opens its save files
        if game.is_epic()
            && game.source.cloud_saves
            && !game.source.save_path.is_empty()
            && let Err(e) =
                crate::store::epic::sync_saves_download(&game.source.app_id, &game.source.save_path)
        {
            tracing::warn!("cloud save download failed: {} (launching anyway)", e);
        }

        // drop stale in-memory log from the previous session before streaming new lines
        crate::game_logs::reset_log(&config.game_id);

        // save_game_logs is opt-in; we still run ther reader so the log viewer works
        let save_to_disk = crate::app_settings::AppSettings::load()
            .behavior
            .save_game_logs;
        let log_path = if save_to_disk {
            tokio::fs::create_dir_all(&self.logs_dir).await.ok();
            self.logs_dir.join(format!(
                "{}_{}.log",
                config.game_id,
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            ))
        } else {
            // placeholder only, never opened
            self.logs_dir
                .join(format!("{}_ephemeral.log", config.game_id))
        };

        let header = format!(
            "=== omikuji log {} ===\ncommand: {:?}\nworking_dir: {}\nenv count: {}\n---",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            config.command,
            config.working_dir.display(),
            config.env.len()
        );
        for line in header.lines() {
            crate::game_logs::append_line(&config.game_id, line.to_string());
        }

        let mut cmd = std::process::Command::new(&config.command[0]);
        if config.command.len() > 1 {
            cmd.args(&config.command[1..]);
        }
        cmd.current_dir(&config.working_dir);
        cmd.env_clear();
        cmd.envs(&config.env);
        cmd.env(GAME_ID_VAR, &config.game_id);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        // setsid makes the child the session leader so we can kill the entire tree (wine, proton, umu-run, the actual game) by sid later.
        // steam manages its own lifecycle, breaks with session detachment.
        #[cfg(unix)]
        if game.runner.runner_type != "steam" {
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    nix::unistd::setsid()
                        .map(|_| ())
                        .map_err(|e| std::io::Error::from_raw_os_error(e as i32))
                });
            }
        }

        if matches!(game.runner.runner_type.as_str(), "native" | "flatpak") {
            crate::launch::alongside::start_host(&game, &config.env);
        }

        let mut child = cmd.spawn()?;
        let pid = child.id();

        tracing::info!("spawned pid: {}", pid);

        crate::discord::set_playing(&game);

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();
        let (log_tx, log_rx) = std::sync::mpsc::channel::<String>();
        {
            let game_id = config.game_id.clone();
            let log_path_for_writer = if save_to_disk {
                Some(log_path.clone())
            } else {
                None
            };
            std::thread::spawn(move || {
                use std::io::Write;
                let mut file = log_path_for_writer.as_ref().and_then(|p| {
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(p)
                        .ok()
                });
                while let Ok(line) = log_rx.recv() {
                    if let Some(ref mut f) = file {
                        let _ = writeln!(f, "{}", line);
                    }
                    crate::game_logs::append_line(&game_id, line);
                }
            });
        }
        if let Some(stdout) = stdout_pipe {
            let tx = log_tx.clone();
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(|l| l.ok()) {
                    let _ = tx.send(line);
                }
            });
        }
        if let Some(stderr) = stderr_pipe {
            let tx = log_tx;
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(|l| l.ok()) {
                    let _ = tx.send(line);
                }
            });
        }

        let proc_id = next_id();
        let started_at = Instant::now();

        let session = GameSession {
            id: proc_id,
            game_id: config.game_id.clone(),
            game_name: config.game_name.clone(),
            state: ProcessState::Running { pid, started_at },
            log_path: log_path.clone(),
        };

        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(proc_id, session);
        }

        let sessions = self.sessions.clone();
        let game_id = config.game_id.clone();
        let post_exit_script = config.post_exit_script.clone();
        let working_dir = config.working_dir.clone();
        let env = config.env.clone();

        std::thread::spawn(move || {
            let exit_status = child.wait();
            let exit_code = exit_status.ok().and_then(|s| s.code());

            // wait until every process in the session exits, not just the
            // tracked parent. legendary and umu-run hand off to wine, so the
            // parent exits early while the real game is still running.
            if let Ok(Some(game)) = crate::library::Library::load_game_by_id(&game_id) {
                if game.runner.runner_type != "steam" {
                    while session_has_live_process(pid) {
                        std::thread::sleep(Duration::from_millis(500));
                    }
                }

                // upload only after game has quit and released save files
                if game.is_epic()
                    && game.source.cloud_saves
                    && !game.source.save_path.is_empty()
                    && let Err(e) = crate::store::epic::sync_saves_upload(
                        &game.source.app_id,
                        &game.source.save_path,
                    )
                {
                    tracing::warn!(pid, "cloud save upload failed: {}", e);
                }
            }

            let playtime_secs = started_at.elapsed().as_secs();

            {
                let mut sessions = sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(&proc_id) {
                    session.state = ProcessState::Exited {
                        code: exit_code,
                        playtime_secs,
                    };
                }
            }

            tracing::info!(
                pid,
                "game '{}' exited with code {:?}, playtime: {}s",
                game_id,
                exit_code,
                playtime_secs
            );

            crate::discord::clear();

            notify_game_exited(&game_id);

            if !post_exit_script.is_empty() {
                tracing::info!(pid, "running post-exit script: {}", post_exit_script);
                let status = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&post_exit_script)
                    .current_dir(&working_dir)
                    .envs(&env)
                    .status();
                match status {
                    Ok(s) if !s.success() => {
                        tracing::warn!(pid, "post-exit script exited with: {}", s)
                    }
                    Err(e) => tracing::error!(pid, "post-exit script failed: {}", e),
                    _ => {}
                }
            }

            match crate::library::Library::load_game_by_id(&game_id) {
                Ok(Some(mut game)) => {
                    game.metadata.playtime += playtime_secs as f64 / 3600.0;
                    game.metadata.last_played =
                        chrono::Local::now().format("%b %-d, %Y").to_string();
                    if let Err(e) = crate::library::Library::save_game_static(&game) {
                        tracing::error!(pid, "failed to save playtime: {}", e);
                    }
                }
                Ok(None) => {
                    tracing::warn!(
                        pid,
                        "game '{}' not found on disk, skipping playtime save",
                        game_id
                    );
                }
                Err(e) => {
                    tracing::error!(pid, "failed to load game for playtime save: {}", e);
                }
            }
        });

        Ok(proc_id)
    }

    pub fn find_by_game_id(&self, game_id: &str) -> Option<GameSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .find(|s| s.game_id == game_id && matches!(s.state, ProcessState::Running { .. }))
            .cloned()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    static ref MANAGER: ProcessManager = ProcessManager::new();
}

use std::collections::{HashSet, VecDeque};

lazy_static::lazy_static! {
    static ref EXITED_GAMES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
    static ref LAUNCHING: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref LAUNCH_REQUESTS: Mutex<VecDeque<LaunchRequest>> = Mutex::new(VecDeque::new());
    static ref EXIT_WAITERS: Mutex<ExitWaiters> = Mutex::new(HashMap::new());
    static ref MARKED_IDS: Mutex<(Option<Instant>, HashSet<String>)> =
        Mutex::new((None, HashSet::new()));
}

type ExitWaiters = HashMap<String, Vec<(u64, tokio::sync::oneshot::Sender<()>)>>;

const MAX_LAUNCH_REQUESTS: usize = 10;
const MARKED_IDS_TTL: Duration = Duration::from_millis(400);
static WAITER_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub fn mark_launching(game_id: &str) {
    if let Ok(mut set) = LAUNCHING.lock() {
        set.insert(game_id.to_string());
    }
}

pub fn clear_launching(game_id: &str) {
    if let Ok(mut set) = LAUNCHING.lock() {
        set.remove(game_id);
    }
}

pub fn is_launching(game_id: &str) -> bool {
    LAUNCHING.lock().is_ok_and(|s| s.contains(game_id))
}

pub struct LaunchSoulGuard {
    _lock: nix::fcntl::Flock<std::fs::File>,
}
// yes this is a dbd reference, yes im mentally ill, yes fuck you too

fn launch_lock_path(game_id: &str) -> PathBuf {
    let mut dir = dirs::runtime_dir().unwrap_or_else(std::env::temp_dir);
    dir.push(format!("omikuji-launch-{}.lock", game_id));
    dir
}

pub fn launch_blocked(game_id: &str) -> bool {
    try_claim_launch(game_id).is_none()
}

pub fn try_claim_launch(game_id: &str) -> Option<LaunchSoulGuard> {
    if is_game_running(game_id) {
        return None;
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(launch_lock_path(game_id))
        .ok()?;

    nix::fcntl::Flock::lock(file, nix::fcntl::FlockArg::LockExclusiveNonblock)
        .ok()
        .map(|lock| LaunchSoulGuard { _lock: lock })
}

pub struct LaunchRequest {
    pub game_id: String,
    waiter: u64,
}

impl LaunchRequest {
    pub fn release(&self) {
        let Ok(mut map) = EXIT_WAITERS.lock() else {
            return;
        };
        let Some(waiters) = map.get_mut(&self.game_id) else {
            return;
        };
        if let Some(pos) = waiters.iter().position(|(id, _)| *id == self.waiter) {
            let (_, tx) = waiters.remove(pos);
            let _ = tx.send(());
        }
        if waiters.is_empty() {
            map.remove(&self.game_id);
        }
    }
}

pub fn release_exit_waiters(game_id: &str) {
    if let Ok(mut map) = EXIT_WAITERS.lock()
        && let Some(waiters) = map.remove(game_id)
    {
        for (_, tx) in waiters {
            let _ = tx.send(());
        }
    }
}

pub fn request_launch(game_id: &str) -> tokio::sync::oneshot::Receiver<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let waiter = WAITER_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    if let Ok(mut map) = EXIT_WAITERS.lock() {
        map.entry(game_id.to_string())
            .or_default()
            .push((waiter, tx));
    }

    let evicted: Vec<LaunchRequest> = match LAUNCH_REQUESTS.lock() {
        Ok(mut queue) => {
            queue.push_back(LaunchRequest {
                game_id: game_id.to_string(),
                waiter,
            });
            let excess = queue.len().saturating_sub(MAX_LAUNCH_REQUESTS);
            queue.drain(..excess).collect()
        }
        Err(_) => Vec::new(),
    };
    for request in evicted {
        request.release();
    }

    rx
}

pub fn take_launch_requests() -> Vec<LaunchRequest> {
    LAUNCH_REQUESTS
        .lock()
        .map(|mut q| q.drain(..).collect())
        .unwrap_or_default()
}

pub fn notify_game_exited(game_id: &str) {
    clear_launching(game_id);
    release_exit_waiters(game_id);
    if let Ok(mut queue) = EXITED_GAMES.lock() {
        queue.push_back(game_id.to_string());
        while queue.len() > 10 {
            queue.pop_front();
        }
    }
}

pub fn take_exited_games() -> Vec<String> {
    if let Ok(mut queue) = EXITED_GAMES.lock() {
        let games: Vec<String> = queue.drain(..).collect();
        games
    } else {
        vec![]
    }
}

// pre-launch update-required notification. consumed by the bridge
// (drain + emit signal + show popup in QML).
#[derive(Debug, Clone)]
pub struct UpdateNotification {
    pub game_id: String,
    // passed back to the update enqueue path so the source knows which game to patch
    pub app_id: String,
    pub from_version: String,
    pub to_version: String,
    // compressed delta bytes (0 when can_diff=false)
    pub download_size: u64,
    // false = server has no diff path, full reinstall required
    pub can_diff: bool,
    // false = game doesnt ship deltas at all (e.g. hi3 today), distinct from "your version too old to delta"
    pub delta_supported: bool,
}

lazy_static::lazy_static! {
    static ref UPDATE_REQUIRED: Mutex<VecDeque<UpdateNotification>> = Mutex::new(VecDeque::new());
}

pub fn notify_update_required(n: UpdateNotification) {
    if let Ok(mut q) = UPDATE_REQUIRED.lock() {
        q.push_back(n);
        while q.len() > 10 {
            q.pop_front();
        }
    }
}

pub fn take_update_notifications() -> Vec<UpdateNotification> {
    if let Ok(mut q) = UPDATE_REQUIRED.lock() {
        q.drain(..).collect()
    } else {
        vec![]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] // no ma ci sta averne 80, magari aggiungiamoci anche parametri di fisica quantistica
pub enum ErrorAction {
    None,
    OpenGameSettings,
    OpenGlobalSettings,
}

impl ErrorAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::OpenGameSettings => "open_game_settings",
            Self::OpenGlobalSettings => "open_global_settings",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorNotification {
    pub game_id: String,
    pub title: String,
    pub message: String,
    pub action: ErrorAction,
}

lazy_static::lazy_static! {
    static ref ERRORS: Mutex<VecDeque<ErrorNotification>> = Mutex::new(VecDeque::new());
}

pub fn notify_error(n: ErrorNotification) {
    if let Ok(mut q) = ERRORS.lock() {
        q.push_back(n);
        while q.len() > 10 {
            q.pop_front();
        }
    }
}

pub fn take_errors() -> Vec<ErrorNotification> {
    if let Ok(mut q) = ERRORS.lock() {
        q.drain(..).collect()
    } else {
        vec![]
    }
}

pub fn manager() -> &'static ProcessManager {
    &MANAGER
}

pub async fn launch_game(config: &crate::launch::LaunchConfig) -> Result<ProcessId> {
    manager().launch(config).await
}

pub fn is_game_running(game_id: &str) -> bool {
    manager().find_by_game_id(game_id).is_some() || running_marked_ids().contains(game_id)
}

// non-blocking stop. schedules SIGTERM on the whole session, then SIGKILL 3s
//later if anything survives. returns true if a tracked session was found.
pub fn stop_game(game_id: &str) -> bool {
    let session_pid = match manager().find_by_game_id(game_id).map(|s| s.state) {
        Some(ProcessState::Running { pid, .. }) => Some(pid),
        _ => None,
    };
    let markers = marker_pids(game_id);
    if session_pid.is_none() && markers.is_empty() {
        return false;
    }

    // for non-steam games, pid == sid (we called setsid on spawn. [my head hurts])
    let is_steam = crate::library::Library::load_game_by_id(game_id)
        .ok()
        .flatten()
        .map(|g| g.runner.runner_type == "steam")
        .unwrap_or(false);

    tracing::info!("stopping game '{}' (pid: {:?})", game_id, session_pid);

    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;

        if is_steam {
            if let Some(pid) = session_pid {
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
            }
            return true;
        }

        let game_id = game_id.to_string();
        std::thread::spawn(move || {
            let gather = || {
                let mut pids = session_pid.map(session_pids).unwrap_or_default();
                for p in marker_pids(&game_id) {
                    if !pids.contains(&p) {
                        pids.push(p);
                    }
                }
                pids
            };

            let term_pids = gather();
            tracing::debug!("SIGTERM to {} tracked pids", term_pids.len());
            for p in &term_pids {
                let _ = kill(Pid::from_raw(*p as i32), Signal::SIGTERM);
            }

            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if gather().is_empty() {
                    tracing::info!("game '{}' stopped gracefully", game_id);
                    return;
                }
            }

            let kill_pids = gather();
            tracing::warn!(
                "game '{}' ignored SIGTERM, SIGKILLing {} survivors",
                game_id,
                kill_pids.len()
            );
            for p in &kill_pids {
                let _ = kill(Pid::from_raw(*p as i32), Signal::SIGKILL);
            }
        });
    }

    #[cfg(not(unix))]
    {
        tracing::error!("stop not implemented for non-unix platforms");
        return false;
    }

    true
}

// comm can contain spaces and parens so we parse from the last ')' as the reliable field delimiter
#[cfg(target_os = "linux")]
fn session_pids(sid: u32) -> Vec<u32> {
    let my_pid = std::process::id();
    let mut pids = Vec::new();

    let Ok(entries) = std::fs::read_dir("/proc") else {
        return pids;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        let Ok(pid) = name_str.parse::<u32>() else {
            continue;
        };
        if pid == my_pid {
            continue;
        }

        if session_of(pid) == Some(sid) {
            pids.push(pid);
        }
    }
    pids
}

#[cfg(target_os = "linux")]
fn marked_processes() -> Vec<(u32, String)> {
    let key = format!("{GAME_ID_VAR}=");
    let key = key.as_bytes();
    let my_pid = std::process::id();
    let mut found = Vec::new();

    let Ok(entries) = std::fs::read_dir("/proc") else {
        return found;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        let Ok(pid) = name_str.parse::<u32>() else {
            continue;
        };
        if pid == my_pid {
            continue;
        }

        let Ok(environ) = std::fs::read(entry.path().join("environ")) else {
            continue;
        };
        if let Some(id) = environ
            .split(|b| *b == 0)
            .find_map(|var| var.strip_prefix(key))
        {
            found.push((pid, String::from_utf8_lossy(id).into_owned()));
        }
    }
    found
}

fn marker_pids(game_id: &str) -> Vec<u32> {
    marked_processes()
        .into_iter()
        .filter(|(_, id)| id == game_id)
        .map(|(pid, _)| pid)
        .collect()
}

// i mean the launcher isnt even meant at all outside linux but well, do it now and forget it forever
#[cfg(not(target_os = "linux"))]
fn session_pids(_sid: u32) -> Vec<u32> {
    Vec::new()
}

#[cfg(not(target_os = "linux"))]
fn marker_pids(_game_id: &str) -> Vec<u32> {
    Vec::new()
}

#[cfg(not(target_os = "linux"))]
fn marked_processes() -> Vec<(u32, String)> {
    Vec::new()
}

fn session_of(pid: u32) -> Option<u32> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rparen = stat.rfind(')')?;
    stat[rparen + 1..].split_whitespace().nth(3)?.parse().ok()
}

fn runs_exe(pid: u32, exe_name: &str) -> bool {
    let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) else {
        return false;
    };
    let comm = comm.trim().to_lowercase();
    !comm.is_empty() && exe_name.to_lowercase().starts_with(&comm)
}

fn running_marked_ids() -> HashSet<String> {
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
            crate::library::Library::load_game_by_id(game_id)
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

fn session_has_live_process(sid: u32) -> bool {
    !session_pids(sid).is_empty()
}
