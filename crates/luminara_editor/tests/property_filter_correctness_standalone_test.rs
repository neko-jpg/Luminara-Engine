//! Standalone property-based test for Filter Correctness
//!
//! This is a minimal version that doesn't depend on GPUI/wgpu to work around
//! Windows version conflicts in wgpu-hal.
//!
//! **Validates: Requirements 3.3**
//!
//! **Property 5: Filter Correctness**

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

    pub fn should_include_result(&self, result_type: SearchPrefix) -> bool {
        match self {
            SearchPrefix::None => true,
            _ => *self == result_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchResult {
    name: String,
    result_type: SearchPrefix,
}

fn filter_results(prefix: SearchPrefix, results: &[SearchResult]) -> Vec<SearchResult> {
    results
        .iter()
        .filter(|r| prefix.should_include_result(r.result_type))
        .cloned()
        .collect()
}

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

impl Arbitrary for SearchResult {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        ("[a-z]{1,10}", any::<SearchPrefix>())
            .prop_map(|(name, result_type)| SearchResult { name, result_type })
            .boxed()
    }
}

/// **Validates: Requirements 3.3**
#[test]
fn property_filter_inclusion_correctness() {
    proptest!(|(
        prefix in any::<SearchPrefix>(),
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let filtered = filter_results(prefix, &results);

        if prefix == SearchPrefix::None {
            prop_assert_eq!(filtered.len(), results.len());
        }

        if prefix != SearchPrefix::None {
            for result in &filtered {
                prop_assert_eq!(result.result_type, prefix);
            }
        }

        let expected_count = results.iter()
            .filter(|r| prefix.should_include_result(r.result_type))
            .count();
        prop_assert_eq!(filtered.len(), expected_count);
    });
}

/// **Validates: Requirements 3.3**
#[test]
fn property_filter_exclusion_correctness() {
    proptest!(|(
        results in prop::collection::vec(any::<SearchResult>(), 0..50),
    )| {
        let entity_filtered = filter_results(SearchPrefix::Entity, &results);
        for result in &entity_filtered {
            prop_assert_eq!(result.result_type, SearchPrefix::Entity);
        }

        let asset_filtered = filter_results(SearchPrefix::Asset, &results);
        for result in &asset_filtered {
            prop_assert_eq!(result.result_type, SearchPrefix::Asset);
        }
    });
}
