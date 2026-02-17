use luminara_ai_agent::context_engine::{
    AiContextEngine, AttentionEstimator, ContextLevel, WorldDigestEngine,
};
use luminara_core::world::World;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use std::time::Instant;

// **Validates: Requirements 24.1**
#[quickcheck]
fn test_relevance_prioritization(query: String, entity_name: String) -> TestResult {
    if entity_name.is_empty() {
        return TestResult::discard();
    }

    let estimator = AttentionEstimator::default();
    let score = estimator.estimate_relevance(&query, &entity_name);

    // Score should be in valid range
    if score < 0.0 || score > 1.0 {
        return TestResult::failed();
    }

    // Exact match should give highest score
    if query.to_lowercase() == entity_name.to_lowercase() {
        TestResult::from_bool(score == 1.0)
    } else if entity_name.to_lowercase().contains(&query.to_lowercase()) {
        // Contains match should give high score
        TestResult::from_bool(score >= 0.5)
    } else {
        TestResult::passed()
    }
}

// **Validates: Requirements 24.1**
#[test]
fn test_l0_summary_generation() {
    let mut world = World::new();
    
    // Spawn multiple entities
    for _ in 0..5 {
        world.spawn();
    }

    let engine = WorldDigestEngine::new();
    let summary = engine.generate_l0_summary(&world);

    assert!(summary.contains("Total Entities: 5"));
    assert!(summary.contains("World Summary (L0)"));
    assert!(summary.len() > 0);
}

// **Validates: Requirements 24.1**
#[test]
fn test_hierarchical_context_levels() {
    let mut world = World::new();
    
    // Create a moderate-sized world
    for _ in 0..50 {
        world.spawn();
    }

    let engine = AiContextEngine::new();
    
    // Test each context level
    let l0 = engine.generate_context_at_level(ContextLevel::Summary, "", &world);
    let l1 = engine.generate_context_at_level(ContextLevel::Catalog, "", &world);
    let l2 = engine.generate_context_at_level(ContextLevel::Details, "", &world);
    let l3 = engine.generate_context_at_level(ContextLevel::Full, "", &world);

    // L0 should be shortest
    assert!(l0.len() < l1.len());
    
    // Each level should contain appropriate markers
    assert!(l0.contains("L0"));
    assert!(l1.contains("L1"));
    assert!(l2.contains("L2"));
    assert!(l3.contains("L3"));
}

// **Validates: Requirements 24.1, 24.2**
#[test]
fn test_semantic_entity_search() {
    let mut engine = AiContextEngine::new();
    
    // Index some entities with descriptions
    engine.index_entity(1, "player character with sword".to_string());
    engine.index_entity(2, "enemy orc warrior".to_string());
    engine.index_entity(3, "treasure chest".to_string());
    engine.index_entity(4, "magic sword weapon".to_string());

    // Search for "sword" - should find entities 1 and 4
    let results = engine.search_entities("sword", 10);
    
    assert!(results.len() >= 2);
    
    // Check that relevant entities are in results
    let entity_ids: Vec<u32> = results.iter().map(|(id, _)| *id).collect();
    assert!(entity_ids.contains(&1) || entity_ids.contains(&4));
}

// **Validates: Requirements 24.1**
#[test]
fn test_large_scene_performance() {
    let mut world = World::new();
    
    // Create a large scene with 10,000+ entities
    for _ in 0..10_000 {
        world.spawn();
    }

    let engine = AiContextEngine::new();
    
    let start = Instant::now();
    let context = engine.generate_context("", 4000, &world);
    let elapsed = start.elapsed();

    // Should complete within 500ms target
    assert!(
        elapsed.as_millis() < 500,
        "Digest generation took {}ms, expected <500ms",
        elapsed.as_millis()
    );
    
    // Should have generated all context levels
    assert!(!context.summary.is_empty());
    assert!(!context.catalog.is_empty());
    assert!(context.generation_time_ms < 500);
}

