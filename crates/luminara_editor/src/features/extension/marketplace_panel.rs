//! Marketplace and Development Panel Component
//!
//! Displays marketplace recommendations and development tools:
//! - Search bar for marketplace
//! - Recommended extension cards with install button
//! - Templates and API Docs tabs at the bottom

use gpui::{
    div, px, svg, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext,
};
use std::sync::Arc;

use crate::ui::theme::Theme;

/// Marketplace extension card data
#[derive(Debug, Clone)]
pub struct MarketplaceExtension {
    /// Extension name
    name: String,
    /// Icon name
    icon: String,
    /// Rating (e.g., "4.8")
    rating: String,
    /// Download count
    downloads: String,
}

impl MarketplaceExtension {
    /// Create a new marketplace extension
    pub fn new(name: &str, icon: &str, rating: &str, downloads: &str) -> Self {
        Self {
            name: name.to_string(),
            icon: icon.to_string(),
            rating: rating.to_string(),
            downloads: downloads.to_string(),
        }
    }
}

/// Development tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevTab {
    /// Templates view
    Templates,
    /// API Docs view
    ApiDocs,
}

/// The Marketplace and Development Panel component
pub struct MarketplacePanel {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Marketplace extensions
    marketplace_extensions: Vec<MarketplaceExtension>,
    /// Current dev tab
    dev_tab: DevTab,
}

