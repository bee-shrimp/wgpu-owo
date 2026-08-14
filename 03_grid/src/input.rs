// ------------------------------------------------------------------- imports

use std::collections::{HashMap, HashSet};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{
    ecs::{Pos, Size},
    world::{self, LOGIC_HEIGHT, LOGIC_WIDTH},
};

// ------------------------------------------------------------------- input handler

pub struct InputHandler {
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: HashMap<MouseButton, Pos>,
    cursor_pos: Pos,
    window_size: Size,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pressed_buttons: HashMap::new(),
            cursor_pos: Pos::default(),
            window_size: Size::default(),
        }
    }
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

    pub fn resize(&mut self, window_size: Size) {
        self.window_size = window_size
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

    pub fn update_cursor_pos(&mut self, pos: Pos) {
        self.cursor_pos = pos
    }

    pub fn insert_button(&mut self, button: MouseButton) {
        println!("{:?}", self.cursor_pos);

        let world_pos = match cursor_pos_to_world_pos(&self.window_size, &self.cursor_pos) {
            Some(pos) => pos,
            None => {
                println!("none");
                return;
            }
        };

        println!("{:?}", world_pos);

        self.pressed_buttons.insert(button, world_pos);
    }

    pub fn remove_button(&mut self, button: MouseButton) {
        self.pressed_buttons.remove(&button);
    }

    pub fn get_button_pos(&self, button: MouseButton) -> &Pos {
        self.pressed_buttons
            .get(&button)
            .expect("failed to get button pos")
    }
}

fn cursor_pos_to_world_pos(window_size: &Size, cursor_pos: &Pos) -> Option<Pos> {
    let w_ratio = window_size.w as f32 / LOGIC_WIDTH as f32;
    let h_ratio = window_size.h as f32 / LOGIC_HEIGHT as f32;
    let aspect_ratio = w_ratio.min(h_ratio);

    println!("ratio: {:?}", aspect_ratio);

    let world_width = LOGIC_WIDTH as f32 * aspect_ratio;
    let world_height = LOGIC_HEIGHT as f32 * aspect_ratio;

    let offset_x = (window_size.w as f32 - world_width) / 2.0;
    let offset_y = (window_size.h as f32 - world_height) / 2.0;

    let cursor_x = cursor_pos.x as f32;
    let cursor_y = cursor_pos.y as f32;

    if cursor_x <= offset_x
        || cursor_y <= offset_y
        || cursor_x >= offset_x + world_width
        || cursor_y >= offset_y + world_height
    {
        return None;
    }

    let world_x = (cursor_x - offset_x) / aspect_ratio;
    let world_y = (cursor_y - offset_y) / aspect_ratio;

    let world_pos = Pos {
        x: world_x as f32,
        y: world_y as f32,
    };

    Some(world_pos)
}
