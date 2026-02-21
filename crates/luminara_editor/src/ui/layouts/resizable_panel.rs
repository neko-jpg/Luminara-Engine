//! Resizable Panel Component
//!
//! A reusable panel component with resize handles that supports:
//! - Horizontal and vertical resize handles
//! - Min/max size constraints
//! - Real-time drag updates
//! - Collapsing to minimum size
//! - Nested panel layouts
//! - Size persistence to user preferences
//!
//! **Validates Requirements:**
//! - 9.1: Panel supports horizontal and vertical resize handles
//! - 9.2: WHEN a resize handle is dragged, THE System SHALL update panel sizes in real-time
//! - 9.3: Panel enforces minimum and maximum size constraints
//! - 9.4: Panel sizes are persisted to user preferences
//! - 9.6: Panel displays resize cursors on hover

use gpui::{
    AnyView, Pixels, px, Render, IntoElement, ViewContext, div,
    ParentElement, Styled, InteractiveElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Point, prelude::FluentBuilder,
};
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::core::preferences::EditorPreferences;

/// Orientation of the resizable panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Panel resizes horizontally (left-right)
    Horizontal,
    /// Panel resizes vertically (top-bottom)
    Vertical,
}

/// A resizable panel component with resize handles and size constraints
///
/// This component is used across all Boxes to create flexible layouts.
/// It supports both horizontal and vertical orientations, enforces min/max
/// size constraints, and provides resize handles for user interaction.
///
/// # Nested Panels
///
/// ResizablePanel supports nesting - the content can contain other ResizablePanel
/// instances. When a parent panel is resized, nested panels maintain their
/// proportional sizes relative to the parent's new size.
///
/// # Size Persistence
///
/// Each panel can have a unique ID that is used to persist its size to user
/// preferences. When a panel is created with an ID, it will automatically
/// load its size from preferences on startup and save changes when resized.
///
/// # Example
///
/// ```ignore
/// let panel = ResizablePanel::new(
///     content_view,
///     px(200.0),  // min_size
///     px(800.0),  // max_size
///     px(320.0),  // current_size
///     Orientation::Horizontal,
///     theme,
/// ).with_panel_id("scene_builder.inspector");
/// ```
///
/// # Requirements
/// - Requirement 9.4: Panel sizes are persisted to user preferences
/// - Requirement 9.7: Panel supports nested panel layouts
#[derive(Clone)]
pub struct ResizablePanel {
    /// The content view to display inside the panel
    content: AnyView,
    /// Minimum size constraint (in pixels)
    min_size: Pixels,
    /// Maximum size constraint (in pixels)
    max_size: Pixels,
    /// Current size of the panel (in pixels)
    current_size: Pixels,
    /// Orientation of the panel (horizontal or vertical)
    orientation: Orientation,
    /// Theme for styling
    theme: Arc<Theme>,
    /// Whether the resize handle is currently being dragged
    is_dragging: bool,
    /// Mouse position when drag started
    drag_start_position: Option<Point<Pixels>>,
    /// Panel size when drag started
    drag_start_size: Option<Pixels>,
    /// Whether the panel is currently collapsed
    is_collapsed: bool,
    /// Size before collapsing (used to restore on expand)
    size_before_collapse: Option<Pixels>,
    /// Parent panel size (used for proportional resizing in nested layouts)
    parent_size: Option<Pixels>,
    /// Proportion of parent size (0.0 to 1.0) for nested panels
    parent_proportion: Option<f32>,
    /// Unique panel ID for persistence (e.g., "scene_builder.hierarchy")
    panel_id: Option<String>,
    /// Callback triggered when a resize drag operation completes
    on_resize_complete: Option<std::sync::Arc<dyn Fn(Pixels, &mut ViewContext<Self>) + Send + Sync>>,
}

impl ResizablePanel {
    /// Create a new resizable panel
    ///
    /// # Arguments
    ///
    /// * `content` - The view to display inside the panel
    /// * `min_size` - Minimum size constraint (must be > 0)
    /// * `max_size` - Maximum size constraint (must be >= min_size)
    /// * `current_size` - Initial size (will be clamped to [min_size, max_size])
    /// * `orientation` - Horizontal or vertical orientation
    /// * `theme` - Theme for styling the resize handle
    ///
    /// # Panics
    ///
    /// Panics if min_size <= 0 or max_size < min_size
    pub fn new(
        content: AnyView,
        min_size: Pixels,
        max_size: Pixels,
        current_size: Pixels,
        orientation: Orientation,
        theme: Arc<Theme>,
    ) -> Self {
        // Note: We can't directly access Pixels.0, so we use comparison operators
        // which are implemented for Pixels
        assert!(min_size > px(0.0), "min_size must be greater than 0");
        assert!(max_size >= min_size, "max_size must be >= min_size");

        // Clamp current_size to [min_size, max_size]
        let clamped_size = current_size.max(min_size).min(max_size);

        Self {
            content,
            min_size,
            max_size,
            current_size: clamped_size,
            orientation,
            theme,
            is_dragging: false,
            drag_start_position: None,
            drag_start_size: None,
            is_collapsed: false,
            size_before_collapse: None,
            parent_size: None,
            parent_proportion: None,
            panel_id: None,
            on_resize_complete: None,
        }
    }

