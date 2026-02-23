//! AI Agent Bottom Tabs (Vizia version)

use crate::ui::theme::Theme;
use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum BottomTab {
    Console,
    DbExplorer,
    BuildOutput,
    Diagnostics,
}

#[derive(Lens, Clone, Data)]
pub struct BottomTabPanelState {
    pub theme: Arc<Theme>,
    pub active_tab: BottomTab,
}

impl BottomTabPanelState {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            active_tab: BottomTab::Console,
        }
    }
}
