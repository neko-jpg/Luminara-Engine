use luminara_core::shared_types::Resource;
use std::collections::HashMap;
use std::hash::Hash;
use crate::input_map::ActionBinding;
use crate::Input;

/// A trait for types that can represent an input action.
/// typically an enum.
pub trait InputAction: Send + Sync + 'static + Eq + Hash + Clone + Copy {}

// Blanket implementation for any type that meets the requirements
impl<T> InputAction for T where T: Send + Sync + 'static + Eq + Hash + Clone + Copy {}

/// A map of actions to their bindings.
#[derive(Debug, Clone)]
pub struct ActionMap<A: InputAction> {
    pub bindings: HashMap<A, ActionBinding>,
}

impl<A: InputAction> Default for ActionMap<A> {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

impl<A: InputAction> Resource for ActionMap<A> {}

impl<A: InputAction> ActionMap<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, action: A, binding: ActionBinding) {
        self.bindings.insert(action, binding);
    }

    pub fn get_binding(&self, action: A) -> Option<&ActionBinding> {
        self.bindings.get(&action)
    }
}

/// Extension trait to add action querying methods to `Input`.
pub trait InputExt {
    fn action_pressed<A: InputAction>(&self, action: A, map: &ActionMap<A>) -> bool;
    fn action_just_pressed<A: InputAction>(&self, action: A, map: &ActionMap<A>) -> bool;
    fn action_just_released<A: InputAction>(&self, action: A, map: &ActionMap<A>) -> bool;

    fn action_pressed_for_player<A: InputAction>(&self, action: A, map: &ActionMap<A>, gamepad_id: u32) -> bool;
    fn action_just_pressed_for_player<A: InputAction>(&self, action: A, map: &ActionMap<A>, gamepad_id: u32) -> bool;
    fn action_just_released_for_player<A: InputAction>(&self, action: A, map: &ActionMap<A>, gamepad_id: u32) -> bool;
}

impl InputExt for Input {
    fn action_pressed<A: InputAction>(&self, action: A, map: &ActionMap<A>) -> bool {
        InputExt::action_pressed_for_player(self, action, map, self.primary_gamepad_id)
    }

    fn action_just_pressed<A: InputAction>(&self, action: A, map: &ActionMap<A>) -> bool {
        InputExt::action_just_pressed_for_player(self, action, map, self.primary_gamepad_id)
    }

    fn action_just_released<A: InputAction>(&self, action: A, map: &ActionMap<A>) -> bool {
        InputExt::action_just_released_for_player(self, action, map, self.primary_gamepad_id)
    }

    fn action_pressed_for_player<A: InputAction>(&self, action: A, map: &ActionMap<A>, gamepad_id: u32) -> bool {
        if let Some(binding) = map.bindings.get(&action) {
            binding.inputs.iter().any(|&source| self.source_pressed_internal(source, gamepad_id))
        } else {
            false
        }
    }

    fn action_just_pressed_for_player<A: InputAction>(&self, action: A, map: &ActionMap<A>, gamepad_id: u32) -> bool {
        if let Some(binding) = map.bindings.get(&action) {
            binding.inputs.iter().any(|&source| self.source_just_pressed_internal(source, gamepad_id))
        } else {
            false
        }
    }

    fn action_just_released_for_player<A: InputAction>(&self, action: A, map: &ActionMap<A>, gamepad_id: u32) -> bool {
        if let Some(binding) = map.bindings.get(&action) {
            binding.inputs.iter().any(|&source| self.source_just_released_internal(source, gamepad_id))
        } else {
            false
        }
    }
}
