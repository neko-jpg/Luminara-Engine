//! Asset Inspector Component
//!
//! Inspector panel for displaying asset metadata with:
//! - Asset name and basic properties
//! - File information (path, size, hash)
//! - Type-specific settings (3D settings for models, etc.)
//! - Usage information
//! - Tags

use gpui::{
    div, px, IntoElement, InteractiveElement, ParentElement, Render, Styled, ViewContext, prelude::FluentBuilder,
};
use std::sync::Arc;

use crate::ui::theme::Theme;
use super::{AssetItem, AssetType};

/// A single property row in the inspector
#[derive(Debug, Clone)]
pub struct AssetProperty {
    /// Property label
    pub label: String,
    /// Property value
    pub value: String,
}

impl AssetProperty {
    /// Create a new property
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
        }
    }
}

/// Asset metadata for inspector
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    /// Asset name
    pub name: String,
    /// Asset type
    pub asset_type: AssetType,
    /// File path
    pub path: String,
    /// File type display name
    pub type_name: String,
    /// File size
    pub size: String,
    /// SHA256 hash (truncated)
    pub sha256: String,
    /// Creation date
    pub created: String,
    /// Tags
    pub tags: Vec<String>,
    /// Properties
    pub properties: Vec<AssetProperty>,
    /// Usage list
    pub usage: Vec<String>,
}

impl AssetMetadata {
    /// Create empty metadata
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            asset_type: AssetType::Unknown,
            path: String::new(),
            type_name: String::new(),
            size: String::new(),
            sha256: String::new(),
            created: String::new(),
            tags: Vec::new(),
            properties: Vec::new(),
            usage: Vec::new(),
        }
    }

    /// Create from asset item
    pub fn from_asset(asset: &AssetItem) -> Self {
        let mut metadata = Self::empty();
        metadata.name = asset.name().to_string();
        metadata.asset_type = asset.asset_type();
        metadata.type_name = asset.asset_type().display_name().to_string();
        
        if let Some(path) = asset.path() {
            metadata.path = path.to_string();
        }
        
        if let Some(size) = asset.size() {
            metadata.size = size.to_string();
        }
        
        metadata
    }

    /// Create sample metadata for hero.glb
    pub fn sample_hero_glb() -> Self {
        Self {
            name: "hero.glb".to_string(),
            asset_type: AssetType::Model,
            path: "assets/models/hero.glb".to_string(),
            type_name: "3D Model".to_string(),
            size: "2.4 MB (1.8 MB compressed)".to_string(),
            sha256: "3f4e...9a2c".to_string(),
            created: "2025-02-10".to_string(),
            tags: vec!["Character".to_string(), "Hero".to_string()],
            properties: vec![
                AssetProperty::new("Scale Factor", "1.0"),
                AssetProperty::new("Up Axis", "Y-Up"),
                AssetProperty::new("Collider", "Convex Hull"),
                AssetProperty::new("Animations", "3 clips"),
                AssetProperty::new("LODs", "LOD0 (100%)"),
                AssetProperty::new("Vertices", "12,450"),
            ],
            usage: vec![
                "Main Scene (Player entity)".to_string(),
                "Level2 (Enemy variant)".to_string(),
                "Prefab 'hero_prefab'".to_string(),
            ],
        }
    }
}

/// Asset Inspector component
pub struct AssetInspector {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Current metadata
    metadata: AssetMetadata,
}

