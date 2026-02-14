use luminara_render::overlay::{OverlayRenderer, OverlayCommand};

#[test]
fn test_ui_coordinate_scaling() {
    let mut renderer = OverlayRenderer::new();

    // Draw text at (10, 20) with scale 1.0
    renderer.draw_text(10.0, 20.0, "Test", [1.0; 4], 1.0);

    // Inspect commands
    // With my change, draw_text adds shadow text + main text.
    assert_eq!(renderer.commands.len(), 2);

    match &renderer.commands[0] {
        OverlayCommand::Text { x, y, color, .. } => {
            // Shadow
            assert_eq!(*x, 11.0);
            assert_eq!(*y, 21.0);
            assert_eq!(color[0], 0.0); // Black shadow
        },
        _ => panic!("Expected Text command for shadow"),
    }

    match &renderer.commands[1] {
        OverlayCommand::Text { x, y, color, .. } => {
            // Main text
            assert_eq!(*x, 10.0);
            assert_eq!(*y, 20.0);
            assert_eq!(color[0], 1.0); // White text
        },
        _ => panic!("Expected Text command for main text"),
    }

    // DPI scaling test would involve winit scale factor which is handled in window system
    // Here we test that our renderer respects input coordinates.
}
