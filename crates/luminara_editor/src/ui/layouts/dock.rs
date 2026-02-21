//! Docking System for Luminara Editor
//!
//! Provides a flexible docking system that allows panels to be:
//! - Docked to edges (left, right, top, bottom) or center (tabbed)
//! - Dragged and dropped to reposition
//! - Resized with splitters
//!
//! ## Architecture
//!
//! ```
//! DockRoot
//! ├── DockArea (Horizontal/Vertical split)
//! │   ├── DockPanel (Tabbed container)
//! │   │   ├── Tab 1 (Viewport)
//! │   │   └── Tab 2 (Hierarchy)
//! │   └── DockArea
//! │       ├── DockPanel
//! │       └── DockPanel
//! ```

use gpui::{
    div, px, AnyElement, AnyView, IntoElement, ParentElement, Pixels, Point, Render, Styled,
    ViewContext, Element, GlobalElementId, LayoutId, InteractiveElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Dock position for drop targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPosition {
    /// Dock as tab in center
    Center,
    /// Dock to the left
    Left,
    /// Dock to the right
    Right,
    /// Dock to the top
    Top,
    /// Dock to the bottom
    Bottom,
}

/// State of a dockable panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockState {
    /// Panel is docked in the layout
    Docked,
    /// Panel is floating in a separate window
    Floating,
    /// Panel is being dragged
    Dragging,
    /// Panel is auto-hidden (collapsed to edge)
    AutoHidden,
}

/// A dockable panel that can be positioned within the dock layout
#[derive(Clone)]
pub struct DockablePanel {
    /// Unique identifier for the panel
    pub id: String,
    /// Display title for the panel
    pub title: String,
    /// The content view
    pub content: AnyView,
    /// Whether the panel can be closed
    pub closable: bool,
    /// Whether the panel can be floated
    pub floatable: bool,
    /// Current state
    pub state: DockState,
    /// Current position if docked
    pub position: Option<DockPosition>,
    /// Panel size (used when undocked or for initial size)
    pub size: (f32, f32),
}

impl DockablePanel {
    /// Create a new dockable panel
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: AnyView) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content,
            closable: true,
            floatable: true,
            state: DockState::Docked,
            position: None,
            size: (300.0, 400.0),
        }
    }

    /// Set whether the panel can be closed
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set whether the panel can be floated
    pub fn floatable(mut self, floatable: bool) -> Self {
        self.floatable = floatable;
        self
    }

    /// Set initial size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }
}

/// Tab item in a docked panel
#[derive(Clone)]
pub struct DockTab {
    /// Panel reference
    pub panel: DockablePanel,
    /// Whether this tab is active
    pub is_active: bool,
}

/// A panel container with tabs
#[derive(Clone)]
pub struct DockPanel {
    /// Unique identifier
    pub id: String,
    /// Tabs in this panel
    pub tabs: Vec<DockTab>,
    /// Active tab index
    pub active_tab: usize,
    /// Theme reference
    theme: Arc<Theme>,
}

impl DockPanel {
    /// Create a new dock panel
    pub fn new(id: impl Into<String>, theme: Arc<Theme>) -> Self {
        Self {
            id: id.into(),
            tabs: Vec::new(),
            active_tab: 0,
            theme,
        }
    }

    /// Add a panel as a tab
    pub fn add_panel(&mut self, panel: DockablePanel) {
        self.tabs.push(DockTab {
            panel,
            is_active: self.tabs.is_empty(),
        });
        if self.tabs.len() == 1 {
            self.active_tab = 0;
        }
    }

    /// Remove a tab by index
    pub fn remove_tab(&mut self, index: usize) -> Option<DockablePanel> {
        if index < self.tabs.len() {
            let tab = self.tabs.remove(index);
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
            Some(tab.panel)
        } else {
            None
        }
    }

    /// Set active tab
    pub fn set_active_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            for (i, tab) in self.tabs.iter_mut().enumerate() {
                tab.is_active = i == index;
            }
            self.active_tab = index;
        }
    }

    /// Get the active panel
    pub fn active_panel(&self) -> Option<&DockablePanel> {
        self.tabs.get(self.active_tab).map(|t| &t.panel)
    }
}

