use luminara_core::shared_types::Resource;
use instant::{Duration, Instant};

// Since we don't have a derive macro for Resource yet, implement it manually
// #[derive(Resource)]
#[derive(Debug, Clone)]
pub struct Time {
    startup: Instant,
    last_frame: Instant,
    delta: Duration,
    delta_seconds: f32,
    elapsed: Duration,
    elapsed_seconds: f32,
    frame_count: u64,
    fixed_timestep: f32,
    fixed_accumulator: f32,
}

impl Resource for Time {}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            startup: now,
            last_frame: now,
            delta: Duration::from_secs(0),
            delta_seconds: 0.0,
            elapsed: Duration::from_secs(0),
            elapsed_seconds: 0.0,
            frame_count: 0,
            fixed_timestep: 1.0 / 60.0, // Default to 60Hz fixed update
            fixed_accumulator: 0.0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        self.delta = delta;
        self.delta_seconds = delta.as_secs_f32();

        self.elapsed = now - self.startup;
        self.elapsed_seconds = self.elapsed.as_secs_f32();

        self.frame_count += 1;

        // Accumulate time for fixed update
        // Cap large delta times to avoid spiral of death (e.g. max 0.25s)
        let delta_clamped = self.delta_seconds.min(0.25);
        self.fixed_accumulator += delta_clamped;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn fps(&self) -> f32 {
        if self.delta_seconds > 0.0 {
            1.0 / self.delta_seconds
        } else {
            0.0
        }
    }

    // FixedUpdate logic
    pub fn fixed_timestep(&self) -> f32 {
        self.fixed_timestep
    }

    // This function checks if we have enough accumulated time to run a fixed update step.
    // If true, it consumes the time step from the accumulator.
    // It should be called in a loop in the FixedUpdate stage system runner until it returns false.
    pub fn should_run_fixed_update(&mut self) -> bool {
        if self.fixed_accumulator >= self.fixed_timestep {
            self.fixed_accumulator -= self.fixed_timestep;
            true
        } else {
            false
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}
