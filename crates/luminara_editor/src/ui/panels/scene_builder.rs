use vizia::prelude::*;
use crate::ui::icons::*;
use crate::ui::state::EditorState;

pub fn build(cx: &mut Context) {
    HStack::new(cx, |cx| {
        // --- Left Sidebar: Hierarchy ---
        VStack::new(cx, |cx| {
            // Header
            HStack::new(cx, |cx| {
                Label::new(cx, "SCENE HIERARCHY")
                    .class("panel-header-text");
                Element::new(cx).width(Stretch(1.0));
                Svg::new(cx, ICON_SEARCH)
                    .class("icon-small");
            })
            .class("panel-header");

            // Toolbar
            HStack::new(cx, |cx| {
                Textbox::new(cx, EditorState::dummy_text)
                    .class("search-box");
                Button::new(cx, |cx| Svg::new(cx, ICON_PLUS).class("icon-small"))
                    .on_press(|_| {})
                    .class("icon-btn");
            })
            .class("hierarchy-toolbar");

            // Tree
            VStack::new(cx, |cx| {
                // Item 1: Player
                HStack::new(cx, |cx| {
                    Svg::new(cx, ICON_CHEVRON_DOWN).class("icon-tiny");
                    Svg::new(cx, ICON_CUBES).class("icon-small").class("accent-icon");
                    Label::new(cx, "Player").class("tree-item-text");
                    Element::new(cx).width(Stretch(1.0));
                    Svg::new(cx, ICON_CHECK).class("icon-tiny").class("success-icon");
                })
                .class("tree-item")
                .class("selected");

                // Item 2: Main Camera (Child)
                HStack::new(cx, |cx| {
                    Element::new(cx).width(Pixels(16.0)); // Indent
                    Svg::new(cx, ICON_CHEVRON_RIGHT).class("icon-tiny").class("hidden"); // Spacer
                    Svg::new(cx, ICON_CUBES).class("icon-small").class("warning-icon"); // Camera icon placeholder
                    Label::new(cx, "Main Camera").class("tree-item-text");
                })
                .class("tree-item");

            })
            .class("tree-view");

        })
        .class("sidebar-left");

        // --- Center: Viewport ---
        VStack::new(cx, |cx| {
            // Toolbar
            HStack::new(cx, |cx| {
                // Tools
                HStack::new(cx, |cx| {
                    Button::new(cx, |cx| Svg::new(cx, ICON_MOVE).class("icon-small"))
                        .on_press(|_| println!("Move Tool"))
                        .class("tool-btn").class("active");
                    Button::new(cx, |cx| Svg::new(cx, ICON_ROTATE).class("icon-small"))
                        .on_press(|_| println!("Rotate Tool"))
                        .class("tool-btn");
                    Button::new(cx, |cx| Svg::new(cx, ICON_SCALE).class("icon-small"))
                        .on_press(|_| println!("Scale Tool"))
                        .class("tool-btn");
                })
                .class("toolbar-group");

                Element::new(cx).width(Pixels(1.0)).background_color(Color::rgb(60, 60, 60)); // Separator

                Label::new(cx, "Perspective").class("toolbar-label");

                Element::new(cx).width(Stretch(1.0));

                Label::new(cx, "Gizmos").class("toolbar-label");
            })
            .class("viewport-toolbar");

            // 3D Viewport Placeholder
            VStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CUBES)
                            .width(Pixels(64.0))
                            .height(Pixels(64.0))
                            .class("opacity-50");
                        Label::new(cx, "3D Viewport")
                            .class("h2");
                        Label::new(cx, "WGPU Rendering Surface")
                            .class("text-muted");
                    })
                    .child_space(Stretch(1.0));
            })
            .class("viewport-content")
            .padding(Stretch(1.0));

        })
        .class("center-panel");

        // --- Right Sidebar: Inspector ---
        VStack::new(cx, |cx| {
            // Header
            HStack::new(cx, |cx| {
                Label::new(cx, "INSPECTOR").class("panel-header-text");
                Element::new(cx).width(Stretch(1.0));
                Svg::new(cx, ICON_SETTINGS).class("icon-small");
            })
            .class("panel-header");

            // Content
            ScrollView::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    // Header Entity Info
                    HStack::new(cx, |cx| {
                         Checkbox::new(cx, true).on_toggle(|_| {});
                         Textbox::new(cx, EditorState::dummy_text).class("entity-name-input");
                    }).class("inspector-row");

                    // Transform Component
                    VStack::new(cx, |cx| {
                        HStack::new(cx, |cx| {
                             Label::new(cx, "Transform").class("component-title");
                             Element::new(cx).width(Stretch(1.0));
                             Svg::new(cx, ICON_SETTINGS).class("icon-tiny");
                        }).class("component-header");

                        // Position
                        HStack::new(cx, |cx| {
                            Label::new(cx, "Position").width(Pixels(80.0)).class("prop-label");
                            HStack::new(cx, |cx| {
                                Label::new(cx, "X").class("axis-label-x");
                                Textbox::new(cx, EditorState::dummy_text).class("prop-input");
                            }).class("vector-field");
                            HStack::new(cx, |cx| {
                                Label::new(cx, "Y").class("axis-label-y");
                                Textbox::new(cx, EditorState::dummy_text).class("prop-input");
                            }).class("vector-field");
                             HStack::new(cx, |cx| {
                                Label::new(cx, "Z").class("axis-label-z");
                                Textbox::new(cx, EditorState::dummy_text).class("prop-input");
                            }).class("vector-field");
                        }).class("prop-row");

                        // Rotation
                        HStack::new(cx, |cx| {
                            Label::new(cx, "Rotation").width(Pixels(80.0)).class("prop-label");
                            // Simplified for brevity
                             Label::new(cx, "0, 0, 0").class("text-muted");
                        }).class("prop-row");

                         // Scale
                        HStack::new(cx, |cx| {
                            Label::new(cx, "Scale").width(Pixels(80.0)).class("prop-label");
                            Label::new(cx, "1, 1, 1").class("text-muted");
                        }).class("prop-row");
                    })
                    .class("component-box");

                    // Add Component Button
                    Button::new(cx, |cx| {
                        Label::new(cx, "Add Component")
                    })
                    .on_press(|_| println!("Add Component Clicked"))
                    .class("btn-primary")
                    .class("w-full");

                })
                .class("inspector-content");
            });
        })
        .class("sidebar-right");
    });
}
