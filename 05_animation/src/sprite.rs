// ------------------------------------------------------------------- enum of sprites
use crate::config::{SPRITE_GRID_SIZE, SPRITE_SHEET_SIZE};
use crate::ecs::Size;

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Sprite {
    #[default]
    Walk,
}

// ------------------------------------------------------------------- return SpriteData of sprite

/// returns `SpriteData` of self
impl Sprite {
    pub fn uv_data(self, flame: u8) -> SpriteData {
        let gx = flame;
        let gy = match self {
            Self::Walk => 0,
        };

        SpriteData::new(GridPos { gx, gy })
    }
}

// ------------------------------------------------------------------- structs for SpriteData

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

// ------------------------------------------------------------------- SpriteData struct

#[derive(Debug, Clone, Copy)]
pub struct SpriteData {
    pub uv_offset: UVOffset,
    pub uv_size: Size,
}

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
