use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext,
    ElementId, AnyElement, ClickEvent, WindowContext, View, prelude::*, Overflow,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

pub struct Dropdown {
    id: ElementId,
    options: Vec<String>,
    selected_index: Option<usize>,
    is_open: bool,
    on_change: Option<Arc<dyn Fn(&str, usize, &mut WindowContext) + Send + Sync>>,
}

impl Dropdown {
    pub fn new(id: impl Into<ElementId>, options: Vec<String>, selected_index: Option<usize>) -> Self {
        Self {
            id: id.into(),
            options,
            selected_index,
            is_open: false,
            on_change: None,
        }
    }

    pub fn on_change(mut self, handler: impl Fn(&str, usize, &mut WindowContext) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn set_open(&mut self, is_open: bool) {
        self.is_open = is_open;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn set_selected(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = Some(index);
            self.is_open = false;
        }
    }
}

// Note: To have internal state like `is_open`, Dropdown typically needs to be part of a View
// or manage state via the parent View's update cycle.
// However, since we want a reusable component, we'll design it to be updated by the parent view
// for the `is_open` state, OR we can make `Dropdown` a View itself if it needs complex internal state.
//
// For simplicity in this architecture, we will implement `Render` for `View<Dropdown>`
// so it can handle its own state updates (open/close).

impl Render for Dropdown {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = Theme::default_dark();
        let is_open = self.is_open;
        let selected_text = self.selected_index
            .and_then(|i| self.options.get(i))
            .cloned()
            .unwrap_or_else(|| "Select...".to_string());

        let view = cx.view().clone();

        div()
            .id(self.id.clone())
            .relative() // For absolute positioning of the menu
            .w_full()
            // Main button
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .h(px(32.0))
                    .px(theme.spacing.sm)
                    .bg(theme.colors.surface)
                    .border_1()
                    .border_color(if is_open { theme.colors.accent } else { theme.colors.border })
                    .rounded(theme.borders.md)
                    .cursor_pointer()
                    .hover(|s| s.border_color(theme.colors.accent))
                    .id("dropdown-button")
                    .on_click(move |_e, cx| {
                        view.update(cx, |this, cx| {
                            this.toggle();
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_size(theme.typography.sm)
                            .text_color(theme.colors.text)
                            .child(selected_text)
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(theme.colors.text_secondary)
                            .child(if is_open { "▲" } else { "▼" })
                    )
            )
            // Dropdown menu (only if open)
            .when(is_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(px(36.0)) // Below the button
                        .left_0()
                        .w_full()
                        // .z_index(100) // Temporarily removed to fix compilation
                        .bg(theme.colors.surface)
                        .border_1()
                        .border_color(theme.colors.border)
                        .rounded(theme.borders.md)
                        .shadow_md()
                        .max_h(px(200.0))
                        .overflow_y_hidden() // Fallback since scroll method is elusive
                        .children(
                            self.options.iter().enumerate().map(|(idx, option)| {
                                let option_clone = option.clone();
                                let is_selected = self.selected_index == Some(idx);
                                let view = cx.view().clone();
                                let on_change = self.on_change.clone();

                                div()
                                    .flex()
                                    .items_center()
                                    .w_full()
                                    .px(theme.spacing.sm)
                                    .py(px(6.0))
                                    .text_size(theme.typography.sm)
                                    .text_color(if is_selected { theme.colors.accent } else { theme.colors.text })
                                    .bg(if is_selected { theme.colors.surface_active } else { theme.colors.surface })
                                    .hover(|s| s.bg(theme.colors.surface_hover).cursor_pointer())
                                    .child(option_clone.clone())
                                    .id(idx)
                                    .on_click(move |_e, cx| {
                                        let selected_opt = option_clone.clone();
                                        view.update(cx, |this, cx| {
                                            this.set_selected(idx);
                                            // Notify parent via callback
                                            if let Some(callback) = &on_change {
                                                callback(&selected_opt, idx, cx);
                                            }
                                            cx.notify();
                                        });
                                    })
                            })
                        )
                )
            })
    }
}
