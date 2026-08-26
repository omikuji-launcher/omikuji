// ok so i might note this is very funny and insanity pls understand
use anyhow::{Result, anyhow};
use std::net::SocketAddr;
use std::sync::OnceLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::downloads::throttle;

const RELAY_BUF: usize = 64 * 1024;
const MAX_HEAD: usize = 32 * 1024;

// both are a slice of time at the current rate: too small and downloads crawl
// since one socket only carries window/RTT, too large and it all arrives in one lump ffs
const READ_AHEAD: f64 = 0.01;
const RCVBUF_RANGE: (u64, u64) = (64 * 1024, 4 * 1024 * 1024);
const RELAY_SLICE: f64 = 0.001;
const RELAY_RANGE: (usize, usize) = (8 * 1024, RELAY_BUF);

static ADDR: OnceLock<SocketAddr> = OnceLock::new();

pub async fn address() -> Option<SocketAddr> {
    if let Some(addr) = ADDR.get() {
        return Some(*addr);
    }
    match start().await {
        Ok(addr) => Some(*ADDR.get_or_init(|| addr)),
        Err(e) => {
            tracing::error!(
                "bandwidth proxy failed to start: {} - downloads will run unthrottled",
                e
            );
            None
        }
    }
}

pub async fn env_vars() -> Vec<(String, String)> {
    if crate::app_settings::AppSettings::load()
        .download
        .bandwidth_mb_per_sec
        <= 0.0
    {
        return Vec::new();
    }
    let Some(addr) = address().await else {
        return Vec::new();
    };
    let url = format!("http://{}", addr);
    ["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY"]
        .iter()
        .map(|k| ((*k).to_string(), url.clone()))
        .collect()
}

async fn start() -> Result<SocketAddr> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((client, _)) => {
                    tokio::spawn(async move {
                        if let Err(e) = serve(client).await {
                            tracing::debug!("proxy conection ended: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("proxy accept failed: {}", e);
                    return;
                }
            }
        }
    });
    tracing::info!("bandwidth proxy listening on {}", addr);
    Ok(addr)
}

async fn serve(mut client: TcpStream) -> Result<()> {
    let (head, leftover) = read_head(&mut client).await?;
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or_default().to_string();
    let mut parts = request_line.split(' ');
    let method = parts.next().unwrap_or_default().to_string();
    let target = parts.next().unwrap_or_default().to_string();

    if method.eq_ignore_ascii_case("CONNECT") {
        tunnel(client, &target).await
    } else {
        forward(client, &method, &target, lines, &leftover).await
    }
}

async fn read_head(client: &mut TcpStream) -> Result<(String, Vec<u8>)> {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        let n = client.read(&mut chunk).await?;
        if n == 0 {
            return Err(anyhow!("client closed before sending a request"));
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(end) = find_head_end(&buf) {
            let head = String::from_utf8_lossy(&buf[..end]).into_owned();
            return Ok((head, buf[end + 4..].to_vec()));
        }
        if buf.len() > MAX_HEAD {
            return Err(anyhow!("request head too large"));
        }
    }
}

fn find_head_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

async fn tunnel(mut client: TcpStream, target: &str) -> Result<()> {
    let upstream = TcpStream::connect(target).await;
    let mut upstream = match upstream {
        Ok(s) => s,
        Err(e) => {
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return Err(anyhow!("connect {}: {}", target, e));
        }
    };
    clamp_read_ahead(&upstream);
    client
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await?;

    let (mut cr, mut cw) = client.split();
    let (mut ur, mut uw) = upstream.split();

    let up = tokio::io::copy(&mut cr, &mut uw);
    let down = relay_throttled(&mut ur, &mut cw);

    tokio::select! {
        r = up => { r?; }
        r = down => { r?; }
    }
    Ok(())
}

async fn forward<'a>(
    mut client: TcpStream,
    method: &str,
    target: &str,
    headers: impl Iterator<Item = &'a str>,
    leftover: &[u8],
) -> Result<()> {
    let rest = target
        .split_once("://")
        .map(|(_, r)| r)
        .ok_or_else(|| anyhow!("not an absolute request target: {}", target))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let host_port = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{}:80", authority)
    };

    let mut request = format!("{} {} HTTP/1.1\r\n", method, path);
    for line in headers {
        if line.is_empty() {
            continue;
        }
        let name = line.split(':').next().unwrap_or_default().trim();
        if name.eq_ignore_ascii_case("proxy-connection")
            || name.eq_ignore_ascii_case("connection")
            || name.eq_ignore_ascii_case("keep-alive")
        {
            continue;
        }
        request.push_str(line);
        request.push_str("\r\n");
    }
    request.push_str("Connection: close\r\n\r\n");

    let mut upstream = TcpStream::connect(&host_port).await?;
    clamp_read_ahead(&upstream);
    upstream.write_all(request.as_bytes()).await?;
    if !leftover.is_empty() {
        upstream.write_all(leftover).await?;
    }

    let (mut ur, mut uw) = upstream.split();
    let (mut cr, mut cw) = client.split();

    let up = tokio::io::copy(&mut cr, &mut uw);
    let down = relay_throttled(&mut ur, &mut cw);

    tokio::select! {
        r = up => { r?; }
        r = down => { r?; }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "needs network"]
    async fn tunnels_https_through_connect() {
        let addr = start().await.expect("proxy binds");
        let client = reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(format!("http://{}", addr)).unwrap())
            .build()
            .unwrap();
        let resp = client
            .get("https://api.github.com/zen")
            .header("user-agent", "omikuji-proxy-test")
            .send()
            .await
            .expect("request goes through the tunnel");
        assert!(resp.status().is_success());
        assert!(!resp.text().await.unwrap().is_empty());
    }
}

fn clamp_read_ahead(stream: &TcpStream) {
    use std::os::fd::AsRawFd;

    let rate = throttle::global().limit_bps();
    if rate == 0 {
        return;
    }
    let size =
        (((rate as f64 * READ_AHEAD) as u64).clamp(RCVBUF_RANGE.0, RCVBUF_RANGE.1)) as libc::c_int;
    unsafe {
        libc::setsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &size as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }
}

async fn relay_throttled<R, W>(from: &mut R, to: &mut W) -> Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut buf = vec![0u8; RELAY_BUF];
    loop {
        let rate = throttle::global().limit_bps();
        let quantum = if rate > 0 {
            ((rate as f64 * RELAY_SLICE) as usize).clamp(RELAY_RANGE.0, RELAY_RANGE.1)
        } else {
            RELAY_BUF
        };
        let n = from.read(&mut buf[..quantum]).await?;
        if n == 0 {
            let _ = to.shutdown().await;
            return Ok(());
        }
        to.write_all(&buf[..n]).await?;
        throttle::global().take(n).await;
    }
}
