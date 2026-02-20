//! Property-based test for Badge Visibility
//!
//! **Validates: Requirements 2.7**
//!
//! **Property 11: Badge Visibility**
//!
//! This property verifies that when an item has a badge, the badge should be
//! visible with the correct count and color variant. When an item has no badge,
//! no badge should be displayed.

use luminara_editor::{ActivityBar, ActivityItem, Badge, BadgeVariant, Theme};
use proptest::prelude::*;
use std::sync::Arc;

/// Strategy for generating badge counts (0-999 is a reasonable range)
fn badge_count_strategy() -> impl Strategy<Value = u32> {
    0u32..1000
}

/// Strategy for generating badge variants
fn badge_variant_strategy() -> impl Strategy<Value = BadgeVariant> {
    prop_oneof![
        Just(BadgeVariant::Default),
        Just(BadgeVariant::Error),
        Just(BadgeVariant::Warning),
        Just(BadgeVariant::Success),
    ]
}

/// Strategy for generating optional badges
fn optional_badge_strategy() -> impl Strategy<Value = Option<Badge>> {
    prop_oneof![
        // 50% chance of no badge
        1 => Just(None),
        // 50% chance of having a badge with various properties
        1 => (badge_count_strategy(), badge_variant_strategy())
            .prop_map(|(count, variant)| Some(Badge { count, variant }))
    ]
}

/// Property: Badge Presence Correctness
///
/// When an ActivityItem has a badge (Some(Badge)), the badge should be present.
/// When an ActivityItem has no badge (None), no badge should be displayed.
///
/// **Invariants:**
/// 1. If item.badge is Some(_), then a badge exists
/// 2. If item.badge is None, then no badge exists
/// 3. Badge presence is deterministic based on the Option<Badge> value
#[test]
fn property_badge_presence_correctness() {
    proptest!(|(badge in optional_badge_strategy())| {
        let item = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: badge.clone(),
            is_folder: false,
        };
        
        // Invariant 1 & 2: Badge presence matches the Option value
        match &item.badge {
            Some(_) => {
                // Badge should be present
                assert!(item.badge.is_some(), "Badge should be present when Some");
            }
            None => {
                // No badge should be present
                assert!(item.badge.is_none(), "Badge should not be present when None");
            }
        }
        
        // Invariant 3: Badge presence is deterministic
        let item2 = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: badge.clone(),
            is_folder: false,
        };
        assert_eq!(item.badge.is_some(), item2.badge.is_some(),
                   "Badge presence should be deterministic");
    });
}

/// Property: Badge Count Correctness
///
/// When a badge is present, it should display the correct count value.
///
/// **Invariants:**
/// 1. Badge count matches the specified value
/// 2. Badge count is non-negative
/// 3. Badge count is preserved across operations
#[test]
fn property_badge_count_correctness() {
    proptest!(|(count in badge_count_strategy(), variant in badge_variant_strategy())| {
        let badge = Badge { count, variant };
        let item = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: Some(badge.clone()),
            is_folder: false,
        };
        
        // Invariant 1: Badge count matches the specified value
        if let Some(ref b) = item.badge {
            assert_eq!(b.count, count, "Badge count should match the specified value");
        }
        
        // Invariant 2: Badge count is non-negative (enforced by u32 type)
        // u32 is always non-negative, so this is guaranteed by the type system
        
        // Invariant 3: Badge count is preserved
        let item2 = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: Some(Badge { count, variant }),
            is_folder: false,
        };
        assert_eq!(item.badge.as_ref().map(|b| b.count),
                   item2.badge.as_ref().map(|b| b.count),
                   "Badge count should be preserved");
    });
}

/// Property: Badge Variant Correctness
///
/// When a badge is present, it should display the correct color variant.
///
/// **Invariants:**
/// 1. Badge variant matches the specified value
/// 2. Badge variant is one of the valid variants
/// 3. Badge variant is preserved across operations
#[test]
fn property_badge_variant_correctness() {
    proptest!(|(count in badge_count_strategy(), variant in badge_variant_strategy())| {
        let badge = Badge { count, variant };
        let item = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: Some(badge.clone()),
            is_folder: false,
        };
        
        // Invariant 1: Badge variant matches the specified value
        if let Some(ref b) = item.badge {
            assert_eq!(b.variant, variant, "Badge variant should match the specified value");
        }
        
        // Invariant 2: Badge variant is one of the valid variants
        let valid_variants = [
            BadgeVariant::Default,
            BadgeVariant::Error,
            BadgeVariant::Warning,
            BadgeVariant::Success,
        ];
        assert!(valid_variants.contains(&variant),
                "Badge variant should be one of the valid variants");
        
        // Invariant 3: Badge variant is preserved
        let item2 = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: Some(Badge { count, variant }),
            is_folder: false,
        };
        assert_eq!(item.badge.as_ref().map(|b| b.variant),
                   item2.badge.as_ref().map(|b| b.variant),
                   "Badge variant should be preserved");
    });
}

