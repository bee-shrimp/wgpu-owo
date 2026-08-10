// ------------------------------------------------------------------- imports

use std::collections::HashSet;
use winit::keyboard::KeyCode;

// ------------------------------------------------------------------- direction enum

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    #[default]
    Still,
}

// ------------------------------------------------------------------- direction x/y

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Directions {
    pub x: Direction,
    pub y: Direction,
}

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

    #[allow(unused)]
    pub fn is_still(&self) -> bool {
        self.pressed.is_empty()
    }

    #[allow(unused)]
    pub fn get_arrow_dir(&self) -> Directions {
        let direction_x = self
            .pressed
            .iter()
            .find_map(|c| match c {
                KeyCode::ArrowLeft => Some(Direction::Left),
                KeyCode::ArrowRight => Some(Direction::Right),
                _ => None,
            })
            .unwrap_or_default();

        let direction_y = self
            .pressed
            .iter()
            .find_map(|c| match c {
                KeyCode::ArrowUp => Some(Direction::Up),
                KeyCode::ArrowDown => Some(Direction::Down),
                _ => None,
            })
            .unwrap_or_default();

        Directions {
            x: direction_x,
            y: direction_y,
        }
    }

    #[allow(unused)]
    pub fn get_wasd_dir(&self) -> Directions {
        let direction_x = self
            .pressed
            .iter()
            .find_map(|c| match c {
                KeyCode::KeyA => Some(Direction::Left),
                KeyCode::KeyD => Some(Direction::Right),
                _ => None,
            })
            .unwrap_or_default();

        let direction_y = self
            .pressed
            .iter()
            .find_map(|c| match c {
                KeyCode::KeyW => Some(Direction::Up),
                KeyCode::KeyS => Some(Direction::Down),
                _ => None,
            })
            .unwrap_or_default();

        Directions {
            x: direction_x,
            y: direction_y,
        }
    }
}
