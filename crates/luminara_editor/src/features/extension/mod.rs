//! Extension Manager Module
//!
//! Provides a comprehensive extension management interface with:
//! - Installed extensions list with toggle switches (left panel)
//! - Extension details with manifest info (center panel)
//! - Marketplace and development tools (right panel)
//! - Extension console and API docs (bottom panel)
//!
//! Layout structure matching HTML prototype:
//! ```
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │ Menu Bar (File, Edit, Extensions, Marketplace, Development, Help)       │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │ Toolbar (Check Updates, Install, New, Installed, Marketplace, Develop) │
//! ├──────────────────┬─────────────────────────┬────────────────────────────┤
//! │                  │                         │                            │
//! │ Installed List   │   Extension Details     │   Marketplace / Dev        │
//! │ (280px fixed)    │   (flexible)            │   (300px fixed)            │
//! │                  │                         │                            │
//! ├──────────────────┴─────────────────────────┴────────────────────────────┤
//! │ Bottom Tab Panel (Extension Console / API Docs / Change Log)            │
//! └─────────────────────────────────────────────────────────────────────────┘
//!

pub mod extension_box;
pub mod toolbar;
pub mod installed_panel;
pub mod detail_panel;
pub mod marketplace_panel;
pub mod bottom_tabs;

pub use extension_box::ExtensionBox;
pub use toolbar::{ExtensionToolbar, ToolbarTab};
pub use installed_panel::{InstalledPanel, ExtensionItem};
pub use detail_panel::{DetailPanel, DetailTab};
pub use marketplace_panel::{MarketplacePanel, DevTab};
pub use bottom_tabs::{ExtensionBottomTabs, BottomTabKind};
