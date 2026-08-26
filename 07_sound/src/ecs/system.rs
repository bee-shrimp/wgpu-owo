// ---------------------------------------------------------------- imports

use anyhow::{Context, Result};

use crate::ecs::entity::{Entity, EntityManager};
use crate::ecs::{Components, Pos, Size};

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH, RECT_HEIGHT, RECT_WIDTH};
use crate::renderer::InstanceData;

use crate::animation::{AnimationId, AnimationRegistry, AnimationState, init_animation_registry};
use crate::app::input::InputState;
use crate::sound::SoundPlayer;
use crate::sprite::SpriteData;
use crate::world::WorldData;

// ---------------------------------------------------------------- systems storage

pub struct Systems {}

impl Systems {
    pub fn init(world: &mut WorldData) -> Result<()> {
        init_animation_registry(&mut world.anim_registry)?;

        let center = Pos {
            x: f32::from(LOGIC_WIDTH / 2 - RECT_WIDTH / 2),
            y: f32::from(LOGIC_HEIGHT / 2 - RECT_HEIGHT / 2),
        };

        create_entity_with_pos(
            &mut world.entity_manager,
            &mut world.components,
            center,
            AnimationId::new(0),
        )
        .context("failed to create entity")?;
        Ok(())
    }

    pub fn update(
        world: &mut WorldData,
        input: &InputState,
        sound: &SoundPlayer,
        dt: f32,
    ) -> Result<()> {
        AnimationSystem::update(world, dt)?;

        let create_triggered = CreateEntitySystem::update(world, input)?;

        let kill_triggered = KillEntitySystem::update(world, input)?;

        if create_triggered == Some(true) || kill_triggered == Some(true) {
            SoundSystem::play(input, sound)?;
        }

        Ok(())
    }

    pub fn update_instances(world: &mut WorldData) {
        InstanceDataBuildSystem::update(world);
    }
}

// ---------------------------------------------------------------- sound system

