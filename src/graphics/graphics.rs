use graphics::consts;
use super::super::util::util;
use super::super::mem::mem::Memory;
use super::super::cpu::ioregister;
use super::super::cpu;

pub struct BGWindowLayer {
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
            addr_start: addr_start,
            addr_end: addr_end,
            tile_table_addr_pattern_0: tile_table_addr_pattern_0,
            is_tile_number_signed: is_signed,
            is_background: is_background,
        }
    }

    //returns updated line
    pub fn update_buffer(&self, buffer: &mut [u8], memory: &Memory) -> Option<u8> {
        //TODO verify all window stuff.
        let curr_line: u8 = ioregister::LYRegister::value(memory);
        if curr_line > 143 {
            return None;
        }
        let scx: u8 = memory.read_byte(cpu::consts::SCX_REGISTER_ADDR);
        let mut ypos: u16 = curr_line as u16;
        if self.is_background {
            ypos += memory.read_byte(cpu::consts::SCY_REGISTER_ADDR) as u16;
        } else {
            ypos -= memory.read_byte(cpu::consts::WY_REGISTER_ADDR) as u16;
        }
        let tile_row: u16 = (ypos/8)*32; //TODO ypos >> 3 is faster?
        for i in 0..consts::DISPLAY_WIDTH_PX {
            let xpos: u16 =
                if self.is_background {
                    ((scx as u32) + i) as u16
                } else {
                    (i - memory.read_byte(cpu::consts::WX_REGISTER_ADDR) as u32) as u16
                };
            let tile_col: u16 = xpos/8; //TODO xpos >> 3 is faster?
            let tile_addr: u16 = self.addr_start + tile_row + tile_col;
            let tile_number: u8 = memory.read_byte(tile_addr);
            //each tile uses 16 bytes.
            let tile_location: u16 =
                if self.is_tile_number_signed {
                    let tile_number16: u16 = util::sign_extend(tile_number)*16;
                    if util::is_neg16(tile_number16) {
                        self.tile_table_addr_pattern_0 - util::twos_complement(tile_number16)
                    } else {
                        self.tile_table_addr_pattern_0 + tile_number16
                    }
                } else {
                    self.tile_table_addr_pattern_0 + ((tile_number as u16)*16)
                };
            let tile_line: u16 = ypos % 8;
            let tile_col: u8 = (xpos % 8) as u8;
            //two bytes representing 8 pixel indexes
            let lhs: u8 = memory.read_byte(tile_location + tile_line) >> (7 - tile_col);
            let rhs: u8 = memory.read_byte(tile_location + tile_line + 1) >> (7 - tile_col);
            let pixel_index: u8 =
                ioregister::bg_window_palette(((lhs << 1) | rhs) & 0b11, memory);

            //Apply palette
            let (r,g,b) = match pixel_index {
                0b00 => (255,255,255),
                0b01 => (204,204,204),
                0b10 => (119,119,119),
                0b11 => (0,0,0),
                _ => unreachable!(),
            };

            //*4 and +4 because of rgba.
            //let pos: usize =
            //    ((curr_line as u32 * consts::DISPLAY_WIDTH_PX * 4) + (i*4)) as usize;
            let pos: usize = (i*4) as usize;
            buffer[pos] = r;
            buffer[pos+1] = g;
            buffer[pos+2] = b;
            buffer[pos+3] = 255; //alpha
        }

        Some(curr_line)
    }
}


pub fn apply_palette(indexed_image: &[u8]) -> Vec<u8> {
    //*4 because it is RGBA
    let mut res: Vec<u8> = Vec::with_capacity(indexed_image.len()*4);
    for color_index in indexed_image {
    }
    res
}