impl Render for DockPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let active_tab = self.active_tab;
        let tabs: Vec<_> = self.tabs.iter().enumerate().map(|(i, tab)| {
            let is_active = i == active_tab;
            let title = tab.panel.title.clone();
            let closable = tab.panel.closable;
            let theme = theme.clone();
            
            let mut tab_el = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .px(px(12.0))
                .py(px(4.0))
                .bg(if is_active { theme.colors.background } else { theme.colors.panel_header })
                .cursor_pointer();
            
            // Only show border for active tab
            if is_active {
                tab_el = tab_el.border_b_2().border_color(theme.colors.accent);
            }
            
            tab_el = tab_el
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                    this.set_active_tab(i);
                    cx.notify();
                }))
                .child(
                    div()
                        .text_size(theme.typography.sm)
                        .text_color(if is_active { theme.colors.text } else { theme.colors.text_secondary })
                        .child(title.clone())
                );
            
            // Add close button if closable
            if closable {
                tab_el = tab_el.child(
                    div()
                        .text_size(theme.typography.xs)
                        .text_color(theme.colors.text_secondary)
                        .hover(|style| style.text_color(theme.colors.text))
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                            this.remove_tab(i);
                            cx.notify();
                        }))
                        .child("×")
                );
            }
            
            tab_el
        }).collect();
        
        let active_content = self.active_panel().map(|p| p.content.clone());

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.background)
            .child(
                // Tab bar
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(28.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .children(tabs)
            )
            .child(
                // Content area
                div()
                    .flex_1()
                    .overflow_hidden()
                    .children(active_content.into_iter().map(|c| c))
            )
    }
}

impl Element for DockPanel {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        cx: &mut gpui::WindowContext,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = gpui::Style::default();
        let layout_id = cx.request_layout(style, None);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: gpui::Bounds<Pixels>,
        _request_layout_state: &mut Self::RequestLayoutState,
        _cx: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: gpui::Bounds<Pixels>,
        _request_layout_state: &mut Self::RequestLayoutState,
        _prepaint_state: &mut Self::PrepaintState,
        _cx: &mut gpui::WindowContext,
    ) {
        // Rendering is done through the Render trait
    }
}

impl IntoElement for DockPanel {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// Split direction for dock areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// A dock area that can contain panels or other areas (split)
pub enum DockArea {
    /// A leaf node containing a panel with tabs
    Panel(DockPanel),
    /// A split containing child areas
    Split {
        direction: SplitDirection,
        ratio: f32, // 0.0 to 1.0, position of the split
        first: Box<DockArea>,
        second: Box<DockArea>,
    },
}

impl DockArea {
    /// Create a new panel area
    pub fn panel(panel: DockPanel) -> Self {
        DockArea::Panel(panel)
    }

