//! # Unified Gizmo System
//!
//! Centralizes all debug visualization with support for multiple modes:
//! - Physics: colliders, velocity vectors, contact points
//! - Rendering: wireframes, normals, bounding boxes
//! - Transforms: coordinate axes, hierarchy connections
//! - Audio: source positions, attenuation ranges
//!
//! Integrates with OverlayRenderer for 2D overlays and uses the CommandBuffer
//! for 3D gizmo rendering.

use crate::{CommandBuffer, GizmoCategories, Gizmos, OverlayRenderer};
use luminara_core::shared_types::Resource;
use luminara_math::{Color, Vec3};
use std::collections::HashMap;

// ============================================================================
// Visualization Modes
// ============================================================================

/// Visualization mode for debug rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualizationMode {
    /// Physics debug visualization (colliders, velocities, contacts)
    Physics,
    /// Rendering debug visualization (wireframes, normals, overdraw)
    Rendering,
    /// Transform debug visualization (axes, hierarchy)
    Transforms,
    /// Audio debug visualization (sources, attenuation)
    Audio,
    /// Custom user-defined mode
    Custom(&'static str),
}

impl VisualizationMode {
    /// Get the category name for this mode
    pub fn category(&self) -> &str {
        match self {
            Self::Physics => "physics",
            Self::Rendering => "rendering",
            Self::Transforms => "transforms",
            Self::Audio => "audio",
            Self::Custom(name) => name,
        }
    }
}

// ============================================================================
// Gizmo System
// ============================================================================

/// Unified system for all debug visualization
pub struct GizmoSystem {
    /// Active visualization modes
    active_modes: HashMap<VisualizationMode, bool>,
    /// Global gizmo visibility
    enabled: bool,
    /// Physics visualization settings
    physics_settings: PhysicsVisualizationSettings,
    /// Rendering visualization settings
    rendering_settings: RenderingVisualizationSettings,
    /// Transform visualization settings
    transform_settings: TransformVisualizationSettings,
    /// Audio visualization settings
    audio_settings: AudioVisualizationSettings,
}

impl Resource for GizmoSystem {}

impl Default for GizmoSystem {
    fn default() -> Self {
        let mut active_modes = HashMap::new();
        active_modes.insert(VisualizationMode::Physics, false);
        active_modes.insert(VisualizationMode::Rendering, false);
        active_modes.insert(VisualizationMode::Transforms, false);
        active_modes.insert(VisualizationMode::Audio, false);

        Self {
            active_modes,
            enabled: true,
            physics_settings: PhysicsVisualizationSettings::default(),
            rendering_settings: RenderingVisualizationSettings::default(),
            transform_settings: TransformVisualizationSettings::default(),
            audio_settings: AudioVisualizationSettings::default(),
        }
    }
}

impl GizmoSystem {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Global Control ──────────────────────────────────────────────────

    /// Enable or disable all gizmo rendering
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if gizmo system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Toggle gizmo system on/off
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    // ── Mode Control ────────────────────────────────────────────────────

    /// Enable a specific visualization mode
    pub fn enable_mode(&mut self, mode: VisualizationMode) {
        self.active_modes.insert(mode, true);
    }

    /// Disable a specific visualization mode
    pub fn disable_mode(&mut self, mode: VisualizationMode) {
        self.active_modes.insert(mode, false);
    }

    /// Toggle a specific visualization mode
    pub fn toggle_mode(&mut self, mode: VisualizationMode) -> bool {
        let entry = self.active_modes.entry(mode).or_insert(false);
        *entry = !*entry;
        *entry
    }

    /// Check if a mode is active
    pub fn is_mode_active(&self, mode: VisualizationMode) -> bool {
        self.enabled && *self.active_modes.get(&mode).unwrap_or(&false)
    }

    /// Get all active modes
    pub fn active_modes(&self) -> Vec<VisualizationMode> {
        self.active_modes
            .iter()
            .filter_map(|(mode, &active)| if active { Some(*mode) } else { None })
            .collect()
    }

