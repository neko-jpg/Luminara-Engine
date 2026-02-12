use num_cpus;

pub struct ThreadConfig {
    pub num_worker_threads: usize,
    pub stack_size: usize,
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self {
            num_worker_threads: num_cpus::get().max(2) - 1,
            stack_size: 2 * 1024 * 1024, // 2MB
        }
    }
}
