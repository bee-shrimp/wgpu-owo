// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::config::{
    MAX_COL, MAX_ENTITIES, RECT_HEIGHT, RECT_WIDTH, SPRITE_GRID_SIZE, SPRITE_SHEET_SIZE,
};
use crate::ecs::{Components, EntityManager, Pos, ReactSystem, Size};
use crate::renderer::InstanceData;

// ------------------------------------------------------------------- enum of sprites

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Sprite {
    RedFlower,    // 0
    YellowFlower, // 1
}

// ------------------------------------------------------------------- return SpriteData of sprite

impl Sprite {
    fn uv_data(self) -> SpriteData {
        match self {
            Sprite::RedFlower => SpriteData::new(GridPos { gx: 0, gy: 0 }),
            Sprite::YellowFlower => SpriteData::new(GridPos { gx: 1, gy: 0 }),
        }
    }
}

// ------------------------------------------------------------------- SpriteData struct

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

// ------------------------------------------------------------------- structs for SpriteData

#[derive(Debug, Clone, Copy)]
struct UVOffset {
    u: f32,
    v: f32,
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
    // --------------------------------------------------------------- init world with entities

    pub fn init(&mut self) -> anyhow::Result<()> {
        create_entities(&mut self.entity_manager, &mut self.components)
            .context("failed to create entities")?;

        self.instances.reserve(MAX_ENTITIES);

        self.update_instances()?;
        Ok(())
    }

    // --------------------------------------------------------------- update

    pub fn update(&mut self, click_pos: Pos, _dt: f32) -> anyhow::Result<()> {
        ReactSystem::update(&mut self.components, click_pos)?;

        self.update_instances()?;

        Ok(())
    }

    pub fn update_instances(&mut self) -> anyhow::Result<()> {
        self.instances.clear();

        let new_instances = build_instance_data(&self.components)?;
        self.instances.extend(new_instances);

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
    for _ in 0..MAX_ENTITIES {
        // ----------------------------------------------------------- get usable id

        let entity = entity_manager.spawn().context("no free slot")?;

        // ----------------------------------------------------------- add data to vec[id]
        components
            .add_alive(entity)
            .context("failed to add alive")?;

        components
            .add_position(
                entity,
                Pos {
                    x: f32::from(RECT_WIDTH * (entity.index as u8 % MAX_COL)),
                    y: f32::from(RECT_HEIGHT * (entity.index as u8 / MAX_COL)),
                },
            )
            .context("failed to add position")?;

        components
            .add_size(
                entity,
                Size {
                    w: f32::from(RECT_WIDTH),
                    h: f32::from(RECT_HEIGHT),
                },
            )
            .context("failed to add size")?;

        if entity.index.is_multiple_of(2) {
            components
                .add_sprite(entity, Sprite::RedFlower)
                .context("failed to add sprite data from hashmap")?;
        } else {
            components
                .add_sprite(entity, Sprite::YellowFlower)
                .context("failed to add sprite data from hashmap")?;
        }
    }
    Ok(())
}

pub fn build_instance_data(components: &Components) -> anyhow::Result<Vec<InstanceData>> {
    Ok((0..MAX_ENTITIES)
        .filter_map(|id| build_instance_for_entity(id, components))
        .collect())
}

pub fn build_instance_for_entity(id: usize, components: &Components) -> Option<InstanceData> {
    if !components.alive[id] {
        return None;
    }

    let position = components.positions[id];
    let size = components.sizes[id];
    let sprite_data = components.sprites[id].uv_data();

    Some(InstanceData {
        position: [position.x, position.y],
        size: [size.w, size.h],
        sprite_offset: [sprite_data.uv_offset.u, sprite_data.uv_offset.v],
        sprite_size: [sprite_data.uv_size.w, sprite_data.uv_size.h],
    })
}
