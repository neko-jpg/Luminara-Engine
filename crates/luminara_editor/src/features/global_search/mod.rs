//! Global Search overlay component
//!
//! Provides a Cmd+K/Ctrl+K triggered search overlay with 2-column layout
//! for searching across entities, assets, and code in the project.
//!
//! This implementation matches the HTML prototype design with:
//! - Left column (38%): Search input, prefix hints, filter chips, results list, recent items
//! - Right column (62%): Preview panel with metadata, tabs, and detail view

use crate::core::state::EditorState;
use crate::ui::theme::Theme;
use gpui::{
    div, px, rgb, IntoElement, ParentElement, Render, Styled, ViewContext,
    Hsla, actions, InteractiveElement, FocusHandle,
    FontWeight,
};
use gpui::prelude::FluentBuilder;
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

actions!(global_search, [Toggle, Close]);

/// Search filter prefix types
///
/// Prefixes allow users to filter search results by type:
/// - `>` - Commands (editor commands, actions)
/// - `@` - Entities (scene objects, game entities)
/// - `#` - Assets (textures, models, audio, etc.)
/// - `:` - DB Query (database queries)
/// - `?` - AI Ask (AI assistant queries)
/// - `!` - Scripts (script files)
/// - `/` - Scene (scenes/timelines)
/// - `%` - Logic Node (logic graph nodes)
///
/// # Requirements
/// - Requirement 3.3: Support prefix-based filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchPrefix {
    /// `>` - Commands
    Command,
    /// `@` - Entities
    Entity,
    /// `#` - Assets
    Asset,
    /// `:` - DB Query
    DbQuery,
    /// `?` - AI Ask
    AiAsk,
    /// `!` - Scripts
    Script,
    /// `/` - Scene
    Scene,
    /// `%` - Logic Node
    LogicNode,
    /// No prefix - search all types
    None,
}

impl SearchPrefix {
    /// Parse a prefix from the start of a query string
    ///
    /// # Arguments
    /// * `query` - The search query string
    ///
    /// # Returns
    /// A tuple of (prefix, remaining_query)
    pub fn parse(query: &str) -> (Self, &str) {
        if query.is_empty() {
            return (Self::None, query);
        }

        match query.chars().next() {
            Some('>') => (Self::Command, &query[1..]),
            Some('@') => (Self::Entity, &query[1..]),
            Some('#') => (Self::Asset, &query[1..]),
            Some(':') => (Self::DbQuery, &query[1..]),
            Some('?') => (Self::AiAsk, &query[1..]),
            Some('!') => (Self::Script, &query[1..]),
            Some('/') => (Self::Scene, &query[1..]),
            Some('%') => (Self::LogicNode, &query[1..]),
            _ => (Self::None, query),
        }
    }

    /// Get the display name for this prefix type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Command => "Commands",
            Self::Entity => "Entities",
            Self::Asset => "Assets",
            Self::DbQuery => "DB",
            Self::AiAsk => "AI",
            Self::Script => "Scripts",
            Self::Scene => "Scenes",
            Self::LogicNode => "Logic",
            Self::None => "All",
        }
    }

    /// Get the prefix character
    pub fn prefix_char(&self) -> Option<char> {
        match self {
            Self::Command => Some('>'),
            Self::Entity => Some('@'),
            Self::Asset => Some('#'),
            Self::DbQuery => Some(':'),
            Self::AiAsk => Some('?'),
            Self::Script => Some('!'),
            Self::Scene => Some('/'),
            Self::LogicNode => Some('%'),
            Self::None => None,
        }
    }

    /// Get the icon for this prefix type
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Command => "⚡",
            Self::Entity => "🧊",
            Self::Asset => "🖼️",
            Self::DbQuery => "🗄️",
            Self::AiAsk => "🤖",
            Self::Script => "📜",
            Self::Scene => "🎬",
            Self::LogicNode => "🔀",
            Self::None => "🔍",
        }
    }

    /// Get all category types (excluding None)
    pub fn all_categories() -> Vec<Self> {
        vec![
            Self::Entity,
            Self::Asset,
            Self::Command,
            Self::Script,
        ]
    }

    /// Get filter chip categories for UI display
    pub fn filter_categories() -> Vec<Self> {
        vec![
            Self::None,      // All
            Self::Entity,    // Entities
            Self::Script,    // Scripts
            Self::LogicNode, // Logic
            Self::Scene,     // Media
            Self::DbQuery,   // DB
        ]
    }
}

/// Filter chip category for quick filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterCategory {
    All,
    Entities,
    Scripts,
    Logic,
    Media,
    Db,
}

