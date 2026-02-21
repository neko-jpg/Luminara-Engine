//! Bottom Tab Panel Component
//!
//! Bottom panel with Console, Assets, DB Query, and AI Assistant tabs

use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, ViewContext,
    InteractiveElement, MouseButton, MouseDownEvent, ClickEvent, prelude::FluentBuilder,
};
use std::sync::Arc;
use crate::ui::theme::Theme;
use crate::ui::components::{Button, TextInput, ButtonVariant};
use crate::core::state::EditorStateManager;

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
    pub path: String,
    pub icon: String,
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Model,
    Texture,
    Material,
    Script,
    Scene,
    Other,
}

impl AssetType {
    pub fn icon(&self) -> &'static str {
        match self {
            AssetType::Model => "◆",
            AssetType::Texture => "🖼",
            AssetType::Material => "◈",
            AssetType::Script => "📜",
            AssetType::Scene => "🎬",
            AssetType::Other => "📄",
        }
    }
}

/// Chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub is_user: bool,
    pub text: String,
    pub timestamp: String,
}

/// DB Query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Bottom tab panel component
pub struct BottomTabPanel {
    theme: Arc<Theme>,
    state: Option<gpui::Model<EditorStateManager>>,
    logs: Vec<LogEntry>,
    assets: Vec<AssetItem>,
    filtered_assets: Vec<AssetItem>,
    asset_search_text: String,
    // DB Query state
    query_text: String,
    query_results: Option<QueryResult>,
    is_query_executing: bool,
    query_history: Vec<String>,
    // AI Assistant state
    chat_messages: Vec<ChatMessage>,
    chat_input: String,
    is_ai_processing: bool,
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
            AssetItem { name: "Player.fbx".to_string(), path: "models/Player.fbx".to_string(), icon: "◆".to_string(), asset_type: AssetType::Model },
            AssetItem { name: "Enemy.fbx".to_string(), path: "models/Enemy.fbx".to_string(), icon: "◆".to_string(), asset_type: AssetType::Model },
            AssetItem { name: "Terrain.png".to_string(), path: "textures/Terrain.png".to_string(), icon: "🖼".to_string(), asset_type: AssetType::Texture },
            AssetItem { name: "Main.mat".to_string(), path: "materials/Main.mat".to_string(), icon: "◈".to_string(), asset_type: AssetType::Material },
            AssetItem { name: "GameLogic.rs".to_string(), path: "scripts/GameLogic.rs".to_string(), icon: "📜".to_string(), asset_type: AssetType::Script },
            AssetItem { name: "MainScene.scene".to_string(), path: "scenes/MainScene.scene".to_string(), icon: "🎬".to_string(), asset_type: AssetType::Scene },
        ];

        let chat_messages = vec![
            ChatMessage {
                is_user: false,
                text: "Hello! I'm your AI assistant. How can I help you today?".to_string(),
                timestamp: "10:00".to_string(),
            },
        ];

        let filtered_assets = assets.clone();

        Self {
            theme,
            state: None,
            logs,
            assets,
            filtered_assets,
            asset_search_text: String::new(),
            query_text: "SELECT * FROM entities WHERE name CONTAINS 'Player';".to_string(),
            query_results: None,
            is_query_executing: false,
            query_history: vec![
                "SELECT * FROM entities;".to_string(),
                "SELECT * FROM components WHERE type = 'Transform';".to_string(),
            ],
            chat_messages,
            chat_input: String::new(),
            is_ai_processing: false,
        }
    }

    /// Set state model
    pub fn with_state(mut self, state: gpui::Model<EditorStateManager>, cx: &mut ViewContext<Self>) -> Self {
        cx.observe(&state, |_this: &mut BottomTabPanel, _model, cx| {
            cx.notify();
        }).detach();

        self.state = Some(state);
        self
    }

    /// Get active tab
    pub fn active_tab(&self, cx: &gpui::AppContext) -> BottomTab {
        if let Some(state) = &self.state {
            let active = state.read(cx).session.active_bottom_tab.clone();
            match active.as_str() {
                "Console" => BottomTab::Console,
                "Asset Browser" => BottomTab::Assets,
                "DB Query" => BottomTab::DBQuery,
                "AI Assistant" => BottomTab::AIAssistant,
                _ => BottomTab::Console,
            }
        } else {
            BottomTab::Console
        }
    }

    /// Add a log entry
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.logs.push(LogEntry {
            timestamp,
            level,
            message,
        });
        // Keep only last 1000 logs
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    /// Clear logs
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    /// Filter assets based on search text
    fn filter_assets(&mut self) {
        if self.asset_search_text.is_empty() {
            self.filtered_assets = self.assets.clone();
        } else {
            let search_lower = self.asset_search_text.to_lowercase();
            self.filtered_assets = self.assets
                .iter()
                .filter(|asset| {
                    asset.name.to_lowercase().contains(&search_lower) ||
                    asset.path.to_lowercase().contains(&search_lower)
                })
                .cloned()
                .collect();
        }
    }

    /// Execute DB query
    fn execute_query(&mut self, cx: &mut ViewContext<Self>) {
        self.is_query_executing = true;
        cx.notify();

        // Simulate async query execution
        let query = self.query_text.clone();
        
        // Add to history if not already there
        if !self.query_history.contains(&query) {
            self.query_history.push(query.clone());
        }

        // Mock query execution - in real implementation, this would query SurrealDB
        cx.spawn(|this, mut cx| async move {
            // Simulate network delay
            cx.background_executor().timer(std::time::Duration::from_millis(500)).await;
            
            this.update(&mut cx, |panel, cx| {
                panel.is_query_executing = false;
                
                // Mock result based on query
                if query.to_uppercase().contains("SELECT") {
                    panel.query_results = Some(QueryResult {
                        columns: vec!["id".to_string(), "name".to_string(), "type".to_string()],
                        rows: vec![
                            vec!["entity:1".to_string(), "Player".to_string(), "Character".to_string()],
                            vec!["entity:2".to_string(), "Enemy".to_string(), "NPC".to_string()],
                            vec!["entity:3".to_string(), "Camera".to_string(), "Camera".to_string()],
                        ],
                    });
                } else {
                    panel.query_results = Some(QueryResult {
                        columns: vec!["result".to_string()],
                        rows: vec![vec!["Query executed successfully".to_string()]],
                    });
                }
                
                cx.notify();
            }).ok();
        }).detach();
    }

    /// Clear query results
    fn clear_query(&mut self) {
        self.query_results = None;
    }

    /// Send AI message
    fn send_ai_message(&mut self, cx: &mut ViewContext<Self>) {
        if self.chat_input.trim().is_empty() {
            return;
        }

        let user_message = self.chat_input.clone();
        let timestamp = chrono::Local::now().format("%H:%M").to_string();
        
        // Add user message
        self.chat_messages.push(ChatMessage {
            is_user: true,
            text: user_message.clone(),
            timestamp: timestamp.clone(),
        });
        
        self.chat_input.clear();
        self.is_ai_processing = true;
        cx.notify();

        // Simulate AI response
        cx.spawn(|this, mut cx| async move {
            // Simulate processing time
            cx.background_executor().timer(std::time::Duration::from_millis(1000)).await;
            
            this.update(&mut cx, |panel, cx| {
                panel.is_ai_processing = false;
                
                // Generate mock response based on user input
                let response = if user_message.to_lowercase().contains("create") || 
                                  user_message.to_lowercase().contains("spawn") {
                    "I'll help you create that entity. You can use the Hierarchy panel to add a new GameObject, or I can generate a script for you. Would you like me to create a procedural mesh or use a prefab?"
                } else if user_message.to_lowercase().contains("query") || 
                          user_message.to_lowercase().contains("database") {
                    "You can use the DB Query tab to run SurrealQL queries. Try: `SELECT * FROM entities WHERE name CONTAINS 'Player'`"
                } else if user_message.to_lowercase().contains("help") {
                    "I can help you with:\n- Creating and editing entities\n- Writing SurrealDB queries\n- Generating procedural content\n- Explaining engine features"
                } else {
                    "I understand. Let me know if you need help with scene editing, database queries, or any other aspect of the Luminara Engine."
                };
                
                let response_timestamp = chrono::Local::now().format("%H:%M").to_string();
                panel.chat_messages.push(ChatMessage {
                    is_user: false,
                    text: response.to_string(),
                    timestamp: response_timestamp,
                });
                
                cx.notify();
            }).ok();
        }).detach();
    }

    /// Render tab button
    fn render_tab(&self, tab: BottomTab, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_active = self.active_tab(cx) == tab;
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
                if let Some(state) = &this.state {
                    state.update(cx, |state, cx| {
                        state.set_active_bottom_tab(tab_clone.label().to_string(), cx);
                    });
                }
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
    fn render_assets_content(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let search_text = self.asset_search_text.clone();

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
                        TextInput::new("asset_search")
                            .placeholder("Search assets...")
                            .value(search_text)
                            .on_change(cx.listener(|this, text: &str, _cx| {
                                this.asset_search_text = text.to_string();
                                this.filter_assets();
                            }))
                    )
                    .child(
                        // Filter dropdown placeholder
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
                                    .child("All Types")
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
                    .children(self.filtered_assets.iter().map(|asset| {
                        let theme = theme.clone();
                        let asset = asset.clone();
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
                                    .child(asset.asset_type.icon())
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
    fn render_db_query_content(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let query_text = self.query_text.clone();
        let is_executing = self.is_query_executing;

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
                        TextInput::new("query_input")
                            .value(query_text)
                            .on_change(cx.listener(|this, text: &str, _cx| {
                                this.query_text = text.to_string();
                            }))
                    )
            )
            .child(
                // Query history dropdown
                div()
                    .flex()
                    .flex_row()
                    .gap(theme.spacing.sm)
                    .child(
                        div()
                            .px(theme.spacing.sm)
                            .py(theme.spacing.xs)
                            .bg(theme.colors.surface_hover)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.xs)
                                    .child("History")
                            )
                    )
                    .children(self.query_history.iter().take(3).map(|history_item| {
                        let item = history_item.clone();
                        let theme = theme.clone();
                        div()
                            .px(theme.spacing.sm)
                            .py(theme.spacing.xs)
                            .bg(theme.colors.surface)
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded(theme.borders.xs)
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.colors.surface_hover))
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event: &MouseDownEvent, cx| {
                                this.query_text = item.clone();
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.xs)
                                    .max_w(px(150.0))
                                    .child(history_item.chars().take(20).collect::<String>() + "...")
                            )
                    }))
            )
            .child(
                // Action buttons
                div()
                    .flex()
                    .flex_row()
                    .gap(theme.spacing.sm)
                    .child(
                        Button::new("execute_query", if is_executing { "Executing..." } else { "Execute" })
                            .variant(ButtonVariant::Primary)
                            .disabled(is_executing)
                            .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                this.execute_query(cx);
                            }))
                    )
                    .child(
                        Button::new("clear_query", "Clear")
                            .variant(ButtonVariant::Secondary)
                            .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                this.clear_query();
                                cx.notify();
                            }))
                    )
            )
            .child(
                // Query results
                if let Some(results) = &self.query_results {
                    div()
                        .flex_1()
                        .bg(theme.colors.background)
                        .border_1()
                        .border_color(theme.colors.border)
                        .rounded(theme.borders.xs)
                        .overflow_hidden()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .bg(theme.colors.surface_hover)
                                .border_b_1()
                                .border_color(theme.colors.border)
                                .children(results.columns.iter().map(|col| {
                                    let theme = theme.clone();
                                    div()
                                        .flex_1()
                                        .px(theme.spacing.sm)
                                        .py(theme.spacing.xs)
                                        .child(
                                            div()
                                                .text_color(theme.colors.text)
                                                .text_size(theme.typography.sm)
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .child(col.clone())
                                        )
                                }))
                        )
                        .children(results.rows.iter().map(|row| {
                            let theme = theme.clone();
                            div()
                                .flex()
                                .flex_row()
                                .border_b_1()
                                .border_color(theme.colors.border)
                                .children(row.iter().map(|cell| {
                                    let theme = theme.clone();
                                    div()
                                        .flex_1()
                                        .px(theme.spacing.sm)
                                        .py(theme.spacing.xs)
                                        .child(
                                            div()
                                                .text_color(theme.colors.text)
                                                .text_size(theme.typography.sm)
                                                .child(cell.clone())
                                        )
                                }))
                        }))
                        .into_any_element()
                } else {
                    div()
                        .text_color(theme.colors.text_secondary)
                        .text_size(theme.typography.xs)
                        .child("Query results will appear here...")
                        .into_any_element()
                }
            )
    }

    /// Render AI assistant content
    fn render_ai_assistant_content(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let chat_input = self.chat_input.clone();
        let is_processing = self.is_ai_processing;

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
                            .when(!is_user, |this| this.child(div().flex_1())) // Spacer for AI messages
                            .child(
                                div()
                                    .max_w(px(400.0))
                                    .p(theme.spacing.md)
                                    .rounded(theme.borders.md)
                                    .bg(if is_user { theme.colors.accent } else { theme.colors.surface_hover })
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_color(if is_user { theme.colors.background } else { theme.colors.text })
                                                    .text_size(theme.typography.sm)
                                                    .child(msg.text.clone())
                                            )
                                            .child(
                                                div()
                                                    .mt(theme.spacing.xs)
                                                    .text_color(if is_user { theme.colors.background.opacity(0.7) } else { theme.colors.text_secondary })
                                                    .text_size(theme.typography.xs)
                                                    .child(msg.timestamp.clone())
                                            )
                                    )
                            )
                            .when(is_user, |this| this.child(div().flex_1())) // Spacer for user messages
                    }))
                    .when(is_processing, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_row()
                                .w_full()
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .p(theme.spacing.md)
                                        .rounded(theme.borders.md)
                                        .bg(theme.colors.surface_hover)
                                        .child(
                                            div()
                                                .text_color(theme.colors.text_secondary)
                                                .text_size(theme.typography.sm)
                                                .child("AI is thinking...")
                                        )
                                )
                        )
                    })
            )
            .child(
                // Input area
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .gap(theme.spacing.sm)
                    .child(
                        TextInput::new("chat_input")
                            .placeholder("Ask the AI assistant...")
                            .value(chat_input)
                            .on_change(cx.listener(|this, text: &str, _cx| {
                                this.chat_input = text.to_string();
                            }))
                    )
                    .child(
                        Button::new("send_message", "Send")
                            .variant(ButtonVariant::Primary)
                            .disabled(is_processing || self.chat_input.trim().is_empty())
                            .on_click(cx.listener(|this, _event: &ClickEvent, cx| {
                                this.send_ai_message(cx);
                            }))
                    )
            )
    }

    /// Render active tab content
    fn render_tab_content(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let active_tab = self.active_tab(cx);
        match active_tab {
            BottomTab::Console => self.render_console_content().into_any_element(),
            BottomTab::Assets => self.render_assets_content(cx).into_any_element(),
            BottomTab::DBQuery => self.render_db_query_content(cx).into_any_element(),
            BottomTab::AIAssistant => self.render_ai_assistant_content(cx).into_any_element(),
        }
    }
}

impl Render for BottomTabPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
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
                    .child(self.render_tab_content(cx))
            )
    }
}
