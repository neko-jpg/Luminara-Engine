//! Theme system for Luminara Editor
//!
//! Defines color palettes, spacing scales, typography scales, and border radius values
//! matching the HTML prototypes for consistent styling across the editor UI.

use gpui::{Hsla, Pixels, px, rgb};

/// Main theme struct containing all styling information
#[derive(Debug, Clone)]
pub struct Theme {
    pub colors: ColorPalette,
    pub spacing: SpacingScale,
    pub typography: TypographyScale,
    pub borders: BorderScale,
    pub transitions: TransitionConfig,
}

impl Theme {
    /// Create the default dark theme matching HTML prototypes
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

/// Color palette matching the HTML prototypes
#[derive(Debug, Clone)]
pub struct ColorPalette {
    /// Main background color (#1a1a1a)
    pub background: Hsla,
    /// Surface color for panels (#2a2a2a)
    pub surface: Hsla,
    /// Surface hover state
    pub surface_hover: Hsla,
    /// Surface active/selected state
    pub surface_active: Hsla,
    /// Border color
    pub border: Hsla,
    /// Primary text color
    pub text: Hsla,
    /// Secondary text color (dimmed)
    pub text_secondary: Hsla,
    /// Accent color (#8a8aff - blue/purple)
    pub accent: Hsla,
    /// Accent hover state
    pub accent_hover: Hsla,
    /// Error color (red)
    pub error: Hsla,
    /// Warning color (yellow/orange)
    pub warning: Hsla,
    /// Success color (green)
    pub success: Hsla,
    
