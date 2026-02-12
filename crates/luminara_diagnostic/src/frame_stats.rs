use luminara_core::shared_types::Resource;
use std::collections::VecDeque;

pub struct FrameStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub frame_time_history: VecDeque<f32>,
    pub max_history: usize,
}

impl Resource for FrameStats {}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            frame_time_history: VecDeque::with_capacity(120),
            max_history: 120,
        }
    }
}

impl FrameStats {
    pub fn average_fps(&self) -> f32 {
        if self.frame_time_history.is_empty() {
            return 0.0;
        }
        let avg_ms: f32 = self.frame_time_history.iter().sum::<f32>() / self.frame_time_history.len() as f32;
        if avg_ms > 0.0 {
            1000.0 / avg_ms
        let avg_frame_time: f32 = self.frame_time_history.iter().sum::<f32>() / self.frame_time_history.len() as f32;
        if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Returns the p-th percentile frame time in milliseconds.
    /// p should be in range [0.0, 100.0].
    /// Calculates the percentile frame time in milliseconds.
    /// `p` should be between 0.0 and 100.0 (e.g. 99.0 for P99).
    pub fn percentile_frame_time(&self, p: f32) -> f32 {
        if self.frame_time_history.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f32> = self.frame_time_history.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((p / 100.0) * (sorted.len() as f32 - 1.0)).round() as usize;
        let mut sorted = self.frame_time_history.iter().cloned().collect::<Vec<f32>>();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p = p.clamp(0.0, 100.0);
        let index = ((p / 100.0) * (sorted.len() - 1) as f32).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}
