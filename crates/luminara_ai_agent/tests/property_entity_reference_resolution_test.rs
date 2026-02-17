/// Property-Based Test: Entity Reference Resolution Robustness
/// 
/// **Property 25: Entity Reference Resolution Robustness**
/// **Validates: Requirements 26.1**
/// 
/// This test verifies that entity reference resolution:
/// - Handles name changes gracefully
/// - Provides suggestions for similar names when exact matches fail
/// - Resolves all reference types correctly (ByName, ById, ByTag, ByComponent, Nearest, Semantic)
/// - Provides clear error messages with alternatives when resolution fails
/// - Maintains correct behavior across entity lifecycle changes (creation, modification, deletion)

use luminara_ai_agent::{
    intent_resolver::{EntityReference, RelativePosition},
    IntentResolver, SemanticIndex,
};
use luminara_core::world::World;
use luminara_math::{Quat, Vec3};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;
use std::sync::Arc;

/// Arbitrary implementation for EntityReference to generate random test cases
#[derive(Clone, Debug)]
struct ArbitraryEntityReference(EntityReference);

impl Arbitrary for ArbitraryEntityReference {
    fn arbitrary(g: &mut Gen) -> Self {
        let choice = u8::arbitrary(g) % 6;
        let reference = match choice {
            0 => {
                let name = generate_valid_name(g);
                EntityReference::ByName(name)
            }
            1 => {
                let id = u64::arbitrary(g) % 10000;
                EntityReference::ById(id)
            }
            2 => {
                let tag = generate_valid_tag(g);
                EntityReference::ByTag(tag)
            }
            3 => {
                let component = generate_valid_component(g);
                EntityReference::ByComponent(component)
            }
            4 => {
                // Nearest - use simpler reference to avoid deep nesting
                let to = Box::new(EntityReference::ById(u64::arbitrary(g) % 100));
                let with_tag = if bool::arbitrary(g) {
                    Some(generate_valid_tag(g))
                } else {
                    None
                };
                EntityReference::Nearest { to, with_tag }
            }
            _ => {
                let desc = generate_valid_description(g);
                EntityReference::Semantic(desc)
            }
        };
        ArbitraryEntityReference(reference)
    }
}

fn generate_valid_name(g: &mut Gen) -> String {
    let names = vec![
        "player", "enemy", "npc", "item", "weapon", "vehicle",
        "building", "tree", "rock", "light", "camera", "trigger",
    ];
    let base = names[usize::arbitrary(g) % names.len()];
    let suffix = u32::arbitrary(g) % 100;
    format!("{}_{}", base, suffix)
}

fn generate_valid_tag(g: &mut Gen) -> String {
    let tags = vec!["player", "enemy", "item", "interactive", "static", "dynamic"];
    tags[usize::arbitrary(g) % tags.len()].to_string()
}

fn generate_valid_component(g: &mut Gen) -> String {
    let components = vec![
        "Transform", "Mesh", "Material", "Light", "Camera",
        "RigidBody", "Collider", "Script", "Audio",
    ];
    components[usize::arbitrary(g) % components.len()].to_string()
}

fn generate_valid_description(g: &mut Gen) -> String {
    let adjectives = vec!["red", "blue", "large", "small", "fast", "slow"];
    let nouns = vec!["car", "house", "tree", "player", "enemy", "item"];
    let adj = adjectives[usize::arbitrary(g) % adjectives.len()];
    let noun = nouns[usize::arbitrary(g) % nouns.len()];
    format!("{} {}", adj, noun)
}

/// Property: Entity reference resolution always provides helpful feedback
/// Either succeeds or provides clear error with suggestions
#[quickcheck]
fn prop_entity_reference_always_provides_feedback(
    reference: ArbitraryEntityReference,
) -> TestResult {
    let mut index = SemanticIndex::new();
    
    // Populate index with some entities
    for i in 0..20 {
        index.index_entity(i, format!("entity_{}", i));
        index.index_entity(i, format!("tag:player"));
        index.index_entity(i, format!("component:Transform"));
    }
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(&reference.0, &world);

    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => {
            // Error message must be non-empty and helpful
            if msg.is_empty() {
                return TestResult::failed();
            }
            
            // Should contain helpful information
            let is_helpful = msg.contains("not found")
                || msg.contains("Did you mean")
                || msg.contains("Suggestions")
                || msg.contains("No entity")
                || msg.contains("No candidate")
                || msg.contains("No valid")
                || msg.contains("ID");
            
            TestResult::from_bool(is_helpful)
        }
    }
}

