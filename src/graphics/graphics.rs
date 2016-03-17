use graphics::consts;
use super::super::util::util;
use super::super::mem::mem::Memory;
use super::super::cpu::ioregister;
use super::super::cpu;

pub struct Graphics {
    // FIXME: find the correct size to bg_wn_pixel_indexes
    bg_wn_pixel_indexes: [u8; 160 * 144 * 4],
    pub screen_buffer: [u8; 160 * 144 * 4],
    bg_on: bool,
    wn_on: bool,
    sprites_on: bool,
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics {
            screen_buffer: [255; 160 * 144 * 4],
            bg_wn_pixel_indexes: [0; 160 * 144 * 4],
            bg_on: true,
            wn_on: true,
            sprites_on: true,
        }
    }

    pub fn restart(&mut self) {
        self.screen_buffer = [255; 160 * 144 * 4];
        self.bg_wn_pixel_indexes = [0; 160 * 144 * 4];
        self.bg_on = true;
        self.wn_on = true;
        self.sprites_on = true;
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

        bg_on = bg_on & self.bg_on;
        wn_on = wn_on & self.wn_on;

        if !bg_on && !wn_on {
            return;
        }

        let curr_line: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
        if curr_line >= consts::DISPLAY_HEIGHT_PX {
            return;
        }
        let scx: u8 = memory.read_byte(cpu::consts::SCX_REGISTER_ADDR);
        let scy: u8 = memory.read_byte(cpu::consts::SCY_REGISTER_ADDR);
        let mut ypos: u16 = curr_line.wrapping_add(scy) as u16;
        let wy: u8 = memory.read_byte(cpu::consts::WY_REGISTER_ADDR);
        let wx: u8 = memory.read_byte(cpu::consts::WX_REGISTER_ADDR).wrapping_sub(7);

        let mut is_window: bool = false;

        let startx: u8 = if bg_on {
            0
        } else {
            wx
        };

        let (tile_table_addr_pattern_0, is_tile_number_signed) =
            if ioregister::LCDCRegister::is_tile_data_0(&memory) {
                (consts::TILE_DATA_TABLE_0_ADDR_START, true)
            } else {
                (consts::TILE_DATA_TABLE_1_ADDR_START, false)
            };

        let mut tile_row: u16 = (ypos / 8) * 32;
        let mut tile_line: u16 = (ypos % 8) * 2;
        for i in startx..consts::DISPLAY_WIDTH_PX {
            if wn_on && !is_window && i >= wx && wx < consts::DISPLAY_WIDTH_PX && curr_line >= wy {

                is_window = true;
                ypos = (curr_line - wy) as u16;
                tile_row = (ypos / 8) * 32;
                tile_line = (ypos % 8) * 2;
            }

            let xpos: u16 = if is_window {
                i.wrapping_sub(wx) as u16
            } else {
                scx.wrapping_add(i) as u16
            };

            let buffer_pos: usize = (curr_line as usize * consts::DISPLAY_WIDTH_PX as usize) +
                                    (i as usize);

            if !bg_on && !is_window {
                self.bg_wn_pixel_indexes[buffer_pos] = 0;
                continue;
            }
            let addr_start = if is_window {
                if ioregister::LCDCRegister::is_window_tile_map_display_normal(&memory) {
                    consts::BG_NORMAL_ADDR_START
                } else {
                    consts::BG_WINDOW_ADDR_START
                }
            } else {
                if ioregister::LCDCRegister::is_bg_tile_map_display_normal(&memory) {
                    consts::BG_NORMAL_ADDR_START
                } else {
                    consts::BG_WINDOW_ADDR_START
                }
            };

            let tile_col_bg: u16 = xpos >> 3;
            let tile_addr: u16 = addr_start + tile_row + tile_col_bg;
            let tile_location: u16 = if is_tile_number_signed {
                let mut tile_number: u16 = util::sign_extend(memory.read_byte(tile_addr));
                if util::is_neg16(tile_number) {
                    tile_number = 128 - util::twos_complement(tile_number);
                } else {
                    tile_number += 128;
                }
                tile_table_addr_pattern_0 + (tile_number * consts::TILE_SIZE_BYTES as u16)
            } else {
                tile_table_addr_pattern_0 +
                (memory.read_byte(tile_addr) as u16 * consts::TILE_SIZE_BYTES as u16)
            };
            let tile_col: u16 = xpos % 8;
            // two bytes representing 8 pixel indexes
            let lhs: u8 = memory.read_byte(tile_location + tile_line) >> (7 - tile_col);
            let rhs: u8 = memory.read_byte(tile_location + tile_line + 1) >> (7 - tile_col);
            let pixel_data: u8 = ((rhs << 1) & 0b10) | (lhs & 0b01);
            let pixel_index: u8 = ioregister::bg_window_palette(pixel_data, memory);

            // Apply palette
            let (r, g, b) = match pixel_index {
                0b00 => (255, 255, 255),
                0b01 => (192, 192, 192),
                0b10 => (96, 96, 96),
                0b11 => (0, 0, 0),
                _ => unreachable!(),
            };

            self.bg_wn_pixel_indexes[buffer_pos] = pixel_data;

            let buffer_pos: usize = buffer_pos * 4; //*4 because of RGBA

            self.screen_buffer[buffer_pos] = r;
            self.screen_buffer[buffer_pos + 1] = g;
            self.screen_buffer[buffer_pos + 2] = b;
            self.screen_buffer[buffer_pos + 3] = 255; //alpha
        }
    }

    fn draw_sprites(&mut self, memory: &Memory) {
        // TODO draw sprites based on X priority.
        if !ioregister::LCDCRegister::is_sprite_display_on(memory) || !self.sprites_on {
            return;
        }

        let curr_line: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
        if curr_line >= consts::DISPLAY_HEIGHT_PX || !self.sprites_on {
            return;
        }
        let mut index: u16 = 160; //40*4: 40 sprites that use 4 bytes
        while index != 0 {
            index -= 4;
            let sprite_8_16: bool = ioregister::LCDCRegister::is_sprite_8_16_on(memory);
            let height: u8 = if sprite_8_16 {
                16
            } else {
                8
            };
            let mut y: i16 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index) as i16;
            if y == 0 || y >= 160 {
                continue;
            }
            y -= 16;
            if ((curr_line as i16) < y) || (curr_line as i16 >= y + height as i16) {
                // outside sprite
                continue;
            }

            let mut x: i16 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 1) as i16;
            if x == 0 || x >= 168 {
                continue;
            }

            let tile_number: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 2);

            let tile_location: u16 = consts::SPRITE_PATTERN_TABLE_ADDR_START +
                                     (tile_number as u16 * consts::TILE_SIZE_BYTES as u16);

            let flags: u8 = memory.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + index + 3);
            let above_bg: bool = (flags >> 7) & 0b1 == 0b0;
            let y_flip: bool = (flags >> 6) & 0b1 == 0b1;
            let x_flip: bool = (flags >> 5) & 0b1 == 0b1;
            let obp0: bool = (flags >> 4) & 0b1 == 0b0;

            x -= 8;
            let endx: u8 = if x + 8 >= consts::DISPLAY_WIDTH_PX as i16 {
                consts::DISPLAY_WIDTH_PX.wrapping_sub(x as u8)
            } else {
                8
            };
            let tile_line: u8 = (curr_line as i16 - y) as u8;
            for tile_col in 0..endx {
                // tile_line*2 because each tile uses 2 bytes per line.
                let lhs: u8 = memory.read_byte(tile_location + (tile_line as u16 * 2)) >>
                              (7 - tile_col);
                let rhs: u8 = memory.read_byte(tile_location + (tile_line as u16 * 2) + 1) >>
                              (7 - tile_col);
                let pixel_data: u8 = ((rhs << 1) & 0b10) | (lhs & 0b01);
                if pixel_data == 0 {
                    continue;
                }
                let pixel_index: u8 = ioregister::sprite_palette(obp0, pixel_data, memory);
                let (r, g, b, a) = match pixel_index {
                    0b00 => (255, 255, 255, 255),
                    0b01 => (192, 192, 192, 255),
                    0b10 => (96, 96, 96, 255),
                    0b11 => (0, 0, 0, 255),
                    _ => unreachable!(),
                };

                let mut buffer_pos: usize;

                if y_flip {
                    buffer_pos =
                        (y.wrapping_add(height as i16 - 1 - tile_line as i16) as u16) as usize *
                        consts::DISPLAY_WIDTH_PX as usize;
                } else {
                    // y + tile_line = curr_line
                    buffer_pos = curr_line as usize * consts::DISPLAY_WIDTH_PX as usize;
                }

                let old_pos: usize = buffer_pos;
                if x_flip {
                    buffer_pos += (x.wrapping_add(7 - tile_col as i16) as u16) as usize;
                } else {
                    buffer_pos += (x.wrapping_add(tile_col as i16) as u16) as usize;
                }

                if buffer_pos < old_pos {
                    continue;
                }

                if above_bg || self.bg_wn_pixel_indexes[buffer_pos] == 0 {
                    buffer_pos *= 4;
                    if buffer_pos > self.screen_buffer.len() - 4 {
                        continue;
                    }
                    self.screen_buffer[buffer_pos] = r;
                    self.screen_buffer[buffer_pos + 1] = g;
                    self.screen_buffer[buffer_pos + 2] = b;
                    self.screen_buffer[buffer_pos + 3] = a;
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
