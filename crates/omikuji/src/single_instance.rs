use std::fs;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

pub async fn check() -> bool {
    let socket_path = match dirs::runtime_dir() {
        Some(mut p) => {
            p.push("omikuji.sock");
            p
        }
        None => {
            let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            PathBuf::from(format!("/tmp/omikuji-{}.sock", user))
        }
    };

    if let Ok(mut stream) = UnixStream::connect(&socket_path).await {
        let _ = stream.write_all(b"focus").await;
        return false;
    }

    let _ = fs::remove_file(&socket_path);

    if let Some(parent) = socket_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    tokio::spawn(async move {
        if let Ok(listener) = UnixListener::bind(&socket_path) {
            loop {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut buf = [0u8; 5];
                    if let Ok(n) = stream.read(&mut buf).await {
                        if n > 0 && &buf[..n] == b"focus" {
                            crate::bridge::tray::omikuji_tray_event_show();
                        }
                    }
                }
            }
        }
    });

    true
}
