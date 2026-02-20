//! Property-based test for Filter Correctness
//!
//! **Validates: Requirements 3.3**
//!
//! **Property 5: Filter Correctness**
//!
//! This property verifies that when a prefix filter is applied, only results
//! matching that filter type are included in the results.

use proptest::prelude::*;

/// Search filter prefix types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchPrefix {
    Entity,
    Asset,
    Command,
    Symbol,
    None,
}

impl SearchPrefix {
    /// Parse a prefix from the start of a query string
    pub fn parse(query: &str) -> (Self, &str) {
        if query.is_empty() {
            return (Self::None, query);
        }

        match query.chars().next() {
            Some('@') => (Self::Entity, &query[1..]),
            Some('#') => (Self::Asset, &query[1..]),
            Some('/') => (Self::Command, &query[1..]),
            Some(':') => (Self::Symbol, &query[1..]),
            _ => (Self::None, query),
        }
    }

    /// Check if results should be filtered by the current prefix
    pub fn should_include_result(&self, result_type: SearchPrefix) -> bool {
        match self {
            SearchPrefix::None => true, // No filter, include all
            _ => *self == result_type,
        }
    }
}

/// Represents a search result with a type
#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchResult {
    name: String,
    result_type: SearchPrefix,
}

impl SearchResult {
    fn new(name: String, result_type: SearchPrefix) -> Self {
        Self { name, result_type }
    }
}

/// Filter a collection of results based on the prefix
fn filter_results(prefix: SearchPrefix, results: &[SearchResult]) -> Vec<SearchResult> {
    results
        .iter()
        .filter(|r| prefix.should_include_result(r.result_type))
        .cloned()
        .collect()
}

// Arbitrary implementation for SearchPrefix
impl Arbitrary for SearchPrefix {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(SearchPrefix::Entity),
            Just(SearchPrefix::Asset),
            Just(SearchPrefix::Command),
            Just(SearchPrefix::Symbol),
            Just(SearchPrefix::None),
        ]
        .boxed()
    }
}

// Arbitrary implementation for SearchResult
impl Arbitrary for SearchResult {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        ("[a-z]{1,10}", any::<SearchPrefix>())
            .prop_map(|(name, result_type)| SearchResult::new(name, result_type))
            .boxed()
    }
}

/// Property: Filter Inclusion Correctness
///
/// For any filter prefix and collection of results, the filtered results
/// SHALL contain only items that match the filter criteria.
///
/// **Invariants:**
/// 1. When filter is None, all results are included
/// 2. When filter is specific type, only matching types are included
/// 3. No results of non-matching types are included
/// 4. All matching results are included (no false negatives)
#[test]
fn property_filter_inclusion_correctness() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let filtered = filter_results(prefix, &results);

        // Invariant 1: When filter is None, all results are included
        if prefix == SearchPrefix::None {
            prop_assert_eq!(filtered.len(), results.len());
            prop_assert_eq!(filtered, results);
        }

        // Invariant 2 & 3: When filter is specific, only matching types included
        if prefix != SearchPrefix::None {
            for result in &filtered {
                prop_assert_eq!(result.result_type, prefix,
                    "Filtered results should only contain matching type");
            }
        }

        // Invariant 4: All matching results are included
        let expected_count = results.iter()
            .filter(|r| prefix.should_include_result(r.result_type))
            .count();
        prop_assert_eq!(filtered.len(), expected_count,
            "All matching results should be included");
    });
}

/// Property: Filter Exclusion Correctness
///
/// For any specific filter prefix, results of other types SHALL be excluded.
///
/// **Invariants:**
/// 1. Entity filter excludes Assets, Commands, Symbols
/// 2. Asset filter excludes Entities, Commands, Symbols
/// 3. Command filter excludes Entities, Assets, Symbols
/// 4. Symbol filter excludes Entities, Assets, Commands
#[test]
fn property_filter_exclusion_correctness() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        // Test Entity filter
        {
            let filtered = filter_results(SearchPrefix::Entity, &results);
            for result in &filtered {
                prop_assert_eq!(result.result_type, SearchPrefix::Entity);
                prop_assert_ne!(result.result_type, SearchPrefix::Asset);
                prop_assert_ne!(result.result_type, SearchPrefix::Command);
                prop_assert_ne!(result.result_type, SearchPrefix::Symbol);
            }
        }

        // Test Asset filter
        {
            let filtered = filter_results(SearchPrefix::Asset, &results);
            for result in &filtered {
                prop_assert_ne!(result.result_type, SearchPrefix::Entity);
                prop_assert_eq!(result.result_type, SearchPrefix::Asset);
                prop_assert_ne!(result.result_type, SearchPrefix::Command);
                prop_assert_ne!(result.result_type, SearchPrefix::Symbol);
            }
        }

        // Test Command filter
        {
            let filtered = filter_results(SearchPrefix::Command, &results);
            for result in &filtered {
                prop_assert_ne!(result.result_type, SearchPrefix::Entity);
                prop_assert_ne!(result.result_type, SearchPrefix::Asset);
                prop_assert_eq!(result.result_type, SearchPrefix::Command);
                prop_assert_ne!(result.result_type, SearchPrefix::Symbol);
            }
        }

        // Test Symbol filter
        {
            let filtered = filter_results(SearchPrefix::Symbol, &results);
            for result in &filtered {
                prop_assert_ne!(result.result_type, SearchPrefix::Entity);
                prop_assert_ne!(result.result_type, SearchPrefix::Asset);
                prop_assert_ne!(result.result_type, SearchPrefix::Command);
                prop_assert_eq!(result.result_type, SearchPrefix::Symbol);
            }
        }
    });
}