impl FilterCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Entities => "Entities",
            Self::Scripts => "Scripts",
            Self::Logic => "Logic",
            Self::Media => "Media",
            Self::Db => "DB",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::All => "📚",
            Self::Entities => "🧊",
            Self::Scripts => "📜",
            Self::Logic => "🔀",
            Self::Media => "🎬",
            Self::Db => "🗄️",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::All,
            Self::Entities,
            Self::Scripts,
            Self::Logic,
            Self::Media,
            Self::Db,
        ]
    }
}

/// A search result item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// The category this result belongs to
    pub category: SearchPrefix,
    /// The display name of the result
    pub name: String,
    /// Optional context/path information
    pub context: String,
    /// Badge type label
    pub badge: String,
    /// Icon for the result
    pub icon: &'static str,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        category: SearchPrefix,
        name: String,
        context: String,
        badge: String,
        icon: &'static str,
    ) -> Self {
        Self {
            category,
            name,
            context,
            badge,
            icon,
        }
    }
}

/// Recent item for quick access
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentItem {
    pub name: String,
    pub context: String,
    pub icon: &'static str,
}

impl RecentItem {
    pub fn new(name: String, context: String, icon: &'static str) -> Self {
        Self { name, context, icon }
    }
}

/// Preview tab types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewTab {
    Details,
    Code,
    Dependencies,
}

impl PreviewTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Details => "Details",
            Self::Code => "Code",
            Self::Dependencies => "Dependencies",
        }
    }
}

/// Preview panel data
#[derive(Debug, Clone)]
pub struct PreviewData {
    pub title: String,
    pub item_type: String,
    pub last_modified: String,
    pub components: String,
    pub size: String,
    pub json_content: String,
}

impl Default for PreviewData {
    fn default() -> Self {
        Self {
            title: "Player".to_string(),
            item_type: "Entity (Character)".to_string(),
            last_modified: "2025-02-18 14:32".to_string(),
            components: "Transform, Rigidbody, AI".to_string(),
            size: "24 KB".to_string(),
            json_content: r#"{
  "name": "Player",
  "transform": [2.5, 1.0, 0.0],
  "components": ["Transform", "PlayerController"],
  "tags": ["character", "main"]
}"#.to_string(),
        }
    }
}

/// Grouped search results by category
#[derive(Debug, Clone, Default)]
pub struct GroupedResults {
    groups: HashMap<SearchPrefix, Vec<SearchResult>>,
}

impl GroupedResults {
    /// Create a new empty GroupedResults
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Add a result to the appropriate category group
    pub fn add_result(&mut self, result: SearchResult) {
        self.groups
            .entry(result.category)
            .or_insert_with(Vec::new)
            .push(result);
    }

    /// Get results for a specific category
    pub fn get_category(&self, category: SearchPrefix) -> &[SearchResult] {
        self.groups.get(&category).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all non-empty categories in a consistent order
    pub fn categories(&self) -> Vec<SearchPrefix> {
        SearchPrefix::all_categories()
            .into_iter()
            .filter(|cat| !self.get_category(*cat).is_empty())
            .collect()
    }

    /// Get the total number of results across all categories
    pub fn total_count(&self) -> usize {
        self.groups.values().map(|v| v.len()).sum()
    }

    /// Clear all results
    pub fn clear(&mut self) {
        self.groups.clear();
    }

    /// Get top results (first few from each category)
    pub fn get_top_results(&self, limit: usize) -> Vec<(SearchPrefix, Vec<SearchResult>)> {
        let mut top = Vec::new();
        for category in self.categories() {
            let results = self.get_category(category);
            let limited: Vec<_> = results.iter().take(limit).cloned().collect();
            if !limited.is_empty() {
                top.push((category, limited));
            }
        }
        top
    }
}

/// Global Search overlay component
///
/// Displays a modal overlay with:
/// - Search input at the top
/// - 2-column layout: results (38%) and preview (62%)
/// - Keyboard shortcuts: Cmd+K/Ctrl+K to open, Esc to close
/// - Prefix-based filtering: >, @, #, :, ?, !, /, %
/// - Grouped results by category with headers
/// - Real-time preview of selected items
///
/// # Requirements
/// - Requirement 3.1: Display on Cmd+K/Ctrl+K
/// - Requirement 3.2: Use 2-column layout (38% / 62%)
/// - Requirement 3.3: Support prefix-based filtering
/// - Requirement 3.4: Group results by category
/// - Requirement 3.5: Display real-time preview of selected items
pub struct GlobalSearch {
    /// Theme for styling
    theme: Arc<Theme>,
    /// Whether the overlay is visible
    visible: bool,
    /// Search query text (including prefix)
    query: String,
    /// Focus handle for the search input
    #[allow(dead_code)]
    focus_handle: FocusHandle,
    /// Current search prefix filter
    prefix: SearchPrefix,
    /// Filtered query (without prefix)
    filtered_query: String,
    /// Grouped search results
    results: GroupedResults,
    /// Index of the currently selected result
    selected_index: Option<(SearchPrefix, usize)>,
    /// Current filter category
    filter_category: FilterCategory,
    /// Recent items
    recent_items: Vec<RecentItem>,
    /// Preview data
    preview_data: PreviewData,
    /// Active preview tab
    active_preview_tab: PreviewTab,
    /// Shared editor state
    editor_state: Option<Arc<RwLock<EditorState>>>,
}

impl GlobalSearch {
    /// Create a new GlobalSearch overlay
    pub fn new(theme: Arc<Theme>, cx: &mut ViewContext<Self>) -> Self {
        Self::with_state(theme, None, cx)
    }
    
