use luminara_core::{Reflect, ReflectError, ReflectRegistry, TypeKind};

// Test struct with named fields
#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Transform {
    position: f32,
    rotation: f32,
    scale: f32,
}

// Test struct with tuple fields
#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Color(f32, f32, f32, f32);

// Test enum
#[derive(Debug, Clone, PartialEq, Reflect)]
enum Shape {
    Circle,
    Rectangle,
    Triangle,
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Circle
    }
}

// Test unit struct
#[derive(Debug, Clone, PartialEq, Default, Reflect)]
struct Marker;

#[test]
fn test_struct_type_info() {
    let transform = Transform {
        position: 1.0,
        rotation: 2.0,
        scale: 3.0,
    };

    let type_info = transform.type_info();
    assert!(type_info.type_name.contains("Transform"));
    assert_eq!(type_info.kind, TypeKind::Struct);
    assert_eq!(type_info.fields.len(), 3);

    // Check field names
    let field_names: Vec<_> = type_info.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(field_names, vec!["position", "rotation", "scale"]);
}

#[test]
fn test_tuple_struct_type_info() {
    let color = Color(1.0, 0.5, 0.0, 1.0);

    let type_info = color.type_info();
    assert!(type_info.type_name.contains("Color"));
    assert_eq!(type_info.kind, TypeKind::Tuple);
    assert_eq!(type_info.fields.len(), 4);

    // Check field names (should be indices)
    let field_names: Vec<_> = type_info.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(field_names, vec!["0", "1", "2", "3"]);
}

#[test]
fn test_enum_type_info() {
    let shape = Shape::Circle;

    let type_info = shape.type_info();
    assert!(type_info.type_name.contains("Shape"));
    assert_eq!(type_info.kind, TypeKind::Enum);
}

#[test]
fn test_unit_struct_type_info() {
    let marker = Marker;

    let type_info = marker.type_info();
    assert!(type_info.type_name.contains("Marker"));
    assert_eq!(type_info.kind, TypeKind::Value);
    assert_eq!(type_info.fields.len(), 0);
}

#[test]
fn test_field_access() {
    let transform = Transform {
        position: 1.0,
        rotation: 2.0,
        scale: 3.0,
    };

    // Access field by name
    let position_field = transform.field("position").unwrap();
    let position_value = position_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*position_value, 1.0);

    // Access non-existent field
    assert!(transform.field("invalid").is_none());
}

#[test]
fn test_field_mut_access() {
    let mut transform = Transform {
        position: 1.0,
        rotation: 2.0,
        scale: 3.0,
    };

    // Mutably access field
    let position_field = transform.field_mut("position").unwrap();
    let position_value = position_field.as_any_mut().downcast_mut::<f32>().unwrap();
    *position_value = 10.0;

    assert_eq!(transform.position, 10.0);
}

#[test]
fn test_set_field() {
    let mut transform = Transform {
        position: 1.0,
        rotation: 2.0,
        scale: 3.0,
    };

    // Set field with correct type
    let new_value = Box::new(5.0f32);
    transform.set_field("position", new_value).unwrap();
    assert_eq!(transform.position, 5.0);

    // Try to set field with wrong type
    let wrong_value = Box::new(42i32);
    let result = transform.set_field("position", wrong_value);
    assert!(matches!(result, Err(ReflectError::TypeMismatch { .. })));

    // Try to set non-existent field
    let value = Box::new(1.0f32);
    let result = transform.set_field("invalid", value);
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));
}

#[test]
fn test_tuple_field_access() {
    let color = Color(1.0, 0.5, 0.0, 1.0);

    // Access tuple field by index
    let r_field = color.field("0").unwrap();
    let r_value = r_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*r_value, 1.0);

    let g_field = color.field("1").unwrap();
    let g_value = g_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*g_value, 0.5);
}

#[test]
fn test_tuple_set_field() {
    let mut color = Color(1.0, 0.5, 0.0, 1.0);

    // Set tuple field by index
    let new_value = Box::new(0.8f32);
    color.set_field("0", new_value).unwrap();
    assert_eq!(color.0, 0.8);
}

#[test]
fn test_clone_value() {
    let transform = Transform {
        position: 1.0,
        rotation: 2.0,
        scale: 3.0,
    };

    let cloned = transform.clone_value();
    let cloned_transform = cloned.as_any().downcast_ref::<Transform>().unwrap();
    assert_eq!(*cloned_transform, transform);
}

