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
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

impl Default for Pos {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
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
            speed: 1.0,
        }
    }
}

impl Rect {
    fn new(pos: Pos) -> Self {
        Self {
            pos: pos,
            speed: 1.0,
        }
    }
    fn update(&self, dt: f32, dir: (Direction, Direction)) -> Self {
        let dt = if dt >= 0.1 { 0.1 } else { dt };
        let dir_x = match dir.0 {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            _ => 0.0,
        };
        let dir_y = match dir.1 {
            Direction::Down => -1.0,
            Direction::Up => 1.0,
            _ => 0.0,
        };
        let x = self.pos.x + dir_x * self.speed * dt;
        let y = self.pos.y + dir_y * self.speed * dt;
        let speed = self.speed;

        Self {
            pos: Pos { x, y },
            speed,
        }
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
    pub fn update(&self, dt: f32, dir: (Direction, Direction)) -> Self {
        let new_rect = self.rect.update(dt, dir);
        let is_running = self.is_running;

        Self {
            rect: new_rect,
            is_running,
        }
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
