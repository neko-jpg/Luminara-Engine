//! AI Assistant (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Data)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub code: Option<String>,
}

#[derive(Lens, Clone, Data)]
pub struct AIAssistantState {
    pub theme: Arc<Theme>,
    pub messages: Vec<ChatMessage>,
    pub input_text: String,
    pub is_processing: bool,
}

impl AIAssistantState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            messages: Vec::new(),
            input_text: String::new(),
            is_processing: false,
        }
    }

    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(ChatMessage {
            role,
            content,
            code: None,
        });
    }
}