    /// Create a new GlobalSearch overlay with shared state
    pub fn with_state(
        theme: Arc<Theme>, 
        editor_state: Option<Arc<RwLock<EditorState>>>, 
        cx: &mut ViewContext<Self>
    ) -> Self {
        let focus_handle = cx.focus_handle();
        
        // Get initial visibility from state if available
        let initial_visible = editor_state
            .as_ref()
            .map(|s| s.read().global_search_visible)
            .unwrap_or(false);
        
        let mut search = Self {
            theme,
            visible: initial_visible,
            query: String::new(),
            focus_handle,
            prefix: SearchPrefix::None,
            filtered_query: String::new(),
            results: GroupedResults::new(),
            selected_index: None,
            filter_category: FilterCategory::All,
            recent_items: Self::default_recent_items(),
            preview_data: PreviewData::default(),
            active_preview_tab: PreviewTab::Details,
            editor_state,
        };
        
        search.set_mock_results();
        search
    }
    
    /// Sync visibility with shared state
    pub fn sync_with_state(&mut self) {
        if let Some(ref state) = self.editor_state {
            let state_guard = state.read();
            self.visible = state_guard.global_search_visible;
        }
    }

    /// Get default recent items
    fn default_recent_items() -> Vec<RecentItem> {
        vec![
            RecentItem::new("Player Material".to_string(), "2m ago".to_string(), "🎨"),
            RecentItem::new("Physics Settings".to_string(), "yesterday".to_string(), "⚙️"),
            RecentItem::new("Enemy AI".to_string(), "3h ago".to_string(), "🧊"),
        ]
    }

    /// Toggle the visibility of the overlay
    pub fn toggle(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = !self.visible;
        // Sync with shared state if available
        if let Some(ref state) = self.editor_state {
            let mut state_guard = state.write();
            state_guard.set_global_search_visible(self.visible);
        }
    }

    /// Close the overlay
    pub fn close(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = false;
        // Sync with shared state if available
        if let Some(ref state) = self.editor_state {
            let mut state_guard = state.write();
            state_guard.set_global_search_visible(false);
        }
    }

    /// Check if the overlay is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set the search query and update prefix filter
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        let (prefix, filtered) = SearchPrefix::parse(&self.query);
        self.prefix = prefix;
        self.filtered_query = filtered.to_string();
    }

    /// Get the current search query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Set the filter category
    pub fn set_filter_category(&mut self, category: FilterCategory) {
        self.filter_category = category;
    }

    /// Set the active preview tab
    pub fn set_preview_tab(&mut self, tab: PreviewTab) {
        self.active_preview_tab = tab;
    }

    /// Select a result item
    pub fn select_result(&mut self, category: SearchPrefix, index: usize) {
        self.selected_index = Some((category, index));
    }

    /// Check if a result should be included based on current filter
    pub fn should_include_result(&self, result_type: SearchPrefix) -> bool {
        match self.prefix {
            SearchPrefix::None => true,
            _ => self.prefix == result_type,
        }
    }

    /// Add a search result
    pub fn add_result(&mut self, result: SearchResult) {
        if self.should_include_result(result.category) {
            self.results.add_result(result);
        }
    }

