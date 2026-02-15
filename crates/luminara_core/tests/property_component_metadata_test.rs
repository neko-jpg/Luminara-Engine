use luminara_core::{Reflect, ReflectRegistry, TypeKind};
use proptest::prelude::*;

// ============================================================================
// Property Test 8: Component Metadata Completeness
// Feature: pre-editor-engine-audit
// Validates: Requirements 7.1, 7.2
// ============================================================================

/// **Validates: Requirements 7.1**
/// WHEN registering components, THE System SHALL capture type metadata
/// including field names, types, and descriptions

/// **Validates: Requirements 7.2**
/// WHEN querying component metadata, THE System SHALL provide schema
/// information for all registered components

// Test components with various field configurations
#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct SimpleComponent {
    value: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct MultiFieldComponent {
    position_x: f32,
    position_y: f32,
    position_z: f32,
    enabled: bool,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct TupleComponent(f32, f32, f32);

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct NestedComponent {
    inner: SimpleComponent,
    scale: f32,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
enum ComponentState {
    Active,
    Inactive,
    Paused,
}

impl Default for ComponentState {
    fn default() -> Self {
        ComponentState::Active
    }
}

/// Strategy for generating field counts
fn field_count_strategy() -> impl Strategy<Value = usize> {
    1usize..=10
}

/// Strategy for generating component type names
fn component_type_strategy() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec![
        "SimpleComponent",
        "MultiFieldComponent",
        "TupleComponent",
        "NestedComponent",
        "ComponentState",
    ])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 8: Component Metadata Completeness**
    ///
    /// For any registered component type, the reflection system should provide
    /// complete metadata including field names, types, and type information.
    ///
    /// **Validates: Requirements 7.1, 7.2**
    #[test]
    fn prop_component_metadata_completeness(
        component_type in component_type_strategy()
    ) {
        let mut registry = ReflectRegistry::new();

        // Register all test components
        registry.register::<SimpleComponent>();
        registry.register::<MultiFieldComponent>();
        registry.register::<TupleComponent>();
        registry.register::<NestedComponent>();
        registry.register::<ComponentState>();

        // Construct the component by type name
        let full_type_name = format!("luminara_core::{}", component_type);
        let instance = registry.construct(&full_type_name);

        prop_assert!(
            instance.is_some(),
            "Component type '{}' should be constructible from registry",
            component_type
        );

        let instance = instance.unwrap();
        let type_info = instance.type_info();

        // Verify type metadata is complete
        prop_assert!(
            !type_info.type_name.is_empty(),
            "Type name should not be empty"
        );

        prop_assert!(
            type_info.type_name.contains(component_type),
            "Type name should contain component type: expected '{}', got '{}'",
            component_type,
            type_info.type_name
        );

        // Verify type kind is appropriate
        match component_type {
            "SimpleComponent" | "MultiFieldComponent" | "NestedComponent" => {
                prop_assert_eq!(
                    type_info.kind,
                    TypeKind::Struct,
                    "Struct components should have Struct kind"
                );
            }
            "TupleComponent" => {
                prop_assert_eq!(
                    type_info.kind,
                    TypeKind::Tuple,
                    "Tuple components should have Tuple kind"
                );
            }
            "ComponentState" => {
                prop_assert_eq!(
                    type_info.kind,
                    TypeKind::Enum,
                    "Enum components should have Enum kind"
                );
            }
            _ => {}
        }

        // Verify field information is complete for struct and tuple types
        if matches!(type_info.kind, TypeKind::Struct | TypeKind::Tuple) {
            prop_assert!(
                !type_info.fields.is_empty(),
                "Struct and tuple components should have field information"
            );

            // Verify each field has complete metadata
            for field in &type_info.fields {
                prop_assert!(
                    !field.name.is_empty(),
                    "Field name should not be empty"
                );

                prop_assert!(
                    !field.type_name.is_empty(),
                    "Field type name should not be empty"
                );

                // Verify field is accessible
                let field_value = instance.field(&field.name);
                prop_assert!(
                    field_value.is_some(),
                    "Field '{}' should be accessible via reflection",
                    field.name
                );
            }
        }
    }

    /// **Property 8 (variant): All Registered Components Have Metadata**
    ///
    /// For any set of registered components, all should have complete metadata
    /// accessible through the registry.
    ///
    /// **Validates: Requirements 7.1, 7.2**
    #[test]
    fn prop_all_registered_components_have_metadata(
        component_count in 1usize..=5
    ) {
        let mut registry = ReflectRegistry::new();

        // Register components based on count
        let components: Vec<(&str, Box<dyn Fn(&mut ReflectRegistry)>)> = vec![
            ("SimpleComponent", Box::new(|r: &mut ReflectRegistry| r.register::<SimpleComponent>())),
            ("MultiFieldComponent", Box::new(|r: &mut ReflectRegistry| r.register::<MultiFieldComponent>())),
            ("TupleComponent", Box::new(|r: &mut ReflectRegistry| r.register::<TupleComponent>())),
            ("NestedComponent", Box::new(|r: &mut ReflectRegistry| r.register::<NestedComponent>())),
            ("ComponentState", Box::new(|r: &mut ReflectRegistry| r.register::<ComponentState>())),
        ];

        let selected_components: Vec<_> = components.into_iter().take(component_count).collect();

        // Register selected components
        for (_, register_fn) in &selected_components {
            register_fn(&mut registry);
        }

        prop_assert_eq!(
            registry.len(),
            component_count,
            "Registry should contain exactly {} components",
            component_count
        );

        // Verify all registered components have complete metadata
        for (component_name, _) in &selected_components {
            let full_type_name = format!("luminara_core::{}", component_name);
            let type_info = registry.get_type_info_by_name(&full_type_name);

            prop_assert!(
                type_info.is_some(),
                "Component '{}' should have type info in registry",
                component_name
            );

            let type_info = type_info.unwrap();

            prop_assert!(
                !type_info.type_name.is_empty(),
                "Type name should not be empty for '{}'",
                component_name
            );

            prop_assert!(
                type_info.type_name.contains(component_name),
                "Type name should contain component name"
            );
        }
    }

    /// **Property 8 (variant): Field Metadata Accuracy**
    ///
    /// For any component with fields, the field metadata should accurately
    /// reflect the actual field structure.
    ///
    /// **Validates: Requirements 7.1, 7.2**
    #[test]
    fn prop_field_metadata_accuracy(
        _seed in any::<u64>()
    ) {
        let component = MultiFieldComponent {
            position_x: 1.0,
            position_y: 2.0,
            position_z: 3.0,
            enabled: true,
            name: String::from("test"),
        };

        let type_info = component.type_info();

        // Verify expected field count
        prop_assert_eq!(
            type_info.fields.len(),
            5,
            "MultiFieldComponent should have 5 fields"
        );

        // Verify field names match actual struct fields
        let field_names: Vec<_> = type_info.fields.iter().map(|f| f.name.as_str()).collect();
        prop_assert!(
            field_names.contains(&"position_x"),
            "Should have position_x field"
        );
        prop_assert!(
            field_names.contains(&"position_y"),
            "Should have position_y field"
        );
        prop_assert!(
            field_names.contains(&"position_z"),
            "Should have position_z field"
        );
        prop_assert!(
            field_names.contains(&"enabled"),
            "Should have enabled field"
        );
        prop_assert!(
            field_names.contains(&"name"),
            "Should have name field"
        );

        // Verify field types are correct
        for field in &type_info.fields {
            match field.name.as_str() {
                "position_x" | "position_y" | "position_z" => {
                    prop_assert!(
                        field.type_name.contains("f32"),
                        "Position fields should be f32 type"
                    );
                }
                "enabled" => {
                    prop_assert!(
                        field.type_name.contains("bool"),
                        "Enabled field should be bool type"
                    );
                }
                "name" => {
                    prop_assert!(
                        field.type_name.contains("String"),
                        "Name field should be String type"
                    );
                }
                _ => {}
            }
        }

        // Verify all fields are accessible
        for field in &type_info.fields {
            let field_value = component.field(&field.name);
            prop_assert!(
                field_value.is_some(),
                "Field '{}' should be accessible",
                field.name
            );
        }
    }

    /// **Property 8 (variant): Tuple Component Field Indexing**
    ///
    /// For tuple components, field metadata should use numeric indices
    /// and all fields should be accessible by index.
    ///
    /// **Validates: Requirements 7.1, 7.2**
    #[test]
    fn prop_tuple_component_field_indexing(
        x in any::<f32>(),
        y in any::<f32>(),
        z in any::<f32>()
    ) {
        let component = TupleComponent(x, y, z);
        let type_info = component.type_info();

        prop_assert_eq!(
            type_info.kind,
            TypeKind::Tuple,
            "Should be tuple type"
        );

        prop_assert_eq!(
            type_info.fields.len(),
            3,
            "Tuple should have 3 fields"
        );

        // Verify field names are numeric indices
        let field_names: Vec<_> = type_info.fields.iter().map(|f| f.name.as_str()).collect();
        prop_assert_eq!(field_names, vec!["0", "1", "2"], "Field names should be indices");

        // Verify all fields are accessible by index
        for i in 0..3 {
            let field_value = component.field(&i.to_string());
            prop_assert!(
                field_value.is_some(),
                "Field at index {} should be accessible",
                i
            );
        }

        // Verify field types are correct
        for field in &type_info.fields {
            prop_assert!(
                field.type_name.contains("f32"),
                "All tuple fields should be f32 type"
            );
        }
    }

    /// **Property 8 (variant): Nested Component Metadata**
    ///
    /// For components with nested fields, metadata should be complete
    /// for both outer and inner fields.
    ///
    /// **Validates: Requirements 7.1, 7.2**
    #[test]
    fn prop_nested_component_metadata(
        inner_value in any::<f32>(),
        scale in any::<f32>()
    ) {
        let component = NestedComponent {
            inner: SimpleComponent { value: inner_value },
            scale,
        };

        let type_info = component.type_info();

        prop_assert_eq!(
            type_info.fields.len(),
            2,
            "NestedComponent should have 2 fields"
        );

        // Verify outer fields
        let field_names: Vec<_> = type_info.fields.iter().map(|f| f.name.as_str()).collect();
        prop_assert!(field_names.contains(&"inner"), "Should have inner field");
        prop_assert!(field_names.contains(&"scale"), "Should have scale field");

        // Access nested field
        let inner_field = component.field("inner");
        prop_assert!(inner_field.is_some(), "Inner field should be accessible");

        let inner_component = inner_field.unwrap();
        let inner_type_info = inner_component.type_info();

        // Verify nested component has its own metadata
        prop_assert!(
            inner_type_info.type_name.contains("SimpleComponent"),
            "Nested component should have correct type name"
        );

        prop_assert_eq!(
            inner_type_info.fields.len(),
            1,
            "SimpleComponent should have 1 field"
        );

        // Verify nested field is accessible
        let nested_value_field = inner_component.field("value");
        prop_assert!(
            nested_value_field.is_some(),
            "Nested value field should be accessible"
        );
    }
}

