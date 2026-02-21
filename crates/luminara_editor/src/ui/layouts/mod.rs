pub mod activity_bar;
pub mod dock;
pub mod resizable_panel;
pub mod workspace;

pub use workspace::{
    WorkspaceLayout, MenuBar, Toolbar, BottomPanel,
    MENU_BAR_HEIGHT, TOOLBAR_HEIGHT, BOTTOM_PANEL_HEIGHT,
    LEFT_PANEL_WIDTH, RIGHT_PANEL_WIDTH,
};

pub use dock::{
    DockArea, DockablePanel, DockPanel, DockRoot, DockLayoutBuilder,
    DockPosition, DockState, SplitDirection,
};
