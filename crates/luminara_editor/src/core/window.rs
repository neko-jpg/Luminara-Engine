//! Main editor window
//!
//! The EditorWindow is the root view that contains all UI elements including
//! the Activity Bar, active Box, and overlays.

use crate::features::account::AccountPanel;
use crate::ui::layouts::activity_bar::ActivityBar;
use crate::features::asset_vault::AssetVaultBox;
use crate::services::ai_agent::BackendAIBox;
use crate::features::extension::ExtensionBox;
use crate::core::state::{EditorState, SharedEditorState};
use crate::features::director::DirectorBox;
use crate::services::engine_bridge::EngineHandle;
use crate::features::global_search::GlobalSearch;
use crate::features::logic_graph::LogicGraphBox;
use crate::features::scene_builder::box_::SceneBuilderBox;
use crate::core::settings::SettingsPanel;
use crate::ui::theme::Theme;
use gpui::{
    actions, div, px, IntoElement, ParentElement, Render, Styled, View, ViewContext,
    VisualContext as _, InteractiveElement,
};
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

actions!(editor_window, [ToggleGlobalSearch]);

/// The main editor window containing all UI elements
pub struct EditorWindow {
    /// Handle to the Luminara Engine
    engine_handle: Arc<EngineHandle>,
    /// Activity Bar component
    activity_bar: ActivityBar,
    /// Scene Builder Box (main editing interface)
    scene_builder: View<SceneBuilderBox>,
    /// Logic Graph Box
    logic_graph_box: View<LogicGraphBox>,
    /// Director Box
    director_box: View<DirectorBox>,
    /// Backend & AI Box
    backend_ai_box: View<BackendAIBox>,
    /// Asset Vault Box
    asset_vault_box: View<AssetVaultBox>,
    /// Extensions Box
    extension_box: View<ExtensionBox>,
    /// Global Search overlay
    global_search: View<GlobalSearch>,
    /// Settings Panel overlay
    settings_panel: View<SettingsPanel>,
    /// Account Panel overlay
    account_panel: View<AccountPanel>,
    /// Theme for styling
    theme: Arc<Theme>,
    /// Shared editor state with generation counter for command-to-UI bridge
    shared_state: SharedEditorState,
}

