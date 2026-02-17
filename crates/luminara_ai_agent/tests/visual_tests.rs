use luminara_ai_agent::{CaptureConfig, VisualFeedbackSystem};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn test_capture_format_compliance(w: u32, h: u32) -> TestResult {
    // Avoid too large allocations for tests
    if w == 0 || h == 0 || w > 1024 || h > 1024 {
        return TestResult::discard();
    }

    use luminara_core::world::World;
    let world = World::new();
    let sys = VisualFeedbackSystem::new();

    let config = CaptureConfig {
        resolution: (w, h),
        ..Default::default()
    };

    let data = sys.capture_viewport(&world, config);
    TestResult::from_bool(data.image_data.len() == (w * h * 3) as usize)
}

#[quickcheck]
fn test_compression_efficiency(target_reduction: f32) -> TestResult {
    if target_reduction <= 0.0 || target_reduction >= 1.0 {
        return TestResult::discard();
    }

    let sys = VisualFeedbackSystem::new();
    let original = vec![0u8; 1000];

    let compressed = sys.compress_image(&original, target_reduction);

    TestResult::from_bool(compressed.len() < original.len())
}
