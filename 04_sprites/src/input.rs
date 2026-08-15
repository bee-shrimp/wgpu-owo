// ------------------------------------------------------------------- imports

use std::collections::HashSet;
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{
    ecs::{Pos, Size},
    world::{LOGIC_HEIGHT, LOGIC_WIDTH},
};

// ------------------------------------------------------------------- input handler

#[derive(Default)]
pub struct InputHandler {
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: [Option<Pos>; 2], // [0] left, [1] right,
    cursor_pos: Pos,
    window_size: Size,
}

// ------------------------------------------------------------------- take input from app

impl InputHandler {
    pub fn new(window_size: Size) -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_buttons: [None, None],
            cursor_pos: Pos::default(),
            window_size,
        }
    }

    pub const fn resize(&mut self, window_size: Size) {
        self.window_size = window_size;
    }

    pub fn insert_key(&mut self, code: KeyCode) {
        self.pressed_keys.insert(code);
    }

    pub fn remove_key(&mut self, code: KeyCode) {
        self.pressed_keys.remove(&code);
    }

    pub fn has_key(&self, code: KeyCode) -> bool {
        self.pressed_keys.contains(&code)
    }

    pub const fn update_cursor_pos(&mut self, pos: Pos) {
        self.cursor_pos = pos;
    }

    pub fn insert_button(&mut self, button: MouseButton) {
        // println!("{:?}", self.cursor_pos);

        let Some(world_pos) = cursor_pos_to_world_pos(self.window_size, self.cursor_pos) else {
            // println!("none");
            return;
        };

        // println!("{:?}", world_pos);
        match button {
            MouseButton::Left => self.pressed_buttons[0] = Some(world_pos),
            MouseButton::Right => self.pressed_buttons[1] = Some(world_pos),
            _ => {}
        }
    }

    pub const fn remove_button(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.pressed_buttons[0] = None,
            MouseButton::Right => self.pressed_buttons[1] = None,
            _ => {}
        }
    }

    pub const fn get_button_pos(&self, button: MouseButton) -> Option<Pos> {
        match button {
            MouseButton::Left => self.pressed_buttons[0],
            MouseButton::Right => self.pressed_buttons[1],
            _ => None,
        }
    }
}

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