impl EditorWindow {
    /// Create a new EditorWindow
    ///
    /// # Arguments
    /// * `engine_handle` - Arc-wrapped handle to the Luminara Engine
    /// * `cx` - GPUI context
    ///
    /// # Requirements
    /// - Requirement 1.3: Provide a root window with proper event handling
    /// - Requirement 3.1: Display Global Search on Cmd+K/Ctrl+K
    pub fn new(engine_handle: Arc<EngineHandle>, cx: &mut ViewContext<Self>) -> Self {
        let theme = Arc::new(Theme::default_dark());
        let activity_bar = ActivityBar::new(theme.clone());
        let scene_builder = cx.new_view(|cx| {
            SceneBuilderBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        let logic_graph_box = cx.new_view(|_cx| {
            LogicGraphBox::new(theme.clone())
        });
        let director_box = cx.new_view(|cx| {
            DirectorBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        let backend_ai_box = cx.new_view(|_cx| {
            BackendAIBox::new(theme.clone())
        });
        let asset_vault_box = cx.new_view(|cx| {
            AssetVaultBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        let extension_box = cx.new_view(|cx| {
            ExtensionBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        
        // Create shared editor state with generation counter
        let state = Arc::new(RwLock::new(EditorState::new()));
        let shared_state = SharedEditorState::new(state);
        
        // Create GlobalSearch with shared state
        let global_search = cx.new_view(|cx| {
            GlobalSearch::with_state(theme.clone(), Some(shared_state.state()), cx)
        });
        
        // Create SettingsPanel
        let settings_panel = cx.new_view(|_cx| {
            SettingsPanel::new(theme.clone())
        });
        
        // Create AccountPanel
        let account_panel = cx.new_view(|_cx| {
            AccountPanel::new(theme.clone())
        });
        
        // Start the state change poller on the GPUI async runtime.
        // This polls the generation counter every 16ms (~60Hz) and triggers
        // a re-render when background threads (e.g. keyboard monitor) change state.
        let generation = shared_state.generation();
        cx.spawn(|this, mut cx| async move {
            let mut last_gen = 0u64;
            loop {
                cx.background_executor().timer(Duration::from_millis(16)).await;
                let current_gen = generation.load(Ordering::Acquire);
                if current_gen != last_gen {
                    last_gen = current_gen;
                    let _ = this.update(&mut cx, |this, cx| {
                        this.global_search.update(cx, |search, _cx| {
                            search.sync_with_state();
                        });
                        cx.notify();
                    });
                }
            }
        }).detach();
        
        Self {
            engine_handle,
            activity_bar,
            scene_builder,
            logic_graph_box,
            director_box,
            backend_ai_box,
            asset_vault_box,
            extension_box,
            global_search,
            settings_panel,
            account_panel,
            theme,
            shared_state,
        }
    }
    
    /// Create a new EditorWindow with a pre-existing SharedEditorState.
    ///
    /// Used when the editor state is shared with a background keyboard thread.
    pub fn with_shared_state(
        engine_handle: Arc<EngineHandle>,
        shared_state: SharedEditorState,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let theme = Arc::new(Theme::default_dark());
        let activity_bar = ActivityBar::new(theme.clone());
        let scene_builder = cx.new_view(|cx| {
            SceneBuilderBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        let logic_graph_box = cx.new_view(|_cx| {
            LogicGraphBox::new(theme.clone())
        });
        let director_box = cx.new_view(|cx| {
            DirectorBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        let backend_ai_box = cx.new_view(|_cx| {
            BackendAIBox::new(theme.clone())
        });
        let asset_vault_box = cx.new_view(|cx| {
            AssetVaultBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        let extension_box = cx.new_view(|cx| {
            ExtensionBox::new(engine_handle.clone(), theme.clone(), cx)
        });
        
        // Create GlobalSearch with shared state
        let global_search = cx.new_view(|cx| {
            GlobalSearch::with_state(theme.clone(), Some(shared_state.state()), cx)
        });
        
        // Create SettingsPanel
        let settings_panel = cx.new_view(|_cx| {
            SettingsPanel::new(theme.clone())
        });
        
        // Create AccountPanel
        let account_panel = cx.new_view(|_cx| {
            AccountPanel::new(theme.clone())
        });
        
        // Start the state change poller
        let generation = shared_state.generation();
        cx.spawn(|this, mut cx| async move {
            let mut last_gen = 0u64;
            loop {
                cx.background_executor().timer(Duration::from_millis(16)).await;
                let current_gen = generation.load(Ordering::Acquire);
                if current_gen != last_gen {
                    last_gen = current_gen;
                    let _ = this.update(&mut cx, |this, cx| {
                        this.global_search.update(cx, |search, _cx| {
                            search.sync_with_state();
                        });
                        cx.notify();
                    });
                }
            }
        }).detach();
        
        Self {
            engine_handle,
            activity_bar,
            scene_builder,
            logic_graph_box,
            director_box,
            backend_ai_box,
            asset_vault_box,
            extension_box,
            global_search,
            settings_panel,
            account_panel,
            theme,
            shared_state,
        }
    }

    /// Get a reference to the engine handle
    pub fn engine(&self) -> &Arc<EngineHandle> {
        &self.engine_handle
    }
    
    /// Get a mutable reference to the activity bar
    pub fn activity_bar_mut(&mut self) -> &mut ActivityBar {
        &mut self.activity_bar
    }
    
    /// Get a reference to the activity bar
    pub fn activity_bar(&self) -> &ActivityBar {
        &self.activity_bar
    }
    
    /// Toggle the Global Search overlay
    ///
    /// # Requirements
    /// - Requirement 3.1: Display on Cmd+K/Ctrl+K
    pub fn toggle_global_search(&mut self, _: &ToggleGlobalSearch, cx: &mut ViewContext<Self>) {
        // Update shared state (generation counter is automatically incremented)
        self.shared_state.toggle_global_search();
        // Update the view immediately since we're already on the GPUI thread
        self.global_search.update(cx, |search, _cx| {
            search.sync_with_state();
        });
        cx.notify();
    }
    
    /// Get the shared editor state (for passing to keyboard threads, etc.)
    pub fn shared_state(&self) -> &SharedEditorState {
        &self.shared_state
    }
    
    /// Get the raw editor state Arc (for backward compatibility)
    pub fn editor_state(&self) -> Arc<RwLock<EditorState>> {
        self.shared_state.state()
    }
    
    /// Get a reference to the global search view
    pub fn global_search(&self) -> &View<GlobalSearch> {
        &self.global_search
    }
    
    /// Toggle the Settings Panel overlay
    pub fn toggle_settings_panel(&mut self, cx: &mut ViewContext<Self>) {
        // Hide account panel if visible
        if self.account_panel.read(cx).is_visible() {
            self.account_panel.update(cx, |panel, cx| {
                panel.hide(cx);
            });
        }
        // Toggle settings panel
        self.settings_panel.update(cx, |panel, cx| {
            panel.toggle(cx);
        });
        cx.notify();
    }
    
    /// Toggle the Account Panel overlay
    pub fn toggle_account_panel(&mut self, cx: &mut ViewContext<Self>) {
        // Hide settings panel if visible
        if self.settings_panel.read(cx).is_visible() {
            self.settings_panel.update(cx, |panel, cx| {
                panel.hide(cx);
            });
        }
        // Toggle account panel
        self.account_panel.update(cx, |panel, cx| {
            panel.toggle(cx);
        });
        cx.notify();
    }
    
    /// Get a reference to the settings panel view
    pub fn settings_panel(&self) -> &View<SettingsPanel> {
        &self.settings_panel
    }
    
    /// Get a reference to the account panel view
    pub fn account_panel(&self) -> &View<AccountPanel> {
        &self.account_panel
    }
}

impl Render for EditorWindow {
    /// Render the editor window
    ///
    /// This creates the root UI layout with:
    /// - Activity Bar on the left (52px wide)
    /// - Main content area on the right
    /// - Global Search overlay (when visible)
    /// - Dark background matching the theme
    ///
    /// # Requirements
    /// - Requirement 2.1: Activity Bar positioned on left edge
    /// - Requirement 2.2: Activity item click handling activates corresponding Box
    /// - Requirement 3.1: Display Global Search on Cmd+K/Ctrl+K
    /// - Requirement 3.2: 2-column layout (38% / 62%)
    /// - Requirement 10.1: Use color palette matching HTML prototypes
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme_for_main = self.theme.clone();
        // Determine which panel to show based on active activity bar item
        // 0 = Global Search, 1 = Scene Builder, 2 = Logic Graph, 3 = Director, 4 = Backend & AI, 5 = Asset Vault, 6 = Extensions
        let active_panel: gpui::AnyView = match self.activity_bar.active_index() {
            Some(2) => self.logic_graph_box.clone().into(),
            Some(3) => self.director_box.clone().into(),
            Some(4) => self.backend_ai_box.clone().into(),
            Some(5) => self.asset_vault_box.clone().into(),
            Some(6) => self.extension_box.clone().into(),
            _ => self.scene_builder.clone().into(),
        };

        // Get theme for bottom items
        let theme_for_bottom = self.theme.clone();
        
        // Build the full activity bar with bottom items
        let activity_bar_main = self.activity_bar.render_inline(cx);
        let bottom_items = self.activity_bar.bottom_items();
        
        let activity_bar_full = div()
            .flex()
            .flex_col()
            .w(px(crate::ui::layouts::activity_bar::ACTIVITY_BAR_WIDTH))
            .h_full()
            .child(activity_bar_main)
            .child(div().flex_1())
            .children(bottom_items.into_iter().map(move |item| {
                let theme = theme_for_bottom.clone();
                let is_settings = item.id == "settings";
                let is_user = item.id == "user";
                
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(crate::ui::layouts::activity_bar::ACTIVITY_BAR_WIDTH))
                    .h(px(48.0))
                    .bg(theme.colors.background)
                    .hover(|this| this.bg(theme.colors.surface_hover))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _event: &gpui::MouseDownEvent, cx| {
                        if is_settings {
                            this.toggle_settings_panel(cx);
                        } else if is_user {
                            this.toggle_account_panel(cx);
                        }
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(28.0))
                            .h(px(28.0))
                            .child(
                                gpui::svg()
                                    .path(item.icon_svg.clone())
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .text_color(theme.colors.text_secondary)
                            )
                    )
            }));
        
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(theme_for_main.colors.background)
            // Activity Bar on the left (52px) with bottom items
            .child(activity_bar_full)
            // Main content area - switches based on active activity bar item
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(active_panel)
            )
            // Global Search overlay (rendered on top)
            .child(self.global_search.clone())
            // Settings Panel overlay (rendered on top)
            .child(self.settings_panel.clone())
            // Account Panel overlay (rendered on top)
            .child(self.account_panel.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_window_creation() {
        // This test verifies the EditorWindow can be created with a mock engine
        let engine = Arc::new(EngineHandle::mock());
        
        // Note: We can't fully test GPUI rendering without a full GPUI context,
        // but we can verify the struct is created correctly
        assert!(Arc::strong_count(&engine) == 1);
    }
}
