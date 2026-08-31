// ---------------------------------------------------------------- imports
use anyhow::Result;

use crate::animation::AnimationState;
use crate::config::MAX_ENTITIES;
use crate::ecs::entity::Entity;

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

#[derive(Debug, Clone, Copy)]
pub struct Components {
    pub positions: ComponentStorage<Pos>,
    pub sizes: ComponentStorage<Size>,
    pub animations: ComponentStorage<AnimationState>,
}

// ---------------------------------------------------------------- component storage

#[derive(Debug, Clone, Copy)]
pub struct ComponentStorage<T> {
    components: [T; MAX_ENTITIES],
    pub alive: [bool; MAX_ENTITIES],
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

    // pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> + '_ {
    //     self.components
    //         .iter()
    //         .enumerate()
    //         .filter(|(i, _)| self.alive.get(*i).is_some_and(|state| *state))
    //         .map(|(i, t)| (Entity::new(i), t))
    // }
    //
    // pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> + '_ {
    //     self.components
    //         .iter_mut()
    //         .enumerate()
    //         .filter(|(i, _)| self.alive.get(*i).is_some_and(|state| *state))
    //         .map(|(i, t)| (Entity::new(i), t))
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
