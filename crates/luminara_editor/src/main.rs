//! Luminara Editor - Main Entry Point
//!
//! Launches the Vizia-based editor with full engine integration.
//! Provides a VS Code-style layout with activity bar, panels, and a status bar.

use luminara_asset::AssetServer;
use luminara_core::App;
use luminara_editor::services::engine_bridge::EngineHandle;
use luminara_editor::ui::theme::Theme;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use vizia::prelude::*;

// ──────────────────────────────────────────────
//  Data Model
// ──────────────────────────────────────────────

const PANEL_NAMES: [&str; 7] = [
    "Global Search",
    "Scene Builder",
    "Logic Graph",
    "Director",
    "Backend & AI",
    "Asset Vault",
    "Extensions",
];

const PANEL_ICONS: [&str; 7] = [
    "\u{1F50D}", // 🔍
    "\u{1F3AC}", // 🎬
    "\u{1F500}", // 🔀
    "\u{1F3AD}", // 🎭
    "\u{1F916}", // 🤖
    "\u{1F4E6}", // 📦
    "\u{1F9E9}", // 🧩
];

#[derive(Lens)]
pub struct EditorData {
    active_panel: usize,
}

pub enum EditorEvent {
    SetPanel(usize),
}

impl Model for EditorData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event: &EditorEvent, _| match app_event {
            EditorEvent::SetPanel(idx) => {
                self.active_panel = *idx;
            }
        });
    }
}

// ──────────────────────────────────────────────
//  Main
// ──────────────────────────────────────────────

