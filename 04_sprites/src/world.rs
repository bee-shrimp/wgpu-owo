// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::ecs::{Components, EntityManager, Pos, ReactSystem, Size};
use crate::renderer::InstanceData;

// ------------------------------------------------------------------- logical size of pixel art

pub const LOGIC_WIDTH: u8 = 128;
pub const LOGIC_HEIGHT: u8 = 128;

// ------------------------------------------------------------------- number of col and row

pub const MAX_COL: u8 = 4;
pub const MAX_ROW: u8 = 4;

// ------------------------------------------------------------------- consts for rect

pub const RECT_WIDTH: u8 = LOGIC_WIDTH / MAX_COL;

pub const RECT_HEIGHT: u8 = LOGIC_HEIGHT / MAX_ROW;

pub const NUM_RECTS: u8 = MAX_COL * MAX_ROW;

// ------------------------------------------------------------------- sprite

const SPRITE_SHEET_SIZE: u8 = 255;
const SPRITE_GRID_SIZE: u8 = 16;

// ------------------------------------------------------------------- struct for sprites

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Sprite {
    RedFlower,    // 0
    YellowFlower, // 1
}

#[derive(Debug, Clone, Copy)]
struct UVOffset {
    u: f32,
    v: f32,
}

#[derive(Debug, Clone, Copy)]
struct SpriteData {
    uv_offset: UVOffset,
    uv_size: Size,
}

impl SpriteData {
    fn new(g_pos: GridPos) -> Self {
        let sprite_sheet_size = f32::from(SPRITE_SHEET_SIZE);
        let grid_size = f32::from(SPRITE_GRID_SIZE);

        let cell_size = sprite_sheet_size / grid_size;

        let pixel_u = f32::from(g_pos.gx) * cell_size;
        let pixel_v = f32::from(g_pos.gy) * cell_size;

        let uv_offset_u = pixel_u / sprite_sheet_size;
        let uv_offset_v = pixel_v / sprite_sheet_size;

        let uv_size = cell_size / sprite_sheet_size;

        Self {
            uv_offset: UVOffset {
                u: uv_offset_u,
                v: uv_offset_v,
            },
            uv_size: Size {
                w: uv_size,
                h: uv_size,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GridPos {
    gx: u8,
    gy: u8,
}

// ------------------------------------------------------------------- world struct

pub struct World {
    entity_manager: EntityManager,
    components: Components,
    instances: Vec<InstanceData>,
    sprite_data: [SpriteData; 2],
    is_running: bool,
}

// ------------------------------------------------------------------- default empty world

impl Default for World {
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            components: Components::new(),
            instances: Vec::new(),
            sprite_data: [
                SpriteData::new(GridPos { gx: 0, gy: 0 }),
                SpriteData::new(GridPos { gx: 1, gy: 0 }),
            ],
            is_running: true,
        }
    }
}

impl World {
    // --------------------------------------------------------------- init world with rects

    pub fn init(&mut self) -> anyhow::Result<()> {
        create_entities(&mut self.entity_manager, &mut self.components)
            .context("failed to create rects")?;

        self.update_instances()?;
        Ok(())
    }

    pub fn update(&mut self, click_pos: Pos) -> anyhow::Result<()> {
        ReactSystem::update(&mut self.components, click_pos)?;

        self.update_instances()?;

        Ok(())
    }

    pub fn update_instances(&mut self) -> anyhow::Result<()> {
        self.instances.clear();
        self.instances
            .extend(build_instance_data(&self.components, self.sprite_data)?);

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

fn create_entities(
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

        if id.is_multiple_of(2) {
            components
                .add_sprite(id, Sprite::RedFlower)
                .context("failed to add sprite data from hashmap")?;
        } else {
            components
                .add_sprite(id, Sprite::YellowFlower)
                .context("failed to add sprite data from hashmap")?;
        }
    }
    Ok(())
}

fn build_instance_data(
    components: &Components,
    sprite_data: [SpriteData; 2],
) -> anyhow::Result<Vec<InstanceData>> {
    let mut instances = Vec::new();

    for id in 0..components.max_entities {
        // ----------------------------------------------------------- get data from vec[id]

        let Some(position) = components.get_position(id) else {
            continue;
        };

        let Some(size) = components.get_size(id) else {
            continue;
        };

        let Some(sprite) = components.get_sprite(id) else {
            continue;
        };

        let sprite_data = match sprite {
            Sprite::RedFlower => sprite_data[0],
            Sprite::YellowFlower => sprite_data[1],
        };

        // ----------------------------------------------------------- build InstanceData

        instances.push(InstanceData {
            position: [position.x, position.y],
            size: [size.w, size.h],
            sprite_offset: [sprite_data.uv_offset.u, sprite_data.uv_offset.v],
            sprite_size: [sprite_data.uv_size.w, sprite_data.uv_size.h],
        });
    }

    Ok(instances)
}
