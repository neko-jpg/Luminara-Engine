//! Backend & AI Box Component
//!
//! The main container for the Backend & AI interface with:
//! - Toolbar with Run button, Mode selector, and Status bar
//! - 2-column layout: Script Editor (left) + AI Assistant (right)
//! - Bottom tab panel: Console, DB Explorer, Build Output, Diagnostics

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::services::ai_agent::{
    Toolbar, ScriptEditor, AIAssistant, BottomTabPanel, FileTree,
};

/// The Backend & AI Box component
///
/// This is the main interface for:
/// - Script editing (Rust/WASM)
/// - Database query editing
/// - AI-assisted code generation
///
/// Layout:
/// ```
/// ┌─────────────────────────────────────────────────────────────────────────┐
/// │ Toolbar (Run, Mode, Status)                                             │
/// ├──────────────┬─────────────────────────┬────────────────────────────────┤
/// │              │                         │                                │
/// │   File Tree  │   Script Editor         │   AI Assistant                 │
/// │   (Explorer) │   (Tabbed code editor)  │   (Chat interface)             │
/// │              │                         │                                │
/// ├──────────────┴─────────────────────────┴────────────────────────────────┤
/// │ Bottom Tab Panel (Console/DB/Build/Diagnostics)                         │
/// └─────────────────────────────────────────────────────────────────────────┘
/// ```
pub struct BackendAIBox {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Toolbar component
    toolbar: Toolbar,
    /// File Tree component
    file_tree: FileTree,
    /// Script Editor component
    script_editor: ScriptEditor,
    /// AI Assistant component
    ai_assistant: AIAssistant,
    /// Bottom Tab Panel component
    bottom_tabs: BottomTabPanel,
}

impl BackendAIBox {
    /// Create a new Backend & AI Box
    ///
    /// # Arguments
    /// * `theme` - Arc-wrapped theme for styling
    pub fn new(theme: Arc<Theme>) -> Self {
        let toolbar = Toolbar::new(theme.clone());
        let file_tree = FileTree::new(theme.clone());
        let script_editor = ScriptEditor::new(theme.clone());
        let ai_assistant = AIAssistant::new(theme.clone());
        let bottom_tabs = BottomTabPanel::new(theme.clone());
        
        Self {
            theme,
            toolbar,
            file_tree,
            script_editor,
            ai_assistant,
            bottom_tabs,
        }
    }
    
    /// Create with default dark theme
    pub fn default_dark() -> Self {
        Self::new(Arc::new(Theme::default_dark()))
    }
}

impl Render for BackendAIBox {
    /// Render the Backend & AI Box
    ///
    /// Layout structure:
    /// - Toolbar at the top
    /// - Main area: 2-column layout (Script Editor left, AI Assistant right)
    /// - Bottom tab panel at the bottom
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.background)
            // Toolbar
            .child(self.toolbar.render())
            // Main content area: 3-column layout (File Tree | Script Editor | AI Assistant)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .gap(px(4.0))
                    .p(px(4.0))
                    // Left: File Tree (220px fixed width)
                    .child(
                        div()
                            .flex()
                            .w(px(220.0))
                            .h_full()
                            .child(self.file_tree.render())
                    )
                    // Center: Script Editor (flexible)
                    .child(
                        div()
                            .flex()
                            .flex_1() // Flexible width
                            .h_full()
                            .child(self.script_editor.render())
                    )
                    // Right: AI Assistant (320px fixed width)
                    .child(
                        div()
                            .flex()
                            .w(px(320.0))
                            .h_full()
                            .child(self.ai_assistant.render())
                    )
            )
            // Bottom tab panel
            .child(self.bottom_tabs.render())
    }
}

// Note: For a 1.5:1 flex ratio between Script Editor and AI Assistant,
// we use flex_1() for both and rely on the fixed width (320px) of the AI Assistant panel
