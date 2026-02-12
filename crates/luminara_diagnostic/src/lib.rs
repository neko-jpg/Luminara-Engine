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
pub use logging::*;
pub use profiler::*;
pub use diagnostics::*;
pub use frame_stats::*;
pub use plugin::*;

/// Internal re-exports for macros
#[doc(hidden)]
pub mod reexports {
    pub use instant;
}