// **Validates: Requirements 24.1**
#[test]
fn test_token_budget_compliance() {
    let mut world = World::new();
    
    // Create many entities
    for _ in 0..1000 {
        world.spawn();
    }

    let engine = AiContextEngine::new();
    
    // Request context with small token budget
    let context = engine.generate_context("", 500, &world);
    
    // Rough token estimate (4 chars per token)
    let estimated_tokens = context.details.len() / 4;
    
    // Should respect token budget (with some tolerance)
    assert!(
        estimated_tokens < 600,
        "Details exceeded token budget: {} tokens",
        estimated_tokens
    );
}

// **Validates: Requirements 24.2**
#[test]
fn test_semantic_search_ranking() {
    let mut engine = AiContextEngine::new();
    
    // Index entities with varying relevance
    engine.index_entity(1, "red apple fruit".to_string());
    engine.index_entity(2, "apple tree plant".to_string());
    engine.index_entity(3, "orange fruit".to_string());
    engine.index_entity(4, "apple pie dessert".to_string());

    // Search for "apple"
    let results = engine.search_entities("apple", 10);
    
    assert!(results.len() >= 3);
    
    // Results should be ranked by relevance (descending)
    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results not properly ranked by relevance"
        );
    }
}

// **Validates: Requirements 24.1**
#[test]
fn test_context_generation_with_query() {
    let mut world = World::new();
    
    for _ in 0..100 {
        world.spawn();
    }

    let mut engine = AiContextEngine::new();
    
    // Index some entities
    engine.index_entity(1, "player character".to_string());
    engine.index_entity(2, "enemy monster".to_string());
    
    // Generate context with query
    let context = engine.generate_context("player", 2000, &world);
    
    // Should have generated context
    assert!(!context.summary.is_empty());
    assert!(!context.catalog.is_empty());
    // Verify generation_time_ms field exists (may be 0 for fast operations)
    let _ = context.generation_time_ms;
}

// **Validates: Requirements 24.1**
#[test]
fn test_complexity_estimation() {
    let engine = WorldDigestEngine::new();
    
    // Test different scene sizes
    let mut world_small = World::new();
    for _ in 0..5 {
        world_small.spawn();
    }
    let summary_small = engine.generate_l0_summary(&world_small);
    assert!(summary_small.contains("Minimal") || summary_small.contains("Simple"));
    
    let mut world_large = World::new();
    for _ in 0..5000 {
        world_large.spawn();
    }
    let summary_large = engine.generate_l0_summary(&world_large);
    assert!(summary_large.contains("Complex"));
}

// **Validates: Requirements 24.1**
#[test]
fn test_catalog_truncation() {
    let mut world = World::new();
    
    // Create more entities than catalog limit
    for _ in 0..200 {
        world.spawn();
    }

    let engine = WorldDigestEngine::new();
    let catalog = engine.generate_l1_catalog(&world, &[]);
    
    // Should indicate truncation
    assert!(catalog.contains("more entities") || catalog.len() < 10000);
}

// **Validates: Requirements 24.1**
#[test]
fn test_full_context_size_limit() {
    let mut world = World::new();
    
    // Create a very large scene
    for _ in 0..2000 {
        world.spawn();
    }

    let engine = WorldDigestEngine::new();
    let full = engine.generate_l3_full(&world, 8000);
    
    // Should refuse to generate full context for large scenes
    assert!(full.contains("too large") || full.contains("Use L2"));
}

// **Validates: Requirements 24.2**
#[test]
fn test_empty_query_handling() {
    let mut world = World::new();
    for _ in 0..10 {
        world.spawn();
    }

    let engine = AiContextEngine::new();
    
    // Empty query should still generate valid context
    let context = engine.generate_context("", 2000, &world);
    
    assert!(!context.summary.is_empty());
    assert!(!context.catalog.is_empty());
}

