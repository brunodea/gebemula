use graphic::consts;
use super::super::util::util;
use super::super::mem::mem::Memory;
use super::super::cpu::ioregister;

struct Tile {
    data: [u8; consts::TILE_SIZE_BYTES],
}

impl Tile {
    pub fn new(data: [u8; consts::TILE_SIZE_BYTES]) -> Tile {
        Tile {
            data: data,
        }
    }

    //returns 0, 1, 2 or 3; representing the gray shade of pixel in
    //position pixel_line/pixel_column;
    //pixel_line and pixel_column ranges are 0-7.
    //pixel_line 0 is the upper 8 pixels;
    //pixel_column 0 is the leftmost pixel;
    pub fn pixel_data(&self, pixel_line: u8, pixel_column: u8) -> u8 {
        let lhs: u8 = (self.data[pixel_line as usize * 2] >> pixel_column) & 0b1;
        let rhs: u8 = (self.data[(pixel_line as usize * 2) + 1] >> pixel_column) & 0b1;

        (lhs << 1) | rhs
    }
}

struct BackgroundMap {
    bg_last_addr: u16,
    bg_addr_end: u16,
    tile_table_addr_pattern_0: u16,
    is_tile_number_signed: bool,
    memory: Box<Memory>,
}

impl BackgroundMap {
    pub fn new(memory: Box<Memory>) -> BackgroundMap {
        let (bg_addr_start, bg_addr_end) =
            if ioregister::LCDCRegister::is_bg_tile_map_display_normal(&memory) {
                (consts::BG_NORMAL_ADDR_START, consts::BG_NORMAL_ADDR_END)
            } else {
                (consts::BG_WINDOW_ADDR_START, consts::BG_WINDOW_ADDR_END)
            };
        let (tile_table_addr_pattern_0, is_signed) =
            if ioregister::LCDCRegister::is_tile_data_0(&memory) {
                (consts::TILE_DATA_TABLE_0_ADDR_START, false)
            } else {
                (consts::TILE_DATA_TABLE_1_ADDR_START + 0x800, true)
            };
        BackgroundMap {
            bg_addr_end: bg_addr_end,
            bg_last_addr: bg_addr_start,
            tile_table_addr_pattern_0: tile_table_addr_pattern_0,
            is_tile_number_signed: is_signed,
            memory: memory,
        }
    }
}

impl Iterator for BackgroundMap {
    type Item = Tile;

    fn next(&mut self) -> Option<Tile> {
        if self.bg_last_addr == self.bg_addr_end {
            None
        } else {
            self.bg_last_addr += 1;
            let tile_number: u8 = self.memory.read_byte(self.bg_last_addr);
            let tile_location: u16 =
                if self.is_tile_number_signed{
                    let tile_number16: u16 = util::sign_extend(tile_number);
                    if util::is_neg16(tile_number16) {
                        self.tile_table_addr_pattern_0 - util::twos_complement(tile_number16)
                    } else {
                        self.tile_table_addr_pattern_0 + tile_number16
                    }
                } else {
                    self.tile_table_addr_pattern_0 + (tile_number as u16)
                };
            let mut tile_data: [u8; consts::TILE_SIZE_BYTES] = [0; consts::TILE_SIZE_BYTES];
            for i in 0..consts::TILE_SIZE_BYTES {
                tile_data[i] = self.memory.read_byte(tile_location + i as u16);
            }
            Some(Tile::new(tile_data))
        }
    }
}
