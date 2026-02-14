#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseSmoothing {
    /// Smoothing factor between 0.0 (no smoothing) and 1.0 (full smoothing/lag).
    /// Typical values are 0.5 - 0.9.
    pub factor: f32,
    pub smoothed_delta: crate::Vec2,
}

impl Default for MouseSmoothing {
    fn default() -> Self {
        Self {
            factor: 0.0,
            smoothed_delta: crate::Vec2::ZERO,
        }
    }
}

impl MouseSmoothing {
    pub fn new(factor: f32) -> Self {
        Self {
            factor: factor.clamp(0.0, 0.999),
            smoothed_delta: crate::Vec2::ZERO,
        }
    }

    pub fn update(&mut self, raw_delta: crate::Vec2) -> crate::Vec2 {
        if self.factor <= std::f32::EPSILON {
            self.smoothed_delta = raw_delta;
        } else {
            // Exponential Moving Average
            // smoothed = alpha * raw + (1 - alpha) * smoothed_prev
            // where alpha = 1 - factor
            let alpha = 1.0 - self.factor;
            self.smoothed_delta = self.smoothed_delta.lerp(raw_delta, alpha);
        }
        self.smoothed_delta
    }

    pub fn reset(&mut self) {
        self.smoothed_delta = crate::Vec2::ZERO;
    }
}
