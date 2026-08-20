// ------------------------------------------------------------------- imports

use crate::config::MAX_ENTITIES;

// ------------------------------------------------------------------- struct for entities

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub index: usize,
}

impl Entity {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

// ------------------------------------------------------------------- entity manager struct

pub struct EntityManager {
    next_id: usize,
    free_list: Vec<usize>,
}

impl EntityManager {
    /// creates new entity manager struct
    pub const fn new() -> Self {
        Self {
            next_id: 0,
            free_list: Vec::new(),
        }
    }

    /// returns an Entity with an id.
    /// reuses id of despawned entities if there are any.
    /// creates new id if free list is empty.
    /// returns None if entity slot is full.
    pub fn spawn(&mut self) -> Option<Entity> {
        if let Some(id) = self.free_list.pop() {
            //
            // ------------------------------------------------------- reuse id if available
            Some(Entity::new(id))
        } else if self.next_id < MAX_ENTITIES {
            //
            // ------------------------------------------------------- new id
            let id = self.next_id;
            self.next_id += 1;
            Some(Entity::new(id))
        } else {
            // ------------------------------------------------------- slot is full
            None
        }
    }

    // /// stores id for reuse
    // pub fn despawn(&mut self, id: usize) -> anyhow::Result<()> {
    //     if id >= MAX_ENTITIES {
    //         anyhow::bail!("invalid entity id");
    //     }
    //     self.free_list.push(id);
    //     Ok(())
    // }
}
