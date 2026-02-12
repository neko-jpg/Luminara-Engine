use std::io::Write;

/// Engine logging macros (re-export standard log macros)
pub use log::{debug, error, info, trace, warn};

/// Initializes the logger with default settings.
/// It respects the `RUST_LOG` environment variable, defaulting to `Info` level.
pub fn init_logging() {
    init_logging_with_level(log::LevelFilter::Info);
}

/// Initializes the logger with a specific default log level.
pub fn init_logging_with_level(level: log::LevelFilter) {
    // We use a try_init to avoid panics if called multiple times (e.g. in tests)
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level.to_string()))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {:5} {}] {}",
                buf.timestamp_millis(),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .try_init();
}

/// Logs a message only once.
#[macro_export]
macro_rules! log_once {
    ($level:expr, $($arg:tt)*) => {
        {
            use std::sync::atomic::{AtomicBool, Ordering};
            static OCCURRED: AtomicBool = AtomicBool::new(false);
            if !OCCURRED.swap(true, Ordering::Relaxed) {
                $crate::logging::log!($level, $($arg)*);
            }
        }
    };
}

/// Logs a message at most once every `interval_secs`.
#[macro_export]
macro_rules! log_throttled {
    ($interval_secs:expr, $level:expr, $($arg:tt)*) => {
        {
            use std::sync::Mutex;
            use $crate::reexports::instant::{Instant, Duration};
            static LAST_LOGGED: Mutex<Option<Instant>> = Mutex::new(None);
            if let Ok(mut last) = LAST_LOGGED.lock() {
                let now = Instant::now();
                let should_log = match *last {
                    None => true,
                    Some(last_time) => now.duration_since(last_time) >= Duration::from_secs_f32($interval_secs as f32),
                };
                if should_log {
                    *last = Some(now);
                    $crate::logging::log!($level, $($arg)*);
                }
            }
        }
    };
}

// Internal re-export for macros
#[doc(hidden)]
pub use log::log;