    // ── Settings Access ─────────────────────────────────────────────────

    /// Get physics visualization settings
    pub fn physics_settings(&self) -> &PhysicsVisualizationSettings {
        &self.physics_settings
    }

    /// Get mutable physics visualization settings
    pub fn physics_settings_mut(&mut self) -> &mut PhysicsVisualizationSettings {
        &mut self.physics_settings
    }

    /// Get rendering visualization settings
    pub fn rendering_settings(&self) -> &RenderingVisualizationSettings {
        &self.rendering_settings
    }

    /// Get mutable rendering visualization settings
    pub fn rendering_settings_mut(&mut self) -> &mut RenderingVisualizationSettings {
        &mut self.rendering_settings
    }

    /// Get transform visualization settings
    pub fn transform_settings(&self) -> &TransformVisualizationSettings {
        &self.transform_settings
    }

    /// Get mutable transform visualization settings
    pub fn transform_settings_mut(&mut self) -> &mut TransformVisualizationSettings {
        &mut self.transform_settings
    }

    /// Get audio visualization settings
    pub fn audio_settings(&self) -> &AudioVisualizationSettings {
        &self.audio_settings
    }

    /// Get mutable audio visualization settings
    pub fn audio_settings_mut(&mut self) -> &mut AudioVisualizationSettings {
        &mut self.audio_settings
    }

    // ── Integration with GizmoCategories ────────────────────────────────

    /// Sync with GizmoCategories resource
    pub fn sync_with_categories(&self, categories: &mut GizmoCategories) {
        if !self.enabled {
            categories.disable_all();
            return;
        }

        for (mode, &active) in &self.active_modes {
            categories.set_enabled(mode.category(), active);
        }
    }

    // ── Drawing Helpers ─────────────────────────────────────────────────

    /// Draw physics debug visualization
    pub fn draw_physics(
        &self,
        buffer: &mut CommandBuffer,
        collider_position: Vec3,
        collider_half_extents: Vec3,
    ) {
        if !self.is_mode_active(VisualizationMode::Physics) {
            return;
        }

        let settings = &self.physics_settings;
        if settings.show_colliders {
            Gizmos::cube_cat(
                buffer,
                collider_position,
                collider_half_extents,
                settings.collider_color,
                "physics",
            );
        }
    }

    /// Draw velocity vector
    pub fn draw_velocity(
        &self,
        buffer: &mut CommandBuffer,
        position: Vec3,
        velocity: Vec3,
    ) {
        if !self.is_mode_active(VisualizationMode::Physics) {
            return;
        }

        let settings = &self.physics_settings;
        if settings.show_velocities {
            let end = position + velocity * settings.velocity_scale;
            Gizmos::arrow(buffer, position, end, settings.velocity_color);
        }
    }

    /// Draw contact point
    pub fn draw_contact_point(&self, buffer: &mut CommandBuffer, position: Vec3) {
        if !self.is_mode_active(VisualizationMode::Physics) {
            return;
        }

        let settings = &self.physics_settings;
        if settings.show_contacts {
            Gizmos::sphere_cat(
                buffer,
                position,
                0.05,
                settings.contact_color,
                "physics",
            );
        }
    }

    /// Draw transform axes
    pub fn draw_transform_axes(&self, buffer: &mut CommandBuffer, position: Vec3, scale: f32) {
        if !self.is_mode_active(VisualizationMode::Transforms) {
            return;
        }

        let settings = &self.transform_settings;
        if settings.show_axes {
            Gizmos::axes(buffer, position, scale * settings.axes_length);
        }
    }

    /// Draw hierarchy connection between parent and child
    pub fn draw_hierarchy_connection(
        &self,
        buffer: &mut CommandBuffer,
        parent_position: Vec3,
        child_position: Vec3,
    ) {
        if !self.is_mode_active(VisualizationMode::Transforms) {
            return;
        }

        let settings = &self.transform_settings;
        if settings.show_hierarchy {
            Gizmos::line_cat(
                buffer,
                parent_position,
                child_position,
                settings.hierarchy_color,
                "transforms",
            );
        }
    }

