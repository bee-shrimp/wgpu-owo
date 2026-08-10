// ------------------------------------------------------------------- imports

use crate::input::{Direction, Directions};

// ------------------------------------------------------------------- consts for rect

pub const RECT_SIZE: u32 = 50;
const RECT_SPEED: f32 = 60.0;

// ------------------------------------------------------------------- logical size of pixel art

pub const LOGIC_WIDTH: u32 = 320;
pub const LOGIC_HEIGHT: u32 = 240;

// ------------------------------------------------------------------- epsilon for rect movement

pub const EPSILON: f32 = 0.001;

// ------------------------------------------------------------------- struct for rect position

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

// ------------------------------------------------------------------- default pos is the center

impl Default for Pos {
    fn default() -> Self {
        Self {
            x: (LOGIC_WIDTH / 2 - RECT_SIZE / 2) as f32,
            y: (LOGIC_HEIGHT / 2 - RECT_SIZE / 2) as f32,
        }
    }
}

// ------------------------------------------------------------------- rect struct

#[derive(Debug, Clone, Copy)]
struct Rect {
    pos: Pos,
    speed: f32,
}

// ------------------------------------------------------------------- default rect

impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Pos::default(),
            speed: RECT_SPEED,
        }
    }
}

impl Rect {
    // --------------------------------------------------------------- apply new pos
    fn update(&mut self, pos: Pos) {
        self.pos = pos
    }

    // --------------------------------------------------------------- calculate velosity

    fn calc_velosity(&self, dir: Directions) -> (f32, f32) {
        let dx = match dir.x {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            _ => 0.0,
        };
        let dy = match dir.y {
            Direction::Down => 1.0,
            Direction::Up => -1.0,
            _ => 0.0,
        };
        (dx, dy)
    }

    // --------------------------------------------------------------- calculate new pos

    fn calc_pos(&self, dt: f32, dir: Directions) -> Pos {
        let (dx, dy) = self.calc_velosity(dir);
        let x =
            (self.pos.x + dx * self.speed * dt).clamp(0.0, LOGIC_WIDTH as f32 - RECT_SIZE as f32);
        let y =
            (self.pos.y + dy * self.speed * dt).clamp(0.0, LOGIC_HEIGHT as f32 - RECT_SIZE as f32);
        Pos { x, y }
    }
}

// ------------------------------------------------------------------- world struct

pub struct World {
    rect: Rect,
    is_running: bool,
}

// ------------------------------------------------------------------- default world w/ rect at the center

impl Default for World {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            is_running: true,
        }
    }
}

impl World {
    pub fn update(&mut self, dt: f32, arrow_dir: Directions) {
        let rect_pos = self.rect.calc_pos(dt, arrow_dir);

        self.rect.update(rect_pos);
    }

    pub fn toggle_running(&mut self) {
        self.is_running = !self.is_running
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn get_rect_pos(&self) -> Pos {
        self.rect.pos
    }
}
