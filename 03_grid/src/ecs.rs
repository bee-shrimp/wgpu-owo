// ------------------------------------------------------------------- imports

use crate::world::{LOGIC_HEIGHT, LOGIC_WIDTH, NUM_ENTITIES};

// ------------------------------------------------------------------- struct for rects

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
pub struct Vel {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Default for Size {
    fn default() -> Self {
        Self { w: 0, h: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub fn to_f32_array(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }
}

// ------------------------------------------------------------------- entity manager struct

pub struct EntityManager {
    next_id: u32,
    free_list: Vec<u32>,
}

impl EntityManager {
    // --------------------------------------------------------------- create new entity manager struct

    pub fn new() -> Self {
        Self {
            next_id: 0,
            free_list: Vec::new(),
        }
    }

    // --------------------------------------------------------------- return id

    pub fn spawn(&mut self) -> u32 {
        if let Some(id) = self.free_list.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    // --------------------------------------------------------------- store id for reuse

    // fn despawn(&mut self, id: u32) {
    //     self.free_list.push(id);
    // }
}

// ------------------------------------------------------------------- components struct

pub struct Components {
    positions: Vec<Option<Pos>>,
    velocities: Vec<Option<Vel>>,
    sizes: Vec<Option<Size>>,
    colours: Vec<Option<Colour>>,
    pub max_entities: usize,
}

impl Components {
    // --------------------------------------------------------------- create new components struct

    pub fn new() -> Self {
        let max_entities = NUM_ENTITIES as usize;
        Self {
            positions: vec![None; max_entities],
            velocities: vec![None; max_entities],
            sizes: vec![None; max_entities],
            colours: vec![None; max_entities],
            max_entities,
        }
    }

    // --------------------------------------------------------------- add Some(x) to vec[id]

    pub fn add_position(&mut self, id: u32, pos: Pos) -> anyhow::Result<()> {
        if id >= self.max_entities as u32 {
            anyhow::bail!("entity id over flow")
        }

        self.positions[id as usize] = Some(pos);
        Ok(())
    }

    pub fn add_velocity(&mut self, id: u32, vel: Vel) -> anyhow::Result<()> {
        if id >= self.max_entities as u32 {
            anyhow::bail!("entity id over flow")
        }

        self.velocities[id as usize] = Some(vel);
        Ok(())
    }

    pub fn add_size(&mut self, id: u32, size: Size) -> anyhow::Result<()> {
        if id >= self.max_entities as u32 {
            anyhow::bail!("entity id over flow")
        }
        self.sizes[id as usize] = Some(size);

        Ok(())
    }

    pub fn add_colour(&mut self, id: u32, colour: Colour) -> anyhow::Result<()> {
        if id >= self.max_entities as u32 {
            anyhow::bail!("entity id over flow")
        }
        self.colours[id as usize] = Some(colour);

        Ok(())
    }

    // --------------------------------------------------------------- get Some(x) in vec[id]

    pub fn get_position(&self, id: u32) -> Option<&Pos> {
        self.positions.get(id as usize).and_then(|p| p.as_ref())
    }

    pub fn get_position_mut(&mut self, id: u32) -> Option<&mut Pos> {
        self.positions.get_mut(id as usize).and_then(|p| p.as_mut())
    }

    pub fn get_velocity(&self, id: u32) -> Option<&Vel> {
        self.velocities.get(id as usize).and_then(|v| v.as_ref())
    }

    pub fn get_size(&self, id: u32) -> Option<&Size> {
        self.sizes.get(id as usize).and_then(|s| s.as_ref())
    }

    pub fn get_colour(&self, id: u32) -> Option<&Colour> {
        self.colours.get(id as usize).and_then(|c| c.as_ref())
    }
}

// ------------------------------------------------------------------- movement system struct

pub struct MovementSystem;

impl MovementSystem {
    pub fn update(components: &mut Components, dt: f32, time: f32) {
        for id in 0..components.max_entities {
            // ------------------------------------------------------- get data from components

            let pos = match components.get_position(id as u32) {
                Some(p) => p,
                None => continue,
            };

            let vel = match components.get_velocity(id as u32) {
                Some(v) => v,
                None => continue,
            };

            let size = match components.get_size(id as u32) {
                Some(s) => s,
                None => continue,
            };

            // ------------------------------------------------------- calc new pos

            let new_pos = calc_pos(id as u32, pos, vel, size, dt, time);

            // ------------------------------------------------------- apply new pos

            *components
                .get_position_mut(id as u32)
                .expect("failed to get position mut") = new_pos;
        }
    }
}

fn calc_pos(id: u32, pos: &Pos, vel: &Vel, size: &Size, dt: f32, time: f32) -> Pos {
    let x: f32;
    let y: f32;

    // ----------------------------------------------------------- replace to top (w/ random x) if out of boundary

    if pos.y >= LOGIC_HEIGHT as f32 {
        x = rand::random_range(0..LOGIC_WIDTH - size.w as u32) as f32;
        y = -(size.h as f32);

    // ----------------------------------------------------------- wavy move + fall down
    //
    } else {
        let offset_x = f32::sin(time + (id as f32 * vel.dx)) * 0.1;
        x = pos.x + offset_x;
        y = pos.y + dt * vel.dy as f32
    }

    Pos { x, y }
}
