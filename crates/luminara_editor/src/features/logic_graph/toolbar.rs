//! Logic Graph Toolbar
//!
//! Toolbar for the Logic Graph editor with simulation controls,
//! tool selection (Select, Pan, Zoom), and layout options.

use crate::ui::theme::Theme;
use gpui::{
    div, px, IntoElement, ParentElement, RenderOnce, Styled, InteractiveElement,
    WindowContext,
};
use std::sync::Arc;

/// Available tools in the logic graph editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// Select and move nodes
    Select,
    /// Pan the canvas
    Pan,
    /// Zoom in/out
    Zoom,
    /// Auto layout nodes
    AutoLayout,
    /// Align nodes
    Align,
}

impl Tool {
    /// Get the icon character for this tool
    pub fn icon(&self) -> &'static str {
        match self {
            Tool::Select => "↔",
            Tool::Pan => "⟲",
            Tool::Zoom => "🔍",
            Tool::AutoLayout => "□",
            Tool::Align => "≡",
        }
    }

    /// Get the display name for this tool
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Select => "Select",
            Tool::Pan => "Pan",
            Tool::Zoom => "Zoom",
            Tool::AutoLayout => "Auto Layout",
            Tool::Align => "Align",
        }
    }
}

/// Simulation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    /// Simulation is stopped
    Stopped,
    /// Simulation is running
    Running,
}

/// Toolbar for the Logic Graph editor
#[derive(Debug, Clone)]
pub struct LogicGraphToolbar {
    /// Currently active tool
    pub active_tool: Tool,
    /// Current simulation state
    pub simulation: SimulationState,
    /// Theme for styling
    theme: Arc<Theme>,
}

impl LogicGraphToolbar {
    /// Create a new toolbar
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            active_tool: Tool::Select,
            simulation: SimulationState::Stopped,
            theme,
        }
    }

    /// Set the active tool
    pub fn set_tool(&mut self, tool: Tool) {
        self.active_tool = tool;
    }

    /// Toggle simulation state
    pub fn toggle_simulation(&mut self) {
        self.simulation = match self.simulation {
            SimulationState::Stopped => SimulationState::Running,
            SimulationState::Running => SimulationState::Stopped,
        };
    }

    /// Start simulation
    pub fn start_simulation(&mut self) {
        self.simulation = SimulationState::Running;
    }

    /// Stop simulation
    pub fn stop_simulation(&mut self) {
        self.simulation = SimulationState::Stopped;
    }

    /// Get all available tools
    pub fn tools() -> Vec<Tool> {
        vec![
            Tool::Select,
            Tool::Pan,
            Tool::Zoom,
            Tool::AutoLayout,
            Tool::Align,
        ]
    }

    /// Render the toolbar
    pub fn render(&mut self, _cx: &mut WindowContext) -> impl IntoElement {
        let theme = self.theme.clone();
        let active_tool = self.active_tool;
        let simulation = self.simulation;

        div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .w_full()
            .h(px(40.0))
            .px(px(16.0))
            .bg(theme.colors.toolbar_bg)
            .border_b_1()
            .border_color(theme.colors.border)
            // Simulation controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .bg(theme.colors.surface_active)
                    .rounded(px(6.0))
                    .px(px(4.0))
                    .py(px(2.0))
                    .child(
                        div()
                            .px(px(10.0))
                            .py(px(5.0))
                            .rounded(px(5.0))
                            .bg(if simulation == SimulationState::Running {
                                theme.colors.toolbar_active
                            } else {
                                theme.colors.surface_active
                            })
                            .text_color(if simulation == SimulationState::Running {
                                theme.colors.text
                            } else {
                                theme.colors.text_secondary
                            })
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("▶ Simulate")
                    )
                    .child(
                        div()
                            .px(px(8.0))
                            .py(px(5.0))
                            .rounded(px(5.0))
                            .text_color(theme.colors.text_secondary)
                            .hover(|style| style.bg(theme.colors.surface_hover))
                            .child("⏹")
                    )
            )
            // Separator
            .child(
                div()
                    .w(px(1.0))
                    .h(px(24.0))
                    .bg(theme.colors.border)
            )
            // Tool buttons
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .bg(theme.colors.surface_active)
                    .rounded(px(6.0))
                    .px(px(4.0))
                    .py(px(2.0))
                    .children(
                        Self::tools().into_iter().map(move |tool| {
                            let is_active = active_tool == tool;
                            let theme = theme.clone();
                            
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .px(px(10.0))
                                .py(px(5.0))
                                .rounded(px(5.0))
                                .bg(if is_active {
                                    theme.colors.toolbar_active
                                } else {
                                    theme.colors.surface_active
                                })
                                .text_color(if is_active {
                                    theme.colors.text
                                } else {
                                    theme.colors.text_secondary
                                })
                                .hover(|this| this.bg(theme.colors.surface_hover))
                                .child(tool.icon())
                                .child(tool.name())
                        })
                    )
            )
    }
}

