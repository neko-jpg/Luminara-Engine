use luminara_ai_agent::{SchemaDiscoveryService, schema::{ComponentSchema, FieldSchema}};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[test]
fn test_schema_detail_progression() {
    let mut service = SchemaDiscoveryService::new();
    let schema = ComponentSchema {
        name: "TestComp".into(),
        description: "A test component".into(),
        category: "Test".into(),
        fields: vec![
            FieldSchema { name: "val".into(), type_name: "f32".into() }
        ],
    };
    service.register_schema(schema);

    let l0 = service.get_l0_schema();
    assert!(l0.contains("Test: TestComp"));

    let l1 = service.get_l1_schema("TestComp").unwrap();
    assert!(l1.contains("val: f32"));
    assert!(!l1.contains("A test component")); // L1 usually brief? In my impl, L1 has fields. L2 has description.

    let l2 = service.get_l2_schema("TestComp").unwrap();
    assert!(l2.contains("val: f32"));
    assert!(l2.contains("A test component"));
}

#[quickcheck]
fn test_schema_completeness(name: String, category: String) -> TestResult {
    if name.is_empty() || category.is_empty() { return TestResult::discard(); }

    let mut service = SchemaDiscoveryService::new();
    let schema = ComponentSchema {
        name: name.clone(),
        description: "desc".into(),
        category: category.clone(),
        fields: vec![],
    };
    service.register_schema(schema);

    let l0 = service.get_l0_schema();
    // Should contain category and name
    TestResult::from_bool(l0.contains(&category) && l0.contains(&name))
}
