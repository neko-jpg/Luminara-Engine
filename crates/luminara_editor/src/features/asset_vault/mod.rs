//! Asset Vault Box Module
//!
//! The Asset Vault provides a comprehensive asset management interface with:
//! - Directory tree navigation (left panel)
//! - Asset grid with thumbnails (center panel)
//! - Asset inspector with metadata (right panel)
//! - Import log and diagnostics (bottom panel)
//!
//! Layout structure matching HTML prototype:
//! ```
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │ Menu Bar (File, Edit, Assets, View, Help)                               │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │ Toolbar (Grid/List view, Sort, Filter, Status)                          │
//! ├──────────────────┬─────────────────────────┬────────────────────────────┤
//! │                  │                         │                            │
//! │ Directory Tree   │   Asset Grid + Preview  │   Asset Inspector          │
//! │ (260px fixed)    │   (flexible)            │   (320px fixed)            │
//! │                  │                         │                            │
//! ├──────────────────┴─────────────────────────┴────────────────────────────┤
//! │ Bottom Tab Panel (Import Log / Missing Assets / Duplicates)             │
//! └─────────────────────────────────────────────────────────────────────────┘
//!

pub mod asset_vault_box;
pub mod directory_tree;
pub mod asset_grid;
pub mod inspector;
pub mod bottom_tabs;
pub mod toolbar;

pub use asset_vault_box::AssetVaultBox;
pub use directory_tree::{DirectoryTree, TreeItem, TreeItemType};
pub use asset_grid::{AssetGrid, AssetItem, AssetType};
pub use inspector::{AssetInspector, AssetMetadata, AssetProperty};
pub use bottom_tabs::{AssetVaultBottomTabs, TabKind};
pub use toolbar::{AssetVaultToolbar, ViewMode, SortMode};