struct SoundSystem;
impl SoundSystem {
    fn play(input: &InputState, sound: &SoundPlayer) -> Result<()> {
        if input.left_click.is_some() {
            sound.play_spawn()?;
        }

        if input.right_click.is_some() {
            sound.play_despawn()?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- animation system

pub struct AnimationSystem;
impl AnimationSystem {
    fn update(world: &mut WorldData, dt: f32) -> Result<()> {
        world.update_targets.clear();
        world
            .update_targets
            .extend(world.components.with_animation_state());

        for entity in &world.update_targets {
            let mut state = *world
                .components
                .animations
                .get(*entity)
                .context("animation system: failed to get animation state")?;

            state.elapsed += dt;

            let def = world.anim_registry.get_def(AnimationId::new(state.id));

            if state.elapsed >= def.duration_per_frame {
                state.elapsed -= def.duration_per_frame;
                state.current_frame = (state.current_frame + 1) % def.frame_count as u8;
            }

            *world
                .components
                .animations
                .get_mut(*entity)
                .context("animation system: failed to get mut animation")? = state;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- create entity system

pub struct CreateEntitySystem {}

impl CreateEntitySystem {
    fn update(world: &mut WorldData, input: &InputState) -> Result<Option<bool>> {
        let Some(pos) = input.left_click else {
            return Ok(None);
        };

        create_entity_with_pos(
            &mut world.entity_manager,
            &mut world.components,
            pos,
            AnimationId::new(1),
        )
        .context("failed to create entity")?;

        let triggered = true;

        Ok(Some(triggered))
    }
}
fn create_entity_with_pos(
    entity_manager: &mut EntityManager,
    components: &mut Components,
    pos: Pos,
    anim_id: AnimationId,
) -> Result<()> {
    let Some(entity) = entity_manager.spawn() else {
        return Ok(());
    };

    components.positions.insert(entity, pos)?;

    components.sizes.insert(
        entity,
        Size {
            w: f32::from(RECT_WIDTH),
            h: f32::from(RECT_HEIGHT),
        },
    )?;

    components.animations.insert(
        entity,
        AnimationState {
            current_frame: 0,
            elapsed: 0.0,
            id: anim_id.index as u8,
        },
    )?;

    Ok(())
}

// /// adds data to `ComponentStorage`[[entity.index]].
// /// uses `MAX_ENTITIES` to determine the number of entities.
// pub fn create_grid_entities(
//     entity_manager: &mut EntityManager,
//     components: &mut Components,
//     // sprite: Sprite,
//     anim_id: AnimationId,
// ) -> Result<()> {
//     use config::MAX_COL;
//     for _ in 0..MAX_ENTITIES {
//         let entity = entity_manager.spawn().context("no free slot")?;
//
//         components
//             .positions
//             .insert(
//                 entity,
//                 Pos {
//                     x: f32::from(RECT_WIDTH * (entity.index as u8 % MAX_COL)),
//                     y: f32::from(RECT_HEIGHT * (entity.index as u8 / MAX_COL)),
//                 },
//             )
//             .context("failed to add position")?;
//
//         components
//             .sizes
//             .insert(
//                 entity,
//                 Size {
//                     w: f32::from(RECT_WIDTH),
//                     h: f32::from(RECT_HEIGHT),
//                 },
//             )
//             .context("failed to add size")?;
//
//         // components
//         //     .sprites
//         //     .insert(entity, sprite)
//         //     .context("failed to add sprite data")?;
//
//         components
//             .animations
//             .insert(
//                 entity,
//                 AnimationState {
//                     current_frame: 0,
//                     elapsed: 0.0,
//                     id: anim_id.index as u8,
//                 },
//             )
//             .context("failed to add animation")?;
//     }
//     Ok(())
// }

pub struct KillEntitySystem {}

impl KillEntitySystem {
    fn update(world: &mut WorldData, input: &InputState) -> Result<Option<bool>> {
        let Some(pos) = input.right_click else {
            return Ok(None);
        };

        world.update_targets.clear();
        world
            .update_targets
            .extend(world.components.with_pos_and_size());

        kill_entity_with_pos(
            &mut world.entity_manager,
            &mut world.components,
            &world.update_targets,
            pos,
        )?;

        let triggered = true;

        Ok(Some(triggered))
    }
}

fn kill_entity_with_pos(
    entity_manager: &mut EntityManager,
    components: &mut Components,
    entities_with_pos_and_size: &[Entity],
    click_pos: Pos,
) -> Result<()> {
    for entity in entities_with_pos_and_size {
        let entity = *entity;

        let pos = components
            .positions
            .get(entity)
            .context("kill entity system: failed to get pos")?;
        let size = components
            .sizes
            .get(entity)
            .context("kill entity system: failed to get pos")?;

        if click_pos.x >= pos.x
            && pos.x + size.w >= click_pos.x
            && click_pos.y >= pos.y
            && pos.y + size.h >= click_pos.y
        {
            entity_manager.despawn(entity)?;
            components.positions.remove(entity)?;
            components.sizes.remove(entity)?;
            components.animations.remove(entity)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------- instance update system

pub struct InstanceDataBuildSystem;
impl InstanceDataBuildSystem {
    /// build `InstanceData` from components data.
    fn update(world: &mut WorldData) {
        world.instances.clear();

        world.update_targets.clear();
        world.update_targets.extend(world.components.iter_alive());

        world.instances.extend(instance_data_iter(
            &world.components,
            &world.anim_registry,
            &world.update_targets,
        ));
    }
}

/// returns iterator of `InstanceData` for entities in `update_targets`.
fn instance_data_iter(
    components: &Components,
    registry: &AnimationRegistry,
    update_targets: &[Entity],
) -> impl Iterator<Item = InstanceData> {
    update_targets
        .iter()
        .filter_map(|entity| build_instance_data(*entity, components, registry))
}

/// creates `InstanceData` for each entity.
fn build_instance_data(
    entity: Entity,
    components: &Components,
    registry: &AnimationRegistry,
) -> Option<InstanceData> {
    let position = components.positions.get(entity)?;
    let size = components.sizes.get(entity)?;

    let anim_state = components.animations.get(entity)?;
    let g_pos = registry.get_frame(AnimationId::new(anim_state.id), anim_state.current_frame);
    let sprite_data = SpriteData::new(g_pos);

    Some(InstanceData {
        position: [position.x, position.y],
        size: [size.w, size.h],
        sprite_offset: [sprite_data.uv_offset.u, sprite_data.uv_offset.v],
        sprite_size: [sprite_data.uv_size.w, sprite_data.uv_size.h],
    })
}

// ---------------------------------------------------------------- toggle sprite system

// pub struct ToggleSpriteSystem;
//
// impl ToggleSpriteSystem {
//     /// toggle sprite of clicked position.
//     pub fn update(components: &mut Components, click_pos: Pos) -> Result<()> {
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
