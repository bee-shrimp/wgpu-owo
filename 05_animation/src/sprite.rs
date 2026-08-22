// ---------------------------------------------------------------- imports

use crate::config::{SPRITE_GRID_SIZE, SPRITE_SHEET_SIZE};
use crate::ecs::Size;

// ---------------------------------------------------------------- enum of sprites
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Sprite {
    #[default]
    Walk,
}

pub struct Animation {
    pub flames: Vec<GridPos>,
    pub flame_duration: f32,
    pub max_flame_idx: u8,
}

// ---------------------------------------------------------------- Sprite methods

impl Sprite {
    /// returns `Animation` of self
    pub fn animation(self) -> Animation {
        match self {
            Self::Walk => Animation {
                flames: vec![
                    GridPos { gx: 0, gy: 0 },
                    GridPos { gx: 1, gy: 0 },
                    GridPos { gx: 2, gy: 0 },
                    GridPos { gx: 3, gy: 0 },
                    GridPos { gx: 4, gy: 0 },
                    GridPos { gx: 5, gy: 0 },
                    GridPos { gx: 6, gy: 0 },
                    GridPos { gx: 7, gy: 0 },
                ],
                flame_duration: 0.16,
                max_flame_idx: 7,
            },
        }
    }

    /// returns `SpriteData` of self
    pub fn uv_data(self, flame_idx: u8) -> SpriteData {
        let g_pos = self.animation().flames[flame_idx as usize];
        SpriteData::new(g_pos)
    }
}

// ---------------------------------------------------------------- structs for SpriteData

#[derive(Debug, Clone, Copy)]
pub struct UVOffset {
    pub u: f32,
    pub v: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct GridPos {
    pub gx: u8,
    pub gy: u8,
}

// ---------------------------------------------------------------- SpriteData struct

#[derive(Debug, Clone, Copy)]
pub struct SpriteData {
    pub uv_offset: UVOffset,
    pub uv_size: Size,
}

// ---------------------------------------------------------------- SpriteData methods

impl SpriteData {
    fn new(g_pos: GridPos) -> Self {
        let sprite_sheet_size = f32::from(SPRITE_SHEET_SIZE);
        let grid_size = f32::from(SPRITE_GRID_SIZE);

        let cell_size = sprite_sheet_size / grid_size;

        let pixel_u = f32::from(g_pos.gx) * cell_size;
        let pixel_v = f32::from(g_pos.gy) * cell_size;

        let uv_offset_u = pixel_u / sprite_sheet_size;
        let uv_offset_v = pixel_v / sprite_sheet_size;

        let uv_size = cell_size / sprite_sheet_size;

        Self {
            uv_offset: UVOffset {
                u: uv_offset_u,
                v: uv_offset_v,
            },
            uv_size: Size {
                w: uv_size,
                h: uv_size,
            },
        }
    }
}
