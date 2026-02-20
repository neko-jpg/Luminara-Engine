//! AI Assistant Component
//!
//! A chat-like interface for AI assistance with:
//! - Chat message history (user and assistant messages)
//! - Code block display with action buttons
//! - Input area with voice, attach, and context buttons

use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Styled, svg,
};
use gpui::prelude::FluentBuilder;
use std::sync::Arc;
use crate::ui::theme::Theme;

/// Role of a chat message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    /// Message from the AI assistant
    Assistant,
    /// Message from the user
    User,
}

/// A single chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Optional code block
    pub code_block: Option<CodeBlock>,
}

/// A code block within a message
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Language identifier
    pub language: String,
    /// Code content
    pub code: String,
}

impl ChatMessage {
    /// Create a new text message
    pub fn text(role: MessageRole, content: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
            code_block: None,
        }
    }
    
    /// Create a message with code block
    pub fn with_code(role: MessageRole, content: &str, language: &str, code: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
            code_block: Some(CodeBlock {
                language: language.to_string(),
                code: code.to_string(),
            }),
        }
    }
}

/// AI Assistant panel component
pub struct AIAssistant {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Chat messages
    messages: Vec<ChatMessage>,
    /// Current input value
    input_value: String,
}

impl AIAssistant {
    /// Create a new AI Assistant with demo messages
    pub fn new(theme: Arc<Theme>) -> Self {
        let messages = vec![
            ChatMessage::with_code(
                MessageRole::Assistant,
                "🤖 How can I help?\n// 例: プレイヤーの移動スクリプト",
                "rust",
                "// 例: プレイヤーの移動スクリプト"
            ),
            ChatMessage::text(
                MessageRole::User,
                "プレイヤーのジャンプ処理を書いて"
            ),
            ChatMessage::with_code(
                MessageRole::Assistant,
                "Rustのシステム例:",
                "rust",
                r#"#[system]
fn jump(input: Res<Input>, mut query: Query<&mut Transform, With<Player>>) {
    if input.just_pressed("space") {
        // ジャンプ力適用
    }
}"#
            ),
        ];
        
        Self {
            theme,
            messages,
            input_value: "ジャンプのベストプラクティス".to_string(),
        }
    }
    
    /// Add a new message
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }
    
    /// Set input value
    pub fn set_input_value(&mut self, value: String) {
        self.input_value = value;
    }
    
    /// Render the AI Assistant panel
    pub fn render(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        let messages = self.messages.clone();
        let input_value = self.input_value.clone();
        
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
                                    .path("icons/robot.svg")
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .text_color(theme.colors.accent)
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("AI Assistant")
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
            // Chat messages area
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p(theme.spacing.sm)
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .size_full()
                            .gap(theme.spacing.md)
                            .children(
                                messages.into_iter().map(|msg| {
                                    self.render_message(msg)
                                })
                            )
                    )
            )
            // Input area
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .p(theme.spacing.sm)
                    .gap(theme.spacing.xs)
                    // Input field
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .items_center()
                            .gap(theme.spacing.sm)
                            .bg(theme.colors.surface)
                            .rounded(theme.borders.rounded)
                            .border_1()
                            .border_color(theme.colors.border)
                            .px(theme.spacing.md)
                            .py(theme.spacing.sm)
                            .child(
                                div()
                                    .flex_1()
                                    .child(
                                        div()
                                            .text_color(theme.colors.text)
                                            .text_size(theme.typography.sm)
                                            .child(input_value)
                                    )
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .child(
                                        svg()
                                            .path("icons/send.svg")
                                            .w(px(16.0))
                                            .h(px(16.0))
                                            .text_color(theme.colors.accent)
                                    )
                            )
                    )
                    // Extra actions
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .items_center()
                            .gap(theme.spacing.lg)
                            .child(
                                self.render_action_button("icons/microphone.svg", "Voice")
                            )
                            .child(
                                self.render_action_button("icons/paperclip.svg", "Attach")
                            )
                            .child(
                                self.render_action_button("icons/database.svg", "Context")
                            )
                    )
            )
    }
    
    /// Render a chat message
    fn render_message(&self, message: ChatMessage) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_user = message.role == MessageRole::User;
        let bubble_color = if is_user {
            theme.colors.success.opacity(0.2)
        } else {
            theme.colors.accent.opacity(0.15)
        };
        
        div()
            .flex()
            .flex_col()
            .w_full()
            .items_end()
            .child(
                div()
                    .max_w(px(400.0))
                    .p(theme.spacing.md)
                    .rounded_l(theme.borders.lg)
                    .rounded_r(if is_user { theme.borders.xs } else { theme.borders.lg })
                    .bg(bubble_color)
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child(message.content.clone())
                    )
                    .when(message.code_block.is_some(), |this: gpui::Div| {
                        let code_block = message.code_block.clone().unwrap();
                        this.child(self.render_code_block(code_block))
                    })
            )
    }
    
    /// Render a code block
    fn render_code_block(&self, code_block: CodeBlock) -> impl IntoElement {
        let theme = self.theme.clone();
        let code = code_block.code.clone();
        
        div()
            .mt(theme.spacing.sm)
            .bg(theme.colors.canvas_background)
            .rounded(theme.borders.md)
            .border_l_3()
            .border_color(theme.colors.accent)
            .overflow_hidden()
            .child(
                div()
                    .p(theme.spacing.md)
                    .child(
                        div()
                            // Note: Font customization requires platform-specific setup in GPUI v0.159.5
                            .text_size(theme.typography.sm)
                            .text_color(theme.colors.text_secondary)
                            .child(code)
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .justify_end()
                    .gap(theme.spacing.sm)
                    .p(theme.spacing.sm)
                    .child(
                        self.render_code_action_button("Copy")
                    )
                    .child(
                        self.render_code_action_button("Apply")
                    )
                    .when(code_block.code.lines().count() > 5, |this| {
                        this.child(self.render_code_action_button("Edit"))
                    })
            )
    }
    
    /// Render a code action button
    fn render_code_action_button(&self, label: &str) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = label.to_string();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing.xs)
            .px(theme.spacing.md)
            .py(theme.spacing.xs)
            .rounded(theme.borders.rounded)
            .bg(theme.colors.surface)
            .hover(|this| this.bg(theme.colors.toolbar_active))
            .cursor_pointer()
            .child(
                svg()
                    .path(match label.as_str() {
                        "Copy" => "icons/copy.svg",
                        "Apply" => "icons/check.svg",
                        "Edit" => "icons/edit.svg",
                        _ => "icons/circle.svg",
                    })
                    .w(px(12.0))
                    .h(px(12.0))
                    .text_color(theme.colors.text_secondary)
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.xs)
                    .child(label)
            )
    }
    
    /// Render an action button (Voice, Attach, Context)
    fn render_action_button(&self, icon_path: &str, label: &str) -> impl IntoElement {
        let theme = self.theme.clone();
        let label = label.to_string();
        let icon_path = icon_path.to_string();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing.xs)
            .cursor_pointer()
            .child(
                svg()
                    .path(icon_path)
                    .w(px(12.0))
                    .h(px(12.0))
                    .text_color(theme.colors.text_secondary)
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.xs)
                    .child(label)
            )
    }
}
