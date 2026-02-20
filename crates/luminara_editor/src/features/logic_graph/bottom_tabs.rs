//! Bottom Tab Panel
//!
//! The bottom panel with tabs for DB Query, AI Assistant, Node Palette, and Variables.
//! Matches the HTML prototype design.

use crate::ui::theme::Theme;
use super::{Variable, VariableScope, NodePaletteItem};
use gpui::{
    div, px, IntoElement, ParentElement, RenderOnce, Styled, InteractiveElement,
    WindowContext,
};
use std::sync::Arc;

/// Kind of tab in the bottom panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    DbQuery,
    AiAssistant,
    NodePalette,
    Variables,
}

impl TabKind {
    /// Get the tab icon
    pub fn icon(&self) -> &'static str {
        match self {
            TabKind::DbQuery => "🗄",
            TabKind::AiAssistant => "🤖",
            TabKind::NodePalette => "🎨",
            TabKind::Variables => "📊",
        }
    }

    /// Get the tab label
    pub fn label(&self) -> &'static str {
        match self {
            TabKind::DbQuery => "DB Query",
            TabKind::AiAssistant => "AI Assistant",
            TabKind::NodePalette => "Node Palette",
            TabKind::Variables => "Variables",
        }
    }
}

/// A tab item in the bottom panel
#[derive(Debug, Clone)]
pub struct BottomTab {
    pub kind: TabKind,
    pub active: bool,
}

impl BottomTab {
    /// Create a new tab
    pub fn new(kind: TabKind) -> Self {
        Self { kind, active: false }
    }

    /// Set active state
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

/// The bottom tab panel component
#[derive(Debug, Clone)]
pub struct BottomTabPanel {
    /// Tabs
    tabs: Vec<BottomTab>,
    /// Active tab index
    active_tab: usize,
    /// Theme
    theme: Arc<Theme>,
    /// Query text (for DB Query tab)
    query_text: String,
    /// AI messages
    ai_messages: Vec<(bool, String)>, // (is_ai, message)
    /// AI input text
    ai_input: String,
    /// Variables list
    variables: Vec<Variable>,
    /// Node palette items
    palette_items: Vec<NodePaletteItem>,
}

impl BottomTabPanel {
    /// Create a new panel with default tabs
    pub fn new(theme: Arc<Theme>) -> Self {
        let tabs = vec![
            BottomTab::new(TabKind::DbQuery).with_active(true),
            BottomTab::new(TabKind::AiAssistant),
            BottomTab::new(TabKind::NodePalette),
            BottomTab::new(TabKind::Variables),
        ];

        let variables = vec![
            Variable::new("player_hp", "100", VariableScope::Global),
            Variable::new("has_sword", "true", VariableScope::Global),
            Variable::new("gold", "150", VariableScope::Global),
            Variable::new("boss_defeated", "false", VariableScope::Scene),
        ];

        let ai_messages = vec![
            (true, "どのようなフローにしますか？".to_string()),
            (false, "村からダンジョンへの分岐で、鍵を持っている場合のみ進めるようにして".to_string()),
        ];

        Self {
            tabs,
            active_tab: 0,
            theme,
            query_text: "SELECT * FROM logic_node WHERE graph_id = 'main_quest' AND kind = 'condition';".to_string(),
            ai_messages,
            ai_input: String::new(),
            variables,
            palette_items: NodePaletteItem::default_palette(),
        }
    }