impl RenderOnce for LogicGraphToolbar {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let mut toolbar = self;
        LogicGraphToolbar::render(&mut toolbar, cx)
    }
}

/// Status bar information
#[derive(Debug, Clone)]
pub struct StatusBarInfo {
    pub node_count: usize,
    pub edge_count: usize,
    pub graph_name: String,
    pub database: String,
    pub ai_status: String,
}

impl StatusBarInfo {
    /// Create new status bar info
    pub fn new(node_count: usize, edge_count: usize, graph_name: impl Into<String>) -> Self {
        Self {
            node_count,
            edge_count,
            graph_name: graph_name.into(),
            database: "surreal: logic_graph".to_string(),
            ai_status: "AI ready".to_string(),
        }
    }

    /// Render the status bar
    pub fn render(&self, theme: Arc<Theme>) -> impl IntoElement {
        let theme2 = theme.clone();
        let theme3 = theme.clone();
        let theme4 = theme.clone();

        div()
            .flex()
            .items_center()
            .gap(px(16.0))
            .ml_auto()
            .px(px(12.0))
            .py(px(4.0))
            .bg(theme.colors.surface)
            .rounded(px(20.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_color(theme.colors.text_secondary)
                    .text_size(px(12.0))
                    .child("◈")
                    .child(format!("{} nodes, {} edges", self.node_count, self.edge_count))
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_color(theme2.colors.text_secondary)
                    .text_size(px(12.0))
                    .child("⎇")
                    .child(self.graph_name.clone())
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_color(theme3.colors.text_secondary)
                    .text_size(px(12.0))
                    .child("🗄")
                    .child(self.database.clone())
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_color(theme4.colors.accent)
                    .text_size(px(12.0))
                    .child("🤖")
                    .child(self.ai_status.clone())
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tools = LogicGraphToolbar::tools();
        assert_eq!(tools.len(), 5);
        assert!(tools.contains(&Tool::Select));
        assert!(tools.contains(&Tool::Pan));
    }

    #[test]
    fn test_tool_properties() {
        assert_eq!(Tool::Select.name(), "Select");
        assert_eq!(Tool::Pan.name(), "Pan");
        assert_eq!(Tool::Zoom.name(), "Zoom");
        
        assert!(!Tool::Select.icon().is_empty());
        assert!(!Tool::Pan.icon().is_empty());
    }

    #[test]
    fn test_simulation_state() {
        let mut toolbar = LogicGraphToolbar::new(Arc::new(Theme::default_dark()));
        
        assert_eq!(toolbar.simulation, SimulationState::Stopped);
        
        toolbar.start_simulation();
        assert_eq!(toolbar.simulation, SimulationState::Running);
        
        toolbar.stop_simulation();
        assert_eq!(toolbar.simulation, SimulationState::Stopped);
        
        toolbar.start_simulation();
        toolbar.toggle_simulation();
        assert_eq!(toolbar.simulation, SimulationState::Stopped);
    }

    #[test]
    fn test_tool_selection() {
        let mut toolbar = LogicGraphToolbar::new(Arc::new(Theme::default_dark()));
        
        assert_eq!(toolbar.active_tool, Tool::Select);
        
        toolbar.set_tool(Tool::Pan);
        assert_eq!(toolbar.active_tool, Tool::Pan);
        
        toolbar.set_tool(Tool::Zoom);
        assert_eq!(toolbar.active_tool, Tool::Zoom);
    }

    #[test]
    fn test_status_bar_info() {
        let info = StatusBarInfo::new(8, 7, "Main Quest");
        
        assert_eq!(info.node_count, 8);
        assert_eq!(info.edge_count, 7);
        assert_eq!(info.graph_name, "Main Quest");
        assert_eq!(info.database, "surreal: logic_graph");
        assert_eq!(info.ai_status, "AI ready");
    }
}
