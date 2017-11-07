use std::collections::HashMap;

/// Holds the keyboard state.
pub struct KeyboardState {
    /// `true` is pressed, `false` is released
    map: HashMap<u32, bool>,
}

impl KeyboardState {
    pub fn new() -> KeyboardState {
        KeyboardState {
            map: HashMap::new(),
        }
    }

    pub fn is_key_pressed(&self, keycode: u32) -> bool {
        *self.map.get(&keycode).unwrap_or(&false)
    }

    pub fn update_key(&mut self, keycode: u32, pressed: bool) {
        self.map.insert(keycode, pressed);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    // TODO: Interact with the OS's mapping to get the name of every keycode
}