// ------------------------------------------------------------------- imports

use std::collections::{HashMap, HashSet};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{
    ecs::{Pos, Size},
    world::{LOGIC_HEIGHT, LOGIC_WIDTH},
};

// ------------------------------------------------------------------- input handler

#[derive(Default)]
pub struct InputHandler {
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: HashMap<MouseButton, Pos>,
    cursor_pos: Pos,
    window_size: Size,
}

// ------------------------------------------------------------------- take input from app

impl InputHandler {
    pub fn new(window_size: Size) -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_buttons: HashMap::new(),
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

        self.pressed_buttons.insert(button, world_pos);
    }

    pub fn remove_button(&mut self, button: MouseButton) {
        self.pressed_buttons.remove(&button);
    }

    pub fn get_button_pos(&self, button: MouseButton) -> Option<Pos> {
        self.pressed_buttons.get(&button).copied()
    }
}

fn cursor_pos_to_world_pos(window_size: Size, cursor_pos: Pos) -> Option<Pos> {
    let w_ratio = window_size.w / f32::from(LOGIC_WIDTH);
    let h_ratio = window_size.h / f32::from(LOGIC_HEIGHT);
    let aspect_ratio = w_ratio.min(h_ratio);

    let world_width = LOGIC_WIDTH as f32 * aspect_ratio;
    let world_height = LOGIC_HEIGHT as f32 * aspect_ratio;

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
