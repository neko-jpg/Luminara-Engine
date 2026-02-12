use luminara_window::*;
use std::path::PathBuf;

#[test]
fn test_window_descriptor_default() {
    let desc = WindowDescriptor::default();
    assert_eq!(desc.title, "Luminara Game");
    assert_eq!(desc.width, 1280);
    assert_eq!(desc.height, 720);
}

#[test]
fn test_window_event_serialization() {
    let event = WindowEvent::Resized {
        width: 800,
        height: 600,
    };
    let serialized = serde_json::to_string(&event).unwrap();
    let deserialized: WindowEvent = serde_json::from_str(&serialized).unwrap();
    assert_eq!(event, deserialized);

    let event = WindowEvent::DroppedFile(PathBuf::from("test.txt"));
    let serialized = serde_json::to_string(&event).unwrap();
    let deserialized: WindowEvent = serde_json::from_str(&serialized).unwrap();
    assert_eq!(event, deserialized);
}
