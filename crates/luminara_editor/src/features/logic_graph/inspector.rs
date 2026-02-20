//! Node Inspector Panel
//!
//! The right-side panel for editing node properties, conditions, and actions.
//! Matches the HTML prototype design with property rows, condition builder,
//! action list, and related links.

use crate::ui::theme::Theme;
use super::{GraphNode, Condition, Action};
use gpui::{
    div, px, IntoElement, ParentElement, RenderOnce, Styled, InteractiveElement,
    WindowContext, prelude::FluentBuilder,
};
use std::sync::Arc;

/// The node inspector component
#[derive(Debug, Clone)]
pub struct NodeInspector {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Conditions for condition builder
    conditions: Vec<Condition>,
    /// Actions for on_enter
    actions: Vec<Action>,
    /// View mode (Visual or YAML)
    view_mode: ViewMode,
}

/// View mode for the inspector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Visual,
    Yaml,
}

impl NodeInspector {
    /// Create a new inspector
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            conditions: Vec::new(),
            actions: Vec::new(),
            view_mode: ViewMode::Visual,
        }
    }

    /// Set the node to inspect
    pub fn with_node(mut self, node: &GraphNode) -> Self {
        // Initialize with sample data based on node type
        match node.kind {
            super::NodeKind::Condition => {
                self.conditions = vec![
                    Condition::new("has_sword", "==", "true"),
                    Condition::new("gold", ">=", "100").with_or(),
                ];
            }
            _ => {}
        }
        
        // Sample actions
        self.actions = vec![
            Action::new("🐉", "spawn dragon", "prefab: enemy_dragon"),
            Action::new("🎵", "play music", "boss_battle.mp3"),
            Action::new("💬", "show UI", "BossWarning"),
        ];
        
        self
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Toggle view mode
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Visual => ViewMode::Yaml,
            ViewMode::Yaml => ViewMode::Visual,
        };
    }

    /// Render an inspector section header
    fn render_section_header(&self, title: impl Into<String>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .mb(px(8.0))
            .child(
                div()
                    .w(px(3.0))
                    .h(px(14.0))
                    .rounded(px(2.0))
                    .bg(theme.colors.accent)
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.colors.text)
                    .child(title.into())
            )
    }

    /// Render a property row
    fn render_property_row(&self, label: impl Into<String>, value: impl IntoElement) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .items_center()
            .mb(px(8.0))
            .child(
                div()
                    .w(px(80.0))
                    .text_size(px(12.0))
                    .text_color(theme.colors.text_secondary)
                    .child(label.into())
            )
            .child(
                div()
                    .flex_1()
                    .child(value)
            )
    }

    /// Render the condition builder
    fn render_condition_builder(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .p(px(10.0))
            .mb(px(10.0))
            .bg(theme.colors.condition_bg)
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.colors.accent.opacity(0.3))
            .children(
                self.conditions.iter().enumerate().map(|(i, cond)| {
                    let theme = theme.clone();
                    let connector = if cond.is_and { "AND" } else { "OR" };
                    
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .mb(px(6.0))
                        .child(
                            div()
                                .text_color(theme.colors.text_secondary)
                                .child("☰")
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.colors.text)
                                .child(if i == 0 { "IF".to_string() } else { connector.to_string() })
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded(px(4.0))
                                .bg(theme.colors.surface_active)
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.colors.accent)
                                .child(cond.variable.clone())
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded(px(4.0))
                                .bg(theme.colors.surface)
                                .text_size(px(11.0))
                                .text_color(theme.colors.text_secondary)
                                .child(cond.operator.clone())
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded(px(4.0))
                                .bg(theme.colors.surface_active)
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.colors.text)
                                .child(cond.value.clone())
                        )
                })
            )
            .child(
                div()
                    .mt(px(8.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(6.0))
                            .bg(theme.colors.toolbar_active)
                            .rounded(px(4.0))
                            .text_size(px(12.0))
                            .text_color(theme.colors.text)
                            .hover(|style| style.bg(theme.colors.accent_hover))
                            .child("+")
                            .child("Add Condition")
                    )
            )
    }

    /// Render the action list
    fn render_action_list(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .mb(px(10.0))
            .children(
                self.actions.iter().map(|action| {
                    let theme = theme.clone();
                    
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(8.0))
                        .py(px(6.0))
                        .border_b_1()
                        .border_color(theme.colors.border)
                        .child(
                            div()
                                .text_size(px(14.0))
                                .child(action.icon.clone())
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.colors.text)
                                .child(action.name.clone())
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(theme.colors.text_secondary)
                                .child(format!("({})", action.detail))
                        )
                })
            )
            .child(
                div()
                    .mt(px(6.0))
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(6.0))
                            .px(px(10.0))
                            .py(px(8.0))
                            .bg(theme.colors.surface_active)
                            .rounded(px(4.0))
                            .text_size(px(12.0))
                            .text_color(theme.colors.text)
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("+")
                            .child("Add Action")
                    )
            )
    }

    /// Render related links
    fn render_related_links(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let links = vec![
            ("📦", "Scene Builder", "dungeon"),
            ("🎬", "Director", "boss_cutscene"),
            ("📜", "Script", "ai/dragon_behavior.rs"),
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .children(
                links.into_iter().map(|(icon, category, name)| {
                    let theme = theme.clone();
                    
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .child(icon)
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.colors.accent)
                                .child(format!("{}: {}", category, name))
                        )
                })
            )
    }

    /// Render view mode toggle
    fn render_view_toggle(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let current_mode = self.view_mode;

        div()
            .flex()
            .gap(px(8.0))
            .mt(px(16.0))
            .child(
                div()
                    .px(px(12.0))
                    .py(px(6.0))
                    .rounded(px(20.0))
                    .bg(if current_mode == ViewMode::Visual {
                        theme.colors.toolbar_active
                    } else {
                        theme.colors.surface_active
                    })
                    .text_size(px(12.0))
                    .text_color(if current_mode == ViewMode::Visual {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .child("👁 Visual")
            )
            .child(
                div()
                    .px(px(12.0))
                    .py(px(6.0))
                    .rounded(px(20.0))
                    .bg(if current_mode == ViewMode::Yaml {
                        theme.colors.toolbar_active
                    } else {
                        theme.colors.surface_active
                    })
                    .text_size(px(12.0))
                    .text_color(if current_mode == ViewMode::Yaml {
                        theme.colors.text
                    } else {
                        theme.colors.text_secondary
                    })
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .child("</> YAML")
            )
    }

    /// Render the full inspector
    pub fn render(&self, node: Option<GraphNode>) -> impl IntoElement {
        let theme = self.theme.clone();

        if let Some(node) = node {
            let title = node.display_title().to_string();
            let kind_color = node.icon_color;
            let node_kind = node.kind;

            div()
                .flex()
                .flex_col()
                .h_full()
                .bg(theme.colors.surface)
                .border_1()
                .border_color(theme.colors.border)
                .rounded_tl(px(4.0))
                .rounded_tr(px(4.0))
                // Header
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.0))
                        .py(px(8.0))
                        .bg(theme.colors.panel_header)
                        .border_b_1()
                        .border_color(theme.colors.border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.colors.text_secondary)
                                .child("ℹ")
                                .child("Node Inspector")
                        )
                        .child(
                            div()
                                .text_color(theme.colors.text_secondary)
                                .child("⋮")
                        )
                )
                // Content
                .child(
                    div()
                        .flex_1()
                        .p(px(12.0))
                        .overflow_hidden()
                        // Node info section
                        .child(self.render_section_header(format!("Node: {}", title.clone())))
                        .child(self.render_property_row("ID", 
                            div().text_size(px(12.0)).text_color(theme.colors.text).child(format!("node_{}", node.id.0))))
                        .child(self.render_property_row("Label",
                            div()
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(theme.colors.surface_active)
                                .rounded(px(4.0))
                                .text_size(px(12.0))
                                .text_color(theme.colors.text)
                                .child(title)))
                        .child(self.render_property_row("Type",
                            div()
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(kind_color)
                                .child(format!("({})", node_kind.display_name()))))
                        // Condition section (for condition nodes)
                        .when(node_kind == super::NodeKind::Condition, |_this| {
                            _this.child(self.render_section_header("Condition"))
                                .child(self.render_condition_builder())
                        })
                        // Actions section
                        .child(self.render_section_header("Actions (on_enter)"))
                        .child(self.render_action_list())
                        // Related section
                        .child(self.render_section_header("Related"))
                        .child(self.render_related_links())
                        // View toggle
                        .child(self.render_view_toggle())
                )
        } else {
            // No node selected
            div()
                .flex()
                .flex_col()
                .h_full()
                .bg(theme.colors.surface)
                .border_1()
                .border_color(theme.colors.border)
                .rounded_tl(px(4.0))
                .rounded_tr(px(4.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.0))
                        .py(px(8.0))
                        .bg(theme.colors.panel_header)
                        .border_b_1()
                        .border_color(theme.colors.border)
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.colors.text_secondary)
                                .child("Node Inspector")
                        )
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.colors.text_secondary)
                                .child("Select a node to inspect")
                        )
                )
        }
    }
}

