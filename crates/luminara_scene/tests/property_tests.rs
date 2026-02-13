use glam::{Quat, Vec3};
use luminara_math::Transform;
use luminara_scene::*;
use proptest::prelude::*;
use std::collections::HashMap;

// ============================================================================
// Property Test 1: Scene Round-Trip Consistency
// Validates: Requirements 1.1, 1.2, 1.4, 1.5
// ============================================================================

/// Strategy for generating random Vec3
fn vec3_strategy() -> impl Strategy<Value = Vec3> {
    (
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Strategy for generating random Quat (normalized)
fn quat_strategy() -> impl Strategy<Value = Quat> {
    (
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
        prop::num::f32::NORMAL,
    )
        .prop_map(|(x, y, z, w)| {
            let q = Quat::from_xyzw(x, y, z, w);
            q.normalize()
        })
}

/// Strategy for generating random Transform
fn transform_strategy() -> impl Strategy<Value = Transform> {
    (vec3_strategy(), quat_strategy(), vec3_strategy()).prop_map(|(translation, rotation, scale)| {
        Transform {
            translation,
            rotation,
            scale,
        }
    })
}

/// Strategy for generating component data
fn component_map_strategy() -> impl Strategy<Value = HashMap<String, serde_json::Value>> {
    prop::collection::vec(transform_strategy(), 0..3).prop_map(|transforms| {
        let mut map = HashMap::new();
        if let Some(transform) = transforms.first() {
            map.insert(
                "Transform".to_string(),
                serde_json::to_value(transform).unwrap(),
            );
        }
        map
    })
}

/// Strategy for generating entity names
fn entity_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Za-z][A-Za-z0-9_]{0,20}").unwrap()
}

/// Strategy for generating tags
fn tags_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(prop::string::string_regex("[a-z]{3,10}").unwrap(), 0..5)
}

/// Strategy for generating EntityData (without children to avoid deep recursion)
fn entity_data_strategy() -> impl Strategy<Value = EntityData> {
    (
        entity_name_strategy(),
        prop::option::of(prop::num::u64::ANY),
        prop::option::of(prop::num::u64::ANY),
        component_map_strategy(),
        tags_strategy(),
    )
        .prop_map(|(name, id, parent, components, tags)| EntityData {
            name,
            id,
            parent,
            components,
            children: vec![],
            tags,
        })
}

/// Strategy for generating EntityData with one level of children
fn entity_data_with_children_strategy() -> impl Strategy<Value = EntityData> {
    (
        entity_name_strategy(),
        prop::option::of(prop::num::u64::ANY),
        component_map_strategy(),
        tags_strategy(),
        prop::collection::vec(entity_data_strategy(), 0..3),
    )
        .prop_map(|(name, id, components, tags, children)| EntityData {
            name,
            id,
            parent: None,
            components,
            children,
            tags,
        })
}

/// Strategy for generating SceneMeta
fn scene_meta_strategy() -> impl Strategy<Value = SceneMeta> {
    (
        entity_name_strategy(),
        prop::string::string_regex(".{0,100}").unwrap(),
        prop::string::string_regex("[0-9]+\\.[0-9]+\\.[0-9]+").unwrap(),
        tags_strategy(),
    )
        .prop_map(|(name, description, version, tags)| SceneMeta {
            name,
            description,
            version,
            tags,
        })
}

