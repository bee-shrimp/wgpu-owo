// ---------------------------------------------------------------- imports

use anyhow::{Context, Result};

use crate::ecs::component::Vel;
use crate::ecs::entity::{Entity, EntityManager};
use crate::ecs::{ComponentStorage, Components, Pos, Size};

use crate::config::{LOGIC_HEIGHT, LOGIC_WIDTH, MAX_ENTITIES, RECT_HEIGHT, RECT_WIDTH};
use crate::renderer::InstanceData;

use crate::animation::{AnimationId, AnimationRegistry, AnimationState, init_animation_registry};
use crate::app::input::InputState;
use crate::sound::SoundPlayer;
use crate::sprite::SpriteData;
use crate::world::WorldData;

#[derive(Debug, Clone, Copy)]
enum Command {
    Movement(Entity, Pos),
    Collision(Entity, Axis),
    Spawn(Entity, Pos),
    Despawn(Entity),
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
}

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
        sound: &SoundPlayer,
    ) -> Result<()> {
        for command in self.queue.drain(..) {
            match command {
                Command::Movement(entity, new_pos) => {
                    *components
                        .positions
                        .get_mut(entity)
                        .context("get mut fail")? = new_pos
                }
                Command::Spawn(entity, pos) => {
                    // create_entity_with_pos(entity, components, pos, AnimationId::new(1))?;
                    SoundSystem::play_spawn(sound)?;
                }
                Command::Despawn(entity) => {
                    kill_entity(entity_manager, components, entity)?;
                    SoundSystem::play_despawn(sound)?;
                }
                _ => {} //TODO: remove
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
    pub fn init(&mut self, world: &mut WorldData) -> Result<()> {
        init_animation_registry(&mut world.anim_registry)?;
        self.commands.queue.reserve(MAX_ENTITIES);

        let upper = Pos {
            x: f32::from(LOGIC_WIDTH / 2 - RECT_WIDTH / 2),
            y: f32::from(LOGIC_HEIGHT / 2 - RECT_HEIGHT * 2),
        };
        let entity1 = world
            .entity_manager
            .spawn()
            .ok_or_else(|| anyhow::anyhow!("failed to spawn first entity"))?;
        let up = Vel { vx: 0.0, vy: -0.5 };

        create_entity_with_pos(
            entity1,
            &mut world.components,
            upper,
            up,
            AnimationId::new(0),
        )?;

        let lower = Pos {
            x: f32::from(LOGIC_WIDTH / 2 - RECT_WIDTH / 2),
            y: f32::from(LOGIC_HEIGHT / 2 + RECT_HEIGHT),
        };
        let entity2 = world
            .entity_manager
            .spawn()
            .ok_or_else(|| anyhow::anyhow!("failed to spawn first entity"))?;
        let down = Vel { vx: 0.0, vy: 0.5 };

        create_entity_with_pos(
            entity2,
            &mut world.components,
            lower,
            down,
            AnimationId::new(1),
        )?;

        Ok(())
    }

    pub fn update(
        &mut self,
        world: &mut WorldData,
        input: &InputState,
        sound: &SoundPlayer,
        dt: f32,
    ) -> Result<()> {
        // -------------------------------------------------------- animation update

        let animations = &mut world.components.animations;
        let registry = &world.anim_registry;
        AnimationSystem::update(animations, registry, dt)?;

        let positions = &world.components.positions;
        let sizes = &world.components.sizes;
        let velocities = &mut world.components.velocities;
        // MovementSystem::update(positions, velocities, input);
        // CollisionSystem::update(sizes, new_positions));
        // -------------------------------------------------------- spawn update

        // let entity_manager = &mut world.entity_manager;
        // if let Some(command) = CreateEntitySystem::update(entity_manager, input) {
        //     self.commands.queue.push(command);
        // }

        // -------------------------------------------------------- kill update

        // let positions = &world.components.positions;
        // let sizes = &world.components.sizes;
        // let alive = &world.components.positions.alive;
        //
        // if let Some(command) = KillEntitySystem::update(positions, sizes, alive, input) {
        //     self.commands.queue.push(command);
        // }

        // -------------------------------------------------------- apply commands
        self.commands
            .apply(&mut world.entity_manager, &mut world.components, sound)?;

        Ok(())
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

// ---------------------------------------------------------------- movement system

pub struct MovementSystem;
impl MovementSystem {
    fn update(
        positions: &ComponentStorage<Pos>,
        velocities: &ComponentStorage<Vel>,
        _input: &InputState,
    ) -> impl Iterator<Item = (Entity, Pos)> {
        positions
            .iter()
            .zip(velocities.iter())
            .map(|((entity, pos), (_, vel))| (entity, calc_movement(*pos, *vel)))
    }
}

fn calc_movement(pos: Pos, vel: Vel) -> Pos {
    let x = pos.x + vel.vx;
    let y = pos.y + vel.vy;
    let new_pos = Pos { x, y };

    new_pos
}

// ---------------------------------------------------------------- collision system

pub struct CollisionSystem;
impl CollisionSystem {
    // fn update(sizes: &ComponentStorage<Size>, new_positions: impl Iterator<Item = (Entity,Pos)>) -> Option<Command> {
    //     new_positions.map(|(entity, pos)| (entity,pos,sizes.get(entity).ok_or("e")?)).for_each(|(entity,pos,size)| is_window_edge(pos, *size).then(find_wall_collision_axis(pos,size)));
    //
    //     if is_window_edge(new_pos, *size ) {
    //         let axis = find_wall_collision_axis(new_pos, size);
    //         return Some(Command::Collision(entity, axis));
    //     };
    //     if is_entity_collision() {
    //         let axis = find_entity_collision_axis();
    //     }
    //     todo!()
    // }
}

//todo: entity collision.

fn is_window_edge(new_pos: Pos, size: Size) -> bool {
    new_pos.x + size.w >= f32::from(LOGIC_WIDTH)
        || new_pos.x <= 0.0
        || new_pos.y + size.h >= f32::from(LOGIC_HEIGHT)
        || new_pos.y <= 0.0
}

fn find_wall_collision_axis(pos: Pos, size: Size) -> Axis {
    let x = ((pos.x - 0.0).abs()).max((pos.x + size.w - f32::from(LOGIC_WIDTH)).abs());
    let y = ((pos.y - 0.0).abs()).max((pos.y + size.h - f32::from(LOGIC_HEIGHT)).abs());
    let bigger = x.max(y);

    if bigger == x { Axis::X } else { Axis::Y }
}

// ---------------------------------------------------------------- animation system

pub struct AnimationSystem;
impl AnimationSystem {
    fn update(
        animations: &mut ComponentStorage<AnimationState>,
        registry: &AnimationRegistry,
        dt: f32,
    ) -> Result<()> {
        animations
            .iter_mut()
            .try_for_each(|(_, state)| -> Result<()> {
                state.elapsed += dt;

                let def = registry.get_def(AnimationId::new(state.id));

                if state.elapsed >= def.duration_per_frame {
                    state.elapsed -= def.duration_per_frame;
                    state.current_frame =
                        (state.current_frame + 1) % u8::try_from(def.frame_count)?;
                }
                Ok(())
            })?;

        Ok(())
    }
}

// ---------------------------------------------------------------- create entity system

pub struct CreateEntitySystem;

impl CreateEntitySystem {
    fn update(entity_manager: &mut EntityManager, input: &InputState) -> Option<Command> {
        let pos = input.left_click?;

        let entity = entity_manager.spawn()?;

        Some(Command::Spawn(entity, pos))
    }
}

fn create_entity_with_pos(
    entity: Entity,
    components: &mut Components,
    pos: Pos,
    vel: Vel,
    anim_id: AnimationId,
) -> Result<()> {
    components.positions.insert(entity, pos)?;

    components.velocities.insert(entity, vel)?;

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
    fn update(
        positions: &ComponentStorage<Pos>,
        sizes: &ComponentStorage<Size>,
        alive: &[bool; MAX_ENTITIES],
        input: &InputState,
    ) -> Option<Command> {
        let click_pos = input.right_click?;

        for idx in 0..MAX_ENTITIES {
            if !alive.get(idx).is_some_and(|state| *state) {
                continue;
            }

            let entity = Entity::new(idx);

            let pos = positions.get(entity)?;
            let size = sizes.get(entity)?;

            if click_pos.x >= pos.x
                && pos.x + size.w >= click_pos.x
                && click_pos.y >= pos.y
                && pos.y + size.h >= click_pos.y
            {
                return Some(Command::Despawn(entity));
            }
        }

        None
    }
}

fn kill_entity(
    entity_manager: &mut EntityManager,
    components: &mut Components,
    entity: Entity,
) -> Result<()> {
    entity_manager.despawn(entity)?;
    components.positions.remove(entity)?;
    // components.velocities.remove(entity)?;
    components.sizes.remove(entity)?;
    components.animations.remove(entity)?;
    Ok(())
}

// ---------------------------------------------------------------- instance update system

pub struct InstanceDataBuilder;
impl InstanceDataBuilder {
    /// build `InstanceData` from components data.
    pub fn update(world: &mut WorldData) {
        world.instances.clear();

        let alive = &world.components.positions.alive;
        world.instances.extend(instance_data_iter(
            &world.components,
            &world.anim_registry,
            alive,
        ));
    }
}

/// returns iterator of `InstanceData` for entities in `update_targets`.
fn instance_data_iter(
    components: &Components,
    registry: &AnimationRegistry,
    alive: &[bool; MAX_ENTITIES],
) -> impl Iterator<Item = InstanceData> {
    alive
        .iter()
        .enumerate()
        .filter(|(_, state)| **state)
        .map(|(idx, _)| Entity::new(idx))
        .filter_map(|entity| build_instance_data(entity, components, registry))
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
