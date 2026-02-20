//! Director Box - Timeline and Animation Editor
//!
//! The Director is a timeline-based animation and cutscene editor with:
//! - Timeline editor with keyframe tracks
//! - Viewport preview for camera paths and animations
//! - Keyframe inspector for editing animation properties
//! - Transport controls (play, pause, stop, seek)
//! - Track management (add, remove, organize tracks)

pub mod director_box;
pub mod inspector;
pub mod timeline;
pub mod toolbar;
pub mod viewport_panel;

pub use director_box::DirectorBox;
pub use inspector::KeyframeInspector;
pub use timeline::{Timeline, Track, TrackKind, Keyframe, KeyframeKind};
pub use toolbar::DirectorToolbar;
pub use viewport_panel::DirectorViewportPanel;