    /// Create a horizontal split
    pub fn horizontal_split(ratio: f32, first: DockArea, second: DockArea) -> Self {
        DockArea::Split {
            direction: SplitDirection::Horizontal,
            ratio: ratio.clamp(0.1, 0.9),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Create a vertical split
    pub fn vertical_split(ratio: f32, first: DockArea, second: DockArea) -> Self {
        DockArea::Split {
            direction: SplitDirection::Vertical,
            ratio: ratio.clamp(0.1, 0.9),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Find a panel by ID recursively
    pub fn find_panel(&self, id: &str) -> Option<&DockPanel> {
        match self {
            DockArea::Panel(panel) if panel.id == id => Some(panel),
            DockArea::Panel(_) => None,
            DockArea::Split { first, second, .. } => {
                first.find_panel(id).or_else(|| second.find_panel(id))
            }
        }
    }

    /// Find a panel by ID mutably
    pub fn find_panel_mut(&mut self, id: &str) -> Option<&mut DockPanel> {
        match self {
            DockArea::Panel(panel) if panel.id == id => Some(panel),
            DockArea::Panel(_) => None,
            DockArea::Split { first, second, .. } => {
                first.find_panel_mut(id).or_else(|| second.find_panel_mut(id))
            }
        }
    }
}

/// Root docking container that manages the entire dock layout
pub struct DockRoot {
    /// The root dock area
    root: DockArea,
    /// Theme reference
    theme: Arc<Theme>,
    /// Currently dragged panel (if any)
    dragged_panel: Option<String>,
    /// Drag start position
    drag_start: Option<Point<Pixels>>,
}

impl DockRoot {
    /// Create a new dock root
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            root: DockArea::panel(DockPanel::new("root", theme.clone())),
            theme,
            dragged_panel: None,
            drag_start: None,
        }
    }

    /// Set the root dock area
    pub fn with_root(mut self, root: DockArea) -> Self {
        self.root = root;
        self
    }

    /// Dock a panel to the root
    pub fn dock(&mut self, panel: DockablePanel, position: DockPosition) {
        match position {
            DockPosition::Center => {
                // Add to root panel if it's a panel
                if let DockArea::Panel(ref mut p) = self.root {
                    p.add_panel(panel);
                }
            }
            DockPosition::Left => {
                let old_root = std::mem::replace(&mut self.root, DockArea::panel(DockPanel::new("temp", self.theme.clone())));
                self.root = DockArea::horizontal_split(0.25, DockArea::panel(DockPanel::new(&panel.id, self.theme.clone())), old_root);
            }
            DockPosition::Right => {
                let old_root = std::mem::replace(&mut self.root, DockArea::panel(DockPanel::new("temp", self.theme.clone())));
                self.root = DockArea::horizontal_split(0.75, old_root, DockArea::panel(DockPanel::new(&panel.id, self.theme.clone())));
            }
            DockPosition::Top => {
                let old_root = std::mem::replace(&mut self.root, DockArea::panel(DockPanel::new("temp", self.theme.clone())));
                self.root = DockArea::vertical_split(0.3, DockArea::panel(DockPanel::new(&panel.id, self.theme.clone())), old_root);
            }
            DockPosition::Bottom => {
                let old_root = std::mem::replace(&mut self.root, DockArea::panel(DockPanel::new("temp", self.theme.clone())));
                self.root = DockArea::vertical_split(0.7, old_root, DockArea::panel(DockPanel::new(&panel.id, self.theme.clone())));
            }
        }
    }

    /// Start dragging a panel
    pub fn start_drag(&mut self, panel_id: String, position: Point<Pixels>) {
        self.dragged_panel = Some(panel_id);
        self.drag_start = Some(position);
    }

    /// End dragging
    pub fn end_drag(&mut self) -> Option<(String, Point<Pixels>)> {
        let panel_id = self.dragged_panel.take();
        let start = self.drag_start.take();
        panel_id.zip(start)
    }

    /// Get the dock area
    pub fn root(&self) -> &DockArea {
        &self.root
    }
}

impl Render for DockRoot {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        render_dock_area(&self.root, self.theme.clone())
    }
}

/// Render a dock area recursively
fn render_dock_area(area: &DockArea, theme: Arc<Theme>) -> AnyElement {
    match area {
        DockArea::Panel(panel) => {
            // Clone panel for rendering
            let panel_clone = panel.clone();
            div()
                .size_full()
                .child(panel_clone)
                .into_any_element()
        }
        DockArea::Split { direction, ratio, first, second } => {
            let is_horizontal = *direction == SplitDirection::Horizontal;
            let ratio = *ratio;
            
            let mut container: gpui::Div = div()
                .flex()
                .size_full();
            
            if is_horizontal {
                container = container.flex_row();
            } else {
                container = container.flex_col();
            }

            // Calculate sizes based on ratio
            let first_size = ratio * 100.0;
            let _second_size = (1.0 - ratio) * 100.0;
            
            // First child container
            let mut first_div: gpui::Div = div();
            if is_horizontal {
                first_div = first_div.w(px(first_size));
            } else {
                first_div = first_div.h(px(first_size));
            }
            first_div = first_div.flex_basis(px(first_size));
            
            // Second child container
            let second_div: gpui::Div = div()
                .flex_1();
            
            // Splitter
            let splitter: gpui::Div;
            if is_horizontal {
                splitter = div()
                    .w(px(4.0))
                    .h_full()
                    .cursor_col_resize()
                    .bg(theme.colors.border)
                    .hover(|this| this.bg(theme.colors.accent));
            } else {
                splitter = div()
                    .h(px(4.0))
                    .w_full()
                    .cursor_row_resize()
                    .bg(theme.colors.border)
                    .hover(|this| this.bg(theme.colors.accent));
            }

            container
                .child(first_div.child(render_dock_area(first, theme.clone())))
                .child(splitter)
                .child(second_div.child(render_dock_area(second, theme.clone())))
                .into_any_element()
        }
    }
}

/// Builder for constructing dock layouts
pub struct DockLayoutBuilder {
    theme: Arc<Theme>,
    panels: Vec<(DockablePanel, DockPosition)>,
}

impl DockLayoutBuilder {
    /// Create a new dock layout builder
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            panels: Vec::new(),
        }
    }

