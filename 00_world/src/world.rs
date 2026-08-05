#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub pos_x: f32,
    pub pos_y: f32,
    pub speed: f32,
}

impl Rect {
    pub fn new() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            speed: 1.0,
        }
    }
    pub fn update(&mut self, dt: f32, dir: (Direction, Direction)) -> (f32, f32) {
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
        self.pos_x += dir_x * self.speed * dt;
        self.pos_y += dir_y * self.speed * dt;
        (self.pos_x, self.pos_y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    #[default]
    Still,
}
