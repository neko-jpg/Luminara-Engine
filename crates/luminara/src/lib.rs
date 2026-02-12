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

// ---------------------------------------------------------------------------
// DefaultPlugins — bundles all standard engine plugins
// ---------------------------------------------------------------------------

/// A plugin bundle that registers all standard Luminara Engine plugins.
///
/// The order of registration matters:
/// 1. **PlatformPlugin** — `Time` and `PlatformInfo` resources (required by everything)
/// 2. **DiagnosticPlugin** — logging initialisation and `FrameStats`
/// 3. **WindowPlugin** — opens a window and sets the `winit_runner`
/// 4. **InputPlugin** — keyboard / mouse / gamepad
/// 5. **AssetPlugin** — asset loading and hot-reload
/// 6. **RenderPlugin** — wgpu GPU context, pipeline cache, render systems
/// 7. **ScenePlugin** — transform hierarchy propagation
#[cfg(all(
    feature = "core",
    feature = "platform",
    feature = "diagnostic",
    feature = "window",
    feature = "input",
    feature = "asset",
    feature = "render",
    feature = "scene",
))]
pub struct DefaultPlugins;

#[cfg(all(
    feature = "core",
    feature = "platform",
    feature = "diagnostic",
    feature = "window",
    feature = "input",
    feature = "asset",
    feature = "render",
    feature = "scene",
))]
impl luminara_core::Plugin for DefaultPlugins {
    fn name(&self) -> &str {
        "DefaultPlugins"
    }

    fn build(&self, app: &mut luminara_core::App) {
        use luminara_core::shared_types::AppInterface;

        app.add_plugins(luminara_platform::plugin::PlatformPlugin);
        app.add_plugins(luminara_diagnostic::plugin::DiagnosticPlugin);
        app.add_plugins(luminara_window::WindowPlugin::default());
        app.add_plugins(luminara_input::plugin::InputPlugin);
        app.add_plugins(luminara_asset::AssetPlugin::default());
        app.add_plugins(luminara_render::RenderPlugin);
        app.add_plugins(luminara_scene::ScenePlugin);
    }
}

pub mod prelude {
    #[cfg(feature = "core")]
    pub use luminara_core::*;

    #[cfg(feature = "math")]
    pub use luminara_math::{Color, EulerRot, Mat4, Quat, Rect, Transform, Vec2, Vec3, Vec4};

    #[cfg(feature = "window")]
    pub use luminara_window::{Window, WindowDescriptor};

    #[cfg(feature = "input")]
    pub use luminara_input::Input;

    #[cfg(feature = "render")]
    pub use luminara_render::{Camera, Camera3d, Mesh, Projection};

    #[cfg(feature = "scene")]
    pub use luminara_scene::{Name, Prefab, Scene, ScenePlugin, Tag};

    #[cfg(all(
        feature = "core",
        feature = "platform",
        feature = "diagnostic",
        feature = "window",
        feature = "input",
        feature = "asset",
        feature = "render",
        feature = "scene",
    ))]
    pub use crate::DefaultPlugins;
}

