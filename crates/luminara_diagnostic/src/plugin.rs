use crate::diagnostics::Diagnostics;
use crate::frame_stats::FrameStats;
use crate::logging::init_logging;
use luminara_core::shared_types::{App, AppInterface, CoreStage, Plugin, Res, ResMut};
use luminara_core::system::FunctionMarker;
use luminara_platform::time::Time;

pub struct DiagnosticPlugin;

impl Plugin for DiagnosticPlugin {
    fn name(&self) -> &str {
        "DiagnosticPlugin"
    }

    fn build(&self, app: &mut App) {
        init_logging();
        app.insert_resource(FrameStats::default())
            .insert_resource(Diagnostics::new())
            .add_system::<(
                FunctionMarker,
                ResMut<'static, FrameStats>,
                Res<'static, Time>,
            )>(CoreStage::PostRender, update_frame_stats_system);
    }
}

pub fn update_frame_stats_system(mut stats: ResMut<FrameStats>, time: Res<Time>) {
    stats.fps = time.fps();
    let frame_time_ms = time.delta_seconds() * 1000.0;
    stats.frame_time_ms = frame_time_ms;
    stats.frame_time_history.push_back(frame_time_ms);
    if stats.frame_time_history.len() > stats.max_history {
        stats.frame_time_history.pop_front();
    }
}
