use std::time::{Duration, Instant};

const WINDOW: Duration = Duration::from_millis(250);
const SMOOTHING: f64 = 0.25;

pub struct RateMeter {
    last_bytes: u64,
    last_time: Instant,
    last_bps: u64,
}

impl RateMeter {
    pub fn new(initial_bytes: u64) -> Self {
        Self {
            last_bytes: initial_bytes,
            last_time: Instant::now(),
            last_bps: 0,
        }
    }

    pub fn update(&mut self, total_bytes: u64) -> u64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time);
        if elapsed < WINDOW {
            return self.last_bps;
        }
        let instant = total_bytes.saturating_sub(self.last_bytes) as f64 / elapsed.as_secs_f64();
        self.last_bps = if self.last_bps == 0 {
            instant as u64
        } else {
            (SMOOTHING * instant + (1.0 - SMOOTHING) * self.last_bps as f64) as u64
        };
        self.last_bytes = total_bytes;
        self.last_time = now;
        self.last_bps
    }
}
