//! Integration test for ReflectRegistry functionality
//! Validates Requirements 7.2: Schema information for all registered components

use luminara_core::{Reflect, ReflectRegistry, TypeKind};

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Velocity {
    dx: f32,
    dy: f32,
    dz: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Health {
    current: f32,
    maximum: f32,
}

#[test]
fn test_registry_provides_complete_schema_information() {
    // **Validates: Requirements 7.2**
    // WHEN querying component metadata, THE System SHALL provide schema
    // information for all registered components

    let mut registry = ReflectRegistry::new();

    // Register multiple component types
    registry.register::<Position>();
    registry.register::<Velocity>();
    registry.register::<Health>();

    assert_eq!(registry.len(), 3, "Registry should contain 3 components");

    // Verify Position component schema
    let position_info = registry
        .get_type_info_by_name("luminara_core::Position")
        .expect("Position should be registered");

    assert!(position_info.type_name.contains("Position"));
    assert_eq!(position_info.kind, TypeKind::Struct);
    assert_eq!(position_info.fields.len(), 3);

    let field_names: Vec<_> = position_info
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert!(field_names.contains(&"x"));
    assert!(field_names.contains(&"y"));
    assert!(field_names.contains(&"z"));

    // Verify all fields have type information
    for field in &position_info.fields {
        assert!(!field.name.is_empty(), "Field name should not be empty");
        assert!(
            !field.type_name.is_empty(),
            "Field type name should not be empty"
        );
        assert!(
            field.type_name.contains("f32"),
            "Position fields should be f32"
        );
    }

    // Verify Velocity component schema
    let velocity_info = registry
        .get_type_info_by_name("luminara_core::Velocity")
        .expect("Velocity should be registered");

    assert!(velocity_info.type_name.contains("Velocity"));
    assert_eq!(velocity_info.kind, TypeKind::Struct);
    assert_eq!(velocity_info.fields.len(), 3);

    // Verify Health component schema
    let health_info = registry
        .get_type_info_by_name("luminara_core::Health")
        .expect("Health should be registered");

    assert!(health_info.type_name.contains("Health"));
    assert_eq!(health_info.kind, TypeKind::Struct);
    assert_eq!(health_info.fields.len(), 2);

    let health_field_names: Vec<_> = health_info.fields.iter().map(|f| f.name.as_str()).collect();
    assert!(health_field_names.contains(&"current"));
    assert!(health_field_names.contains(&"maximum"));
}

#[test]
fn test_runtime_type_construction() {
    // **Validates: Requirements 7.2**
    // Test that registered types can be constructed at runtime

    let mut registry = ReflectRegistry::new();
    registry.register::<Position>();
    registry.register::<Velocity>();
    registry.register::<Health>();

    // Construct Position by name
    let position = registry
        .construct("luminara_core::Position")
        .expect("Should construct Position");

    assert!(position.type_info().type_name.contains("Position"));

    // Verify constructed instance has correct default values
    let type_info = position.type_info();
    assert_eq!(type_info.fields.len(), 3);

    // Access fields through reflection
    let x_field = position.field("x").expect("Should have x field");
    let x_value = x_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*x_value, 0.0, "Default value should be 0.0");

    // Construct Velocity by name
    let velocity = registry
        .construct("luminara_core::Velocity")
        .expect("Should construct Velocity");

    assert!(velocity.type_info().type_name.contains("Velocity"));

    // Construct Health by name
    let health = registry
        .construct("luminara_core::Health")
        .expect("Should construct Health");

    assert!(health.type_info().type_name.contains("Health"));
}

#[test]
fn test_type_lookup_by_name() {
    // **Validates: Requirements 7.2**
    // Test that types can be looked up by name

    let mut registry = ReflectRegistry::new();
    registry.register::<Position>();
    registry.register::<Velocity>();
    registry.register::<Health>();

    // Lookup by exact name
    assert!(registry
        .get_type_info_by_name("luminara_core::Position")
        .is_some());
    assert!(registry
        .get_type_info_by_name("luminara_core::Velocity")
        .is_some());
    assert!(registry
        .get_type_info_by_name("luminara_core::Health")
        .is_some());

    // Lookup non-existent type
    assert!(registry
        .get_type_info_by_name("luminara_core::NonExistent")
        .is_none());

    // Verify all registered type names are accessible
    let type_names: Vec<_> = registry.type_names().collect();
    assert_eq!(type_names.len(), 3);
    assert!(type_names.contains(&&"luminara_core::Position".to_string()));
    assert!(type_names.contains(&&"luminara_core::Velocity".to_string()));
    assert!(type_names.contains(&&"luminara_core::Health".to_string()));
}

#[test]
fn test_registry_metadata_completeness() {
    // **Validates: Requirements 7.2**
    // Verify that all metadata is complete and accessible

    let mut registry = ReflectRegistry::new();
    registry.register::<Position>();

    let type_info = registry
        .get_type_info_by_name("luminara_core::Position")
        .expect("Position should be registered");

    // Verify TypeInfo completeness
    assert!(
        !type_info.type_name.is_empty(),
        "Type name should not be empty"
    );
    assert_eq!(type_info.kind, TypeKind::Struct, "Should be struct type");
    assert!(!type_info.fields.is_empty(), "Should have fields");

    // Verify FieldInfo completeness for each field
    for field in &type_info.fields {
        assert!(!field.name.is_empty(), "Field name should not be empty");
        assert!(
            !field.type_name.is_empty(),
            "Field type name should not be empty"
        );
        // TypeId is always valid (non-zero)

        // Verify field is accessible through reflection
        let instance = registry.construct("luminara_core::Position").unwrap();
        let field_value = instance.field(&field.name);
        assert!(
            field_value.is_some(),
            "Field '{}' should be accessible through reflection",
            field.name
        );
    }
}

#[test]
fn test_registry_empty_state() {
    // Test registry in empty state
    let registry = ReflectRegistry::new();

    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
    assert!(registry
        .get_type_info_by_name("luminara_core::Position")
        .is_none());
    assert!(registry.construct("luminara_core::Position").is_none());
}

#[test]
fn test_registry_multiple_registrations() {
    // Test that registering the same type multiple times doesn't cause issues
    let mut registry = ReflectRegistry::new();

    registry.register::<Position>();
    assert_eq!(registry.len(), 1);

    // Register again (should replace)
    registry.register::<Position>();
    assert_eq!(registry.len(), 1, "Should still have 1 type");

    // Verify it still works
    let position = registry.construct("luminara_core::Position");
    assert!(position.is_some());
}
