//! Bottom Tab Panel for Backend & AI Box
//!
//! Contains tabs for:
//! - Console: Script output and logs
//! - DB Explorer: Database table browser
//! - Build Output: Compilation and build status
//! - Diagnostics: Errors and warnings

use gpui::{
    div, px, IntoElement, ParentElement, Styled, svg, InteractiveElement,
};
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Bottom tab types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomTab {
    /// Console output
    Console,
    /// Database explorer
    DBExplorer,
    /// Build output
    BuildOutput,
    /// Diagnostics (errors/warnings)
    Diagnostics,
}

impl BottomTab {
    /// Get the display label for the tab
    pub fn label(&self) -> &'static str {
        match self {
            BottomTab::Console => "Console",
            BottomTab::DBExplorer => "DB Explorer",
            BottomTab::BuildOutput => "Build Output",
            BottomTab::Diagnostics => "Diagnostics",
        }
    }
    
    /// Get the icon path for the tab
    pub fn icon(&self) -> &'static str {
        match self {
            BottomTab::Console => "icons/terminal.svg",
            BottomTab::DBExplorer => "icons/database.svg",
            BottomTab::BuildOutput => "icons/hammer.svg",
            BottomTab::Diagnostics => "icons/alert-triangle.svg",
        }
    }
}

/// Bottom tab panel component
pub struct BottomTabPanel {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Currently active tab
    active_tab: BottomTab,
    /// Console lines
    console_lines: Vec<ConsoleLine>,
    /// DB items
    db_items: Vec<DBItem>,
    /// Build progress (0-100)
    build_progress: u32,
    /// Build steps
    build_steps: Vec<BuildStep>,
    /// Diagnostic items
    diagnostics: Vec<DiagnosticItem>,
}

/// A single console line
#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub content: String,
    pub level: ConsoleLevel,
}

/// Console line level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Info,
    Success,
    Error,
}

/// Database item
#[derive(Debug, Clone)]
pub struct DBItem {
    pub name: String,
    pub count: u32,
}



/// Build step
#[derive(Debug, Clone)]
pub struct BuildStep {
    pub message: String,
    pub status: BuildStatus,
}

/// Build status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStatus {
    Pending,
    InProgress,
    Success,
    Warning,
}

/// Diagnostic item
#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub message: String,
    pub level: DiagnosticLevel,
}

/// Diagnostic level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

impl BottomTabPanel {
    /// Create a new Bottom Tab Panel with demo data
    pub fn new(theme: Arc<Theme>) -> Self {
        let console_lines = vec![
            ConsoleLine { content: "> Compiled player.rs → player.wasm (12ms)".to_string(), level: ConsoleLevel::Success },
            ConsoleLine { content: "> Hot-reloaded 1 system".to_string(), level: ConsoleLevel::Success },
            ConsoleLine { content: "> [Error] physics: invalid mesh collision (enemy.obj)".to_string(), level: ConsoleLevel::Error },
            ConsoleLine { content: "> SurrealQL: SELECT * FROM entity RETURN 24 rows (3ms)".to_string(), level: ConsoleLevel::Info },
        ];
        
        let db_items = vec![
            DBItem { name: "entity".to_string(), count: 243 },
            DBItem { name: "component".to_string(), count: 512 },
            DBItem { name: "scene".to_string(), count: 8 },
            DBItem { name: "asset".to_string(), count: 1024 },
            DBItem { name: "logic_node".to_string(), count: 67 },
            DBItem { name: "logic_edge".to_string(), count: 98 },
            DBItem { name: "timeline".to_string(), count: 4 },
            DBItem { name: "game_var".to_string(), count: 23 },
        ];
        
        let build_steps = vec![
            BuildStep { message: "Building for Web (WASM) ...".to_string(), status: BuildStatus::Success },
            BuildStep { message: "✔ Compiling player.rs".to_string(), status: BuildStatus::Success },
            BuildStep { message: "✔ Compiling enemy.rs".to_string(), status: BuildStatus::Success },
            BuildStep { message: "⚡ wasm-opt: 32% improvement".to_string(), status: BuildStatus::Warning },
            BuildStep { message: "📦 Bundle: dist/ (2.4 MB)".to_string(), status: BuildStatus::Success },
            BuildStep { message: "✅ Build succeeded in 1.2s".to_string(), status: BuildStatus::Success },
        ];
        
        let diagnostics = vec![
            DiagnosticItem { 
                message: "⛔ enemy.rs:24:9: type mismatch: expected f32, found i32".to_string(), 
                level: DiagnosticLevel::Error 
            },
            DiagnosticItem { 
                message: "⚠️ query.sql:3: unused SELECT *".to_string(), 
                level: DiagnosticLevel::Warning 
            },
            DiagnosticItem { 
                message: "⛔ physics.rs:10:22: unresolved import `collision`".to_string(), 
                level: DiagnosticLevel::Error 
            },
        ];
        
        Self {
            theme,
            active_tab: BottomTab::Console,
            console_lines,
            db_items,
            build_progress: 45,
            build_steps,
            diagnostics,
        }
    }
    
    /// Set the active tab
    pub fn set_active_tab(&mut self, tab: BottomTab) {
        self.active_tab = tab;
    }
    
    /// Get the active tab
    pub fn active_tab(&self) -> BottomTab {
        self.active_tab
    }
    