    // Logic Graph specific colors
    /// Canvas background color (#1e1e1e)
    pub canvas_background: Hsla,
    /// Node background color (#2d2d2d)
    pub node_background: Hsla,
    /// Node border color (#4a4a4a)
    pub node_border: Hsla,
    /// Selected node border color (#8a8aff)
    pub node_selected: Hsla,
    /// Input port color (#ffaa8a - orange)
    pub port_input: Hsla,
    /// Output port color (#8a8aff - blue)
    pub port_output: Hsla,
    /// True/Yes port color (#4caf50 - green)
    pub port_true: Hsla,
    /// False/No port color (#f44336 - red)
    pub port_false: Hsla,
    /// Grid dot color (#3a3a3a)
    pub grid_dot: Hsla,
    /// Toolbar background (#2d2d2d)
    pub toolbar_bg: Hsla,
    /// Toolbar button active (#3a5f8a)
    pub toolbar_active: Hsla,
    /// Panel header background (#323232)
    pub panel_header: Hsla,
    /// Condition builder background (#2a2a3a)
    pub condition_bg: Hsla,
}

impl ColorPalette {
    /// Create the default dark color palette matching HTML prototypes
    pub fn default_dark() -> Self {
        Self {
            // Background: #1a1a1a (very dark gray)
            background: rgb(0x1a1a1a).into(),
            // Surface: #2a2a2a (dark gray)
            surface: rgb(0x2a2a2a).into(),
            // Surface hover: slightly lighter
            surface_hover: rgb(0x333333).into(),
            // Surface active: #3a3a3a
            surface_active: rgb(0x3a3a3a).into(),
            // Border: #3a3a3a
            border: rgb(0x3a3a3a).into(),
            // Text: #e0e0e0 (light gray)
            text: rgb(0xe0e0e0).into(),
            // Text secondary: #a0a0a0 (medium gray)
            text_secondary: rgb(0xa0a0a0).into(),
            // Accent: #8a8aff (blue/purple)
            accent: rgb(0x8a8aff).into(),
            // Accent hover: lighter blue/purple
            accent_hover: rgb(0xa0a0ff).into(),
            // Error: #ff5555 (red)
            error: rgb(0xff5555).into(),
            // Warning: #ffaa55 (orange)
            warning: rgb(0xffaa55).into(),
            // Success: #55ff55 (green)
            success: rgb(0x55ff55).into(),
            
            // Logic Graph specific colors
            // Canvas background: #1e1e1e
            canvas_background: rgb(0x1e1e1e).into(),
            // Node background: #2d2d2d
            node_background: rgb(0x2d2d2d).into(),
            // Node border: #4a4a4a
            node_border: rgb(0x4a4a4a).into(),
            // Selected node: #8a8aff
            node_selected: rgb(0x8a8aff).into(),
            // Input port: #ffaa8a (orange)
            port_input: rgb(0xffaa8a).into(),
            // Output port: #8a8aff (blue)
            port_output: rgb(0x8a8aff).into(),
            // True port: #4caf50 (green)
            port_true: rgb(0x4caf50).into(),
            // False port: #f44336 (red)
            port_false: rgb(0xf44336).into(),
            // Grid dot: #3a3a3a
            grid_dot: rgb(0x3a3a3a).into(),
            // Toolbar background: #2d2d2d
            toolbar_bg: rgb(0x2d2d2d).into(),
            // Toolbar active: #3a5f8a
            toolbar_active: rgb(0x3a5f8a).into(),
            // Panel header: #323232
            panel_header: rgb(0x323232).into(),
            // Condition builder: #2a2a3a
            condition_bg: rgb(0x2a2a3a).into(),
        }
    }
}

/// Spacing scale with consistent values
#[derive(Debug, Clone)]
pub struct SpacingScale {
    /// Extra small: 4px
    pub xs: Pixels,
    /// Small: 8px
    pub sm: Pixels,
    /// Medium: 12px
    pub md: Pixels,
    /// Large: 16px
    pub lg: Pixels,
    /// Extra large: 20px
    pub xl: Pixels,
    /// Extra extra large: 24px
    pub xxl: Pixels,
}

impl SpacingScale {
    /// Create the default spacing scale
    pub fn default() -> Self {
        Self {
            xs: px(4.0),
            sm: px(8.0),
            md: px(12.0),
            lg: px(16.0),
            xl: px(20.0),
            xxl: px(24.0),
        }
    }
}

/// Typography scale with consistent font sizes
#[derive(Debug, Clone)]
pub struct TypographyScale {
    /// Extra small: 10px
    pub xs: Pixels,
    /// Small: 11px
    pub sm: Pixels,
    /// Medium: 12px
    pub md: Pixels,
    /// Medium-large: 13px
    pub ml: Pixels,
    /// Large: 14px
    pub lg: Pixels,
    /// Extra large: 16px
    pub xl: Pixels,
    /// Extra extra large: 18px
    pub xxl: Pixels,
}

impl TypographyScale {
    /// Create the default typography scale
    pub fn default() -> Self {
        Self {
            xs: px(10.0),
            sm: px(11.0),
            md: px(12.0),
            ml: px(13.0),
            lg: px(14.0),
            xl: px(16.0),
            xxl: px(18.0),
        }
    }
}

/// Border radius scale with consistent values
#[derive(Debug, Clone)]
pub struct BorderScale {
    /// Extra small: 4px
    pub xs: Pixels,
    /// Small: 6px
    pub sm: Pixels,
    /// Medium: 8px
    pub md: Pixels,
    /// Large: 12px
    pub lg: Pixels,
    /// Extra large: 16px
    pub xl: Pixels,
    /// Extra extra large: 20px
    pub xxl: Pixels,
    /// Rounded: 28px
    pub rounded: Pixels,
}

impl BorderScale {
    /// Create the default border scale
    pub fn default() -> Self {
        Self {
            xs: px(4.0),
            sm: px(6.0),
            md: px(8.0),
            lg: px(12.0),
            xl: px(16.0),
            xxl: px(20.0),
            rounded: px(28.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::default_dark();
        
        // Verify spacing scale
        assert_eq!(theme.spacing.xs, px(4.0));
        assert_eq!(theme.spacing.sm, px(8.0));
        assert_eq!(theme.spacing.md, px(12.0));
        assert_eq!(theme.spacing.lg, px(16.0));
        assert_eq!(theme.spacing.xl, px(20.0));
        assert_eq!(theme.spacing.xxl, px(24.0));
        
        // Verify typography scale
        assert_eq!(theme.typography.xs, px(10.0));
        assert_eq!(theme.typography.sm, px(11.0));
        assert_eq!(theme.typography.md, px(12.0));
        assert_eq!(theme.typography.ml, px(13.0));
        assert_eq!(theme.typography.lg, px(14.0));
        assert_eq!(theme.typography.xl, px(16.0));
        assert_eq!(theme.typography.xxl, px(18.0));
        
        // Verify border scale
        assert_eq!(theme.borders.xs, px(4.0));
        assert_eq!(theme.borders.sm, px(6.0));
        assert_eq!(theme.borders.md, px(8.0));
        assert_eq!(theme.borders.lg, px(12.0));
        assert_eq!(theme.borders.xl, px(16.0));
        assert_eq!(theme.borders.xxl, px(20.0));
        assert_eq!(theme.borders.rounded, px(28.0));
        
        // Verify transition config
        assert_eq!(theme.transitions.hover_duration_ms, 100);
        assert_eq!(theme.transitions.active_duration_ms, 50);
        assert_eq!(theme.transitions.focus_duration_ms, 150);
    }

    #[test]
    fn test_color_palette() {
        let colors = ColorPalette::default_dark();
        
        // Verify colors are within valid ranges
        assert!(colors.background.l >= 0.0 && colors.background.l <= 1.0);
        assert!(colors.surface.l >= 0.0 && colors.surface.l <= 1.0);
        assert!(colors.text.l >= 0.0 && colors.text.l <= 1.0);
        
        // Verify background is darker than surface
        assert!(colors.background.l < colors.surface.l);
        
        // Verify text is lighter than background
        assert!(colors.text.l > colors.background.l);
    }
}


/// Transition configuration for hover states and animations
#[derive(Debug, Clone, Copy)]
pub struct TransitionConfig {
    /// Duration for hover state transitions (100ms)
    pub hover_duration_ms: u64,
    /// Duration for active state transitions (50ms)
    pub active_duration_ms: u64,
    /// Duration for focus state transitions (150ms)
    pub focus_duration_ms: u64,
}

impl TransitionConfig {
    /// Create the default transition configuration
    pub fn default() -> Self {
        Self {
            hover_duration_ms: 100,
            active_duration_ms: 50,
            focus_duration_ms: 150,
        }
    }
}

/// Helper trait for applying hover states to colors
pub trait HoverState {
    /// Apply hover state to a color (slightly lighter)
    fn hover(&self) -> Self;
    /// Apply active state to a color (even lighter)
    fn active(&self) -> Self;
}

impl HoverState for Hsla {
    fn hover(&self) -> Self {
        // Increase lightness by 10% for hover state
        Hsla {
            h: self.h,
            s: self.s,
            l: (self.l + 0.1).min(1.0),
            a: self.a,
        }
    }
    
    fn active(&self) -> Self {
        // Increase lightness by 15% for active state
        Hsla {
            h: self.h,
            s: self.s,
            l: (self.l + 0.15).min(1.0),
            a: self.a,
        }
    }
}


    #[test]
    fn test_hover_state() {
        use crate::ui::theme::HoverState;
        
        let theme = Theme::default_dark();
        let base_color = theme.colors.surface;
        
        // Test hover state
        let hover_color = base_color.hover();
        assert!(hover_color.l > base_color.l, "Hover should be lighter");
        assert_eq!(hover_color.h, base_color.h, "Hue should not change");
        assert_eq!(hover_color.s, base_color.s, "Saturation should not change");
        
        // Test active state
        let active_color = base_color.active();
        assert!(active_color.l > hover_color.l, "Active should be lighter than hover");
        assert_eq!(active_color.h, base_color.h, "Hue should not change");
        assert_eq!(active_color.s, base_color.s, "Saturation should not change");
    }
