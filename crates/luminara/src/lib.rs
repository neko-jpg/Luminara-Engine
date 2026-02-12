#[cfg(feature = "core")]
pub use luminara_core as core;

#[cfg(feature = "math")]
pub use luminara_math as math;

#[cfg(feature = "window")]
pub use luminara_window as window;

#[cfg(feature = "input")]
pub use luminara_input as input;

#[cfg(feature = "asset")]
pub use luminara_asset as asset;

#[cfg(feature = "render")]
pub use luminara_render as render;

#[cfg(feature = "scene")]
pub use luminara_scene as scene;

#[cfg(feature = "platform")]
pub use luminara_platform as platform;

#[cfg(feature = "diagnostic")]
pub use luminara_diagnostic as diagnostic;

pub mod prelude {
    #[cfg(feature = "core")]
    pub use luminara_core::*;

    #[cfg(feature = "math")]
    pub use luminara_math::{Color, Mat4, Quat, Transform, Vec2, Vec3, Vec4};

    #[cfg(feature = "window")]
    pub use luminara_window::{Window, WindowDescriptor};

    #[cfg(feature = "scene")]
    pub use luminara_scene::{Prefab, Scene, ScenePlugin};
}
