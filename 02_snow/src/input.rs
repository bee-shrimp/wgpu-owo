// ------------------------------------------------------------------- imports

use std::collections::HashSet;
use winit::keyboard::KeyCode;

// ------------------------------------------------------------------- input handler

pub struct InputHandler {
    pressed: HashSet<KeyCode>,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }
}

// ------------------------------------------------------------------- take input from app

impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, code: KeyCode) {
        self.pressed.insert(code);
    }

    pub fn remove(&mut self, code: KeyCode) {
        self.pressed.remove(&code);
    }

    pub fn has(&self, code: KeyCode) -> bool {
        self.pressed.contains(&code)
    }
}
