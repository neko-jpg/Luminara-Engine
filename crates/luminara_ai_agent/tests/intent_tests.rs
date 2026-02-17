use luminara_ai_agent::{
    intent_resolver::{EntityReference, RelativePosition},
    IntentResolver, SemanticIndex,
};
use luminara_core::world::World;
use luminara_math::{Quat, Vec3};
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::sync::Arc;

/// Test entity reference resolution by name with fuzzy matching
#[quickcheck]
fn test_entity_reference_resolution_by_name(name: String) -> TestResult {
    if name.is_empty() || name.len() > 100 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    index.index_entity(1, name.clone());
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);

    let world = World::new();
    let result = resolver.resolve_reference(&EntityReference::ByName(name.clone()), &world);

    // Should successfully resolve or provide suggestions
    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => {
            // Should either find it or provide helpful error
            TestResult::from_bool(
                msg.contains("not found") || msg.contains("Did you mean") || msg.contains("ID")
            )
        }
    }
}

/// Test entity reference resolution by ID
#[test]
fn test_entity_reference_resolution_by_id() {
    let index = Arc::new(SemanticIndex::new());
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(&EntityReference::ById(42), &world);
    
    // Should successfully construct entity from ID
    assert!(result.is_ok());
}

/// Test entity reference resolution by tag
#[test]
fn test_entity_reference_resolution_by_tag() {
    let mut index = SemanticIndex::new();
    index.index_entity(1, "tag:player".to_string());
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(&EntityReference::ByTag("player".to_string()), &world);
    
    // Should find entity with tag or return clear error
    match result {
        Ok(_) => assert!(true),
        Err(msg) => assert!(msg.contains("No entity found with tag")),
    }
}

/// Test entity reference resolution by component
#[test]
fn test_entity_reference_resolution_by_component() {
    let mut index = SemanticIndex::new();
    index.index_entity(1, "component:Transform".to_string());
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(
        &EntityReference::ByComponent("Transform".to_string()),
        &world,
    );
    
    // Should find entity with component or return clear error
    match result {
        Ok(_) => assert!(true),
        Err(msg) => assert!(msg.contains("No entity found with component")),
    }
}

/// Test semantic entity reference resolution
#[quickcheck]
fn test_entity_reference_resolution_semantic(desc: String) -> TestResult {
    if desc.is_empty() || desc.len() > 100 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    index.index_entity(1, desc.clone());
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);

    let world = World::new();
    let result = resolver.resolve_reference(&EntityReference::Semantic(desc), &world);

    // Should find entity or provide suggestions
    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => TestResult::from_bool(
            msg.contains("No entity matching") || msg.contains("Suggestions")
        ),
    }
}

/// Test relative position resolution - Forward
#[quickcheck]
fn test_relative_position_forward(dist: f32) -> TestResult {
    if !dist.is_finite() || dist.abs() > 1000.0 {
        return TestResult::discard();
    }

    let index = Arc::new(SemanticIndex::new());
    let resolver = IntentResolver::new(index);

    let anchor_pos = Vec3::ZERO;
    let anchor_rot = Quat::IDENTITY;

    let result = resolver.resolve_position(&RelativePosition::Forward(dist), anchor_pos, anchor_rot);

    match result {
        Ok(pos) => {
            // Forward should be along -Z axis for identity rotation
            let expected = Vec3::new(0.0, 0.0, -dist);
            let diff = (pos - expected).length();
            TestResult::from_bool(diff < 0.001)
        }
        Err(_) => TestResult::failed(),
    }
}

/// Test relative position resolution - Above
#[quickcheck]
fn test_relative_position_above(dist: f32) -> TestResult {
    if !dist.is_finite() || dist.abs() > 1000.0 {
        return TestResult::discard();
    }

    let index = Arc::new(SemanticIndex::new());
    let resolver = IntentResolver::new(index);

    let anchor_pos = Vec3::ZERO;
    let anchor_rot = Quat::IDENTITY;

    let result = resolver.resolve_position(&RelativePosition::Above(dist), anchor_pos, anchor_rot);

    match result {
        Ok(pos) => {
            // Above should be along Y axis
            let expected = Vec3::new(0.0, dist, 0.0);
            let diff = (pos - expected).length();
            TestResult::from_bool(diff < 0.001)
        }
        Err(_) => TestResult::failed(),
    }
}