    /// Clear all search results
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// Set mock results for testing/demo
    pub fn set_mock_results(&mut self) {
        self.clear_results();
        
        // Top Results - mixed categories
        if self.should_include_result(SearchPrefix::Entity) {
            self.add_result(SearchResult::new(
                SearchPrefix::Entity,
                "Player".to_string(),
                "Main Scene · Entity".to_string(),
                "Entity".to_string(),
                "🧊",
            ));
        }
        
        if self.should_include_result(SearchPrefix::Script) {
            self.add_result(SearchResult::new(
                SearchPrefix::Script,
                "player_controller.rs".to_string(),
                "scripts/ · 4.2 KB".to_string(),
                "Script".to_string(),
                "📜",
            ));
        }
        
        if self.should_include_result(SearchPrefix::LogicNode) {
            self.add_result(SearchResult::new(
                SearchPrefix::LogicNode,
                "Dragon Slayer".to_string(),
                "Quests · Logic Node".to_string(),
                "Logic".to_string(),
                "🔀",
            ));
        }
        
        if self.should_include_result(SearchPrefix::Scene) {
            self.add_result(SearchResult::new(
                SearchPrefix::Scene,
                "Intro Cutscene".to_string(),
                "Timeline · 12s".to_string(),
                "Media".to_string(),
                "🎬",
            ));
        }
        
        // Entities section
        if self.should_include_result(SearchPrefix::Entity) {
            self.add_result(SearchResult::new(
                SearchPrefix::Entity,
                "Player_Base".to_string(),
                "Prefabs".to_string(),
                "Prefab".to_string(),
                "👤",
            ));
            self.add_result(SearchResult::new(
                SearchPrefix::Entity,
                "Enemy_Bot".to_string(),
                "AI/".to_string(),
                "Entity".to_string(),
                "🤖",
            ));
        }
    }

