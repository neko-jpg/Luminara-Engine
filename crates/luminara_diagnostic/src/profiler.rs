use std::collections::{HashMap, VecDeque};
use instant::{Duration, Instant};
use std::cell::RefCell;
use std::cell::RefCell;
use instant::{Instant, Duration};

pub struct ProfileScope {
    pub name: String,
    pub samples: VecDeque<Duration>,
    pub max_samples: usize,
}

impl ProfileScope {
    pub fn new(name: String, max_samples: usize) -> Self {
        Self {
            name,
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record(&mut self, duration: Duration) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(duration);
    }

    pub fn average(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::from_secs(0);
        }
        let total: Duration = self.samples.iter().sum();
        total / self.samples.len() as u32
        let sum: Duration = self.samples.iter().sum();
        sum / self.samples.len() as u32
    }

    pub fn max(&self) -> Duration {
        self.samples.iter().max().cloned().unwrap_or(Duration::from_secs(0))
    }

    pub fn min(&self) -> Duration {
        self.samples.iter().min().cloned().unwrap_or(Duration::from_secs(0))
    }
}

pub struct Profiler {
    pub scopes: HashMap<String, ProfileScope>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }

    pub fn record(&mut self, name: &'static str, duration: Duration) {
        let scope = self.scopes.entry(name.to_string()).or_insert_with(|| {
            ProfileScope::new(name.to_string(), 120) // Default 120 samples
        });
        scope.record(duration);
    }
}

thread_local! {
    pub static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::new());
}

pub struct ProfileGuard {
    name: &'static str,
    start: Instant,
}

impl ProfileGuard {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        PROFILER.with(|p| p.borrow_mut().record(self.name, duration));
    }
}

#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _guard = $crate::profiler::ProfileGuard::new($name);
    };
}
