# Badge Notifications

Badge notifications are visual indicators displayed on Activity Bar items to show notification counts or status.

## Features

### Badge Structure

The `Badge` struct contains:
- `count`: The number to display (e.g., number of notifications)
- `variant`: The color variant (Default, Error, Warning, Success)

### Badge Variants

Four color variants are supported:

1. **Default** (Blue) - General notifications
2. **Error** (Red) - Error or alert notifications
3. **Warning** (Orange) - Warning notifications
4. **Success** (Green) - Success notifications

### Visual Design

Badges are rendered as:
- Small circular indicators (16px × 16px)
- Positioned at the top-right corner of the activity icon
- Display the count number in white text (10px font)
- Fully rounded corners (8px border radius)

## Usage

### Adding a Badge to an Activity Item

```rust
use luminara_editor::activity_bar::{ActivityItem, Badge, BadgeVariant};

let item = ActivityItem {
    id: "backend-ai".to_string(),
    icon: "🤖".to_string(),
    title: "Backend & AI".to_string(),
    badge: Some(Badge {
        count: 3,
        variant: BadgeVariant::Default,
    }),
    is_folder: false,
};
```

### Badge Variants Example

```rust
// Default blue badge
let default_badge = Badge {
    count: 5,
    variant: BadgeVariant::Default,
};

// Error red badge
let error_badge = Badge {
    count: 2,
    variant: BadgeVariant::Error,
};

// Warning orange badge
let warning_badge = Badge {
    count: 10,
    variant: BadgeVariant::Warning,
};

// Success green badge
let success_badge = Badge {
    count: 1,
    variant: BadgeVariant::Success,
};
```

### Removing a Badge

To remove a badge from an item, set it to `None`:

```rust
item.badge = None;
```

## Implementation Details

### Rendering

Badges are rendered in the `render_item` method of the `ActivityBar` component. The rendering logic:

1. Checks if the item has a badge (`item.badge.as_ref()`)
2. Determines the badge color based on the variant
3. Creates an absolutely positioned div at the top-right corner
4. Displays the count as text

### Color Mapping

Badge variants map to theme colors:

- `BadgeVariant::Default` → `theme.colors.accent` (blue)
- `BadgeVariant::Error` → `theme.colors.error` (red)
- `BadgeVariant::Warning` → `theme.colors.warning` (orange)
- `BadgeVariant::Success` → `theme.colors.success` (green)

## Requirements Satisfied

This implementation satisfies:

- **Requirement 2.7**: THE Activity_Bar SHALL display badge notifications on items
- Badge rendering with count display
- Multiple color variants for different notification types
- Proper visual positioning and styling

## Testing

Badge functionality is tested in `activity_bar.rs`:

- `test_badge_variants` - Tests badge structure
- `test_badge_notification_display` - Tests badge attachment to items
- `test_badge_count_display` - Tests various count values
- `test_all_badge_variants` - Tests all color variants
- `test_activity_bar_with_badges` - Tests ActivityBar initialization with badges

Run tests with:

```bash
cargo test --package luminara_editor --lib activity_bar::tests
```

## Example

The default ActivityBar includes a badge on the "Backend & AI" item:

```rust
ActivityItem {
    id: "backend-ai".to_string(),
    icon: "🤖".to_string(),
    title: "Backend & AI".to_string(),
    badge: Some(Badge {
        count: 3,
        variant: BadgeVariant::Default,
    }),
    is_folder: false,
}
```

This displays a blue badge with the number "3" on the AI icon.
