# Global Search Preview Panel Implementation

## Overview

This document describes the implementation of the preview panel for the Global Search feature, fulfilling **Requirement 3.5**: Display real-time preview of selected items.

## Implementation Details

### Data Structure Changes

Added to `GlobalSearch` struct:
- `selected_index: Option<(usize, usize)>` - Tracks the currently selected result by category index and result index within that category

### New Methods

#### `set_selected(category_index: usize, result_index: usize)`
Sets the currently selected result for preview display.

**Parameters:**
- `category_index` - Index of the category in the categories list
- `result_index` - Index of the result within that category

**Requirements:** Requirement 3.5 - Update preview on selection change

#### `get_selected_result() -> Option<&SearchResult>`
Returns the currently selected search result for display in the preview panel.

**Returns:** The selected `SearchResult`, or `None` if no selection exists

**Requirements:** Requirement 3.5 - Display real-time preview of selected items

### UI Changes

#### Results Column (38%)
- Added visual indication of selected item using `surface_active` background color
- Results are now visually distinct when selected vs hovered
- Selection state is tracked and persists across renders

#### Preview Column (62%)
The preview panel now displays:

**When an item is selected:**
1. **Preview header** - "Preview" label in secondary text color
2. **Item name** - Large, bold text showing the selected item's name
3. **Item type** - Shows the category (Entities, Assets, Commands, Symbols)
4. **Description section** (if available):
   - "Description" label
   - Full description text from the search result

**When no item is selected:**
- "Preview" header
- "Select an item to preview" placeholder text

### Layout Structure

```
┌─────────────────────────────────────────────────────┐
│ Search Input                                         │
├──────────────────┬──────────────────────────────────┤
│ Results (38%)    │ Preview (62%)                    │
│                  │                                   │
│ Category Header  │ Preview                          │
│ ┌──────────────┐ │ ┌──────────────────────────────┐ │
│ │ Result 1     │ │ │ Item Name (Large, Bold)      │ │
│ │ Description  │ │ │                              │ │
│ └──────────────┘ │ │ Type: Category               │ │
│ ┌──────────────┐ │ │                              │ │
│ │ Result 2     │ │ │ Description                  │ │
│ │ [SELECTED]   │ │ │ Full description text...     │ │
│ └──────────────┘ │ │                              │ │
│                  │ └──────────────────────────────┘ │
└──────────────────┴──────────────────────────────────┘
```

## Requirements Validation

### Requirement 3.5: Display real-time preview of selected items

✅ **Implemented:**
- Preview panel displays selected item details in real-time
- Selection state is tracked via `selected_index` field
- Preview updates immediately when selection changes
- Gracefully handles no selection with placeholder text

### Visual Design

The preview panel follows the theme system:
- Uses `theme.colors.text` for primary content
- Uses `theme.colors.text_secondary` for labels and metadata
- Uses `theme.spacing` for consistent padding and margins
- Uses `theme.typography` for text sizing hierarchy

### Interaction Flow

1. User searches for items using the search input
2. Results are displayed in the left column (38%), grouped by category
3. User hovers over a result → background changes to `surface_hover`
4. User clicks a result → background changes to `surface_active` and preview updates
5. Preview panel (62%) displays:
   - Item name in large, bold text
   - Item type/category
   - Full description (if available)

## Testing

### Unit Tests
Located in `crates/luminara_editor/src/global_search.rs`:
- `test_preview_panel_selection` - Tests selection tracking logic
- `test_preview_panel_no_selection` - Tests empty state handling
- `test_preview_panel_updates_on_selection_change` - Tests selection updates

### Integration Tests
Located in `crates/luminara_editor/tests/global_search_preview_test.rs`:
- `test_preview_panel_displays_selected_item` - Verifies preview displays correct item
- `test_preview_panel_updates_on_selection_change` - Verifies preview updates on selection
- `test_preview_panel_handles_no_selection` - Verifies graceful handling of no selection
- `test_preview_panel_displays_all_result_types` - Verifies all result types display correctly
- `test_preview_panel_handles_results_without_description` - Verifies optional descriptions
- `test_preview_panel_real_time_updates` - Verifies real-time update behavior

## Future Enhancements

Potential improvements for future iterations:

1. **Rich Previews:**
   - Asset thumbnails for textures, models, audio
   - Code syntax highlighting for symbols
   - Entity hierarchy visualization

2. **Keyboard Navigation:**
   - Arrow keys to navigate results
   - Enter to open selected item
   - Tab to switch between results and preview

3. **Quick Actions:**
   - Context menu in preview panel
   - Quick edit/open buttons
   - Copy path/reference buttons

4. **Performance:**
   - Lazy loading of preview content
   - Caching of rendered previews
   - Virtualized result list for large result sets

## Related Files

- `crates/luminara_editor/src/global_search.rs` - Main implementation
- `crates/luminara_editor/tests/global_search_preview_test.rs` - Integration tests
- `crates/luminara_editor/src/theme.rs` - Theme system for styling
- `.kiro/specs/gpui-editor-ui/requirements.md` - Requirements document
- `.kiro/specs/gpui-editor-ui/design.md` - Design document

## References

- **Requirement 3.5:** THE Global_Search SHALL display real-time preview of selected items
- **Design Section:** Preview panel (62% of the 2-column layout) should display detailed information about the currently selected search result, updating in real-time as the user navigates through results
