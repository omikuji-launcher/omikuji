use std::backtrace::Backtrace;
use std::borrow::Cow;
use std::ffi::{CStr, c_char, c_int};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::Level;
use tracing_subscriber::filter::{EnvFilter, FilterExt, Targets, filter_fn};
use tracing_subscriber::fmt::writer::{MakeWriter, OptionalWriter};
use tracing_subscriber::prelude::*;

use crate::log_fmt::ShortTarget;

const STEM: &str = "omikuji";
const KEEP_SESSIONS: usize = 3;
const MAX_BYTES: u64 = 64 * 1024 * 1024;

// these already reach stderr through qt's own handler and the default panic hook
const QT_TARGET: &str = "qt";
const PANIC_TARGET: &str = "panic";

const QT_DEBUG: c_int = 0;
const QT_WARNING: c_int = 1;
const QT_INFO: c_int = 4;

unsafe extern "C" {
    fn omikuji_install_qt_log();
}

struct SessionLog {
    file: File,
    written: AtomicU64,
}

impl Write for &SessionLog {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len() as u64;
        let before = self.written.fetch_add(len, Ordering::Relaxed);
        if before >= MAX_BYTES {
            return Ok(buf.len());
        }
        (&self.file).write_all(buf)?;
        if before + len >= MAX_BYTES {
            (&self.file)
                .write_all(b"log size cap reached, the rest of this session is not written\n")?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.file).flush()
    }
}

static SESSION: OnceLock<SessionLog> = OnceLock::new();

struct SessionWriter;

impl<'a> MakeWriter<'a> for SessionWriter {
    type Writer = OptionalWriter<&'a SessionLog>;

    fn make_writer(&'a self) -> Self::Writer {
        SESSION.get().into()
    }
}

pub fn init() {
    let stderr_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn"))
        .and(filter_fn(|meta| {
            !matches!(meta.target(), QT_TARGET | PANIC_TARGET)
        }));
    let file_filter = Targets::new()
        .with_target("omikuji", Level::DEBUG)
        .with_target("omikuji_core", Level::DEBUG)
        .with_target(QT_TARGET, Level::DEBUG)
        .with_target(PANIC_TARGET, Level::ERROR)
        .with_default(Level::WARN);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(ShortTarget)
                .with_filter(stderr_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(ShortTarget)
                .with_ansi(false)
                .with_writer(SessionWriter)
                .with_filter(file_filter),
        )
        .init();
}

pub fn start_session() {
    let dir = omikuji_core::logs_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!("couldn't create {}: {}", dir.display(), e);
        return;
    }
    prune_sessions(&dir);

    let path = omikuji_core::stamped_log_path(STEM);
    let file = match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            tracing::warn!("couldn't create {}: {}", path.display(), e);
            return;
        }
    };
    let log = SessionLog {
        file,
        written: AtomicU64::new(0),
    };
    if SESSION.set(log).is_err() {
        return;
    }

    install_panic_hook();
    unsafe { omikuji_install_qt_log() };
    tracing::info!(
        "omikuji {} session log at {}",
        env!("CARGO_PKG_VERSION"),
        path.display()
    );
}

fn prune_sessions(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut sessions: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_session_log(path))
        .collect();
    sessions.sort();
    let excess = (sessions.len() + 1).saturating_sub(KEEP_SESSIONS);
    for path in sessions.into_iter().take(excess) {
        if let Err(e) = std::fs::remove_file(&path) {
            tracing::warn!("couldn't remove {}: {}", path.display(), e);
        }
    }
}

fn is_session_log(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix(STEM))
        .and_then(|rest| rest.strip_prefix('_'))
        .and_then(|rest| rest.strip_suffix(".log"))
        .is_some_and(|stamp| {
            stamp.len() == "YYYYmmdd_HHMMSS".len()
                && stamp.chars().all(|c| c.is_ascii_digit() || c == '_')
        })
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(target: PANIC_TARGET, "{info}\n{}", Backtrace::force_capture());
        default_hook(info);
    }));
}

fn c_text<'a>(ptr: *const c_char) -> Cow<'a, str> {
    if ptr.is_null() {
        return Cow::Borrowed("");
    }
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy()
}

#[unsafe(no_mangle)]
pub extern "C" fn omikuji_qt_log_message(
    level: c_int,
    category: *const c_char,
    message: *const c_char,
) {
    let category = c_text(category);
    let message = c_text(message);
    match level {
        QT_DEBUG => tracing::debug!(target: QT_TARGET, "{category}: {message}"),
        QT_INFO => tracing::info!(target: QT_TARGET, "{category}: {message}"),
        QT_WARNING => tracing::warn!(target: QT_TARGET, "{category}: {message}"),
        _ => tracing::error!(target: QT_TARGET, "{category}: {message}"),
    }
}
