// ------------------------------------------------------------------- imports

use std::collections::HashSet;
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{
    config::{LOGIC_HEIGHT, LOGIC_WIDTH},
    ecs::{Pos, Size},
};

// ------------------------------------------------------------------- mouse state

#[derive(Default)]
struct MouseState {
    is_pressed: bool,
    was_pressed: bool,
    has_triggered: bool,
    pos: Option<Pos>,
}

// ------------------------------------------------------------------- input handler

#[derive(Default)]
pub struct InputHandler {
    pressed_keys: HashSet<KeyCode>,
    mouse_left: MouseState,
    cursor_pos: Pos,
    window_size: Size,
}

// ------------------------------------------------------------------- take input from app

impl InputHandler {
    /// creates new InputHandler.
    pub fn new(window_size: Size) -> Self {
        Self {
            pressed_keys: HashSet::new(),
            mouse_left: MouseState::default(),
            cursor_pos: Pos::default(),
            window_size,
        }
    }

    /// updates window size.
    pub fn resize(&mut self, window_size: Size) {
        self.window_size = window_size;
    }

    /// adds keycode to InputHandler.pressed_keys.
    pub fn insert_key(&mut self, code: KeyCode) {
        self.pressed_keys.insert(code);
    }

    /// removes keycode from InputHandler.pressed_keys.
    pub fn remove_key(&mut self, code: KeyCode) {
        self.pressed_keys.remove(&code);
    }

    /// returns if the keycode is in InputHandler.pressed_keys.
    pub fn has_key(&self, code: KeyCode) -> bool {
        self.pressed_keys.contains(&code)
    }

    /// updates InputHandler.cursor_pos.
    pub fn update_cursor_pos(&mut self, pos: Pos) {
        self.cursor_pos = pos;
    }

    /// detects if mouse has been clicked.
    pub fn update_mouse_state(&mut self) {
        self.mouse_left.has_triggered = !self.mouse_left.was_pressed && self.mouse_left.is_pressed;
        if self.mouse_left.has_triggered {
            self.mouse_left.pos = cursor_pos_to_world_pos(self.window_size, self.cursor_pos);
        }

        self.mouse_left.was_pressed = self.mouse_left.is_pressed;
    }

    /// updates MouseState.
    pub fn button_pressed(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => {
                self.mouse_left.is_pressed = true;
            }
            _ => {}
        }
    }

    /// updates MouseState.
    pub fn button_released(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => {
                self.mouse_left.is_pressed = false;
            }
            _ => {}
        }
    }

    /// returns if mouse was clicked.
    pub fn has_triggered(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.mouse_left.has_triggered,
            _ => false,
        }
    }

    /// returns click position.
    pub fn get_click_pos(&self, button: MouseButton) -> Option<Pos> {
        match button {
            MouseButton::Left => self.mouse_left.pos,
            _ => None,
        }
    }
}

/// converts window coord to world coord.
fn cursor_pos_to_world_pos(window_size: Size, cursor_pos: Pos) -> Option<Pos> {
    let w_ratio = window_size.w / f32::from(LOGIC_WIDTH);
    let h_ratio = window_size.h / f32::from(LOGIC_HEIGHT);
    let aspect_ratio = w_ratio.min(h_ratio);

    let world_width = f32::from(LOGIC_WIDTH) * aspect_ratio;
    let world_height = f32::from(LOGIC_HEIGHT) * aspect_ratio;

    let offset_x = (window_size.w - world_width) / 2.0;
    let offset_y = (window_size.h - world_height) / 2.0;

    let cursor_x: f32 = cursor_pos.x;
    let cursor_y: f32 = cursor_pos.y;

    if cursor_x <= offset_x
        || cursor_y <= offset_y
        || cursor_x >= offset_x + world_width
        || cursor_y >= offset_y + world_height
    {
        return None;
    }

    let world_x: f32 = (cursor_x - offset_x) / aspect_ratio;
    let world_y: f32 = (cursor_y - offset_y) / aspect_ratio;

    let world_pos = Pos {
        x: world_x,
        y: world_y,
    };

    Some(world_pos)
}
