use crate::event_queue::EventQueue;
// poll pattern instead of qt_thread.queue becuase queued closures werent reaching qml reliably

#[derive(Debug, Clone)]
pub struct InstallSizeResult {
    pub request_id: String,
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub launch_exe: String,
    pub dlcs: String,
    pub error: String,
}

static SIZE_QUEUE: EventQueue<InstallSizeResult> = EventQueue::new(20);

pub fn push(result: InstallSizeResult) {
    SIZE_QUEUE.push(result);
}

pub fn take_pending() -> Vec<InstallSizeResult> {
    SIZE_QUEUE.drain()
}

// os thread + fresh runtime: cant call block_on inside the app's existing tokio context
fn spawn_blocking_fetch<F, Fut, T, C>(fetch: F, complete: C)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T, String>>,
    T: Send + 'static,
    C: FnOnce(Result<T, String>) + Send + 'static,
{
    std::thread::spawn(move || {
        let result = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt.block_on(fetch()),
            Err(e) => Err(format!("tokio runtime: {}", e)),
        };
        complete(result);
    });
}

pub fn spawn_fetch<F, Fut>(request_id: String, fetch: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<(u64, u64), String>>,
{
    spawn_fetch_ex(request_id, move || async move {
        fetch()
            .await
            .map(|(d, i)| (d, i, String::new(), String::new()))
    });
}

pub fn spawn_fetch_ex<F, Fut>(request_id: String, fetch: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<(u64, u64, String, String), String>>,
{
    spawn_blocking_fetch(fetch, move |result| {
        let pushed = match result {
            Ok((download_bytes, install_bytes, launch_exe, dlcs)) => InstallSizeResult {
                request_id,
                download_bytes,
                install_bytes,
                launch_exe,
                dlcs,
                error: String::new(),
            },
            Err(error) => {
                tracing::error!("install size fetch failed: {}", error);
                InstallSizeResult {
                    request_id,
                    download_bytes: 0,
                    install_bytes: 0,
                    launch_exe: String::new(),
                    dlcs: String::new(),
                    error,
                }
            }
        };
        push(pushed);
    });
}

#[derive(Debug, Clone)]
pub struct GameDetailsResult {
    pub request_id: String,
    pub payload: String,
}

static DETAILS_QUEUE: EventQueue<GameDetailsResult> = EventQueue::new(20);

pub fn take_details_pending() -> Vec<GameDetailsResult> {
    DETAILS_QUEUE.drain()
}

pub fn spawn_fetch_details<F, Fut>(request_id: String, fetch: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    spawn_blocking_fetch(fetch, move |result| {
        let payload = match result {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("game details fetch failed: {}", e);
                String::new()
            }
        };
        DETAILS_QUEUE.push(GameDetailsResult {
            request_id,
            payload,
        });
    });
}
