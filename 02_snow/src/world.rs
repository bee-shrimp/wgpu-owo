// ------------------------------------------------------------------- imports

use std::vec;

use anyhow::Context;

use crate::renderer::InstanceData;

// ------------------------------------------------------------------- logical size of pixel art

pub const LOGIC_WIDTH: u32 = 640;
pub const LOGIC_HEIGHT: u32 = 480;

// ------------------------------------------------------------------- consts for rect

pub const RECT_SIZE: u32 = 2;
const RECT_SPEED: u32 = 45;

const NUM_RECTS: u32 = 100;

// ------------------------------------------------------------------- struct for rects

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Size {
    w: u32,
    h: u32,
}

#[derive(Debug, Clone, Copy)]
struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    fn to_f32_array(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }
}

// ------------------------------------------------------------------- entity manager struct

struct EntityManager {
    next_id: u32,
    free_list: Vec<u32>,
}

impl EntityManager {
    // --------------------------------------------------------------- create new entity manager struct

    fn new() -> Self {
        Self {
            next_id: 0,
            free_list: Vec::new(),
        }
    }

    // --------------------------------------------------------------- return id

    fn spawn(&mut self) -> u32 {
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

struct Components {
    positions: Vec<Option<Pos>>,
    sizes: Vec<Option<Size>>,
    colours: Vec<Option<Colour>>,
    max_entities: usize,
}

impl Components {
    // --------------------------------------------------------------- create new components struct

    fn new() -> Self {
        let max_entities = NUM_RECTS as usize;
        Self {
            positions: vec![None; max_entities],
            sizes: vec![None; max_entities],
            colours: vec![None; max_entities],
            max_entities,
        }
    }

    // --------------------------------------------------------------- add Some(x) to vec[id]

    fn add_position(&mut self, id: u32, pos: Pos) -> anyhow::Result<()> {
        if id >= self.max_entities as u32 {
            anyhow::bail!("entity id over flow")
        }

        self.positions[id as usize] = Some(pos);
        Ok(())
    }

    fn add_size(&mut self, id: u32, size: Size) -> anyhow::Result<()> {
        if id >= self.max_entities as u32 {
            anyhow::bail!("entity id over flow")
        }
        self.sizes[id as usize] = Some(size);

        Ok(())
    }

    fn add_colour(&mut self, id: u32, colour: Colour) -> anyhow::Result<()> {
        if id >= self.max_entities as u32 {
            anyhow::bail!("entity id over flow")
        }
        self.colours[id as usize] = Some(colour);

        Ok(())
    }

    // --------------------------------------------------------------- get Some(x) in vec[id]

    fn get_position(&self, id: u32) -> Option<&Pos> {
        self.positions.get(id as usize).and_then(|p| p.as_ref())
    }

    fn get_position_mut(&mut self, id: u32) -> Option<&mut Pos> {
        self.positions.get_mut(id as usize).and_then(|p| p.as_mut())
    }

    fn get_size(&self, id: u32) -> Option<&Size> {
        self.sizes.get(id as usize).and_then(|s| s.as_ref())
    }

    fn get_colour(&self, id: u32) -> Option<&Colour> {
        self.colours.get(id as usize).and_then(|c| c.as_ref())
    }
}

// ------------------------------------------------------------------- movement system struct

struct MovementSystem;

impl MovementSystem {
    fn update(components: &mut Components, dt: f32, time: f32) {
        for id in 0..components.max_entities {
            // ------------------------------------------------------- get data from components

            let pos = match components.get_position(id as u32) {
                Some(p) => p,
                None => continue,
            };

            let size = match components.get_size(id as u32) {
                Some(s) => s,
                None => continue,
            };

            // ------------------------------------------------------- calc new pos

            let new_pos = calc_pos(id as u32, pos, size, dt, time);

            // ------------------------------------------------------- apply new pos

            *components
                .get_position_mut(id as u32)
                .expect("failed to get position mut") = new_pos;
        }
    }
}

// ------------------------------------------------------------------- world struct

pub struct World {
    entity_manager: EntityManager,
    components: Components,
    instances: Vec<InstanceData>,
    time: f32,
    is_running: bool,
}

// ------------------------------------------------------------------- default empty world

impl Default for World {
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            components: Components::new(),
            instances: Vec::new(),
            time: 0.0,
            is_running: true,
        }
    }
}

impl World {
    // --------------------------------------------------------------- init world with rects

    pub fn init(&mut self) -> anyhow::Result<()> {
        create_rects(&mut self.entity_manager, &mut self.components)
            .context("failed to create rects")?;
        Ok(())
    }

    pub fn update(&mut self, dt: f32) {
        // ----------------------------------------------------------- update rect pos

        MovementSystem::update(&mut self.components, dt, self.time);

        // ----------------------------------------------------------- update instance data

        self.update_instances();

        // ----------------------------------------------------------- update time count

        self.time += dt;
    }

    pub fn update_instances(&mut self) {
        self.instances.clear();
        self.instances = build_instance_data(&self.components)
    }

    pub fn toggle_running(&mut self) {
        self.is_running = !self.is_running
    }

    pub fn get_instances(&self) -> &[InstanceData] {
        &self.instances
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

fn create_rects(
    entity_manager: &mut EntityManager,
    components: &mut Components,
) -> anyhow::Result<()> {
    for _ in 0..components.max_entities {
        let id = entity_manager.spawn();

        components
            .add_position(
                id,
                Pos {
                    x: rand::random_range(0.0..LOGIC_WIDTH as f32 - RECT_SIZE as f32),
                    y: rand::random_range(0.0..LOGIC_HEIGHT as f32 - RECT_SIZE as f32),
                },
            )
            .context("failed to add position")?;

        components
            .add_size(
                id,
                Size {
                    w: RECT_SIZE,
                    h: RECT_SIZE,
                },
            )
            .context("failed to add size")?;

        components
            .add_colour(
                id,
                Colour {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            )
            .context("failed to add colour")?;
    }
    Ok(())
}

fn build_instance_data(components: &Components) -> Vec<InstanceData> {
    let mut instances = Vec::new();

    for id in 0..components.max_entities {
        let pos = match components.get_position(id as u32) {
            Some(p) => p,
            None => continue,
        };

        let size = match components.get_size(id as u32) {
            Some(s) => s,
            None => continue,
        };

        let colour = match components.get_colour(id as u32) {
            Some(c) => c,
            None => continue,
        };

        instances.push(InstanceData {
            position: [pos.x as f32, pos.y as f32],
            colour: colour.to_f32_array(),
            size: [size.w as f32, size.h as f32],
        });
    }
    instances
}

fn calc_pos(id: u32, pos: &Pos, size: &Size, dt: f32, time: f32) -> Pos {
    let x: f32;
    let y: f32;

    // ----------------------------------------------------------- replace to top (w/ random x) if out of boundary

    if pos.y >= LOGIC_HEIGHT as f32 {
        x = rand::random_range(0..LOGIC_WIDTH - RECT_SIZE as u32) as f32;
        y = -(size.h as f32);

    // ----------------------------------------------------------- wavy move + fall down
    //
    } else {
        let offset_x = f32::sin(time + (id as f32 * 0.1)) * 0.1;
        x = pos.x + offset_x;
        y = pos.y + dt * RECT_SPEED as f32
    }

    Pos { x, y }
}