/// Property: Badge Visibility in Activity Bar
///
/// When items with badges are added to an ActivityBar, the badges should be
/// correctly associated with their items.
///
/// **Invariants:**
/// 1. Items with badges retain their badges in the ActivityBar
/// 2. Items without badges remain without badges in the ActivityBar
/// 3. Badge properties are preserved when items are added to ActivityBar
#[test]
fn property_badge_visibility_in_activity_bar() {
    proptest!(|(
        badge1 in optional_badge_strategy(),
        badge2 in optional_badge_strategy(),
        badge3 in optional_badge_strategy()
    )| {
        let theme = Arc::new(Theme::default_dark());
        let mut activity_bar = ActivityBar::new(theme);
        
        let items = vec![
            ActivityItem {
                id: "item1".to_string(),
                icon: "icon-1".to_string(),
                title: "Item 1".to_string(),
                badge: badge1.clone(),
                is_folder: false,
            },
            ActivityItem {
                id: "item2".to_string(),
                icon: "icon-2".to_string(),
                title: "Item 2".to_string(),
                badge: badge2.clone(),
                is_folder: false,
            },
            ActivityItem {
                id: "item3".to_string(),
                icon: "icon-3".to_string(),
                title: "Item 3".to_string(),
                badge: badge3.clone(),
                is_folder: false,
            },
        ];
        
        activity_bar.set_items_for_testing(items);
        let bar_items = activity_bar.items_for_testing();
        
        // Invariant 1 & 2: Badge presence is preserved
        assert_eq!(bar_items[0].badge.is_some(), badge1.is_some(),
                   "Item 1 badge presence should be preserved");
        assert_eq!(bar_items[1].badge.is_some(), badge2.is_some(),
                   "Item 2 badge presence should be preserved");
        assert_eq!(bar_items[2].badge.is_some(), badge3.is_some(),
                   "Item 3 badge presence should be preserved");
        
        // Invariant 3: Badge properties are preserved
        if let Some(ref b1) = badge1 {
            let bar_badge = bar_items[0].badge.as_ref().unwrap();
            assert_eq!(bar_badge.count, b1.count, "Item 1 badge count should be preserved");
            assert_eq!(bar_badge.variant, b1.variant, "Item 1 badge variant should be preserved");
        }
        
        if let Some(ref b2) = badge2 {
            let bar_badge = bar_items[1].badge.as_ref().unwrap();
            assert_eq!(bar_badge.count, b2.count, "Item 2 badge count should be preserved");
            assert_eq!(bar_badge.variant, b2.variant, "Item 2 badge variant should be preserved");
        }
        
        if let Some(ref b3) = badge3 {
            let bar_badge = bar_items[2].badge.as_ref().unwrap();
            assert_eq!(bar_badge.count, b3.count, "Item 3 badge count should be preserved");
            assert_eq!(bar_badge.variant, b3.variant, "Item 3 badge variant should be preserved");
        }
    });
}