    /// Render the prefix hints row
    fn render_prefix_hints(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let hints = vec![
            (">", "Commands", "⚡"),
            ("@", "Entities", "🧊"),
            ("#", "Assets", "🖼️"),
            (":", "DB Query", "🗄️"),
            ("?", "AI Ask", "🤖"),
            ("!", "Scripts", "📜"),
            ("/", "Scene", "🎬"),
            ("%", "Logic Node", "🔀"),
        ];
        
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_wrap()
            .gap(px(8.0))
            .px(px(20.0))
            .py(px(12.0))
            .bg(rgb(0x222222))
            .border_b_1()
            .border_color(rgb(0x2a2a2a))
            .children(
                hints.into_iter().map(move |(prefix, label, icon)| {
                    let theme = theme.clone();
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(10.0))
                        .py(px(5.0))
                        .bg(rgb(0x2d2d2d))
                        .border_1()
                        .border_color(rgb(0x3f3f3f))
                        .rounded(px(30.0))
                        .text_size(px(12.0))
                        .text_color(rgb(0xcccccc))
                        .hover(|this| {
                            this.bg(rgb(0x3a3a4a))
                                .border_color(theme.colors.accent)
                                .text_color(rgb(0xffffff))
                        })
                        .child(icon)
                        .child(
                            div()
                                .child(format!("{} {}", prefix, label))
                        )
                })
            )
    }

    /// Render the filter bar
    fn render_filter_bar(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let categories = FilterCategory::all();
        let current_filter = self.filter_category;
        let theme = self.theme.clone();
        
        div()
            .flex()
            .flex_wrap()
            .gap(px(6.0))
            .px(px(20.0))
            .py(px(12.0))
            .bg(rgb(0x1e1e1e))
            .border_b_1()
            .border_color(rgb(0x2a2a2a))
            .children(
                categories.into_iter().map(move |category| {
                    let theme = theme.clone();
                    let is_active = current_filter == category;
                    
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .px(px(14.0))
                        .py(px(5.0))
                        .rounded(px(30.0))
                        .text_size(px(12.0))
                        .when(is_active, |this| {
                            this.bg(rgb(0x3a5f8a))
                                .border_1()
                                .border_color(theme.colors.accent)
                                .text_color(rgb(0xffffff))
                        })
                        .when(!is_active, |this| {
                            this.bg(rgb(0x2a2a2a))
                                .border_1()
                                .border_color(rgb(0x3a3a3a))
                                .text_color(rgb(0xbbbbbb))
                        })
                        .hover(|this| {
                            this.bg(if is_active { 
                                rgb(0x3a5f8a) 
                            } else { 
                                rgb(0x333333) 
                            })
                        })
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _event: &gpui::MouseDownEvent, _cx| {
                            this.set_filter_category(category);
                        }))
                        .child(category.icon())
                        .child(category.display_name())
                })
            )
    }

    /// Render a single result item
    fn render_result_item(&self, result: &SearchResult, is_selected: bool, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let result = result.clone();
        
        div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .px(px(12.0))
            .py(px(8.0))
            .rounded(px(10.0))
            .when(is_selected, |this| {
                this.bg(rgb(0x2d5a88))
                    .border_1()
                    .border_color(theme.colors.accent)
            })
            .when(!is_selected, |this| {
                this.hover(|this| {
                    this.bg(rgb(0x2d5a88))
                })
            })
            .cursor_pointer()
            .child(
                div()
                    .w(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(18.0))
                    .text_color(theme.colors.accent)
                    .child(result.icon)
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child(result.name)
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(0x888888))
                            .child(result.context)
                    )
            )
            .child(
                div()
                    .px(px(8.0))
                    .py(px(2.0))
                    .bg(rgb(0x3a3a3a))
                    .rounded(px(16.0))
                    .text_size(px(9.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xaaaaaa))
                    .child(result.badge)
            )
    }

    /// Render the results list
    fn render_results_list(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let _theme = self.theme.clone();
        let selected = self.selected_index;
        let has_results = !self.results.categories().is_empty();
        
        div()
            .flex_1()
            .overflow_hidden()
            .px(px(8.0))
            .pb(px(20.0))
            .children(
                if !has_results {
                    // Show placeholder when no results
                    vec![
                        div()
                            .p(px(20.0))
                            .child(
                                div()
                                    .text_color(rgb(0x888888))
                                    .text_size(px(13.0))
                                    .child("Type to search...")
                            )
                            .into_any_element()
                    ]
                } else {
                    // Get top results from all categories
                    let top_results: Vec<_> = SearchPrefix::all_categories()
                        .into_iter()
                        .flat_map(|cat| {
                            self.results.get_category(cat).iter().take(2).enumerate()
                                .map(move |(i, r)| (cat, i, r.clone()))
                        })
                        .take(4)
                        .collect();
                    
                    let mut elements: Vec<gpui::AnyElement> = Vec::new();
                    
                    // Top Results section header
                    if !top_results.is_empty() {
                        elements.push(
                            div()
                                .px(px(4.0))
                                .pt(px(16.0))
                                .pb(px(8.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .text_size(px(11.0))
                                        .text_color(rgb(0x6a6a6a))
                                        .font_weight(FontWeight::BOLD)
                                        .child("⚡")
                                        .child("TOP RESULTS")
                                )
                                .into_any_element()
                        );
                        
                        for (cat, i, result) in &top_results {
                            let is_selected = selected == Some((*cat, *i));
                            elements.push(
                                self.render_result_item(result, is_selected, cx)
                                    .into_any_element()
                            );
                        }
                    }
                    
                    // Category sections
                    for category in self.results.categories() {
                        let results = self.results.get_category(category);
                        if results.is_empty() {
                            continue;
                        }
                        
                        // Section header
                        elements.push(
                            div()
                                .px(px(4.0))
                                .pt(px(16.0))
                                .pb(px(8.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .text_size(px(11.0))
                                        .text_color(rgb(0x6a6a6a))
                                        .font_weight(FontWeight::BOLD)
                                        .child(category.icon())
                                        .child(category.display_name().to_uppercase())
                                )
                                .into_any_element()
                        );
                        
                        for (i, result) in results.iter().enumerate() {
                            let is_selected = selected == Some((category, i));
                            elements.push(
                                self.render_result_item(result, is_selected, cx)
                                    .into_any_element()
                            );
                        }
                    }
                    
                    elements
                }
            )
    }

    /// Render the recent items section
    fn render_recent_section(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let recent_items = self.recent_items.clone();
        
        div()
            .px(px(20.0))
            .py(px(16.0))
            .border_t_1()
            .border_color(rgb(0x2a2a2a))
            .bg(rgb(0x1b1b1b))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(px(11.0))
                    .text_color(rgb(0x6a6a6a))
                    .font_weight(FontWeight::BOLD)
                    .mb(px(12.0))
                    .child("🕐")
                    .child("RECENT")
            )
            .children(
                recent_items.into_iter().map(move |item| {
                    let theme = theme.clone();
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(12.0))
                        .py(px(8.0))
                        .rounded(px(10.0))
                        .hover(|this| this.bg(rgb(0x3a5f8a)))
                        .cursor_pointer()
                        .child(
                            div()
                                .w(px(20.0))
                                .text_color(theme.colors.accent)
                                .text_size(px(14.0))
                                .child(item.icon)
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(13.0))
                                .text_color(rgb(0xcccccc))
                                .child(item.name)
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(rgb(0x888888))
                                .child(item.context)
                        )
                })
            )
    }

    /// Render the preview panel
    fn render_preview_panel(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        let preview = self.preview_data.clone();
        let active_tab = self.active_preview_tab;
        
        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            // Preview header
            .child(
                div()
                    .px(px(24.0))
                    .pt(px(20.0))
                    .pb(px(12.0))
                    .border_b_1()
                    .border_color(rgb(0x2a2a2a))
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .text_size(px(16.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xcccccc))
                            .child(
                                div()
                                    .text_color(theme.colors.accent)
                                    .child("🧊")
                            )
                            .child(format!("{} (Entity)", preview.title))
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .px(px(14.0))
                                    .py(px(6.0))
                                    .bg(rgb(0x2a2a2a))
                                    .border_1()
                                    .border_color(rgb(0x3a3a3a))
                                    .rounded(px(30.0))
                                    .text_size(px(12.0))
                                    .text_color(rgb(0xcccccc))
                                    .hover(|this| {
                                        this.bg(rgb(0x3a5f8a))
                                            .text_color(rgb(0xffffff))
                                    })
                                    .cursor_pointer()
                                    .child("✏️")
                                    .child("Edit")
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .px(px(14.0))
                                    .py(px(6.0))
                                    .bg(rgb(0x2a2a2a))
                                    .border_1()
                                    .border_color(rgb(0x3a3a3a))
                                    .rounded(px(30.0))
                                    .text_size(px(12.0))
                                    .text_color(rgb(0xcccccc))
                                    .hover(|this| {
                                        this.bg(rgb(0x3a5f8a))
                                            .text_color(rgb(0xffffff))
                                    })
                                    .cursor_pointer()
                                    .child("📁")
                                    .child("Show in Box")
                            )
                    )
            )
            // Preview content
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .px(px(24.0))
                    .py(px(20.0))
                    // Preview placeholder
                    .child(
                        div()
                            .h(px(200.0))
                            .bg(rgb(0x2a2a2a))
                            .rounded(px(16.0))
                            .border_1()
                            .border_color(rgb(0x3a3a3a))
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(12.0))
                            .text_color(rgb(0x6a6a6a))
                            .child(
                                div()
                                    .text_size(px(48.0))
                                    .text_color(theme.colors.accent)
                                    .child("🧊")
                            )
                            .child("3D Preview")
                            .mb(px(24.0))
                    )
                    // Metadata grid
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(16.0))
                            .children([
                                ("Type", preview.item_type),
                                ("Last Modified", preview.last_modified),
                                ("Components", preview.components),
                                ("Size", preview.size),
                            ].into_iter().map(|(label, value)| {
                                div()
                                    .bg(rgb(0x2a2a2a))
                                    .rounded(px(12.0))
                                    .p(px(16.0))
                                    .border_1()
                                    .border_color(rgb(0x3a3a3a))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(rgb(0x888888))
                                            .font_weight(FontWeight::BOLD)
                                            .mb(px(6.0))
                                            .child(label.to_uppercase())
                                    )
                                    .child(
                                        div()
                                            .text_size(px(15.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xeeeeee))
                                            .child(value)
                                    )
                            }))
                    )
                    // Preview tabs
                    .child(
                        div()
                            .flex()
                            .gap(px(4.0))
                            .mt(px(24.0))
                            .mb(px(12.0))
                            .border_b_1()
                            .border_color(rgb(0x2a2a2a))
                            .children(
                                [PreviewTab::Details, PreviewTab::Code, PreviewTab::Dependencies]
                                    .into_iter()
                                    .map(|tab| {
                                        let is_active = active_tab == tab;
                                        div()
                                            .px(px(16.0))
                                            .py(px(8.0))
                                            .text_size(px(12.0))
                                            .when(is_active, |this| {
                                                this.text_color(theme.colors.accent)
                                                    .border_b_2()
                                                    .border_color(theme.colors.accent)
                                            })
                                            .when(!is_active, |this| {
                                                this.text_color(rgb(0xaaaaaa))
                                                    .cursor_pointer()
                                                    .hover(|this| this.text_color(rgb(0xcccccc)))
                                            })
                                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _event: &gpui::MouseDownEvent, _cx| {
                                                this.set_preview_tab(tab);
                                            }))
                                            .child(tab.display_name())
                                    })
                            )
                    )
                    // Detail panel
                    .child(
                        div()
                            .bg(rgb(0x2a2a2a))
                            .rounded(px(12.0))
                            .p(px(16.0))
                            .border_1()
                            .border_color(rgb(0x3a3a3a))
                            .text_size(px(13.0))
                            .text_color(rgb(0xcccccc))
                            .min_h(px(120.0))
                            .child(
                                div()
                                    .child(preview.json_content)
                            )
                    )
            )
            // Footer with keyboard hints
            .child(
                div()
                    .px(px(24.0))
                    .py(px(16.0))
                    .bg(rgb(0x1a1a1a))
                    .border_t_1()
                    .border_color(rgb(0x2a2a2a))
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .gap(px(20.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .text_size(px(12.0))
                                    .text_color(rgb(0x6a6a6a))
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(3.0))
                                            .bg(rgb(0x2a2a2a))
                                            .border_1()
                                            .border_color(rgb(0x4a4a4a))
                                            .rounded(px(4.0))
                                            .text_size(px(11.0))
                                            .text_color(rgb(0xcccccc))
                                            .child("↑")
                                    )
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(3.0))
                                            .bg(rgb(0x2a2a2a))
                                            .border_1()
                                            .border_color(rgb(0x4a4a4a))
                                            .rounded(px(4.0))
                                            .text_size(px(11.0))
                                            .text_color(rgb(0xcccccc))
                                            .child("↓")
                                    )
                                    .child("Navigate")
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .text_size(px(12.0))
                                    .text_color(rgb(0x6a6a6a))
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(3.0))
                                            .bg(rgb(0x2a2a2a))
                                            .border_1()
                                            .border_color(rgb(0x4a4a4a))
                                            .rounded(px(4.0))
                                            .text_size(px(11.0))
                                            .text_color(rgb(0xcccccc))
                                            .child("↵")
                                    )
                                    .child("Open")
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .text_size(px(12.0))
                                    .text_color(rgb(0x6a6a6a))
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(3.0))
                                            .bg(rgb(0x2a2a2a))
                                            .border_1()
                                            .border_color(rgb(0x4a4a4a))
                                            .rounded(px(4.0))
                                            .text_size(px(11.0))
                                            .text_color(rgb(0xcccccc))
                                            .child("Tab")
                                    )
                                    .child("Complete")
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .text_size(px(12.0))
                                    .text_color(rgb(0x6a6a6a))
                                    .child(
                                        div()
                                            .px(px(6.0))
                                            .py(px(3.0))
                                            .bg(rgb(0x2a2a2a))
                                            .border_1()
                                            .border_color(rgb(0x4a4a4a))
                                            .rounded(px(4.0))
                                            .text_size(px(11.0))
                                            .text_color(rgb(0xcccccc))
                                            .child("⌘K")
                                    )
                                    .child("Preview")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .text_size(px(12.0))
                            .text_color(theme.colors.accent)
                            .cursor_pointer()
                            .hover(|this| this.text_color(rgb(0xffffff)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event: &gpui::MouseDownEvent, cx| {
                                this.close(cx);
                            }))
                            .child("✕")
                            .child("Esc")
                    )
            )
    }
}

