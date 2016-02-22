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

    pub fn indexed_pixels(&self, memory: &Memory) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(consts::TILE_SIZE_PIXELS*consts::TILE_SIZE_PIXELS);
        for i in 0..consts::TILE_SIZE_PIXELS {
            for j in 0..consts::TILE_SIZE_PIXELS {
                res.push(ioregister::bg_window_palette(self.pixel_data(i,j), memory));
            }
        }
        res
    }
}

pub struct BGWindowLayer {
    last_addr: u16,
    addr_start: u16,
    addr_end: u16,
    tile_table_addr_pattern_0: u16,
    is_tile_number_signed: bool,
    is_background: bool,
}

impl BGWindowLayer {
    pub fn new(is_background: bool, memory: &Memory) -> BGWindowLayer {
        let (addr_start, addr_end) =
            if is_background {
                if ioregister::LCDCRegister::is_bg_tile_map_display_normal(&memory) {
                    (consts::BG_NORMAL_ADDR_START, consts::BG_NORMAL_ADDR_END)
                } else {
                    (consts::BG_WINDOW_ADDR_START, consts::BG_WINDOW_ADDR_END)
                }
            } else {
                if ioregister::LCDCRegister::is_window_tile_map_display_normal(&memory) {
                    (consts::BG_NORMAL_ADDR_START, consts::BG_NORMAL_ADDR_END)
                } else {
                    (consts::BG_WINDOW_ADDR_START, consts::BG_WINDOW_ADDR_END)
                }
            };
        let (tile_table_addr_pattern_0, is_signed) =
            if ioregister::LCDCRegister::is_tile_data_0(&memory) {
                (consts::TILE_DATA_TABLE_0_ADDR_START + 0x800, true)
            } else {
                (consts::TILE_DATA_TABLE_1_ADDR_START, false)
            };
        BGWindowLayer {
            last_addr: addr_start,
            addr_start: addr_start,
            addr_end: addr_end,
            tile_table_addr_pattern_0: tile_table_addr_pattern_0,
            is_tile_number_signed: is_signed,
            is_background: is_background,
        }
    }

    pub fn next_tile(&mut self, memory: &Memory) -> Option<Tile> {
        if self.last_addr == self.addr_end {
            //TODO use circularity?
            self.last_addr = self.addr_start;
            None
        } else {
            let tile_number: u8 = memory.read_byte(self.last_addr);
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
            self.last_addr += 1;
            let mut tile_data: [u8; consts::TILE_SIZE_BYTES] = [0; consts::TILE_SIZE_BYTES];
            for i in 0..consts::TILE_SIZE_BYTES {
                tile_data[i] = memory.read_byte(tile_location + i as u16);
            }
            Some(Tile::new(tile_data))
        }
    }

    //returns list of indexes to pallet with the size of the display.
    pub fn resize_to_display(&mut self, memory: &Memory) -> Vec<u8> {
        //let start: usize = (bg_line * consts::BG_MAP_SIZE_PIXELS as usize) + bg_column;
        let mut res: Vec<u8> = Vec::with_capacity(
            (consts::DISPLAY_WIDTH_PX*consts::DISPLAY_HEIGHT_PX) as usize);
        let mut line: usize = 
            if self.is_background {
                memory.read_byte(cpu::consts::SCY_REGISTER_ADDR) as usize
            } else {
                0
            };
        let mut column: usize = 
            if self.is_background {
                memory.read_byte(cpu::consts::SCX_REGISTER_ADDR) as usize
            } else {
                0
            };

        let bg: Vec<u8> = self.indexed_pixels(memory);
        for _ in 0..(consts::DISPLAY_HEIGHT_PX*consts::DISPLAY_WIDTH_PX) {
            res.push(bg[(line*consts::DISPLAY_WIDTH_PX as usize) + column]);
            column += 1;
            if column == consts::DISPLAY_WIDTH_PX as usize {
                column = 0;
                line += 1;
                if line == consts::DISPLAY_HEIGHT_PX as usize {
                    line = 0;
                }
            }
        }
        res
    }

    pub fn indexed_pixels(&mut self, memory: &Memory) -> Vec<u8> {
        //bg map has 32x32 tiles and each tile has 8x8 pixels.
        let mut image: Vec<u8> = Vec::with_capacity(
            consts::BG_MAP_SIZE_TILES*consts::TILE_SIZE_PIXELS*consts::TILE_SIZE_PIXELS);
        while let Some(tile) = self.next_tile(memory) {
            for pixel in tile.indexed_pixels(memory) {
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

