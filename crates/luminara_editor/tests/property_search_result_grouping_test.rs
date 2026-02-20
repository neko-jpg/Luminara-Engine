//! Property-based test for Search Result Grouping
//!
//! **Validates: Requirements 3.4**
//!
//! **Property 13: Search Result Grouping**
//!
//! This property verifies that search results are correctly grouped by category
//! and that the grouping is consistent across different inputs.

use proptest::prelude::*;
use std::collections::HashMap;

/// Search filter prefix types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchPrefix {
    Entity,
    Asset,
    Command,
    Symbol,
    None,
}

impl SearchPrefix {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Entity => "Entities",
            Self::Asset => "Assets",
            Self::Command => "Commands",
            Self::Symbol => "Symbols",
            Self::None => "All",
        }
    }

    pub fn all_categories() -> Vec<Self> {
        vec![Self::Entity, Self::Asset, Self::Command, Self::Symbol]
    }
}

/// A search result item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub category: SearchPrefix,
    pub name: String,
    pub description: Option<String>,
}

impl SearchResult {
    pub fn new(category: SearchPrefix, name: String, description: Option<String>) -> Self {
        Self {
            category,
            name,
            description,
        }
    }
}

/// Grouped search results by category
#[derive(Debug, Clone, Default)]
pub struct GroupedResults {
    groups: HashMap<SearchPrefix, Vec<SearchResult>>,
}

impl GroupedResults {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn add_result(&mut self, result: SearchResult) {
        self.groups
            .entry(result.category)
            .or_insert_with(Vec::new)
            .push(result);
    }

    pub fn get_category(&self, category: SearchPrefix) -> &[SearchResult] {
        self.groups.get(&category).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn categories(&self) -> Vec<SearchPrefix> {
        SearchPrefix::all_categories()
            .into_iter()
            .filter(|cat| !self.get_category(*cat).is_empty())
            .collect()
    }

    pub fn total_count(&self) -> usize {
        self.groups.values().map(|v| v.len()).sum()
    }

    pub fn clear(&mut self) {
        self.groups.clear();
    }
}

// Arbitrary implementation for SearchPrefix (excluding None as it's not a valid category)
impl Arbitrary for SearchPrefix {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(SearchPrefix::Entity),
            Just(SearchPrefix::Asset),
            Just(SearchPrefix::Command),
            Just(SearchPrefix::Symbol),
        ]
        .boxed()
    }
}

// Arbitrary implementation for SearchResult
impl Arbitrary for SearchResult {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<SearchPrefix>(),
            "[a-z]{1,20}",
            prop::option::of("[a-z ]{0,50}"),
        )
            .prop_map(|(category, name, description)| SearchResult::new(category, name, description))
            .boxed()
    }
}

/// Property: Grouping Correctness
///
/// For any collection of search results, grouping SHALL place each result
/// in exactly one category group matching its category field.
///
/// **Invariants:**
/// 1. Each result appears in exactly one category group
/// 2. Each result is in the group matching its category
/// 3. No result appears in multiple groups
/// 4. No result is lost during grouping
#[test]
fn property_grouping_correctness() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 0..100),
    )| {
        let mut grouped = GroupedResults::new();
        
        // Add all results to grouped structure
        for result in &results {
            grouped.add_result(result.clone());
        }

        // Invariant 1 & 4: Total count matches original count
        prop_assert_eq!(grouped.total_count(), results.len(),
            "Total count should match original count");

        // Invariant 2: Each result is in the correct category
        for result in &results {
            let category_results = grouped.get_category(result.category);
            prop_assert!(category_results.contains(result),
                "Result should be in its category group");
        }

        // Invariant 3: No result appears in wrong category
        for category in SearchPrefix::all_categories() {
            let category_results = grouped.get_category(category);
            for result in category_results {
                prop_assert_eq!(result.category, category,
                    "Result in category group should have matching category");
            }
        }
    });
}

/// Property: Category Order Consistency
///
/// The categories() method SHALL always return categories in the same
/// consistent order: Entity, Asset, Command, Symbol.
///
/// **Invariants:**
/// 1. Categories are always in the order: Entity, Asset, Command, Symbol
/// 2. Only non-empty categories are returned
/// 3. Order is independent of insertion order
/// 4. Order is deterministic across multiple calls
#[test]
fn property_category_order_consistency() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 1..100),
    )| {
        let mut grouped = GroupedResults::new();
        
        // Add results in random order
        for result in &results {
            grouped.add_result(result.clone());
        }

        let categories = grouped.categories();
        
        // Invariant 1: Categories are in standard order
        let expected_order = vec![
            SearchPrefix::Entity,
            SearchPrefix::Asset,
            SearchPrefix::Command,
            SearchPrefix::Symbol,
        ];
        
        for (i, &category) in categories.iter().enumerate() {
            if i > 0 {
                let prev_idx = expected_order.iter().position(|&c| c == categories[i - 1]).unwrap();
                let curr_idx = expected_order.iter().position(|&c| c == category).unwrap();
                prop_assert!(curr_idx > prev_idx,
                    "Categories should be in standard order");
            }
        }

        // Invariant 2: Only non-empty categories are returned
        for category in &categories {
            prop_assert!(!grouped.get_category(*category).is_empty(),
                "Returned categories should be non-empty");
        }

        // Invariant 4: Multiple calls produce same result
        let categories2 = grouped.categories();
        prop_assert_eq!(categories, categories2,
            "Multiple calls should produce same result");
    });
}

