use luminara_core::shared_types::{App, Plugin, CoreStage, Res, ResMut, AppInterface};
use luminara_platform::time::Time;
use crate::logging::{init_logging, info};
use crate::frame_stats::FrameStats;
use crate::diagnostics::Diagnostics;

pub struct DiagnosticPlugin;

impl Plugin for DiagnosticPlugin {
    fn name(&self) -> &str {
        "DiagnosticPlugin"
    }

    fn build(&self, app: &mut App) {
        init_logging();
        app.insert_resource(FrameStats::default())
            .insert_resource(Diagnostics::new())
            .add_system(CoreStage::PostRender, update_frame_stats_system);
    }
}

pub fn update_frame_stats_system(mut stats: ResMut<'_, FrameStats>, time: Res<'_, Time>) {
    stats.fps = time.fps();
    let frame_time_ms = time.delta_seconds() * 1000.0;
    stats.frame_time_ms = frame_time_ms;
    stats.frame_time_history.push_back(frame_time_ms);
    if stats.frame_time_history.len() > stats.max_history {
        stats.frame_time_history.pop_front();
    }
}

/// FPS display system (for development)
pub fn log_fps_system(stats: Res<'_, FrameStats>, time: Res<'_, Time>) {
    if time.frame_count() % 60 == 0 {
        info!(
            "FPS: {:.1} | Frame: {:.2}ms",
            stats.average_fps(),
            stats.frame_time_ms
        );
    }
}