#[test]
fn test_serialize_json() {
    let transform = Transform {
        position: 1.0,
        rotation: 2.0,
        scale: 3.0,
    };

    let json = transform.serialize_json();
    assert!(json.is_object());

    let obj = json.as_object().unwrap();
    assert_eq!(obj.get("position").unwrap().as_f64().unwrap(), 1.0);
    assert_eq!(obj.get("rotation").unwrap().as_f64().unwrap(), 2.0);
    assert_eq!(obj.get("scale").unwrap().as_f64().unwrap(), 3.0);
}

#[test]
fn test_deserialize_json() {
    let mut transform = Transform {
        position: 0.0,
        rotation: 0.0,
        scale: 0.0,
    };

    let json = serde_json::json!({
        "position": 10.0,
        "rotation": 20.0,
        "scale": 30.0
    });

    transform.deserialize_json(&json).unwrap();
    assert_eq!(transform.position, 10.0);
    assert_eq!(transform.rotation, 20.0);
    assert_eq!(transform.scale, 30.0);
}

#[test]
fn test_tuple_serialize_json() {
    let color = Color(1.0, 0.5, 0.0, 1.0);

    let json = color.serialize_json();
    assert!(json.is_array());

    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 4);
    assert_eq!(arr[0].as_f64().unwrap(), 1.0);
    assert_eq!(arr[1].as_f64().unwrap(), 0.5);
}

#[test]
fn test_tuple_deserialize_json() {
    let mut color = Color(0.0, 0.0, 0.0, 0.0);

    let json = serde_json::json!([0.8, 0.6, 0.4, 0.2]);

    color.deserialize_json(&json).unwrap();
    assert_eq!(color.0, 0.8);
    assert_eq!(color.1, 0.6);
    assert_eq!(color.2, 0.4);
    assert_eq!(color.3, 0.2);
}

#[test]
fn test_enum_serialize_json() {
    let shape = Shape::Rectangle;

    let json = shape.serialize_json();
    assert_eq!(json.as_str().unwrap(), "Rectangle");
}

#[test]
fn test_registry_register() {
    let mut registry = ReflectRegistry::new();

    registry.register::<Transform>();
    registry.register::<Color>();
    registry.register::<Shape>();

    assert_eq!(registry.len(), 3);
    assert!(!registry.is_empty());
}

#[test]
fn test_registry_get_type_info() {
    let mut registry = ReflectRegistry::new();
    registry.register::<Transform>();

    let type_info = registry.get_type_info_by_name("luminara_core::Transform");
    assert!(type_info.is_some());

    let type_info = registry.get_type_info_by_name("NonExistent");
    assert!(type_info.is_none());
}

#[test]
fn test_registry_construct() {
    let mut registry = ReflectRegistry::new();
    registry.register::<Transform>();

    let instance = registry.construct("luminara_core::Transform").unwrap();
    let transform = instance.as_any().downcast_ref::<Transform>().unwrap();
    assert_eq!(*transform, Transform::default());
}

#[test]
fn test_registry_type_names() {
    let mut registry = ReflectRegistry::new();
    registry.register::<Transform>();
    registry.register::<Color>();

    let names: Vec<_> = registry.type_names().collect();
    assert_eq!(names.len(), 2);
}

#[test]
fn test_primitive_reflection() {
    let mut value = 42i32;

    let type_info = value.type_info();
    assert_eq!(type_info.type_name, "i32");
    assert_eq!(type_info.kind, TypeKind::Value);

    // Test serialization
    let json = value.serialize_json();
    assert_eq!(json, serde_json::json!(42));

    // Test deserialization
    value.deserialize_json(&serde_json::json!(100)).unwrap();
    assert_eq!(value, 100);
}

#[test]
fn test_string_reflection() {
    let mut value = String::from("hello");

    let type_info = value.type_info();
    assert_eq!(type_info.type_name, "String");

    // Test serialization
    let json = value.serialize_json();
    assert_eq!(json, serde_json::json!("hello"));

    // Test deserialization
    value.deserialize_json(&serde_json::json!("world")).unwrap();
    assert_eq!(value, "world");
}

#[test]
fn test_nested_struct_field_access() {
    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Entity {
        position: Position,
        health: f32,
    }

    let entity = Entity {
        position: Position { x: 10.0, y: 20.0 },
        health: 100.0,
    };

    // Access nested field
    let position_field = entity.field("position").unwrap();
    let position = position_field.as_any().downcast_ref::<Position>().unwrap();
    assert_eq!(position.x, 10.0);
    assert_eq!(position.y, 20.0);

    // Access field on nested struct
    let x_field = position.field("x").unwrap();
    let x_value = x_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*x_value, 10.0);
}

