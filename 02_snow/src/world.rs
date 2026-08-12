// ------------------------------------------------------------------- imports

use crate::renderer::InstanceData;

// ------------------------------------------------------------------- logical size of pixel art

pub const LOGIC_WIDTH: u32 = 640;
pub const LOGIC_HEIGHT: u32 = 480;

// ------------------------------------------------------------------- consts for rect

pub const RECT_SIZE: u8 = 2;
const RECT_SPEED: u8 = 45;
const NUM_RECTS: u8 = 100;

// ------------------------------------------------------------------- struct for rects

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Colour(u8, u8, u8);

#[derive(Debug, Clone, Copy)]
struct Size {
    w: u8,
    h: u8,
}

// ------------------------------------------------------------------- rect struct

#[derive(Debug, Clone, Copy)]
struct Rect {
    size: Size,
    colour: Colour,
    pos: Pos,
    speed: f32,
    id: u8,
}

impl Rect {
    // --------------------------------------------------------------- apply new pos

    fn update(&mut self, pos: Pos) {
        self.pos = pos
    }

    // --------------------------------------------------------------- calculate new pos

    fn calc_pos(&self, dt: f32, time: f32) -> Pos {
        let x: f32;
        let y: f32;

        // ----------------------------------------------------------- replace to top (w/ random x) if out of boundary
        if self.pos.y >= LOGIC_HEIGHT as f32 {
            x = rand::random_range(0..LOGIC_WIDTH - RECT_SIZE as u32) as f32;
            y = -(self.size.h as f32);

        // ----------------------------------------------------------- wavy move + fall down
        } else {
            let offset_x = f32::sin(time + (self.id as f32 * 0.1)) * 0.1;
            x = self.pos.x + offset_x;
            y = self.pos.y + dt * self.speed
        }

        Pos { x, y }
    }
}

// ------------------------------------------------------------------- world struct

pub struct World {
    rects: Vec<Rect>,
    instances: Vec<InstanceData>,
    time: f32,
    is_running: bool,
}

// ------------------------------------------------------------------- default world w/ rects

impl Default for World {
    fn default() -> Self {
        let rects = create_rects();

        Self {
            rects,
            instances: Vec::new(),
            time: 0.0,
            is_running: true,
        }
    }
}

impl World {
    pub fn update(&mut self, dt: f32) {
        for rect in &mut self.rects {
            let new_pos = rect.calc_pos(dt, self.time);
            rect.update(new_pos);
        }

        self.time += dt;
        self.update_instances();
    }

    pub fn toggle_running(&mut self) {
        self.is_running = !self.is_running
    }

    fn update_instances(&mut self) {
        self.instances.clear();

        for r in &self.rects {
            self.instances.push(build_instance_data(&r))
        }
    }

    pub fn get_instances(&self) -> &[InstanceData] {
        &self.instances
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

fn create_rects() -> Vec<Rect> {
    (0..NUM_RECTS)
        .into_iter()
        .map(|i| Rect {
            id: i,
            pos: Pos {
                x: rand::random_range(0..LOGIC_WIDTH as i32 - RECT_SIZE as i32) as f32,
                y: rand::random_range(0..LOGIC_HEIGHT as i32 - RECT_SIZE as i32) as f32,
            },
            colour: Colour(255, 255, 255),

            size: Size {
                w: RECT_SIZE + i / 10,
                h: RECT_SIZE + i / 10,
            },
            speed: RECT_SPEED as f32 + (i as f32 * 0.01),
        })
        .collect::<Vec<Rect>>()
}

fn build_instance_data(r: &Rect) -> InstanceData {
    InstanceData {
        position: [r.pos.x as f32, r.pos.y as f32],
        colour: [
            (r.colour.0 / 255) as f32,
            (r.colour.1 / 255) as f32,
            (r.colour.2 / 255) as f32,
        ],
        size: [r.size.w as f32, r.size.h as f32],
    }
}
