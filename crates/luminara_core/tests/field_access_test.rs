/// Unit tests for field access and modification functionality
/// Task 10.4: Implement field access and modification
/// Requirements: 7.3
use luminara_core::{Reflect, ReflectError};

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Transform {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Entity {
    name: String,
    transform: Transform,
    health: f32,
}

// Test 1: Get field by name
#[test]
fn test_get_field_by_name() {
    let entity = Entity {
        name: String::from("Player"),
        transform: Transform::default(),
        health: 100.0,
    };

    // Get top-level field
    let name_field = entity.field("name").unwrap();
    let name_value = name_field.as_any().downcast_ref::<String>().unwrap();
    assert_eq!(name_value, "Player");

    let health_field = entity.field("health").unwrap();
    let health_value = health_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*health_value, 100.0);

    // Get nested struct field
    let transform_field = entity.field("transform").unwrap();
    let transform_value = transform_field
        .as_any()
        .downcast_ref::<Transform>()
        .unwrap();
    assert_eq!(*transform_value, Transform::default());
}

// Test 2: Get non-existent field returns None
#[test]
fn test_get_nonexistent_field() {
    let entity = Entity::default();

    assert!(entity.field("invalid_field").is_none());
    assert!(entity.field("").is_none());
}

// Test 3: Set field with type validation
#[test]
fn test_set_field_with_type_validation() {
    let mut entity = Entity {
        name: String::from("Player"),
        transform: Transform::default(),
        health: 100.0,
    };

    // Set field with correct type
    let new_health = Box::new(75.0f32);
    entity.set_field("health", new_health).unwrap();
    assert_eq!(entity.health, 75.0);

    // Set string field
    let new_name = Box::new(String::from("Enemy"));
    entity.set_field("name", new_name).unwrap();
    assert_eq!(entity.name, "Enemy");
}

// Test 4: Set field with wrong type fails
#[test]
fn test_set_field_type_mismatch() {
    let mut entity = Entity::default();

    // Try to set f32 field with i32
    let wrong_value = Box::new(42i32);
    let result = entity.set_field("health", wrong_value);
    assert!(matches!(result, Err(ReflectError::TypeMismatch { .. })));

    // Try to set String field with f32
    let wrong_value = Box::new(3.14f32);
    let result = entity.set_field("name", wrong_value);
    assert!(matches!(result, Err(ReflectError::TypeMismatch { .. })));
}

// Test 5: Set non-existent field fails
#[test]
fn test_set_nonexistent_field() {
    let mut entity = Entity::default();

    let value = Box::new(100.0f32);
    let result = entity.set_field("invalid_field", value);
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));
}

// Test 6: Support nested field access (single level)
#[test]
fn test_nested_field_access_single_level() {
    let entity = Entity {
        name: String::from("Player"),
        transform: Transform {
            position: Vec3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            rotation: Vec3::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        },
        health: 100.0,
    };

    // Access nested field using path
    let position_field = entity.field_path("transform.position").unwrap();
    let position_value = position_field.as_any().downcast_ref::<Vec3>().unwrap();
    assert_eq!(position_value.x, 10.0);
    assert_eq!(position_value.y, 20.0);
    assert_eq!(position_value.z, 30.0);
}

// Test 7: Support nested field access (multiple levels)
#[test]
fn test_nested_field_access_multiple_levels() {
    let entity = Entity {
        name: String::from("Player"),
        transform: Transform {
            position: Vec3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            rotation: Vec3 {
                x: 0.0,
                y: 90.0,
                z: 0.0,
            },
            scale: Vec3 {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
        },
        health: 100.0,
    };

    // Access deeply nested field
    let x_field = entity.field_path("transform.position.x").unwrap();
    let x_value = x_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*x_value, 10.0);

    let y_field = entity.field_path("transform.rotation.y").unwrap();
    let y_value = y_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*y_value, 90.0);

    let scale_z_field = entity.field_path("transform.scale.z").unwrap();
    let scale_z_value = scale_z_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*scale_z_value, 2.0);
}

// Test 8: Modify nested field
#[test]
fn test_modify_nested_field() {
    let mut entity = Entity {
        name: String::from("Player"),
        transform: Transform {
            position: Vec3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            rotation: Vec3::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        },
        health: 100.0,
    };

    // Modify nested field using path
    let new_x = Box::new(99.0f32);
    entity
        .set_field_path("transform.position.x", new_x)
        .unwrap();
    assert_eq!(entity.transform.position.x, 99.0);

    // Modify another nested field
    let new_scale_y = Box::new(3.0f32);
    entity
        .set_field_path("transform.scale.y", new_scale_y)
        .unwrap();
    assert_eq!(entity.transform.scale.y, 3.0);
}