    /// Render the bottom tab panel
    pub fn render(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let active_tab = self.active_tab;
        
        div()
            .flex()
            .flex_col()
            .w_full()
            .h(px(200.0))
            .min_h(px(160.0))
            .max_h(px(260.0))
            .bg(theme.colors.surface)
            .border_t_1()
            .border_color(theme.colors.border)
            // Tab header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .h(px(32.0))
                    .bg(theme.colors.panel_header)
                    .border_b_1()
                    .border_color(theme.colors.border)
                    .px(theme.spacing.sm)
                    .gap(px(4.0))
                    .items_center()
                    .child(self.render_tab_button(BottomTab::Console, active_tab == BottomTab::Console))
                    .child(self.render_tab_button(BottomTab::DBExplorer, active_tab == BottomTab::DBExplorer))
                    .child(self.render_tab_button(BottomTab::BuildOutput, active_tab == BottomTab::BuildOutput))
                    .child(self.render_tab_button(BottomTab::Diagnostics, active_tab == BottomTab::Diagnostics))
            )
            // Tab content
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p(theme.spacing.sm)
                    .overflow_hidden()
                    .child(self.render_tab_content(active_tab))
            )
    }
    
    /// Render a tab button
    fn render_tab_button(&self, tab: BottomTab, is_active: bool) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing.xs)
            .px(theme.spacing.md)
            .py(theme.spacing.sm)
            .border_b_2()
            .border_color(if is_active {
                theme.colors.accent
            } else {
                gpui::transparent_black()
            })
            .cursor_pointer()
            .child(
                svg()
                    .path(tab.icon())
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
                        theme.colors.accent
                    } else {
                        theme.colors.text_secondary
                    })
                    .text_size(theme.typography.sm)
                    .child(tab.label())
            )
    }
    
    /// Render the content for the active tab
    fn render_tab_content(&self, tab: BottomTab) -> gpui::AnyElement {
        match tab {
            BottomTab::Console => self.render_console_content().into_any_element(),
            BottomTab::DBExplorer => self.render_db_explorer_content().into_any_element(),
            BottomTab::BuildOutput => self.render_build_output_content().into_any_element(),
            BottomTab::Diagnostics => self.render_diagnostics_content().into_any_element(),
        }
    }
    
    /// Render console tab content
    fn render_console_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let lines = self.console_lines.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .gap(theme.spacing.xs)
            .children(
                lines.into_iter().map(|line| {
                    let color = match line.level {
                        ConsoleLevel::Info => theme.colors.text_secondary,
                        ConsoleLevel::Success => theme.colors.success,
                        ConsoleLevel::Error => theme.colors.error,
                    };
                    
                    div()
                        // Note: Font customization requires platform-specific setup in GPUI v0.159.5
                        .text_size(theme.typography.sm)
                        .text_color(color)
                        .child(line.content)
                })
            )
    }
    
    /// Render DB explorer tab content
    fn render_db_explorer_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let items = self.db_items.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .gap(theme.spacing.sm)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(theme.spacing.xs)
                    .children(
                        items.into_iter().map(|item| {
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .bg(theme.colors.surface_hover)
                                .rounded(theme.borders.xs)
                                .px(theme.spacing.sm)
                                .py(theme.spacing.xs)
                                .cursor_pointer()
                                .hover(|this| this.bg(theme.colors.toolbar_active))
                                .child(
                                    div()
                                        .text_color(theme.colors.text)
                                        .text_size(theme.typography.sm)
                                        .child(format!("{} ({})", item.name, item.count))
                                )
                        })
                    )
            )
            .child(
                div()
                    .mt(theme.spacing.sm)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.xs)
                    .cursor_pointer()
                    .child(
                        svg()
                            .path("icons/table.svg")
                            .w(px(12.0))
                            .h(px(12.0))
                            .text_color(theme.colors.text_secondary)
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child("SELECT * FROM entity LIMIT 50 (click to run)")
                    )
            )
    }
    
    /// Render build output tab content
    fn render_build_output_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let steps = self.build_steps.clone();
        let progress = self.build_progress;
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .gap(theme.spacing.sm)
            // Progress bar
            .child(
                div()
                    .w_full()
                    .h(px(20.0))
                    .bg(theme.colors.surface_hover)
                    .rounded(theme.borders.rounded)
                    .overflow_hidden()
                    .child(
                        div()
                            .w(px(progress as f32 * 2.0))
                            .h_full()
                            .bg(theme.colors.accent)
                    )
            )
            // Build steps
            .children(
                steps.into_iter().map(|step| {
                    let color = match step.status {
                        BuildStatus::Pending => theme.colors.text_secondary,
                        BuildStatus::InProgress => theme.colors.accent,
                        BuildStatus::Success => theme.colors.success,
                        BuildStatus::Warning => theme.colors.warning,
                    };
                    
                    div()
                        .text_color(color)
                        .text_size(theme.typography.sm)
                        .child(step.message)
                })
            )
    }
    
    /// Render diagnostics tab content
    fn render_diagnostics_content(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let diagnostics = self.diagnostics.clone();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .gap(theme.spacing.xs)
            .children(
                diagnostics.into_iter().map(|diag| {
                    let color = match diag.level {
                        DiagnosticLevel::Error => theme.colors.error,
                        DiagnosticLevel::Warning => theme.colors.warning,
                    };
                    
                    div()
                        .text_color(color)
                        .text_size(theme.typography.sm)
                        .child(diag.message)
                })
            )
            .child(
                div()
                    .mt(theme.spacing.sm)
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("2 errors, 1 warning")
            )
    }
}
