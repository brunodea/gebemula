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
    let wx: i16 = memory.read_byte(cpu::consts::WX_REGISTER_ADDR) as i16 - 7;

    let bg_on: bool = ioregister::LCDCRegister::is_bg_window_display_on(memory);
    let wn_on: bool = ioregister::LCDCRegister::is_window_display_on(memory);
    let mut is_window: bool = false;

    let startx: i16 = if bg_on { 0 } else { wx };

    let (tile_table_addr_pattern_0, is_tile_number_signed) =
        if ioregister::LCDCRegister::is_tile_data_0(&memory) {
            (consts::TILE_DATA_TABLE_0_ADDR_START + 0x800, true)
        } else {
            (consts::TILE_DATA_TABLE_1_ADDR_START, false)
        };

    let mut tile_row: u16 = (ypos/8)*32; //TODO ypos >> 3 is faster?
    let mut tile_line: u16 = (ypos % 8)*2;
    for i in startx..consts::DISPLAY_WIDTH_PX as i16 {
        if wn_on && wx < consts::DISPLAY_WIDTH_PX as i16 + 7 && i >= wx && !is_window {
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
                scx + (i as u8)
            } else {
                (i - wx) as u8
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

pub fn draw_sprites(buffer: &mut [u8; 160*144*4], memory: &Memory) {
    let curr_line: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
    if curr_line >= consts::DISPLAY_HEIGHT_PX {
        return;
    }
    let mut index: u16 = 39 * 4;
    while index > 0 {
        let sprite_8_16: bool = ioregister::LCDCRegister::is_sprite_8_16_on(memory);
        let height: u8 = if sprite_8_16 { 16 } else { 8 };
        //TODO draw sprites based on X priority.
        let y: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index) - 16; //y = 0 || y >= 160 hides the sprite
        if (curr_line < y) || (curr_line > y + height) {
            //outside sprite
            index -= 4;
            continue;
        }
        let x: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 1) - 8; //x = 0 || x >= 168 hides the sprite
        let tile_number: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 2);

        let mut tile_location: u16 = consts::SPRITE_PATTERN_TABLE_ADDR_START +
            (tile_number as u16 * consts::TILE_SIZE_BYTES as u16);

        let flags: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 3);
        let above_bg: bool = (flags >> 7) & 0b1 == 0b1;
        let y_flip: bool = (flags >> 6) & 0b1 == 0b1;
        let x_flip: bool = (flags >> 5) & 0b1 == 0b1;
        let obp0: bool = (flags >> 4) & 0b1 == 0b0;

        let num_bits: u8 = if sprite_8_16 { 8*8 } else { 8*16 };
        for i in 0..num_bits {
            let tile_col: u8 = i % 8;
            let tile_line: u8 = i / 8;

            if sprite_8_16 {
                if tile_line < 8 {
                    tile_location = consts::SPRITE_PATTERN_TABLE_ADDR_START +
                        ((tile_number & 0xFE) as u16 * consts::TILE_SIZE_BYTES as u16);
                } else {
                    tile_location = consts::SPRITE_PATTERN_TABLE_ADDR_START +
                        ((tile_number | 0x01) as u16 * consts::TILE_SIZE_BYTES as u16);
                }
            }

            //tile_line*2 because each tile uses 2 bytes per line.
            let lhs: u8 = memory.read_byte(tile_location + (tile_line as u16 * 2)) >> (7 - tile_col);
            let rhs: u8 = memory.read_byte(tile_location + (tile_line as u16 * 2) + 1) >> (7 - tile_col);
            let pixel_data: u8 = ((rhs << 1) & 0b10) | (lhs & 0b01);
            if pixel_data == 0 {
                continue;
            }
            let pixel_index: u8 =
                ioregister::sprite_palette(obp0, pixel_data, memory);
            let (r,g,b,a) = match pixel_index {
                0b00 => (255,255,255,255),
                0b01 => (192,192,192,255),
                0b10 => (96,96,96,255),
                0b11 => (0,0,0,255),
                _ => unreachable!(),
            };

            let mut pos: usize;

            if y_flip {
                pos =
                    if sprite_8_16 {
                        (y + 15 - tile_line) as usize * consts::DISPLAY_WIDTH_PX as usize * 4
                    } else {
                        (y + 7 - tile_line) as usize * consts::DISPLAY_WIDTH_PX as usize * 4
                    };
            } else {
                pos = (y + tile_line) as usize * consts::DISPLAY_WIDTH_PX as usize * 4;
            }

            if x_flip {
                pos += (x + 7 - tile_col) as usize * 4;
            } else {
                pos += (x + tile_col) as usize * 4;
            }

            //if above_bg {
            {
                buffer[pos] = r;
                buffer[pos+1] = g;
                buffer[pos+2] = b;
                buffer[pos+3] = a;
            }
        }

        index -= 4;
    }
}
