// poll pattern instead of qt_thread.queue becuase queued closures werent reaching qml reliably

use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct InstallSizeResult {
    pub request_id: String,
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub error: String,
}

lazy_static::lazy_static! {
    static ref SIZE_QUEUE: Mutex<VecDeque<InstallSizeResult>> = Mutex::new(VecDeque::new());
}

pub fn push(result: InstallSizeResult) {
    let Ok(mut q) = SIZE_QUEUE.lock() else { return };
    q.push_back(result);
    while q.len() > 20 {
        q.pop_front();
    }
}

pub fn take_pending() -> Vec<InstallSizeResult> {
    SIZE_QUEUE
        .lock()
        .map(|mut q| q.drain(..).collect())
        .unwrap_or_default()
}

// os thread + fresh runtime: cant call block_on inside the app's existing tokio context
pub fn spawn_fetch<F, Fut>(request_id: String, fetch: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<(u64, u64), String>>,
{
    std::thread::spawn(move || {
        let result = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt.block_on(fetch()),
            Err(e) => Err(format!("tokio runtime: {}", e)),
        };
        let pushed = match result {
            Ok((download_bytes, install_bytes)) => InstallSizeResult {
                request_id,
                download_bytes,
                install_bytes,
                error: String::new(),
            },
            Err(error) => {
                tracing::error!("install size fetch failed: {}", error);
                InstallSizeResult {
                    request_id,
                    download_bytes: 0,
                    install_bytes: 0,
                    error,
                }
            }
        };
        push(pushed);
    });
}

#[derive(Debug, Clone)]
pub struct FileDialogResult {
    pub request_id: String,
    pub path: String,
}

lazy_static::lazy_static! {
    static ref FILE_DIALOG_QUEUE: Mutex<VecDeque<FileDialogResult>> = Mutex::new(VecDeque::new());
}

pub fn push_file_dialog(result: FileDialogResult) {
    let Ok(mut q) = FILE_DIALOG_QUEUE.lock() else { return };
    q.push_back(result);
    while q.len() > 20 {
        q.pop_front();
    }
}

pub fn take_file_dialog_pending() -> Vec<FileDialogResult> {
    FILE_DIALOG_QUEUE
        .lock()
        .map(|mut q| q.drain(..).collect())
        .unwrap_or_default()
}