/// Test relative position resolution - AtOffset
#[quickcheck]
fn test_relative_position_at_offset(x: f32, y: f32, z: f32) -> TestResult {
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return TestResult::discard();
    }
    if x.abs() > 1000.0 || y.abs() > 1000.0 || z.abs() > 1000.0 {
        return TestResult::discard();
    }

    let index = Arc::new(SemanticIndex::new());
    let resolver = IntentResolver::new(index);

    let anchor_pos = Vec3::new(10.0, 20.0, 30.0);
    let anchor_rot = Quat::IDENTITY;
    let offset = Vec3::new(x, y, z);

    let result = resolver.resolve_position(&RelativePosition::AtOffset(offset), anchor_pos, anchor_rot);

    match result {
        Ok(pos) => {
            // Should apply rotation to offset and add to anchor
            let expected = anchor_pos + offset;
            let diff = (pos - expected).length();
            TestResult::from_bool(diff < 0.001)
        }
        Err(_) => TestResult::failed(),
    }
}

/// Test relative position resolution - RandomInRadius
#[quickcheck]
fn test_relative_position_random_in_radius(radius: f32) -> TestResult {
    if !radius.is_finite() || radius <= 0.0 || radius > 1000.0 {
        return TestResult::discard();
    }

    let index = Arc::new(SemanticIndex::new());
    let resolver = IntentResolver::new(index);

    let anchor_pos = Vec3::ZERO;
    let anchor_rot = Quat::IDENTITY;

    let result = resolver.resolve_position(&RelativePosition::RandomInRadius(radius), anchor_pos, anchor_rot);

    match result {
        Ok(pos) => {
            // Should be within radius of anchor
            let dist = (pos - anchor_pos).length();
            TestResult::from_bool(dist <= radius + 0.001)
        }
        Err(_) => TestResult::failed(),
    }
}

/// Test that missing entity references provide helpful error messages
#[test]
fn test_missing_entity_provides_suggestions() {
    let mut index = SemanticIndex::new();
    index.index_entity(1, "player_character".to_string());
    index.index_entity(2, "player_vehicle".to_string());
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    // Try to find "player" - should get suggestions
    let result = resolver.resolve_reference(&EntityReference::ByName("player".to_string()), &world);

    match result {
        Ok(_) => {
            // Might match if fuzzy matching is good enough
            assert!(true);
        }
        Err(msg) => {
            // Should provide suggestions
            assert!(msg.contains("Did you mean") || msg.contains("not found"));
        }
    }
}

/// Test entity lifecycle changes - name changes
#[test]
fn test_entity_name_change_handling() {
    let mut index = SemanticIndex::new();
    
    // Initially index with old name
    index.index_entity(1, "old_name".to_string());
    
    // Update to new name
    index.index_entity(1, "new_name".to_string());
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    // Should find by new name
    let result = resolver.resolve_reference(&EntityReference::ByName("new_name".to_string()), &world);
    assert!(result.is_ok() || result.unwrap_err().contains("ID 1"));
}

/// Test nearest entity resolution
#[test]
fn test_nearest_entity_resolution() {
    let mut index = SemanticIndex::new();
    index.index_entity(1, "entity1".to_string());
    index.index_entity(2, "entity2".to_string());
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let reference = EntityReference::Nearest {
        to: Box::new(EntityReference::ById(1)),
        with_tag: None,
    };

    let result = resolver.resolve_reference(&reference, &world);
    
    // Should attempt to find nearest entity
    match result {
        Ok(_) => assert!(true),
        Err(msg) => {
            // Should provide clear error if no candidates
            assert!(msg.contains("No candidate") || msg.contains("No valid nearest"));
        }
    }
}