    /// Set active tab
    pub fn set_active_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs[self.active_tab].active = false;
            self.active_tab = index;
            self.tabs[index].active = true;
        }
    }

    /// Get the active tab kind
    pub fn active_tab_kind(&self) -> TabKind {
        self.tabs[self.active_tab].kind
    }

    /// Set query text
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query_text = query.into();
    }

    /// Add AI message
    pub fn add_ai_message(&mut self, is_ai: bool, message: impl Into<String>) {
        self.ai_messages.push((is_ai, message.into()));
    }

    /// Set AI input
    pub fn set_ai_input(&mut self, input: impl Into<String>) {
        self.ai_input = input.into();
    }

    /// Send AI message
    pub fn send_ai_message(&mut self) {
        if !self.ai_input.is_empty() {
            let message = self.ai_input.clone();
            self.add_ai_message(false, message);
            self.ai_input.clear();
        }
    }

    /// Render the tab header
    fn render_tab_header(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .px(px(8.0))
            .bg(theme.colors.panel_header)
            .border_b_1()
            .border_color(theme.colors.border)
            .children(
                self.tabs.iter().enumerate().map(|(_index, tab)| {
                    let is_active = tab.active;
                    let theme = theme.clone();
                    
                    div()
                        .px(px(16.0))
                        .py(px(8.0))
                        .text_size(px(12.0))
                        .text_color(if is_active {
                            theme.colors.accent
                        } else {
                            theme.colors.text_secondary
                        })
                        .border_b_2()
                        .border_color(if is_active {
                            theme.colors.accent
                        } else {
                            gpui::transparent_black()
                        })
                        .bg(if is_active {
                            theme.colors.condition_bg
                        } else {
                            theme.colors.panel_header
                        })
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(tab.kind.icon())
                        .child(tab.kind.label())
                })
            )
    }

    /// Render DB Query tab content
    fn render_db_query(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .p(px(12.0))
            .gap(px(8.0))
            // Query editor
            .child(
                div()
                    .px(px(10.0))
                    .py(px(8.0))
                    .bg(theme.colors.canvas_background)
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(theme.colors.border)
                    .font_family("monospace")
                    .text_size(px(12.0))
                    .text_color(theme.colors.text)
                    .child(self.query_text.clone())
            )
            // Execute button
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(14.0))
                    .py(px(6.0))
                    .bg(theme.colors.toolbar_active)
                    .rounded(px(6.0))
                    .text_size(px(12.0))
                    .text_color(theme.colors.text)
                    .child("▶")
                    .child("Execute")
            )
            // Results table
            .child(
                div()
                    .mt(px(4.0))
                    .child(
                        div()
                            .w_full()

                            .child(
                                div()
                                    .flex()
                                    .border_b_1()
                                    .border_color(theme.colors.border)
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.colors.text)
                                            .child("id")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.colors.text)
                                            .child("kind")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.colors.text)
                                            .child("label")
                                    )
                            )
                            .child(
                                div()
                                    .flex()
                                    .border_b_1()
                                    .border_color(theme.colors.border.opacity(0.5))
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .text_color(theme.colors.text_secondary)
                                            .child("node_1")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .text_color(theme.colors.text_secondary)
                                            .child("state")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .text_color(theme.colors.text_secondary)
                                            .child("Village")
                                    )
                            )
                            .child(
                                div()
                                    .flex()
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .text_color(theme.colors.text_secondary)
                                            .child("node_42")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .text_color(theme.colors.text_secondary)
                                            .child("condition")
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .text_size(px(11.0))
                                            .text_color(theme.colors.text_secondary)
                                            .child("Branch")
                                    )
                            )
                    )
            )
    }

    /// Render AI Assistant tab content
    fn render_ai_assistant(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .h_full()
            .p(px(12.0))
            // Messages
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .overflow_hidden()
                    .max_h(px(150.0))
                    .mb(px(10.0))
                    .children(
                        self.ai_messages.iter().map(|(is_ai, message)| {
                            let theme = theme.clone();
                            
                            div()
                                .flex()
                                .items_start()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .text_color(if *is_ai {
                                            theme.colors.accent
                                        } else {
                                            theme.colors.text_secondary
                                        })
                                        .child(if *is_ai { "🤖" } else { "👤" })
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(theme.colors.text)
                                        .child(message.clone())
                                )
                        })
                    )
            )
            // Input area
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .mt(px(12.0))
                    .child(
                        div()
                            .flex_1()
                            .px(px(14.0))
                            .py(px(8.0))
                            .bg(theme.colors.surface_active)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded_full()
                            .text_size(px(12.0))
                            .text_color(theme.colors.text)
                            .child(if self.ai_input.is_empty() {
                                "AIに指示...".to_string()
                            } else {
                                self.ai_input.clone()
                            })
                    )
                    .child(
                        div()
                            .px(px(12.0))
                            .py(px(8.0))
                            .bg(theme.colors.toolbar_active)
                            .rounded_full()
                            .text_size(px(12.0))
                            .text_color(theme.colors.text)
                            .hover(|style| style.bg(theme.colors.accent_hover))
                            .child("📨")
                    )
            )
    }

    /// Render Node Palette tab content
    fn render_node_palette(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .p(px(12.0))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.0))
                    .children(
                        self.palette_items.iter().map(|item| {
                            let theme = theme.clone();
                            
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(6.0))
                                .p(px(8.0))
                                .bg(theme.colors.surface_active)
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(theme.colors.border)
                                .hover(|style| style.bg(theme.colors.surface_hover))
                                .child(
                                    div()
                                        .w(px(12.0))
                                        .h(px(12.0))
                                        .rounded_full()
                                        .bg(item.color)
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(theme.colors.text)
                                        .child(item.name.clone())
                                )
                        })
                    )
            )
            .child(
                div()
                    .mt(px(12.0))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(px(12.0))
                    .text_color(theme.colors.text_secondary)
                    .child("⭐")
                    .child("お気に入り: Condition, State")
            )
    }

    /// Render Variables tab content
    fn render_variables(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .p(px(12.0))
            .child(
                div()
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .border_b_1()
                            .border_color(theme.colors.border)
                            .child(
                                div()
                                    .flex_1()
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .text_size(px(11.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.colors.text)
                                    .child("key")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .text_size(px(11.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.colors.text)
                                    .child("value")
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .text_size(px(11.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.colors.text)
                                    .child("scope")
                            )
                    )
                    .children(
                        self.variables.iter().map(|var| {
                            let theme = theme.clone();
                            let scope_color = match var.scope {
                                VariableScope::Global => theme.colors.success,
                                VariableScope::Scene => theme.colors.warning,
                                VariableScope::Local => theme.colors.text_secondary,
                            };
                            
                            div()
                                .flex()
                                .border_b_1()
                                .border_color(theme.colors.border.opacity(0.5))
                                .child(
                                    div()
                                        .flex_1()
                                        .px(px(8.0))
                                        .py(px(6.0))
                                        .text_size(px(11.0))
                                        .text_color(theme.colors.text)
                                        .child(var.key.clone())
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .px(px(8.0))
                                        .py(px(6.0))
                                        .text_size(px(11.0))
                                        .text_color(theme.colors.text_secondary)
                                        .child(var.value.clone())
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .px(px(8.0))
                                        .py(px(6.0))
                                        .text_size(px(11.0))
                                        .text_color(scope_color)
                                        .child(format!("{:?}", var.scope).to_lowercase())
                                )
                        })
                    )
            )
            .child(
                div()
                    .mt(px(12.0))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(12.0))
                    .py(px(6.0))
                    .bg(theme.colors.surface_active)
                    .rounded(px(6.0))
                    .text_size(px(12.0))
                    .text_color(theme.colors.text)
                    .hover(|style| style.bg(theme.colors.surface_hover))
                    .child("+")
                    .child("Add variable")
            )
    }

    /// Render the active tab content
    fn render_active_content(&self) -> impl IntoElement {
        match self.active_tab_kind() {
            TabKind::DbQuery => div().child(self.render_db_query()).into_any_element(),
            TabKind::AiAssistant => div().child(self.render_ai_assistant()).into_any_element(),
            TabKind::NodePalette => div().child(self.render_node_palette()).into_any_element(),
            TabKind::Variables => div().child(self.render_variables()).into_any_element(),
        }
    }

    /// Render the full panel
    pub fn render(&self) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .min_h(px(220.0))
            .max_h(px(300.0))
            .bg(theme.colors.surface)
            .border_t_1()
            .border_color(theme.colors.border)
            .child(self.render_tab_header())
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.render_active_content())
            )
    }
}

