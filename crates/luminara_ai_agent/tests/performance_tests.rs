use luminara_ai_agent::{PerformanceAdvisor, ImpactSeverity, intent_resolver::{AiIntent, EntityReference, RelativePosition}};
use luminara_math::Vec3;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_impact_estimation_basic() {
    use luminara_core::world::World;
    let world = World::new();
    let advisor = PerformanceAdvisor::new();

    // Simulate spawn intent
    let intent = AiIntent::SpawnRelative {
        anchor: EntityReference::ByName("test".into()),
        offset: RelativePosition::AtOffset(Vec3::ZERO),
        template: "cube".into(),
    };

    let impact = advisor.estimate_impact(&intent, &world);

    // Should be low impact for single spawn
    assert_eq!(impact.severity, ImpactSeverity::Low);
    assert!(impact.predicted_fps > 0.0);
}

#[quickcheck]
fn test_performance_context_completeness(fps: f32) -> TestResult {
    if !fps.is_finite() || fps < 0.0 { return TestResult::discard(); }

    use luminara_core::world::World;
    let world = World::new();
    let mut advisor = PerformanceAdvisor::new();

    advisor.update_metrics(&world, fps);
    let ctx = advisor.generate_context();

    // Should contain FPS and Entity count
    TestResult::from_bool(ctx.contains("FPS") && ctx.contains("Entities"))
}