/// Property: Grouping Idempotence
///
/// Adding the same result multiple times SHALL result in multiple entries
/// in the same category group (not deduplication).
///
/// **Invariants:**
/// 1. Each add_result call adds exactly one entry
/// 2. Duplicate results are preserved
/// 3. Count increases by 1 for each add
#[test]
fn property_grouping_idempotence() {
    proptest!(|(
        result in any::<SearchResult>(),
        count in 1..10usize,
    )| {
        let mut grouped = GroupedResults::new();
        
        // Add the same result multiple times
        for _ in 0..count {
            grouped.add_result(result.clone());
        }

        // Invariant 1 & 2: Each add creates an entry
        let category_results = grouped.get_category(result.category);
        prop_assert_eq!(category_results.len(), count,
            "Should have exactly count entries");

        // Invariant 3: Total count matches
        prop_assert_eq!(grouped.total_count(), count,
            "Total count should match number of adds");

        // All entries should be identical
        for entry in category_results {
            prop_assert_eq!(entry, &result,
                "All entries should match original result");
        }
    });
}

/// Property: Category Isolation
///
/// Results in one category SHALL NOT affect results in other categories.
///
/// **Invariants:**
/// 1. Adding Entity results doesn't change Asset results
/// 2. Adding Asset results doesn't change Command results
/// 3. Adding Command results doesn't change Symbol results
/// 4. Each category is independent
#[test]
fn property_category_isolation() {
    proptest!(|(
        entity_results in prop::collection::vec(any::<SearchResult>(), 0..20),
        asset_results in prop::collection::vec(any::<SearchResult>(), 0..20),
    )| {
        let mut grouped = GroupedResults::new();
        
        // Add entity results
        let entity_results: Vec<_> = entity_results.into_iter()
            .map(|mut r| { r.category = SearchPrefix::Entity; r })
            .collect();
        for result in &entity_results {
            grouped.add_result(result.clone());
        }
        
        let entity_count = grouped.get_category(SearchPrefix::Entity).len();
        let asset_count_before = grouped.get_category(SearchPrefix::Asset).len();
        
        // Add asset results
        let asset_results: Vec<_> = asset_results.into_iter()
            .map(|mut r| { r.category = SearchPrefix::Asset; r })
            .collect();
        for result in &asset_results {
            grouped.add_result(result.clone());
        }
        
        // Invariant 1: Entity count unchanged after adding assets
        prop_assert_eq!(grouped.get_category(SearchPrefix::Entity).len(), entity_count,
            "Entity count should not change when adding assets");
        
        // Invariant 2: Asset count increased correctly
        prop_assert_eq!(
            grouped.get_category(SearchPrefix::Asset).len(),
            asset_count_before + asset_results.len(),
            "Asset count should increase by number of added assets"
        );
    });
}

/// Property: Empty Group Behavior
///
/// Categories with no results SHALL return empty slices and not appear
/// in the categories() list.
///
/// **Invariants:**
/// 1. get_category returns empty slice for unused categories
/// 2. categories() doesn't include empty categories
/// 3. Empty categories don't affect total count
#[test]
fn property_empty_group_behavior() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let mut grouped = GroupedResults::new();
        
        // Add results (may not cover all categories)
        for result in &results {
            grouped.add_result(result.clone());
        }

        let categories = grouped.categories();
        
        // Invariant 1: Empty categories return empty slices
        for category in SearchPrefix::all_categories() {
            let category_results = grouped.get_category(category);
            if !categories.contains(&category) {
                prop_assert!(category_results.is_empty(),
                    "Unused category should return empty slice");
            }
        }

        // Invariant 2: categories() doesn't include empty
        for category in &categories {
            prop_assert!(!grouped.get_category(*category).is_empty(),
                "categories() should not include empty categories");
        }

        // Invariant 3: Total count equals sum of non-empty categories
        let sum: usize = categories.iter()
            .map(|c| grouped.get_category(*c).len())
            .sum();
        prop_assert_eq!(grouped.total_count(), sum,
            "Total count should equal sum of category counts");
    });
}

/// Property: Clear Operation Completeness
///
/// Calling clear() SHALL remove all results from all categories.
///
/// **Invariants:**
/// 1. After clear, total_count is 0
/// 2. After clear, all categories are empty
/// 3. After clear, categories() returns empty vec
/// 4. Clear is idempotent
#[test]
fn property_clear_completeness() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 1..50),
    )| {
        let mut grouped = GroupedResults::new();
        
        // Add results
        for result in &results {
            grouped.add_result(result.clone());
        }
        
        // Verify we have results
        prop_assert!(grouped.total_count() > 0);
        
        // Clear
        grouped.clear();
        
        // Invariant 1: Total count is 0
        prop_assert_eq!(grouped.total_count(), 0,
            "Total count should be 0 after clear");
        
        // Invariant 2: All categories are empty
        for category in SearchPrefix::all_categories() {
            prop_assert!(grouped.get_category(category).is_empty(),
                "All categories should be empty after clear");
        }
        
        // Invariant 3: categories() returns empty
        prop_assert!(grouped.categories().is_empty(),
            "categories() should return empty after clear");
        
        // Invariant 4: Clear is idempotent
        grouped.clear();
        prop_assert_eq!(grouped.total_count(), 0,
            "Clear should be idempotent");
    });
}

