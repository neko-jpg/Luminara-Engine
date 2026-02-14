use luminara_ai_agent::{AiContextEngine, schema::ComponentSchema};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_context_generation_completeness() {
    use luminara_core::world::World;
    let mut world = World::new();
    let _ = world.spawn();

    let mut engine = AiContextEngine::new();
    engine.schema_service_mut().register_schema(ComponentSchema {
        name: "TestComp".into(),
        description: "Desc".into(),
        category: "Test".into(),
        fields: vec![],
    });

    let context = engine.generate_context("query", 1000, &world);

    assert!(context.summary.contains("Entities: 1"));
    assert!(context.schemas.contains("Test: TestComp"));
    assert!(context.catalog.contains("Entity Catalog"));
}

#[quickcheck]
fn test_context_schema_prioritization(schema_name: String) -> TestResult {
    if schema_name.is_empty() { return TestResult::discard(); }

    use luminara_core::world::World;
    let world = World::new();

    let mut engine = AiContextEngine::new();
    engine.schema_service_mut().register_schema(ComponentSchema {
        name: schema_name.clone(),
        description: "Desc".into(),
        category: "Test".into(),
        fields: vec![],
    });

    // Even if query is irrelevant, L0 schemas should be included in MVP
    let context = engine.generate_context("irrelevant", 1000, &world);

    TestResult::from_bool(context.schemas.contains(&schema_name))
}