impl Render for GlobalSearch {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        // Sync with shared state before rendering
        // This allows external commands (keyboard shortcuts) to toggle visibility
        self.sync_with_state();
        
        if !self.visible {
            return div().into_any_element();
        }

        let backdrop_color = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.9,
        };
        
        let theme = self.theme.clone();

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(backdrop_color)
            .flex()
            .justify_center()
            .pt(px(60.0)) // 5vh approximation
            .child(
                div()
                    .w(px(1200.0)) // 1400px max, scaled for reasonable display
                    .h(px(700.0))  // 90vh approximation
                    .max_w(px(1200.0))
                    .bg(rgb(0x1a1a1a))
                    .rounded(px(28.0))
                    .border_1()
                    .border_color(rgb(0x3a3a3a))
                    .shadow_lg()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    // Left column (38%)
                    .child(
                        div()
                            .w(px(456.0)) // 38% of 1200px
                            .h_full()
                            .bg(rgb(0x1e1e1e))
                            .border_r_1()
                            .border_color(rgb(0x2a2a2a))
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            // Left header with search input
                            .child(
                                div()
                                    .px(px(20.0))
                                    .pt(px(16.0))
                                    .pb(px(8.0))
                                    .border_b_1()
                                    .border_color(rgb(0x2a2a2a))
                                    .child(
                                        div()
                                            .flex()
                                            .justify_between()
                                            .items_center()
                                            .mb(px(12.0))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(8.0))
                                                    .text_size(px(16.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xcccccc))
                                                    .child(
                                                        div()
                                                            .text_color(theme.colors.accent)
                                                            .child("🔍")
                                                    )
                                                    .child("Global Search")
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .w(px(40.0))
                                                    .h(px(40.0))
                                                    .rounded(px(20.0))
                                                    .text_size(px(24.0))
                                                    .text_color(rgb(0xaaaaaa))
                                                    .cursor_pointer()
                                                    .hover(|this| {
                                                        this.bg(rgb(0x3a3a3a))
                                                            .text_color(rgb(0xffffff))
                                                    })
                                                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _event: &gpui::MouseDownEvent, cx| {
                                                        this.close(cx);
                                                    }))
                                                    .child("✕")
                                            )
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .bg(rgb(0x2a2a2a))
                                            .rounded(px(40.0))
                                            .px(px(16.0))
                                            .py(px(4.0))
                                            .border_1()
                                            .border_color(rgb(0x3a3a3a))
                                            .child(
                                                div()
                                                    .text_color(theme.colors.accent)
                                                    .text_size(px(20.0))
                                                    .mr(px(12.0))
                                                    .child("🔍")
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .child(
                                                        div()
                                                            .text_size(px(18.0))
                                                            .text_color(rgb(0xffffff))
                                                            .child(if self.query.is_empty() {
                                                                "Search everything...".to_string()
                                                            } else {
                                                                self.query.clone()
                                                            })
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .px(px(10.0))
                                                    .py(px(4.0))
                                                    .bg(rgb(0x3a3a3a))
                                                    .rounded(px(20.0))
                                                    .text_size(px(13.0))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme.colors.accent)
                                                    .child("⌘P")
                                            )
                                    )
                            )
                            // Prefix hints
                            .child(self.render_prefix_hints(cx))
                            // Filter bar
                            .child(self.render_filter_bar(cx))
                            // Results list
                            .child(self.render_results_list(cx))
                            // Recent section
                            .child(self.render_recent_section(cx))
                    )
                    // Right column (62%)
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .bg(rgb(0x202020))
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(self.render_preview_panel(cx))
                    )
            )
            .into_any_element()
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
        let (prefix, query) = SearchPrefix::parse(">save");
        assert_eq!(prefix, SearchPrefix::Command);
        assert_eq!(query, "save");
    }

    #[test]
    fn test_prefix_parse_symbol() {
        let (prefix, query) = SearchPrefix::parse(":function");
        assert_eq!(prefix, SearchPrefix::DbQuery);
        assert_eq!(query, "function");
    }

    #[test]
    fn test_prefix_parse_none() {
        let (prefix, query) = SearchPrefix::parse("search");
        assert_eq!(prefix, SearchPrefix::None);
        assert_eq!(query, "search");
    }

    #[test]
    fn test_prefix_display_names() {
        assert_eq!(SearchPrefix::Entity.display_name(), "Entities");
        assert_eq!(SearchPrefix::Asset.display_name(), "Assets");
        assert_eq!(SearchPrefix::Command.display_name(), "Commands");
        assert_eq!(SearchPrefix::None.display_name(), "All");
    }

    #[test]
    fn test_filter_category_display() {
        assert_eq!(FilterCategory::All.display_name(), "All");
        assert_eq!(FilterCategory::Entities.display_name(), "Entities");
        assert_eq!(FilterCategory::Scripts.display_name(), "Scripts");
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            "Main Scene".to_string(),
            "Entity".to_string(),
            "🧊",
        );
        
        assert_eq!(result.category, SearchPrefix::Entity);
        assert_eq!(result.name, "Player");
        assert_eq!(result.context, "Main Scene");
        assert_eq!(result.badge, "Entity");
    }

    #[test]
    fn test_grouped_results_add_and_get() {
        let mut groups = GroupedResults::new();
        
        groups.add_result(SearchResult::new(
            SearchPrefix::Entity,
            "Player".to_string(),
            "Scene".to_string(),
            "Entity".to_string(),
            "🧊",
        ));
        groups.add_result(SearchResult::new(
            SearchPrefix::Asset,
            "texture.png".to_string(),
            "assets/".to_string(),
            "Asset".to_string(),
            "🖼️",
        ));
        
        assert_eq!(groups.total_count(), 2);
        assert_eq!(groups.get_category(SearchPrefix::Entity).len(), 1);
        assert_eq!(groups.get_category(SearchPrefix::Asset).len(), 1);
    }

    #[test]
    fn test_recent_item_creation() {
        let item = RecentItem::new(
            "Test Item".to_string(),
            "2m ago".to_string(),
            "🧊",
        );
        
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.context, "2m ago");
        assert_eq!(item.icon, "🧊");
    }

    #[test]
    fn test_preview_tab_display() {
        assert_eq!(PreviewTab::Details.display_name(), "Details");
        assert_eq!(PreviewTab::Code.display_name(), "Code");
        assert_eq!(PreviewTab::Dependencies.display_name(), "Dependencies");
    }
}
