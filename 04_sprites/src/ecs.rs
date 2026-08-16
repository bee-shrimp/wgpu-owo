// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::world::{MAX_COL, NUM_RECTS, RECT_HEIGHT, RECT_WIDTH, Sprite};

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

// ------------------------------------------------------------------- entity manager struct

pub struct EntityManager {
    next_id: u8,
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

    pub fn spawn(&mut self) -> u8 {
        if let Some(id) = self.free_list.pop() {
            id as u8
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    // --------------------------------------------------------------- store id for reuse

    // fn despawn(&mut self, id: u8) {
    //     self.free_list.push(id);
    // }
}

// ------------------------------------------------------------------- components struct

pub struct Components {
    positions: Vec<Option<Pos>>,
    sizes: Vec<Option<Size>>,
    sprites: Vec<Option<Sprite>>,
    pub max_entities: u8,
}

impl Components {
    // --------------------------------------------------------------- create new components struct

    pub fn new() -> Self {
        let max_entities = usize::from(NUM_RECTS);
        Self {
            positions: vec![None; max_entities],
            sizes: vec![None; max_entities],
            sprites: vec![None; max_entities],
            max_entities: NUM_RECTS,
        }
    }

    // --------------------------------------------------------------- add Some(x) to vec[id]

    pub fn add_position(&mut self, id: u8, pos: Pos) -> anyhow::Result<()> {
        if id >= self.max_entities {
            anyhow::bail!("entity id over flow");
        }

        self.positions[id as usize] = Some(pos);
        Ok(())
    }

    pub fn add_size(&mut self, id: u8, size: Size) -> anyhow::Result<()> {
        if id >= self.max_entities {
            anyhow::bail!("entity id over flow")
        }
        self.sizes[id as usize] = Some(size);

        Ok(())
    }

    pub fn add_sprite(&mut self, id: u8, sprite: Sprite) -> anyhow::Result<()> {
        if id >= self.max_entities {
            anyhow::bail!("entity id over flow")
        }
        self.sprites[id as usize] = Some(sprite);

        Ok(())
    }

    // --------------------------------------------------------------- get Some(x) in vec[id]

    pub fn get_position(&self, id: u8) -> Option<&Pos> {
        self.positions.get(id as usize).and_then(|p| p.as_ref())
    }

    pub fn get_size(&self, id: u8) -> Option<&Size> {
        self.sizes.get(id as usize).and_then(|s| s.as_ref())
    }

    pub fn get_sprite(&self, id: u8) -> Option<&Sprite> {
        self.sprites.get(id as usize).and_then(|s| s.as_ref())
    }

    pub fn get_sprite_mut(&mut self, id: u8) -> Option<&mut Sprite> {
        self.sprites.get_mut(id as usize).and_then(|s| s.as_mut())
    }
}

// ------------------------------------------------------------------- react system struct

pub struct ReactSystem;

impl ReactSystem {
    pub fn update(components: &mut Components, click_pos: Pos) -> anyhow::Result<()> {
        let gx = (click_pos.x / f32::from(RECT_WIDTH)).floor() as u8;
        let gy = (click_pos.y / f32::from(RECT_HEIGHT)).floor() as u8;

        let idx = gy * MAX_COL + gx;

        let sprite = components.get_sprite(idx).context("failed to get sprite")?;

        let new_sprite = match sprite {
            Sprite::RedFlower => Sprite::YellowFlower,
            Sprite::YellowFlower => Sprite::RedFlower,
        };

        *components
            .get_sprite_mut(idx)
            .context("failed to get sprite mut")? = new_sprite;

        Ok(())
    }
}