// Test 9: Nested field access with invalid path
#[test]
fn test_nested_field_access_invalid_path() {
    let entity = Entity::default();

    // Invalid intermediate path
    assert!(entity.field_path("invalid.position.x").is_none());

    // Invalid final segment
    assert!(entity.field_path("transform.invalid").is_none());

    // Invalid deeply nested path
    assert!(entity.field_path("transform.position.invalid").is_none());
}

// Test 10: Mutable nested field access
#[test]
fn test_mutable_nested_field_access() {
    let mut entity = Entity {
        name: String::from("Player"),
        transform: Transform {
            position: Vec3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            rotation: Vec3::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        },
        health: 100.0,
    };

    // Get mutable reference to nested field
    let x_field = entity.field_path_mut("transform.position.x").unwrap();
    let x_value = x_field.as_any_mut().downcast_mut::<f32>().unwrap();
    *x_value = 50.0;

    assert_eq!(entity.transform.position.x, 50.0);

    // Modify another nested field
    let health_field = entity.field_path_mut("health").unwrap();
    let health_value = health_field.as_any_mut().downcast_mut::<f32>().unwrap();
    *health_value = 25.0;

    assert_eq!(entity.health, 25.0);
}

// Test 11: Complex nested structure
#[test]
fn test_complex_nested_structure() {
    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Inventory {
        gold: i32,
        items: i32,
    }

    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Player {
        entity: Entity,
        inventory: Inventory,
        level: i32,
    }

    let mut player = Player {
        entity: Entity {
            name: String::from("Hero"),
            transform: Transform {
                position: Vec3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                rotation: Vec3::default(),
                scale: Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
            },
            health: 100.0,
        },
        inventory: Inventory {
            gold: 500,
            items: 10,
        },
        level: 5,
    };

    // Access deeply nested field
    let pos_x = player.field_path("entity.transform.position.x").unwrap();
    let pos_x_value = pos_x.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*pos_x_value, 1.0);

    // Modify deeply nested field
    let new_pos_x = Box::new(100.0f32);
    player
        .set_field_path("entity.transform.position.x", new_pos_x)
        .unwrap();
    assert_eq!(player.entity.transform.position.x, 100.0);

    // Access and modify inventory
    let gold = player.field_path("inventory.gold").unwrap();
    let gold_value = gold.as_any().downcast_ref::<i32>().unwrap();
    assert_eq!(*gold_value, 500);

    let new_gold = Box::new(1000i32);
    player.set_field_path("inventory.gold", new_gold).unwrap();
    assert_eq!(player.inventory.gold, 1000);
}

// Test 12: Field access on primitive types
#[test]
fn test_field_access_on_primitives() {
    let mut value = 42i32;

    // Primitives don't have fields
    assert!(value.field("anything").is_none());
    assert!(value.field_mut("anything").is_none());

    // Setting field on primitive should fail
    let new_value = Box::new(100i32);
    let result = value.set_field("anything", new_value);
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));
}

// Test 13: Empty path handling
#[test]
fn test_empty_path_handling() {
    let mut entity = Entity::default();

    // Empty path should fail
    let result = entity.set_field_path("", Box::new(1.0f32));
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));
}

// Test 14: Single segment path (should work like set_field)
#[test]
fn test_single_segment_path() {
    let mut entity = Entity {
        name: String::from("Player"),
        transform: Transform::default(),
        health: 100.0,
    };

    // Single segment path should work
    let new_health = Box::new(50.0f32);
    entity.set_field_path("health", new_health).unwrap();
    assert_eq!(entity.health, 50.0);

    // Access with single segment
    let health_field = entity.field_path("health").unwrap();
    let health_value = health_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*health_value, 50.0);
}

// Test 15: Type validation across nested paths
#[test]
fn test_type_validation_nested_paths() {
    let mut entity = Entity {
        name: String::from("Player"),
        transform: Transform {
            position: Vec3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            rotation: Vec3::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        },
        health: 100.0,
    };

    // Try to set nested field with wrong type
    let wrong_value = Box::new(42i32);
    let result = entity.set_field_path("transform.position.x", wrong_value);
    assert!(matches!(result, Err(ReflectError::TypeMismatch { .. })));

    // Verify original value unchanged
    assert_eq!(entity.transform.position.x, 10.0);
}
