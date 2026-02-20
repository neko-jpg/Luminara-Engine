//! Script Editor Component
//!
//! A code editor-like panel with:
//! - Tab bar for open files
//! - Code display area with syntax highlighting styling
//! - Support for Rust, SQL, and other languages

use gpui::{
    div, px, IntoElement, ParentElement, Styled, svg, InteractiveElement,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// A single tab in the script editor
#[derive(Debug, Clone)]
pub struct ScriptTab {
    /// File name
    pub name: String,
    /// File icon (based on language)
    pub icon: String,
    /// Whether this tab is active
    pub is_active: bool,
}

impl ScriptTab {
    /// Create a new script tab
    pub fn new(name: &str, language: &str) -> Self {
        let icon = match language {
            "rust" => "icons/file-code.svg",
            "sql" => "icons/database.svg",
            _ => "icons/file.svg",
        }.to_string();
        
        Self {
            name: name.to_string(),
            icon,
            is_active: false,
        }
    }
    
    /// Set as active tab
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}

/// Script Editor component
pub struct ScriptEditor {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Open tabs
    tabs: Vec<ScriptTab>,
    /// Currently active tab index
    active_tab_index: usize,
    /// Editor content (for demo purposes)
    content: String,
}

impl ScriptEditor {
    /// Create a new Script Editor with demo content
    pub fn new(theme: Arc<Theme>) -> Self {
        let mut tabs = vec![
            ScriptTab::new("player.rs", "rust"),
            ScriptTab::new("enemy.rs", "rust"),
            ScriptTab::new("query.sql", "sql"),
        ];
        tabs[0].set_active(true);
        
        let content = r#"use luminara::prelude::*;

#[system]
fn player_movement(
    input: Res<Input>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>
) {
    for mut tf in &mut query {
        if input.pressed("right") {
            tf.position.x += 5.0 * time.delta();
        }
        if input.pressed("left") {
            tf.position.x -= 5.0 * time.delta();
        }
        // ジャンプ処理
        if input.just_pressed("space") && tf.grounded {
            tf.velocity.y = 8.0;
        }
    }
}"#.to_string();
        
        Self {
            theme,
            tabs,
            active_tab_index: 0,
            content,
        }
    }
    
    /// Set the active tab
    pub fn set_active_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs[self.active_tab_index].set_active(false);
            self.active_tab_index = index;
            self.tabs[index].set_active(true);
        }
    }
    
    /// Render the script editor
    pub fn render(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let tabs = self.tabs.clone();
        let content = self.content.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.surface)
            .border_1()
            .border_color(theme.colors.border)
            .rounded_t(theme.borders.xs)
            // Panel header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(32.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .items_center()
                    .px(theme.spacing.md)
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(theme.spacing.xs)
                            .child(
                                svg()
                                    .path("icons/code.svg")
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(theme.colors.accent)
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Script Editor")
                            )
                    )
                    .child(
                        svg()
                            .path("icons/more-vertical.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
            )
            // Tab bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(32.0))
                    .bg(theme.colors.surface)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .px(theme.spacing.sm)
                    .gap(px(2.0))
                    .items_end()
                    .children(
                        tabs.into_iter().map(|tab| {
                            self.render_tab(tab)
                        })
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .p(theme.spacing.xs)
                            .cursor_pointer()
                            .child(
                                svg()
                                    .path("icons/plus.svg")
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(theme.colors.text_secondary)
                            )
                    )
            )
            // Code editor area
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .bg(theme.colors.canvas_background)
                    .p(theme.spacing.md)
                    .child(
                        div()
                            .size_full()
                            // Note: Font customization requires platform-specific setup in GPUI v0.159.5
                            .text_size(theme.typography.ml)
                            .text_color(theme.colors.text)
                            .child(content)
                    )
            )
    }
    
    /// Render a single tab
    fn render_tab(&self, tab: ScriptTab) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_active = tab.is_active;
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing.xs)
            .px(theme.spacing.md)
            .py(px(6.0))
            .rounded_t(theme.borders.xs)
            .bg(if is_active {
                theme.colors.canvas_background
            } else {
                theme.colors.surface
            })
            .border_t_1()
            .border_l_1()
            .border_r_1()
            .border_color(if is_active {
                theme.colors.accent
            } else {
                theme.colors.border
            })
            .cursor_pointer()
            .child(
                svg()
                    .path(tab.icon)
                    .w(px(12.0))
                    .h(px(12.0))
                    .text_color(if is_active {
                        theme.colors.accent
                    } else {
                        theme.colors.text_secondary
                    })
            )
            .child(
                div()
                    .text_color(if is_active {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .text_size(theme.typography.sm)
                    .child(tab.name)
            )
            .child(
                div()
                    .ml(theme.spacing.xs)
                    .cursor_pointer()
                    .opacity(0.6)
                    .hover(|this| this.opacity(1.0))
                    .child(
                        svg()
                            .path("icons/x.svg")
                            .w(px(12.0))
                            .h(px(12.0))
                            .text_color(theme.colors.text_secondary)
                    )
            )
    }
}
