#[derive(Debug, Default)]
pub struct SessionTally {
    total: u64,
    session_total: Option<u64>,
    session_done: Option<u64>,
}

impl SessionTally {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            ..Self::default()
        }
    }

    pub fn set_session_total(&mut self, bytes: u64) {
        self.session_total = Some(bytes);
        if self.total == 0 {
            self.total = bytes;
        }
    }

    pub fn set_session_done(&mut self, bytes: u64) {
        self.session_done = Some(bytes);
    }

    pub fn session_done(&self) -> u64 {
        self.session_done.unwrap_or(0)
    }

    pub fn snapshot(&self) -> Option<(f64, u64, u64)> {
        let (session_total, session_done) = (self.session_total?, self.session_done?);
        if self.total == 0 {
            return None;
        }
        let done = (session_done + self.total.saturating_sub(session_total)).min(self.total);
        Some((done as f64 / self.total as f64 * 100.0, done, self.total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_continues_from_earlier_runs() {
        const MIB: u64 = 1024 * 1024;
        let mut tally = SessionTally::new(1000 * MIB);
        tally.set_session_total(600 * MIB);
        assert!(tally.snapshot().is_none());
        tally.set_session_done(0);
        assert_eq!(tally.snapshot(), Some((40.0, 400 * MIB, 1000 * MIB)));
    }
}