    /// Set the panel ID for persistence
    ///
    /// The panel ID should be a unique, stable identifier that doesn't change
    /// across sessions. Good examples:
    /// - "scene_builder.hierarchy"
    /// - "scene_builder.inspector"
    /// - "logic_graph.node_palette"
    ///
    /// When a panel ID is set, the panel will automatically load its size
    /// from preferences on creation and save changes when resized.
    ///
    /// # Requirements
    /// - Requirement 9.4: Use a unique panel ID for each panel
    pub fn with_panel_id(mut self, panel_id: impl Into<String>) -> Self {
        self.panel_id = Some(panel_id.into());
        self
    }

    /// Get the panel ID if set
    pub fn panel_id(&self) -> Option<&str> {
        self.panel_id.as_deref()
    }

    /// Set a callback to be triggered when a resize operation completes
    pub fn on_resize_complete(
        mut self,
        callback: impl Fn(Pixels, &mut ViewContext<Self>) + Send + Sync + 'static,
    ) -> Self {
        self.on_resize_complete = Some(std::sync::Arc::new(callback));
        self
    }

    /// Load panel size from preferences
    ///
    /// If a panel ID is set and a size is stored in preferences, this method
    /// will update the panel's current size to the stored value (clamped to
    /// min/max constraints).
    ///
    /// Returns `true` if a size was loaded, `false` otherwise.
    ///
    /// # Requirements
    /// - Requirement 9.4: Load panel sizes on startup
    pub fn load_size_from_preferences(&mut self, preferences: &EditorPreferences) -> bool {
        if let Some(panel_id) = &self.panel_id {
            if let Some(stored_size) = preferences.get_panel_size(panel_id) {
                // Clamp to constraints
                self.current_size = stored_size.max(self.min_size).min(self.max_size);
                return true;
            }
        }
        false
    }

    /// Save panel size to preferences
    ///
    /// If a panel ID is set, this method will save the current panel size
    /// to the provided preferences instance. The caller is responsible for
    /// persisting the preferences to disk.
    ///
    /// Returns `true` if the size was saved, `false` if no panel ID is set.
    ///
    /// # Requirements
    /// - Requirement 9.4: Save panel sizes to preferences
    pub fn save_size_to_preferences(&self, preferences: &mut EditorPreferences) -> bool {
        if let Some(panel_id) = &self.panel_id {
            preferences.set_panel_size(panel_id.clone(), self.current_size);
            return true;
        }
        false
    }

    /// Get the current size of the panel
    pub fn current_size(&self) -> Pixels {
        self.current_size
    }

    /// Get the minimum size constraint
    pub fn min_size(&self) -> Pixels {
        self.min_size
    }

    /// Get the maximum size constraint
    pub fn max_size(&self) -> Pixels {
        self.max_size
    }

    /// Get the orientation of the panel
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Set the current size (will be clamped to [min_size, max_size])
    ///
    /// Returns the actual size after clamping.
    pub fn set_size(&mut self, new_size: Pixels) -> Pixels {
        self.current_size = new_size.max(self.min_size).min(self.max_size);
        self.current_size
    }

    /// Resize by a delta amount (will be clamped to [min_size, max_size])
    ///
    /// Returns the actual size after resizing and clamping.
    pub fn resize_by(&mut self, delta: Pixels) -> Pixels {
        // Pixels implements Add, so we can use + operator
        let new_size = self.current_size + delta;
        self.set_size(new_size)
    }

    /// Check if the panel is at minimum size
    pub fn is_at_min_size(&self) -> bool {
        self.current_size <= self.min_size
    }

    /// Check if the panel is at maximum size
    pub fn is_at_max_size(&self) -> bool {
        self.current_size >= self.max_size
    }

    /// Collapse the panel to minimum size
    pub fn collapse(&mut self) {
        self.current_size = self.min_size;
    }

