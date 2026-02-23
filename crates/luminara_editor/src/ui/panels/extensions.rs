use vizia::prelude::*;
use crate::ui::icons::*;

pub fn build(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "EXTENSIONS MARKETPLACE").class("h2").class("p-4");

        ScrollView::new(cx, |cx| {
            VStack::new(cx, |cx| {
                // Extension 1
                HStack::new(cx, |cx| {
                    Svg::new(cx, ICON_EXTENSIONS).class("icon-large");
                    VStack::new(cx, |cx| {
                        Label::new(cx, "VFX Graph").class("ext-title");
                        Label::new(cx, "Advanced visual effects editor").class("ext-desc");
                    });
                    Element::new(cx).left(Stretch(1.0)); // push to right
                    Button::new(cx, |cx| Label::new(cx, "Install"))
                        .on_press(|_| println!("Install VFX Graph"))
                        .class("btn-primary");
                })
                .class("extension-item");

                 // Extension 2
                HStack::new(cx, |cx| {
                    Svg::new(cx, ICON_EXTENSIONS).class("icon-large");
                    VStack::new(cx, |cx| {
                        Label::new(cx, "Terrain Builder").class("ext-title");
                        Label::new(cx, "Procedural terrain generation tools").class("ext-desc");
                    });
                    Element::new(cx).left(Stretch(1.0));
                    Button::new(cx, |cx| Label::new(cx, "Installed"))
                        .on_press(|_| {})
                        .class("btn-disabled");
                })
                .class("extension-item");
            })
            .class("extension-list");
        });
    });
}