impl AssetInspector {
    /// Create a new empty inspector
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            metadata: AssetMetadata::empty(),
        }
    }

    /// Create with sample metadata
    pub fn with_sample_data(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            metadata: AssetMetadata::sample_hero_glb(),
        }
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: AssetMetadata) {
        self.metadata = metadata;
    }

    /// Clear metadata
    pub fn clear_metadata(&mut self) {
        self.metadata = AssetMetadata::empty();
    }

    /// Get current metadata
    pub fn metadata(&self) -> &AssetMetadata {
        &self.metadata
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
            .h(px(32.0))
            .px(px(12.0))
            .bg(theme.colors.surface)
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(theme.typography.sm)
                    .text_color(theme.colors.text_secondary)
                    .child("ℹ")
                    .child("Inspector")
            )
            .child(
                div()
                    .text_color(theme.colors.text_secondary)
                    .child("⋮")
            )
    }

    /// Render a property row
    fn render_property_row(&self, property: &AssetProperty) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .py(px(3.0))
            .child(
                div()
                    .text_size(theme.typography.sm)
                    .text_color(theme.colors.text_secondary)
                    .child(property.label.clone())
            )
            .child(
                div()
                    .text_size(theme.typography.sm)
                    .text_color(theme.colors.accent)
                    .max_w(px(150.0))
                    .child(property.value.clone())
            )
    }

    /// Render a tag
    fn render_tag(&self, tag: &str) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .px(px(6.0))
            .py(px(2.0))
            .bg(theme.colors.accent)
            .rounded(px(12.0))
            .text_size(px(10.0))
            .text_color(theme.colors.text)
            .child(tag.to_string())
    }

    /// Render a section group
    fn render_group(&self, title: &str, children: Vec<impl IntoElement>) -> impl IntoElement {
        let theme = self.theme.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .pb(px(12.0))
            .border_b_1()
            .border_color(theme.colors.border)
            .child(
                div()
                    .py(px(8.0))
                    .text_size(theme.typography.sm)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.colors.text)
                    .child(title.to_string())
            )
            .children(children)
    }
}

impl Render for AssetInspector {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let metadata = self.metadata.clone();

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
                    .flex_1()
                    .overflow_hidden()
                    .p(px(12.0))
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    // Basic Info Group
                    .child(
                        self.render_group(
                            &format!("{} {}", metadata.asset_type.icon(), metadata.name),
                            vec![
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(self.render_property_row(&AssetProperty::new("Path", &metadata.path)))
                                    .child(self.render_property_row(&AssetProperty::new("Type", &metadata.type_name)))
                                    .child(self.render_property_row(&AssetProperty::new("Size", &metadata.size)))
                                    .child(self.render_property_row(&AssetProperty::new("SHA256", &metadata.sha256)))
                                    .child(self.render_property_row(&AssetProperty::new("Created", &metadata.created)))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(4.0))
                                            .py(px(4.0))
                                            .child(
                                                div()
                                                    .text_size(theme.typography.sm)
                                                    .text_color(theme.colors.text_secondary)
                                                    .child("Tags")
                                            )
                                            .child(div().flex_1())
                                            .children(
                                                metadata.tags.iter().map(|tag| {
                                                    self.render_tag(tag)
                                                })
                                            )
                                    )
                                    .into_element()
                            ]
                        )
                    )
                    // 3D Settings Group (only for models)
                    .when(metadata.asset_type == AssetType::Model, |this| {
                        this.child(
                            self.render_group(
                                "3D Settings",
                                metadata.properties.iter().map(|prop| {
                                    self.render_property_row(prop).into_element()
                                }).collect()
                            )
                        )
                    })
                    // Usage Group
                    .child(
                        self.render_group(
                            "Usage",
                            vec![
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .children(
                                        metadata.usage.iter().map(|item| {
                                            div()
                                                .text_size(px(11.0))
                                                .text_color(theme.colors.text_secondary)
                                                .child(format!("• {}", item))
                                        })
                                    )
                                    .into_element()
                            ]
                        )
                    )
                    // Import Settings Button
                    .child(
                        div()
                            .mt_auto()
                            .px(px(12.0))
                            .py(px(6.0))
                            .bg(theme.colors.accent)
                            .rounded(px(20.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(6.0))
                            .text_size(theme.typography.sm)
                            .text_color(theme.colors.text)
                            .hover(|this| this.opacity(0.8))
                            .cursor_pointer()
                            .child("⚙")
                            .child("Import Settings")
                    )
            )
    }
}