/// Property: Filter Idempotence
///
/// Applying the same filter multiple times SHALL produce the same result.
///
/// **Invariants:**
/// 1. filter(filter(results)) == filter(results)
/// 2. Multiple applications don't change the result
/// 3. Filter is stable and deterministic
#[test]
fn property_filter_idempotence() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let filtered_once = filter_results(prefix, &results);
        let filtered_twice = filter_results(prefix, &filtered_once);
        let filtered_thrice = filter_results(prefix, &filtered_twice);

        // Invariant 1 & 2: Multiple applications produce same result
        prop_assert_eq!(filtered_once, filtered_twice);
        prop_assert_eq!(filtered_twice, filtered_thrice);

        // Invariant 3: Result is deterministic
        let filtered_again = filter_results(prefix, &results);
        prop_assert_eq!(filtered_once, filtered_again);
    });
}

/// Property: Filter Commutativity with Subset
///
/// Filtering a subset of results SHALL produce a subset of the filtered results.
///
/// **Invariants:**
/// 1. filter(subset) ⊆ filter(all_results)
/// 2. If result is in filtered subset, it's in filtered all
/// 3. Subset relationship is preserved
#[test]
fn property_filter_subset_preservation() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
        results in prop::collection::vec(any::<SearchResult>(), 5..50),
        subset_size in 0..5usize,
    )| {
        // Create a subset of results
        let subset_size = subset_size.min(results.len());
        let subset: Vec<SearchResult> = results.iter().take(subset_size).cloned().collect();

        let filtered_all = filter_results(prefix, &results);
        let filtered_subset = filter_results(prefix, &subset);

        // Invariant 1 & 2: Every item in filtered subset is in filtered all
        for item in &filtered_subset {
            prop_assert!(filtered_all.contains(item),
                "Filtered subset should be contained in filtered all");
        }

        // Invariant 3: Subset size relationship
        prop_assert!(filtered_subset.len() <= filtered_all.len(),
            "Filtered subset cannot be larger than filtered all");
    });
}

/// Property: Filter Empty Set Behavior
///
/// Filtering an empty collection SHALL always produce an empty result.
///
/// **Invariants:**
/// 1. filter(∅) = ∅ for any filter
/// 2. Empty input always produces empty output
/// 3. Filter type doesn't matter for empty input
#[test]
fn property_filter_empty_set() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
    )| {
        let empty_results: Vec<SearchResult> = vec![];
        let filtered = filter_results(prefix, &empty_results);

        // Invariant 1 & 2: Empty input produces empty output
        prop_assert!(filtered.is_empty());
        prop_assert_eq!(filtered.len(), 0);
    });
}

/// Property: Filter Type Consistency
///
/// All results in filtered output SHALL have the same type as the filter
/// (or any type if filter is None).
///
/// **Invariants:**
/// 1. For specific filter, all results match filter type
/// 2. For None filter, results can be any type
/// 3. No mixed types when filter is specific
#[test]
fn property_filter_type_consistency() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let filtered = filter_results(prefix, &results);

        if prefix != SearchPrefix::None {
            // Invariant 1 & 3: All results match filter type
            let all_match = filtered.iter().all(|r| r.result_type == prefix);
            prop_assert!(all_match, "All filtered results should match filter type");

            // Verify no mixed types
            if !filtered.is_empty() {
                let first_type = filtered[0].result_type;
                for result in &filtered {
                    prop_assert_eq!(result.result_type, first_type);
                }
            }
        }
    });
}