impl RenderOnce for BottomTabPanel {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div().child("BottomTabPanel")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let panel = BottomTabPanel::new(Arc::new(Theme::default_dark()));
        
        assert_eq!(panel.tabs.len(), 4);
        assert_eq!(panel.active_tab, 0);
        assert_eq!(panel.active_tab_kind(), TabKind::DbQuery);
    }

    #[test]
    fn test_tab_switching() {
        let mut panel = BottomTabPanel::new(Arc::new(Theme::default_dark()));
        
        assert!(panel.tabs[0].active);
        assert!(!panel.tabs[1].active);
        
        panel.set_active_tab(1);
        
        assert!(!panel.tabs[0].active);
        assert!(panel.tabs[1].active);
        assert_eq!(panel.active_tab, 1);
    }

    #[test]
    fn test_tab_kinds() {
        assert_eq!(TabKind::DbQuery.label(), "DB Query");
        assert_eq!(TabKind::AiAssistant.label(), "AI Assistant");
        assert_eq!(TabKind::NodePalette.label(), "Node Palette");
        assert_eq!(TabKind::Variables.label(), "Variables");
    }

    #[test]
    fn test_ai_messages() {
        let mut panel = BottomTabPanel::new(Arc::new(Theme::default_dark()));
        
        let initial_count = panel.ai_messages.len();
        panel.add_ai_message(false, "Test message");
        
        assert_eq!(panel.ai_messages.len(), initial_count + 1);
        assert_eq!(panel.ai_messages.last().unwrap().1, "Test message");
    }

    #[test]
    fn test_query_setting() {
        let mut panel = BottomTabPanel::new(Arc::new(Theme::default_dark()));
        
        panel.set_query("SELECT * FROM nodes;");
        assert_eq!(panel.query_text, "SELECT * FROM nodes;");
    }
}
