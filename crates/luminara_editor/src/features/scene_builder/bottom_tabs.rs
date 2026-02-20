//! Bottom Tab Panel Component
//!
//! Bottom panel with Console, Assets, DB Query, and AI Assistant tabs

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Bottom tab types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomTab {
    Console,
    Assets,
    DBQuery,
    AIAssistant,
}

impl BottomTab {
    pub fn label(&self) -> &'static str {
        match self {
            BottomTab::Console => "Console",
            BottomTab::Assets => "Asset Browser",
            BottomTab::DBQuery => "DB Query",
            BottomTab::AIAssistant => "AI Assistant",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BottomTab::Console => "▶",
            BottomTab::Assets => "📁",
            BottomTab::DBQuery => "🗄",
            BottomTab::AIAssistant => "🤖",
        }
    }
}

/// Log entry for console
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn color(&self, theme: &Theme) -> gpui::Hsla {
        match self {
            LogLevel::Info => theme.colors.text,
            LogLevel::Warning => theme.colors.warning,
            LogLevel::Error => theme.colors.error,
        }
    }
}

/// Asset item
#[derive(Debug, Clone)]
pub struct AssetItem {
    pub name: String,
    pub icon: String,
}

/// Chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
}

/// Bottom tab panel component
pub struct BottomTabPanel {
    theme: Arc<Theme>,
    active_tab: BottomTab,
    logs: Vec<LogEntry>,
    assets: Vec<AssetItem>,
    query_text: String,
    chat_messages: Vec<ChatMessage>,
    #[allow(dead_code)]
    chat_input: String,
}

impl BottomTabPanel {
    /// Create a new bottom tab panel
    pub fn new(theme: Arc<Theme>) -> Self {
        let logs = vec![
            LogEntry {
                timestamp: "00:00:00".to_string(),
                level: LogLevel::Info,
                message: "Console initialized".to_string(),
            },
            LogEntry {
                timestamp: "00:00:01".to_string(),
                level: LogLevel::Warning,
                message: "Physics: Invalid mesh collision".to_string(),
            },
            LogEntry {
                timestamp: "00:00:02".to_string(),
                level: LogLevel::Error,
                message: "Failed to load texture: file not found".to_string(),
            },
        ];

        let assets = vec![
            AssetItem { name: "Player.fbx".to_string(), icon: "◆".to_string() },
            AssetItem { name: "Enemy.fbx".to_string(), icon: "◆".to_string() },
            AssetItem { name: "Terrain.png".to_string(), icon: "🖼".to_string() },
            AssetItem { name: "Main.mat".to_string(), icon: "◈".to_string() },
        ];

        let chat_messages = vec![
            ChatMessage {
                is_user: false,
                text: "Hello! I'm your AI assistant. How can I help you today?".to_string(),
            },
        ];

        Self {
            theme,
            active_tab: BottomTab::Console,
            logs,
            assets,
            query_text: "SELECT * FROM entities WHERE name CONTAINS 'Player';".to_string(),
            chat_messages,
            chat_input: String::new(),
        }
    }

    /// Set active tab
    pub fn set_active_tab(&mut self, tab: BottomTab) {
        self.active_tab = tab;
    }

    /// Get active tab
    pub fn active_tab(&self) -> BottomTab {
        self.active_tab
    }

