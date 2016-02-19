use graphics::consts;
use super::super::util::util;
use super::super::mem::mem::Memory;
use super::super::cpu::ioregister;

pub struct Tile {
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

    pub fn rgb(&self) -> Vec<(u8, u8, u8)> {
        let mut res: Vec<(u8, u8, u8)> = Vec::new();
        for i in 0..consts::TILE_SIZE_PIXELS {
            for j in 0..consts::TILE_SIZE_PIXELS {
                //let rgb = match ioregister::bg_window_palette(self.pixel_data(i, j) {
                let rgb = match self.pixel_data(i as u8, j as u8) {
                    0b00 => (255, 255, 255),
                    0b01 => (128, 128, 128),
                    0b10 => (64, 64, 64),
                    0b11 => (0, 0, 0),
                    _ => unreachable!(),
                };
                res.push(rgb);
            }
        }
        res.clone()
    }
}

pub struct BackgroundMap {
    bg_last_addr: u16,
    bg_addr_end: u16,
    tile_table_addr_pattern_0: u16,
    is_tile_number_signed: bool,
}

impl BackgroundMap {
    pub fn new(memory: &Memory) -> BackgroundMap {
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
        }
    }

    pub fn next_tile(&mut self, memory: &Memory) -> Option<Tile> {
        if self.bg_last_addr == self.bg_addr_end {
            None
        } else {
            self.bg_last_addr += 1;
            let tile_number: u8 = memory.read_byte(self.bg_last_addr);
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
                tile_data[i] = memory.read_byte(tile_location + i as u16);
            }
            Some(Tile::new(tile_data))
        }
    }

    pub fn background_rgb(memory: &Memory) -> Vec<(u8, u8, u8)> {
        let mut bg_map: BackgroundMap = BackgroundMap::new(memory);
        let mut image: Vec<(u8, u8, u8)> = Vec::new();
        while let Some(tile) = bg_map.next_tile(memory) {
            for pixels in tile.rgb().iter() {
                image.push(*pixels);
            }
        }
        image
    }
}
