use graphics::consts;
use super::super::util::util;
use super::super::mem::mem::Memory;
use super::super::cpu::ioregister;
use super::super::cpu;

pub fn update_line_buffer(buffer: &mut [u8; 160*144*4], memory: &Memory) {
    let curr_line: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
    if curr_line >= consts::DISPLAY_HEIGHT_PX {
        return;
    }
    let scx: u8 = memory.read_byte(cpu::consts::SCX_REGISTER_ADDR);
    let mut ypos: u16 =
        curr_line as u16 + memory.read_byte(cpu::consts::SCY_REGISTER_ADDR) as u16;
    let wy: u8 = memory.read_byte(cpu::consts::WY_REGISTER_ADDR);
    let wx: u8 = memory.read_byte(cpu::consts::WX_REGISTER_ADDR) - 7;

    let bg_on: bool = ioregister::LCDCRegister::is_bg_window_display_on(memory);
    let wn_on: bool = ioregister::LCDCRegister::is_window_display_on(memory);
    let mut is_window: bool = false;

    let startx: u8 = if bg_on { 0 } else { wx };

    let (tile_table_addr_pattern_0, is_tile_number_signed) =
        if ioregister::LCDCRegister::is_tile_data_0(&memory) {
            (consts::TILE_DATA_TABLE_0_ADDR_START + 0x800, true)
        } else {
            (consts::TILE_DATA_TABLE_1_ADDR_START, false)
        };

    let mut tile_row: u16 = (ypos/8)*32; //TODO ypos >> 3 is faster?
    let mut tile_line: u16 = (ypos % 8)*2;
    for i in startx..consts::DISPLAY_WIDTH_PX {
        if wn_on && wx < consts::DISPLAY_WIDTH_PX + 7 && i >= wx && !is_window {
            //Display Window
            if curr_line >= wy && wy < consts::DISPLAY_HEIGHT_PX {
                is_window = true;
                ypos = (curr_line - wy) as u16;
                tile_row = (ypos/8)*32; //TODO ypos >> 3 is faster?
                tile_line = (ypos % 8)*2;
            }
        }
        let xpos: u8 =
            if !is_window {
                scx + i
            } else {
                i - wx
            };

        let addr_start =
            if !is_window {
                if ioregister::LCDCRegister::is_bg_tile_map_display_normal(&memory) {
                    consts::BG_NORMAL_ADDR_START
                } else {
                    consts::BG_WINDOW_ADDR_START
                }
            } else {
                if ioregister::LCDCRegister::is_window_tile_map_display_normal(&memory) {
                    consts::BG_NORMAL_ADDR_START
                } else {
                    consts::BG_WINDOW_ADDR_START
                }
            };

        let tile_col: u16 = (xpos as u16)/8; //TODO xpos >> 3 is faster?
        let tile_addr: u16 = addr_start + tile_row + tile_col;
        let tile_location: u16 =
            if is_tile_number_signed {
                let tile_number: u16 = util::sign_extend(memory.read_byte(tile_addr));
                if util::is_neg16(tile_number) {
                    tile_table_addr_pattern_0 - (util::twos_complement(tile_number) * consts::TILE_SIZE_BYTES as u16)
                } else {
                    tile_table_addr_pattern_0 + (tile_number * consts::TILE_SIZE_BYTES as u16)
                }
            } else {
                tile_table_addr_pattern_0 + (memory.read_byte(tile_addr) as u16 * consts::TILE_SIZE_BYTES as u16)
            };
        let tile_col: u8 = (xpos % 8) as u8;
        //two bytes representing 8 pixel indexes
        let lhs: u8 = memory.read_byte(tile_location + tile_line) >> (7 - tile_col);
        let rhs: u8 = memory.read_byte(tile_location + tile_line + 1) >> (7 - tile_col);
        let pixel_index: u8 =
            ioregister::bg_window_palette(((rhs << 1) & 0b10) | (lhs & 0b01), memory);

        //Apply palette
        let (r,g,b) = match pixel_index {
            0b00 => (255,255,255),
            0b01 => (192,192,192),
            0b10 => (96,96,96),
            0b11 => (0,0,0),
            _ => unreachable!(),
        };

        let pos: usize = (curr_line as usize * consts::DISPLAY_WIDTH_PX as usize * 4) +
            (i as usize * 4);

        buffer[pos] = r;
        buffer[pos+1] = g;
        buffer[pos+2] = b;
        buffer[pos+3] = 255; //alpha
    }
}
