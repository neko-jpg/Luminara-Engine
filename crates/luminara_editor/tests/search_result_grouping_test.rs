//! Integration tests for Global Search result grouping
//!
//! Tests the result grouping and category display functionality.
//! 
//! **Validates: Requirements 3.4**

use std::collections::HashMap;

/// Search filter prefix types (copied for testing)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            Some("Main character".to_string()),
        );
        
        assert_eq!(result.category, SearchPrefix::Entity);
        assert_eq!(result.name, "Player");
        assert_eq!(result.description, Some("Main character".to_string()));
    }

    #[test]
    fn test_grouped_results_add_and_get() {
        let mut groups = GroupedResults::new();
        
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        
        let entities = groups.get_category(SearchPrefix::Entity);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].name, "Player");
        assert_eq!(entities[1].name, "Enemy");
        
        let assets = groups.get_category(SearchPrefix::Asset);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "texture.png");
        
        let commands = groups.get_category(SearchPrefix::Command);
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_grouped_results_categories() {
        let mut groups = GroupedResults::new();
        
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Symbol,
            "render_fn".to_string(),
            None,
        ));
        
        let categories = groups.categories();
        
        // Should return categories in order: Entity, Asset, Command, Symbol
        // But only those with results
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0], SearchPrefix::Entity);
        assert_eq!(categories[1], SearchPrefix::Symbol);
    }

    #[test]
    fn test_grouped_results_total_count() {
        let mut groups = GroupedResults::new();
        
        assert_eq!(groups.total_count(), 0);
        
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        
        assert_eq!(groups.total_count(), 3);
    }

    #[test]
    fn test_grouped_results_clear() {
        let mut groups = GroupedResults::new();
        
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        
        assert_eq!(groups.total_count(), 1);
        
        groups.clear();
        
        assert_eq!(groups.total_count(), 0);
        assert_eq!(groups.categories().len(), 0);
    }

    #[test]
    fn test_search_prefix_all_categories() {
        let categories = SearchPrefix::all_categories();
        
        assert_eq!(categories.len(), 4);
        assert_eq!(categories[0], SearchPrefix::Entity);
        assert_eq!(categories[1], SearchPrefix::Asset);
        assert_eq!(categories[2], SearchPrefix::Command);
        assert_eq!(categories[3], SearchPrefix::Symbol);
    }

    #[test]
    fn test_grouped_results_maintains_order() {
        let mut groups = GroupedResults::new();
        
        // Add in non-standard order
        groups.add_result(SearchResult::new(
            SearchPrefix::Symbol,
            "symbol1".to_string(),
            None,
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "entity1".to_string(),
            None,
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Command,
            "command1".to_string(),
            None,
        ));
        
        let categories = groups.categories();
        
        // Should still return in standard order
        assert_eq!(categories[0], SearchPrefix::Entity);
        assert_eq!(categories[1], SearchPrefix::Command);
        assert_eq!(categories[2], SearchPrefix::Symbol);
    }

    #[test]
    fn test_grouped_results_multiple_items_per_category() {
        let mut groups = GroupedResults::new();
        
        // Add multiple items to each category
        for i in 0..5 {
            groups.add_result(SearchResult::new(
                SearchPrefix::Entity,
                format!("Entity_{}", i),
                Some(format!("Description {}", i)),
            ));
        }
        
        for i in 0..3 {
            groups.add_result(SearchResult::new(
                SearchPrefix::Asset,
                format!("Asset_{}", i),
                None,
            ));
        }
        
        assert_eq!(groups.get_category(SearchPrefix::Entity).len(), 5);
        assert_eq!(groups.get_category(SearchPrefix::Asset).len(), 3);
        assert_eq!(groups.total_count(), 8);
    }

    #[test]
    fn test_category_display_names() {
        assert_eq!(SearchPrefix::Entity.display_name(), "Entities");
        assert_eq!(SearchPrefix::Asset.display_name(), "Assets");
        assert_eq!(SearchPrefix::Command.display_name(), "Commands");
        assert_eq!(SearchPrefix::Symbol.display_name(), "Symbols");
        assert_eq!(SearchPrefix::None.display_name(), "All");
    }

    #[test]
    fn test_empty_grouped_results() {
        let groups = GroupedResults::new();
        
        assert_eq!(groups.total_count(), 0);
        assert_eq!(groups.categories().len(), 0);
        assert_eq!(groups.get_category(SearchPrefix::Entity).len(), 0);
    }
}
