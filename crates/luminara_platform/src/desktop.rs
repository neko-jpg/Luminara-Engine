use crate::{PlatformInfo, Os, Arch};

pub fn get_platform_info() -> PlatformInfo {
    let os = if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::MacOS
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else if cfg!(target_arch = "wasm32") {
        Os::Web
    } else if cfg!(target_os = "android") {
        Os::Android
    } else if cfg!(target_os = "ios") {
        Os::Ios
    } else {
        Os::Unknown
    };

    let arch = if cfg!(target_arch = "x86_64") {
        Arch::X86_64
    } else if cfg!(target_arch = "aarch64") {
        Arch::Aarch64
    } else if cfg!(target_arch = "wasm32") {
        Arch::Wasm32
    } else {
        Arch::Unknown
    };

    let is_debug = cfg!(debug_assertions);

    PlatformInfo {
        os,
        arch,
        is_debug,
    }
}
