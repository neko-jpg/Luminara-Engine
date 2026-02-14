use luminara_input::{ActionMap, InputAction, InputExt, Input};
use luminara_input::input_map::{ActionBinding, InputSource};
use luminara_input::keyboard::Key;
use luminara_input::mouse::MouseButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestAction {
    Jump,
    Fire,
}

// Blanket impl covers TestAction, so no manual impl needed.

#[test]
fn test_action_mapping_consistency() {
    let mut map = ActionMap::<TestAction>::new();

    // Bind Jump to Space
    map.bind(TestAction::Jump, ActionBinding {
        inputs: vec![InputSource::Key(Key::Space)],
    });

    // Bind Fire to Mouse Left
    map.bind(TestAction::Fire, ActionBinding {
        inputs: vec![InputSource::MouseButton(MouseButton::Left)],
    });

    let mut input = Input::default();

    // Verify initial state
    assert!(!InputExt::action_pressed(&input, TestAction::Jump, &map));
    assert!(!InputExt::action_pressed(&input, TestAction::Fire, &map));

    // Simulate Space press
    input.keyboard.pressed.insert(Key::Space);
    assert!(InputExt::action_pressed(&input, TestAction::Jump, &map));
    assert!(!InputExt::action_pressed(&input, TestAction::Fire, &map));

    // Simulate Mouse Left press
    input.mouse.buttons.insert(MouseButton::Left);
    assert!(InputExt::action_pressed(&input, TestAction::Jump, &map));
    assert!(InputExt::action_pressed(&input, TestAction::Fire, &map));

    // Simulate Space release
    input.keyboard.pressed.remove(&Key::Space);
    assert!(!InputExt::action_pressed(&input, TestAction::Jump, &map));
    assert!(InputExt::action_pressed(&input, TestAction::Fire, &map));
}
