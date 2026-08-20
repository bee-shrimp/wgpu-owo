// ------------------------------------------------------------------- logical size of pixel art

pub const LOGIC_WIDTH: u8 = 128;
pub const LOGIC_HEIGHT: u8 = 128;

// ------------------------------------------------------------------- number of col and row

pub const MAX_COL: u8 = 4;
pub const MAX_ROW: u8 = 4;

// ------------------------------------------------------------------- consts for rect

pub const RECT_WIDTH: u8 = LOGIC_WIDTH / MAX_COL;

pub const RECT_HEIGHT: u8 = LOGIC_HEIGHT / MAX_ROW;

pub const MAX_ENTITIES: usize = (MAX_COL * MAX_ROW) as usize;

// ------------------------------------------------------------------- sprite

pub const SPRITE_SHEET_SIZE: u16 = 256;
pub const SPRITE_GRID_SIZE: u8 = 16;