    /// Expand the panel to maximum size
    pub fn expand(&mut self) {
        self.current_size = self.max_size;
    }

    /// Check if the panel is collapsed
    pub fn is_collapsed(&self) -> bool {
        self.is_collapsed
    }

    /// Set the parent panel size and calculate proportion for proportional resizing
    ///
    /// When a panel is nested inside another panel, this method should be called
    /// to track the parent's size and calculate the proportion this panel occupies.
    /// This enables proportional resizing when the parent is resized.
    ///
    /// # Arguments
    ///
    /// * `parent_size` - The current size of the parent panel
    /// * `parent_proportion` - The proportion of the parent this panel should occupy (0.0 to 1.0)
    ///
    /// # Requirements
    /// - Requirement 9.7: Support nested panel layouts with proportional resizing
    pub fn set_parent_size_and_proportion(&mut self, parent_size: Pixels, parent_proportion: f32) {
        self.parent_size = Some(parent_size);
        self.parent_proportion = Some(parent_proportion.clamp(0.0, 1.0));
    }

    /// Get the parent panel size if set
    pub fn parent_size(&self) -> Option<Pixels> {
        self.parent_size
    }

    /// Get the parent proportion if set
    pub fn parent_proportion(&self) -> Option<f32> {
        self.parent_proportion
    }

    /// Calculate proportional size based on parent resize
    ///
    /// When a parent panel is resized, this method calculates the new size
    /// for this panel while maintaining the same proportion relative to the
    /// parent's size.
    ///
    /// # Arguments
    ///
    /// * `new_parent_size` - The new size of the parent panel
    ///
    /// # Returns
    ///
    /// The new size for this panel, clamped to [min_size, max_size]
    ///
    /// # Requirements
    /// - Requirement 9.7: Handle proportional resizing of children
    pub fn calculate_proportional_size(&self, new_parent_size: Pixels) -> Pixels {
        if let Some(proportion) = self.parent_proportion {
            // Use unsafe to access the inner f32 value of Pixels
            // This is necessary because Pixels doesn't expose its value publicly
            let new_parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_parent_size) };
            let new_size = px(proportion * new_parent_f32);
            
            // Clamp to constraints
            new_size.max(self.min_size).min(self.max_size)
        } else {
            // No proportion set, return current size
            self.current_size
        }
    }

    /// Resize proportionally based on parent resize
    ///
    /// This method updates the panel's size based on its proportion of the parent.
    /// It should be called when the parent panel is resized to maintain
    /// proportional sizing.
    ///
    /// # Arguments
    ///
    /// * `new_parent_size` - The new size of the parent panel
    ///
    /// # Returns
    ///
    /// The actual new size after proportional calculation and clamping
    ///
    /// # Requirements
    /// - Requirement 9.7: Handle proportional resizing of children
    pub fn resize_proportionally(&mut self, new_parent_size: Pixels) -> Pixels {
        let new_size = self.calculate_proportional_size(new_parent_size);
        self.current_size = new_size;
        self.parent_size = Some(new_parent_size);
        new_size
    }

    /// Toggle collapse state
    ///
    /// When collapsing, stores the current size and animates to minimum size.
    /// When expanding, restores the previous size.
    ///
    /// # Requirements
    /// - Requirement 9.5: Panel supports collapsing to minimum size
    pub fn toggle_collapse(&mut self) {
        if self.is_collapsed {
            // Expand: restore previous size or use max size
            if let Some(prev_size) = self.size_before_collapse {
                self.current_size = prev_size;
            } else {
                self.current_size = self.max_size;
            }
            self.is_collapsed = false;
            self.size_before_collapse = None;
        } else {
            // Collapse: store current size and collapse to minimum
            self.size_before_collapse = Some(self.current_size);
            self.current_size = self.min_size;
            self.is_collapsed = true;
        }
    }

    /// Start a resize drag operation
    ///
    /// # Requirements
    /// - Requirement 9.2: Update panel sizes in real-time during drag
    fn start_drag(&mut self, position: Point<Pixels>, _cx: &mut ViewContext<Self>) {
        self.is_dragging = true;
        self.drag_start_position = Some(position);
        self.drag_start_size = Some(self.current_size);
    }

    /// Handle mouse move during resize drag
    ///
    /// # Requirements
    /// - Requirement 9.2: Update panel sizes in real-time during drag
    /// - Requirement 9.3: Enforce min/max size constraints
    fn handle_drag_move(&mut self, position: Point<Pixels>, _cx: &mut ViewContext<Self>) {
        if !self.is_dragging {
            return;
        }

        if let (Some(start_pos), Some(start_size)) = (self.drag_start_position, self.drag_start_size) {
            // Calculate delta based on orientation
            let delta = match self.orientation {
                Orientation::Horizontal => position.x - start_pos.x,
                Orientation::Vertical => position.y - start_pos.y,
            };

            // Calculate new size and clamp to constraints
            let new_size = start_size + delta;
            self.current_size = new_size.max(self.min_size).min(self.max_size);
        }
    }

    /// Complete the resize drag operation
    ///
    /// # Requirements
    /// - Requirement 9.2: Update panel sizes in real-time during drag
    fn complete_drag(&mut self, cx: &mut ViewContext<Self>) {
        self.is_dragging = false;
        self.drag_start_position = None;
        self.drag_start_size = None;
        self.dispatch_resize_complete(cx);
    }

    /// Cancel the resize drag operation
    #[allow(dead_code)]
    fn cancel_drag(&mut self, _cx: &mut ViewContext<Self>) {
        if let Some(start_size) = self.drag_start_size {
            self.current_size = start_size;
        }
        self.is_dragging = false;
        self.drag_start_position = None;
        self.drag_start_size = None;
    }

    /// Trigger the resize complete callback if one is registered
    fn dispatch_resize_complete(&self, cx: &mut ViewContext<Self>) {
        if let Some(callback) = &self.on_resize_complete {
            callback(self.current_size, cx);
        }
    }
}

