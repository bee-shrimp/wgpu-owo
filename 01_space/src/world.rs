pub const RECT_SIZE: u32 = 50;
const RECT_SPEED: f32 = 60.0;

// ----------------------------------------------------------------------------------- logical size of pixel art
pub const LOGIC_WIDTH: u32 = 320;
pub const LOGIC_HEIGHT: u32 = 240;

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

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

impl Default for Pos {
    fn default() -> Self {
        Self {
            x: (LOGIC_WIDTH / 2 - RECT_SIZE / 2) as f32,
            y: (LOGIC_HEIGHT / 2 - RECT_SIZE / 2) as f32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    pos: Pos,
    speed: f32,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Pos::default(),
            speed: RECT_SPEED,
        }
    }
}

impl Rect {
    fn new(pos: Pos) -> Self {
        Self {
            pos: pos,
            speed: RECT_SPEED,
        }
    }
    fn update(&mut self, dt: f32, dir: Directions) {
        let dir_x = match dir.x {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            _ => 0.0,
        };
        let dir_y = match dir.y {
            Direction::Down => 1.0,
            Direction::Up => -1.0,
            _ => 0.0,
        };
        self.pos.x += dir_x * self.speed * dt;
        self.pos.y += dir_y * self.speed * dt;
    }
}

pub struct World {
    rect: Rect,
    is_running: bool,
}

impl Default for World {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            is_running: true,
        }
    }
}

impl World {
    pub fn new() -> Self {
        let rect = Rect::new(Pos::default());

        Self {
            rect,
            is_running: true,
        }
    }
    pub fn update(&mut self, dt: f32, dir: Directions) {
        self.rect.update(dt, dir);
    }

    pub fn toggle_running(&self) -> Self {
        Self {
            rect: self.rect,
            is_running: !self.is_running,
        }
    }
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn get_rect_pos(&self) -> Pos {
        self.rect.pos
    }
}
