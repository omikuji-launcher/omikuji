use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

const MSG_FOCUS: &[u8] = b"focus";

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
        let _ = std::fs::remove_file(self.0.as_ref());
    }
}

pub async fn check() -> bool {
    // whatever works right?
    let bypass = std::env::var("OMIKUJI_BYPASS_SINGLE_INSTANCE").is_ok();
    let path = socket_path();

    if !bypass {
        if let Ok(mut stream) = UnixStream::connect(&path).await {
            let _ = stream.write_all(MSG_FOCUS).await;
            return false;
        }
    } else {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let _ = std::fs::remove_file(&path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[single_instance] Failed to bind socket at {path:?}: {e}");
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
                        let mut buf = vec![0u8; 64];
                        match stream.read(&mut buf).await {
                            Ok(n) if n > 0 && &buf[..n] == MSG_FOCUS => {
                                crate::bridge::tray::omikuji_tray_event_show();
                            }
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("[single_instance] Read error: {e}");
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[single_instance] Accept error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    });

    true
}
