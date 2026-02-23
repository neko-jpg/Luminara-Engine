use vizia::prelude::*;
use crate::ui::icons::*;
use crate::ui::state::EditorState;

pub fn build(cx: &mut Context) {
    // 2-Column Layout
    HStack::new(cx, |cx| {
        // --- Left: Search & Filter ---
        VStack::new(cx, |cx| {
            // Header
            HStack::new(cx, |cx| {
                Label::new(cx, "Global Search").class("h2");
                Element::new(cx).width(Stretch(1.0));
            })
            .class("p-4");

            // Search Input
            HStack::new(cx, |cx| {
                Svg::new(cx, ICON_SEARCH).class("icon-medium").class("text-muted");
                Textbox::new(cx, EditorState::dummy_text)
                    .class("search-input-large")
                    .class("flex-1");
                Label::new(cx, "⌘P").class("shortcut-badge");
            })
            .class("search-bar-large");

            // Filters
            HStack::new(cx, |cx| {
                for (label, icon) in [
                    ("All", ICON_SEARCH),
                    ("Entities", ICON_CUBES),
                    ("Assets", ICON_FOLDER),
                    ("Scripts", ICON_FILE),
                ] {
                    HStack::new(cx, move |cx| {
                        Svg::new(cx, icon).class("icon-tiny");
                        Label::new(cx, label).class("filter-text");
                    })
                    .class("filter-chip")
                    .toggle_class("active", label == "All"); // Mock active
                }
            })
            .class("filter-bar");

            // Results List
            ScrollView::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "TOP RESULTS").class("section-header");

                    // Result Item 1
                    HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CUBES).class("icon-medium").class("accent-icon");
                        VStack::new(cx, |cx| {
                            Label::new(cx, "Player").class("result-title");
                            Label::new(cx, "Main Scene • Entity").class("result-subtitle");
                        });
                        Element::new(cx).width(Stretch(1.0));
                        Label::new(cx, "Entity").class("type-badge");
                    })
                    .class("result-item")
                    .class("selected");

                    // Result Item 2
                     HStack::new(cx, |cx| {
                        Svg::new(cx, ICON_FILE).class("icon-medium");
                        VStack::new(cx, |cx| {
                            Label::new(cx, "player_controller.rs").class("result-title");
                            Label::new(cx, "scripts/ • 4.2 KB").class("result-subtitle");
                        });
                        Element::new(cx).width(Stretch(1.0));
                        Label::new(cx, "Script").class("type-badge");
                    })
                    .class("result-item");

                })
                .class("results-list");
            });

        })
        .class("search-left-panel");

        // --- Right: Preview ---
        VStack::new(cx, |cx| {
             HStack::new(cx, |cx| {
                Svg::new(cx, ICON_CUBES).class("icon-medium").class("accent-icon");
                Label::new(cx, "Player (Entity)").class("h2");
                Element::new(cx).width(Stretch(1.0));
                Button::new(cx, |cx| Label::new(cx, "Edit"))
                    .on_press(|_| {})
                    .class("btn-outline");
            })
            .class("preview-header");

            // Preview Content
            VStack::new(cx, |cx| {
                // 3D Preview Placeholder
                VStack::new(cx, |cx| {
                        Svg::new(cx, ICON_CUBES).class("icon-large").class("opacity-50");
                        Label::new(cx, "3D Preview").class("text-muted");
                    })
                    .class("preview-box");

                // Meta Data
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Type:").class("meta-label");
                        Label::new(cx, "Entity (Character)").class("meta-value");
                    });
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Components:").class("meta-label");
                        Label::new(cx, "Transform, Rigidbody, AI").class("meta-value");
                    });
                })
                .class("meta-info");

                // JSON/Code Preview
                VStack::new(cx, |cx| {
                        Label::new(cx, "{\n  \"name\": \"Player\",\n  \"transform\": [2.5, 1.0, 0.0]\n}")
                            .class("code-text");
                    })
                    .class("code-block");

            })
            .class("preview-content");

        })
        .class("search-right-panel");

    })
    .class("global-search-container");
}