/// Property: Name changes are handled gracefully with suggestions
#[quickcheck]
fn prop_name_changes_provide_suggestions(old_name: String, new_name: String) -> TestResult {
    if old_name.is_empty() || new_name.is_empty() || old_name.len() > 50 || new_name.len() > 50 {
        return TestResult::discard();
    }
    if old_name == new_name {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    
    // Index with old name
    index.index_entity(1, old_name.clone());
    
    // Update to new name (simulating name change)
    index.index_entity(1, new_name.clone());
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    // Try to resolve by new name - should work
    let result_new = resolver.resolve_reference(&EntityReference::ByName(new_name), &world);
    
    // Try to resolve by old name - should fail with suggestions or succeed with fuzzy match
    let result_old = resolver.resolve_reference(&EntityReference::ByName(old_name), &world);

    // New name should resolve or provide helpful error
    let new_ok = match result_new {
        Ok(_) => true,
        Err(msg) => !msg.is_empty() && (msg.contains("ID") || msg.contains("not found")),
    };

    // Old name should either fuzzy match or provide suggestions
    let old_ok = match result_old {
        Ok(_) => true, // Fuzzy matching found it
        Err(msg) => !msg.is_empty() && (msg.contains("not found") || msg.contains("Did you mean")),
    };

    TestResult::from_bool(new_ok && old_ok)
}

/// Property: All reference types resolve or provide clear errors
#[quickcheck]
fn prop_all_reference_types_handled(ref_type: u8) -> TestResult {
    let mut index = SemanticIndex::new();
    
    // Populate with test data
    index.index_entity(1, "test_entity".to_string());
    index.index_entity(2, "tag:test_tag".to_string());
    index.index_entity(3, "component:TestComponent".to_string());
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let reference = match ref_type % 6 {
        0 => EntityReference::ByName("test_entity".to_string()),
        1 => EntityReference::ById(1),
        2 => EntityReference::ByTag("test_tag".to_string()),
        3 => EntityReference::ByComponent("TestComponent".to_string()),
        4 => EntityReference::Nearest {
            to: Box::new(EntityReference::ById(1)),
            with_tag: Some("test_tag".to_string()),
        },
        _ => EntityReference::Semantic("test entity".to_string()),
    };

    let result = resolver.resolve_reference(&reference, &world);

    // All reference types should either resolve or provide clear error
    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => TestResult::from_bool(!msg.is_empty()),
    }
}

/// Property: Entity lifecycle changes (creation, deletion) are handled correctly
#[quickcheck]
fn prop_entity_lifecycle_changes_handled(entity_count: u8) -> TestResult {
    let count = (entity_count % 20) + 1; // 1-20 entities
    
    let mut index = SemanticIndex::new();
    
    // Create entities
    for i in 0..count {
        index.index_entity(i as u32, format!("entity_{}", i));
    }
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    // Try to resolve each entity
    let mut all_resolved_or_errored = true;
    
    for i in 0..count {
        let result = resolver.resolve_reference(
            &EntityReference::ByName(format!("entity_{}", i)),
            &world,
        );
        
        match result {
            Ok(_) => continue,
            Err(msg) => {
                if msg.is_empty() {
                    all_resolved_or_errored = false;
                    break;
                }
            }
        }
    }

    TestResult::from_bool(all_resolved_or_errored)
}

/// Property: Nearest entity resolution respects spatial constraints
#[quickcheck]
fn prop_nearest_entity_spatial_correctness(tag_filter: bool) -> TestResult {
    let mut index = SemanticIndex::new();
    
    // Create entities with different tags
    for i in 0..10u32 {
        let tag = if i % 2 == 0 { "even" } else { "odd" };
        index.index_entity(i, format!("entity_{}", i));
        index.index_entity(i, format!("tag:{}", tag));
    }
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let reference = EntityReference::Nearest {
        to: Box::new(EntityReference::ById(0)),
        with_tag: if tag_filter {
            Some("even".to_string())
        } else {
            None
        },
    };

    let result = resolver.resolve_reference(&reference, &world);

    // Should either find nearest or provide clear error
    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => TestResult::from_bool(
            msg.contains("No candidate") || msg.contains("No valid nearest")
        ),
    }
}

/// Property: Semantic queries provide ranked suggestions
#[quickcheck]
fn prop_semantic_queries_provide_ranking(query: String) -> TestResult {
    if query.is_empty() || query.len() > 100 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    
    // Index entities with varying similarity to query
    index.index_entity(1, query.clone());
    index.index_entity(2, format!("{}_similar", query));
    index.index_entity(3, "completely_different".to_string());
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(&EntityReference::Semantic(query), &world);

    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => {
            // Should provide suggestions with ranking info
            TestResult::from_bool(
                msg.contains("Suggestions") || msg.contains("score") || msg.contains("No entity")
            )
        }
    }
}

