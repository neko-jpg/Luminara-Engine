use std::time::{Instant, Duration};

pub struct Time {
    startup: Instant,
    last_update: Instant,
    delta: Duration,
    elapsed: Duration,
}

impl Default for Time {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            startup: now,
            last_update: now,
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta = now - self.last_update;
        self.elapsed = now - self.startup;
        self.last_update = now;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }
}