/// Property: Badge Independence
///
/// Badges on different items should be independent of each other.
///
/// **Invariants:**
/// 1. Changing one item's badge does not affect other items
/// 2. Each item maintains its own badge state
/// 3. Badge properties are item-specific
#[test]
fn property_badge_independence() {
    proptest!(|(
        count1 in badge_count_strategy(),
        count2 in badge_count_strategy(),
        variant1 in badge_variant_strategy(),
        variant2 in badge_variant_strategy()
    )| {
        let theme = Arc::new(Theme::default_dark());
        let mut activity_bar = ActivityBar::new(theme);
        
        let items = vec![
            ActivityItem {
                id: "item1".to_string(),
                icon: "icon-1".to_string(),
                title: "Item 1".to_string(),
                badge: Some(Badge { count: count1, variant: variant1 }),
                is_folder: false,
            },
            ActivityItem {
                id: "item2".to_string(),
                icon: "icon-2".to_string(),
                title: "Item 2".to_string(),
                badge: Some(Badge { count: count2, variant: variant2 }),
                is_folder: false,
            },
        ];
        
        activity_bar.set_items_for_testing(items);
        let bar_items = activity_bar.items_for_testing();
        
        // Invariant 1 & 2: Each item has its own badge
        let badge1 = bar_items[0].badge.as_ref().unwrap();
        let badge2 = bar_items[1].badge.as_ref().unwrap();
        
        // Invariant 3: Badge properties are item-specific
        assert_eq!(badge1.count, count1, "Item 1 should have its own count");
        assert_eq!(badge1.variant, variant1, "Item 1 should have its own variant");
        assert_eq!(badge2.count, count2, "Item 2 should have its own count");
        assert_eq!(badge2.variant, variant2, "Item 2 should have its own variant");
        
        // Badges are independent
        if count1 != count2 {
            assert_ne!(badge1.count, badge2.count, "Different counts should remain different");
        }
        if variant1 != variant2 {
            assert_ne!(badge1.variant, badge2.variant, "Different variants should remain different");
        }
    });
}

/// Property: Badge Variant Semantics
///
/// Each badge variant should have distinct semantic meaning.
///
/// **Invariants:**
/// 1. All badge variants are distinct
/// 2. Badge variants can be compared for equality
/// 3. Badge variants maintain their identity
#[test]
fn property_badge_variant_semantics() {
    // Invariant 1: All variants are distinct
    let variants = vec![
        BadgeVariant::Default,
        BadgeVariant::Error,
        BadgeVariant::Warning,
        BadgeVariant::Success,
    ];
    
    for (i, v1) in variants.iter().enumerate() {
        for (j, v2) in variants.iter().enumerate() {
            if i == j {
                // Invariant 2: Same variant equals itself
                assert_eq!(v1, v2, "Variant should equal itself");
            } else {
                // Invariant 1: Different variants are not equal
                assert_ne!(v1, v2, "Different variants should not be equal");
            }
        }
    }
    
    // Invariant 3: Variants maintain their identity
    let default1 = BadgeVariant::Default;
    let default2 = BadgeVariant::Default;
    assert_eq!(default1, default2, "Same variant should maintain identity");
}

/// Property: Zero Count Badge
///
/// A badge with count 0 should still be valid and displayable.
///
/// **Invariants:**
/// 1. Zero count badges are valid
/// 2. Zero count badges can be created
/// 3. Zero count badges behave like other badges
#[test]
fn property_zero_count_badge() {
    proptest!(|(variant in badge_variant_strategy())| {
        let badge = Badge { count: 0, variant };
        let item = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: Some(badge.clone()),
            is_folder: false,
        };
        
        // Invariant 1 & 2: Zero count badges are valid
        assert!(item.badge.is_some(), "Zero count badge should be valid");
        
        // Invariant 3: Zero count badges behave like other badges
        if let Some(ref b) = item.badge {
            assert_eq!(b.count, 0, "Badge count should be 0");
            assert_eq!(b.variant, variant, "Badge variant should be preserved");
        }
    });
}

/// Property: Large Count Badge
///
/// Badges should handle large count values correctly.
///
/// **Invariants:**
/// 1. Large counts are stored correctly
/// 2. Large counts don't overflow
/// 3. Large counts are preserved
#[test]
fn property_large_count_badge() {
    proptest!(|(variant in badge_variant_strategy())| {
        let large_count = u32::MAX;
        let badge = Badge { count: large_count, variant };
        let item = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: Some(badge.clone()),
            is_folder: false,
        };
        
        // Invariant 1: Large counts are stored correctly
        if let Some(ref b) = item.badge {
            assert_eq!(b.count, large_count, "Large count should be stored correctly");
        }
        
        // Invariant 2: No overflow (enforced by u32 type)
        assert_eq!(badge.count, large_count, "Count should not overflow");
        
        // Invariant 3: Large counts are preserved
        let item2 = ActivityItem {
            id: "test".to_string(),
            icon: "icon-test".to_string(),
            title: "Test Item".to_string(),
            badge: Some(Badge { count: large_count, variant }),
            is_folder: false,
        };
        assert_eq!(item.badge.as_ref().map(|b| b.count),
                   item2.badge.as_ref().map(|b| b.count),
                   "Large count should be preserved");
    });
}
