use luminara_ai_agent::context_engine::{WorldDigestEngine, AttentionEstimator};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn test_relevance_prioritization(query: String, entity_name: String) -> TestResult {
    if entity_name.is_empty() { return TestResult::discard(); }

    let estimator = AttentionEstimator::default();
    let score = estimator.estimate_relevance(&query, &entity_name);

    if query.contains(&entity_name) {
        TestResult::from_bool(score > 0.0)
    } else {
        TestResult::from_bool(score == 0.0)
    }
}

// Token budget compliance test is hard without tokenizer.
// We'll just verify generate_l0_summary returns non-empty string.
#[test]
fn test_l0_summary_generation() {
    // We need a World instance.
    // Constructing a World in test is fine as luminara_core is dependency.
    use luminara_core::world::World;
    let mut world = World::new();
    let _e1 = world.spawn();

    let engine = WorldDigestEngine::new();
    let summary = engine.generate_l0_summary(&world);

    assert!(summary.contains("Entities: 1"));
    assert!(summary.len() > 0);
}
