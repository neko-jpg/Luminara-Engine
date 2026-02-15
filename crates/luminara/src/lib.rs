pub mod prelude {
    // Manually re-export main types instead of using prelude modules if they don't exist
    #[cfg(feature = "core")]
    pub use luminara_core::shared_types::*;
    #[cfg(feature = "core")]
    pub use luminara_core::world::World;

    #[cfg(feature = "math")]
    pub use luminara_math::{Quat, Transform, Vec3};

    #[cfg(feature = "input")]
    pub use luminara_input::{keyboard::Key, mouse::MouseButton, Input};

    #[cfg(feature = "ai_agent")]
    pub use luminara_ai_agent::*;
    #[cfg(feature = "script")]
    pub use luminara_script::*;

    #[cfg(feature = "db")]
    pub use luminara_db::prelude::*;
}

#[cfg(feature = "ai_agent")]
pub use luminara_ai_agent as ai_agent;
#[cfg(feature = "asset")]
pub use luminara_asset as asset;
#[cfg(feature = "audio")]
pub use luminara_audio as audio;
#[cfg(feature = "core")]
pub use luminara_core as core;
#[cfg(feature = "diagnostic")]
pub use luminara_diagnostic as diagnostic;
#[cfg(feature = "input")]
pub use luminara_input as input;
#[cfg(feature = "math")]
pub use luminara_math as math;
#[cfg(feature = "physics")]
pub use luminara_physics as physics;
#[cfg(feature = "platform")]
pub use luminara_platform as platform;
#[cfg(feature = "render")]
pub use luminara_render as render;
#[cfg(feature = "scene")]
pub use luminara_scene as scene;
#[cfg(feature = "script")]
pub use luminara_script as script;
#[cfg(feature = "script_lua")]
pub use luminara_script_lua as script_lua;
#[cfg(feature = "script_wasm")]
pub use luminara_script_wasm as script_wasm;
#[cfg(feature = "window")]
pub use luminara_window as window;
#[cfg(feature = "db")]
pub use luminara_db as db;