fn main() -> Result<(), ApplicationError> {
    // Initialize engine
    let mut engine_app = App::new();

    use luminara_scene::scene::Name;

    let camera = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(camera, Name::new("Main Camera"));

    let light = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(light, Name::new("Directional Light"));

    let player = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(player, Name::new("Player Character"));

    let cube = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(cube, Name::new("Cube Mesh"));

    let ground = engine_app.world.spawn();
    let _ = engine_app
        .world
        .add_component(ground, Name::new("Ground Plane"));

    let world = Arc::new(RwLock::new(engine_app.world));
    let asset_server = Arc::new(AssetServer::new(PathBuf::from("assets")));

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let database = Arc::new(
        rt.block_on(luminara_editor::Database::new_memory())
            .expect("Failed to create database"),
    );
    drop(rt);

    let render_pipeline = Arc::new(RwLock::new(luminara_editor::RenderPipeline::mock()));

    let _engine_handle = Arc::new(EngineHandle::new(
        world,
        asset_server,
        database,
        render_pipeline,
    ));

    let theme = Theme::default_dark();
    let c = theme.colors.clone();

    // Extract all colors as Copy values for use in closures
    let bg = c.background;
    let surface = c.surface;
    let text = c.text;
    let text_sec = c.text_secondary;
    let accent = c.accent;
    let _border_col = c.border;
    let toolbar_bg = c.toolbar_bg;
    let panel_hdr = c.panel_header;
    let canvas_bg = c.canvas_background;

    Application::new(move |cx| {
        // Apply custom CSS theme
        cx.add_stylesheet(EDITOR_CSS)
            .expect("Failed to add stylesheet");

        // Install data model
        EditorData { active_panel: 1 }.build(cx);

        // Root vertical layout
        VStack::new(cx, move |cx| {
            // ═══════ Menu Bar ═══════
            HStack::new(cx, move |cx| {
                for item in &[
                    "File", "Edit", "View", "Project", "Build", "Tools", "Help",
                ] {
                    Label::new(cx, *item)
                        .class("menu-item")
                        .color(text_sec)
                        .font_size(12.0);
                }
            })
            .class("menu-bar")
            .height(Pixels(28.0))
            .width(Stretch(1.0))
            .background_color(toolbar_bg)
            .horizontal_gap(Pixels(0.0))
            .padding_left(Pixels(60.0))
            .padding_top(Stretch(1.0))
            .padding_bottom(Stretch(1.0));

            // ═══════ Main Area: Activity Bar + Content ═══════
            HStack::new(cx, move |cx| {
                // ─── Activity Bar (left 52px) ───
                VStack::new(cx, move |cx| {
                    // Top items (panels 0-6)
                    VStack::new(cx, move |cx| {
                        for (i, icon) in PANEL_ICONS.iter().enumerate() {
                            let panel_idx = i;
                            Button::new(cx, move |cx| {
                                Label::new(cx, *icon)
                                    .font_size(18.0)
                                    .width(Stretch(1.0))
                                    .height(Stretch(1.0))
                                    .padding(Stretch(1.0))
                            })
                            .on_press(move |ex| ex.emit(EditorEvent::SetPanel(panel_idx)))
                            .class("activity-item")
                            .width(Pixels(48.0))
                            .height(Pixels(44.0))
                            .left(Pixels(2.0));
                        }
                    })
                    .vertical_gap(Pixels(2.0))
                    .padding_top(Pixels(4.0));

                    // Spacer
                    Element::new(cx).height(Stretch(1.0));

                    // Bottom (settings icon)
                    Label::new(cx, "\u{2699}\u{FE0F}") // ⚙️
                        .font_size(18.0)
                        .width(Pixels(48.0))
                        .height(Pixels(44.0))
                        .padding(Stretch(1.0))
                        .color(text_sec)
                        .left(Pixels(2.0))
                        .bottom(Pixels(8.0));
                })
                .class("activity-bar")
                .width(Pixels(52.0))
                .height(Stretch(1.0))
                .background_color(surface);

                // ─── Content Area (right side) ───
                VStack::new(cx, move |cx| {
                    // Panel toolbar (shows active panel name)
                    HStack::new(cx, move |cx| {
                        Binding::new(cx, EditorData::active_panel, move |cx, lens| {
                            let panel = lens.get(cx);
                            Label::new(cx, PANEL_NAMES[panel])
                                .color(text)
                                .font_size(13.0)
                                .font_weight(FontWeight(700));
                        });
                    })
                    .class("panel-toolbar")
                    .height(Pixels(36.0))
                    .width(Stretch(1.0))
                    .background_color(toolbar_bg)
                    .padding_left(Pixels(16.0))
                    .padding_top(Stretch(1.0))
                    .padding_bottom(Stretch(1.0));

                    // Three-column layout: left sidebar + center + right sidebar
                    HStack::new(cx, move |cx| {
                        // ─── Left sidebar (hierarchy) ───
                        VStack::new(cx, move |cx| {
                            // Header: EXPLORER
                            HStack::new(cx, move |cx| {
                                Label::new(cx, "EXPLORER")
                                    .color(text_sec)
                                    .font_size(11.0)
                                    .font_weight(FontWeight(700));
                            })
                            .height(Pixels(28.0))
                            .width(Stretch(1.0))
                            .background_color(panel_hdr)
                            .padding_left(Pixels(12.0))
                            .padding_top(Stretch(1.0))
                            .padding_bottom(Stretch(1.0));

                            // File tree
                            VStack::new(cx, move |cx| {
                                for (label, depth) in [
                                    ("\u{1F4C1} Scenes", 0u32),
                                    ("  \u{1F4C4} Main.scene", 1),
                                    ("\u{1F4C1} Scripts", 0),
                                    ("  \u{1F4C4} player.lua", 1),
                                    ("  \u{1F4C4} enemy_ai.lua", 1),
                                    ("\u{1F4C1} Assets", 0),
                                    ("  \u{1F4C1} Models", 0),
                                    ("  \u{1F4C1} Textures", 0),
                                    ("  \u{1F4C1} Audio", 0),
                                ] {
                                    Label::new(cx, label)
                                        .color(if depth > 0 { text_sec } else { text })
                                        .font_size(12.0)
                                        .left(Pixels(4.0 + depth as f32 * 8.0))
                                        .height(Pixels(22.0))
                                        .padding_top(Stretch(1.0))
                                        .padding_bottom(Stretch(1.0));
                                }
                            })
                            .padding_left(Pixels(8.0))
                            .padding_top(Pixels(4.0))
                            .width(Stretch(1.0))
                            .height(Stretch(1.0));

                            // Header: SCENE ENTITIES
                            HStack::new(cx, move |cx| {
                                Label::new(cx, "SCENE ENTITIES")
                                    .color(text_sec)
                                    .font_size(11.0)
                                    .font_weight(FontWeight(700));
                            })
                            .height(Pixels(28.0))
                            .width(Stretch(1.0))
                            .background_color(panel_hdr)
                            .padding_left(Pixels(12.0))
                            .padding_top(Stretch(1.0))
                            .padding_bottom(Stretch(1.0));

                            // Entity list
                            VStack::new(cx, move |cx| {
                                for name in [
                                    "\u{1F3A5} Main Camera",
                                    "\u{2600}\u{FE0F} Directional Light",
                                    "\u{1F9D1} Player Character",
                                    "\u{1F7E6} Cube Mesh",
                                    "\u{1F7EB} Ground Plane",
                                ] {
                                    Label::new(cx, name)
                                        .color(text_sec)
                                        .font_size(12.0)
                                        .left(Pixels(4.0))
                                        .height(Pixels(22.0))
                                        .padding_top(Stretch(1.0))
                                        .padding_bottom(Stretch(1.0))
                                        .class("entity-item");
                                }
                            })
                            .padding_left(Pixels(8.0))
                            .padding_top(Pixels(4.0))
                            .width(Stretch(1.0))
                            .height(Stretch(1.0));
                        })
                        .class("left-sidebar")
                        .width(Pixels(240.0))
                        .height(Stretch(1.0))
                        .background_color(surface);

                        // ─── Center content (viewport / canvas) ───
                        VStack::new(cx, move |cx| {
                            Binding::new(cx, EditorData::active_panel, move |cx, lens| {
                                let panel = lens.get(cx);
                                build_panel_content(cx, panel, canvas_bg, text, text_sec, accent);
                            });
                        })
                        .class("center-content")
                        .width(Stretch(1.0))
                        .height(Stretch(1.0))
                        .background_color(canvas_bg);

                        // ─── Right sidebar (inspector) ───
                        VStack::new(cx, move |cx| {
                            // Header: INSPECTOR
                            HStack::new(cx, move |cx| {
                                Label::new(cx, "INSPECTOR")
                                    .color(text_sec)
                                    .font_size(11.0)
                                    .font_weight(FontWeight(700));
                            })
                            .height(Pixels(28.0))
                            .width(Stretch(1.0))
                            .background_color(panel_hdr)
                            .padding_left(Pixels(12.0))
                            .padding_top(Stretch(1.0))
                            .padding_bottom(Stretch(1.0));

                            // Transform section
                            VStack::new(cx, move |cx| {
                                Label::new(cx, "Transform")
                                    .color(text)
                                    .font_size(12.0)
                                    .font_weight(FontWeight(700));

                                Element::new(cx).height(Pixels(8.0));

                                for (label, val) in [
                                    ("Position", "0.00, 2.00, 5.00"),
                                    ("Rotation", "0.00, 0.00, 0.00"),
                                    ("Scale   ", "1.00, 1.00, 1.00"),
                                ] {
                                    HStack::new(cx, move |cx| {
                                        Label::new(cx, label)
                                            .color(text_sec)
                                            .font_size(11.0)
                                            .width(Pixels(64.0));
                                        Label::new(cx, val)
                                            .color(text)
                                            .font_size(11.0);
                                    })
                                    .height(Pixels(22.0))
                                    .padding_top(Stretch(1.0))
                                    .padding_bottom(Stretch(1.0));
                                }

                                Element::new(cx).height(Pixels(16.0));

                                Label::new(cx, "Components")
                                    .color(text)
                                    .font_size(12.0)
                                    .font_weight(FontWeight(700));

                                Element::new(cx).height(Pixels(8.0));

                                Label::new(cx, "No entity selected")
                                    .color(text_sec)
                                    .font_size(11.0);
                            })
                            .padding_left(Pixels(12.0))
                            .padding_top(Pixels(12.0))
                            .width(Stretch(1.0))
                            .height(Stretch(1.0));
                        })
                        .class("right-sidebar")
                        .width(Pixels(280.0))
                        .height(Stretch(1.0))
                        .background_color(surface);
                    })
                    .height(Stretch(1.0))
                    .width(Stretch(1.0));

                    // ─── Bottom panel (console / output) ───
                    VStack::new(cx, move |cx| {
                        // Tab bar
                        HStack::new(cx, move |cx| {
                            Label::new(cx, "Console")
                                .color(text)
                                .font_size(11.0)
                                .font_weight(FontWeight(700));
                            Label::new(cx, "Output")
                                .color(text_sec)
                                .font_size(11.0);
                            Label::new(cx, "Problems")
                                .color(text_sec)
                                .font_size(11.0);
                            Label::new(cx, "DB Explorer")
                                .color(text_sec)
                                .font_size(11.0);
                        })
                        .height(Pixels(28.0))
                        .width(Stretch(1.0))
                        .background_color(panel_hdr)
                        .padding_left(Pixels(12.0))
                        .padding_top(Stretch(1.0))
                        .padding_bottom(Stretch(1.0))
                        .horizontal_gap(Pixels(16.0));

                        // Console output
                        VStack::new(cx, move |cx| {
                            let green = Color::rgb(0x4e, 0xc9, 0xb0);
                            Label::new(cx, "[INFO]  Luminara Editor v0.1.0 started")
                                .color(green)
                                .font_size(11.0);
                            Label::new(cx, "[INFO]  Engine initialized successfully")
                                .color(green)
                                .font_size(11.0);
                            Label::new(cx, "[INFO]  Scene loaded: 5 entities")
                                .color(green)
                                .font_size(11.0);
                            Label::new(cx, "[INFO]  Database connected (in-memory)")
                                .color(green)
                                .font_size(11.0);
                            Label::new(cx, "> Ready")
                                .color(text_sec)
                                .font_size(11.0);
                        })
                        .padding_left(Pixels(12.0))
                        .padding_top(Pixels(6.0))
                        .vertical_gap(Pixels(2.0))
                        .width(Stretch(1.0))
                        .height(Stretch(1.0));
                    })
                    .class("bottom-panel")
                    .height(Pixels(160.0))
                    .width(Stretch(1.0))
                    .background_color(bg);
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0));
            })
            .height(Stretch(1.0))
            .width(Stretch(1.0));

            // ═══════ Status Bar ═══════
            HStack::new(cx, move |cx| {
                Label::new(cx, "Luminara Engine v0.1.0")
                    .color(Color::rgb(0xff, 0xff, 0xff))
                    .font_size(11.0);

                Element::new(cx).width(Stretch(1.0));

                Binding::new(cx, EditorData::active_panel, move |cx, lens| {
                    let panel = lens.get(cx);
                    Label::new(cx, PANEL_NAMES[panel])
                        .color(Color::rgb(0xff, 0xff, 0xff))
                        .font_size(11.0);
                });

                Label::new(cx, "FPS: 60")
                    .color(Color::rgb(0xff, 0xff, 0xff))
                    .font_size(11.0);

                Label::new(cx, "Entities: 5")
                    .color(Color::rgb(0xff, 0xff, 0xff))
                    .font_size(11.0);
            })
            .class("status-bar")
            .height(Pixels(22.0))
            .width(Stretch(1.0))
            .background_color(Color::rgb(0x00, 0x7a, 0xcc))
            .padding_left(Pixels(10.0))
            .padding_right(Pixels(10.0))
            .padding_top(Stretch(1.0))
            .padding_bottom(Stretch(1.0))
            .horizontal_gap(Pixels(16.0));
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0));
    })
    .title("Luminara Editor")
    .inner_size((1400, 900))
    .run()
}

