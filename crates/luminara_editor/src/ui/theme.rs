//! Theme system for Luminara Editor (Vizia version)
//!
//! Defines color palettes, spacing scales, typography scales, and border radius values
//! matching the HTML prototypes for consistent styling across the editor UI.

use std::sync::Arc;
use vizia::prelude::*;

#[derive(Debug, Clone)]
pub struct Theme {
    pub colors: ColorPalette,
    pub spacing: SpacingScale,
    pub typography: TypographyScale,
    pub borders: BorderScale,
    pub transitions: TransitionConfig,
}

impl Theme {
    pub fn default_dark() -> Self {
        Self {
            colors: ColorPalette::default_dark(),
            spacing: SpacingScale::default(),
            typography: TypographyScale::default(),
            borders: BorderScale::default(),
            transitions: TransitionConfig::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_dark()
    }
}

#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub border: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub canvas_background: Color,
    pub node_background: Color,
    pub node_border: Color,
    pub node_selected: Color,
    pub port_input: Color,
    pub port_output: Color,
    pub port_true: Color,
    pub port_false: Color,
    pub grid_dot: Color,
    pub toolbar_bg: Color,
    pub toolbar_active: Color,
    pub panel_header: Color,
    pub condition_bg: Color,
}

impl ColorPalette {
    pub fn default_dark() -> Self {
        Self {
            background: Color::rgb(0x1a, 0x1a, 0x1a),
            surface: Color::rgb(0x2a, 0x2a, 0x2a),
            surface_hover: Color::rgb(0x33, 0x33, 0x33),
            surface_active: Color::rgb(0x3a, 0x3a, 0x3a),
            border: Color::rgb(0x3a, 0x3a, 0x3a),
            text: Color::rgb(0xe0, 0xe0, 0xe0),
            text_secondary: Color::rgb(0xa0, 0xa0, 0xa0),
            accent: Color::rgb(0x8a, 0x8a, 0xff),
            accent_hover: Color::rgb(0xa0, 0xa0, 0xff),
            error: Color::rgb(0xff, 0x55, 0x55),
            warning: Color::rgb(0xff, 0xaa, 0x55),
            success: Color::rgb(0x55, 0xff, 0x55),
            canvas_background: Color::rgb(0x1e, 0x1e, 0x1e),
            node_background: Color::rgb(0x2d, 0x2d, 0x2d),
            node_border: Color::rgb(0x4a, 0x4a, 0x4a),
            node_selected: Color::rgb(0x8a, 0x8a, 0xff),
            port_input: Color::rgb(0xff, 0xaa, 0x8a),
            port_output: Color::rgb(0x8a, 0x8a, 0xff),
            port_true: Color::rgb(0x4c, 0xaf, 0x50),
            port_false: Color::rgb(0xf4, 0x43, 0x36),
            grid_dot: Color::rgb(0x3a, 0x3a, 0x3a),
            toolbar_bg: Color::rgb(0x2d, 0x2d, 0x2d),
            toolbar_active: Color::rgb(0x3a, 0x5f, 0x8a),
            panel_header: Color::rgb(0x32, 0x32, 0x32),
            condition_bg: Color::rgb(0x2a, 0x2a, 0x3a),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpacingScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl SpacingScale {
    pub fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 20.0,
            xxl: 24.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypographyScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub ml: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl TypographyScale {
    pub fn default() -> Self {
        Self {
            xs: 10.0,
            sm: 11.0,
            md: 12.0,
            ml: 13.0,
            lg: 14.0,
            xl: 16.0,
            xxl: 18.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorderScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub rounded: f32,
}

impl BorderScale {
    pub fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 6.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            xxl: 20.0,
            rounded: 28.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TransitionConfig {
    pub hover_duration_ms: u64,
    pub active_duration_ms: u64,
    pub focus_duration_ms: u64,
}

impl TransitionConfig {
    pub fn default() -> Self {
        Self {
            hover_duration_ms: 100,
            active_duration_ms: 50,
            focus_duration_ms: 150,
        }
    }
}
