// ------------------------------------------------------------------- imports

use crate::config::MAX_ENTITIES;
use crate::ecs::entity::Entity;
use crate::sprite::Sprite;

// ------------------------------------------------------------------- struct for rects

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

// ------------------------------------------------------------------- components struct for world

pub struct Components {
    pub positions: ComponentStorage<Pos>,
    pub sizes: ComponentStorage<Size>,
    pub sprites: ComponentStorage<Sprite>,
    pub flames: ComponentStorage<u8>,
    pub elapsed: ComponentStorage<f32>,
}

// ------------------------------------------------------------------- component srorage

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
    pub fn insert(&mut self, entity: Entity, component: T) -> anyhow::Result<()> {
        if entity.index >= MAX_ENTITIES {
            anyhow::bail!("entity index out of bounds");
        }
        self.components[entity.index] = component;
        self.alive[entity.index] = true;
        Ok(())
    }

    /// returns Some(&T) if entity is alive.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        if entity.index >= MAX_ENTITIES || !self.alive[entity.index] {
            return None;
        }
        Some(&self.components[entity.index])
    }

    /// returns Some(&mut T) if entity is alive.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if entity.index >= MAX_ENTITIES || !self.alive[entity.index] {
            return None;
        }
        Some(&mut self.components[entity.index])
    }

    // /// removes T from components array and kill entity(set alive == false).
    // pub fn remove(&mut self, entity: Entity) -> anyhow::Result<()> {
    //     if entity.index >= MAX_ENTITIES || !self.alive[entity.index] {
    //         anyhow::bail!("entity not found");
    //     }
    //     self.alive[entity.index] = false;
    //     Ok(())
    // }
    //
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
    // /// returns how many entities are alive.
    // pub fn count_alive(&self) -> usize {
    //     self.alive.iter().filter(|&&a| a).count()
    // }
}
