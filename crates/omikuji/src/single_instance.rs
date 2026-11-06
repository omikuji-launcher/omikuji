use std::os::unix::net::UnixStream as BlockingUnixStream;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use omikuji_core::process::{self, ErrorNotification};

use crate::bridge::tray;

const MSG_FOCUS: &[u8] = b"focus";
const MSG_ERRORS: &[u8] = b"errors ";
const MSG_LAUNCH: &[u8] = b"launch ";
const MSG_STOP: &[u8] = b"stop ";
const HANDOFF_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

fn send_game_message(prefix: &[u8], game_id: &str) -> Option<BlockingUnixStream> {
    use std::io::Write;

    let mut stream = BlockingUnixStream::connect(socket_path()).ok()?;
    stream
        .write_all(&[prefix, game_id.as_bytes()].concat())
        .ok()?;
    stream.shutdown(std::net::Shutdown::Write).ok()?;
    Some(stream)
}

pub fn hand_off_launch(game_id: &str) -> bool {
    use std::io::Read;

    let Some(mut stream) = send_game_message(MSG_LAUNCH, game_id) else {
        return false;
    };

    let stop_id = game_id.to_owned();
    process::on_stop_signal(move || {
        send_game_message(MSG_STOP, &stop_id);
    });

    let _ = stream.read(&mut [0u8; 1]);
    true
}

fn handoff_payload() -> Vec<u8> {
    let pending = process::take_errors();
    if pending.is_empty() {
        return MSG_FOCUS.to_vec();
    }

    match serde_json::to_vec(&pending) {
        Ok(json) => [MSG_ERRORS, &json].concat(),
        Err(e) => {
            tracing::error!("failed to encode errors for handoff: {e}");
            MSG_FOCUS.to_vec()
        }
    }
}

fn accept_handoff(buf: &[u8]) {
    if let Some(json) = buf.strip_prefix(MSG_ERRORS) {
        match serde_json::from_slice::<Vec<ErrorNotification>>(json) {
            Ok(items) => items.into_iter().for_each(process::notify_error),
            Err(e) => tracing::error!("failed to decode handed off errors: {e}"),
        }
    } else if buf != MSG_FOCUS {
        return;
    }

    tray::omikuji_tray_event_show();
}

async fn dispatch_message(buf: &[u8]) {
    let game_id = |id| String::from_utf8_lossy(id).into_owned();

    if let Some(id) = buf.strip_prefix(MSG_LAUNCH) {
        let _ = process::request_launch(&game_id(id)).await;
    } else if let Some(id) = buf.strip_prefix(MSG_STOP) {
        process::stop_game(&game_id(id));
    } else {
        accept_handoff(buf);
    }
}

fn socket_path() -> PathBuf {
    match dirs::runtime_dir() {
        Some(mut p) => {
            p.push("omikuji.sock");
            p
        }
        None => {
            let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            PathBuf::from(format!("/tmp/omikuji-{}.sock", user))
        }
    }
}

struct SocketGuard(Arc<PathBuf>);

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = fs_err::remove_file(self.0.as_ref());
    }
}

pub async fn check() -> bool {
    // whatever works right?
    let bypass = std::env::var("OMIKUJI_BYPASS_SINGLE_INSTANCE").is_ok();
    let path = socket_path();

    if !bypass {
        if let Ok(mut stream) = UnixStream::connect(&path).await {
            let _ = stream.write_all(&handoff_payload()).await;
            let _ = stream.shutdown().await;
            return false;
        }
    } else {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let _ = fs_err::remove_file(&path);
    if let Some(parent) = path.parent() {
        let _ = fs_err::create_dir_all(parent);
    }

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("failed to bind socket at {path:?}: {e}");
            return true;
        }
    };

    let guard = SocketGuard(Arc::new(path));

    tokio::spawn(async move {
        let _guard = guard;

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    tokio::spawn(async move {
                        let mut buf = Vec::new();
                        let read =
                            tokio::time::timeout(HANDOFF_TIMEOUT, stream.read_to_end(&mut buf))
                                .await;

                        match read {
                            Ok(Ok(n)) if n > 0 => dispatch_message(&buf).await,
                            Ok(Ok(_)) => {}
                            Ok(Err(e)) => tracing::error!("read error: {e}"),
                            Err(_) => tracing::error!("timed out reading from socket"),
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("accept error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    });

    true
}
