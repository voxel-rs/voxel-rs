use piston::input::keyboard::Key;

use std::collections::HashMap;

/// Holds the keyboard state.
pub struct KeyboardState {
    /// `true` is pressed, `false` is released
    map: HashMap<Key, bool>,
}

impl KeyboardState {
    pub fn new() -> KeyboardState {
        KeyboardState {
            map: HashMap::new(),
        }
    }

    pub fn is_key_pressed(&self, keycode: Key) -> bool {
        *self.map.get(&keycode).unwrap_or(&false)
    }

    pub fn update_key(&mut self, keycode: Key, pressed: bool) {
        self.map.insert(keycode, pressed);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    // TODO: Interact with the OS's mapping to get the name of every keycode
}