//! # Target Game System
//!
//! Implements a target shooting mini-game with crosshair rendering,
//! raycast hit detection, score tracking, and visual feedback.

use luminara_core::{Component, Resource};
use luminara_math::{Color, Vec3};

/// Marks an entity as a shootable target.
#[derive(Debug, Clone)]
pub struct Target {
    /// Point value when hit.
    pub points: u32,
    /// Whether this target has been hit.
    pub hit: bool,
    /// Visual feedback timer (seconds remaining for hit flash).
    pub hit_flash_timer: f32,
    /// Original color (for restoration after flash).
    pub original_color: Color,
}

impl Component for Target {
    fn type_name() -> &'static str {
        "Target"
    }
}

impl Default for Target {
    fn default() -> Self {
        Self {
            points: 10,
            hit: false,
            hit_flash_timer: 0.0,
            original_color: Color::rgb(1.0, 0.2, 0.2),
        }
    }
}

/// Global score tracker for the target game.
#[derive(Debug, Clone)]
pub struct TargetGameState {
    /// Total score accumulated.
    pub score: u32,
    /// Number of shots fired.
    pub shots_fired: u32,
    /// Number of hits.
    pub hits: u32,
    /// Whether the target game mode is active.
    pub active: bool,
    /// Hit feedback message + timer.
    pub feedback_message: String,
    pub feedback_timer: f32,
    /// Crosshair visibility.
    pub show_crosshair: bool,
    /// Last hit position (for visual effects).
    pub last_hit_pos: Option<Vec3>,
}

impl Resource for TargetGameState {}

impl Default for TargetGameState {
    fn default() -> Self {
        Self {
            score: 0,
            shots_fired: 0,
            hits: 0,
            active: false,
            feedback_message: String::new(),
            feedback_timer: 0.0,
            show_crosshair: true,
            last_hit_pos: None,
        }
    }
}

impl TargetGameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle the target game on/off.
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if self.active {
            self.reset();
        }
    }

    /// Reset scores.
    pub fn reset(&mut self) {
        self.score = 0;
        self.shots_fired = 0;
        self.hits = 0;
        self.feedback_message.clear();
        self.feedback_timer = 0.0;
    }

    /// Record a hit.
    pub fn register_hit(&mut self, points: u32, position: Vec3) {
        self.hits += 1;
        self.score += points;
        self.last_hit_pos = Some(position);
        self.feedback_message = format!("+{} HIT!", points);
        self.feedback_timer = 1.5;
    }

    /// Record a miss.
    pub fn register_miss(&mut self) {
        self.shots_fired += 1;
        self.feedback_message = "MISS".to_string();
        self.feedback_timer = 0.8;
    }

    /// Record a shot (called on every click).
    pub fn record_shot(&mut self) {
        self.shots_fired += 1;
    }

    /// Update timers.
    pub fn update(&mut self, dt: f32) {
        if self.feedback_timer > 0.0 {
            self.feedback_timer -= dt;
            if self.feedback_timer <= 0.0 {
                self.feedback_message.clear();
            }
        }
    }

    /// Get accuracy percentage.
    pub fn accuracy(&self) -> f32 {
        if self.shots_fired == 0 {
            0.0
        } else {
            (self.hits as f32 / self.shots_fired as f32) * 100.0
        }
    }
}

/// Draw crosshair on the overlay renderer.
/// Call this from the HUD system with screen dimensions.
pub fn draw_crosshair(
    overlay: &mut luminara_render::OverlayRenderer,
    screen_w: f32,
    screen_h: f32,
    color: [f32; 4],
) {
    let cx = screen_w * 0.5;
    let cy = screen_h * 0.5;
    let size = 12.0;
    let thickness = 2.0;
    let gap = 3.0;

    // Horizontal lines (left and right of center with gap)
    overlay.draw_rect(
        cx - size,
        cy - thickness * 0.5,
        size - gap,
        thickness,
        color,
    );
    overlay.draw_rect(cx + gap, cy - thickness * 0.5, size - gap, thickness, color);

    // Vertical lines (top and bottom of center with gap)
    overlay.draw_rect(
        cx - thickness * 0.5,
        cy - size,
        thickness,
        size - gap,
        color,
    );
    overlay.draw_rect(cx - thickness * 0.5, cy + gap, thickness, size - gap, color);

    // Center dot
    overlay.draw_rect(cx - 1.0, cy - 1.0, 2.0, 2.0, color);
}

/// Draw target game HUD (score, accuracy, feedback).
pub fn draw_target_hud(
    overlay: &mut luminara_render::OverlayRenderer,
    state: &TargetGameState,
    screen_w: f32,
    _screen_h: f32,
) {
    let scale = 1.2;
    let cw = 8.0 * scale;
    let ch = 8.0 * scale;

    // Score panel (top-center)
    let score_text = format!(
        "SCORE: {} | HITS: {}/{} | ACC: {:.0}%",
        state.score,
        state.hits,
        state.shots_fired,
        state.accuracy()
    );
    let panel_w = score_text.len() as f32 * cw + 24.0;
    let panel_x = (screen_w - panel_w) * 0.5;
    let panel_y = 10.0;

    overlay.draw_gradient_rect(
        panel_x,
        panel_y,
        panel_w,
        ch + 14.0,
        [0.1, 0.0, 0.2, 0.9],
        [0.0, 0.0, 0.1, 0.7],
    );
    overlay.draw_text_outlined(
        panel_x + 12.0,
        panel_y + 7.0,
        &score_text,
        [1.0, 0.9, 0.2, 1.0],
        [0.0, 0.0, 0.0, 1.0],
        scale,
    );

    // Hit feedback (center of screen, fading)
    if state.feedback_timer > 0.0 {
        let alpha = (state.feedback_timer / 1.5).min(1.0);
        let is_hit = state.feedback_message.contains("HIT");
        let color = if is_hit {
            [0.2, 1.0, 0.2, alpha]
        } else {
            [1.0, 0.3, 0.3, alpha]
        };
        let msg = &state.feedback_message;
        let msg_w = msg.len() as f32 * cw * 1.5;
        overlay.draw_text_outlined(
            (screen_w - msg_w) * 0.5,
            80.0,
            msg,
            color,
            [0.0, 0.0, 0.0, alpha],
            scale * 1.5,
        );
    }
}