impl MarketplacePanel {
    /// Create a new Marketplace Panel
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            marketplace_extensions: Vec::new(),
            dev_tab: DevTab::Templates,
        }
    }

    /// Create with sample data for testing
    pub fn with_sample_data(theme: Arc<Theme>) -> Self {
        let marketplace_extensions = vec![
            MarketplaceExtension::new("Performance Monitor", "chart-line", "4.8", "2.3k"),
            MarketplaceExtension::new("Cinematic Sequencer", "video", "4.5", "1.1k"),
            MarketplaceExtension::new("Voxel Importer", "cubes", "4.2", "890"),
        ];

        Self {
            theme,
            marketplace_extensions,
            dev_tab: DevTab::Templates,
        }
    }

    /// Set the current dev tab
    pub fn set_dev_tab(&mut self, tab: DevTab) {
        self.dev_tab = tab;
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
                            .path("icons/store.svg")
                            .w(px(14.0))
                            .h(px(14.0))
                            .text_color(theme.colors.text_secondary)
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.md)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Marketplace / Dev")
                    )
            )
            .child(
                svg()
                    .path("icons/dots-vertical.svg")
                    .w(px(14.0))
                    .h(px(14.0))
                    .text_color(theme.colors.text_secondary)
            )
    }

    /// Render the search input
    fn render_search(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .w_full()
            .h(px(36.0))
            .bg(theme.colors.surface)
            .rounded(px(18.0))
            .border_1()
            .border_color(theme.colors.border)
            .flex()
            .items_center()
            .px(theme.spacing.md)
            .mb(theme.spacing.md)
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.md)
                    .child("Search marketplace...")
            )
    }

    /// Render a marketplace extension card
    fn render_marketplace_card(&self, extension: &MarketplaceExtension) -> impl IntoElement {
        let theme = self.theme.clone();
        let extension = extension.clone();
        
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .p(theme.spacing.md)
            .mb(px(8.0))
            .rounded(theme.borders.md)
            .bg(theme.colors.panel_header)
            .gap(theme.spacing.md)
            // Icon
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .rounded(theme.borders.md)
                    .bg(rgb_to_hsla(0x4a4a6a))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(format!("icons/{}.svg", extension.icon))
                            .w(px(16.0))
                            .h(px(16.0))
                            .text_color(theme.colors.text)
                    )
            )
            // Info
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
                            .child(extension.name)
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(4.0))
                            .child(
                                svg()
                                    .path("icons/star.svg")
                                    .w(px(10.0))
                                    .h(px(10.0))
                                    .text_color(theme.colors.text_secondary)
                            )
                            .child(
                                div()
                                    .text_color(theme.colors.text_secondary)
                                    .text_size(theme.typography.sm)
                                    .child(format!("{} • {} downloads", extension.rating, extension.downloads))
                            )
                    )
            )
            // Install button
            .child(
                div()
                    .px(theme.spacing.md)
                    .py(px(4.0))
                    .rounded(px(15.0))
                    .bg(theme.colors.toolbar_active)
                    .cursor_pointer()
                    .hover(|this| this.opacity(0.8))
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child("Install")
                    )
            )
    }

    /// Render the development mini tabs
    fn render_dev_tabs(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_row()
            .w_full()
            .border_b_1()
            .border_color(theme.colors.border)
            .mb(theme.spacing.sm)
            .gap(px(2.0))
            .child(
                div()
                    .px(theme.spacing.md)
                    .py(px(4.0))
                    .rounded_t(px(16.0))
                    .bg(if self.dev_tab == DevTab::Templates { theme.colors.toolbar_active } else { theme.colors.panel_header })
                    .text_color(if self.dev_tab == DevTab::Templates { theme.colors.text } else { theme.colors.text_secondary })
                    .text_size(theme.typography.sm)
                    .cursor_pointer()
                    .hover(|this| {
                        if self.dev_tab != DevTab::Templates {
                            this.bg(theme.colors.surface_hover)
                        } else {
                            this
                        }
                    })
                    .child("Templates")
            )
            .child(
                div()
                    .px(theme.spacing.md)
                    .py(px(4.0))
                    .rounded_t(px(16.0))
                    .bg(if self.dev_tab == DevTab::ApiDocs { theme.colors.toolbar_active } else { theme.colors.panel_header })
                    .text_color(if self.dev_tab == DevTab::ApiDocs { theme.colors.text } else { theme.colors.text_secondary })
                    .text_size(theme.typography.sm)
                    .cursor_pointer()
                    .hover(|this| {
                        if self.dev_tab != DevTab::ApiDocs {
                            this.bg(theme.colors.surface_hover)
                        } else {
                            this
                        }
                    })
                    .child("API Docs")
            )
    }

    /// Render the Templates tab content
    fn render_templates_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_col()
            .gap(theme.spacing.md)
            // New Extension template (highlighted)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .w_full()
                    .p(theme.spacing.lg)
                    .rounded(theme.borders.md)
                    .border_1()
                    
                    .border_color(rgb_to_hsla(0x6a8a8a))
                    .bg(rgb_to_hsla(0x2a3a3a))
                    .child(
                        svg()
                            .path("icons/puzzle.svg")
                            .w(px(24.0))
                            .h(px(24.0))
                            .text_color(theme.colors.text)
                            .mb(theme.spacing.sm)
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.md)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("New Extension")
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text_secondary)
                            .text_size(theme.typography.sm)
                            .child("Create from template")
                    )
                    .child(
                        div()
                            .mt(theme.spacing.md)
                            .px(theme.spacing.md)
                            .py(px(4.0))
                            .rounded(px(15.0))
                            .bg(theme.colors.toolbar_active)
                            .cursor_pointer()
                            .hover(|this| this.opacity(0.8))
                            .child(
                                div()
                                    .text_color(theme.colors.text)
                                    .text_size(theme.typography.sm)
                                    .child("Generate")
                            )
                    )
            )
            // Widget template
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .p(theme.spacing.md)
                    .rounded(theme.borders.md)
                    .border_1()
                    
                    .border_color(rgb_to_hsla(0x6a8a8a))
                    .bg(rgb_to_hsla(0x2a3a3a))
                    .gap(px(8.0))
                    .child(
                        svg()
                            .path("icons/palette.svg")
                            .w(px(16.0))
                            .h(px(16.0))
                            .text_color(theme.colors.text)
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child("Widget template")
                    )
            )
            // Logic Node template
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .p(theme.spacing.md)
                    .rounded(theme.borders.md)
                    .border_1()
                    
                    .border_color(rgb_to_hsla(0x6a8a8a))
                    .bg(rgb_to_hsla(0x2a3a3a))
                    .gap(px(8.0))
                    .child(
                        svg()
                            .path("icons/code-branch.svg")
                            .w(px(16.0))
                            .h(px(16.0))
                            .text_color(theme.colors.text)
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(theme.typography.sm)
                            .child("Logic Node template")
                    )
            )
    }

    /// Render the API Docs tab content
    fn render_api_docs_tab(&self) -> impl IntoElement {
        let theme = self.theme.clone();
        
        div()
            .w_full()
            .p(theme.spacing.md)
            .rounded(theme.borders.sm)
            .bg(rgb_to_hsla(0x1e1e1e))
            .font_family("monospace")
            .child(
                div()
                    .text_color(theme.colors.accent)
                    .text_size(theme.typography.sm)
                    .child("LuminaraBox trait")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("• fn register(…)")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("• fn widgets() → Vec<Box<dyn Widget>>")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .text_size(theme.typography.sm)
                    .child("• fn handle_message(&mut self, …)")
            )
            .child(
                div()
                    .mt(theme.spacing.sm)
                    .text_color(theme.colors.accent)
                    .text_size(theme.typography.sm)
                    .child("More: See extension.toml format")
            )
    }

    /// Render the dev section content
    fn render_dev_section(&self) -> impl IntoElement {
        match self.dev_tab {
            DevTab::Templates => self.render_templates_tab().into_any_element(),
            DevTab::ApiDocs => self.render_api_docs_tab().into_any_element(),
        }
    }
}

impl Render for MarketplacePanel {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        
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
                    // Search
                    .child(self.render_search())
                    // Marketplace cards
                    .children(
                        self.marketplace_extensions.iter().map(|ext| self.render_marketplace_card(ext))
                    )
                    // Separator
                    .child(
                        div()
                            .w_full()
                            .h(px(1.0))
                            .bg(theme.colors.border)
                            .my(theme.spacing.md)
                    )
                    // Dev tabs
                    .child(self.render_dev_tabs())
                    // Dev content
                    .child(self.render_dev_section())
            )
    }
}

/// Helper to convert RGB to Hsla
fn rgb_to_hsla(rgb: u32) -> gpui::Hsla {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    
    let s = if max == min {
        0.0
    } else {
        let d = max - min;
        if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) }
    };
    
    let h = if max == min {
        0.0
    } else if max == r {
        ((g - b) / (max - min) + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / (max - min) + 2.0) / 6.0
    } else {
        ((r - g) / (max - min) + 4.0) / 6.0
    };
    
    gpui::Hsla { h, s, l, a: 1.0 }
}
