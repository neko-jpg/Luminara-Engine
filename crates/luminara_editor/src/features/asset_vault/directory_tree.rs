//! Directory Tree (Vizia v0.3)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<DirectoryEntry>,
}

#[derive(Clone)]
pub struct DirectoryTreeState {
    pub theme: Arc<Theme>,
    pub root: Option<DirectoryEntry>,
}

impl DirectoryTreeState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self { theme, root: None }
    }

    pub fn build(&mut self, cx: &mut Context) {
        let surface = self.theme.colors.surface;

        VStack::new(cx, |_cx| {})
            .width(Pixels(200.0))
            .height(Stretch(1.0))
            .background_color(surface);
    }
}
