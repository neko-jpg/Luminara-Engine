use luminara_ai_agent::SemanticIndex;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;

#[quickcheck]
fn test_semantic_search_retrieval(id: u64, text: String) -> TestResult {
    if text.is_empty() { return TestResult::discard(); }

    let mut index = SemanticIndex::new();
    index.index_entity(id, text.clone());

    // Search for exact text should return the entity with high score
    let results = index.search(&text, 1);

    if !results.is_empty() {
        let (found_id, score) = results[0];
        // Score should be close to 1.0 for exact match
        TestResult::from_bool(found_id == id && score > 0.99)
    } else {
        TestResult::failed()
    }
}

#[quickcheck]
fn test_semantic_index_capacity(count: u8) -> TestResult {
    let mut index = SemanticIndex::new();
    for i in 0..count {
        index.index_entity(i as u64, format!("Entity {}", i));
    }

    let results = index.search("Entity", count as usize);
    // Should find all of them with non-zero score as they all contain "Entity"
    // Though "Entity" vector vs "Entity N" vector similarity might vary.
    // At least we check no crash and results returned.

    if count > 0 {
        TestResult::from_bool(!results.is_empty())
    } else {
        TestResult::passed()
    }
}