impl RenderOnce for NodeInspector {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        // Render with no node selected - the caller should use render() directly
        div().child("NodeInspector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{GraphNode, NodeId, NodeKind};

    #[test]
    fn test_inspector_creation() {
        let inspector = NodeInspector::new(Arc::new(Theme::default_dark()));
        assert_eq!(inspector.view_mode, ViewMode::Visual);
        assert!(inspector.conditions.is_empty());
    }

    #[test]
    fn test_view_mode_toggle() {
        let mut inspector = NodeInspector::new(Arc::new(Theme::default_dark()));
        
        assert_eq!(inspector.view_mode, ViewMode::Visual);
        
        inspector.toggle_view_mode();
        assert_eq!(inspector.view_mode, ViewMode::Yaml);
        
        inspector.toggle_view_mode();
        assert_eq!(inspector.view_mode, ViewMode::Visual);
    }

    #[test]
    fn test_with_condition_node() {
        let node = GraphNode::new(NodeId::new(1), NodeKind::Condition, "Branch", (0.0, 0.0));
        let inspector = NodeInspector::new(Arc::new(Theme::default_dark()))
            .with_node(&node);
        
        assert!(!inspector.conditions.is_empty());
        assert!(!inspector.actions.is_empty());
    }

    #[test]
    fn test_condition_builder() {
        let cond1 = Condition::new("has_sword", "==", "true");
        assert_eq!(cond1.variable, "has_sword");
        assert!(cond1.is_and);
        
        let cond2 = Condition::new("gold", ">=", "100").with_or();
        assert!(!cond2.is_and);
    }

    #[test]
    fn test_action_creation() {
        let action = Action::new("🐉", "spawn dragon", "prefab: enemy_dragon");
        assert_eq!(action.icon, "🐉");
        assert_eq!(action.name, "spawn dragon");
        assert_eq!(action.detail, "prefab: enemy_dragon");
    }
}