// Additional edge case tests

#[test]
fn test_simple_component_metadata() {
    // **Validates: Requirements 7.1, 7.2**
    let component = SimpleComponent { value: 42.0 };
    let type_info = component.type_info();

    assert!(type_info.type_name.contains("SimpleComponent"));
    assert_eq!(type_info.kind, TypeKind::Struct);
    assert_eq!(type_info.fields.len(), 1);

    let field = &type_info.fields[0];
    assert_eq!(field.name, "value");
    assert!(field.type_name.contains("f32"));

    // Verify field is accessible
    let field_value = component.field("value").unwrap();
    let value = field_value.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*value, 42.0);
}

#[test]
fn test_multi_field_component_metadata() {
    // **Validates: Requirements 7.1, 7.2**
    let component = MultiFieldComponent {
        position_x: 1.0,
        position_y: 2.0,
        position_z: 3.0,
        enabled: true,
        name: String::from("test"),
    };

    let type_info = component.type_info();

    assert!(type_info.type_name.contains("MultiFieldComponent"));
    assert_eq!(type_info.kind, TypeKind::Struct);
    assert_eq!(type_info.fields.len(), 5);

    // Verify all fields have metadata
    let field_names: Vec<_> = type_info.fields.iter().map(|f| f.name.as_str()).collect();
    assert!(field_names.contains(&"position_x"));
    assert!(field_names.contains(&"position_y"));
    assert!(field_names.contains(&"position_z"));
    assert!(field_names.contains(&"enabled"));
    assert!(field_names.contains(&"name"));

    // Verify all fields are accessible
    assert!(component.field("position_x").is_some());
    assert!(component.field("position_y").is_some());
    assert!(component.field("position_z").is_some());
    assert!(component.field("enabled").is_some());
    assert!(component.field("name").is_some());
}

