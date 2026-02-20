//! Unit tests for scene serialization system
//!
//! Tests cover:
//! - Entity serialization/deserialization
//! - Component serialization/deserialization
//! - Version tagging
//! - Data integrity validation
//! - Error handling

use luminara_core::World;
use luminara_math::{Transform, Vec3, Quat};
use luminara_scene::{Scene, Name, SCENE_FORMAT_VERSION};

#[test]
fn test_empty_scene_serialization() {
    let world = World::new();
    let scene = Scene::from_world(&world);
    
    // Verify scene metadata
    assert_eq!(scene.meta.version, SCENE_FORMAT_VERSION);
    assert_eq!(scene.entities.len(), 0);
}

#[test]
fn test_single_entity_serialization() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("TestEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    
    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].name, "TestEntity");
}

#[test]
fn test_entity_with_transform_serialization() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("TransformEntity")).unwrap();
    
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    world.add_component(entity, transform).unwrap();
    
    let scene = Scene::from_world(&world);
    
    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].name, "TransformEntity");
    assert!(scene.entities[0].components.contains_key("Transform"));
}

#[test]
fn test_scene_ron_serialization() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("TestEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();
    
    // Verify RON string contains expected data
    assert!(ron_string.contains("TestEntity"));
    assert!(ron_string.contains(SCENE_FORMAT_VERSION));
}

#[test]
fn test_scene_ron_deserialization() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("OriginalEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();
    
    // Deserialize from RON
    let loaded_scene = Scene::from_ron(&ron_string).unwrap();
    
    assert_eq!(loaded_scene.entities.len(), 1);
    assert_eq!(loaded_scene.entities[0].name, "OriginalEntity");
}

#[test]
fn test_scene_roundtrip_serialization() {
    let mut world = World::new();
    
    // Create entity with transform
    let entity = world.spawn();
    world.add_component(entity, Name::new("RoundtripEntity")).unwrap();
    let transform = Transform {
        translation: Vec3::new(5.0, 10.0, 15.0),
        rotation: Quat::from_xyzw(0.0, 0.707, 0.0, 0.707),
        scale: Vec3::new(2.0, 2.0, 2.0),
    };
    world.add_component(entity, transform).unwrap();
    
    // Serialize to RON
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();
    
    // Deserialize from RON
    let loaded_scene = Scene::from_ron(&ron_string).unwrap();
    
    // Spawn into new world
    let mut new_world = World::new();
    let spawned_entities = loaded_scene.spawn_into(&mut new_world);
    
    assert_eq!(spawned_entities.len(), 1);
    
    // Verify transform was preserved
    let loaded_transform = new_world.get_component::<Transform>(spawned_entities[0]).unwrap();
    assert!((loaded_transform.translation.x - 5.0).abs() < 0.001);
    assert!((loaded_transform.translation.y - 10.0).abs() < 0.001);
    assert!((loaded_transform.translation.z - 15.0).abs() < 0.001);
    assert!((loaded_transform.scale.x - 2.0).abs() < 0.001);
}

#[test]
fn test_multiple_entities_serialization() {
    let mut world = World::new();
    
    for i in 0..5 {
        let entity = world.spawn();
        world.add_component(entity, Name::new(format!("Entity_{}", i))).unwrap();
    }
    
    let scene = Scene::from_world(&world);
    assert_eq!(scene.entities.len(), 5);
    
    // Verify all entities are present
    for i in 0..5 {
        let name = format!("Entity_{}", i);
        assert!(scene.entities.iter().any(|e| e.name == name));
    }
}

#[test]
fn test_scene_validation_success() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("ValidEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    
    // Validation should succeed
    assert!(scene.validate().is_ok());
}

#[test]
fn test_scene_version_tagging() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("VersionedEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    
    // Verify version is set correctly
    assert_eq!(scene.meta.version, SCENE_FORMAT_VERSION);
}

#[test]
fn test_scene_json_serialization() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("JsonEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    let json_string = scene.to_json().unwrap();
    
    // Verify JSON string contains expected data
    assert!(json_string.contains("JsonEntity"));
    assert!(json_string.contains(SCENE_FORMAT_VERSION));
}

#[test]
fn test_scene_json_deserialization() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("JsonEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    let json_string = scene.to_json().unwrap();
    
    // Deserialize from JSON
    let loaded_scene = Scene::from_json(&json_string).unwrap();
    
    assert_eq!(loaded_scene.entities.len(), 1);
    assert_eq!(loaded_scene.entities[0].name, "JsonEntity");
}

#[test]
fn test_entity_without_name_gets_default_name() {
    let mut world = World::new();
    let entity = world.spawn();
    // Don't add a Name component
    
    let scene = Scene::from_world(&world);
    
    assert_eq!(scene.entities.len(), 1);
    // Should have a default name like "Entity_0"
    assert!(scene.entities[0].name.starts_with("Entity_"));
}

#[test]
fn test_transform_component_serialization() {
    let mut world = World::new();
    let entity = world.spawn();
    
    let transform = Transform {
        translation: Vec3::new(100.0, 200.0, 300.0),
        rotation: Quat::from_xyzw(0.0, 0.0, 0.707, 0.707),
        scale: Vec3::new(3.0, 4.0, 5.0),
    };
    world.add_component(entity, transform).unwrap();
    
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();
    
    // Verify transform data is in RON
    assert!(ron_string.contains("Transform"));
    assert!(ron_string.contains("translation"));
    assert!(ron_string.contains("rotation"));
    assert!(ron_string.contains("scale"));
}

#[test]
fn test_scene_spawn_into_empty_world() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("SpawnTest")).unwrap();
    
    let scene = Scene::from_world(&world);
    
    // Spawn into a new empty world
    let mut new_world = World::new();
    let spawned = scene.spawn_into(&mut new_world);
    
    assert_eq!(spawned.len(), 1);
    assert_eq!(new_world.entities().len(), 1);
}

#[test]
fn test_scene_spawn_into_existing_world() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("SceneEntity")).unwrap();
    
    let scene = Scene::from_world(&world);
    
    // Create a world with existing entities
    let mut target_world = World::new();
    target_world.spawn();
    target_world.spawn();
    
    let initial_count = target_world.entities().len();
    let spawned = scene.spawn_into(&mut target_world);
    
    assert_eq!(spawned.len(), 1);
    assert_eq!(target_world.entities().len(), initial_count + 1);
}

#[test]
fn test_empty_scene_ron_format() {
    let world = World::new();
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();
    
    // Should be valid RON
    let _: Scene = ron::from_str(&ron_string).unwrap();
}

#[test]
fn test_scene_metadata_preservation() {
    let mut world = World::new();
    let entity = world.spawn();
    world.add_component(entity, Name::new("MetaTest")).unwrap();
    
    let scene = Scene::from_world(&world);
    let ron_string = scene.to_ron().unwrap();
    let loaded_scene = Scene::from_ron(&ron_string).unwrap();
    
    // Verify metadata is preserved
    assert_eq!(loaded_scene.meta.version, scene.meta.version);
    assert_eq!(loaded_scene.meta.name, scene.meta.name);
}
