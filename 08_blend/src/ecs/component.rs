// ---------------------------------------------------------------- imports

use crate::animation::AnimationState;
use crate::config::MAX_ENTITIES;
use crate::ecs::entity::Entity;
use anyhow::Result;

// ---------------------------------------------------------------- struct for rects

#[derive(Debug, Clone, Copy, Default)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

// ---------------------------------------------------------------- components struct for world

pub struct Components {
    pub positions: ComponentStorage<Pos>,
    pub sizes: ComponentStorage<Size>,
    pub animations: ComponentStorage<AnimationState>,
}
impl Components {
    pub fn iter_alive(&self) -> impl Iterator<Item = Entity> {
        (0..MAX_ENTITIES)
            .filter(move |&i| self.positions.alive.get(i).is_some_and(|state| *state))
            .map(Entity::new)
    }

    pub fn with_pos_and_size(&self) -> impl Iterator<Item = Entity> {
        (0..MAX_ENTITIES)
            .filter(move |&i| self.positions.alive.get(i).is_some_and(|state| *state))
            .filter(move |&i| self.sizes.alive.get(i).is_some_and(|state| *state))
            .map(Entity::new)
    }

    pub fn with_animation_state(&self) -> impl Iterator<Item = Entity> {
        (0..MAX_ENTITIES)
            .filter(move |&i| self.animations.alive.get(i).is_some_and(|state| *state))
            .map(Entity::new)
    }
}

// ---------------------------------------------------------------- component storage

pub struct ComponentStorage<T> {
    components: [T; MAX_ENTITIES],
    alive: [bool; MAX_ENTITIES],
}

impl<T: Default> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            components: std::array::from_fn(|_| T::default()),
            alive: [false; MAX_ENTITIES],
        }
    }

    /// adds T to component storage.
    pub fn insert(&mut self, entity: Entity, component: T) -> Result<()> {
        if entity.index >= MAX_ENTITIES {
            anyhow::bail!("entity index out of bounds");
        }

        if let Some(slot) = self.components.get_mut(entity.index) {
            *slot = component;
        }

        if let Some(slot) = self.alive.get_mut(entity.index) {
            *slot = true;
        }

        Ok(())
    }

    /// returns Some(&T) if entity is alive.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        if entity.index >= MAX_ENTITIES || self.alive.get(entity.index).is_some_and(|state| !*state)
        {
            return None;
        }

        self.components.get(entity.index)
    }

    /// returns Some(&mut T) if entity is alive.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if entity.index >= MAX_ENTITIES || self.alive.get(entity.index).is_some_and(|state| !*state)
        {
            return None;
        }
        self.components.get_mut(entity.index)
    }

    /// removes T from components array and kill entity(set alive == false).
    pub fn remove(&mut self, entity: Entity) -> Result<()> {
        if entity.index >= MAX_ENTITIES || self.alive.get(entity.index).is_some_and(|state| !*state)
        {
            anyhow::bail!("entity not found");
        }

        if let Some(state) = self.alive.get_mut(entity.index) {
            *state = false;
        }

        Ok(())
    }

    // /// returns iterator (Entity, &T).
    // pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> + '_ {
    //     (0..MAX_ENTITIES).filter_map(move |i| {
    //         if self.alive[i] {
    //             Some((Entity::new(i), &self.components[i]))
    //         } else {
    //             None
    //         }
    //     })
    // }
    //
    // /// returns how many entities are alive. mainly for debug.
    // pub fn count_alive(&self) -> usize {
    //     self.alive.iter().filter(|&&a| a).count()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get_component() {
        let mut storage = ComponentStorage::<Pos>::new();
        let entity = Entity::new(0);
        let position = Pos { x: 10.0, y: 20.0 };

        storage.insert(entity, position).unwrap();

        let result = storage.get(entity).unwrap();

        assert_eq!(result.x, 10.0);
        assert_eq!(result.y, 20.0);
    }
}
