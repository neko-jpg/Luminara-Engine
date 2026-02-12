pub use log::{debug, error, info, trace, warn};
pub use instant;

pub fn init_logging() {
    init_logging_with_level(log::LevelFilter::Info);
}

pub fn init_logging_with_level(level: log::LevelFilter) {
    let mut builder = env_logger::Builder::from_default_env();
    builder
        .filter_level(level)
        .format_timestamp_millis();

    // Safety: ignore error if already initialized
    let _ = builder.try_init();
}

#[macro_export]
macro_rules! log_once {
    ($level:expr, $($arg:tt)*) => {
        {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| {
                $crate::logging::log!($level, $($arg)*);
            });
        }
    };
}

// Internal helper for macros to use log!
#[doc(hidden)]
pub use log::log;

#[macro_export]
macro_rules! log_throttled {
    ($interval_secs:expr, $level:expr, $($arg:tt)*) => {
        {
            static LAST_LOG_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            static START_TIME: std::sync::OnceLock<$crate::logging::instant::Instant> = std::sync::OnceLock::new();
            let start = *START_TIME.get_or_init($crate::logging::instant::Instant::now);
            let now_ms = $crate::logging::instant::Instant::now().duration_since(start).as_millis() as u64;

            let last = LAST_LOG_MS.load(std::sync::atomic::Ordering::Relaxed);
            let interval_ms = ($interval_secs as f64 * 1000.0) as u64;

            if now_ms >= last + interval_ms {
                if LAST_LOG_MS.compare_exchange(last, now_ms, std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed).is_ok() {
                    $crate::logging::log!($level, $($arg)*);
                }
            }
        }
    };
}
