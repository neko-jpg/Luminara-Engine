//! Settings Content Components (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;

pub struct TextInput {
    pub value: String,
}

impl TextInput {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

pub struct SettingsContent {
    pub theme: Arc<Theme>,
}
