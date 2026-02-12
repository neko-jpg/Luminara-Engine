use luminara_input::Input;
use luminara_input::keyboard::Key;
use luminara_input::mouse::MouseButton;
use luminara_input::input_map::InputMap;

#[test]
fn test_keyboard_state_transitions() {
    let mut input = Input::default();

    let key = Key::A;
    input.keyboard.pressed.insert(key);
    input.keyboard.just_pressed.insert(key);

    assert!(input.pressed(key));
    assert!(input.just_pressed(key));
    assert!(!input.just_released(key));

    input.keyboard.clear_just_states();
    assert!(input.pressed(key));
    assert!(!input.just_pressed(key));

    input.keyboard.pressed.remove(&key);
    input.keyboard.just_released.insert(key);
    assert!(!input.pressed(key));
    assert!(input.just_released(key));
}

#[test]
fn test_mouse_state_transitions() {
    let mut input = Input::default();
    let button = MouseButton::Left;

    input.mouse.buttons.insert(button);
    input.mouse.just_pressed.insert(button);

    assert!(input.mouse_pressed(button));
    assert!(input.mouse_just_pressed(button));

    input.mouse.clear_just_states();
    assert!(input.mouse_pressed(button));
    assert!(!input.mouse_just_pressed(button));
}

#[test]
fn test_input_map_axis_ramping() {
    let mut input = Input::default();
    let map = InputMap::default_mappings();

    // Press D (positive horizontal). Sensitivity is 3.0.
    input.keyboard.pressed.insert(Key::D);

    // After 0.1s, value should be 0.3 (3.0 * 0.1)
    input.update_axes(&map, 0.1);
    assert!((input.axis("horizontal") - 0.3).abs() < 0.001);

    // After another 0.3s, it should hit 1.0
    input.update_axes(&map, 0.3);
    assert_eq!(input.axis("horizontal"), 1.0);

    // Release D. Gravity is 3.0.
    input.keyboard.pressed.remove(&Key::D);
    input.update_axes(&map, 0.1);
    assert!((input.axis("horizontal") - 0.7).abs() < 0.001);
}

#[test]
fn test_input_map_actions() {
    let mut input = Input::default();
    let map = InputMap::default_mappings();

    // Press Space (Jump)
    input.keyboard.pressed.insert(Key::Space);
    input.keyboard.just_pressed.insert(Key::Space);

    assert!(input.action_pressed("jump", &map));
    assert!(input.action_just_pressed("jump", &map));

    input.keyboard.clear_just_states();
    assert!(input.action_pressed("jump", &map));
    assert!(!input.action_just_pressed("jump", &map));
}

#[test]
fn test_multi_gamepad() {
    let mut input = Input::default();
    let map = InputMap::default_mappings();

    // We can't easily mock gilrs, but we can manually populate GamepadInput
    if let Some(gamepad) = &mut input.gamepad {
        use luminara_input::gamepad::GamepadButton;

        // Gamepad 1 presses South (Jump)
        gamepad.pressed.entry(1).or_default().insert(GamepadButton::South);

        // Query for player 1
        assert!(input.action_pressed_for_player("jump", &map, 1));
        // Primary player is 0 by default, so they shouldn't see it
        assert!(!input.action_pressed("jump", &map));

        // Change primary to 1
        input.primary_gamepad_id = 1;
        assert!(input.action_pressed("jump", &map));
    }
}