// ──────────────────────────────────────────────
//  Panel Content Builder
// ──────────────────────────────────────────────

fn build_panel_content(
    cx: &mut Context,
    panel: usize,
    canvas_bg: Color,
    text: Color,
    text_sec: Color,
    accent: Color,
) {
    match panel {
        0 => {
            // Global Search
            VStack::new(cx, move |cx| {
                Element::new(cx).height(Pixels(60.0));

                Label::new(cx, "\u{1F50D} Global Search")
                    .color(text)
                    .font_size(18.0)
                    .font_weight(FontWeight(700))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0));

                Element::new(cx).height(Pixels(16.0));

                // Search input placeholder
                HStack::new(cx, move |cx| {
                    Label::new(cx, "Search entities, assets, scripts, commands...")
                        .color(text_sec)
                        .font_size(13.0);
                })
                .class("search-box")
                .height(Pixels(36.0))
                .width(Pixels(600.0))
                .background_color(Color::rgb(0x33, 0x33, 0x33))
                .padding_left(Pixels(12.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0))
                .left(Stretch(1.0))
                .right(Stretch(1.0));

                Element::new(cx).height(Pixels(20.0));

                Label::new(
                    cx,
                    "Prefixes:  >Command  @Entity  #Asset  :DB  ?AI  !Script  /Scene  %Logic",
                )
                .color(text_sec)
                .font_size(11.0)
                .left(Stretch(1.0))
                .right(Stretch(1.0));
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
        }
        1 => {
            // Scene Builder viewport
            VStack::new(cx, move |cx| {
                // Viewport toolbar
                HStack::new(cx, move |cx| {
                    for tool in &["Move", "Rotate", "Scale"] {
                        Button::new(cx, move |cx| {
                            Label::new(cx, *tool).font_size(11.0).color(text_sec)
                        })
                        .class("tool-btn")
                        .height(Pixels(26.0));
                    }
                    Element::new(cx).width(Stretch(1.0));
                    Label::new(cx, "Perspective")
                        .color(text_sec)
                        .font_size(11.0);
                })
                .height(Pixels(32.0))
                .width(Stretch(1.0))
                .background_color(Color::rgba(0, 0, 0, 60))
                .padding_left(Pixels(8.0))
                .padding_right(Pixels(8.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0))
                .horizontal_gap(Pixels(4.0));

                // 3D Viewport placeholder
                VStack::new(cx, move |cx| {
                    Element::new(cx).height(Stretch(1.0));
                    Label::new(cx, "\u{1F3AC} 3D Viewport")
                        .color(text)
                        .font_size(16.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Label::new(cx, "WGPU rendering area")
                        .color(text_sec)
                        .font_size(12.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Element::new(cx).height(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(canvas_bg);
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0));
        }
        2 => {
            // Logic Graph
            VStack::new(cx, move |cx| {
                // Graph toolbar
                HStack::new(cx, move |cx| {
                    for tool in &["Select", "Connect", "Pan"] {
                        Button::new(cx, move |cx| {
                            Label::new(cx, *tool).font_size(11.0).color(text_sec)
                        })
                        .class("tool-btn")
                        .height(Pixels(26.0));
                    }
                })
                .height(Pixels(32.0))
                .width(Stretch(1.0))
                .background_color(Color::rgba(0, 0, 0, 60))
                .padding_left(Pixels(8.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0))
                .horizontal_gap(Pixels(4.0));

                // Graph canvas placeholder
                VStack::new(cx, move |cx| {
                    Element::new(cx).height(Stretch(1.0));
                    Label::new(cx, "\u{1F500} Logic Graph")
                        .color(text)
                        .font_size(16.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Label::new(cx, "Visual scripting node graph")
                        .color(text_sec)
                        .font_size(12.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Element::new(cx).height(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0));
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
        }
        3 => {
            // Director
            VStack::new(cx, move |cx| {
                // Viewport area
                VStack::new(cx, move |cx| {
                    Element::new(cx).height(Stretch(1.0));
                    Label::new(cx, "\u{1F3AD} Director - Cinematic Timeline")
                        .color(text)
                        .font_size(16.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Label::new(cx, "Keyframe animation & sequencing")
                        .color(text_sec)
                        .font_size(12.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Element::new(cx).height(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0));

                // Timeline bar
                HStack::new(cx, move |cx| {
                    Label::new(cx, "\u{25B6}\u{FE0F}") // ▶️
                        .color(text)
                        .font_size(14.0);
                    Label::new(cx, "00:00:00")
                        .color(accent)
                        .font_size(12.0);
                    Element::new(cx).width(Stretch(1.0));
                    Label::new(cx, "30 FPS")
                        .color(text_sec)
                        .font_size(11.0);
                })
                .height(Pixels(32.0))
                .width(Stretch(1.0))
                .background_color(Color::rgb(0x25, 0x25, 0x25))
                .padding_left(Pixels(12.0))
                .padding_right(Pixels(12.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0))
                .horizontal_gap(Pixels(12.0));

                // Timeline tracks placeholder
                Element::new(cx)
                    .height(Pixels(120.0))
                    .width(Stretch(1.0))
                    .background_color(Color::rgb(0x1e, 0x1e, 0x1e));
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
        }
        4 => {
            // Backend & AI
            HStack::new(cx, move |cx| {
                // File tree
                VStack::new(cx, move |cx| {
                    HStack::new(cx, move |cx| {
                        Label::new(cx, "FILES")
                            .color(text_sec)
                            .font_size(11.0)
                            .font_weight(FontWeight(700));
                    })
                    .height(Pixels(28.0))
                    .background_color(Color::rgb(0x32, 0x32, 0x32))
                    .padding_left(Pixels(12.0))
                    .padding_top(Stretch(1.0))
                    .padding_bottom(Stretch(1.0));

                    VStack::new(cx, move |cx| {
                        for f in [
                            "\u{1F4C4} main.lua",
                            "\u{1F4C4} player.lua",
                            "\u{1F4C4} enemy_ai.lua",
                            "\u{1F4C4} inventory.lua",
                        ] {
                            Label::new(cx, f)
                                .color(text_sec)
                                .font_size(12.0)
                                .height(Pixels(22.0))
                                .padding_top(Stretch(1.0))
                                .padding_bottom(Stretch(1.0));
                        }
                    })
                    .padding_left(Pixels(8.0))
                    .padding_top(Pixels(4.0));
                })
                .width(Pixels(200.0))
                .height(Stretch(1.0))
                .background_color(Color::rgb(0x25, 0x25, 0x25));

                // Script editor area
                VStack::new(cx, move |cx| {
                    Element::new(cx).height(Stretch(1.0));
                    Label::new(cx, "\u{1F916} Backend & AI")
                        .color(text)
                        .font_size(16.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Label::new(cx, "Script editor with AI assistance")
                        .color(text_sec)
                        .font_size(12.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Element::new(cx).height(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .background_color(canvas_bg);

                // AI assistant sidebar
                VStack::new(cx, move |cx| {
                    HStack::new(cx, move |cx| {
                        Label::new(cx, "AI ASSISTANT")
                            .color(text_sec)
                            .font_size(11.0)
                            .font_weight(FontWeight(700));
                    })
                    .height(Pixels(28.0))
                    .background_color(Color::rgb(0x32, 0x32, 0x32))
                    .padding_left(Pixels(12.0))
                    .padding_top(Stretch(1.0))
                    .padding_bottom(Stretch(1.0));

                    VStack::new(cx, move |cx| {
                        Label::new(cx, "Ask me anything about your project...")
                            .color(text_sec)
                            .font_size(12.0);
                    })
                    .padding_left(Pixels(12.0))
                    .padding_top(Pixels(12.0));
                })
                .width(Pixels(280.0))
                .height(Stretch(1.0))
                .background_color(Color::rgb(0x25, 0x25, 0x25));
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0));
        }
        5 => {
            // Asset Vault
            VStack::new(cx, move |cx| {
                // Asset toolbar
                HStack::new(cx, move |cx| {
                    for item in &["Import", "Export", "Refresh"] {
                        Button::new(cx, move |cx| {
                            Label::new(cx, *item).font_size(11.0).color(text_sec)
                        })
                        .class("tool-btn")
                        .height(Pixels(26.0));
                    }
                    Element::new(cx).width(Stretch(1.0));
                    Label::new(cx, "Grid View")
                        .color(text_sec)
                        .font_size(11.0);
                })
                .height(Pixels(32.0))
                .width(Stretch(1.0))
                .background_color(Color::rgba(0, 0, 0, 60))
                .padding_left(Pixels(8.0))
                .padding_right(Pixels(8.0))
                .padding_top(Stretch(1.0))
                .padding_bottom(Stretch(1.0))
                .horizontal_gap(Pixels(4.0));

                // Asset grid placeholder
                VStack::new(cx, move |cx| {
                    Element::new(cx).height(Stretch(1.0));
                    Label::new(cx, "\u{1F4E6} Asset Vault")
                        .color(text)
                        .font_size(16.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Label::new(cx, "Browse and manage project assets")
                        .color(text_sec)
                        .font_size(12.0)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0));
                    Element::new(cx).height(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0));
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
        }
        6 => {
            // Extensions
            VStack::new(cx, move |cx| {
                Element::new(cx).height(Stretch(1.0));
                Label::new(cx, "\u{1F9E9} Extensions")
                    .color(text)
                    .font_size(16.0)
                    .left(Stretch(1.0))
                    .right(Stretch(1.0));
                Label::new(cx, "Marketplace and installed extensions")
                    .color(text_sec)
                    .font_size(12.0)
                    .left(Stretch(1.0))
                    .right(Stretch(1.0));
                Element::new(cx).height(Stretch(1.0));
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .background_color(canvas_bg);
        }
        _ => {
            Label::new(cx, "Unknown Panel")
                .color(text)
                .font_size(14.0);
        }
    }
}

// ──────────────────────────────────────────────
//  CSS Theme
// ──────────────────────────────────────────────

const EDITOR_CSS: &str = r#"
* {
    border-color: #3a3a3a;
}

.menu-bar {
    border-bottom-width: 1px;
}

.activity-bar {
    border-right-width: 1px;
}

button.activity-item {
    background-color: transparent;
    border-width: 0px;
    border-radius: 6px;
    cursor: hand;
}

button.activity-item:hover {
    background-color: #3a3a3a;
}

button.activity-item:active {
    background-color: #4a4a4a;
}

button.tool-btn {
    background-color: #3a3a3a;
    border-width: 0px;
    border-radius: 4px;
    cursor: hand;
    padding-left: 8px;
    padding-right: 8px;
}

button.tool-btn:hover {
    background-color: #4a4a4a;
}

.panel-toolbar {
    border-bottom-width: 1px;
}

.left-sidebar {
    border-right-width: 1px;
}

.right-sidebar {
    border-left-width: 1px;
}

.bottom-panel {
    border-top-width: 1px;
}

.entity-item:hover {
    background-color: #2a2a3a;
}

.menu-item {
    cursor: hand;
    padding-left: 6px;
    padding-right: 6px;
    padding-top: 2px;
    padding-bottom: 2px;
    border-radius: 4px;
}

.menu-item:hover {
    background-color: #3a3a3a;
}

.search-box {
    border-radius: 6px;
}

.status-bar label {
    font-size: 11;
}
"#;
