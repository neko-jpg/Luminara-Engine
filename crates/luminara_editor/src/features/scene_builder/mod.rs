//! Scene Builder Module
//!
//! A modular, well-structured Scene Builder implementation that recreates
//! the HTML prototype UI using GPUI.
//!
//! ## Module Structure
//! - `box_`: Main SceneBuilderBox container
//! - `menu_bar`: Top menu bar (File, Edit, View, etc.)
//! - `toolbar`: Main toolbar with transform tools
//! - `hierarchy`: Scene hierarchy panel (left)
//! - `viewport`: 3D viewport panel (center)
//! - `inspector`: Inspector panel (right)
//! - `bottom_tabs`: Bottom tab panel (Console, Assets, DB Query, AI)

pub mod box_;
pub mod bottom_tabs;
pub mod hierarchy;
pub mod inspector;
pub mod menu_bar;
pub mod toolbar;
pub mod viewport;

// Re-export main types
pub use box_::{SceneBuilderBox, SceneBuilderState};
pub use bottom_tabs::{BottomTab, BottomTabPanel};
pub use hierarchy::{HierarchyPanel, HierarchyItem};
pub use inspector::{InspectorPanel, TransformEditor};
pub use menu_bar::{MenuBar, MenuItem};
pub use toolbar::{MainToolbar, ToolMode};
pub use viewport::{Viewport3D, GridBackground};
