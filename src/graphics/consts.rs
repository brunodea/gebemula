pub const DISPLAY_HEIGHT_PX: u8 = 144;
pub const DISPLAY_WIDTH_PX: u8 = 160;

pub const TILE_SIZE_BYTES: usize = 16;

pub const TILE_DATA_TABLE_0_ADDR_START: u16 = 0x8800;
pub const TILE_DATA_TABLE_1_ADDR_START: u16 = 0x8000;

pub const SPRITE_PATTERN_TABLE_ADDR_START: u16 = 0x8000;
pub const SPRITE_ATTRIBUTE_TABLE: u16 = 0xFE00;

pub const BG_NORMAL_ADDR_START: u16 = 0x9800;
pub const BG_WINDOW_ADDR_START: u16 = 0x9C00;

pub const DMG_PALETTE: [(u8, u8, u8); 4] = [
    (137, 143, 110),
    (87, 92, 72),
    (35, 40, 34),
    (16, 21, 21)];
