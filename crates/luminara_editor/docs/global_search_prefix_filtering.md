# Global Search Prefix-Based Filtering Implementation

## Overview

This document describes the implementation of prefix-based filtering for the Global Search component (Task 8.2).

## Requirements

**Requirement 3.3**: THE Global_Search SHALL support prefix-based filtering (@, #, /, :)

## Implementation

### SearchPrefix Enum

A new `SearchPrefix` enum was added to represent the different filter types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchPrefix {
    Entity,   // @ prefix
    Asset,    // # prefix
    Command,  // / prefix
    Symbol,   // : prefix
    None,     // No prefix
}
```

### Prefix Detection

The `SearchPrefix::parse()` method detects prefixes at the start of search queries:

- `@` - Filters for entities (scene objects, game entities)
- `#` - Filters for assets (textures, models, audio, etc.)
- `/` - Filters for commands (editor commands, actions)
- `:` - Filters for symbols (functions, classes, variables in code)

**Example Usage:**
```rust
let (prefix, query) = SearchPrefix::parse("@player");
// prefix = SearchPrefix::Entity
// query = "player"

let (prefix, query) = SearchPrefix::parse("#texture.png");
// prefix = SearchPrefix::Asset
// query = "texture.png"
```

### GlobalSearch Component Updates

The `GlobalSearch` struct was updated to track the current filter:

```rust
pub struct GlobalSearch {
    theme: Arc<Theme>,
    visible: bool,
    query: String,              // Full query including prefix
    focus_handle: FocusHandle,
    prefix: SearchPrefix,       // Current filter type
    filtered_query: String,     // Query without prefix
}
```

### Query Processing

The `set_query()` method automatically parses the prefix and updates the filter:

```rust
pub fn set_query(&mut self, query: String) {
    self.query = query;
    let (prefix, filtered) = SearchPrefix::parse(&self.query);
    self.prefix = prefix;
    self.filtered_query = filtered.to_string();
}
```

### Result Filtering

The `should_include_result()` method determines if a search result should be displayed based on the current filter:

```rust
pub fn should_include_result(&self, result_type: SearchPrefix) -> bool {
    match self.prefix {
        SearchPrefix::None => true,  // No filter, include all
        _ => self.prefix == result_type,
    }
}
```

### UI Updates

The search input now displays:
- Placeholder text showing available prefixes: "Search... (@ entities, # assets, / commands, : symbols)"
- Active filter and query when a prefix is used: "Filter: Entities | Query: player"
- Results section header shows the current filter: "Results - Entities"

## Testing

Comprehensive unit tests were added to verify:

1. **Prefix Parsing**
   - Entity prefix (@)
   - Asset prefix (#)
   - Command prefix (/)
   - Symbol prefix (:)
   - No prefix (plain search)
   - Empty queries
   - Prefix-only queries

2. **Display Names**
   - Correct display names for each prefix type
   - Correct prefix characters

3. **Filtering Logic**
   - No filter includes all result types
   - Entity filter includes only entities
   - Asset filter includes only assets
   - Command filter includes only commands
   - Symbol filter includes only symbols

4. **Edge Cases**
   - Special characters in queries
   - Unicode characters in queries
   - Query update workflows

## Integration

The prefix filtering system integrates with:

1. **Search Input**: Automatically detects and parses prefixes as the user types
2. **Results Display**: Filters results based on the current prefix
3. **Preview Panel**: Shows filtered results in the preview
4. **Database Queries**: Will be used to filter database queries (future task 8.10)

## Future Enhancements

When implementing task 8.10 (Database Integration), the prefix filter will be used to:
- Query only entities when `@` prefix is used
- Query only assets when `#` prefix is used
- Query only commands when `/` prefix is used
- Query only symbols when `:` prefix is used
- Query all types when no prefix is used

## Files Modified

- `crates/luminara_editor/src/global_search.rs` - Added SearchPrefix enum and filtering logic
- `crates/luminara_editor/tests/global_search_prefix_test.rs` - Added comprehensive unit tests

## Status

✅ Task 8.2 Complete - Prefix detection and filtering logic implemented
- Prefix parsing: ✅
- Filter state management: ✅
- UI updates: ✅
- Unit tests: ✅
- Integration with database queries: ⏳ (Task 8.10)
