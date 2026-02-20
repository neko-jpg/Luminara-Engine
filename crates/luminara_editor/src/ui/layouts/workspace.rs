//! Workspace Layout Framework
//!
//! Provides a unified layout structure for all editor feature panels.
//! This eliminates duplicate layout code across SceneBuilderBox, LogicGraphBox,
//! DirectorBox, AssetVaultBox, etc.
//!
//! Standard Layout Structure:
//! ```
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Menu Bar (optional, 32px)                                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Toolbar (optional, 44px)                                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │   Main Content Area                                         │
//! │   (Left Panel | Center Panel | Right Panel)                 │
//! │                                                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Bottom Panel (optional, 200px)                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use gpui::{
    div, px, IntoElement, ParentElement, Styled, AnyElement, InteractiveElement,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Standard dimensions for workspace layout
pub const MENU_BAR_HEIGHT: f32 = 32.0;
pub const TOOLBAR_HEIGHT: f32 = 44.0;
pub const BOTTOM_PANEL_HEIGHT: f32 = 200.0;
pub const LEFT_PANEL_WIDTH: f32 = 260.0;
pub const RIGHT_PANEL_WIDTH: f32 = 320.0;

/// Builder for constructing a workspace layout
pub struct WorkspaceLayout {
    theme: Arc<Theme>,
    menu_bar: Option<AnyElement>,
    toolbar: Option<AnyElement>,
    left_panel: Option<AnyElement>,
    center_panel: Option<AnyElement>,
    right_panel: Option<AnyElement>,
    bottom_panel: Option<AnyElement>,
    gap: f32,
}

impl WorkspaceLayout {
    /// Create a new workspace layout builder with the given theme
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            menu_bar: None,
            toolbar: None,
            left_panel: None,
            center_panel: None,
            right_panel: None,
            bottom_panel: None,
            gap: 4.0,
        }
    }

    /// Set the menu bar element (top, 32px height)
    pub fn menu_bar(mut self, element: impl IntoElement) -> Self {
        self.menu_bar = Some(element.into_any_element());
        self
    }

    /// Set the toolbar element (below menu, 44px height)
    pub fn toolbar(mut self, element: impl IntoElement) -> Self {
        self.toolbar = Some(element.into_any_element());
        self
    }

    /// Set the left panel element (fixed width, 260px)
    pub fn left_panel(mut self, element: impl IntoElement) -> Self {
        self.left_panel = Some(element.into_any_element());
        self
    }

    /// Set the center panel element (flexible width)
    pub fn center_panel(mut self, element: impl IntoElement) -> Self {
        self.center_panel = Some(element.into_any_element());
        self
    }

    /// Set the right panel element (fixed width, 320px)
    pub fn right_panel(mut self, element: impl IntoElement) -> Self {
        self.right_panel = Some(element.into_any_element());
        self
    }

    /// Set the bottom panel element (fixed height, 200px)
    pub fn bottom_panel(mut self, element: impl IntoElement) -> Self {
        self.bottom_panel = Some(element.into_any_element());
        self
    }

    /// Set the gap between panels (default: 4px)
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Build the layout into an element
    pub fn build(self) -> impl IntoElement {
        let theme = self.theme;
        let gap = px(self.gap);

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.background);

        // Menu bar
        if let Some(menu_bar) = self.menu_bar {
            root = root.child(menu_bar);
        }

        // Toolbar
        if let Some(toolbar) = self.toolbar {
            root = root.child(toolbar);
        }

        // Main content area
        let mut main_area = div()
            .flex()
            .flex_row()
            .flex_1()
            .w_full()
            .overflow_hidden();

        if self.gap > 0.0 {
            main_area = main_area.gap(gap);
        }

        // Left panel
        if let Some(left) = self.left_panel {
            main_area = main_area.child(
                div()
                    .w(px(LEFT_PANEL_WIDTH))
                    .h_full()
                    .child(left)
            );
        }

        // Center panel (flexible)
        if let Some(center) = self.center_panel {
            main_area = main_area.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(center)
            );
        }

        // Right panel
        if let Some(right) = self.right_panel {
            main_area = main_area.child(
                div()
                    .w(px(RIGHT_PANEL_WIDTH))
                    .h_full()
                    .child(right)
            );
        }

        root = root.child(main_area);

        // Bottom panel
        if let Some(bottom) = self.bottom_panel {
            root = root.child(
                div()
                    .h(px(BOTTOM_PANEL_HEIGHT))
                    .min_h(px(150.0))
                    .max_h(px(400.0))
                    .w_full()
                    .child(bottom)
            );
        }

        root
    }
}

impl IntoElement for WorkspaceLayout {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.build().into_any_element()
    }
}

/// Standard menu bar component with consistent styling
pub struct MenuBar {
    theme: Arc<Theme>,
    items: Vec<String>,
}

impl MenuBar {
    /// Create a new menu bar with the given theme
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            items: Vec::new(),
        }
    }

    /// Set the menu items
    pub fn items(mut self, items: Vec<&str>) -> Self {
        self.items = items.into_iter().map(|s| s.to_string()).collect();
        self
    }
}

impl IntoElement for MenuBar {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let theme = self.theme;

        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(MENU_BAR_HEIGHT))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            .items_center()
            .px(theme.spacing.md)
            .gap(theme.spacing.lg)
            .children(
                self.items.into_iter().map(move |label| {
                    let theme = theme.clone();
                    
                    div()
                        .px(theme.spacing.md)
                        .py(theme.spacing.xs)
                        .rounded(theme.borders.xs)
                        .hover(|this| this.bg(theme.colors.surface_hover))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_color(theme.colors.text)
                                .text_size(theme.typography.md)
                                .child(label)
                        )
                })
            )
            .into_any_element()
    }
}

/// Standard toolbar container with consistent styling
pub struct Toolbar {
    theme: Arc<Theme>,
    content: AnyElement,
    height: f32,
}

impl Toolbar {
    /// Create a new toolbar with the given theme and content
    pub fn new(theme: Arc<Theme>, content: impl IntoElement) -> Self {
        Self {
            theme,
            content: content.into_any_element(),
            height: TOOLBAR_HEIGHT,
        }
    }

    /// Set a custom height (default: 44px)
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl IntoElement for Toolbar {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let theme = self.theme;

        div()
            .flex()
            .items_center()
            .w_full()
            .h(px(self.height))
            .px(theme.spacing.sm)
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(self.content)
            .into_any_element()
    }
}

/// Bottom panel container with consistent styling
pub struct BottomPanel {
    theme: Arc<Theme>,
    content: AnyElement,
    height: f32,
}

impl BottomPanel {
    /// Create a new bottom panel with the given theme and content
    pub fn new(theme: Arc<Theme>, content: impl IntoElement) -> Self {
        Self {
            theme,
            content: content.into_any_element(),
            height: BOTTOM_PANEL_HEIGHT,
        }
    }

    /// Set a custom height (default: 200px)
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl IntoElement for BottomPanel {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let theme = self.theme;

        div()
            .h(px(self.height))
            .min_h(px(150.0))
            .max_h(px(400.0))
            .w_full()
            .bg(theme.colors.background)
            .border_t_1()
            .border_color(theme.colors.border)
            .child(self.content)
            .into_any_element()
    }
}
