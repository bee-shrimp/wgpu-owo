// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::ecs::{Colour, Components, EntityManager, MovementSystem, Pos, Size, Vel};
use crate::renderer::InstanceData;

// ------------------------------------------------------------------- logical size of pixel art

pub const LOGIC_WIDTH: u32 = 640;
pub const LOGIC_HEIGHT: u32 = 480;

// ------------------------------------------------------------------- consts for rect

pub const RECT_SIZE: u32 = 2;
pub const NUM_ENTITIES: u32 = 100;

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
        self.update_instances();
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
        // ----------------------------------------------------------- get usable id

        let id = entity_manager.spawn();

        // ----------------------------------------------------------- add data to vec[id]

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
            .add_velocity(id, Vel { dx: 0.1, dy: 60.0 })
            .context("failed to add velocity")?;

        components
            .add_size(
                id,
                Size {
                    w: RECT_SIZE + (id / 10),
                    h: RECT_SIZE + (id / 10),
                },
            )
            .context("failed to add size")?;

        components
            .add_colour(
                id,
                Colour {
                    r: rand::random_range(200..250),
                    g: rand::random_range(50..100),
                    b: rand::random_range(200..250),
                },
            )
            .context("failed to add colour")?;
    }
    Ok(())
}

fn build_instance_data(components: &Components) -> Vec<InstanceData> {
    let mut instances = Vec::new();

    for id in 0..components.max_entities {
        // ----------------------------------------------------------- get data from vec[id]

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

        // ----------------------------------------------------------- build InstanceData

        instances.push(InstanceData {
            position: [pos.x as f32, pos.y as f32],
            colour: colour.to_f32_array(),
            size: [size.w as f32, size.h as f32],
        });
    }

    instances
}
