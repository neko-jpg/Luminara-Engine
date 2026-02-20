//! Settings Panel Component
//!
//! A full-screen settings overlay.

use crate::ui::theme::Theme;
use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled,
    ViewContext, View, VisualContext,
};
use std::sync::Arc;

/// Settings category types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Editor,
    Shortcuts,
    AiAssistant,
    Database,
    Extensions,
    Build,
    Advanced,
}

impl SettingsCategory {
    /// Get all categories
    pub fn all() -> Vec<(SettingsCategory, &'static str, &'static str)> {
        vec![
            (SettingsCategory::General, "General", "icons/sliders.svg"),
            (SettingsCategory::Editor, "Editor", "icons/edit.svg"),
            (SettingsCategory::Shortcuts, "Shortcuts", "icons/keyboard.svg"),
            (SettingsCategory::AiAssistant, "AI", "icons/robot.svg"),
            (SettingsCategory::Database, "Database", "icons/database.svg"),
            (SettingsCategory::Extensions, "Extensions", "icons/puzzle.svg"),
            (SettingsCategory::Build, "Build", "icons/hammer.svg"),
            (SettingsCategory::Advanced, "Advanced", "icons/cog.svg"),
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Editor => "Editor",
            SettingsCategory::Shortcuts => "Shortcuts",
            SettingsCategory::AiAssistant => "AI Assistant",
            SettingsCategory::Database => "Database",
            SettingsCategory::Extensions => "Extensions",
            SettingsCategory::Build => "Build",
            SettingsCategory::Advanced => "Advanced",
        }
    }
}

/// The Settings Panel component
pub struct SettingsPanel {
    active_category: SettingsCategory,
    #[allow(dead_code)]
    theme: Arc<Theme>,
    visible: bool,
}

impl SettingsPanel {
    /// Create a new SettingsPanel
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            active_category: SettingsCategory::General,
            theme,
            visible: false,
        }
    }

    /// Create as a GPUI View
    pub fn view(theme: Arc<Theme>, cx: &mut gpui::WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self::new(theme))
    }

    pub fn show(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = true;
    }

    pub fn hide(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = false;
    }

    pub fn toggle(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn render_sidebar(&self, _cx: &ViewContext<Self>) -> impl IntoElement {
        let categories = SettingsCategory::all();
        
        div()
            .w(px(220.0))
            .h_full()
            .bg(self.theme.colors.background)
            .border_r_1()
            .border_color(self.theme.colors.border)
            .p(px(12.0))
            .pt(px(24.0))
            .flex()
            .flex_col()
            .gap(px(4.0))
            .children(categories.into_iter().map(|(cat, name, _icon)| {
                let is_active = self.active_category == cat;
                
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .px(px(16.0))
                    .py(px(10.0))
                    .rounded(px(8.0))
                    .bg(if is_active { self.theme.colors.toolbar_active } else { self.theme.colors.background })
                    .text_color(if is_active { self.theme.colors.text } else { self.theme.colors.text_secondary })
                    .text_size(px(13.0))
                    .child(name)
            }))
    }

    fn render_content(&self, _cx: &ViewContext<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .bg(self.theme.colors.background)
            .p(px(24.0))
            .child(
                div()
                    .text_size(px(20.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(self.theme.colors.text)
                    .child(self.active_category.name())
            )
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        if !self.visible {
            return div();
        }

        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .w_full()
            .h_full()
            .bg(self.theme.colors.surface)
            .flex()
            .flex_row()
            .child(self.render_sidebar(cx))
            .child(self.render_content(cx))
    }
}