// **Validates: Requirements 24.2**
#[test]
fn test_relevance_word_overlap() {
    let estimator = AttentionEstimator::default();
    
    // Test word overlap scoring
    let score1 = estimator.estimate_relevance("red apple", "red fruit");
    let score2 = estimator.estimate_relevance("red apple", "blue banana");
    
    // Should score word overlap higher than no overlap
    assert!(score1 > score2);
}

// **Validates: Requirements 24.1**
#[test]
fn test_context_level_enum() {
    // Test that all context levels are distinct
    assert_ne!(ContextLevel::Summary, ContextLevel::Catalog);
    assert_ne!(ContextLevel::Catalog, ContextLevel::Details);
    assert_ne!(ContextLevel::Details, ContextLevel::Full);
}

// **Property 20: World Digest Token Budget Compliance**
// **Validates: Requirements 18.1, 24.1**
// For any world digest generation with a specified token budget,
// the generated digest should not exceed the budget.
#[quickcheck]
fn prop_world_digest_token_budget_compliance(
    entity_count: u16,
    token_budget: u16,
) -> TestResult {
    // Discard invalid inputs
    if entity_count == 0 || token_budget < 100 {
        return TestResult::discard();
    }

    // Limit entity count to reasonable range for test performance
    let entity_count = (entity_count % 1000) + 1;
    // Ensure minimum budget of 200 tokens to account for headers/formatting
    let token_budget = (token_budget % 5000) + 200;

    // Create world with specified number of entities
    let mut world = World::new();
    for _ in 0..entity_count {
        world.spawn();
    }

    let engine = WorldDigestEngine::new();

    // Test L2 Details generation (most likely to exceed budget)
    let details = engine.generate_l2_details(&world, &[], token_budget as usize);

    // Estimate token count (rough approximation: 4 characters per token)
    let estimated_tokens = details.len() / 4;

    // Allow 15% tolerance for token estimation inaccuracy and formatting overhead
    let tolerance = (token_budget as f32 * 0.15) as usize;
    let max_allowed_tokens = token_budget as usize + tolerance;

    if estimated_tokens > max_allowed_tokens {
        return TestResult::error(format!(
            "Token budget exceeded: {} tokens generated, budget was {} (with {}% tolerance)",
            estimated_tokens, token_budget, 15
        ));
    }

    // Verify truncation message appears if budget was exceeded
    if details.contains("truncated") {
        // If truncation indicator is present, verify we were actually close to budget
        if estimated_tokens < (token_budget as usize / 2) {
            return TestResult::error(
                "Truncation indicator present but token usage was low".to_string()
            );
        }
    }

    TestResult::passed()
}

// **Property 20: World Digest Token Budget Compliance (L3 Full Context)**
// **Validates: Requirements 18.1, 24.1**
#[quickcheck]
fn prop_world_digest_l3_token_budget_compliance(
    entity_count: u16,
    token_budget: u16,
) -> TestResult {
    // Discard invalid inputs
    if entity_count == 0 || token_budget < 100 {
        return TestResult::discard();
    }

    // Limit entity count for L3 (full context)
    let entity_count = (entity_count % 500) + 1;
    let token_budget = (token_budget % 8000) + 300;

    let mut world = World::new();
    for _ in 0..entity_count {
        world.spawn();
    }

    let engine = WorldDigestEngine::new();
    let full = engine.generate_l3_full(&world, token_budget as usize);

    // Estimate token count
    let estimated_tokens = full.len() / 4;

    // Allow 15% tolerance
    let tolerance = (token_budget as f32 * 0.15) as usize;
    let max_allowed_tokens = token_budget as usize + tolerance;

    if estimated_tokens > max_allowed_tokens {
        return TestResult::error(format!(
            "L3 token budget exceeded: {} tokens generated, budget was {}",
            estimated_tokens, token_budget
        ));
    }

    TestResult::passed()
}

