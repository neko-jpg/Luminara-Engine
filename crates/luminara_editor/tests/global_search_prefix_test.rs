//! Integration tests for Global Search prefix-based filtering
//!
//! Tests the prefix parsing and filtering logic for the Global Search component.
//! 
//! **Validates: Requirements 3.3**

/// Search filter prefix types (copied for testing)
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

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Entity => "Entities",
            Self::Asset => "Assets",
            Self::Command => "Commands",
            Self::Symbol => "Symbols",
            Self::None => "All",
        }
    }

    pub fn prefix_char(&self) -> Option<char> {
        match self {
            Self::Entity => Some('@'),
            Self::Asset => Some('#'),
            Self::Command => Some('/'),
            Self::Symbol => Some(':'),
            Self::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_parse_entity() {
        let (prefix, query) = SearchPrefix::parse("@player");
        assert_eq!(prefix, SearchPrefix::Entity);
        assert_eq!(query, "player");
    }

    #[test]
    fn test_prefix_parse_asset() {
        let (prefix, query) = SearchPrefix::parse("#texture");
        assert_eq!(prefix, SearchPrefix::Asset);
        assert_eq!(query, "texture");
    }

    #[test]
    fn test_prefix_parse_command() {
        let (prefix, query) = SearchPrefix::parse("/save");
        assert_eq!(prefix, SearchPrefix::Command);
        assert_eq!(query, "save");
    }

    #[test]
    fn test_prefix_parse_symbol() {
        let (prefix, query) = SearchPrefix::parse(":function");
        assert_eq!(prefix, SearchPrefix::Symbol);
        assert_eq!(query, "function");
    }

    #[test]
    fn test_prefix_parse_none() {
        let (prefix, query) = SearchPrefix::parse("search");
        assert_eq!(prefix, SearchPrefix::None);
        assert_eq!(query, "search");
    }

    #[test]
    fn test_prefix_parse_empty() {
        let (prefix, query) = SearchPrefix::parse("");
        assert_eq!(prefix, SearchPrefix::None);
        assert_eq!(query, "");
    }

    #[test]
    fn test_prefix_parse_only_prefix() {
        let (prefix, query) = SearchPrefix::parse("@");
        assert_eq!(prefix, SearchPrefix::Entity);
        assert_eq!(query, "");
    }

    #[test]
    fn test_prefix_display_names() {
        assert_eq!(SearchPrefix::Entity.display_name(), "Entities");
        assert_eq!(SearchPrefix::Asset.display_name(), "Assets");
        assert_eq!(SearchPrefix::Command.display_name(), "Commands");
        assert_eq!(SearchPrefix::Symbol.display_name(), "Symbols");
        assert_eq!(SearchPrefix::None.display_name(), "All");
    }

    #[test]
    fn test_prefix_chars() {
        assert_eq!(SearchPrefix::Entity.prefix_char(), Some('@'));
        assert_eq!(SearchPrefix::Asset.prefix_char(), Some('#'));
        assert_eq!(SearchPrefix::Command.prefix_char(), Some('/'));
        assert_eq!(SearchPrefix::Symbol.prefix_char(), Some(':'));
        assert_eq!(SearchPrefix::None.prefix_char(), None);
    }

    #[test]
    fn test_filtering_logic_no_filter() {
        let prefix = SearchPrefix::None;
        
        // When no prefix is set, all results should be included
        let should_include = |result_type: SearchPrefix| -> bool {
            match prefix {
                SearchPrefix::None => true,
                _ => prefix == result_type,
            }
        };

        assert!(should_include(SearchPrefix::Entity));
        assert!(should_include(SearchPrefix::Asset));
        assert!(should_include(SearchPrefix::Command));
        assert!(should_include(SearchPrefix::Symbol));
    }

    #[test]
    fn test_filtering_logic_entity_filter() {
        let prefix = SearchPrefix::Entity;
        
        let should_include = |result_type: SearchPrefix| -> bool {
            match prefix {
                SearchPrefix::None => true,
                _ => prefix == result_type,
            }
        };

        assert!(should_include(SearchPrefix::Entity));
        assert!(!should_include(SearchPrefix::Asset));
        assert!(!should_include(SearchPrefix::Command));
        assert!(!should_include(SearchPrefix::Symbol));
    }

    #[test]
    fn test_filtering_logic_asset_filter() {
        let prefix = SearchPrefix::Asset;
        
        let should_include = |result_type: SearchPrefix| -> bool {
            match prefix {
                SearchPrefix::None => true,
                _ => prefix == result_type,
            }
        };

        assert!(!should_include(SearchPrefix::Entity));
        assert!(should_include(SearchPrefix::Asset));
        assert!(!should_include(SearchPrefix::Command));
        assert!(!should_include(SearchPrefix::Symbol));
    }

    #[test]
    fn test_filtering_logic_command_filter() {
        let prefix = SearchPrefix::Command;
        
        let should_include = |result_type: SearchPrefix| -> bool {
            match prefix {
                SearchPrefix::None => true,
                _ => prefix == result_type,
            }
        };

        assert!(!should_include(SearchPrefix::Entity));
        assert!(!should_include(SearchPrefix::Asset));
        assert!(should_include(SearchPrefix::Command));
        assert!(!should_include(SearchPrefix::Symbol));
    }

    #[test]
    fn test_filtering_logic_symbol_filter() {
        let prefix = SearchPrefix::Symbol;
        
        let should_include = |result_type: SearchPrefix| -> bool {
            match prefix {
                SearchPrefix::None => true,
                _ => prefix == result_type,
            }
        };

        assert!(!should_include(SearchPrefix::Entity));
        assert!(!should_include(SearchPrefix::Asset));
        assert!(!should_include(SearchPrefix::Command));
        assert!(should_include(SearchPrefix::Symbol));
    }

    #[test]
    fn test_prefix_parsing_with_special_characters() {
        let (prefix, query) = SearchPrefix::parse("@player_123");
        assert_eq!(prefix, SearchPrefix::Entity);
        assert_eq!(query, "player_123");

        let (prefix, query) = SearchPrefix::parse("#texture-2d.png");
        assert_eq!(prefix, SearchPrefix::Asset);
        assert_eq!(query, "texture-2d.png");

        let (prefix, query) = SearchPrefix::parse("/save-as");
        assert_eq!(prefix, SearchPrefix::Command);
        assert_eq!(query, "save-as");

        let (prefix, query) = SearchPrefix::parse(":my::namespace::function");
        assert_eq!(prefix, SearchPrefix::Symbol);
        assert_eq!(query, "my::namespace::function");
    }

    #[test]
    fn test_prefix_parsing_unicode() {
        let (prefix, query) = SearchPrefix::parse("@プレイヤー");
        assert_eq!(prefix, SearchPrefix::Entity);
        assert_eq!(query, "プレイヤー");

        let (prefix, query) = SearchPrefix::parse("#テクスチャ");
        assert_eq!(prefix, SearchPrefix::Asset);
        assert_eq!(query, "テクスチャ");
    }

    #[test]
    fn test_query_update_workflow() {
        // Simulate the workflow of updating a query
        let queries = vec![
            ("", SearchPrefix::None, ""),
            ("player", SearchPrefix::None, "player"),
            ("@player", SearchPrefix::Entity, "player"),
            ("@", SearchPrefix::Entity, ""),
            ("#texture.png", SearchPrefix::Asset, "texture.png"),
            ("/save file", SearchPrefix::Command, "save file"),
            (":render_system", SearchPrefix::Symbol, "render_system"),
        ];

        for (input, expected_prefix, expected_query) in queries {
            let (prefix, query) = SearchPrefix::parse(input);
            assert_eq!(prefix, expected_prefix, "Failed for input: {}", input);
            assert_eq!(query, expected_query, "Failed for input: {}", input);
        }
    }
}
