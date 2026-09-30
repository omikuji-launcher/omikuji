// os thread + fresh runtime: cant call block_on inside the app's existing tokio context
pub fn spawn<F, Fut, T, C>(fetch: F, complete: C)
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

pub fn block_on<F, Fut, T>(fetch: F) -> Result<T, String>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = T>,
    T: Send + 'static,
{
    std::thread::spawn(move || match tokio::runtime::Runtime::new() {
        Ok(rt) => Ok(rt.block_on(fetch())),
        Err(e) => Err(format!("tokio runtime: {}", e)),
    })
    .join()
    .map_err(|_| "background worker panicked".to_string())?
}