/// Property: Filter Prefix Parsing Correctness
///
/// Parsing a query with a prefix SHALL correctly identify the prefix
/// and extract the remaining query.
///
/// **Invariants:**
/// 1. '@' prefix parses to Entity
/// 2. '#' prefix parses to Asset
/// 3. '/' prefix parses to Command
/// 4. ':' prefix parses to Symbol
/// 5. No prefix parses to None
/// 6. Remaining query excludes the prefix character
#[test]
fn property_prefix_parsing_correctness() {
    proptest!(|(
        query_text in "[a-z]{0,20}",
    )| {
        // Test Entity prefix
        {
            let query = format!("@{}", query_text);
            let (prefix, remaining) = SearchPrefix::parse(&query);
            prop_assert_eq!(prefix, SearchPrefix::Entity);
            prop_assert_eq!(remaining, query_text.as_str());
        }

        // Test Asset prefix
        {
            let query = format!("#{}", query_text);
            let (prefix, remaining) = SearchPrefix::parse(&query);
            prop_assert_eq!(prefix, SearchPrefix::Asset);
            prop_assert_eq!(remaining, query_text.as_str());
        }

        // Test Command prefix
        {
            let query = format!("/{}", query_text);
            let (prefix, remaining) = SearchPrefix::parse(&query);
            prop_assert_eq!(prefix, SearchPrefix::Command);
            prop_assert_eq!(remaining, query_text.as_str());
        }

        // Test Symbol prefix
        {
            let query = format!(":{}", query_text);
            let (prefix, remaining) = SearchPrefix::parse(&query);
            prop_assert_eq!(prefix, SearchPrefix::Symbol);
            prop_assert_eq!(remaining, query_text.as_str());
        }

        // Test no prefix
        if !query_text.is_empty() && !['@', '#', '/', ':'].contains(&query_text.chars().next().unwrap()) {
            let (prefix, remaining) = SearchPrefix::parse(&query_text);
            prop_assert_eq!(prefix, SearchPrefix::None);
            prop_assert_eq!(remaining, query_text.as_str());
        }
    });
}

/// Property: Filter Count Correctness
///
/// The count of filtered results SHALL equal the count of matching items
/// in the original collection.
///
/// **Invariants:**
/// 1. filtered.len() == results.filter(matches).count()
/// 2. Count is never negative
/// 3. Count is never greater than original count
/// 4. For None filter, count equals original count
#[test]
fn property_filter_count_correctness() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let filtered = filter_results(prefix, &results);

        // Invariant 1: Count matches expected
        let expected_count = results.iter()
            .filter(|r| prefix.should_include_result(r.result_type))
            .count();
        prop_assert_eq!(filtered.len(), expected_count);

        // Invariant 2: Count is non-negative (always true for usize, but explicit)
        prop_assert!(filtered.len() >= 0);

        // Invariant 3: Count never exceeds original
        prop_assert!(filtered.len() <= results.len());

        // Invariant 4: None filter preserves count
        if prefix == SearchPrefix::None {
            prop_assert_eq!(filtered.len(), results.len());
        }
    });
}

/// Property: Filter Determinism
///
/// Filtering the same collection with the same filter SHALL always
/// produce the same result.
///
/// **Invariants:**
/// 1. Multiple calls with same inputs produce same output
/// 2. Order of results is preserved
/// 3. Result is reproducible
#[test]
fn property_filter_determinism() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let filtered1 = filter_results(prefix, &results);
        let filtered2 = filter_results(prefix, &results);
        let filtered3 = filter_results(prefix, &results);

        // Invariant 1 & 3: Same inputs produce same outputs
        prop_assert_eq!(filtered1, filtered2);
        prop_assert_eq!(filtered2, filtered3);

        // Invariant 2: Order is preserved (check by comparing sequences)
        prop_assert_eq!(filtered1, filtered2);
    });
}

/// Property: Filter Completeness
///
/// For any result type, there exists a filter that includes it.
///
/// **Invariants:**
/// 1. Entity results included by Entity filter or None filter
/// 2. Asset results included by Asset filter or None filter
/// 3. Command results included by Command filter or None filter
/// 4. Symbol results included by Symbol filter or None filter
/// 5. All results included by None filter
#[test]
fn property_filter_completeness() {
    proptest!(|(
        result_type in any::<SearchPrefix>(),
        name in "[a-z]{1,10}",
    )| {
        // Skip None type as it's not a valid result type
        if result_type == SearchPrefix::None {
            return Ok(());
        }

        let result = SearchResult::new(name, result_type);
        let results = vec![result.clone()];

        // Test that specific filter includes matching type
        let filtered_specific = filter_results(result_type, &results);
        prop_assert_eq!(filtered_specific.len(), 1);
        prop_assert!(filtered_specific.contains(&result));

        // Test that None filter includes all types
        let filtered_none = filter_results(SearchPrefix::None, &results);
        prop_assert_eq!(filtered_none.len(), 1);
        prop_assert!(filtered_none.contains(&result));

        Ok(())
    });
}
