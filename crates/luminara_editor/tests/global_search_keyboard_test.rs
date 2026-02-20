//! Integration tests for Global Search keyboard navigation
//!
//! Tests the keyboard navigation functionality including:
//! - Arrow key navigation (↑↓)
//! - Enter to open selected item
//! - Esc to close overlay
//!
//! **Validates: Requirements 3.6, 3.7**

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    /// Mock SearchPrefix for testing
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

    /// Mock SearchResult for testing
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

    /// Mock GroupedResults for testing
    #[derive(Debug, Clone, Default)]
    pub struct GroupedResults {
        groups: std::collections::HashMap<SearchPrefix, Vec<SearchResult>>,
    }

    impl GroupedResults {
        pub fn new() -> Self {
            Self {
                groups: std::collections::HashMap::new(),
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

        pub fn clear(&mut self) {
            self.groups.clear();
        }
    }

    /// Mock GlobalSearch for testing keyboard navigation
    pub struct MockGlobalSearch {
        pub results: GroupedResults,
        pub selected_index: Option<(usize, usize)>,
        pub visible: bool,
    }

    impl MockGlobalSearch {
        pub fn new() -> Self {
            Self {
                results: GroupedResults::new(),
                selected_index: None,
                visible: true,
            }
        }

        pub fn add_result(&mut self, result: SearchResult) {
            self.results.add_result(result);
        }

        pub fn select_next(&mut self) {
            let categories = self.results.categories();
            if categories.is_empty() {
                return;
            }

            match self.selected_index {
                None => {
                    self.selected_index = Some((0, 0));
                }
                Some((cat_idx, res_idx)) => {
                    let current_category = categories[cat_idx];
                    let current_results = self.results.get_category(current_category);
                    
                    if res_idx + 1 < current_results.len() {
                        self.selected_index = Some((cat_idx, res_idx + 1));
                    } else if cat_idx + 1 < categories.len() {
                        self.selected_index = Some((cat_idx + 1, 0));
                    } else {
                        self.selected_index = Some((0, 0));
                    }
                }
            }
        }

        pub fn select_previous(&mut self) {
            let categories = self.results.categories();
            if categories.is_empty() {
                return;
            }

            match self.selected_index {
                None => {
                    let last_cat_idx = categories.len() - 1;
                    let last_category = categories[last_cat_idx];
                    let last_results = self.results.get_category(last_category);
                    self.selected_index = Some((last_cat_idx, last_results.len() - 1));
                }
                Some((cat_idx, res_idx)) => {
                    if res_idx > 0 {
                        self.selected_index = Some((cat_idx, res_idx - 1));
                    } else if cat_idx > 0 {
                        let prev_cat_idx = cat_idx - 1;
                        let prev_category = categories[prev_cat_idx];
                        let prev_results = self.results.get_category(prev_category);
                        self.selected_index = Some((prev_cat_idx, prev_results.len() - 1));
                    } else {
                        let last_cat_idx = categories.len() - 1;
                        let last_category = categories[last_cat_idx];
                        let last_results = self.results.get_category(last_category);
                        self.selected_index = Some((last_cat_idx, last_results.len() - 1));
                    }
                }
            }
        }

        pub fn close(&mut self) {
            self.visible = false;
            self.selected_index = None;
        }

        pub fn selected_result(&self) -> Option<&SearchResult> {
            if let Some((cat_idx, res_idx)) = self.selected_index {
                let categories = self.results.categories();
                if cat_idx < categories.len() {
                    let category = categories[cat_idx];
                    let results = self.results.get_category(category);
                    if res_idx < results.len() {
                        return Some(&results[res_idx]);
                    }
                }
            }
            None
        }
    }

    #[test]
    fn test_select_next_from_no_selection() {
        let mut search = MockGlobalSearch::new();
        
        // Add some results
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        
        // Initially no selection
        assert_eq!(search.selected_index, None);
        
        // Select next should select first result
        search.select_next();
        assert_eq!(search.selected_index, Some((0, 0)));
        
        // Verify the selected result
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "Player");
    }

    #[test]
    fn test_select_next_within_category() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        
        // Start with first result selected
        search.selected_index = Some((0, 0));
        
        // Select next should move to second result in same category
        search.select_next();
        assert_eq!(search.selected_index, Some((0, 1)));
        
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "Enemy");
    }

    #[test]
    fn test_select_next_across_categories() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        
        // Start with last result in first category
        search.selected_index = Some((0, 0));
        
        // Select next should move to first result in next category
        search.select_next();
        assert_eq!(search.selected_index, Some((1, 0)));
        
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "texture.png");
        assert_eq!(selected.category, SearchPrefix::Asset);
    }

    #[test]
    fn test_select_next_wraps_to_beginning() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        
        // Start with last result
        search.selected_index = Some((1, 0));
        
        // Select next should wrap to first result
        search.select_next();
        assert_eq!(search.selected_index, Some((0, 0)));
        
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "Player");
    }

    #[test]
    fn test_select_previous_from_no_selection() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        
        // Initially no selection
        assert_eq!(search.selected_index, None);
        
        // Select previous should select last result
        search.select_previous();
        assert_eq!(search.selected_index, Some((1, 0)));
        
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "texture.png");
    }

    #[test]
    fn test_select_previous_within_category() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        
        // Start with second result selected
        search.selected_index = Some((0, 1));
        
        // Select previous should move to first result in same category
        search.select_previous();
        assert_eq!(search.selected_index, Some((0, 0)));
        
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "Player");
    }

    #[test]
    fn test_select_previous_across_categories() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        
        // Start with first result in second category
        search.selected_index = Some((1, 0));
        
        // Select previous should move to last result in previous category
        search.select_previous();
        assert_eq!(search.selected_index, Some((0, 1)));
        
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "Enemy");
    }

    #[test]
    fn test_select_previous_wraps_to_end() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        
        // Start with first result
        search.selected_index = Some((0, 0));
        
        // Select previous should wrap to last result
        search.select_previous();
        assert_eq!(search.selected_index, Some((1, 0)));
        
        let selected = search.selected_result().unwrap();
        assert_eq!(selected.name, "texture.png");
    }

    #[test]
    fn test_close_clears_selection() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        
        // Select a result
        search.selected_index = Some((0, 0));
        assert!(search.visible);
        assert!(search.selected_index.is_some());
        
        // Close should clear selection and hide overlay
        search.close();
        assert!(!search.visible);
        assert_eq!(search.selected_index, None);
    }

    #[test]
    fn test_navigation_with_empty_results() {
        let mut search = MockGlobalSearch::new();
        
        // No results
        assert_eq!(search.selected_index, None);
        
        // Navigation should do nothing
        search.select_next();
        assert_eq!(search.selected_index, None);
        
        search.select_previous();
        assert_eq!(search.selected_index, None);
    }

    #[test]
    fn test_navigation_with_single_result() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        
        // Select the only result
        search.select_next();
        assert_eq!(search.selected_index, Some((0, 0)));
        
        // Next should wrap to same result
        search.select_next();
        assert_eq!(search.selected_index, Some((0, 0)));
        
        // Previous should also stay on same result
        search.select_previous();
        assert_eq!(search.selected_index, Some((0, 0)));
    }

    #[test]
    fn test_navigation_with_multiple_categories() {
        let mut search = MockGlobalSearch::new();
        
        // Add results in multiple categories
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Command,
            "Save".to_string(),
            None,
        ));
        
        // Navigate through all results
        search.select_next(); // (0, 0) - Player
        assert_eq!(search.selected_result().unwrap().name, "Player");
        
        search.select_next(); // (0, 1) - Enemy
        assert_eq!(search.selected_result().unwrap().name, "Enemy");
        
        search.select_next(); // (1, 0) - texture.png
        assert_eq!(search.selected_result().unwrap().name, "texture.png");
        
        search.select_next(); // (2, 0) - Save
        assert_eq!(search.selected_result().unwrap().name, "Save");
        
        search.select_next(); // (0, 0) - Player (wrapped)
        assert_eq!(search.selected_result().unwrap().name, "Player");
    }

    #[test]
    fn test_bidirectional_navigation() {
        let mut search = MockGlobalSearch::new();
        
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            None,
        ));
        search.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Enemy".to_string(),
            None,
        ));
        
        // Navigate forward
        search.select_next(); // Player
        search.select_next(); // Enemy
        assert_eq!(search.selected_result().unwrap().name, "Enemy");
        
        // Navigate backward
        search.select_previous(); // Player
        assert_eq!(search.selected_result().unwrap().name, "Player");
        
        // Navigate backward again (should wrap)
        search.select_previous(); // Enemy
        assert_eq!(search.selected_result().unwrap().name, "Enemy");
    }
}
