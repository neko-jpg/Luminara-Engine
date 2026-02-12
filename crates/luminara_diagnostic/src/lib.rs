pub mod logging;
pub mod profiler;
pub mod diagnostics;
pub mod frame_stats;
pub mod plugin;

pub use logging::{init_logging, init_logging_with_level, debug, error, info, trace, warn};
pub use profiler::{Profiler, ProfileScope, ProfileGuard, PROFILER};
pub use diagnostics::{Diagnostics, DiagnosticEntry};
pub use frame_stats::FrameStats;
pub use plugin::DiagnosticPlugin;