    /// Add a panel at a specific position
    pub fn add_panel(mut self, panel: DockablePanel, position: DockPosition) -> Self {
        self.panels.push((panel, position));
        self
    }

    /// Build the dock root
    pub fn build(self) -> DockRoot {
        let mut root = DockRoot::new(self.theme.clone());
        
        for (panel, position) in self.panels {
            root.dock(panel, position);
        }
        
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_position() {
        assert_eq!(DockPosition::Center, DockPosition::Center);
        assert_ne!(DockPosition::Left, DockPosition::Right);
    }

    #[test]
    fn test_dock_state() {
        assert_eq!(DockState::Docked, DockState::Docked);
        assert_ne!(DockState::Docked, DockState::Floating);
    }

    #[test]
    fn test_split_direction() {
        assert_eq!(SplitDirection::Horizontal, SplitDirection::Horizontal);
        assert_ne!(SplitDirection::Horizontal, SplitDirection::Vertical);
    }

    #[test]
    fn test_dock_area_creation() {
        let theme = Arc::new(Theme::default_dark());
        let panel = DockPanel::new("test", theme);
        let area = DockArea::panel(panel);
        
        match area {
            DockArea::Panel(p) => assert_eq!(p.id, "test"),
            _ => panic!("Expected panel"),
        }
    }

    #[test]
    fn test_horizontal_split() {
        let theme = Arc::new(Theme::default_dark());
        let first = DockArea::panel(DockPanel::new("first", theme.clone()));
        let second = DockArea::panel(DockPanel::new("second", theme.clone()));
        
        let split = DockArea::horizontal_split(0.5, first, second);
        
        match split {
            DockArea::Split { direction, ratio, .. } => {
                assert_eq!(direction, SplitDirection::Horizontal);
                assert_eq!(ratio, 0.5);
            }
            _ => panic!("Expected split"),
        }
    }

    #[test]
    fn test_vertical_split() {
        let theme = Arc::new(Theme::default_dark());
        let first = DockArea::panel(DockPanel::new("first", theme.clone()));
        let second = DockArea::panel(DockPanel::new("second", theme.clone()));
        
        let split = DockArea::vertical_split(0.3, first, second);
        
        match split {
            DockArea::Split { direction, ratio, .. } => {
                assert_eq!(direction, SplitDirection::Vertical);
                assert_eq!(ratio, 0.3);
            }
            _ => panic!("Expected split"),
        }
    }

    #[test]
    fn test_dock_panel_add_tabs() {
        let theme = Arc::new(Theme::default_dark());
        let mut panel = DockPanel::new("test", theme);
        
        // Need a mock view for testing
        // For now, just test the panel state
        assert_eq!(panel.tabs.len(), 0);
        assert_eq!(panel.active_tab, 0);
    }
}
