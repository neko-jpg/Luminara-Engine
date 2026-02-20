//! Account Panel Component
//!
//! A full-screen account overlay similar to the HTML prototype.
//! Contains profile card on the left and settings on the right.

use crate::ui::theme::Theme;
use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, InteractiveElement,
    ViewContext, View, VisualContext,
};
use std::sync::Arc;

/// Settings category chip
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountSettingsCategory {
    /// General settings
    General,
    /// Editor settings
    Editor,
    /// Theme settings
    Theme,
    /// Keyboard settings
    Keyboard,
    /// Extensions settings
    Extensions,
    /// Account settings
    Account,
}

impl AccountSettingsCategory {
    /// Get all categories
    pub fn all() -> Vec<(AccountSettingsCategory, &'static str)> {
        vec![
            (AccountSettingsCategory::General, "一般"),
            (AccountSettingsCategory::Editor, "エディタ"),
            (AccountSettingsCategory::Theme, "テーマ"),
            (AccountSettingsCategory::Keyboard, "キーボード"),
            (AccountSettingsCategory::Extensions, "拡張機能"),
            (AccountSettingsCategory::Account, "アカウント"),
        ]
    }
}

/// User profile information
#[derive(Debug, Clone)]
pub struct UserProfile {
    /// Display name
    pub display_name: String,
    /// Email address
    pub email: String,
    /// Plan type
    pub plan: String,
    /// Number of projects
    pub project_count: u32,
    /// Number of assets
    pub asset_count: String,
    /// Team size
    pub team_size: u32,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            display_name: "田中 太郎".to_string(),
            email: "taro.tanaka@luminara.dev".to_string(),
            plan: "Pro プラン".to_string(),
            project_count: 12,
            asset_count: "2.4k".to_string(),
            team_size: 32,
        }
    }
}

/// The Account Panel component
pub struct AccountPanel {
    /// User profile information
    profile: UserProfile,
    /// Current active settings category
    active_category: AccountSettingsCategory,
    /// Theme for styling
    #[allow(dead_code)]
    theme: Arc<Theme>,
    /// Whether the panel is visible
    visible: bool,
}