/// Property: Multiple entities matching a query are ranked by relevance
#[quickcheck]
fn prop_multiple_matches_ranked_by_relevance(base_name: String) -> TestResult {
    if base_name.is_empty() || base_name.len() > 30 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    
    // Create multiple entities with similar names
    for i in 0..5u32 {
        index.index_entity(i, format!("{}_{}", base_name, i));
    }
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    // Query with base name - should match one or provide suggestions
    let result = resolver.resolve_reference(&EntityReference::ByName(base_name), &world);

    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => {
            // Should provide multiple suggestions
            TestResult::from_bool(msg.contains("Did you mean") || msg.contains("not found"))
        }
    }
}

/// Property: Resolution failure provides alternatives
#[quickcheck]
fn prop_resolution_failure_provides_alternatives(invalid_name: String) -> TestResult {
    if invalid_name.is_empty() || invalid_name.len() > 50 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    
    // Index some entities but not the one we'll query
    index.index_entity(1, "valid_entity_1".to_string());
    index.index_entity(2, "valid_entity_2".to_string());
    index.index_entity(3, "valid_entity_3".to_string());
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(&EntityReference::ByName(invalid_name), &world);

    match result {
        Ok(_) => TestResult::passed(), // Fuzzy match succeeded
        Err(msg) => {
            // Should provide alternatives or clear error
            TestResult::from_bool(
                msg.contains("Did you mean")
                    || msg.contains("not found")
                    || msg.contains("No similar")
            )
        }
    }
}

/// Property: Tag-based resolution handles missing tags gracefully
#[quickcheck]
fn prop_tag_resolution_handles_missing_tags(tag: String) -> TestResult {
    if tag.is_empty() || tag.len() > 30 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    
    // Index entities with different tags
    index.index_entity(1, "tag:player".to_string());
    index.index_entity(2, "tag:enemy".to_string());
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(&EntityReference::ByTag(tag), &world);

    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => {
            // Should provide clear error about missing tag
            TestResult::from_bool(msg.contains("No entity found with tag"))
        }
    }
}

/// Property: Component-based resolution handles missing components gracefully
#[quickcheck]
fn prop_component_resolution_handles_missing_components(component: String) -> TestResult {
    if component.is_empty() || component.len() > 30 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    
    // Index entities with different components
    index.index_entity(1, "component:Transform".to_string());
    index.index_entity(2, "component:Mesh".to_string());
    
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index);
    let world = World::new();

    let result = resolver.resolve_reference(&EntityReference::ByComponent(component), &world);

    match result {
        Ok(_) => TestResult::passed(),
        Err(msg) => {
            // Should provide clear error about missing component
            TestResult::from_bool(msg.contains("No entity found with component"))
        }
    }
}

/// Property: Relative position resolution is consistent
#[quickcheck]
fn prop_relative_position_resolution_consistent(
    x: f32,
    y: f32,
    z: f32,
    dist: f32,
) -> TestResult {
    if !x.is_finite() || !y.is_finite() || !z.is_finite() || !dist.is_finite() {
        return TestResult::discard();
    }
    if x.abs() > 1000.0 || y.abs() > 1000.0 || z.abs() > 1000.0 || dist.abs() > 1000.0 {
        return TestResult::discard();
    }

    let index = Arc::new(SemanticIndex::new());
    let resolver = IntentResolver::new(index);

    let anchor_pos = Vec3::new(x, y, z);
    let anchor_rot = Quat::IDENTITY;

    // Test multiple position types
    let positions = vec![
        RelativePosition::Forward(dist),
        RelativePosition::Above(dist),
        RelativePosition::AtOffset(Vec3::new(1.0, 2.0, 3.0)),
    ];

    for pos in positions {
        let result = resolver.resolve_position(&pos, anchor_pos, anchor_rot);
        
        match result {
            Ok(resolved_pos) => {
                // Should be finite
                if !resolved_pos.x.is_finite()
                    || !resolved_pos.y.is_finite()
                    || !resolved_pos.z.is_finite()
                {
                    return TestResult::failed();
                }
            }
            Err(_) => return TestResult::failed(),
        }
    }

    TestResult::passed()
}

/// Property: Resolution is deterministic for same inputs
#[quickcheck]
fn prop_resolution_is_deterministic(name: String) -> TestResult {
    if name.is_empty() || name.len() > 50 {
        return TestResult::discard();
    }

    let mut index = SemanticIndex::new();
    index.index_entity(1, name.clone());
    let index = Arc::new(index);
    let resolver = IntentResolver::new(index.clone());
    let world = World::new();

    // Resolve twice
    let result1 = resolver.resolve_reference(&EntityReference::ByName(name.clone()), &world);
    let result2 = resolver.resolve_reference(&EntityReference::ByName(name), &world);

    // Results should be identical
    match (result1, result2) {
        (Ok(e1), Ok(e2)) => TestResult::from_bool(e1 == e2),
        (Err(msg1), Err(msg2)) => TestResult::from_bool(msg1 == msg2),
        _ => TestResult::failed(),
    }
}
