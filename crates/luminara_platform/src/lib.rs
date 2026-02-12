pub mod desktop;
pub mod file_system;
pub mod plugin;
pub mod thread;
pub mod time;

pub use file_system::FileSystem;
pub use plugin::PlatformPlugin;
pub use thread::ThreadConfig;
pub use time::Time;

use luminara_core::shared_types::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOS,
    Linux,
    Web,
    Android,
    Ios,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X86_64,
    Aarch64,
    Wasm32,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os: Os,
    pub arch: Arch,
    pub is_debug: bool,
}

impl Resource for PlatformInfo {}

impl PlatformInfo {
    pub fn current() -> Self {
        desktop::get_platform_info()
    }
}
