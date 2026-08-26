use lazy_static::lazy_static;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::app_settings::AppSettings;
use crate::downloads::rate::RateMeter;

const BYTES_PER_MB: u64 = 1_000_000;
const BURST_SECONDS: f64 = 0.2;
const MIN_BURST: f64 = 64.0 * 1024.0;

lazy_static! {
    static ref THROTTLE: Throttle = Throttle::new();
}

pub struct Throttle {
    rate: AtomicU64,
    bucket: Mutex<Bucket>,
    wire: AtomicU64,
    meter: Mutex<RateMeter>,
}

struct Bucket {
    tokens: f64,
    last: Instant,
}

pub fn global() -> &'static Throttle {
    &THROTTLE
}

impl Throttle {
    fn new() -> Self {
        Self {
            rate: AtomicU64::new(0),
            bucket: Mutex::new(Bucket {
                tokens: 0.0,
                last: Instant::now(),
            }),
            wire: AtomicU64::new(0),
            meter: Mutex::new(RateMeter::new(0)),
        }
    }

    pub fn reload(&self) {
        let mb = AppSettings::load().download.bandwidth_mb_per_sec.max(0.0);
        self.rate
            .store((mb * BYTES_PER_MB as f64) as u64, Ordering::Relaxed);
        self.wire.store(0, Ordering::Relaxed);
        *self.meter.lock().unwrap() = RateMeter::new(0);
    }

    pub fn limit_bps(&self) -> u64 {
        self.rate.load(Ordering::Relaxed)
    }

    pub fn speed_bps(&self) -> u64 {
        let total = self.wire.load(Ordering::Relaxed);
        self.meter.lock().unwrap().update(total)
    }

    pub async fn take(&self, bytes: usize) {
        self.wire.fetch_add(bytes as u64, Ordering::Relaxed);

        let rate = self.rate.load(Ordering::Relaxed) as f64;
        if rate <= 0.0 {
            return;
        }
        let burst = (rate * BURST_SECONDS).max(MIN_BURST);
        let mut owed = bytes as f64;

        while owed > 0.0 {
            let wait = {
                let mut b = self.bucket.lock().unwrap();
                let now = Instant::now();
                b.tokens = (b.tokens + now.duration_since(b.last).as_secs_f64() * rate).min(burst);
                b.last = now;

                let spent = owed.min(b.tokens);
                b.tokens -= spent;
                owed -= spent;
                Duration::from_secs_f64(owed.min(burst) / rate)
            };

            if owed > 0.0 {
                tokio::time::sleep(wait).await;
            }
        }
    }
}