    /// Highlight a selected entity with a bounding sphere
    pub fn draw_entity_highlight(
        &self,
        buffer: &mut CommandBuffer,
        position: Vec3,
        radius: f32,
    ) {
        if !self.is_mode_active(VisualizationMode::Transforms) {
            return;
        }

        let settings = &self.transform_settings;
        Gizmos::sphere_cat(
            buffer,
            position,
            radius,
            settings.selection_color,
            "transforms",
        );
    }

    /// Draw entity with full transform visualization (axes + hierarchy + selection)
    pub fn draw_entity_transform(
        &self,
        buffer: &mut CommandBuffer,
        position: Vec3,
        parent_position: Option<Vec3>,
        is_selected: bool,
        scale: f32,
    ) {
        if !self.is_mode_active(VisualizationMode::Transforms) {
            return;
        }

        // Draw axes
        self.draw_transform_axes(buffer, position, scale);

        // Draw hierarchy connection if parent exists
        if let Some(parent_pos) = parent_position {
            self.draw_hierarchy_connection(buffer, parent_pos, position);
        }

        // Draw selection highlight if selected
        if is_selected {
            self.draw_entity_highlight(buffer, position, scale * 1.5);
        }
    }

    /// Draw bounding box
    pub fn draw_bounding_box(
        &self,
        buffer: &mut CommandBuffer,
        center: Vec3,
        half_extents: Vec3,
    ) {
        if !self.is_mode_active(VisualizationMode::Rendering) {
            return;
        }

        let settings = &self.rendering_settings;
        if settings.show_bounds {
            Gizmos::cube_cat(
                buffer,
                center,
                half_extents,
                settings.bounds_color,
                "rendering",
            );
        }
    }

    /// Draw audio source with volume visualization
    pub fn draw_audio_source(
        &self,
        buffer: &mut CommandBuffer,
        position: Vec3,
        attenuation_radius: f32,
    ) {
        self.draw_audio_source_with_volume(buffer, position, attenuation_radius, None);
    }

    /// Draw audio source with optional volume level visualization
    pub fn draw_audio_source_with_volume(
        &self,
        buffer: &mut CommandBuffer,
        position: Vec3,
        attenuation_radius: f32,
        volume: Option<f32>,
    ) {
        if !self.is_mode_active(VisualizationMode::Audio) {
            return;
        }

        let settings = &self.audio_settings;
        if settings.show_sources {
            // Draw source position with size based on volume if provided
            let source_radius = if let Some(vol) = volume {
                0.1 + (vol.clamp(0.0, 1.0) * 0.2) // Scale from 0.1 to 0.3 based on volume
            } else {
                0.1
            };

            Gizmos::sphere_cat(
                buffer,
                position,
                source_radius,
                settings.source_color,
                "audio",
            );

            // Draw attenuation range
            if settings.show_attenuation {
                Gizmos::circle(
                    buffer,
                    position,
                    attenuation_radius,
                    32,
                    settings.attenuation_color,
                );
            }

            // Draw volume indicator as a vertical bar if volume is provided
            if let Some(vol) = volume {
                if settings.show_volume {
                    let vol_clamped = vol.clamp(0.0, 1.0);
                    let bar_height = vol_clamped * 2.0; // Max height of 2 units
                    let bar_start = position + Vec3::new(0.0, 0.0, 0.0);
                    let bar_end = position + Vec3::new(0.0, bar_height, 0.0);
                    
                    // Color based on volume level (green to red)
                    let volume_color = Color::rgb(
                        vol_clamped,           // Red increases with volume
                        1.0 - vol_clamped,     // Green decreases with volume
                        0.0,
                    );
                    
                    Gizmos::line_cat(
                        buffer,
                        bar_start,
                        bar_end,
                        volume_color,
                        "audio",
                    );
                }
            }
        }
    }

