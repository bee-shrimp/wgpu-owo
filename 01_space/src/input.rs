use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    #[default]
    Still,
}

#[derive(Debug, Clone, Copy)]
pub struct Directions {
    pub x: Direction,
    pub y: Direction,
}

pub struct InputHandler {
    pressed_keys: HashSet<KeyCode>,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }
}
impl InputHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, code: KeyCode) {
        self.pressed_keys.insert(code);
    }

    pub fn remove(&mut self, code: &KeyCode) {
        self.pressed_keys.remove(code);
    }

    pub fn get_pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }

    pub fn get_directions(&self) -> Directions {
        let mut direction_x: Direction = Direction::Still;
        let mut direction_y: Direction = Direction::Still;

        if self.pressed_keys.contains(&KeyCode::ArrowUp) {
            direction_y = Direction::Up
        }

        if self.pressed_keys.contains(&KeyCode::ArrowLeft) {
            direction_x = Direction::Left
        }
        if self.pressed_keys.contains(&KeyCode::ArrowDown) {
            direction_y = Direction::Down
        }

        if self.pressed_keys.contains(&KeyCode::ArrowRight) {
            direction_x = Direction::Right
        }

        Directions {
            x: direction_x,
            y: direction_y,
        }
    }
}