#[test]
fn test_tuple_component_metadata() {
    // **Validates: Requirements 7.1, 7.2**
    let component = TupleComponent(1.0, 2.0, 3.0);
    let type_info = component.type_info();

    assert!(type_info.type_name.contains("TupleComponent"));
    assert_eq!(type_info.kind, TypeKind::Tuple);
    assert_eq!(type_info.fields.len(), 3);

    // Verify field names are indices
    let field_names: Vec<_> = type_info.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(field_names, vec!["0", "1", "2"]);

    // Verify fields are accessible by index
    assert!(component.field("0").is_some());
    assert!(component.field("1").is_some());
    assert!(component.field("2").is_some());
}

#[test]
fn test_enum_component_metadata() {
    // **Validates: Requirements 7.1, 7.2**
    let component = ComponentState::Active;
    let type_info = component.type_info();

    assert!(type_info.type_name.contains("ComponentState"));
    assert_eq!(type_info.kind, TypeKind::Enum);
    // Enums don't have field metadata in this implementation
}

#[test]
fn test_nested_component_metadata() {
    // **Validates: Requirements 7.1, 7.2**
    let component = NestedComponent {
        inner: SimpleComponent { value: 10.0 },
        scale: 2.0,
    };

    let type_info = component.type_info();

    assert!(type_info.type_name.contains("NestedComponent"));
    assert_eq!(type_info.kind, TypeKind::Struct);
    assert_eq!(type_info.fields.len(), 2);

    // Access nested component
    let inner_field = component.field("inner").unwrap();
    let inner_type_info = inner_field.type_info();

    assert!(inner_type_info.type_name.contains("SimpleComponent"));
    assert_eq!(inner_type_info.fields.len(), 1);

    // Access nested field
    let nested_value = inner_field.field("value").unwrap();
    let value = nested_value.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*value, 10.0);
}

