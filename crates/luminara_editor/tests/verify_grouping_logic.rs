//! Verification test for grouping logic
//! 
//! This test verifies the core grouping logic works correctly
//! without requiring the full GPUI dependency chain.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchPrefix {
    Entity,
    Asset,
    Command,
    Symbol,
}

impl SearchPrefix {
    pub fn all_categories() -> Vec<Self> {
        vec![Self::Entity, Self::Asset, Self::Command, Self::Symbol]
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grouping_correctness() {
        let mut grouped = GroupedResults::new();
        
        // Add results from different categories
        grouped.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            Some("Main character".to_string()),
        ));
        grouped.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        grouped.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        grouped.add_result(SearchResult::new(
            SearchPrefix::Command,
            "Save".to_string(),
            None,
        ));

        // Verify total count
        assert_eq!(grouped.total_count(), 4);

        // Verify each category has correct results
        assert_eq!(grouped.get_category(SearchPrefix::Entity).len(), 2);
        assert_eq!(grouped.get_category(SearchPrefix::Asset).len(), 1);
        assert_eq!(grouped.get_category(SearchPrefix::Command).len(), 1);
        assert_eq!(grouped.get_category(SearchPrefix::Symbol).len(), 0);

        // Verify results are in correct categories
        let entities = grouped.get_category(SearchPrefix::Entity);
        assert_eq!(entities[0].name, "Player");
        assert_eq!(entities[1].name, "Enemy");
    }

    #[test]
    fn test_category_order_consistency() {
        let mut grouped = GroupedResults::new();
        
        // Add in non-standard order
        grouped.add_result(SearchResult::new(SearchPrefix::Symbol, "symbol".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Entity, "entity".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Command, "command".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Asset, "asset".to_string(), None));

        let categories = grouped.categories();
        
        // Should return in standard order: Entity, Asset, Command, Symbol
        assert_eq!(categories.len(), 4);
        assert_eq!(categories[0], SearchPrefix::Entity);
        assert_eq!(categories[1], SearchPrefix::Asset);
        assert_eq!(categories[2], SearchPrefix::Command);
        assert_eq!(categories[3], SearchPrefix::Symbol);
    }

    #[test]
    fn test_empty_categories_not_displayed() {
        let mut grouped = GroupedResults::new();
        
        // Only add Entity and Asset results
        grouped.add_result(SearchResult::new(SearchPrefix::Entity, "entity".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Asset, "asset".to_string(), None));

        let categories = grouped.categories();
        
        // Should only return non-empty categories
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0], SearchPrefix::Entity);
        assert_eq!(categories[1], SearchPrefix::Asset);
        
        // Empty categories should return empty slices
        assert!(grouped.get_category(SearchPrefix::Command).is_empty());
        assert!(grouped.get_category(SearchPrefix::Symbol).is_empty());
    }

    #[test]
    fn test_category_isolation() {
        let mut grouped = GroupedResults::new();
        
        // Add Entity results
        grouped.add_result(SearchResult::new(SearchPrefix::Entity, "entity1".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Entity, "entity2".to_string(), None));
        
        let entity_count = grouped.get_category(SearchPrefix::Entity).len();
        assert_eq!(entity_count, 2);
        
        // Add Asset results
        grouped.add_result(SearchResult::new(SearchPrefix::Asset, "asset1".to_string(), None));
        
        // Entity count should not change
        assert_eq!(grouped.get_category(SearchPrefix::Entity).len(), entity_count);
        
        // Asset count should be 1
        assert_eq!(grouped.get_category(SearchPrefix::Asset).len(), 1);
    }

    #[test]
    fn test_clear_operation() {
        let mut grouped = GroupedResults::new();
        
        // Add results
        grouped.add_result(SearchResult::new(SearchPrefix::Entity, "entity".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Asset, "asset".to_string(), None));
        
        assert_eq!(grouped.total_count(), 2);
        
        // Clear
        grouped.clear();
        
        // Verify everything is cleared
        assert_eq!(grouped.total_count(), 0);
        assert!(grouped.categories().is_empty());
        assert!(grouped.get_category(SearchPrefix::Entity).is_empty());
        assert!(grouped.get_category(SearchPrefix::Asset).is_empty());
    }

    #[test]
    fn test_data_preservation() {
        let mut grouped = GroupedResults::new();
        
        let original = SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            Some("Main character".to_string()),
        );
        
        grouped.add_result(original.clone());
        
        let results = grouped.get_category(SearchPrefix::Entity);
        assert_eq!(results.len(), 1);
        
        // Verify all fields are preserved
        assert_eq!(results[0].name, original.name);
        assert_eq!(results[0].description, original.description);
        assert_eq!(results[0].category, original.category);
    }

    #[test]
    fn test_duplicate_results_preserved() {
        let mut grouped = GroupedResults::new();
        
        let result = SearchResult::new(SearchPrefix::Entity, "Player".to_string(), None);
        
        // Add same result 3 times
        grouped.add_result(result.clone());
        grouped.add_result(result.clone());
        grouped.add_result(result.clone());
        
        // Should have 3 entries
        assert_eq!(grouped.get_category(SearchPrefix::Entity).len(), 3);
        assert_eq!(grouped.total_count(), 3);
    }

    #[test]
    fn test_total_count_consistency() {
        let mut grouped = GroupedResults::new();
        
        grouped.add_result(SearchResult::new(SearchPrefix::Entity, "e1".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Entity, "e2".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Asset, "a1".to_string(), None));
        grouped.add_result(SearchResult::new(SearchPrefix::Command, "c1".to_string(), None));
        
        // Total should equal sum of all categories
        let sum: usize = SearchPrefix::all_categories()
            .iter()
            .map(|c| grouped.get_category(*c).len())
            .sum();
        
        assert_eq!(grouped.total_count(), sum);
        assert_eq!(grouped.total_count(), 4);
    }
}
