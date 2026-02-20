//! Extension Manager Bottom Tabs Component
//!
//! Displays bottom tab panel with:
//! - Extension Console (logs)
//! - API Docs
//! - Change Log

use gpui::{
    div, px, svg, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Bottom tab kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomTabKind {
    /// Extension console with logs
    ExtensionConsole,
    /// API documentation
    ApiDocs,
    /// Change log
    ChangeLog,
}

/// The Extension Manager Bottom Tabs component
pub struct ExtensionBottomTabs {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Currently active tab
    current_tab: BottomTabKind,
    /// Console log entries
    console_logs: Vec<String>,
    /// API docs content
    api_docs_content: String,
    /// Change log entries
    change_log: Vec<String>,
}

impl ExtensionBottomTabs {
    /// Create a new Extension Bottom Tabs component
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            current_tab: BottomTabKind::ExtensionConsole,
            console_logs: Self::default_console_logs(),
            api_docs_content: Self::default_api_docs(),
            change_log: Self::default_change_log(),
        }
    }

    /// Default console logs
    fn default_console_logs() -> Vec<String> {
        vec![
            "[14:32:12] Extension 'Shader Editor' loaded".to_string(),
            "[14:32:12] Registered widget: ShaderGraphCanvas".to_string(),
            "[14:32:12] Registered component: CustomShader".to_string(),
            "[14:32:15] Command 'shader.compile' executed successfully".to_string(),
        ]
    }

    /// Default API docs content
    fn default_api_docs() -> String {
        "📘 Extension API (Rust)\ntrait LuminaraBox { ... }\n📦 See extension.toml format documentation".to_string()
    }

    /// Default change log
    fn default_change_log() -> Vec<String> {
        vec![
            "• 2025-02-18: Shader Editor 1.0.0 released".to_string(),
            "• 2025-02-10: AI Assistant 2.3.0 added".to_string(),
        ]
    }

    /// Set the current tab
    pub fn set_tab(&mut self, tab: BottomTabKind) {
        self.current_tab = tab;
    }

    /// Get the current tab
    pub fn current_tab(&self) -> BottomTabKind {
        self.current_tab
    }

    /// Add a console log entry
    pub fn add_log(&mut self, message: String) {
        self.console_logs.push(message);
    }

    /// Render a tab header item
    fn render_tab_item(&self, tab: BottomTabKind, icon: &str, label: &str) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_active = self.current_tab == tab;
        let label = label.to_string();
        let icon = icon.to_string();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(theme.spacing.lg)
            .py(px(8.0))
            .border_b_2()
            .border_color(if is_active { theme.colors.accent } else { gpui::hsla(0.0, 0.0, 0.0, 0.0) })
            .bg(if is_active { theme.colors.condition_bg } else { gpui::hsla(0.0, 0.0, 0.0, 0.0) })
            .text_color(if is_active { theme.colors.accent } else { theme.colors.text_secondary })
            .text_size(theme.typography.md)
            .cursor_pointer()
            .hover(|this| {
                if !is_active {
                    this.bg(theme.colors.surface_hover)
                } else {
                    this
                }
            })
            .child(
                svg()
                    .path(format!("icons/{}.svg", icon))
                    .w(px(14.0))
                    .h(px(14.0))
            )
            .child(label)
    }

    /// Render the tab header
    fn render_tab_header(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(36.0))
            .bg(theme.colors.panel_header)
            .border_b_1()
            .border_color(theme.colors.border)
            .px(px(8.0))
            .gap(px(4.0))
            .child(self.render_tab_item(BottomTabKind::ExtensionConsole, "terminal", "Extension Console"))
            .child(self.render_tab_item(BottomTabKind::ApiDocs, "book", "API Docs"))
            .child(self.render_tab_item(BottomTabKind::ChangeLog, "history", "Change Log"))
    }

    /// Render the Extension Console tab content
    fn render_console_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .w_full()
            .h_full()
            .p(theme.spacing.md)
            .bg(rgb_to_hsla(0x1e1e1e))
            .font_family("monospace")
            .text_size(theme.typography.sm)
            .children(
                self.console_logs.iter().map(|log| {
                    div()
                        .text_color(theme.colors.text_secondary)
                        .child(log.clone())
                })
            )
    }

    /// Render the API Docs tab content
    fn render_api_docs_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .w_full()
            .h_full()
            .p(theme.spacing.md)
            .text_color(theme.colors.text)
            .text_size(theme.typography.md)
            .child(self.api_docs_content.clone())
    }

    /// Render the Change Log tab content
    fn render_change_log_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .w_full()
            .h_full()
            .p(theme.spacing.md)
            .text_color(theme.colors.text)
            .text_size(theme.typography.md)
            .children(
                self.change_log.iter().map(|entry| {
                    div()
                        .text_color(theme.colors.text_secondary)
                        .child(entry.clone())
                })
            )
    }

    /// Render the current tab content
    fn render_tab_content(&self) -> impl IntoElement {
        match self.current_tab {
            BottomTabKind::ExtensionConsole => self.render_console_tab().into_any_element(),
            BottomTabKind::ApiDocs => self.render_api_docs_tab().into_any_element(),
            BottomTabKind::ChangeLog => self.render_change_log_tab().into_any_element(),
        }
    }
}

impl Render for ExtensionBottomTabs {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.surface)
            .border_t_1()
            .border_color(theme.colors.border)
            // Tab header
            .child(self.render_tab_header())
            // Tab content
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.render_tab_content())
            )
    }
}

/// Helper to convert RGB to Hsla
fn rgb_to_hsla(rgb: u32) -> gpui::Hsla {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    
    let s = if max == min {
        0.0
    } else {
        let d = max - min;
        if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) }
    };
    
    let h = if max == min {
        0.0
    } else if max == r {
        ((g - b) / (max - min) + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / (max - min) + 2.0) / 6.0
    } else {
        ((r - g) / (max - min) + 4.0) / 6.0
    };
    
    gpui::Hsla { h, s, l, a: 1.0 }
}
