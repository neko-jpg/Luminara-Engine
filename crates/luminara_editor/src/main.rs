//! Luminara Editor - Main Entry Point

use vizia::prelude::*;
use luminara_editor::ui::{
    activity_bar::ActivityBar,
    icons,
    panels::{
        asset_vault, backend_ai, director, extensions, global_search, logic_graph, scene_builder,
    },
    state::{EditorData, EditorEvent, EditorState, PanelType},
    theme::STYLE,
};

#[derive(Lens)]
pub struct MainState {
    // Only for window properties if needed, mostly using EditorState
}

impl Model for MainState {}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        // Load global styles
        cx.add_stylesheet(STYLE).expect("Failed to add stylesheet");

        // Initialize State
        EditorState::default().build(cx);
        MainState {}.build(cx);

        // Main Layout
        VStack::new(cx, |cx| {
            // --- Menu Bar ---
            HStack::new(cx, |cx| {
                for item in ["File", "Edit", "Assets", "GameObject", "Component", "Window", "AI", "Help"] {
                    Label::new(cx, item)
                        .class("menu-item")
                        .color(Color::rgb(0xdd, 0xdd, 0xdd))
                        .font_size(12.0)
                        .padding_left(Pixels(8.0))
                        .padding_right(Pixels(8.0))
                        .padding_top(Pixels(4.0))
                        .padding_bottom(Pixels(4.0))
                        .on_hover(|cx| {
                            cx.background_color(Color::rgb(0x4a, 0x4a, 0x4a));
                        });
                }
            })
            .height(Pixels(28.0))
            .background_color(Color::rgb(0x2d, 0x2d, 0x2d))
            .border_bottom_width(Pixels(1.0))
            .border_color(Color::rgb(0x3a, 0x3a, 0x3a));

            // --- Main Content Area ---
            HStack::new(cx, |cx| {
                // Activity Bar
                ActivityBar::new(cx);

                // Panel Content
                VStack::new(cx, |cx| {
                    Binding::new(cx, EditorState::active_panel, |cx, panel| {
                        let p = panel.get(cx);
                        match p {
                            PanelType::GlobalSearch => global_search::build(cx),
                            PanelType::SceneBuilder => scene_builder::build(cx),
                            PanelType::LogicGraph => logic_graph::build(cx),
                            PanelType::Director => director::build(cx),
                            PanelType::BackendAI => backend_ai::build(cx),
                            PanelType::AssetVault => asset_vault::build(cx),
                            PanelType::Extensions => extensions::build(cx),
                        }
                    });
                })
                .class("center-panel");
            })
            .child_space(Stretch(1.0)); // Expand to fill

            // --- Status Bar ---
            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Svg::new(cx, icons::ICON_CUBES).class("icon-tiny");
                    Label::new(cx, "120 FPS").font_size(11.0);
                }).col_between(Pixels(4.0));

                HStack::new(cx, |cx| {
                    Svg::new(cx, icons::ICON_CUBES).class("icon-tiny");
                    Label::new(cx, "32 Entities").font_size(11.0);
                }).col_between(Pixels(4.0));

                Element::new(cx).child_space(Stretch(1.0));

                Label::new(cx, "Luminara Engine v0.1.0")
                    .font_size(11.0)
                    .color(Color::rgb(0xaa, 0xaa, 0xaa));
            })
            .height(Pixels(24.0))
            .background_color(Color::rgb(0x2a, 0x2a, 0x2a))
            .padding_left(Pixels(12.0))
            .padding_right(Pixels(12.0))
            .align_items(Align::Center)
            .col_between(Pixels(16.0));
        });
    })
    .title("Luminara Editor")
    .inner_size((1400, 900))
    .run()
}
