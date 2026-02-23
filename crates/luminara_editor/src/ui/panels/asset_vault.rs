use vizia::prelude::*;
use crate::ui::icons::*;
use crate::ui::state::EditorState;

pub fn build(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Toolbar
        HStack::new(cx, |cx| {
            Button::new(cx, |cx| Label::new(cx, "Import"))
                .on_press(|_| println!("Import Asset"))
                .class("btn-outline");

            Button::new(cx, |cx| Svg::new(cx, ICON_REFRESH).class("icon-small"))
                .on_press(|_| println!("Refresh Assets"))
                .class("icon-btn");

            Element::new(cx).width(Stretch(1.0)); // Spacer

            Textbox::new(cx, EditorState::dummy_text)
                .class("search-box-small");

             Svg::new(cx, ICON_GRID).class("icon-small");
        })
        .class("toolbar");

        // Asset Grid
        ScrollView::new(cx, |cx| {
            // Flex Wrap for grid simulation in Vizia (Flow layout)
            HStack::new(cx, |cx| {
                 for i in 0..10 {
                    VStack::new(cx, move |cx| {
                        Svg::new(cx, ICON_CUBES).class("icon-large").class("asset-icon");
                        Label::new(cx, &format!("Asset {}", i)).class("asset-label");
                    })
                    .class("asset-item")
                    .on_press(move |_| println!("Selected Asset {}", i));
                 }
            })
            .class("asset-grid-container")
            .toggle_class("flex-wrap", true);
        });
    });
}