    /// Draw text overlay (integrates with OverlayRenderer)
    pub fn draw_text_overlay(
        &self,
        overlay: &mut OverlayRenderer,
        x: f32,
        y: f32,
        text: &str,
        color: [f32; 4],
    ) {
        if !self.enabled {
            return;
        }
        overlay.draw_text(x, y, text, color, 1.0);
    }

    /// Draw status overlay showing active modes
    pub fn draw_status_overlay(&self, overlay: &mut OverlayRenderer, x: f32, y: f32) {
        if !self.enabled {
            return;
        }

        let mut y_offset = y;
        overlay.draw_text(x, y_offset, "Gizmos:", [1.0, 1.0, 1.0, 1.0], 1.0);
        y_offset += 12.0;

        for (mode, &active) in &self.active_modes {
            if active {
                let text = format!("  {} ON", mode.category());
                overlay.draw_text(x, y_offset, &text, [0.5, 1.0, 0.5, 1.0], 1.0);
                y_offset += 12.0;
            }
        }
    }
}

// ============================================================================
// Visualization Settings
// ============================================================================

/// Settings for physics visualization
#[derive(Debug, Clone)]
pub struct PhysicsVisualizationSettings {
    pub show_colliders: bool,
    pub show_velocities: bool,
    pub show_contacts: bool,
    pub collider_color: Color,
    pub velocity_color: Color,
    pub contact_color: Color,
    pub velocity_scale: f32,
}

impl Default for PhysicsVisualizationSettings {
    fn default() -> Self {
        Self {
            show_colliders: true,
            show_velocities: true,
            show_contacts: true,
            collider_color: Color::rgba(0.0, 1.0, 0.0, 0.5),
            velocity_color: Color::rgb(1.0, 1.0, 0.0),
            contact_color: Color::rgb(1.0, 0.0, 0.0),
            velocity_scale: 1.0,
        }
    }
}

/// Settings for rendering visualization
#[derive(Debug, Clone)]
pub struct RenderingVisualizationSettings {
    pub show_wireframe: bool,
    pub show_normals: bool,
    pub show_overdraw: bool,
    pub show_bounds: bool,
    pub wireframe_color: Color,
    pub normal_color: Color,
    pub bounds_color: Color,
}

impl Default for RenderingVisualizationSettings {
    fn default() -> Self {
        Self {
            show_wireframe: false,
            show_normals: false,
            show_overdraw: false,
            show_bounds: true,
            wireframe_color: Color::rgba(1.0, 1.0, 1.0, 0.3),
            normal_color: Color::rgb(0.0, 0.5, 1.0),
            bounds_color: Color::rgba(1.0, 0.5, 0.0, 0.5),
        }
    }
}

/// Settings for transform visualization
#[derive(Debug, Clone)]
pub struct TransformVisualizationSettings {
    pub show_axes: bool,
    pub show_hierarchy: bool,
    pub axes_length: f32,
    pub hierarchy_color: Color,
    pub selection_color: Color,
}

impl Default for TransformVisualizationSettings {
    fn default() -> Self {
        Self {
            show_axes: true,
            show_hierarchy: false,
            axes_length: 1.0,
            hierarchy_color: Color::rgba(0.5, 0.5, 1.0, 0.5),
            selection_color: Color::rgba(1.0, 1.0, 0.0, 0.3),
        }
    }
}

/// Settings for audio visualization
#[derive(Debug, Clone)]
pub struct AudioVisualizationSettings {
    pub show_sources: bool,
    pub show_attenuation: bool,
    pub show_volume: bool,
    pub source_color: Color,
    pub attenuation_color: Color,
}

impl Default for AudioVisualizationSettings {
    fn default() -> Self {
        Self {
            show_sources: true,
            show_attenuation: true,
            show_volume: true,
            source_color: Color::rgb(0.0, 1.0, 1.0),
            attenuation_color: Color::rgba(0.0, 1.0, 1.0, 0.2),
        }
    }
}
