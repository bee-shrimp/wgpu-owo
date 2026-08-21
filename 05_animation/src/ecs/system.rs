use crate::ecs::entity::{Entity, EntityManager};
use crate::ecs::{Components, Pos, Size};
use anyhow::Context;

use crate::config::{MAX_COL, MAX_ENTITIES, RECT_HEIGHT, RECT_WIDTH};
use crate::renderer::InstanceData;
use crate::sprite::{Animation, Sprite};

// ------------------------------------------------------------------- create entity struct

pub struct CreateEntitySystem;

impl CreateEntitySystem {
    /// adds data to `ComponentStorage`[[entity.index]].
    /// uses `MAX_ENTITIES` to determine the number of entities.
    pub fn create_grid_entities(
        entity_manager: &mut EntityManager,
        components: &mut Components,
        sprite: Sprite,
    ) -> anyhow::Result<()> {
        for _ in 0..MAX_ENTITIES {
            // ----------------------------------------------------------- get usable id

            let entity = entity_manager.spawn().context("no free slot")?;

            // ----------------------------------------------------------- add data to ComponentStorage[entity.index]
            components
                .positions
                .insert(
                    entity,
                    Pos {
                        x: f32::from(RECT_WIDTH * (entity.index as u8 % MAX_COL)),
                        y: f32::from(RECT_HEIGHT * (entity.index as u8 / MAX_COL)),
                    },
                )
                .context("failed to add position")?;

            components
                .sizes
                .insert(
                    entity,
                    Size {
                        w: f32::from(RECT_WIDTH),
                        h: f32::from(RECT_HEIGHT),
                    },
                )
                .context("failed to add size")?;

            components
                .sprites
                .insert(entity, sprite)
                .context("failed to add sprite data")?;

            components
                .flames
                .insert(entity, 0)
                .context("failed to add flame")?;

            components
                .elapsed
                .insert(entity, 0.0)
                .context("failed to add elapsed")?;
        }
        Ok(())
    }
}

// ------------------------------------------------------------------- instance update system struct

#[derive(Debug, Clone, Copy)]
pub struct InstanceDataBuildSystem;
impl InstanceDataBuildSystem {
    /// creates Vec of `InstanceData` from components data.
    pub fn update(components: &Components) -> Vec<InstanceData> {
        (0..MAX_ENTITIES)
            .filter_map(|id| build_instance_data(Entity::new(id), components))
            .collect()
    }
}

/// creates `InstanceData` for each entity.
pub fn build_instance_data(entity: Entity, components: &Components) -> Option<InstanceData> {
    let position = components.positions.get(entity)?;
    let size = components.sizes.get(entity)?;
    let flame = components.flames.get(entity)?;
    let sprite_data = components.sprites.get(entity)?.uv_data(*flame);

    Some(InstanceData {
        position: [position.x, position.y],
        size: [size.w, size.h],
        sprite_offset: [sprite_data.uv_offset.u, sprite_data.uv_offset.v],
        sprite_size: [sprite_data.uv_size.w, sprite_data.uv_size.h],
    })
}

pub struct AnimationSystem;
impl AnimationSystem {
    pub fn update(components: &mut Components, dt: f32) -> anyhow::Result<()> {
        for id in 0..MAX_ENTITIES {
            let entity = Entity::new(id);
            let sprite = components
                .sprites
                .get(entity)
                .context("failed to get sprite")?;

            let animation = sprite.animation();

            // -----------------------------------------------------------

            let next_flame = calc_next_flame(entity, components, &animation)?;

            let elapsed = components
                .elapsed
                .get(entity)
                .context("failed to get mut elapsed")?;

            let elapsed_plus_dt = *elapsed + dt;

            let next_elapsed: f32;

            if elapsed_plus_dt >= animation.flame_duration {
                *components
                    .flames
                    .get_mut(entity)
                    .context("failed to get mut flame")? = next_flame;
                next_elapsed = 0.0;
            } else {
                next_elapsed = elapsed_plus_dt;
            }

            *components
                .elapsed
                .get_mut(entity)
                .context("failed to get mut elapsed")? = next_elapsed;
        }
        Ok(())
    }
}

fn calc_next_flame(
    entity: Entity,
    components: &Components,
    animation: &Animation,
) -> anyhow::Result<u8> {
    let flame = components
        .flames
        .get(entity)
        .context("failed to get mut flame")?;

    let flame_plus_one = *flame + 1;

    let next_flame: u8 = if flame_plus_one > animation.max_flame_idx {
        0
    } else {
        flame_plus_one
    };

    Ok(next_flame)
}

// ------------------------------------------------------------------- toggle sprite system

// pub struct ToggleSpriteSystem;
//
// impl ToggleSpriteSystem {
//     /// toggle sprite of clicked position.
//     pub fn update(components: &mut Components, click_pos: Pos) -> anyhow::Result<()> {
//         let gx = (click_pos.x / f32::from(RECT_WIDTH)).floor() as u8;
//         let gy = (click_pos.y / f32::from(RECT_HEIGHT)).floor() as u8;
//
//         let idx = usize::from(gy * MAX_COL + gx);
//
//         let sprite = components
//             .sprites
//             .get(Entity::new(idx))
//             .context("failed to get sprite")?;
//
//         let new_sprite = match sprite {
//             Sprite::RedFlower => Sprite::YellowFlower,
//             Sprite::YellowFlower => Sprite::RedFlower,
//         };
//
//         *components
//             .sprites
//             .get_mut(Entity::new(idx))
//             .context("failed to get sprite mut")? = new_sprite;
//
//         Ok(())
//     }
// }
