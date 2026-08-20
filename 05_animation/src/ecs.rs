// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::config::{MAX_COL, MAX_ENTITIES, RECT_HEIGHT, RECT_WIDTH};
use crate::world::Sprite;

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

// ------------------------------------------------------------------- struct for entities

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub index: usize,
}

impl Entity {
    fn new(index: usize) -> Self {
        Self { index }
    }
}

// ------------------------------------------------------------------- entity manager struct

pub struct EntityManager {
    next_id: usize,
    free_list: Vec<usize>,
}

impl EntityManager {
    // --------------------------------------------------------------- create new entity manager struct

    pub const fn new() -> Self {
        Self {
            next_id: 0,
            free_list: Vec::new(),
        }
    }

    // --------------------------------------------------------------- return id

    pub fn spawn(&mut self) -> Option<Entity> {
        if let Some(id) = self.free_list.pop() {
            // ------------------------------------------------------- reuse slot
            Some(Entity::new(id))
        } else if self.next_id < MAX_ENTITIES {
            // ------------------------------------------------------- new slot
            let id = self.next_id;
            self.next_id += 1;
            Some(Entity::new(id))
        } else {
            // ------------------------------------------------------- slot is full
            None
        }
    }

    // --------------------------------------------------------------- store id for reuse

    // pub fn despawn(&mut self, id: usize) -> anyhow::Result<()> {
    //     if id >= MAX_ENTITIES {
    //         anyhow::bail!("invalid entity id");
    //     }
    //     self.free_list.push(id);
    //     Ok(())
    // }
}

// ------------------------------------------------------------------- components struct

pub struct Components {
    pub positions: [Pos; MAX_ENTITIES],
    pub sizes: [Size; MAX_ENTITIES],
    pub sprites: [Sprite; MAX_ENTITIES],
    pub alive: [bool; MAX_ENTITIES],
}

impl Components {
    // --------------------------------------------------------------- create new components struct

    pub fn new() -> Self {
        Self {
            positions: std::array::from_fn(|_| Pos::default()),
            sizes: std::array::from_fn(|_| Size::default()),
            sprites: std::array::from_fn(|_| Sprite::RedFlower),
            alive: [false; MAX_ENTITIES],
        }
    }

    // --------------------------------------------------------------- set entity !alive (do this after despawn)

    // pub fn kill_entity(&mut self, id: usize) {
    //     if id < MAX_ENTITIES {
    //         self.alive[id] = false;
    //     }
    // }
    //
    // /// 生存しているすべてのエンティティ ID を返すイテレーター
    // pub fn alive_ids(&self) -> impl Iterator<Item = usize> {
    //     (0..MAX_ENTITIES).filter(|&i| self.alive[i])
    // }
    //
    // /// 位置コンポーネントを持つすべてのエンティティを返す
    // pub fn with_positions(&self) -> impl Iterator<Item = (usize, &Pos)> + '_ {
    //     (0..MAX_ENTITIES)
    //         .filter(move |&i| self.alive[i])
    //         .map(move |i| (i, &self.positions[i]))
    // }
    //
    // /// 位置とサイズの両方を持つエンティティを返す（AABB 用）
    // pub fn with_position_and_size(&self) -> impl Iterator<Item = (usize, &Pos, &Size)> + '_ {
    //     (0..MAX_ENTITIES)
    //         .filter(move |&i| self.alive[i])
    //         .filter_map(move |i| {
    //             // ここでは必ず Some だが、将来拡張の余地を残す
    //             Some((i, &self.positions[i], &self.sizes[i]))
    //         })
    // }
    //
    // --------------------------------------------------------------- add data to array[id]

    pub fn add_alive(&mut self, entity: Entity) -> anyhow::Result<()> {
        if entity.index >= MAX_ENTITIES {
            anyhow::bail!("entity entity.index over flow");
        }

        self.alive[entity.index] = true;
        Ok(())
    }

    pub fn add_position(&mut self, entity: Entity, pos: Pos) -> anyhow::Result<()> {
        if entity.index >= MAX_ENTITIES {
            anyhow::bail!("entity entity.index over flow");
        }

        self.positions[entity.index] = pos;
        Ok(())
    }

    pub fn add_size(&mut self, entity: Entity, size: Size) -> anyhow::Result<()> {
        if entity.index >= MAX_ENTITIES {
            anyhow::bail!("entity entity.index over flow")
        }
        self.sizes[entity.index] = size;

        Ok(())
    }

    pub fn add_sprite(&mut self, entity: Entity, sprite: Sprite) -> anyhow::Result<()> {
        if entity.index >= MAX_ENTITIES {
            anyhow::bail!("entity entity.index over flow")
        }
        self.sprites[entity.index] = sprite;

        Ok(())
    }

    // --------------------------------------------------------------- get Some(x) in vec[entity.index]

    // pub fn get_position(&self, entity: Entity) -> Option<&Pos> {
    //     if entity.index >= MAX_ENTITIES || !self.alive[entity.index] {
    //         return None;
    //     }
    //     Some(&self.positions[entity.index])
    // }
    //
    // pub fn get_size(&self, entity: Entity) -> Option<&Size> {
    //     if entity.index >= MAX_ENTITIES || !self.alive[entity.index] {
    //         return None;
    //     }
    //     Some(&self.sizes[entity.index])
    // }

    pub fn get_sprite(&self, entity: Entity) -> Option<&Sprite> {
        if entity.index >= MAX_ENTITIES || !self.alive[entity.index] {
            return None;
        }
        Some(&self.sprites[entity.index])
    }

    pub fn get_sprite_mut(&mut self, entity: Entity) -> Option<&mut Sprite> {
        if entity.index >= MAX_ENTITIES || !self.alive[entity.index] {
            return None;
        }
        Some(&mut self.sprites[entity.index])
    }
}

// ------------------------------------------------------------------- react system struct

pub struct ReactSystem;

impl ReactSystem {
    pub fn update(components: &mut Components, click_pos: Pos) -> anyhow::Result<()> {
        let gx = (click_pos.x / f32::from(RECT_WIDTH)).floor() as u8;
        let gy = (click_pos.y / f32::from(RECT_HEIGHT)).floor() as u8;

        let idx = usize::from(gy * MAX_COL + gx);

        let sprite = components
            .get_sprite(Entity::new(idx))
            .context("failed to get sprite")?;

        let new_sprite = match sprite {
            Sprite::RedFlower => Sprite::YellowFlower,
            Sprite::YellowFlower => Sprite::RedFlower,
        };

        *components
            .get_sprite_mut(Entity::new(idx))
            .context("failed to get sprite mut")? = new_sprite;

        Ok(())
    }
}
