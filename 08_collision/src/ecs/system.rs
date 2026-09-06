// ---------------------------------------------------------------- imports

use color_eyre::eyre::{OptionExt, Result};

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

// ---------------------------------------------------------------- consts

const VX: f32 = 5.0;
const VY: f32 = 0.0;

// ---------------------------------------------------------------- command enum

#[derive(Debug, Clone, Copy)]
enum Command {
    Movement(Entity, Pos),
    Bounce(Entity, Collision),
    Spawn(Entity, Pos),
    Despawn(Entity),
}

// ---------------------------------------------------------------- command queue

struct CommandQueue {
    queue: Vec<Command>,
}

impl CommandQueue {
    fn new() -> Self {
        Self { queue: Vec::new() }
    }

    fn apply(
        &mut self,
        entity_manager: &mut EntityManager,
        components: &mut Components,
        sound: &SoundPlayer,
        dt: f32,
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
                        Vel { vx: VX, vy: VY },
                        AnimationId::new(1),
                    )
                    .and_then(|()| SoundSystem::play_spawn(sound))?;
                }

                Command::Despawn(entity) => {
                    kill_entity(entity_manager, components, entity)
                        .and_then(|()| SoundSystem::play_despawn(sound))?;
                }

                Command::Bounce(entity, collision) => {
                    bounce_entity(
                        &mut components.positions,
                        &mut components.velocities,
                        entity,
                        collision,
                        dt,
                    )?;
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- systems storage

pub struct Systems {
    commands: CommandQueue,
    move_plans: Vec<(Entity, Pos)>,
    detected_collisions: Vec<Collision>,
}
impl Default for Systems {
    fn default() -> Self {
        Self {
            commands: CommandQueue::new(),
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
        self.detected_collisions
            .extend(WindowEdgeCollisionSystem::update(
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

        if let Some(command) = CreateEntitySystem::update(&mut world.entity_manager, input) {
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
            .apply(&mut world.entity_manager, &mut world.components, sound, dt)?;

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
        detected_collisions: &[Collision],
    ) -> impl Iterator<Item = Command> {
        move_plans.iter().map(|(entity, pos)| {
            match detected_collisions
                .iter()
                .copied()
                .map(|collision| match collision {
                    Collision::WindowEdge(collision_entity, _) => (collision_entity, collision),
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

#[derive(Debug, Clone, Copy)]
enum Collision {
    WindowEdge(Entity, Axis),
    Entities(Entity, Axis),
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
}

// ---------------------------------------------------------------- wall collision system

struct WindowEdgeCollisionSystem;
impl WindowEdgeCollisionSystem {
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
                    Some(Collision::WindowEdge(*entity, axis))
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
    // Todo: can be better
    let x_diff = (pos.x.abs()).max((pos.x + size.w - f32::from(LOGIC_WIDTH)).abs());
    let y_diff = (pos.y.abs()).max((pos.y + size.h - f32::from(LOGIC_HEIGHT)).abs());
    let bigger_diff = x_diff.max(y_diff);
    let margin = 0.001;

    if (bigger_diff - x_diff).abs() < margin {
        Axis::X
    } else {
        Axis::Y
    }
}

// ---------------------------------------------------------------- entities collision system

#[derive(Debug, Clone, Copy, PartialEq)]
struct Aabb {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Aabb {
    fn intersects(&self, other: Aabb) -> bool {
        // self edge in other area
        ((is_aligned(self.min_x, self.max_x, other.min_x)
            || is_aligned(self.min_x, self.max_x, other.max_x))
            && (is_aligned(self.min_y, self.max_y, other.min_y)
                || is_aligned(self.min_y, self.max_y, other.max_y)))

            // other edge in self area
            || (is_aligned(other.min_x, other.max_x, self.min_x)
                || is_aligned(other.min_x, other.max_x, self.max_x))
                && (is_aligned(other.min_y, other.max_y, self.min_y)
                    || is_aligned(other.min_y, other.max_y, self.max_y))
    }
}

fn is_aligned(self_min: f32, self_max: f32, other: f32) -> bool {
    self_min <= other && other <= self_max
}

struct EntitiesCollisionSystem;
impl EntitiesCollisionSystem {
    fn update(
        new_positions: &[(Entity, Pos)],
        sizes: &ComponentStorage<Size>,
    ) -> impl Iterator<Item = Collision> {
        new_positions
            .iter()
            .enumerate()
            .map(|(i, a)| {
                new_positions
                    .iter()
                    .skip(i + 1)
                    .map(|b| find_aabb_collision(*a, *b, sizes))
                    .flatten()
                    .flat_map(|(collision_a, collision_b)| [collision_a, collision_b])
            })
            .flatten()
    }
}

fn find_aabb_collision(
    a: (Entity, Pos),
    b: (Entity, Pos),
    sizes: &ComponentStorage<Size>,
) -> Option<(Collision, Collision)> {
    let entity_a = a.0;
    let entity_b = b.0;

    let pos_a = a.1;
    let pos_b = b.1;

    let size_a = sizes.get(entity_a)?;
    let size_b = sizes.get(entity_b)?;

    let aabb_a = Aabb {
        min_x: pos_a.x,
        max_x: pos_a.x + size_a.w,
        min_y: pos_a.y,
        max_y: pos_a.y + size_a.h,
    };
    let aabb_b = Aabb {
        min_x: pos_b.x,
        max_x: pos_b.x + size_b.w,
        min_y: pos_b.y,
        max_y: pos_b.y + size_b.h,
    };

    if aabb_a.intersects(aabb_b) {
        let axis = find_entities_collision_axis(aabb_a, aabb_b);
        Some((
            Collision::Entities(entity_a, axis),
            Collision::Entities(entity_b, axis),
        ))
    } else {
        None
    }
}
fn find_entities_collision_axis(aabb_a: Aabb, aabb_b: Aabb) -> Axis {
    let x_diff = ((aabb_a.min_x - aabb_b.min_x).abs()).max((aabb_a.max_x - aabb_b.max_x).abs());
    let y_diff = ((aabb_a.min_y - aabb_b.min_y).abs()).max((aabb_a.max_y - aabb_b.max_y).abs());
    let bigger_diff = x_diff.max(y_diff);
    let margin = 0.0001;

    if (bigger_diff - x_diff).abs() < margin {
        Axis::X
    } else {
        Axis::Y
    }
}

fn bounce_entity(
    positions: &mut ComponentStorage<Pos>,
    velocities: &mut ComponentStorage<Vel>,
    entity: Entity,
    collision: Collision,
    dt: f32,
) -> Result<()> {
    let Some(pos) = positions.get_mut(entity) else {
        return Ok(());
    };

    let Some(vel) = velocities.get_mut(entity) else {
        return Ok(());
    };

    let axis = match collision {
        Collision::WindowEdge(_, axis) => axis,
        Collision::Entities(_, axis) => axis,
    };

    match axis {
        Axis::X => {
            vel.vx *= -1.0;
            pos.x += vel.vx * dt
        }
        Axis::Y => {
            vel.vy *= -1.0;
            pos.y += vel.vy * dt
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

        let size = Size {
            w: f32::from(RECT_WIDTH),
            h: f32::from(RECT_HEIGHT),
        };

        if is_window_edge(pos, size) {
            return None;
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_definition() {
        let pos = Pos { x: 10.0, y: 10.0 };
        let size = Size { w: 10.0, h: 10.0 };

        let aabb = Aabb {
            min_x: pos.x,
            max_x: pos.x + size.w,
            min_y: pos.y,
            max_y: pos.y + size.h,
        };

        assert_eq!(
            aabb,
            Aabb {
                min_x: 10.0,
                max_x: 20.0,
                min_y: 10.0,
                max_y: 20.0
            }
        );
    }

    #[test]
    fn aabb_collision_mid() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let aabb_b = Aabb {
            min_x: 15.0,
            max_x: 25.0,
            min_y: 15.0,
            max_y: 25.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_x_a_is_left() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let aabb_b = Aabb {
            min_x: 19.0,
            max_x: 29.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_y_a_is_upper() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };
        let aabb_b = Aabb {
            min_x: 15.0,
            max_x: 25.0,
            min_y: 11.0,
            max_y: 21.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_x_b_is_left() {
        let aabb_a = Aabb {
            min_x: 19.0,
            max_x: 29.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let aabb_b = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_y_b_is_upper() {
        let aabb_a = Aabb {
            min_x: 15.0,
            max_x: 25.0,
            min_y: 11.0,
            max_y: 21.0,
        };
        let aabb_b = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_x_upper_right_of_a() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let aabb_b = Aabb {
            min_x: 19.0,
            max_x: 29.0,
            min_y: 1.0,
            max_y: 11.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_upper_right_of_b() {
        let aabb_a = Aabb {
            min_x: 18.0,
            max_x: 28.0,
            min_y: 1.0,
            max_y: 11.0,
        };
        let aabb_b = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_x_aligned() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };
        let aabb_b = Aabb {
            min_x: 20.0,
            max_x: 30.0,
            min_y: 11.0,
            max_y: 21.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_x_small_a() {
        let aabb_a = Aabb {
            min_x: 19.0,
            max_x: 29.0,
            min_y: 12.0,
            max_y: 18.0,
        };
        let aabb_b = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_x_small_b() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };
        let aabb_b = Aabb {
            min_x: 19.0,
            max_x: 29.0,
            min_y: 12.0,
            max_y: 18.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_y_small_a() {
        let aabb_a = Aabb {
            min_x: 12.0,
            max_x: 18.0,
            min_y: 19.0,
            max_y: 29.0,
        };
        let aabb_b = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_y_small_b() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };
        let aabb_b = Aabb {
            min_x: 12.0,
            max_x: 18.0,
            min_y: 19.0,
            max_y: 29.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }

    #[test]
    fn aabb_collision_perfectly_align() {
        let aabb_a = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };
        let aabb_b = Aabb {
            min_x: 10.0,
            max_x: 20.0,
            min_y: 10.0,
            max_y: 20.0,
        };

        let result = aabb_a.intersects(aabb_b);

        assert_eq!(result, true);
    }
}
