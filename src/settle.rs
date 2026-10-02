//! Notices when a value has stopped changing, for "do this once the user is done editing".

use std::time::{Duration, Instant};

pub struct Settle {
    last: u64,
    since: Instant,
}

impl Settle {
    pub fn new(value: u64) -> Self {
        Self {
            last: value,
            since: Instant::now(),
        }
    }

    /// Feed the current value once per frame.
    pub fn observe(&mut self, value: u64, now: Instant) {
        if value != self.last {
            (self.last, self.since) = (value, now);
        }
    }

    pub fn quiet_for(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.since)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_time_resets_on_every_change() {
        let t0 = Instant::now();
        let mut s = Settle { last: 1, since: t0 };
        s.observe(1, t0 + Duration::from_secs(5));
        assert_eq!(s.quiet_for(t0 + Duration::from_secs(5)), Duration::from_secs(5));
        s.observe(2, t0 + Duration::from_secs(6));
        assert_eq!(s.quiet_for(t0 + Duration::from_secs(7)), Duration::from_secs(1));
    }
}