    /// Render tab button
    fn render_tab(&self, tab: BottomTab, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_active = self.active_tab == tab;
        let tab_clone = tab;

        div()
            .px(theme.spacing.lg)
            .py(theme.spacing.sm)
            .border_b_2()
            .border_color(if is_active { theme.colors.accent } else { gpui::transparent_black() })
            .cursor_pointer()
            .hover(|this| {
                if !is_active {
                    this.bg(theme.colors.surface_hover)
                } else {
                    this
                }
            })
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                this.set_active_tab(tab_clone);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing.sm)
                    .child(
                        div()
                            .text_color(if is_active { theme.colors.accent } else { theme.colors.text_secondary })
                            .text_size(theme.typography.sm)
                            .child(tab.icon())
                    )
                    .child(
                        div()
                            .text_color(if is_active { theme.colors.accent } else { theme.colors.text_secondary })
                            .text_size(theme.typography.sm)
                            .child(tab.label())
                    )
            )
    }

    /// Render console content
    fn render_console_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .p(theme.spacing.sm)
            .gap(theme.spacing.xs)
            .overflow_hidden()
            .children(self.logs.iter().map(|log| {
                let theme = theme.clone();
                let color = log.level.color(&theme);

                div()
                    .flex()
                    .flex_row()
                    .gap(theme.spacing.sm)
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.xs)
                            .child(format!("[{}]", log.timestamp))
                    )
                    .child(
                        div()
                            .text_color(color)
                            .text_size(theme.typography.sm)
                            .child(log.message.clone())
                    )
            }))
    }

    /// Render assets content
    fn render_assets_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .p(theme.spacing.sm)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .gap(theme.spacing.sm)
                    .mb(theme.spacing.md)
                    .child(
                        // Search box
                        div()
                            .flex_1()
                            .h(px(28.0))
                            .px(theme.spacing.sm)
                            .bg(theme.colors.background)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child("Search assets...")
                            )
                    )
                    .child(
                        // Filter button
                        div()
                            .px(theme.spacing.md)
                            .h(px(28.0))
                            .bg(theme.colors.surface_hover)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.colors.surface_active))
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.sm)
                                    .child("Filter")
                            )
                    )
            )
            .child(
                // Asset grid
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(theme.spacing.sm)
                    .children(self.assets.iter().map(|asset| {
                        let theme = theme.clone();
                        div()
                            .w(px(80.0))
                            .flex()
                            .flex_col()
                            .items_center()
                            .p(theme.spacing.sm)
                            .bg(theme.colors.surface_hover)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.sm)
                            .hover(|this| this.bg(theme.colors.surface_active))
                            .cursor_pointer()
                            .child(
                                div()
                                    .text_color(theme.colors.accent)
                                    .text_size(theme.typography.xxl)
                                    .child(asset.icon.clone())
                            )
                            .child(
                                div()
                                    .mt(theme.spacing.xs)
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.xs)
                                    .child(asset.name.clone())
                            )
                    }))
            )
    }

    /// Render DB query content
    fn render_db_query_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .p(theme.spacing.sm)
            .gap(theme.spacing.sm)
            .child(
                // Query input
                div()
                    .w_full()
                    .h(px(80.0))
                    .p(theme.spacing.sm)
                    .bg(theme.colors.background)
                    .border_1()
                    .border_color(theme.colors.border)
                    .rounded(theme.borders.xs)
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .font_family("monospace")
                            .child(self.query_text.clone())
                    )
            )
            .child(
                // Execute button
                div()
                    .flex()
                    .flex_row()
                    .gap(theme.spacing.sm)
                    .child(
                        div()
                            .px(theme.spacing.lg)
                            .py(theme.spacing.sm)
                            .bg(theme.colors.accent)
                            .rounded(theme.borders.xs)
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.colors.accent_hover))
                            .child(
                                div()
                                    .text_color(theme.colors.background)
                                    .text_size(theme.typography.sm)
                                    .child("Execute")
                            )
                    )
                    .child(
                        div()
                            .px(theme.spacing.lg)
                            .py(theme.spacing.sm)
                            .bg(theme.colors.surface_hover)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.colors.surface_active))
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.sm)
                                    .child("Clear")
                            )
                    )
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.xs)
                    .child("Query results will appear here...")
            )
    }

    /// Render AI assistant content
    fn render_ai_assistant_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .p(theme.spacing.sm)
            .gap(theme.spacing.sm)
            .child(
                // Chat history
                div()
                    .flex_1()
                    .w_full()
                    .p(theme.spacing.sm)
                    .bg(theme.colors.background)
                    .border_1()
                    .border_color(theme.colors.border)
                    .rounded(theme.borders.xs)
                    .overflow_hidden()
                    .children(self.chat_messages.iter().map(|msg| {
                        let theme = theme.clone();
                        let is_user = msg.is_user;

                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .mb(theme.spacing.sm)
                            .child(div().flex_1()) // Spacer for alignment
                            .child(
                                div()
                                    .max_w(px(400.0))
                                    .p(theme.spacing.md)
                                    .rounded(theme.borders.md)
                                    .bg(if is_user { theme.colors.accent } else { theme.colors.surface_hover })
                                    .child(
                                        div()
                                            .text_color(if is_user { theme.colors.background } else { theme.colors.text })
                                            .text_size(theme.typography.sm)
                                            .child(msg.text.clone())
                                    )
                            )
                    }))
            )
            .child(
                // Input area
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .gap(theme.spacing.sm)
                    .child(
                        div()
                            .flex_1()
                            .h(px(36.0))
                            .px(theme.spacing.md)
                            .bg(theme.colors.background)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.rounded)
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child("Ask the AI assistant...")
                            )
                    )
                    .child(
                        div()
                            .px(theme.spacing.lg)
                            .h(px(36.0))
                            .bg(theme.colors.accent)
                            .rounded(theme.borders.rounded)
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.colors.accent_hover))
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.colors.background)
                                    .text_size(theme.typography.sm)
                                    .child("Send")
                            )
                    )
            )
    }

    /// Render active tab content
    fn render_tab_content(&self) -> impl IntoElement {
        match self.active_tab {
            BottomTab::Console => self.render_console_content().into_any_element(),
            BottomTab::Assets => self.render_assets_content().into_any_element(),
            BottomTab::DBQuery => self.render_db_query_content().into_any_element(),
            BottomTab::AIAssistant => self.render_ai_assistant_content().into_any_element(),
        }
    }
}

impl Render for BottomTabPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .h(px(200.0))
            .min_h(px(150.0))
            .max_h(px(300.0))
            .bg(theme.colors.surface)
            .border_t_1()
            .border_color(theme.colors.border)
            .child(
                // Tab bar
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(36.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .items_center()
                    .px(theme.spacing.sm)
                    .child(self.render_tab(BottomTab::Console, cx))
                    .child(self.render_tab(BottomTab::Assets, cx))
                    .child(self.render_tab(BottomTab::DBQuery, cx))
                    .child(self.render_tab(BottomTab::AIAssistant, cx))
            )
            .child(
                // Tab content
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(self.render_tab_content())
            )
    }
}

