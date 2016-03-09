use graphics::consts;
use super::super::util::util;
use super::super::mem::mem::Memory;
use super::super::cpu::ioregister;
use super::super::cpu;

pub struct Graphics {
    bg_wn_pixel_indexes: [u8; 160*144],
    pub screen_buffer: [u8; 160*144*4],
    bg_on: bool,
    wn_on: bool,
    sprites_on: bool,
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            screen_buffer: [255; 160*144*4],
            bg_wn_pixel_indexes: [0; 160*144],
            bg_on: true,
            wn_on: true,
            sprites_on: true,
        }
    }

    pub fn update(&mut self, memory: &mut Memory) {
        if ioregister::LCDCRegister::is_lcd_display_enable(memory) {
            self.update_line_buffer(memory);
            self.draw_sprites(memory);
        }
    }

    fn update_line_buffer(&mut self, memory: &Memory) {
        let mut bg_on: bool = ioregister::LCDCRegister::is_bg_window_display_on(memory);
        let mut wn_on: bool = ioregister::LCDCRegister::is_window_display_on(memory);
        if !bg_on && !wn_on {
            return;
        }

        bg_on = bg_on & self.bg_on;
        wn_on = wn_on & self.wn_on;

        let curr_line: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
        if curr_line >= consts::DISPLAY_HEIGHT_PX {
            return;
        }
        let scx: u8 = memory.read_byte(cpu::consts::SCX_REGISTER_ADDR);
        let scy: u8 = memory.read_byte(cpu::consts::SCY_REGISTER_ADDR);
        let mut ypos: u16 =
            if curr_line as u16 + scy as u16 > 255 {
                curr_line as u16 + scy as u16 - 256
            } else {
                scy as u16 + curr_line as u16
            };
        let wy: u8 = memory.read_byte(cpu::consts::WY_REGISTER_ADDR);
        let mut wx: u8 = memory.read_byte(cpu::consts::WX_REGISTER_ADDR);

        if wx < 7 {
            wx = 7 - wx;
        } else {
            wx = wx - 7;
        }

        let mut is_window: bool = false;

        let startx: u8 = if bg_on { 0 } else { wx };

        let (tile_table_addr_pattern_0, is_tile_number_signed) =
            if ioregister::LCDCRegister::is_tile_data_0(&memory) {
                (consts::TILE_DATA_TABLE_0_ADDR_START + 0x800, true)
            } else {
                (consts::TILE_DATA_TABLE_1_ADDR_START, false)
            };

        let mut tile_row: u16 = (ypos/8)*32;
        let mut tile_line: u16 = (ypos % 8)*2;
        for i in startx..consts::DISPLAY_WIDTH_PX {
            if wn_on && wx < consts::DISPLAY_WIDTH_PX && i >= wx && !is_window &&
                curr_line >= wy {

                is_window = true;
                ypos = (curr_line - wy) as u16;
                tile_row = (ypos/8)*32;
                tile_line = (ypos % 8)*2;
            }

            let buffer_pos: usize = (curr_line as usize * consts::DISPLAY_WIDTH_PX as usize) +
                (i as usize);

            if !bg_on && !is_window {
                self.bg_wn_pixel_indexes[buffer_pos] = 0;
                continue;
            }

            let xpos: u16 =
                if !is_window {
                    if scx as u16 + i as u16 > 255 {
                        scx as u16 + i as u16 - 256
                    } else {
                        scx as u16 + i as u16
                    }
                } else {
                    i as u16 - wx as u16
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

            let tile_col: u16 = xpos/8;
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
            let tile_col: u16 = xpos % 8;
            //two bytes representing 8 pixel indexes
            let lhs: u8 = memory.read_byte(tile_location + tile_line) >> (7 - tile_col);
            let rhs: u8 = memory.read_byte(tile_location + tile_line + 1) >> (7 - tile_col);
            let pixel_data: u8 = ((rhs << 1) & 0b10) | (lhs & 0b01);
            let pixel_index: u8 =
                ioregister::bg_window_palette(pixel_data, memory);

            //Apply palette
            let (r,g,b) = match pixel_index {
                0b00 => (255,255,255),
                0b01 => (192,192,192),
                0b10 => (96,96,96),
                0b11 => (0,0,0),
                _ => unreachable!(),
            };

            self.bg_wn_pixel_indexes[buffer_pos] = pixel_data;

            let buffer_pos: usize = buffer_pos * 4; //*4 because of RGBA

            self.screen_buffer[buffer_pos] = r;
            self.screen_buffer[buffer_pos+1] = g;
            self.screen_buffer[buffer_pos+2] = b;
            self.screen_buffer[buffer_pos+3] = 255; //alpha
        }
    }

    fn draw_sprites(&mut self, memory: &Memory) {
        if !ioregister::LCDCRegister::is_sprite_display_on(memory) ||
            !self.sprites_on {
            return;
        }

        let curr_line: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
        if curr_line >= consts::DISPLAY_HEIGHT_PX || !self.sprites_on {
            return;
        }
        let mut index: u16 = 160; //40*4: 40 sprites that use 4 bytes
        while index > 0 {
            index -= 4;
            let sprite_8_16: bool = ioregister::LCDCRegister::is_sprite_8_16_on(memory);
            let height: u8 = if sprite_8_16 { 16 } else { 8 };
            //TODO draw sprites based on X priority.
            let mut y: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index);
            if y == 0 || y >= 160 {
                continue;
            }
            if y < 16 {
                y = 0;
            } else {
                y = y - 16;
            }
            if (curr_line < y) || (curr_line >= y + height) {
                //outside sprite
                continue;
            }
            let mut x: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 1);
            if x == 0 || x >= 168 {
                continue;
            }
            if x < 8 {
                x = 0;
            } else {
                x = x - 8;
            }
            let tile_number: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 2);

            let mut tile_location: u16 = consts::SPRITE_PATTERN_TABLE_ADDR_START +
                (tile_number as u16 * consts::TILE_SIZE_BYTES as u16);

            let flags: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 3);
            let above_bg: bool = (flags >> 7) & 0b1 == 0b0;
            let y_flip: bool = (flags >> 6) & 0b1 == 0b1;
            let x_flip: bool = (flags >> 5) & 0b1 == 0b1;
            let obp0: bool = (flags >> 4) & 0b1 == 0b0;

            let tile_line: u8 = curr_line - y;
            if sprite_8_16 {
                if tile_line < 8 {
                    tile_location = consts::SPRITE_PATTERN_TABLE_ADDR_START +
                        ((tile_number as u16 & 0xFE) * consts::TILE_SIZE_BYTES as u16);
                } else {
                    tile_location = consts::SPRITE_PATTERN_TABLE_ADDR_START +
                        (tile_number as u16 * consts::TILE_SIZE_BYTES as u16);
                }
            }

            let startx: u8 = 0;
            for tile_col in startx..8 {
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

                let mut buffer_pos: usize;

                if y_flip {
                    buffer_pos = (y + height - 1 - tile_line) as usize * consts::DISPLAY_WIDTH_PX as usize;
                } else {
                    buffer_pos = curr_line as usize * consts::DISPLAY_WIDTH_PX as usize;
                }

                if x_flip {
                    buffer_pos += (x + 7 - tile_col) as usize;
                } else {
                    buffer_pos += (x + tile_col) as usize;
                }

                if above_bg || self.bg_wn_pixel_indexes[buffer_pos] == 0 {
                    buffer_pos *= 4;
                    if buffer_pos > self.screen_buffer.len() - 4 {
                        continue;
                    }
                    self.screen_buffer[buffer_pos] = r;
                    self.screen_buffer[buffer_pos+1] = g;
                    self.screen_buffer[buffer_pos+2] = b;
                    self.screen_buffer[buffer_pos+3] = a;
                }
            }
        }
    }

    pub fn toggle_bg(&mut self) {
        self.bg_on = !self.bg_on;
        println!("bg: {}", self.bg_on);
    }
    pub fn toggle_wn(&mut self) {
        self.wn_on = !self.wn_on;
        println!("wn: {}", self.wn_on);
    }
    pub fn toggle_sprites(&mut self) {
        self.sprites_on = !self.sprites_on;
        println!("sprites: {}", self.sprites_on);
    }
}
