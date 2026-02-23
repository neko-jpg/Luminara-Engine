//! Settings Category (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SettingsCategoryItem {
    pub name: String,
    pub icon: String,
    pub category_id: u32,
}

impl SettingsCategoryItem {
    pub fn new(name: &str, icon: &str, category_id: u32) -> Self {
        Self {
            name: name.to_string(),
            icon: icon.to_string(),
            category_id,
        }
    }
}