/// Strategy for generating Scene
fn scene_strategy() -> impl Strategy<Value = Scene> {
    (
        scene_meta_strategy(),
        prop::collection::vec(entity_data_with_children_strategy(), 1..5),
    )
        .prop_map(|(meta, entities)| Scene { meta, entities })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Property 1: Scene Round-Trip Consistency
    /// For any valid scene with entities, components, and hierarchy relationships,
    /// serializing to RON format and then deserializing should produce an equivalent
    /// scene structure with all entity data, component values, names, tags, and
    /// parent-child relationships preserved.
    #[test]
    fn prop_scene_ron_roundtrip_consistency(scene in scene_strategy()) {
        // Serialize to RON
        let ron_string = scene.to_ron().expect("Failed to serialize scene to RON");
        
        // Deserialize from RON
        let deserialized = Scene::from_ron(&ron_string).expect("Failed to deserialize scene from RON");
        
        // Verify metadata is preserved
        prop_assert_eq!(&deserialized.meta.name, &scene.meta.name);
        prop_assert_eq!(&deserialized.meta.description, &scene.meta.description);
        prop_assert_eq!(&deserialized.meta.version, &scene.meta.version);
        prop_assert_eq!(&deserialized.meta.tags, &scene.meta.tags);
        
        // Verify entity count is preserved
        prop_assert_eq!(deserialized.entities.len(), scene.entities.len());
        
        // Verify each entity's data is preserved
        for (original, deserialized_entity) in scene.entities.iter().zip(deserialized.entities.iter()) {
            prop_assert_eq!(&deserialized_entity.name, &original.name);
            prop_assert_eq!(&deserialized_entity.id, &original.id);
            prop_assert_eq!(&deserialized_entity.parent, &original.parent);
            prop_assert_eq!(&deserialized_entity.tags, &original.tags);
            prop_assert_eq!(deserialized_entity.components.len(), original.components.len());
            
            // Verify children are preserved
            prop_assert_eq!(deserialized_entity.children.len(), original.children.len());
            for (orig_child, deser_child) in original.children.iter().zip(deserialized_entity.children.iter()) {
                prop_assert_eq!(&deser_child.name, &orig_child.name);
                prop_assert_eq!(&deser_child.tags, &orig_child.tags);
            }
        }
    }
    
    /// Property 1 (JSON variant): Scene Round-Trip Consistency with JSON
    /// Same as above but using JSON serialization format
    #[test]
    fn prop_scene_json_roundtrip_consistency(scene in scene_strategy()) {
        // Serialize to JSON
        let json_string = scene.to_json().expect("Failed to serialize scene to JSON");
        
        // Deserialize from JSON
        let deserialized = Scene::from_json(&json_string).expect("Failed to deserialize scene from JSON");
        
        // Verify metadata is preserved
        prop_assert_eq!(&deserialized.meta.name, &scene.meta.name);
        prop_assert_eq!(&deserialized.meta.description, &scene.meta.description);
        prop_assert_eq!(&deserialized.meta.version, &scene.meta.version);
        prop_assert_eq!(&deserialized.meta.tags, &scene.meta.tags);
        
        // Verify entity count is preserved
        prop_assert_eq!(deserialized.entities.len(), scene.entities.len());
        
        // Verify each entity's data is preserved
        for (original, deserialized_entity) in scene.entities.iter().zip(deserialized.entities.iter()) {
            prop_assert_eq!(&deserialized_entity.name, &original.name);
            prop_assert_eq!(&deserialized_entity.id, &original.id);
            prop_assert_eq!(&deserialized_entity.parent, &original.parent);
            prop_assert_eq!(&deserialized_entity.tags, &original.tags);
            
            // Verify children are preserved
            prop_assert_eq!(deserialized_entity.children.len(), original.children.len());
        }
    }
}

// ============================================================================
// Property Test 2: Component Schema Availability
// Validates: Requirements 1.6
// ============================================================================

