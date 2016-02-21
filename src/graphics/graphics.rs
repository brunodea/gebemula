use graphics::consts;
use super::super::util::util;
use super::super::mem::mem::Memory;
use super::super::cpu::ioregister;
use super::super::cpu;

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
    pub fn pixel_data(&self, pixel_line: usize, pixel_column: usize) -> u8 {
        let rhs: u8 = (self.data[pixel_line * 2] >> pixel_column) & 0b1;
        let lhs: u8 = (self.data[(pixel_line * 2) + 1] >> pixel_column) & 0b1;

        (lhs << 1) | rhs
    }

    pub fn rgb(&self, memory: &Memory) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(consts::TILE_SIZE_PIXELS*consts::TILE_SIZE_PIXELS);
        for i in 0..consts::TILE_SIZE_PIXELS {
            for j in 0..consts::TILE_SIZE_PIXELS {
                res.push(ioregister::bg_window_palette(self.pixel_data(i, j), memory));
            }
        }
        res
    }
}

pub struct BackgroundMap {
    bg_last_addr: u16,
    bg_addr_start: u16,
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
                (consts::TILE_DATA_TABLE_0_ADDR_START + 0x800, true)
            } else {
                (consts::TILE_DATA_TABLE_1_ADDR_START, false)
            };
        BackgroundMap {
            bg_last_addr: bg_addr_start,
            bg_addr_start: bg_addr_start,
            bg_addr_end: bg_addr_end,
            tile_table_addr_pattern_0: tile_table_addr_pattern_0,
            is_tile_number_signed: is_signed,
        }
    }

    pub fn next_tile(&mut self, memory: &Memory) -> Option<Tile> {
        if self.bg_last_addr == self.bg_addr_end {
            //TODO use circularity?
            //self.bg_last_addr = self.bg_addr_start;
            None
        } else {
            let tile_number: u8 = memory.read_byte(self.bg_last_addr);
            let tile_location: u16 =
                if self.is_tile_number_signed {
                    let tile_number16: u16 = util::sign_extend(tile_number);
                    if util::is_neg16(tile_number16) {
                        self.tile_table_addr_pattern_0 - util::twos_complement(tile_number16)
                    } else {
                        self.tile_table_addr_pattern_0 + tile_number16
                    }
                } else {
                    self.tile_table_addr_pattern_0 + (tile_number as u16)
                };
            self.bg_last_addr += 1;
            let mut tile_data: [u8; consts::TILE_SIZE_BYTES] = [0; consts::TILE_SIZE_BYTES];
            for i in 0..consts::TILE_SIZE_BYTES {
                tile_data[i] = memory.read_byte(tile_location + i as u16);
            }
            Some(Tile::new(tile_data))
        }
    }

    //returns list of indexes to pallet with the size of the display.
    pub fn display_rgb(&self, memory: &Memory) -> Vec<u8> {
        let bg_line: usize = memory.read_byte(cpu::consts::SCY_REGISTER_ADDR) as usize;
        let bg_column: usize = memory.read_byte(cpu::consts::SCX_REGISTER_ADDR) as usize;

        let start: usize = (bg_line * consts::BG_MAP_SIZE_PIXELS as usize) + bg_column;
        BackgroundMap::background_rgb(memory)[start..(start+consts::DISPLAY_PIXELS)].to_vec()
    }

    pub fn background_rgb(memory: &Memory) -> Vec<u8> {
        let mut bg_map: BackgroundMap = BackgroundMap::new(memory);
        //bg map has 32x32 tiles and each tile has 8x8 pixels.
        let mut image: Vec<u8> = Vec::with_capacity(
            consts::BG_MAP_SIZE_TILES*consts::TILE_SIZE_PIXELS*consts::TILE_SIZE_PIXELS);
        while let Some(tile) = bg_map.next_tile(memory) {
            for pixel in tile.rgb(memory) {
                image.push(pixel);
            }
        }
        image
    }
}


pub fn apply_palette(indexed_image: &[u8]) -> Vec<u8> {
    //*4 because it is RGBA
    let mut res: Vec<u8> = Vec::with_capacity(indexed_image.len()*4);
    for color_index in indexed_image {
        let (r,g,b) = match *color_index {
            0b00 => (255,255,255),
            0b01 => (127,127,127),
            0b10 => (63,63,63),
            0b11 => (0,0,0),
            _ => unreachable!(),
        };
        res.push(r);
        res.push(g);
        res.push(b);
        res.push(255); //alpha
    }
    res
}
