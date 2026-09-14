use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::downloads::limits::GachaLimits;
use crate::downloads::rate::RateMeter;
use crate::downloads::throttle;
use crate::downloads::{ControlSignal, check_control, report_progress};

// re-gating a verified list re-skips what the hashes condemned, i hate all of this
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Skip {
    SameSize,
    Never,
}

pub fn expected_md5(hex: &str) -> Option<String> {
    (!hex.is_empty()).then(|| hex.to_ascii_lowercase())
}

pub fn file_md5(path: &Path) -> Result<String> {
    use md5::{Digest, Md5};
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Debug, Clone)]
pub struct SyncFile {
    pub rel_path: String,
    pub size: u64,
    pub url: String,
    pub md5: Option<String>,
}

pub struct SyncProgress {
    done: AtomicU64,
    total: u64,
    meter: Mutex<RateMeter>,
}

impl SyncProgress {
    pub fn new(total: u64) -> Arc<Self> {
        Arc::new(Self {
            done: AtomicU64::new(0),
            total,
            meter: Mutex::new(RateMeter::new(0)),
        })
    }

    fn advance(&self, id: &str, delta: u64) {
        self.done.fetch_add(delta, Ordering::Relaxed);
        self.tick(id);
    }

    fn rewind(&self, delta: u64) {
        let capped = delta.min(self.done.load(Ordering::Relaxed));
        self.done.fetch_sub(capped, Ordering::Relaxed);
    }

    fn tick(&self, id: &str) {
        let done = self.done.load(Ordering::Relaxed);
        let pct = if self.total > 0 {
            (done as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };
        let bps = self.meter.lock().unwrap().update(done);
        report_progress(id, pct, done, self.total, bps);
    }
}

// flat indexes disagree on wehter sizes are numbers or decimal strings
pub fn deserialize_size<'de, D>(d: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    use serde::de::Error;
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| D::Error::custom("not a u64")),
        serde_json::Value::String(s) => s.parse().map_err(D::Error::custom),
        other => Err(D::Error::custom(format!(
            "unexpected size type: {:?}",
            other
        ))),
    }
}

pub fn sanitize_rel(rel: &str) -> PathBuf {
    rel.split(['/', '\\'])
        .filter(|p| !p.is_empty() && *p != "." && *p != "..")
        .collect()
}

fn size_matches(path: &Path, size: u64) -> bool {
    matches!(std::fs::metadata(path), Ok(m) if m.len() == size)
}

pub fn is_stale(path: &Path, size: u64, md5: Option<&str>) -> bool {
    if !size_matches(path, size) {
        return true;
    }
    md5.is_some_and(|want| file_md5(path).map_or(true, |got| got != want))
}

pub async fn select_stale(
    id: &str,
    files: Vec<SyncFile>,
    dest_root: &Path,
) -> Result<Option<Vec<SyncFile>>> {
    let total: u64 = files.iter().map(|f| f.size).sum();
    let progress = SyncProgress::new(total);
    let lanes = std::thread::available_parallelism().map_or(4, |n| n.get());

    let stream = futures_util::stream::iter(files.into_iter().map(|file| {
        let id = id.to_string();
        let path = dest_root.join(sanitize_rel(&file.rel_path));
        let progress = progress.clone();
        async move {
            if check_control(&id) != ControlSignal::None {
                return Ok::<_, anyhow::Error>(None);
            }
            let (size, md5) = (file.size, file.md5.clone());
            let stale =
                tokio::task::spawn_blocking(move || is_stale(&path, size, md5.as_deref())).await?;
            progress.advance(&id, size);
            Ok(stale.then_some(file))
        }
    }))
    .buffer_unordered(lanes);

    tokio::pin!(stream);
    let mut stale = Vec::new();
    while let Some(res) = stream.next().await {
        if let Some(file) = res? {
            stale.push(file);
        }
        if check_control(id) != ControlSignal::None {
            return Ok(None);
        }
    }
    Ok(Some(stale))
}

pub async fn download_one(
    id: &str,
    file: &SyncFile,
    dest_root: &Path,
    progress: &SyncProgress,
    skip: Skip,
) -> Result<()> {
    let dest_path = dest_root.join(sanitize_rel(&file.rel_path));
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if skip == Skip::SameSize && size_matches(&dest_path, file.size) {
        progress.advance(id, file.size);
        return Ok(());
    }

    let existing = std::fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
    let resume = existing > 0 && existing < file.size;

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| anyhow!("reqwest client: {}", e))?;
    let mut req = client.get(&file.url);
    if resume {
        req = req.header("Range", format!("bytes={}-", existing));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| anyhow!("GET {}: {}", file.url, e))?;
    let status = resp.status();
    if !(status.is_success() || status.as_u16() == 206) {
        anyhow::bail!("GET {}: http {}", file.url, status);
    }

    // a full body means overwrite, only 206 is a real resume
    let append = status.as_u16() == 206 && resume;
    let mut writer: Box<dyn std::io::Write + Send> = if append {
        Box::new(std::fs::OpenOptions::new().append(true).open(&dest_path)?)
    } else {
        progress.rewind(existing);
        Box::new(std::fs::File::create(&dest_path)?)
    };
    if append {
        progress.advance(id, existing);
    }

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if check_control(id) != ControlSignal::None {
            return Ok(());
        }
        let bytes = chunk.map_err(|e| anyhow!("network: {}", e))?;
        use std::io::Write;
        writer
            .write_all(&bytes)
            .map_err(|e| anyhow!("write: {}", e))?;
        progress.advance(id, bytes.len() as u64);
        throttle::global().take(bytes.len()).await;
    }
    writer.flush()?;
    Ok(())
}

pub async fn sync_all(
    id: &str,
    files: Vec<SyncFile>,
    dest_root: &Path,
    progress: Arc<SyncProgress>,
    skip: Skip,
) -> Result<bool> {
    let files = match skip {
        Skip::SameSize => {
            let root = dest_root.to_path_buf();
            let (present, missing): (Vec<_>, Vec<_>) = tokio::task::spawn_blocking(move || {
                files
                    .into_iter()
                    .partition(|f| size_matches(&root.join(sanitize_rel(&f.rel_path)), f.size))
            })
            .await?;
            progress.advance(id, present.iter().map(|f| f.size).sum());
            missing
        }
        Skip::Never => files,
    };

    let stream = futures_util::stream::iter(files.into_iter().map(|file| {
        let id = id.to_string();
        let dest_root = dest_root.to_path_buf();
        let progress = progress.clone();
        async move {
            if check_control(&id) != ControlSignal::None {
                return Ok::<_, anyhow::Error>(());
            }
            download_one(&id, &file, &dest_root, &progress, Skip::Never).await
        }
    }))
    .buffer_unordered(GachaLimits::load().connections);

    tokio::pin!(stream);
    while let Some(res) = stream.next().await {
        res?;
        if check_control(id) != ControlSignal::None {
            return Ok(false);
        }
    }
    Ok(true)
}
