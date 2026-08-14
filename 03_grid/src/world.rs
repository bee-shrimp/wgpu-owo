// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::ecs::{Colour, Components, EntityManager, Pos, ReactSystem, Size};
use crate::renderer::InstanceData;

// ------------------------------------------------------------------- logical size of pixel art

pub const LOGIC_WIDTH: u8 = 166;
pub const LOGIC_HEIGHT: u8 = 140;

// ------------------------------------------------------------------- number of col and row

pub const MAX_COL: u8 = 4;
pub const MAX_ROW: u8 = 3;

// ------------------------------------------------------------------- consts for rect

pub const RECT_WIDTH: u8 = LOGIC_WIDTH / MAX_COL;

pub const RECT_HEIGHT: u8 = LOGIC_HEIGHT / MAX_ROW;

pub const NUM_GRIDS: u8 = MAX_COL * MAX_ROW;

// ------------------------------------------------------------------- world struct

pub struct World {
    entity_manager: EntityManager,
    components: Components,
    instances: Vec<InstanceData>,
    is_running: bool,
}

// ------------------------------------------------------------------- default empty world

impl Default for World {
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            components: Components::new(),
            instances: Vec::new(),
            is_running: true,
        }
    }
}

impl World {
    // --------------------------------------------------------------- init world with rects

    pub fn init(&mut self) -> anyhow::Result<()> {
        create_rects(&mut self.entity_manager, &mut self.components)
            .context("failed to create rects")?;
        self.update_instances()?;
        Ok(())
    }

    pub fn update(&mut self, click_pos: Pos) -> anyhow::Result<()> {
        // ----------------------------------------------------------- update instance data
        // println!("{:?}", click_pos);
        ReactSystem::update(&mut self.components, click_pos)?;

        self.update_instances()?;

        Ok(())
    }

    pub fn update_instances(&mut self) -> anyhow::Result<()> {
        self.instances.clear();
        self.instances =
            build_instance_data(&self.components).context("failed to build instance data")?;
        Ok(())
    }

    pub const fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn get_instances(&self) -> &[InstanceData] {
        &self.instances
    }

    pub const fn is_running(&self) -> bool {
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
                    x: f32::from(RECT_WIDTH * (id % MAX_COL)),
                    y: f32::from(RECT_HEIGHT * (id / MAX_COL)),
                },
            )
            .context("failed to add position")?;

        components
            .add_size(
                id,
                Size {
                    w: f32::from(RECT_WIDTH),
                    h: f32::from(RECT_HEIGHT),
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

fn build_instance_data(components: &Components) -> anyhow::Result<Vec<InstanceData>> {
    let mut instances = Vec::new();

    for id in 0..components.max_entities {
        // ----------------------------------------------------------- get data from vec[id]

        let Some(position) = components.get_position(u8::try_from(id).context("conversion error")?)
        else {
            continue;
        };

        let Some(size) = components.get_size(u8::try_from(id).context("conversion error")?) else {
            continue;
        };

        let Some(colour) = components.get_colour(u8::try_from(id).context("conversion error")?)
        else {
            continue;
        };

        // ----------------------------------------------------------- build InstanceData

        instances.push(InstanceData {
            position: [position.x, position.y],
            colour: colour.to_f32_array(),
            size: [size.w, size.h],
        });
    }

    Ok(instances)
}