impl AccountPanel {
    /// Create a new AccountPanel
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            profile: UserProfile::default(),
            active_category: AccountSettingsCategory::General,
            theme,
            visible: false,
        }
    }

    /// Create as a GPUI View
    pub fn view(theme: Arc<Theme>, cx: &mut gpui::WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self::new(theme))
    }

    /// Show the account panel
    pub fn show(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = true;
    }

    /// Hide the account panel
    pub fn hide(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self, _cx: &mut ViewContext<Self>) {
        self.visible = !self.visible;
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set active category
    pub fn set_category(&mut self, category: AccountSettingsCategory, _cx: &mut ViewContext<Self>) {
        self.active_category = category;
    }

    /// Render the left column with profile
    fn render_left_column(&self, _cx: &ViewContext<Self>) -> impl IntoElement {
        let profile = self.profile.clone();

        div()
            .w(px(320.0))
            .h_full()
            .bg(self.theme.colors.background)
            .border_r_1()
            .border_color(self.theme.colors.surface)
            .flex()
            .flex_col()
            // Header
            .child(
                div()
                    .p(px(20.0))
                    .pb(px(16.0))
                    .border_b_1()
                    .border_color(self.theme.colors.surface)
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(10.0))
                                    .child(
                                        gpui::svg()
                                            .path("icons/user-circle.svg")
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .text_color(self.theme.colors.accent)
                                    )
                                    .child(
                                        div()
                                            .text_size(px(18.0))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(self.theme.colors.text)
                                            .child("アカウント")
                                    )
                            )
                            .child(
                                // Close button
                                div()
                                    .w(px(40.0))
                                    .h(px(40.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .cursor_pointer()
                                    .text_color(self.theme.colors.text_secondary)
                                    .text_size(px(20.0))
                                    .hover(|this| this.bg(self.theme.colors.surface_active).text_color(gpui::rgb(0xffffff)))
                                    .child("×")
                            )
                    )
                    // Profile Card
                    .child(
                        div()
                            .mt(px(16.0))
                            .bg(self.theme.colors.surface)
                            .rounded(px(24.0))
                            .p(px(24.0))
                            .border_1()
                            .border_color(self.theme.colors.surface_active)
                            .flex()
                            .flex_col()
                            .items_center()
                            .child(
                                gpui::svg()
                                    .path("icons/user-circle.svg")
                                    .w(px(64.0))
                                    .h(px(64.0))
                                    .text_color(self.theme.colors.accent)
                                    .mb(px(12.0))
                            )
                            .child(
                                div()
                                    .text_size(px(20.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(gpui::rgb(0xffffff))
                                    .child(profile.display_name.clone())
                            )
                            .child(
                                div()
                                    .mt(px(4.0))
                                    .text_size(px(13.0))
                                    .text_color(self.theme.colors.accent)
                                    .child(profile.email.clone())
                            )
                            .child(
                                div()
                                    .mt(px(12.0))
                                    .mb(px(20.0))
                                    .bg(self.theme.colors.toolbar_active)
                                    .rounded(px(30.0))
                                    .px(px(16.0))
                                    .py(px(4.0))
                                    .text_size(px(12.0))
                                    .text_color(gpui::rgb(0xffffff))
                                    .border_1()
                                    .border_color(self.theme.colors.accent)
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        gpui::svg()
                                            .path("icons/crown.svg")
                                            .w(px(12.0))
                                            .h(px(12.0))
                                            .text_color(gpui::rgb(0xffffff))
                                    )
                                    .child(profile.plan.clone())
                            )
                            // Stats
                            .child(
                                div()
                                    .w_full()
                                    .pt(px(16.0))
                                    .border_t_1()
                                    .border_color(self.theme.colors.surface_active)
                                    .flex()
                                    .justify_around()
                                    .child(self.render_stat_item(&profile.project_count.to_string(), "プロジェクト"))
                                    .child(self.render_stat_item(&profile.asset_count, "アセット"))
                                    .child(self.render_stat_item(&profile.team_size.to_string(), "チーム"))
                            )
                            // Actions
                            .child(
                                div()
                                    .mt(px(20.0))
                                    .flex()
                                    .gap(px(12.0))
                                    .child(self.render_profile_action_button("編集", "icons/pencil.svg"))
                                    .child(self.render_profile_action_button("サインアウト", "icons/sign-out.svg"))
                            )
                    )
            )
            // Links
            .child(
                div()
                    .p(px(20.0))
                    .mt_auto()
                    .border_t_1()
                    .border_color(self.theme.colors.surface)
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(self.render_link_item("プロフィール管理", "icons/id-card.svg"))
                    .child(self.render_link_item("課金・支払い", "icons/credit-card.svg"))
                    .child(self.render_link_item("セキュリティ", "icons/shield.svg"))
                    .child(self.render_link_item("チーム設定", "icons/users.svg"))
            )
    }

    fn render_stat_item(&self, value: &str, label: &str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .child(
                div()
                    .text_size(px(18.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(self.theme.colors.accent)
                    .child(value.to_string())
            )
            .child(
                div()
                    .mt(px(2.0))
                    .text_size(px(11.0))
                    .text_color(self.theme.colors.text_secondary)
                    .child(label.to_string())
            )
    }

    fn render_profile_action_button(&self, label: &str, _icon: &str) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(6.0))
            .bg(self.theme.colors.surface_active)
            .border_1()
            .border_color(self.theme.colors.node_border)
            .rounded(px(40.0))
            .py(px(10.0))
            .text_size(px(13.0))
            .text_color(self.theme.colors.text)
            .cursor_pointer()
            .hover(|this| this.bg(self.theme.colors.toolbar_active).text_color(gpui::rgb(0xffffff)))
            .child(label.to_string())
    }

    fn render_link_item(&self, label: &str, icon: &str) -> impl IntoElement {
        let icon_path = icon.to_string();
        let label_text = label.to_string();
        div()
            .flex()
            .items_center()
            .gap(px(14.0))
            .px(px(16.0))
            .py(px(12.0))
            .rounded(px(14.0))
            .text_color(self.theme.colors.text_secondary)
            .cursor_pointer()
            .hover(|this| this.bg(self.theme.colors.toolbar_active).text_color(gpui::rgb(0xffffff)))
            .child(
                gpui::svg()
                    .path(icon_path)
                    .w(px(18.0))
                    .h(px(18.0))
                    .text_color(self.theme.colors.accent)
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(14.0))
                    .child(label_text)
            )
            .child(
                gpui::svg()
                    .path("icons/chevron-right.svg")
                    .w(px(12.0))
                    .h(px(12.0))
                    .text_color(self.theme.colors.text_secondary)
            )
    }

    /// Render the right column with settings
    fn render_right_column(&self, _cx: &ViewContext<Self>) -> impl IntoElement {
        let categories = AccountSettingsCategory::all();
        let active = self.active_category;

        div()
            .flex_1()
            .h_full()
            .bg(self.theme.colors.background)
            .flex()
            .flex_col()
            // Header
            .child(
                div()
                    .p(px(20.0))
                    .pb(px(12.0))
                    .border_b_1()
                    .border_color(self.theme.colors.surface)
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                gpui::svg()
                                    .path("icons/sliders.svg")
                                    .w(px(16.0))
                                    .h(px(16.0))
                                    .text_color(self.theme.colors.accent)
                            )
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(self.theme.colors.text_secondary)
                                    .child("設定エディタ")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .child(self.render_header_button("リセット", "icons/undo.svg"))
                            .child(self.render_header_button("保存", "icons/save.svg"))
                    )
            )
            // Category chips
            .child(
                div()
                    .flex()
                    .gap(px(12.0))
                    .p(px(16.0))
                    .pb(px(8.0))
                    .border_b_1()
                    .border_color(self.theme.colors.surface)
                    .children(categories.into_iter().map(|(cat, name)| {
                        let is_active = active == cat;
                        self.render_category_chip(name, is_active)
                    }))
            )
            // Settings list
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .p(px(24.0))
                    .child(self.render_settings_list())
            )
            // Footer
            .child(
                div()
                    .p(px(16.0))
                    .px(px(24.0))
                    .bg(gpui::rgb(0x1a1a1a))
                    .border_t_1()
                    .border_color(self.theme.colors.surface)
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .gap(px(20.0))
                            .text_size(px(12.0))
                            .text_color(self.theme.colors.text_secondary)
                            .child(self.render_key_hint("↑↓", "移動"))
                            .child(self.render_key_hint("↵", "開く"))
                            .child(self.render_key_hint("⌘S", "保存"))
                            .child(self.render_key_hint("⌘,", "設定"))
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .text_size(px(12.0))
                            .text_color(self.theme.colors.accent)
                            .child(
                                gpui::svg()
                                    .path("icons/times.svg")
                                    .w(px(12.0))
                                    .h(px(12.0))
                                    .text_color(self.theme.colors.accent)
                            )
                            .child("Esc で閉じる")
                    )
            )
    }

    fn render_header_button(&self, label: &str, _icon: &str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .bg(self.theme.colors.surface)
            .border_1()
            .border_color(self.theme.colors.surface_active)
            .rounded(px(30.0))
            .px(px(14.0))
            .py(px(6.0))
            .text_size(px(12.0))
            .text_color(self.theme.colors.text_secondary)
            .cursor_pointer()
            .hover(|this| this.bg(self.theme.colors.toolbar_active).text_color(gpui::rgb(0xffffff)))
            .child(label.to_string())
    }

    fn render_category_chip(&self, label: &str, is_active: bool) -> impl IntoElement {
        let (bg_color, text_color, border_color) = if is_active {
            (self.theme.colors.toolbar_active, gpui::rgb(0xffffff), self.theme.colors.accent)
        } else {
            (self.theme.colors.surface, gpui::rgb(0xbbbbbb), self.theme.colors.surface_active)
        };

        div()
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .rounded(px(30.0))
            .px(px(18.0))
            .py(px(6.0))
            .text_size(px(12.0))
            .text_color(text_color)
            .cursor_pointer()
            .hover(|this| {
                if !is_active {
                    this.bg(self.theme.colors.surface_active)
                } else {
                    this
                }
            })
            .child(label.to_string())
    }

    fn render_key_hint(&self, key: &str, label: &str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                div()
                    .bg(self.theme.colors.surface)
                    .border_1()
                    .border_color(self.theme.colors.node_border)
                    .rounded(px(4.0))
                    .px(px(6.0))
                    .py(px(3.0))
                    .text_size(px(11.0))
                    .text_color(self.theme.colors.text_secondary)
                    .child(key.to_string())
            )
            .child(label.to_string())
    }

    fn render_settings_list(&self) -> impl IntoElement {
        // Sample settings based on the HTML prototype
        let settings = vec![
            ("言語 / Language", "エディタの表示言語を変更します", "日本語", "icons/globe.svg"),
            ("フォントサイズ", "コードエディタの文字サイズ", "14px", "icons/text-height.svg"),
            ("テーマ カラー", "ダーク / ライト / ハイコントラスト", "ダークモード (Luminara Dark+)", "icons/palette.svg"),
            ("自動保存", "編集中のファイルを自動的に保存", "checkbox:checked", ""),
            ("AI 提案", "コード補完にAIを利用する", "checkbox:checked", ""),
        ];

        div()
            .flex()
            .flex_col()
            .children(settings.into_iter().map(|(name, desc, value, icon)| {
                self.render_setting_item(name, desc, value, icon)
            }))
            // Danger zone
            .child(
                div()
                    .mt(px(24.0))
                    .bg(gpui::rgb(0x2a1e1e))
                    .rounded(px(16.0))
                    .p(px(16.0))
                    .border_1()
                    .border_color(gpui::rgb(0x8a4a4a))
                    .child(self.render_setting_item_danger("アカウント削除", "この操作は元に戻せません", "削除"))
            )
    }

    fn render_setting_item(&self, name: &str, desc: &str, value: &str, icon: &str) -> impl IntoElement {
        let control = if value.starts_with("checkbox:") {
            let checked = value.contains("checked");
            self.render_toggle_control(checked).into_any_element()
        } else {
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .bg(self.theme.colors.surface)
                .border_1()
                .border_color(self.theme.colors.surface_active)
                .rounded(px(20.0))
                .px(px(16.0))
                .py(px(6.0))
                .text_size(px(12.0))
                .text_color(self.theme.colors.text)
                .children(if !icon.is_empty() {
                    vec![
                        gpui::svg()
                            .path(icon.to_string())
                            .w(px(12.0))
                            .h(px(12.0))
                            .text_color(self.theme.colors.accent)
                            .into_any_element()
                    ]
                } else {
                    vec![]
                })
                .child(value.to_string())
                .into_any_element()
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .py(px(14.0))
            .border_b_1()
            .border_color(self.theme.colors.surface)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(self.theme.colors.text)
                            .child(name.to_string())
                    )
                    .child(
                        div()
                            .mt(px(2.0))
                            .text_size(px(11.0))
                            .text_color(gpui::rgb(0x888888))
                            .child(desc.to_string())
                    )
            )
            .child(control)
    }

    fn render_setting_item_danger(&self, name: &str, desc: &str, action: &str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(self.theme.colors.error)
                            .child(name.to_string())
                    )
                    .child(
                        div()
                            .mt(px(2.0))
                            .text_size(px(11.0))
                            .text_color(gpui::rgb(0x888888))
                            .child(desc.to_string())
                    )
            )
            .child(
                div()
                    .bg(gpui::rgb(0x8a4a4a))
                    .border_1()
                    .border_color(self.theme.colors.error)
                    .rounded(px(20.0))
                    .px(px(16.0))
                    .py(px(6.0))
                    .text_size(px(12.0))
                    .text_color(gpui::rgb(0xffffff))
                    .cursor_pointer()
                    .hover(|this| this.bg(gpui::rgb(0xaa5a5a)))
                    .child(action.to_string())
            )
    }

    fn render_toggle_control(&self, active: bool) -> impl IntoElement {
        let bg_color = if active {
            self.theme.colors.accent
        } else {
            self.theme.colors.node_border
        };

        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                div()
                    .w(px(40.0))
                    .h(px(20.0))
                    .rounded(px(20.0))
                    .bg(bg_color)
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top(px(2.0))
                            .left(if active { px(20.0) } else { px(2.0) })
                            .w(px(16.0))
                            .h(px(16.0))
                            .rounded_full()
                            .bg(gpui::rgb(0xffffff))
                    )
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(self.theme.colors.text)
                    .child(if active { "有効" } else { "無効" })
            )
    }
}

impl Render for AccountPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.clone();
        if !self.visible {
            return div().into_any_element();
        }

        // Full-screen overlay
        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .w_full()
            .h_full()
            .bg(gpui::rgba(0xE6000000))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w_full()
                    .h_full()
                    .bg(gpui::rgb(0x1a1a1a))
                    .flex()
                    .flex_row()
                    .child(self.render_left_column(cx))
                    .child(self.render_right_column(cx))
            )
            .into_any_element()
    }
}