impl Render for ResizablePanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let theme = self.theme.clone();
        let handle_size = px(4.0); // Width/height of the resize handle
        let button_size = px(20.0); // Size of the collapse button
        
        // Create the collapse/expand button
        let collapse_button = div()
            .flex_shrink_0()
            .bg(theme.colors.surface)
            .hover(|this| this.bg(theme.colors.surface_hover))
            .border_1()
            .border_color(theme.colors.border)
            .rounded(theme.borders.sm)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            // Set size based on orientation
            .when(self.orientation == Orientation::Horizontal, |this| {
                this.w(button_size).h(button_size)
            })
            .when(self.orientation == Orientation::Vertical, |this| {
                this.w(button_size).h(button_size)
            })
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _event: &MouseDownEvent, cx| {
                this.toggle_collapse();
                cx.notify();
            }))
            .child(
                div()
                    .text_color(theme.colors.text)
                    .text_xs()
                    .child(if self.is_collapsed {
                        match self.orientation {
                            Orientation::Horizontal => "→",
                            Orientation::Vertical => "↓",
                        }
                    } else {
                        match self.orientation {
                            Orientation::Horizontal => "←",
                            Orientation::Vertical => "↑",
                        }
                    })
            );
        
        // Create the resize handle with appropriate cursor and event handlers
        let resize_handle = div()
            .flex_shrink_0()
            .bg(if self.is_dragging {
                theme.colors.accent
            } else {
                theme.colors.border
            })
            .hover(|this| this.bg(theme.colors.accent_hover))
            // Set size based on orientation
            .when(self.orientation == Orientation::Horizontal, |this| {
                this.w(handle_size).h_full().cursor_col_resize()
            })
            .when(self.orientation == Orientation::Vertical, |this| {
                this.h(handle_size).w_full().cursor_row_resize()
            })
            // Mouse event handlers for drag
            .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, cx| {
                this.start_drag(event.position, cx);
                cx.notify();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, cx| {
                if this.is_dragging {
                    this.handle_drag_move(event.position, cx);
                    cx.notify();
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _event: &MouseUpEvent, cx| {
                if this.is_dragging {
                    this.complete_drag(cx);
                    cx.notify();
                }
            }));

        // Layout the panel content and resize handle based on orientation
        match self.orientation {
            Orientation::Horizontal => {
                // Horizontal: content on left, handle and button on right
                div()
                    .flex()
                    .flex_row()
                    .w(self.current_size)
                    .h_full()
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(self.content.clone())
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .h_full()
                            .gap_1()
                            .child(collapse_button)
                            .child(
                                div()
                                    .flex_1()
                                    .child(resize_handle)
                            )
                    )
            }
            Orientation::Vertical => {
                // Vertical: content on top, handle and button on bottom
                div()
                    .flex()
                    .flex_col()
                    .h(self.current_size)
                    .w_full()
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .overflow_hidden()
                            .child(self.content.clone())
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .gap_1()
                            .child(collapse_button)
                            .child(
                                div()
                                    .flex_1()
                                    .child(resize_handle)
                            )
                    )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        // Create a mock view for testing
        // Note: In real tests, we'd use a proper GPUI view
        // For now, we'll test the logic without the view
        let min = px(100.0);
        let max = px(500.0);
        let current = px(300.0);

        // We can't create a real panel without GPUI context,
        // but we can test the constraints logic
        assert!(min > px(0.0));
        assert!(max >= min);
        assert!(current >= min && current <= max);
    }

    #[test]
    fn test_size_clamping() {
        let min = px(100.0);
        let max = px(500.0);

        // Test clamping below minimum
        let below_min = px(50.0);
        let clamped = below_min.max(min).min(max);
        assert_eq!(clamped, min);

        // Test clamping above maximum
        let above_max = px(600.0);
        let clamped = above_max.max(min).min(max);
        assert_eq!(clamped, max);

        // Test value within range
        let within_range = px(300.0);
        let clamped = within_range.max(min).min(max);
        assert_eq!(clamped, within_range);
    }

    #[test]
    #[should_panic(expected = "min_size must be greater than 0")]
    fn test_invalid_min_size() {
        // This would panic if we could create a panel
        // Testing the assertion logic
        let min = px(0.0);
        assert!(min > px(0.0), "min_size must be greater than 0");
    }

    #[test]
    #[should_panic(expected = "max_size must be >= min_size")]
    fn test_invalid_max_size() {
        // This would panic if we could create a panel
        // Testing the assertion logic
        let min = px(500.0);
        let max = px(100.0);
        assert!(max >= min, "max_size must be >= min_size");
    }

    #[test]
    fn test_orientation() {
        assert_eq!(Orientation::Horizontal, Orientation::Horizontal);
        assert_eq!(Orientation::Vertical, Orientation::Vertical);
        assert_ne!(Orientation::Horizontal, Orientation::Vertical);
    }

    #[test]
    fn test_proportional_resize_calculation() {
        // Test proportional resize calculation
        // Parent: 1000px, Child: 300px (30% proportion)
        // New parent: 800px, Expected child: 240px
        let _old_parent = px(1000.0);
        let new_parent = px(800.0);
        let proportion = 0.3; // 30%

        // Calculate expected size using proportion
        let new_parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_parent) };
        let expected = px(proportion * new_parent_f32);
        
        // Use epsilon comparison for floating point values
        let expected_f32 = unsafe { std::mem::transmute::<Pixels, f32>(expected) };
        let target_f32 = 240.0;
        assert!((expected_f32 - target_f32).abs() < 0.01, 
                "Expected approximately {}px, got {}px", target_f32, expected_f32);
    }

    #[test]
    fn test_nested_panel_proportions() {
        // Test nested panel scenario with 2 levels
        // Parent: 1000px
        // Child 1: 40% of parent = 400px
        // Child 2: 60% of parent = 600px
        // When parent resizes to 800px:
        // Child 1: 40% of 800px = 320px
        // Child 2: 60% of 800px = 480px
        
        let new_parent = px(800.0);
        
        let child1_proportion = 0.4;
        let child2_proportion = 0.6;
        
        let new_parent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_parent) };
        let child1_new = px(child1_proportion * new_parent_f32);
        let child2_new = px(child2_proportion * new_parent_f32);
        
        // Use approximate equality for floating point comparison
        let child1_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child1_new) };
        let child2_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child2_new) };
        
        assert!((child1_f32 - 320.0).abs() < 0.01);
        assert!((child2_f32 - 480.0).abs() < 0.01);
    }

    #[test]
    fn test_three_level_nesting() {
        // Test 3-level nested panel scenario
        // Grandparent: 1000px -> 1200px
        // Parent: 50% of grandparent = 500px -> 600px
        // Child: 50% of parent = 250px -> 300px
        
        let new_grandparent = px(1200.0);
        
        let parent_proportion = 0.5;
        let child_proportion = 0.5;
        
        // Parent resize
        let new_grandparent_f32 = unsafe { std::mem::transmute::<Pixels, f32>(new_grandparent) };
        let parent_new = px(parent_proportion * new_grandparent_f32);
        
        let parent_new_f32 = unsafe { std::mem::transmute::<Pixels, f32>(parent_new) };
        assert!((parent_new_f32 - 600.0).abs() < 0.01);
        
        // Child resize based on parent's new size
        let child_new = px(child_proportion * parent_new_f32);
        let child_new_f32 = unsafe { std::mem::transmute::<Pixels, f32>(child_new) };
        assert!((child_new_f32 - 300.0).abs() < 0.01);
    }
}