#[test]
fn test_nested_field_path_access() {
    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Entity {
        position: Position,
        health: f32,
    }

    let entity = Entity {
        position: Position { x: 10.0, y: 20.0 },
        health: 100.0,
    };

    // Access nested field using path
    let x_field = entity.field_path("position.x").unwrap();
    let x_value = x_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*x_value, 10.0);

    let y_field = entity.field_path("position.y").unwrap();
    let y_value = y_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*y_value, 20.0);

    // Access top-level field using path
    let health_field = entity.field_path("health").unwrap();
    let health_value = health_field.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*health_value, 100.0);

    // Access non-existent path
    assert!(entity.field_path("position.z").is_none());
    assert!(entity.field_path("invalid.field").is_none());
}

#[test]
fn test_nested_field_path_mut_access() {
    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Entity {
        position: Position,
        health: f32,
    }

    let mut entity = Entity {
        position: Position { x: 10.0, y: 20.0 },
        health: 100.0,
    };

    // Mutably access nested field using path
    let x_field = entity.field_path_mut("position.x").unwrap();
    let x_value = x_field.as_any_mut().downcast_mut::<f32>().unwrap();
    *x_value = 50.0;

    assert_eq!(entity.position.x, 50.0);

    // Mutably access top-level field using path
    let health_field = entity.field_path_mut("health").unwrap();
    let health_value = health_field.as_any_mut().downcast_mut::<f32>().unwrap();
    *health_value = 75.0;

    assert_eq!(entity.health, 75.0);
}

#[test]
fn test_set_field_path() {
    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Entity {
        position: Position,
        health: f32,
    }

    let mut entity = Entity {
        position: Position { x: 10.0, y: 20.0 },
        health: 100.0,
    };

    // Set nested field using path
    let new_x = Box::new(99.0f32);
    entity.set_field_path("position.x", new_x).unwrap();
    assert_eq!(entity.position.x, 99.0);

    let new_y = Box::new(88.0f32);
    entity.set_field_path("position.y", new_y).unwrap();
    assert_eq!(entity.position.y, 88.0);

    // Set top-level field using path
    let new_health = Box::new(50.0f32);
    entity.set_field_path("health", new_health).unwrap();
    assert_eq!(entity.health, 50.0);

    // Try to set with wrong type
    let wrong_value = Box::new(42i32);
    let result = entity.set_field_path("position.x", wrong_value);
    assert!(matches!(result, Err(ReflectError::TypeMismatch { .. })));

    // Try to set non-existent path
    let value = Box::new(1.0f32);
    let result = entity.set_field_path("position.z", value);
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));
}

#[test]
fn test_deeply_nested_field_path() {
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
    struct GameObject {
        transform: Transform,
        name: String,
    }

    let mut game_object = GameObject {
        transform: Transform {
            position: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            rotation: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        },
        name: String::from("Player"),
    };

    // Access deeply nested field
    let pos_x = game_object.field_path("transform.position.x").unwrap();
    let pos_x_value = pos_x.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*pos_x_value, 1.0);

    // Modify deeply nested field
    let new_pos_x = Box::new(100.0f32);
    game_object
        .set_field_path("transform.position.x", new_pos_x)
        .unwrap();
    assert_eq!(game_object.transform.position.x, 100.0);

    // Modify another deeply nested field
    let new_scale_y = Box::new(2.5f32);
    game_object
        .set_field_path("transform.scale.y", new_scale_y)
        .unwrap();
    assert_eq!(game_object.transform.scale.y, 2.5);
}

#[test]
fn test_field_path_with_string() {
    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Player {
        name: String,
        score: i32,
    }

    let mut player = Player {
        name: String::from("Alice"),
        score: 100,
    };

    // Access string field via path
    let name_field = player.field_path("name").unwrap();
    let name_value = name_field.as_any().downcast_ref::<String>().unwrap();
    assert_eq!(name_value, "Alice");

    // Modify string field via path
    let new_name = Box::new(String::from("Bob"));
    player.set_field_path("name", new_name).unwrap();
    assert_eq!(player.name, "Bob");
}

#[test]
fn test_field_path_error_handling() {
    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, PartialEq, Default, Reflect)]
    struct Entity {
        position: Position,
        health: f32,
    }

    let mut entity = Entity {
        position: Position { x: 10.0, y: 20.0 },
        health: 100.0,
    };

    // Test invalid intermediate path
    let result = entity.set_field_path("invalid.x", Box::new(1.0f32));
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));

    // Test invalid final path segment
    let result = entity.set_field_path("position.invalid", Box::new(1.0f32));
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));

    // Test empty path
    let result = entity.set_field_path("", Box::new(1.0f32));
    assert!(matches!(result, Err(ReflectError::FieldNotFound(_))));
}
