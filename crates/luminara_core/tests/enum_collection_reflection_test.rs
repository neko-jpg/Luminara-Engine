use luminara_core::reflect::{EnumVariant, FieldInfo, Reflect, ReflectError, TypeInfo, TypeKind};
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Test enum with various variant types
#[derive(Debug, Clone, PartialEq)]
enum GameState {
    MainMenu,
    Playing { level: i32, score: u32 },
    Paused(i32),
    GameOver,
}

impl Reflect for GameState {
    fn type_info(&self) -> &TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| TypeInfo {
            type_name: "GameState".to_string(),
            type_id: TypeId::of::<GameState>(),
            kind: TypeKind::Enum,
            fields: Vec::new(),
        })
    }

    fn field(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }

    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }

    fn set_field(&mut self, name: &str, _value: Box<dyn Reflect>) -> Result<(), ReflectError> {
        Err(ReflectError::FieldNotFound(name.to_string()))
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn serialize_json(&self) -> serde_json::Value {
        match self {
            GameState::MainMenu => serde_json::json!({"variant": "MainMenu"}),
            GameState::Playing { level, score } => serde_json::json!({
                "variant": "Playing",
                "fields": {"level": level, "score": score}
            }),
            GameState::Paused(level) => serde_json::json!({
                "variant": "Paused",
                "fields": [level]
            }),
            GameState::GameOver => serde_json::json!({"variant": "GameOver"}),
        }
    }

    fn deserialize_json(&mut self, _value: &serde_json::Value) -> Result<(), ReflectError> {
        Err(ReflectError::DeserializationError(
            "GameState deserialization not implemented".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn enum_variant(&self) -> Option<EnumVariant> {
        match self {
            GameState::MainMenu => Some(EnumVariant {
                variant_name: "MainMenu".to_string(),
                discriminant: Some(0),
                fields: Vec::new(),
            }),
            GameState::Playing { .. } => Some(EnumVariant {
                variant_name: "Playing".to_string(),
                discriminant: Some(1),
                fields: vec![
                    FieldInfo {
                        name: "level".to_string(),
                        type_name: "i32".to_string(),
                        type_id: TypeId::of::<i32>(),
                        description: Some("Current game level".to_string()),
                        default_value: None,
                    },
                    FieldInfo {
                        name: "score".to_string(),
                        type_name: "u32".to_string(),
                        type_id: TypeId::of::<u32>(),
                        description: Some("Player score".to_string()),
                        default_value: None,
                    },
                ],
            }),
            GameState::Paused(_) => Some(EnumVariant {
                variant_name: "Paused".to_string(),
                discriminant: Some(2),
                fields: vec![FieldInfo {
                    name: "0".to_string(),
                    type_name: "i32".to_string(),
                    type_id: TypeId::of::<i32>(),
                    description: Some("Paused level".to_string()),
                    default_value: None,
                }],
            }),
            GameState::GameOver => Some(EnumVariant {
                variant_name: "GameOver".to_string(),
                discriminant: Some(3),
                fields: Vec::new(),
            }),
        }
    }
}

#[test]
fn test_enum_variant_inspection() {
    // Test unit variant
    let main_menu = GameState::MainMenu;
    let variant = main_menu.enum_variant().unwrap();
    assert_eq!(variant.variant_name, "MainMenu");
    assert_eq!(variant.discriminant, Some(0));
    assert!(variant.fields.is_empty());

    // Test struct variant with fields
    let playing = GameState::Playing {
        level: 5,
        score: 1000,
    };
    let variant = playing.enum_variant().unwrap();
    assert_eq!(variant.variant_name, "Playing");
    assert_eq!(variant.discriminant, Some(1));
    assert_eq!(variant.fields.len(), 2);
    assert_eq!(variant.fields[0].name, "level");
    assert_eq!(variant.fields[0].type_name, "i32");
    assert_eq!(
        variant.fields[0].description,
        Some("Current game level".to_string())
    );
    assert_eq!(variant.fields[1].name, "score");
    assert_eq!(variant.fields[1].type_name, "u32");

    // Test tuple variant
    let paused = GameState::Paused(3);
    let variant = paused.enum_variant().unwrap();
    assert_eq!(variant.variant_name, "Paused");
    assert_eq!(variant.discriminant, Some(2));
    assert_eq!(variant.fields.len(), 1);
    assert_eq!(variant.fields[0].name, "0");
}

#[test]
fn test_enum_discriminant_values() {
    let states = vec![
        GameState::MainMenu,
        GameState::Playing {
            level: 1,
            score: 0,
        },
        GameState::Paused(1),
        GameState::GameOver,
    ];

    let discriminants: Vec<isize> = states
        .iter()
        .map(|s| s.enum_variant().unwrap().discriminant.unwrap())
        .collect();

    assert_eq!(discriminants, vec![0, 1, 2, 3]);
}

#[test]
fn test_vec_iteration() {
    let numbers = vec![10i32, 20, 30, 40, 50];

    // Test iteration
    let mut collected = Vec::new();
    for elem in numbers.collection_iter() {
        if let Some(value) = elem.as_any().downcast_ref::<i32>() {
            collected.push(*value);
        }
    }

    assert_eq!(collected, vec![10, 20, 30, 40, 50]);
}

#[test]
fn test_vec_element_access() {
    let mut vec = vec![1.0f32, 2.0, 3.0, 4.0];

    // Test immutable access
    let elem = vec.collection_get(2).unwrap();
    let value = elem.as_any().downcast_ref::<f32>().unwrap();
    assert_eq!(*value, 3.0);

    // Test mutable access
    let elem_mut = vec.collection_get_mut(2).unwrap();
    let value_mut = elem_mut.as_any_mut().downcast_mut::<f32>().unwrap();
    *value_mut = 99.0;

    assert_eq!(vec[2], 99.0);
}

#[test]
fn test_hashmap_iteration() {
    let mut map = HashMap::new();
    map.insert("health".to_string(), 100i32);
    map.insert("mana".to_string(), 50);
    map.insert("stamina".to_string(), 75);

    // Collect all keys
    let mut keys = map.map_keys();
    keys.sort();
    assert_eq!(keys, vec!["health", "mana", "stamina"]);

    // Verify we can access each value
    for key in &keys {
        let value = map.map_get(key).unwrap();
        assert!(value.as_any().downcast_ref::<i32>().is_some());
    }
}

#[test]
fn test_hashmap_element_access() {
    let mut stats = HashMap::new();
    stats.insert("strength".to_string(), 10i32);
    stats.insert("dexterity".to_string(), 15);
    stats.insert("intelligence".to_string(), 12);

    // Test immutable access
    let strength = stats.map_get("strength").unwrap();
    let value = strength.as_any().downcast_ref::<i32>().unwrap();
    assert_eq!(*value, 10);

    // Test mutable access
    let dex_mut = stats.map_get_mut("dexterity").unwrap();
    let value_mut = dex_mut.as_any_mut().downcast_mut::<i32>().unwrap();
    *value_mut = 20;

    assert_eq!(stats["dexterity"], 20);

    // Test missing key
    assert!(stats.map_get("wisdom").is_none());
}

#[test]
fn test_collection_length() {
    let vec = vec![1, 2, 3, 4, 5];
    assert_eq!(vec.collection_len(), Some(5));

    let mut map = HashMap::new();
    map.insert("a".to_string(), 1);
    map.insert("b".to_string(), 2);
    assert_eq!(map.collection_len(), Some(2));

    // Non-collection types should return None
    let number = 42i32;
    assert_eq!(number.collection_len(), None);
}

#[test]
fn test_nested_collections() {
    // Vec of HashMaps
    let mut vec_of_maps: Vec<HashMap<String, i32>> = Vec::new();

    let mut map1 = HashMap::new();
    map1.insert("x".to_string(), 1);
    map1.insert("y".to_string(), 2);

    let mut map2 = HashMap::new();
    map2.insert("a".to_string(), 10);
    map2.insert("b".to_string(), 20);

    vec_of_maps.push(map1);
    vec_of_maps.push(map2);

    // Test outer collection
    assert_eq!(vec_of_maps.collection_len(), Some(2));

    // Test inner collection
    let first_map = vec_of_maps.collection_get(0).unwrap();
    assert_eq!(first_map.collection_len(), Some(2));

    // Test nested value access
    let value = first_map.map_get("x").unwrap();
    let int_value = value.as_any().downcast_ref::<i32>().unwrap();
    assert_eq!(*int_value, 1);
}

#[test]
fn test_collection_type_info() {
    let vec = vec![1i32, 2, 3];
    let type_info = vec.type_info();
    assert_eq!(type_info.kind, TypeKind::List);

    let map: HashMap<String, i32> = HashMap::new();
    let type_info = map.type_info();
    assert_eq!(type_info.kind, TypeKind::Map);
}

#[test]
fn test_enum_serialization() {
    let main_menu = GameState::MainMenu;
    let json = main_menu.serialize_json();
    assert_eq!(json["variant"], "MainMenu");

    let playing = GameState::Playing {
        level: 3,
        score: 500,
    };
    let json = playing.serialize_json();
    assert_eq!(json["variant"], "Playing");
    assert_eq!(json["fields"]["level"], 3);
    assert_eq!(json["fields"]["score"], 500);

    let paused = GameState::Paused(2);
    let json = paused.serialize_json();
    assert_eq!(json["variant"], "Paused");
    assert_eq!(json["fields"][0], 2);
}

/// **Validates: Requirements 7.6**
/// Property: Enum reflection provides complete variant information
#[test]
fn property_enum_variant_completeness() {
    let states = vec![
        GameState::MainMenu,
        GameState::Playing {
            level: 1,
            score: 100,
        },
        GameState::Paused(5),
        GameState::GameOver,
    ];

    for state in states {
        let variant = state.enum_variant();
        assert!(variant.is_some(), "All enum values should provide variant info");

        let variant = variant.unwrap();
        assert!(!variant.variant_name.is_empty(), "Variant name should not be empty");
        assert!(variant.discriminant.is_some(), "Discriminant should be available");

        // Verify type info is consistent
        let type_info = state.type_info();
        assert_eq!(type_info.kind, TypeKind::Enum);
    }
}

/// **Validates: Requirements 7.7**
/// Property: Collection reflection supports iteration and element access
#[test]
fn property_collection_operations() {
    // Test Vec
    let vec = vec![1i32, 2, 3, 4, 5];

    // Length should be available
    let len = vec.collection_len().expect("Vec should report length");
    assert_eq!(len, 5);

    // All elements should be accessible
    for i in 0..len {
        let elem = vec.collection_get(i);
        assert!(elem.is_some(), "Element at index {} should be accessible", i);
    }

    // Out of bounds should return None
    assert!(vec.collection_get(len).is_none());

    // Iteration should visit all elements
    let mut count = 0;
    for _elem in vec.collection_iter() {
        count += 1;
    }
    assert_eq!(count, len);

    // Test HashMap
    let mut map = HashMap::new();
    map.insert("a".to_string(), 1);
    map.insert("b".to_string(), 2);
    map.insert("c".to_string(), 3);

    let len = map.collection_len().expect("HashMap should report length");
    assert_eq!(len, 3);

    // All keys should be accessible
    let keys = map.map_keys();
    assert_eq!(keys.len(), len);

    for key in keys {
        let value = map.map_get(&key);
        assert!(value.is_some(), "Value for key '{}' should be accessible", key);
    }

    // Non-existent key should return None
    assert!(map.map_get("nonexistent").is_none());
}

/// **Validates: Requirements 7.7**
/// Property: Collection mutations are properly reflected
#[test]
fn property_collection_mutation() {
    // Test Vec mutation
    let mut vec = vec![10i32, 20, 30];

    let elem_mut = vec.collection_get_mut(1).expect("Should get mutable element");
    let value_mut = elem_mut.as_any_mut().downcast_mut::<i32>().expect("Should downcast");
    *value_mut = 99;

    assert_eq!(vec[1], 99, "Mutation should be reflected in original vec");

    // Test HashMap mutation
    let mut map = HashMap::new();
    map.insert("x".to_string(), 5i32);

    let value_mut = map.map_get_mut("x").expect("Should get mutable value");
    let int_mut = value_mut.as_any_mut().downcast_mut::<i32>().expect("Should downcast");
    *int_mut = 42;

    assert_eq!(map["x"], 42, "Mutation should be reflected in original map");
}
