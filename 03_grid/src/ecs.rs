// ------------------------------------------------------------------- imports

use anyhow::Context;

use crate::world::{MAX_COL, NUM_GRIDS, RECT_HEIGHT, RECT_WIDTH};

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

#[derive(Debug, Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub fn to_f32_array(self) -> [f32; 3] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        ]
    }
}

// ------------------------------------------------------------------- entity manager struct

pub struct EntityManager {
    next_id: u8,
    free_list: Vec<u8>,
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
            id
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
    colours: Vec<Option<Colour>>,
    pub max_entities: usize,
}

impl Components {
    // --------------------------------------------------------------- create new components struct

    pub fn new() -> Self {
        let max_entities = NUM_GRIDS as usize;
        Self {
            positions: vec![None; max_entities],
            sizes: vec![None; max_entities],
            colours: vec![None; max_entities],
            max_entities,
        }
    }

    // --------------------------------------------------------------- add Some(x) to vec[id]

    pub fn add_position(&mut self, id: u8, pos: Pos) -> anyhow::Result<()> {
        if id >= u8::try_from(self.max_entities)? {
            anyhow::bail!("entity id over flow")
        }

        self.positions[id as usize] = Some(pos);
        Ok(())
    }

    pub fn add_size(&mut self, id: u8, size: Size) -> anyhow::Result<()> {
        if id >= u8::try_from(self.max_entities)? {
            anyhow::bail!("entity id over flow")
        }
        self.sizes[id as usize] = Some(size);

        Ok(())
    }

    pub fn add_colour(&mut self, id: u8, colour: Colour) -> anyhow::Result<()> {
        if id >= u8::try_from(self.max_entities)? {
            anyhow::bail!("entity id over flow")
        }
        self.colours[id as usize] = Some(colour);

        Ok(())
    }

    // --------------------------------------------------------------- get Some(x) in vec[id]

    pub fn get_position(&self, id: u8) -> Option<&Pos> {
        self.positions.get(id as usize).and_then(|p| p.as_ref())
    }

    pub fn get_size(&self, id: u8) -> Option<&Size> {
        self.sizes.get(id as usize).and_then(|s| s.as_ref())
    }

    pub fn get_colour(&self, id: u8) -> Option<&Colour> {
        self.colours.get(id as usize).and_then(|c| c.as_ref())
    }

    pub fn get_colour_mut(&mut self, id: u8) -> Option<&mut Colour> {
        self.colours.get_mut(id as usize).and_then(|c| c.as_mut())
    }
}

// ------------------------------------------------------------------- react system struct

pub struct ReactSystem;

impl ReactSystem {
    pub fn update(components: &mut Components, click_pos: Pos) -> anyhow::Result<()> {
        let gx = click_pos.x as u8 / RECT_WIDTH;
        let gy = click_pos.y as u8 / RECT_HEIGHT;

        let idx = gy * MAX_COL + gx;

        let new_colour = Colour {
            r: 255,
            g: 255,
            b: 255,
        };

        *components
            .get_colour_mut(idx)
            .context("failed to get colour mut")? = new_colour;

        Ok(())
    }
}
