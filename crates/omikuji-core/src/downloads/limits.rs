use crate::app_settings::AppSettings;

const MAX_CONNECTIONS: i32 = 128;
const MAX_PATCH_THREADS: i32 = 32;
const MAX_WORKERS: i32 = 128;
const MAX_SHARED_MEMORY_MB: i32 = 16384;

#[derive(Debug, Clone, Copy)]
pub struct GachaLimits {
    pub connections: usize,
    pub patch_threads: usize,
}

impl GachaLimits {
    pub fn load() -> Self {
        let g = &AppSettings::load().download.gacha;
        Self {
            connections: g.max_connections.clamp(1, MAX_CONNECTIONS) as usize,
            patch_threads: g.patch_threads.clamp(1, MAX_PATCH_THREADS) as usize,
        }
    }

    pub fn nested(&self, inner: usize) -> (usize, usize) {
        let inner = inner.clamp(1, self.connections);
        (self.connections / inner, inner)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StoreLimits {
    pub workers: i32,
    pub shared_memory_mb: Option<i32>,
}

impl StoreLimits {
    pub fn epic() -> Self {
        let e = &AppSettings::load().download.epic;
        Self {
            workers: e.workers.clamp(0, MAX_WORKERS),
            shared_memory_mb: Some(e.shared_memory_mb.clamp(0, MAX_SHARED_MEMORY_MB)),
        }
    }

    pub fn gog() -> Self {
        Self {
            workers: AppSettings::load()
                .download
                .gog
                .workers
                .clamp(0, MAX_WORKERS),
            shared_memory_mb: None,
        }
    }

    pub fn args(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.workers > 0 {
            out.push("--max-workers".to_string());
            out.push(self.workers.to_string());
        }
        if let Some(mb) = self.shared_memory_mb.filter(|&mb| mb > 0) {
            out.push("--max-shared-memory".to_string());
            out.push(mb.to_string());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_stays_within_budget() {
        for connections in 1..=MAX_CONNECTIONS as usize {
            let limits = GachaLimits {
                connections,
                patch_threads: 4,
            };
            let (outer, inner) = limits.nested(4);
            assert!(outer >= 1 && inner >= 1);
            assert!(outer * inner <= connections);
        }
    }
}
