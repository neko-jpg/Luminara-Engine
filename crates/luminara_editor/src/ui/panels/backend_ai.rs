use vizia::prelude::*;
use crate::ui::icons::*;
use crate::ui::state::EditorState;

pub fn build(cx: &mut Context) {
    HStack::new(cx, |cx| {
        // File Explorer (Script Files)
        VStack::new(cx, |cx| {
            Label::new(cx, "SCRIPTS").class("panel-header");
            VStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Svg::new(cx, ICON_FILE).class("icon-small");
                    Label::new(cx, "player_controller.lua");
                }).class("file-item").class("selected");

                 HStack::new(cx, |cx| {
                    Svg::new(cx, ICON_FILE).class("icon-small");
                    Label::new(cx, "enemy_ai.lua");
                }).class("file-item");
            }).class("file-list");
        })
        .class("sidebar-left")
        .width(Pixels(200.0));

        // Code Editor
        VStack::new(cx, |cx| {
            // Tabs
            HStack::new(cx, |cx| {
                Label::new(cx, "player_controller.lua").class("tab-active");
                Label::new(cx, "enemy_ai.lua").class("tab-inactive");
            }).class("editor-tabs");

            // Code Area
            VStack::new(cx, |cx| {
                     Label::new(cx, "-- Player Controller Script\nfunction update(dt)\n  local input = Input.get_axis()\n  self.position += input * dt\nend")
                        .class("code-font");
                })
                .class("code-area")
                .padding(Pixels(10.0));
        })
        .class("code-editor-panel");

        // AI Assistant
        VStack::new(cx, |cx| {
            Label::new(cx, "AI ASSISTANT").class("panel-header");

            // Chat History
            ScrollView::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    // User msg
                    Label::new(cx, "How do I move the player?")
                        .class("chat-msg-user");

                    // AI msg
                    Label::new(cx, "You can use the Input.get_axis() function in the update loop.")
                        .class("chat-msg-ai");
                })
                .class("chat-history");
            });

            // Input
            HStack::new(cx, |cx| {
                Textbox::new(cx, EditorState::dummy_text)
                    .class("chat-input");
                Button::new(cx, |cx| Svg::new(cx, ICON_TERMINAL).class("icon-small"))
                    .on_press(|_| println!("Send AI Query"))
                    .class("icon-btn");
            })
            .class("chat-input-area");
        })
        .class("sidebar-right")
        .width(Pixels(280.0));
    });
}
