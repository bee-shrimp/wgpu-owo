// ---------------------------------------------------------------- imports

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::MouseButton,
    keyboard::KeyCode,
};

use crate::{
    config::{LOGIC_HEIGHT, LOGIC_WIDTH},
    ecs::Pos,
};

const KEY_COUNT: usize = 255;
const MOUSE_BUTTONS: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct InputState {
    pub left_click: Option<Pos>,
    pub right_click: Option<Pos>,
    pub now_keys: [bool; KEY_COUNT],
}

// ---------------------------------------------------------------- input handler

pub struct InputHandler {
    now_keys: [bool; KEY_COUNT],
    // prev_keys: [bool; KEY_COUNT],
    now_mouse: [bool; MOUSE_BUTTONS],
    prev_mouse: [bool; MOUSE_BUTTONS],
    now_cursor_pos: PhysicalPosition<f64>,
    window_size: PhysicalSize<u32>,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self {
            now_keys: [false; KEY_COUNT],
            // prev_keys: [false; KEY_COUNT],
            now_mouse: [false; MOUSE_BUTTONS],
            prev_mouse: [false; MOUSE_BUTTONS],

            now_cursor_pos: PhysicalPosition::default(),

            window_size: PhysicalSize::default(),
        }
    }
}

// ---------------------------------------------------------------- take input from app

impl InputHandler {
    /// creates new `InputHandler`.
    pub fn new(window_size: PhysicalSize<u32>) -> Self {
        Self {
            window_size,
            ..Default::default()
        }
    }

    /// updates window size.
    pub fn resize(&mut self, window_size: PhysicalSize<u32>) {
        self.window_size = window_size;
    }

    /// adds keycode to `InputHandler.pressed_keys`.
    #[allow(clippy::as_conversions)]
    pub fn insert_key(&mut self, code: KeyCode) {
        if let Some(state) = self.now_keys.get_mut(code as usize) {
            *state = true;
        }
    }

    /// removes keycode from `InputHandler.pressed_keys`.
    #[allow(clippy::as_conversions)]
    pub fn remove_key(&mut self, code: KeyCode) {
        if let Some(state) = self.now_keys.get_mut(code as usize) {
            *state = false;
        }
    }

    /// returns if the keycode is in `InputHandler.pressed_keys`.
    #[allow(clippy::as_conversions)]
    pub fn has_key(&self, code: KeyCode) -> bool {
        self.now_keys.get(code as usize).is_some_and(|state| *state)
    }

    /// updates `InputHandler.cursor_pos`.
    pub fn update_cursor_pos(&mut self, pos: PhysicalPosition<f64>) {
        self.now_cursor_pos = pos;
    }

    /// updates `MouseState`.
    pub fn button_pressed(&mut self, button: MouseButton) {
        let idx = mouse_button_to_usize(button);
        if let Some(state) = self.now_mouse.get_mut(idx) {
            *state = true;
        }
    }

    /// updates `MouseState`.
    pub fn button_released(&mut self, button: MouseButton) {
        let idx = mouse_button_to_usize(button);
        if let Some(state) = self.now_mouse.get_mut(idx) {
            *state = false;
        }
    }

    pub fn update_for_next_frame(&mut self) {
        // self.prev_keys.copy_from_slice(&self.now_keys);
        self.prev_mouse.copy_from_slice(&self.now_mouse);
        // self.prev_cursor_pos = self.now_cursor_pos;
    }

    // /// returns if mouse was clicked.
    pub fn has_triggered(&self, button: MouseButton) -> bool {
        self.now_mouse
            .get(mouse_button_to_usize(button))
            .is_some_and(|state| *state)
            && self
                .prev_mouse
                .get(mouse_button_to_usize(button))
                .is_some_and(|state| !*state)
    }

    /// returns click position.
    pub fn get_click_pos(&self, button: MouseButton) -> Option<Pos> {
        if self.has_triggered(button) {
            cursor_pos_to_world_pos(self.window_size, self.now_cursor_pos)
        } else {
            None
        }
    }

    /// returns input state.
    pub fn get_input_state(&self) -> InputState {
        InputState {
            left_click: self.get_click_pos(MouseButton::Left),
            right_click: self.get_click_pos(MouseButton::Right),
            now_keys: self.now_keys,
        }
    }
}

/// maps `MouseButton` to usize.
fn mouse_button_to_usize(button: MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        _ => 3,
    }
}

// allow as conversion with precision loss and truncation
// since it's only used for ratio calculation.
/// maps window coord to world coord.
#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn cursor_pos_to_world_pos(
    window_size: PhysicalSize<u32>,
    cursor_pos: PhysicalPosition<f64>,
) -> Option<Pos> {
    let window_w = window_size.width as f32;
    let window_h = window_size.height as f32;

    let w_ratio = window_w / f32::from(LOGIC_WIDTH);
    let h_ratio = window_h / f32::from(LOGIC_HEIGHT);
    let aspect_ratio = w_ratio.min(h_ratio);

    let world_width = f32::from(LOGIC_WIDTH) * aspect_ratio;
    let world_height = f32::from(LOGIC_HEIGHT) * aspect_ratio;

    let offset_x = (window_w - world_width) / 2.0;
    let offset_y = (window_h - world_height) / 2.0;

    let cursor_x: f32 = cursor_pos.x as f32;
    let cursor_y: f32 = cursor_pos.y as f32;

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
