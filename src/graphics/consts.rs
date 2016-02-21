pub const DISPLAY_HEIGHT_PX: u32 = 144;
pub const DISPLAY_WIDTH_PX: u32 = 160;
pub const DISPLAY_PIXELS: usize = (DISPLAY_WIDTH_PX*DISPLAY_HEIGHT_PX) as usize;

pub const TILE_SIZE_BYTES: usize = 16;
pub const TILE_SIZE_PIXELS: usize = 8;

pub const TILE_DATA_TABLE_0_ADDR_START: u16 = 0x8800;
pub const TILE_DATA_TABLE_0_ADDR_END: u16 = 0x97FF;
pub const TILE_DATA_TABLE_1_ADDR_START: u16 = 0x8000;
pub const TILE_DATA_TABLE_1_ADDR_END: u16 = 0x8FFF;

pub const BG_MAP_SIZE_TILES: usize = 32*32;
pub const BG_MAP_SIZE_PIXELS: usize = 256;

pub const BG_NORMAL_ADDR_START: u16 = 0x9800;
pub const BG_NORMAL_ADDR_END: u16 = 0x9BFF;
pub const BG_WINDOW_ADDR_START: u16 = 0x9C00;
pub const BG_WINDOW_ADDR_END: u16 = 0x9FFF;