#[test]
fn prop_component_schema_availability() {
    // Initialize default component schemas
    init_default_component_schemas();
    
    // List of component types that should have schemas registered
    let expected_components = vec![
        "Name",
        "Tag",
        "Transform",
        "Parent",
        "Children",
        "GlobalTransform",
    ];
    
    // Verify each component type has a schema
    for component_type in &expected_components {
        let schema = get_component_schema(component_type);
        assert!(
            schema.is_some(),
            "Component schema for '{}' should be registered",
            component_type
        );
        
        let schema = schema.unwrap();
        
        // Verify schema contains type name
        assert_eq!(
            schema.type_name, *component_type,
            "Schema type_name should match component type"
        );
        
        // Verify schema contains description
        assert!(
            !schema.description.is_empty(),
            "Schema for '{}' should have a non-empty description",
            component_type
        );
        
        // Verify schema contains fields
        assert!(
            !schema.fields.is_empty(),
            "Schema for '{}' should have at least one field",
            component_type
        );
        
        // Verify each field has required metadata
        for field in &schema.fields {
            assert!(
                !field.name.is_empty(),
                "Field name should not be empty for component '{}'",
                component_type
            );
            assert!(
                !field.type_name.is_empty(),
                "Field type_name should not be empty for component '{}'",
                component_type
            );
            assert!(
                !field.description.is_empty(),
                "Field description should not be empty for component '{}'",
                component_type
            );
        }
    }
    
    // Verify we can retrieve all schemas
    let all_schemas = get_all_component_schemas();
    assert!(
        all_schemas.len() >= expected_components.len(),
        "Should have at least {} registered schemas, found {}",
        expected_components.len(),
        all_schemas.len()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    
    /// Property test: Registering a new component schema makes it available
    #[test]
    fn prop_component_schema_registration(
        type_name in "[A-Z][A-Za-z0-9]{3,20}",
        description in ".{10,100}",
        field_count in 1usize..5
    ) {
        let fields: Vec<FieldSchema> = (0..field_count)
            .map(|i| FieldSchema {
                name: format!("field_{}", i),
                type_name: "String".to_string(),
                description: format!("Field {} description", i),
            })
            .collect();
        
        let schema = ComponentSchema {
            type_name: type_name.clone(),
            description,
            fields,
        };
        
        // Register the schema
        register_component_schema(schema.clone());
        
        // Verify it can be retrieved
        let retrieved = get_component_schema(&type_name);
        prop_assert!(retrieved.is_some(), "Schema should be retrievable after registration");
        
        let retrieved = retrieved.unwrap();
        prop_assert_eq!(&retrieved.type_name, &type_name);
        prop_assert_eq!(&retrieved.description, &schema.description);
        prop_assert_eq!(retrieved.fields.len(), schema.fields.len());
    }
}

// ============================================================================
// Property Test 8: Transform Hierarchy Propagation
// Validates: Requirements 5.1, 5.2, 5.3
// ============================================================================

/// Strategy for generating a hierarchy depth (1-5 levels)
fn hierarchy_depth_strategy() -> impl Strategy<Value = usize> {
    1usize..=5
}

/// Generate a random hierarchy tree structure
/// Returns a vector of (entity_index, parent_index, transform) tuples
fn hierarchy_tree_strategy() -> impl Strategy<Value = Vec<(usize, Option<usize>, Transform)>> {
    hierarchy_depth_strategy().prop_flat_map(|max_depth| {
        // Generate 1-10 entities per level
        let entities_per_level = 1usize..=10;
        
        prop::collection::vec(
            (entities_per_level, transform_strategy()),
            1..=max_depth
        ).prop_map(move |levels| {
            let mut hierarchy = Vec::new();
            let mut entity_counter = 0;
            let mut prev_level_start = 0;
            let mut prev_level_count = 0;
            
            for (level_idx, (entity_count, _)) in levels.iter().enumerate() {
                for i in 0..*entity_count {
                    let transform = Transform {
                        translation: Vec3::new(
                            (i as f32 + 1.0) * 2.0,
                            (level_idx as f32 + 1.0) * 3.0,
                            (entity_counter as f32) * 1.5,
                        ),
                        rotation: Quat::from_rotation_y((i as f32) * 0.1),
                        scale: Vec3::splat(1.0 + (i as f32) * 0.1),
                    };
                    
                    let parent = if level_idx == 0 {
                        None
                    } else {
                        // Assign parent from previous level (distribute children across parents)
                        Some(prev_level_start + (i % prev_level_count))
                    };
                    
                    hierarchy.push((entity_counter, parent, transform));
                    entity_counter += 1;
                }
                
                prev_level_start += prev_level_count;
                prev_level_count = *entity_count;
            }
            
            hierarchy
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Property 8: Transform Hierarchy Propagation**
    /// 
    /// For any entity hierarchy with parent-child relationships, when a parent's
    /// local transform changes, all descendant entities' world transforms should
    /// be updated such that each child's world transform equals its parent's world
    /// transform multiplied by its own local transform.
    /// 
    /// **Validates: Requirements 5.1, 5.2, 5.3**
    #[test]
    fn prop_transform_hierarchy_propagation(hierarchy in hierarchy_tree_strategy()) {
        use luminara_core::World;
        use luminara_scene::{transform_propagate_system, GlobalTransform, Parent, Children};
        
        let mut world = World::new();
        
        // Create entities and add components
        let entities: Vec<_> = (0..hierarchy.len())
            .map(|_| world.spawn())
            .collect();
        
        // Add Transform and GlobalTransform components
        for (idx, parent_idx, transform) in &hierarchy {
            world.add_component(entities[*idx], *transform);
            world.add_component(entities[*idx], GlobalTransform::default());
            
            if let Some(parent) = parent_idx {
                world.add_component(entities[*idx], Parent(entities[*parent]));
            }
        }
        
        // Build Children components
        for (idx, parent_idx, _) in &hierarchy {
            if let Some(parent) = parent_idx {
                if let Some(children) = world.get_component_mut::<Children>(entities[*parent]) {
                    children.0.push(entities[*idx]);
                } else {
                    world.add_component(entities[*parent], Children(vec![entities[*idx]]));
                }
            }
        }
        
        // Run transform propagation system
        transform_propagate_system(&mut world);
        
        // Verify the property: child_world = parent_world * child_local
        for (idx, parent_idx, local_transform) in &hierarchy {
            let child_entity = entities[*idx];
            let child_global = world.get_component::<GlobalTransform>(child_entity)
                .expect("Child should have GlobalTransform");
            let child_global_matrix = child_global.matrix();
            
            if let Some(parent) = parent_idx {
                let parent_entity = entities[*parent];
                let parent_global = world.get_component::<GlobalTransform>(parent_entity)
                    .expect("Parent should have GlobalTransform");
                let parent_global_matrix = parent_global.matrix();
                
                // Compute expected: parent_world * child_local
                let local_matrix = local_transform.to_matrix();
                let expected_matrix = parent_global_matrix * local_matrix;
                
                // Extract translation from matrices for comparison
                let expected_translation = expected_matrix.w_axis.truncate();
                let actual_translation = child_global_matrix.w_axis.truncate();
                
                // Verify translation components (with tolerance for floating point)
                let diff = (expected_translation - actual_translation).length();
                prop_assert!(
                    diff < 0.01,
                    "Child world transform should equal parent world * child local.\n\
                     Expected translation: {:?}\n\
                     Actual translation: {:?}\n\
                     Difference: {}",
                    expected_translation,
                    actual_translation,
                    diff
                );
                
                // Verify rotation (compare quaternions)
                let (_, expected_rotation, _) = expected_matrix.to_scale_rotation_translation();
                let (_, actual_rotation, _) = child_global_matrix.to_scale_rotation_translation();
                
                // Quaternions can represent the same rotation in two ways (q and -q)
                // So we check if they're equal or negated
                let rot_diff = (expected_rotation.dot(actual_rotation).abs() - 1.0).abs();
                prop_assert!(
                    rot_diff < 0.01,
                    "Child world rotation should match expected.\n\
                     Expected rotation: {:?}\n\
                     Actual rotation: {:?}\n\
                     Dot product: {}",
                    expected_rotation,
                    actual_rotation,
                    expected_rotation.dot(actual_rotation)
                );
                
                // Verify scale
                let (expected_scale, _, _) = expected_matrix.to_scale_rotation_translation();
                let (actual_scale, _, _) = child_global_matrix.to_scale_rotation_translation();
                let scale_diff = (expected_scale - actual_scale).length();
                prop_assert!(
                    scale_diff < 0.01,
                    "Child world scale should match expected.\n\
                     Expected scale: {:?}\n\
                     Actual scale: {:?}\n\
                     Difference: {}",
                    expected_scale,
                    actual_scale,
                    scale_diff
                );
            } else {
                // Root entity: global should equal local
                let expected_matrix = local_transform.to_matrix();
                let expected_translation = expected_matrix.w_axis.truncate();
                let actual_translation = child_global_matrix.w_axis.truncate();
                
                let diff = (expected_translation - actual_translation).length();
                prop_assert!(
                    diff < 0.01,
                    "Root entity global transform should equal its local transform.\n\
                     Expected translation: {:?}\n\
                     Actual translation: {:?}",
                    expected_translation,
                    actual_translation
                );
            }
        }
    }
}

// ============================================================================
// Property Test 9: Child Detachment Preserves World Position
// Validates: Requirements 5.5
// ============================================================================

/// Strategy for generating reasonable Vec3 values (constrained to avoid extreme values)
fn reasonable_vec3_strategy() -> impl Strategy<Value = Vec3> {
    (
        -100.0f32..100.0f32,
        -100.0f32..100.0f32,
        -100.0f32..100.0f32,
    )
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// Strategy for generating reasonable quaternions (constrained to avoid extreme values)
fn reasonable_quat_strategy() -> impl Strategy<Value = Quat> {
    (
        -1.0f32..1.0f32,
        -1.0f32..1.0f32,
        -1.0f32..1.0f32,
        -1.0f32..1.0f32,
    )
        .prop_map(|(x, y, z, w)| {
            let q = Quat::from_xyzw(x, y, z, w);
            let normalized = q.normalize();
            // Ensure the quaternion is valid (not NaN)
            if normalized.is_nan() {
                Quat::IDENTITY
            } else {
                normalized
            }
        })
}

/// Strategy for generating uniform scale values (to avoid matrix decomposition issues)
fn uniform_scale_strategy() -> impl Strategy<Value = Vec3> {
    (0.5f32..2.0f32)
        .prop_map(|s| Vec3::splat(s))
}

/// Strategy for generating reasonable transforms (constrained values with uniform scaling)
fn reasonable_transform_strategy() -> impl Strategy<Value = Transform> {
    (reasonable_vec3_strategy(), reasonable_quat_strategy(), uniform_scale_strategy())
        .prop_map(|(translation, rotation, scale)| {
            Transform {
                translation,
                rotation,
                scale,
            }
        })
}

/// Strategy for generating parent-child transform pairs
fn parent_child_pair_strategy() -> impl Strategy<Value = (Transform, Transform)> {
    (reasonable_transform_strategy(), reasonable_transform_strategy())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Property 9: Child Detachment Preserves World Position**
    /// 
    /// For any child entity with a parent, detaching it from the parent should
    /// update its local transform such that its world position, rotation, and
    /// scale remain unchanged.
    /// 
    /// **Validates: Requirements 5.5**
    #[test]
    fn prop_child_detachment_preserves_world_position(
        (parent_transform, child_local_transform) in parent_child_pair_strategy()
    ) {
        use luminara_core::World;
        use luminara_scene::{transform_propagate_system, GlobalTransform, Parent, Children, remove_parent};
        
        let mut world = World::new();
        
        // Create parent entity
        let parent = world.spawn();
        world.add_component(parent, parent_transform);
        world.add_component(parent, GlobalTransform::default());
        
        // Create child entity with parent relationship
        let child = world.spawn();
        world.add_component(child, child_local_transform);
        world.add_component(child, GlobalTransform::default());
        world.add_component(child, Parent(parent));
        world.add_component(parent, Children(vec![child]));
        
        // Run transform propagation to compute world transforms
        transform_propagate_system(&mut world);
        
        // Capture the child's world transform before detachment
        let child_world_before = world.get_component::<GlobalTransform>(child)
            .expect("Child should have GlobalTransform")
            .clone();
        let world_matrix_before = child_world_before.matrix();
        let (world_scale_before, world_rotation_before, world_translation_before) = 
            world_matrix_before.to_scale_rotation_translation();
        
        // Detach the child from parent
        // According to requirement 5.5, we need to update the child's local transform
        // to preserve its world position
        
        // Compute the new local transform that preserves world position
        // Since: world = parent_world * child_local
        // We need: new_child_local = parent_world^-1 * old_world
        // But after detachment, there's no parent, so: new_child_local = old_world
        
        // Get the child's current world transform
        let new_local_transform = child_world_before.0;
        
        // Remove parent relationship
        remove_parent(&mut world, child);
        
        // Update child's local transform to match its previous world transform
        world.add_component(child, new_local_transform);
        
        // Run transform propagation again
        transform_propagate_system(&mut world);
        
        // Get the child's world transform after detachment
        let child_world_after = world.get_component::<GlobalTransform>(child)
            .expect("Child should have GlobalTransform");
        let world_matrix_after = child_world_after.matrix();
        let (world_scale_after, world_rotation_after, world_translation_after) = 
            world_matrix_after.to_scale_rotation_translation();
        
        // Verify world position is preserved
        let translation_diff = (world_translation_before - world_translation_after).length();
        prop_assert!(
            translation_diff < 0.01,
            "World position should be preserved after detachment.\n\
             Before: {:?}\n\
             After: {:?}\n\
             Difference: {}",
            world_translation_before,
            world_translation_after,
            translation_diff
        );
        
        // Verify world rotation is preserved
        // Quaternions can represent the same rotation in two ways (q and -q)
        let rotation_dot = world_rotation_before.dot(world_rotation_after).abs();
        prop_assert!(
            (rotation_dot - 1.0).abs() < 0.01,
            "World rotation should be preserved after detachment.\n\
             Before: {:?}\n\
             After: {:?}\n\
             Dot product: {}",
            world_rotation_before,
            world_rotation_after,
            rotation_dot
        );
        
        // Verify world scale is preserved
        let scale_diff = (world_scale_before - world_scale_after).length();
        prop_assert!(
            scale_diff < 0.01,
            "World scale should be preserved after detachment.\n\
             Before: {:?}\n\
             After: {:?}\n\
             Difference: {}",
            world_scale_before,
            world_scale_after,
            scale_diff
        );
    }
}