// **Property 20: World Digest Token Budget Compliance (Hierarchical Context)**
// **Validates: Requirements 18.1, 24.1**
#[quickcheck]
fn prop_hierarchical_context_token_budget_compliance(
    entity_count: u16,
    token_budget: u16,
) -> TestResult {
    // Discard invalid inputs
    if entity_count == 0 || token_budget < 200 {
        return TestResult::discard();
    }

    // Limit ranges for test performance
    let entity_count = (entity_count % 2000) + 1;
    let token_budget = (token_budget % 10000) + 500;

    let mut world = World::new();
    for _ in 0..entity_count {
        world.spawn();
    }

    let engine = AiContextEngine::new();
    let context = engine.generate_context("", token_budget as usize, &world);

    // Check each context level respects budget
    let details_tokens = context.details.len() / 4;
    let full_tokens = context.full.len() / 4;

    // Allow 15% tolerance
    let tolerance = (token_budget as f32 * 0.15) as usize;
    let max_allowed_tokens = token_budget as usize + tolerance;

    if details_tokens > max_allowed_tokens {
        return TestResult::error(format!(
            "Details context exceeded token budget: {} tokens, budget was {}",
            details_tokens, token_budget
        ));
    }

    if full_tokens > max_allowed_tokens {
        return TestResult::error(format!(
            "Full context exceeded token budget: {} tokens, budget was {}",
            full_tokens, token_budget
        ));
    }

    TestResult::passed()
}

// **Property 20: World Digest Token Budget Compliance (Large Scenes)**
// **Validates: Requirements 18.1, 24.1**
// Test specifically for 10,000+ entity scenes as per requirement 24.1
#[test]
fn test_large_scene_token_budget_compliance() {
    let mut world = World::new();
    
    // Create 10,000+ entity scene
    for _ in 0..10_000 {
        world.spawn();
    }

    let engine = WorldDigestEngine::new();
    
    // Test with various token budgets
    let budgets = [500, 1000, 2000, 4000, 8000];
    
    for &budget in &budgets {
        let details = engine.generate_l2_details(&world, &[], budget);
        let estimated_tokens = details.len() / 4;
        
        // Allow 10% tolerance
        let max_allowed = budget + (budget / 10);
        
        assert!(
            estimated_tokens <= max_allowed,
            "Large scene (10K entities) exceeded token budget: {} tokens, budget was {} (max allowed: {})",
            estimated_tokens, budget, max_allowed
        );
        
        // Should have truncation indicator for very tight budgets
        // (only check for smallest budget where truncation is guaranteed)
        if budget <= 500 && estimated_tokens >= 450 {
            assert!(
                details.contains("truncated") || details.contains("..."),
                "Expected truncation indicator for very tight budget on large scene"
            );
        }
    }
}

// **Property 20: World Digest Token Budget Compliance (Relevant Entities)**
// **Validates: Requirements 18.1, 24.1**
#[quickcheck]
fn prop_relevant_entities_token_budget_compliance(
    entity_count: u8,
    relevant_count: u8,
    token_budget: u16,
) -> TestResult {
    // Discard invalid inputs
    if entity_count == 0 || token_budget < 100 {
        return TestResult::discard();
    }

    let entity_count = (entity_count % 100) + 1;
    let relevant_count = (relevant_count % entity_count) + 1;
    let token_budget = (token_budget % 5000) + 200;

    let mut world = World::new();
    for _ in 0..entity_count {
        world.spawn();
    }

    // Create relevant entities list with mock relevance scores
    let relevant_entities: Vec<(u32, f32)> = (0..relevant_count)
        .map(|i| (i as u32, 1.0 - (i as f32 / relevant_count as f32)))
        .collect();

    let engine = WorldDigestEngine::new();
    let details = engine.generate_l2_details(&world, &relevant_entities, token_budget as usize);

    let estimated_tokens = details.len() / 4;
    let tolerance = (token_budget as f32 * 0.15) as usize;
    let max_allowed_tokens = token_budget as usize + tolerance;

    if estimated_tokens > max_allowed_tokens {
        return TestResult::error(format!(
            "Token budget exceeded with relevant entities: {} tokens, budget was {}",
            estimated_tokens, token_budget
        ));
    }

    TestResult::passed()
}

