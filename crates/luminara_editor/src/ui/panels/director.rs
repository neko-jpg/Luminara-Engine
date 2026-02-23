use vizia::prelude::*;
use crate::ui::icons::*;

pub fn build(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Viewport (Cinematic Preview)
        VStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Svg::new(cx, ICON_DIRECTOR).class("icon-large").class("opacity-50");
                    Label::new(cx, "Cinematic Preview").class("text-muted");
                })
                .child_space(Stretch(1.0));
            })
            .class("director-viewport")
            .height(Percentage(60.0));

        // Timeline
        VStack::new(cx, |cx| {
            // Transport Controls
            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "|<"))
                    .on_press(|_| println!("Prev Frame"))
                    .class("transport-btn");
                Button::new(cx, |cx| Svg::new(cx, ICON_PLAY).class("icon-small"))
                    .on_press(|_| println!("Play"))
                    .class("transport-btn-primary");
                Button::new(cx, |cx| Label::new(cx, ">|"))
                    .on_press(|_| println!("Next Frame"))
                    .class("transport-btn");

                Label::new(cx, "00:00:12:05").class("timecode");

                Element::new(cx).width(Stretch(1.0));

                Label::new(cx, "30 FPS").class("text-muted");
            })
            .class("timeline-toolbar");

            // Tracks
            ScrollView::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    // Track Header
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Camera Track").width(Pixels(150.0)).class("track-label");
                        // Keyframes timeline
                        VStack::new(cx, |cx| {
                                Element::new(cx).class("keyframe").left(Pixels(20.0));
                                Element::new(cx).class("keyframe").left(Pixels(120.0));
                            })
                            .class("timeline-track");
                    })
                    .class("track-row");

                     HStack::new(cx, |cx| {
                        Label::new(cx, "Player Anim").width(Pixels(150.0)).class("track-label");
                         VStack::new(cx, |cx| {
                                Element::new(cx).class("clip-block").left(Pixels(0.0)).width(Pixels(100.0)).background_color(Color::rgb(60, 100, 160));
                            })
                            .class("timeline-track");
                    })
                    .class("track-row");
                });
            });
        })
        .class("timeline-panel");
    });
}
