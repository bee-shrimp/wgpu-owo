// ---------------------------------------------------------------- imports

use color_eyre::eyre::{Ok, OptionExt, Result};

use crate::ecs::component::Vel;
use crate::ecs::entity::{Entity, EntityManager};
use crate::ecs::system::Collision::Wall;
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
    Bounce(Entity, Collision),
    Spawn(Entity, Pos),
    Despawn(Entity),
}

// #[derive(Debug, Clone, Copy)]
// enum Event {
//     Collision(Entity, Axis),
// }

#[derive(Debug, Clone, Copy)]
enum Collision {
    Wall(Entity, Axis),
    Entities(Entity, Entity),
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
                    move_entity(&mut components.positions, entity, new_pos)?;
                }

                Command::Spawn(entity, pos) => {
                    create_entity(
                        entity,
                        components,
                        pos,
                        Vel { vx: 0.0, vy: 1.0 },
                        AnimationId::new(1),
                    )?;
                    SoundSystem::play_spawn(sound)?;
                }
                Command::Despawn(entity) => {
                    kill_entity(entity_manager, components, entity)?;
                    SoundSystem::play_despawn(sound)?;
                }
                Command::Bounce(entity, collision) => {
                    bounce_entity(
                        &mut components.positions,
                        &mut components.velocities,
                        entity,
                        collision,
                    )?;
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- systems storage

pub struct Systems {
    commands: CommandQuere,
    move_plans: Vec<(Entity, Pos)>,
    detected_collisions: Vec<Collision>,
}
impl Default for Systems {
    fn default() -> Self {
        Self {
            commands: CommandQuere::new(),
            move_plans: Vec::with_capacity(MAX_ENTITIES),
            detected_collisions: Vec::with_capacity(MAX_ENTITIES),
        }
    }
}

impl Systems {
    pub fn init(&mut self, world: &mut WorldData) -> Result<()> {
        init_animation_registry(&mut world.anim_registry)?;
        self.commands.queue.reserve(MAX_ENTITIES);

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

        AnimationSystem::update(&mut world.components.animations, &world.anim_registry, dt)?;

        // -------------------------------------------------------- movement and collision update

        self.move_plans.clear();
        self.move_plans.extend(MovementPlanningSystem::update(
            &world.components.positions,
            &world.components.velocities,
            dt,
        ));

        self.detected_collisions.clear();
        self.detected_collisions.extend(WallCollisionSystem::update(
            &self.move_plans,
            &world.components.sizes,
        ));

        self.detected_collisions
            .extend(EntitiesCollisionSystem::update(
                &self.move_plans,
                &world.components.sizes,
            ));

        self.commands.queue.extend(MovementCommandSystem::update(
            &self.move_plans,
            &self.detected_collisions,
        ));

        // -------------------------------------------------------- spawn update

        let entity_manager = &mut world.entity_manager;
        if let Some(command) = CreateEntitySystem::update(entity_manager, input) {
            self.commands.queue.push(command);
        }

        // -------------------------------------------------------- kill update

        if let Some(command) =
            KillEntitySystem::update(&world.components.positions, &world.components.sizes, input)
        {
            self.commands.queue.push(command);
        }

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

struct MovementPlanningSystem;
impl MovementPlanningSystem {
    fn update(
        positions: &ComponentStorage<Pos>,
        velocities: &ComponentStorage<Vel>,
        dt: f32,
    ) -> impl Iterator<Item = (Entity, Pos)> {
        positions
            .iter()
            .zip(velocities.iter())
            .map(move |((entity, pos), (_, vel))| (entity, calc_movement(*pos, *vel, dt)))
    }
}

fn calc_movement(pos: Pos, vel: Vel, dt: f32) -> Pos {
    let x = pos.x + vel.vx * dt;
    let y = pos.y + vel.vy * dt;
    Pos { x, y }
}

fn move_entity(positions: &mut ComponentStorage<Pos>, entity: Entity, new_pos: Pos) -> Result<()> {
    *positions.get_mut(entity).ok_or_eyre("component missing")? = new_pos;
    Ok(())
}

// ---------------------------------------------------------------- movement command system

struct MovementCommandSystem;
impl MovementCommandSystem {
    fn update(
        move_plans: &[(Entity, Pos)],
        collision_plans: &[Collision],
    ) -> impl Iterator<Item = Command> {
        // Note: too deep nest?
        move_plans.iter().map(|(entity, pos)| {
            match collision_plans
                .iter()
                .copied()
                .map(|collision| match collision {
                    Collision::Wall(collision_entity, _) => (collision_entity, collision),
                    Collision::Entities(collision_entity, _) => (collision_entity, collision),
                })
                .find(|(collision_entity, _)| collision_entity == entity)
            {
                Some((collision_entity, collision)) => Command::Bounce(collision_entity, collision),
                None => Command::Movement(*entity, *pos),
            }
        })
    }
}
// ---------------------------------------------------------------- collision system

struct WallCollisionSystem;
impl WallCollisionSystem {
    fn update(
        new_positions: &[(Entity, Pos)],
        sizes: &ComponentStorage<Size>,
    ) -> impl Iterator<Item = Collision> {
        new_positions
            .iter()
            .map(|(entity, pos)| -> Option<Collision> {
                let size = *sizes.get(*entity)?;

                if is_window_edge(*pos, size) {
                    let axis = find_window_edge_collision_axis(*pos, size);
                    Some(Collision::Wall(*entity, axis))
                } else {
                    None
                }
            })
            .flatten()
    }
}

fn is_window_edge(new_pos: Pos, size: Size) -> bool {
    new_pos.x + size.w >= f32::from(LOGIC_WIDTH)
        || new_pos.x <= 0.0
        || new_pos.y + size.h >= f32::from(LOGIC_HEIGHT)
        || new_pos.y <= 0.0
}

fn find_window_edge_collision_axis(pos: Pos, size: Size) -> Axis {
    let x = ((pos.x - 0.0).abs()).max((pos.x + size.w - f32::from(LOGIC_WIDTH)).abs());
    let y = ((pos.y - 0.0).abs()).max((pos.y + size.h - f32::from(LOGIC_HEIGHT)).abs());
    let bigger = x.max(y);
    let margin = 0.001;

    if (bigger - x).abs() < margin {
        Axis::X
    } else {
        Axis::Y
    }
}

struct EntitiesCollisionSystem;

impl EntitiesCollisionSystem {
    fn update(
        new_positions: &[(Entity, Pos)],
        sizes: &ComponentStorage<Size>,
    ) -> impl Iterator<Item = Collision> {

        // Todo: entity collision.
    }
}

fn bounce_entity(
    positions: &mut ComponentStorage<Pos>,
    velocities: &mut ComponentStorage<Vel>,
    entity: Entity,
    collision: Collision,
) -> Result<()> {
    let Some(vel) = velocities.get_mut(entity) else {
        return Ok(());
    };
    let Some(pos) = positions.get_mut(entity) else {
        return Ok(());
    };

    let axis = match collision {
        Wall(_, axis) => axis,
        _ => Axis::X,
    };

    println!("{:?}", axis);
    println!("{:?}{:?}", vel, pos);

    match axis {
        Axis::X => {
            vel.vx *= -1.0;
            pos.x += vel.vx
        }
        Axis::Y => {
            println!("y axis collision");
            vel.vy *= -1.0;
            pos.y += vel.vy
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- animation system

struct AnimationSystem;
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

struct CreateEntitySystem;

impl CreateEntitySystem {
    fn update(entity_manager: &mut EntityManager, input: &InputState) -> Option<Command> {
        let pos = input.left_click?;

        let entity = entity_manager.spawn()?;

        Some(Command::Spawn(entity, pos))
    }
}

fn create_entity(
    entity: Entity,
    components: &mut Components,
    pos: Pos,
    vel: Vel,
    anim_id: AnimationId,
) -> Result<()> {
    let size = Size {
        w: f32::from(RECT_WIDTH),
        h: f32::from(RECT_HEIGHT),
    };

    if is_window_edge(pos, size) {
        return Ok(());
    }

    components.positions.insert(entity, pos)?;

    components.velocities.insert(entity, vel)?;

    components.sizes.insert(entity, size)?;

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

struct KillEntitySystem;

impl KillEntitySystem {
    fn update(
        positions: &ComponentStorage<Pos>,
        sizes: &ComponentStorage<Size>,
        input: &InputState,
    ) -> Option<Command> {
        let click_pos = input.right_click?;

        positions
            .iter()
            .zip(sizes.iter())
            .find(|((_, pos), (_, size))| is_clicked(**pos, **size, click_pos))
            .map(|((entity, _), (_, _))| Command::Despawn(entity))
    }
}
fn is_clicked(pos: Pos, size: Size, click_pos: Pos) -> bool {
    click_pos.x >= pos.x
        && pos.x + size.w >= click_pos.x
        && click_pos.y >= pos.y
        && pos.y + size.h >= click_pos.y
}

fn kill_entity(
    entity_manager: &mut EntityManager,
    components: &mut Components,
    entity: Entity,
) -> Result<()> {
    entity_manager.despawn(entity)?;
    components.positions.remove(entity)?;
    components.velocities.remove(entity)?;
    components.sizes.remove(entity)?;
    components.animations.remove(entity)?;
    Ok(())
}

// ---------------------------------------------------------------- instance update system

pub struct InstanceDataBuilder;
impl InstanceDataBuilder {
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