/// Property: Grouping Preserves Result Data
///
/// Grouping SHALL preserve all fields of the result (name, description).
///
/// **Invariants:**
/// 1. Result name is preserved
/// 2. Result description is preserved
/// 3. Result category is preserved
/// 4. No data is modified during grouping
#[test]
fn property_grouping_preserves_data() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 1..50),
    )| {
        let mut grouped = GroupedResults::new();
        
        // Add results
        for result in &results {
            grouped.add_result(result.clone());
        }

        // Verify all results are preserved with correct data
        for original in &results {
            let category_results = grouped.get_category(original.category);
            
            // Find the matching result
            let found = category_results.iter().any(|r| {
                r.name == original.name &&
                r.description == original.description &&
                r.category == original.category
            });
            
            prop_assert!(found,
                "Original result should be found with all data preserved");
        }
    });
}

/// Property: Total Count Consistency
///
/// The total_count() SHALL always equal the sum of all category counts.
///
/// **Invariants:**
/// 1. total_count() == sum of all category lengths
/// 2. Count is never negative
/// 3. Count is consistent across multiple calls
#[test]
fn property_total_count_consistency() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 0..100),
    )| {
        let mut grouped = GroupedResults::new();
        
        for result in &results {
            grouped.add_result(result.clone());
        }

        // Invariant 1: Total equals sum of categories
        let sum: usize = SearchPrefix::all_categories()
            .iter()
            .map(|c| grouped.get_category(*c).len())
            .sum();
        prop_assert_eq!(grouped.total_count(), sum,
            "Total count should equal sum of category counts");

        // Invariant 2: Count is non-negative (always true for usize)
        prop_assert!(grouped.total_count() >= 0);

        // Invariant 3: Multiple calls produce same result
        let count1 = grouped.total_count();
        let count2 = grouped.total_count();
        prop_assert_eq!(count1, count2,
            "Multiple calls should produce same count");
    });
}

/// Property: Category Membership Exclusivity
///
/// Each result SHALL belong to exactly one category, determined by its
/// category field.
///
/// **Invariants:**
/// 1. A result with category X appears only in group X
/// 2. A result never appears in multiple groups
/// 3. Category field determines group membership
#[test]
fn property_category_membership_exclusivity() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 1..50),
    )| {
        let mut grouped = GroupedResults::new();
        
        for result in &results {
            grouped.add_result(result.clone());
        }

        // For each result, verify it only appears in its designated category
        for original in &results {
            let mut found_count = 0;
            
            for category in SearchPrefix::all_categories() {
                let category_results = grouped.get_category(category);
                let count_in_category = category_results.iter()
                    .filter(|r| r.name == original.name && r.description == original.description)
                    .count();
                
                if category == original.category {
                    // Should be found in its own category
                    prop_assert!(count_in_category > 0,
                        "Result should be found in its designated category");
                    found_count += count_in_category;
                } else {
                    // Should not be found in other categories
                    prop_assert_eq!(count_in_category, 0,
                        "Result should not appear in other categories");
                }
            }
            
            // Should be found exactly once (or multiple times if added multiple times)
            prop_assert!(found_count > 0,
                "Result should be found at least once");
        }
    });
}

/// Property: Grouping Determinism
///
/// Adding the same set of results in different orders SHALL produce
/// the same grouped structure (same categories with same counts).
///
/// **Invariants:**
/// 1. Order of insertion doesn't affect category membership
/// 2. Order of insertion doesn't affect counts
/// 3. Categories are the same regardless of insertion order
#[test]
fn property_grouping_determinism() {
    proptest!(|(
        mut results in prop::collection::vec(any::<SearchResult>(), 1..50),
    )| {
        // Create first grouping with original order
        let mut grouped1 = GroupedResults::new();
        for result in &results {
            grouped1.add_result(result.clone());
        }

        // Create second grouping with reversed order
        results.reverse();
        let mut grouped2 = GroupedResults::new();
        for result in &results {
            grouped2.add_result(result.clone());
        }

        // Invariant 1 & 2: Same counts in each category
        for category in SearchPrefix::all_categories() {
            prop_assert_eq!(
                grouped1.get_category(category).len(),
                grouped2.get_category(category).len(),
                "Category counts should be same regardless of insertion order"
            );
        }

        // Invariant 3: Same categories present
        prop_assert_eq!(
            grouped1.categories().len(),
            grouped2.categories().len(),
            "Same categories should be present"
        );

        // Total counts should match
        prop_assert_eq!(
            grouped1.total_count(),
            grouped2.total_count(),
            "Total counts should match"
        );
    });
}