#[test]
fn test_registry_provides_metadata_for_all_components() {
    // **Validates: Requirements 7.1, 7.2**
    let mut registry = ReflectRegistry::new();

    registry.register::<SimpleComponent>();
    registry.register::<MultiFieldComponent>();
    registry.register::<TupleComponent>();
    registry.register::<NestedComponent>();
    registry.register::<ComponentState>();

    assert_eq!(registry.len(), 5);

    // Verify all components have metadata
    let type_names = vec![
        "luminara_core::SimpleComponent",
        "luminara_core::MultiFieldComponent",
        "luminara_core::TupleComponent",
        "luminara_core::NestedComponent",
        "luminara_core::ComponentState",
    ];

    for type_name in type_names {
        let type_info = registry.get_type_info_by_name(type_name);
        assert!(
            type_info.is_some(),
            "Type info should exist for {}",
            type_name
        );

        let type_info = type_info.unwrap();
        assert!(!type_info.type_name.is_empty());
        assert!(type_info.type_name.contains(type_name.split("::").last().unwrap()));
    }
}

#[test]
fn test_field_type_ids_are_unique() {
    // **Validates: Requirements 7.1, 7.2**
    let component = MultiFieldComponent {
        position_x: 1.0,
        position_y: 2.0,
        position_z: 3.0,
        enabled: true,
        name: String::from("test"),
    };

    let type_info = component.type_info();

    // Collect type IDs
    let f32_fields: Vec<_> = type_info
        .fields
        .iter()
        .filter(|f| f.type_name.contains("f32"))
        .collect();

    let bool_fields: Vec<_> = type_info
        .fields
        .iter()
        .filter(|f| f.type_name.contains("bool"))
        .collect();

    let string_fields: Vec<_> = type_info
        .fields
        .iter()
        .filter(|f| f.type_name.contains("String"))
        .collect();

    // Verify f32 fields have same type ID
    assert_eq!(f32_fields.len(), 3);
    assert_eq!(f32_fields[0].type_id, f32_fields[1].type_id);
    assert_eq!(f32_fields[1].type_id, f32_fields[2].type_id);

    // Verify different types have different type IDs
    assert_ne!(f32_fields[0].type_id, bool_fields[0].type_id);
    assert_ne!(f32_fields[0].type_id, string_fields[0].type_id);
    assert_ne!(bool_fields[0].type_id, string_fields[0].type_id);
}

#[test]
fn test_component_metadata_is_consistent_across_instances() {
    // **Validates: Requirements 7.1, 7.2**
    let component1 = SimpleComponent { value: 1.0 };
    let component2 = SimpleComponent { value: 2.0 };

    let type_info1 = component1.type_info();
    let type_info2 = component2.type_info();

    // Type info should be the same (same pointer due to OnceLock)
    assert_eq!(type_info1.type_name, type_info2.type_name);
    assert_eq!(type_info1.type_id, type_info2.type_id);
    assert_eq!(type_info1.kind, type_info2.kind);
    assert_eq!(type_info1.fields.len(), type_info2.fields.len());
}
