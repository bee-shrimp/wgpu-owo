// ---------------------------------------------------------------- imports

use anyhow::Context;

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

pub struct Systems {
    sound: SoundSystem,
    animator: AnimationSystem,
    entity_creator: CreateEntitySystem,
    entity_killer: KillEntitySystem,
    instance_data_builder: InstanceDataBuildSystem,
}

impl Systems {
    pub fn new() -> Self {
        Self {
            sound: SoundSystem,
            animator: AnimationSystem,
            entity_creator: CreateEntitySystem::default(),
            entity_killer: KillEntitySystem::default(),
            instance_data_builder: InstanceDataBuildSystem,
        }
    }
    pub fn init(&mut self, world: &mut WorldData) -> anyhow::Result<()> {
        init_animation_registry(&mut world.anim_registry)?;

        let center = Pos {
            x: f32::from(LOGIC_WIDTH / 2 - RECT_WIDTH / 2),
            y: f32::from(LOGIC_HEIGHT / 2 - RECT_HEIGHT / 2),
        };

        self.entity_creator
            .create_entity_with_pos(
                &mut world.entity_manager,
                &mut world.components,
                Some(center),
                AnimationId::new(0),
            )
            .context("failed to create entity")?;
        Ok(())
    }
    pub fn update(
        &mut self,
        world: &mut WorldData,
        input: &InputState,
        sound: &SoundPlayer,
        dt: f32,
    ) -> anyhow::Result<()> {
        world.update_targets.truncate(0);
        world
            .update_targets
            .extend(world.components.with_animation_state());

        self.animator.update(
            &mut world.components,
            &world.anim_registry,
            &world.update_targets,
            dt,
        )?;

        self.entity_creator
            .create_entity_with_pos(
                &mut world.entity_manager,
                &mut world.components,
                input.left_click,
                AnimationId::new(0),
            )
            .context("failed to create entity")?;

        world.update_targets.truncate(0);
        world
            .update_targets
            .extend(world.components.with_pos_and_size());

        self.entity_killer.kill_entity_with_pos(
            &mut world.entity_manager,
            &mut world.components,
            &world.update_targets,
            input.right_click,
        )?;

        if self.entity_killer.has_triggered() || self.entity_creator.has_triggered() {
            self.sound.play(input, sound);
        }

        Ok(())
    }
    pub fn update_instances(&self, world: &mut WorldData) {
        world.instances.truncate(0);

        world.update_targets.truncate(0);
        world.update_targets.extend(world.components.iter_alive());

        world.instances.extend(self.instance_data_builder.update(
            &world.components,
            &world.anim_registry,
            &world.update_targets,
        ));
    }
}
struct SoundSystem;
impl SoundSystem {
    fn play(&self, input: &InputState, sound: &SoundPlayer) {
        if input.left_click.is_some() {
            sound.play_spawn().expect("failed to play sound");
        }

        if input.right_click.is_some() {
            sound.play_despawn().expect("failed to play sound");
        }
    }
}

// ---------------------------------------------------------------- create entity system

#[derive(Debug, Clone, Copy, Default)]
pub struct CreateEntitySystem {
    triggered: bool,
}

impl CreateEntitySystem {
    pub fn create_entity_with_pos(
        &mut self,
        entity_manager: &mut EntityManager,
        components: &mut Components,
        pos: Option<Pos>,
        anim_id: AnimationId,
    ) -> anyhow::Result<()> {
        let Some(pos) = pos else {
            self.triggered = false;
            return Ok(());
        };

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

        self.triggered = true;

        Ok(())
    }

    fn has_triggered(&self) -> bool {
        self.triggered
    }

    // /// adds data to `ComponentStorage`[[entity.index]].
    // /// uses `MAX_ENTITIES` to determine the number of entities.
    // pub fn create_grid_entities(
    //     entity_manager: &mut EntityManager,
    //     components: &mut Components,
    //     // sprite: Sprite,
    //     anim_id: AnimationId,
    // ) -> anyhow::Result<()> {
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
}

#[derive(Debug, Clone, Copy, Default)]
pub struct KillEntitySystem {
    triggered: bool,
}

impl KillEntitySystem {
    pub fn kill_entity_with_pos(
        &mut self,
        entity_manager: &mut EntityManager,
        components: &mut Components,
        entities_with_pos_and_size: &[Entity],
        click_pos: Option<Pos>,
    ) -> anyhow::Result<()> {
        let Some(click_pos) = click_pos else {
            self.triggered = false;
            return Ok(());
        };

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
                self.triggered = true;
            }
        }

        Ok(())
    }
    fn has_triggered(&self) -> bool {
        self.triggered
    }
}

// ---------------------------------------------------------------- instance update system

#[derive(Debug, Clone, Copy)]
pub struct InstanceDataBuildSystem;
impl InstanceDataBuildSystem {
    /// returns iterator of `InstanceData` built from components data.
    pub fn update(
        &self,
        components: &Components,
        registry: &AnimationRegistry,
        alive_entities: &[Entity],
    ) -> impl Iterator<Item = InstanceData> {
        alive_entities
            .iter()
            .filter_map(|entity| build_instance_data(*entity, components, registry))
    }
}

/// creates `InstanceData` for each entity.
pub fn build_instance_data(
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

// ---------------------------------------------------------------- animation system

pub struct AnimationSystem;
impl AnimationSystem {
    pub fn update(
        &self,
        components: &mut Components,
        registry: &AnimationRegistry,
        entities_with_animation_state: &[Entity],
        dt: f32,
    ) -> anyhow::Result<()> {
        for entity in entities_with_animation_state {
            let mut state = *components
                .animations
                .get(*entity)
                .context("animation system: failed to get animation state")?;

            state.elapsed += dt;

            let def = registry.get_def(AnimationId::new(state.id));

            if state.elapsed >= def.duration_per_frame {
                state.elapsed -= def.duration_per_frame;
                state.current_frame = (state.current_frame + 1) % def.frame_count as u8;
            }

            *components
                .animations
                .get_mut(*entity)
                .context("animation system: failed to get mut animation")? = state;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- toggle sprite system

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
