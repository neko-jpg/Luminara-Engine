use vizia::prelude::*;
use crate::ui::icons::*;

pub fn build(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Toolbar
        HStack::new(cx, |cx| {
            Button::new(cx, |cx| Svg::new(cx, ICON_SELECT).class("icon-small"))
                .on_press(|_| println!("Select Node"))
                .class("tool-btn");
             Button::new(cx, |cx| Svg::new(cx, ICON_LOGIC).class("icon-small"))
                .on_press(|_| println!("Connect Node"))
                .class("tool-btn");

             Element::new(cx).width(Stretch(1.0));
             Label::new(cx, "Logic Graph: Main Quest").class("toolbar-label");
        })
        .class("toolbar");

        // Canvas
        VStack::new(cx, |cx| {
                // Background grid (handled by CSS usually, but here placeholder)
                Label::new(cx, "Logic Graph Editor")
                    .class("watermark");

                // Node 1
                VStack::new(cx, |cx| {
                    Label::new(cx, "Event: OnStart").class("node-header");
                    Label::new(cx, "Trigger").class("node-port-out");
                })
                .class("graph-node")
                .left(Pixels(50.0))
                .top(Pixels(50.0));

                 // Node 2
                VStack::new(cx, |cx| {
                    Label::new(cx, "Action: Spawn").class("node-header");
                    HStack::new(cx, |cx| {
                        Label::new(cx, "In").class("node-port-in");
                        Element::new(cx).width(Stretch(1.0));
                        Label::new(cx, "Out").class("node-port-out");
                    });
                     HStack::new(cx, |cx| {
                        Label::new(cx, "Prefab").class("node-port-in");
                    });
                })
                .class("graph-node")
                .left(Pixels(300.0))
                .top(Pixels(100.0));
            })
            .class("graph-canvas");
    });
}
