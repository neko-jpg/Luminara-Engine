use vizia::prelude::*;
use crate::ui::icons::*;
use crate::ui::state::{EditorEvent, EditorState, PanelType, ActivityItem};

pub struct ActivityBar;

impl ActivityBar {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self.build(cx, |cx| {
            // Container for the bar
            VStack::new(cx, |cx| {
                // Iterate over items
                List::new(cx, EditorState::activity_bar_items, |cx, idx, item| {
                    let item_clone = item.get(cx).clone();
                    match item_clone {
                        ActivityItem::Single(panel) => {
                            // Single Item
                            VStack::new(cx, |cx| {
                                 Svg::new(cx, get_icon_for_panel(panel))
                                    .class("icon")
                                    .width(Pixels(24.0))
                                    .height(Pixels(24.0));
                            })
                            .class("activity-item")
                            .toggle_class("active", EditorState::active_panel.map(move |p| *p == panel))
                            .on_press(move |ex| ex.emit(EditorEvent::SetPanel(panel)))
                            .padding_top(Stretch(1.0))
                            .padding_bottom(Stretch(1.0))
                            .padding_left(Stretch(1.0))
                            .padding_right(Stretch(1.0));
                        }
                        ActivityItem::Folder(_name, _items) => {
                            // Folder Item
                            VStack::new(cx, |cx| {
                                Svg::new(cx, ICON_FOLDER)
                                    .class("icon")
                                    .width(Pixels(24.0))
                                    .height(Pixels(24.0));

                                // Badge
                                Label::new(cx, "2") // Mock count
                                    .class("folder-badge")
                                    .position_type(PositionType::SelfDirected)
                                    .top(Pixels(2.0))
                                    .right(Pixels(2.0));
                            })
                            .class("activity-item")
                            .class("is-folder")
                            .padding_top(Stretch(1.0))
                            .padding_bottom(Stretch(1.0))
                            .padding_left(Stretch(1.0))
                            .padding_right(Stretch(1.0));
                        }
                    }
                });

                // Spacer
                Element::new(cx).height(Stretch(1.0));

                // Settings
                 VStack::new(cx, |cx| {
                    Svg::new(cx, ICON_SETTINGS)
                        .class("icon")
                        .width(Pixels(24.0))
                        .height(Pixels(24.0));
                })
                .class("activity-item")
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0))
                .padding_left(Stretch(1.0))
                .padding_right(Stretch(1.0));

                // User
                 VStack::new(cx, |cx| {
                    Svg::new(cx, ICON_USER)
                        .class("icon")
                        .width(Pixels(24.0))
                        .height(Pixels(24.0));
                })
                .class("activity-item")
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0))
                .padding_left(Stretch(1.0))
                .padding_right(Stretch(1.0));

            })
            .width(Pixels(52.0))
            .height(Stretch(1.0))
            .class("activity-bar");
        })
    }
}

fn get_icon_for_panel(panel: PanelType) -> &'static str {
    match panel {
        PanelType::GlobalSearch => ICON_SEARCH,
        PanelType::SceneBuilder => ICON_CUBES,
        PanelType::LogicGraph => ICON_LOGIC,
        PanelType::Director => ICON_DIRECTOR,
        PanelType::BackendAI => ICON_TERMINAL,
        PanelType::AssetVault => ICON_FOLDER,
        PanelType::Extensions => ICON_EXTENSIONS,
    }
}

impl View for ActivityBar {}
