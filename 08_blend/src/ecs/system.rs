// ---------------------------------------------------------------- imports

use anyhow::Result;

use crate::ecs::entity::{Entity, EntityManager};
use crate::ecs::{Components, Pos, Query, Size};

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH, RECT_HEIGHT, RECT_WIDTH};
use crate::renderer::InstanceData;

use crate::animation::{AnimationId, AnimationRegistry, AnimationState, init_animation_registry};
use crate::app::input::InputState;
use crate::sound::SoundPlayer;
use crate::sprite::SpriteData;
use crate::world::WorldData;

#[derive(Debug, Clone, Copy)]
enum Command {
    Spawn(Entity, Pos),
    Despawn(Entity),
}

// #[derive(Debug, Clone, Copy)]
// enum GameEvent {
//     Spawn,
// }

struct CommandQuere {
    queue: Vec<Command>,
}

impl CommandQuere {
    fn new() -> Self {
        Self { queue: Vec::new() }
    }
    fn apply(
        &mut self,
        entity_manager: &mut EntityManager,
        components: &mut Components,
    ) -> Result<()> {
        for command in self.queue.drain(..) {
            match command {
                Command::Spawn(entity, pos) => {
                    create_entity_with_pos(entity, components, pos, AnimationId::new(1))?;
                }
                Command::Despawn(entity) => {
                    kill_entity(entity_manager, components, entity)?;
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- systems storage

pub struct Systems {
    commands: CommandQuere,
}
impl Default for Systems {
    fn default() -> Self {
        Self {
            commands: CommandQuere::new(),
        }
    }
}

impl Systems {
    pub fn init(&self, world: &mut WorldData) -> Result<()> {
        init_animation_registry(&mut world.anim_registry)?;

        let center = Pos {
            x: f32::from(LOGIC_WIDTH / 2 - RECT_WIDTH / 2),
            y: f32::from(LOGIC_HEIGHT / 2 - RECT_HEIGHT / 2),
        };
        let entity = world
            .entity_manager
            .spawn()
            .ok_or(anyhow::anyhow!("failed to spawn first entity"))?;

        create_entity_with_pos(entity, &mut world.components, center, AnimationId::new(0))?;

        Ok(())
    }

    pub fn update(
        &mut self,
        world: &mut WorldData,
        input: &InputState,
        sound: &SoundPlayer,
        dt: f32,
    ) -> Result<()> {
        AnimationSystem::update(world, dt)?;

        if let Some(command) = CreateEntitySystem::update(&mut world.entity_manager, input) {
            self.commands.queue.push(command);
            SoundSystem::play_spawn(sound)?;
        }

        let kill_query = world.query::<(Entity, &Pos, &Size)>();
        if let Some(command) = KillEntitySystem::update(kill_query, input) {
            self.commands.queue.push(command);
            SoundSystem::play_despawn(sound)?;
        }

        self.commands
            .apply(&mut world.entity_manager, &mut world.components)?;
        Ok(())
    }

    pub fn update_instances(world: &mut WorldData) {
        InstanceDataBuildSystem::update(world);
    }
}

// ---------------------------------------------------------------- sound system

struct SoundSystem;
impl SoundSystem {
    fn play_spawn(sound: &SoundPlayer) -> Result<()> {
        sound.play_spawn()
    }

    fn play_despawn(sound: &SoundPlayer) -> Result<()> {
        sound.play_despawn()
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
            let Some(state) = world.components.animations.get_mut(*entity) else {
                continue;
            };

            state.elapsed += dt;

            let def = world.anim_registry.get_def(AnimationId::new(state.id));

            if state.elapsed >= def.duration_per_frame {
                state.elapsed -= def.duration_per_frame;
                state.current_frame = (state.current_frame + 1) % u8::try_from(def.frame_count)?;
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------- create entity system

pub struct CreateEntitySystem;

impl CreateEntitySystem {
    fn update(entity_manager: &mut EntityManager, input: &InputState) -> Option<Command> {
        let Some(pos) = input.left_click else {
            return None;
        };
        let Some(entity) = entity_manager.spawn() else {
            return None;
        };
        Some(Command::Spawn(entity, pos))
    }
}
fn create_entity_with_pos(
    entity: Entity,
    components: &mut Components,
    pos: Pos,
    anim_id: AnimationId,
) -> Result<()> {
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
            id: u8::try_from(anim_id.index)?,
        },
    )?;

    Ok(())
}

// ---------------------------------------------------------------- kill entity system

pub struct KillEntitySystem;

impl KillEntitySystem {
    fn update(query: Query<'_, (Entity, &Pos, &Size)>, input: &InputState) -> Option<Command> {
        let Some(click_pos) = input.right_click else {
            return None;
        };

        query
            .iter()
            .find_map(|i| kill_entity_with_pos(i.0, i.1, i.2, click_pos))
    }
}

fn kill_entity_with_pos(entity: Entity, pos: &Pos, size: &Size, click_pos: Pos) -> Option<Command> {
    if click_pos.x >= pos.x
        && pos.x + size.w >= click_pos.x
        && click_pos.y >= pos.y
        && pos.y + size.h >= click_pos.y
    {
        return Some(Command::Despawn(entity));
    }
    None
}
fn kill_entity(
    entity_manager: &mut EntityManager,
    components: &mut Components,
    entity: Entity,
) -> Result<()> {
    entity_manager.despawn(entity)?;
    components.positions.remove(entity)?;
    components.sizes.remove(entity)?;
    components.animations.remove(entity)?;
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
    let g_pos = registry
        .get_frame(AnimationId::new(anim_state.id), anim_state.current_frame)
        .unwrap_or_default();
    let sprite_data = SpriteData::new(g_pos);

    Some(InstanceData {
        position: [position.x, position.y],
        size: [size.w, size.h],
        sprite_offset: [sprite_data.uv_offset.u, sprite_data.uv_offset.v],
        sprite_size: [sprite_data.uv_size.w, sprite_data.uv_size.h],
    })
}
