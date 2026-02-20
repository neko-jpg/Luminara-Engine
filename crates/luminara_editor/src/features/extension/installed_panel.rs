//! Installed Extensions Panel Component
//!
//! Displays a list of installed extensions with:
//! - Search/filter input at the top
//! - Extension items with icon, name, version, author
//! - Toggle switch for enabling/disabling
//! - Selection highlighting

use gpui::{
    div, px, svg, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Represents an installed extension item
#[derive(Debug, Clone)]
pub struct ExtensionItem {
    /// Unique identifier
    id: String,
    /// Display name
    name: String,
    /// Version string
    version: String,
    /// Author name
    author: String,
    /// Icon name (from icons/)
    icon: String,
    /// Whether the extension is enabled
    enabled: bool,
    /// Description
    description: String,
}

impl ExtensionItem {
    /// Create a new extension item
    pub fn new(id: &str, name: &str, version: &str, author: &str, icon: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            icon: icon.to_string(),
            enabled: true,
            description: String::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get the extension id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the extension name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the author
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Get the icon name
    pub fn icon(&self) -> &str {
        &self.icon
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get description
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// The Installed Extensions Panel component
pub struct InstalledPanel {
    /// Theme for styling
    theme: Arc<Theme>,
    /// List of installed extensions
    extensions: Vec<ExtensionItem>,
    /// Currently selected extension id
    selected_id: Option<String>,
    /// Filter text
    filter: String,
}

impl InstalledPanel {
    /// Create a new Installed Extensions Panel
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            extensions: Vec::new(),
            selected_id: None,
            filter: String::new(),
        }
    }

    /// Create with sample data for testing
    pub fn with_sample_data(theme: Arc<Theme>) -> Self {
        let extensions = vec![
            ExtensionItem::new(
                "shader-editor",
                "Shader Editor",
                "v1.0.0",
                "username",
                "palette"
            )
            .with_description("Node-based shader editor")
            .with_enabled(true),
            ExtensionItem::new(
                "ai-assistant",
                "AI Assistant",
                "v2.3.0",
                "core",
                "robot"
            )
            .with_description("AI-powered coding assistant")
            .with_enabled(true),
            ExtensionItem::new(
                "terrain-generator",
                "Terrain Generator",
                "v0.5.0",
                "nature",
                "mountain"
            )
            .with_description("Procedural terrain generation")
            .with_enabled(false),
            ExtensionItem::new(
                "node-pack-physics",
                "Node Pack: Physics",
                "v1.2.0",
                "phys",
                "plug"
            )
            .with_description("Physics simulation nodes")
            .with_enabled(true),
        ];

        Self {
            theme,
            extensions,
            selected_id: Some("shader-editor".to_string()),
            filter: String::new(),
        }
    }

    /// Set the selected extension
    pub fn set_selected(&mut self, id: Option<String>) {
        self.selected_id = id;
    }

    /// Get the selected extension id
    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    /// Set filter text
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    /// Get filtered extensions
    fn filtered_extensions(&self) -> Vec<&ExtensionItem> {
        if self.filter.is_empty() {
            self.extensions.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.extensions
                .iter()
                .filter(|e| {
                    e.name.to_lowercase().contains(&filter_lower) ||
                    e.id.to_lowercase().contains(&filter_lower)
                })
                .collect()
        }
    }

    /// Render the filter input
    fn render_filter(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .gap(px(4.0))
            .mb(theme.spacing.md)
            .child(
                div()
                    .flex_1()
                    .h(px(32.0))
                    .bg(theme.colors.surface)
                    .rounded(theme.borders.sm)
                    .border_1()
                    .border_color(theme.colors.border)
                    .flex()
                    .items_center()
                    .px(theme.spacing.md)
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.md)
                            .child("Filter...")
                    )
            )
            .child(
                div()
                    .h(px(32.0))
                    .px(theme.spacing.md)
                    .bg(theme.colors.toolbar_active)
                    .rounded(theme.borders.sm)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|this| this.opacity(0.8))
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.md)
                            .child("Go")
                    )
            )
    }

    /// Render a toggle switch
    fn render_toggle(&self, enabled: bool) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .w(px(32.0))
            .h(px(16.0))
            .rounded(px(8.0))
            .bg(if enabled { theme.colors.toolbar_active } else { theme.colors.border })
            .relative()
            .child(
                div()
                    .absolute()
                    .top(px(2.0))
                    .left(if enabled { px(18.0) } else { px(2.0) })
                    .w(px(12.0))
                    .h(px(12.0))
                    .rounded_full()
                    .bg(theme.colors.text)
            )
    }

    /// Render an extension item
    fn render_extension_item(&self, extension: &ExtensionItem) -> impl IntoElement {
        let theme = self.theme.clone();
        let is_selected = self.selected_id.as_ref() == Some(&extension.id);
        let extension = extension.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .p(px(8.0))
            .rounded(theme.borders.sm)
            .gap(px(8.0))
            .border_l_3()
            .border_color(if is_selected { theme.colors.accent } else { theme.colors.surface })
            .bg(if is_selected { 
                theme.colors.toolbar_active 
            } else { 
                theme.colors.surface 
            })
            .hover(|this| {
                if !is_selected {
                    this.bg(theme.colors.surface_hover)
                } else {
                    this
                }
            })
            .cursor_pointer()
            // Extension icon
            .child(
                div()
                    .w(px(28.0))
                    .h(px(28.0))
                    .rounded(theme.borders.sm)
                    .bg(theme.colors.condition_bg)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(format!("icons/{}.svg", extension.icon))
                            .w(px(16.0))
                            .h(px(16.0))
                            .text_color(theme.colors.accent)
                    )
            )
            // Extension info
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.md)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(extension.name.clone())
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child(format!("{} • by {}", extension.version, extension.author))
                            )
                    )
            )
            // Toggle switch
            .child(self.render_toggle(extension.enabled))
    }

    /// Render the panel header
    fn render_header(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(36.0))
            .bg(theme.colors.panel_header)
            .px(theme.spacing.md)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        svg()
                            .path("icons/puzzle.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.md)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Installed Extensions")
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(theme.spacing.md)
                    .child(
                        svg()
                            .path("icons/search.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
                    .child(
                        svg()
                            .path("icons/dots-vertical.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
            )
    }
}

impl Render for InstalledPanel {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let extensions = self.filtered_extensions();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors.surface)
            .border_1()
            .border_color(theme.colors.border)
            .rounded_t(px(4.0))
            // Header
            .child(self.render_header())
            // Content
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p(theme.spacing.md)
                    .overflow_hidden()
                    // Filter input
                    .child(self.render_filter())
                    // Extension list
                    .children(
                        extensions.into_iter().map(|ext| self.render_extension_item(ext))
                    )
            )
    }
}
