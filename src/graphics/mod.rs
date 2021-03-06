pub mod consts;

use super::mem::Memory;
use super::cpu::ioregister;
use super::gebemula::GBMode;

#[derive(Copy, Clone, PartialEq)]
enum TileType {
    Sprite,
    Background,
    Window,
}
// Tile attributes
#[derive(Copy, Clone)]
struct TileAttr(u8);
impl TileAttr {
    pub fn cgb_palette_number(&self) -> u8 {
        self.0 & 0b111
    }
    pub fn tile_vram_bank(&self) -> u8 {
        (self.0 >> 3) & 0b1
    }
    pub fn dmg_palette_number(&self) -> u8 {
        (self.0 >> 4) & 0b1
    }
    pub fn h_flip(&self) -> bool {
        ((self.0 >> 5) & 0b1) == 0b1
    }
    pub fn v_flip(&self) -> bool {
        ((self.0 >> 6) & 0b1) == 0b1
    }
    pub fn priority(&self) -> TileType {
        match (self.0 >> 7) & 0b1 {
            0 => TileType::Sprite,
            1 => TileType::Background,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone)]
struct TilePixel {
    // color number in the palette for the pixel (0-3).
    pub color_number: u8,
    pub tile_attr: TileAttr,
    pub tile_type: TileType,
}
impl TilePixel {
    pub fn new(color_number: u8, tile_attr: TileAttr, tile_type: TileType) -> Self {
        TilePixel {
            color_number: color_number,
            tile_attr: tile_attr,
            tile_type: tile_type,
        }
    }
}
impl Default for TilePixel {
    fn default() -> Self {
        TilePixel {
            color_number: 0,
            tile_attr: TileAttr(0),
            tile_type: TileType::Background,
        }
    }
}

struct Sprite(u16);

impl Sprite {
    fn y(&self, mem: &Memory) -> u8 {
        mem.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + self.0)
    }
    fn x(&self, mem: &Memory) -> u8 {
        mem.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + self.0 + 1)
    }
    fn height(mem: &Memory) -> u8 {
        let sprite_8_16 = ioregister::LCDCRegister::is_sprite_8_16_on(mem);
        if sprite_8_16 {
            16
        } else {
            8
        }
    }

    fn is_not_visible(&self, current_line: i16, mem: &Memory) -> bool {
        let y = self.y(mem);
        let x = self.x(mem);
        let h = Self::height(mem) as i16;
        // outside screen?
        y == 0 || y >= 160 || x == 0 || x >= 168 ||
        // current line outside sprite?
        current_line < (y as i16 - 16) || current_line >= (y as i16 - 16) + h
    }

    fn tile_number(&self, mem: &Memory) -> u8 {
        mem.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + self.0 + 2)
    }
    fn attr(&self, mem: &Memory) -> TileAttr {
        TileAttr(mem.read_byte(consts::SPRITE_ATTRIBUTE_TABLE + self.0 + 3))
    }
}

trait RGB {
    fn rgb(&self, pixel: &TilePixel, memory: &Memory) -> (u8, u8, u8);
    fn mode(&self) -> GBMode;

    fn is_color(&self) -> bool {
        self.mode() == GBMode::Color
    }
    fn is_mono(&self) -> bool {
        self.mode() == GBMode::Mono
    }
}

struct MonoRGB;
impl RGB for MonoRGB {
    fn rgb(&self, pixel: &TilePixel, memory: &Memory) -> (u8, u8, u8) {
        //TODO make sure this write isn't necessary.
        //memory.write_byte(ioregister::VBK_REGISTER_ADDR, 0);
        let pixel_index = match pixel.tile_type {
            TileType::Background | TileType::Window => {
                ioregister::bg_window_palette(pixel.color_number, memory)
            }
            TileType::Sprite => ioregister::sprite_palette(
                pixel.tile_attr.dmg_palette_number() == 0,
                pixel.color_number,
                memory,
            ),
        };
        consts::DMG_PALETTE[pixel_index as usize]
    }

    fn mode(&self) -> GBMode {
        GBMode::Mono
    }
}

struct ColorRGB;
impl ColorRGB {
    fn palette_to_rgb(palette_h: u8, palette_l: u8) -> (u8, u8, u8) {
        let r = palette_l & 0b0001_1111;
        let g = ((palette_h & 0b11) << 3) | (palette_l >> 5);
        let b = (palette_h >> 2) & 0b11111;

        let to255 = |x| (x << 3) | (x >> 2);

        (to255(r), to255(g), to255(b))
    }
}
impl RGB for ColorRGB {
    fn rgb(&self, pixel: &TilePixel, memory: &Memory) -> (u8, u8, u8) {
        let h_addr = (pixel.tile_attr.cgb_palette_number() * 8) + 1 + (pixel.color_number * 2); // each palette uses 8 bytes.
        let l_addr = (pixel.tile_attr.cgb_palette_number() * 8) + (pixel.color_number * 2); // color_number chooses the palette index. *2 because each color intensity uses two bytes.
        let (palette_h, palette_l) = match pixel.tile_type {
            TileType::Background | TileType::Window => (
                memory.read_bg_palette(h_addr),
                memory.read_bg_palette(l_addr),
            ),
            TileType::Sprite => (
                memory.read_sprite_palette(h_addr),
                memory.read_sprite_palette(l_addr),
            ),
        };
        Self::palette_to_rgb(palette_h, palette_l)
    }
    fn mode(&self) -> GBMode {
        GBMode::Color
    }
}

pub struct Graphics {
    bg_wn_pixel_indexes: [TilePixel; 160 * 144],
    pub screen_buffer: [u8; 160 * 144 * 4],
    bg_on: bool,
    wn_on: bool,
    sprites_on: bool,
    rgb: Box<dyn RGB>,
}

impl Default for Graphics {
    fn default() -> Graphics {
        Graphics {
            screen_buffer: [255; 160 * 144 * 4],
            bg_wn_pixel_indexes: [TilePixel::default(); 160 * 144],
            bg_on: true,
            wn_on: true,
            sprites_on: true,
            rgb: Box::new(MonoRGB),
        }
    }
}

impl Graphics {
    pub fn restart(&mut self) {
        self.screen_buffer = [255; 160 * 144 * 4];
        self.bg_wn_pixel_indexes = [TilePixel::default(); 160 * 144];
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

    pub fn set_color(&mut self) {
        self.rgb = Box::new(ColorRGB)
    }

    fn update_line_buffer(&mut self, memory: &mut Memory) {
        // we can't draw below DISPLAY_HEIGHT_PX
        let curr_line = memory.read_byte(ioregister::LY_REGISTER_ADDR);
        if curr_line >= consts::DISPLAY_HEIGHT_PX {
            return;
        }

        let mut bg_on = ioregister::LCDCRegister::is_bg_window_display_on(memory);
        let mut wn_on = ioregister::LCDCRegister::is_window_display_on(memory);

        bg_on = if self.rgb.is_color() {
            self.bg_on
        } else {
            bg_on && self.bg_on
        };
        wn_on = wn_on && self.wn_on;

        if !bg_on && !wn_on {
            return;
        }

        let scx = memory.read_byte(ioregister::SCX_REGISTER_ADDR);
        let scy = memory.read_byte(ioregister::SCY_REGISTER_ADDR);
        let wy = memory.read_byte(ioregister::WY_REGISTER_ADDR);
        let wx = memory
            .read_byte(ioregister::WX_REGISTER_ADDR)
            .wrapping_sub(7);

        let old_vbk = memory.read_byte(ioregister::VBK_REGISTER_ADDR);

        let (tile_map_addr, is_tile_number_signed) =
            if ioregister::LCDCRegister::is_tile_data_0(&memory) {
                (consts::TILE_DATA_TABLE_0_ADDR_START, true)
            } else {
                (consts::TILE_DATA_TABLE_1_ADDR_START, false)
            };

        let mut is_window = false;

        let mut ypos = curr_line.wrapping_add(scy);
        let mut tile_y = ypos >> 3;
        let mut tile_line = ypos & 0b111;

        let startx = if bg_on { 0 } else { wx };
        for i in startx..consts::DISPLAY_WIDTH_PX {
            if wn_on && !is_window && i >= wx && wx < consts::DISPLAY_WIDTH_PX && curr_line >= wy {
                is_window = true;
                ypos = curr_line - wy;

                tile_y = ypos >> 3;
                tile_line = ypos & 0b111;
            }

            let xpos = if is_window {
                i.wrapping_sub(wx)
            } else {
                scx.wrapping_add(i)
            };

            let buffer_pos =
                (curr_line as usize * consts::DISPLAY_WIDTH_PX as usize) + (i as usize);

            if !bg_on && !is_window {
                self.bg_wn_pixel_indexes[buffer_pos] = TilePixel::default();
                continue;
            }

            let addr_start = if is_window {
                if ioregister::LCDCRegister::is_window_tile_map_display_normal(&memory) {
                    consts::BG_NORMAL_ADDR_START
                } else {
                    consts::BG_WINDOW_ADDR_START
                }
            } else if ioregister::LCDCRegister::is_bg_tile_map_display_normal(&memory) {
                consts::BG_NORMAL_ADDR_START
            } else {
                consts::BG_WINDOW_ADDR_START
            };

            let tile_addr = addr_start + (tile_y as u16 * 32) + (xpos as u16 >> 3);
            // tile map is on vram bank 0
            memory.write_byte(ioregister::VBK_REGISTER_ADDR, 0);
            let tile_number = memory.read_byte(tile_addr);
            let tile_location = if is_tile_number_signed {
                (tile_map_addr as i32
                    + ((tile_number as i8 as i32 + 128) * consts::TILE_SIZE_BYTES as i32))
                    as u16
            } else {
                tile_map_addr + (tile_number as u16 * consts::TILE_SIZE_BYTES as u16)
            };

            let mut tile_col = xpos & 0b111;
            let mut attr = None;
            if self.rgb.is_color() {
                // tile attribute is on vram bank 1
                memory.write_byte(ioregister::VBK_REGISTER_ADDR, 1);
                attr = Some(TileAttr(memory.read_byte(tile_addr)));

                if attr.unwrap().h_flip() {
                    tile_col = 7 - tile_col;
                }
                // set vbk to use the correct bank for the tile data.
                memory.write_byte(
                    ioregister::VBK_REGISTER_ADDR,
                    attr.unwrap().tile_vram_bank(),
                );
            }

            let normal = || {
                (
                    memory.read_byte(tile_location + (tile_line as u16 * 2)) >> (7 - tile_col),
                    memory.read_byte(tile_location + (tile_line as u16 * 2) + 1) >> (7 - tile_col),
                )
            };
            let vflip = || {
                (
                    memory.read_byte((tile_location + 15) - (tile_line as u16 * 2) - 1)
                        >> (7 - tile_col),
                    memory.read_byte((tile_location + 15) - (tile_line as u16 * 2))
                        >> (7 - tile_col),
                )
            };
            // two bytes representing 8 pixel indexes
            let (rhs, lhs) = match attr {
                Some(a) => {
                    if a.v_flip() {
                        vflip()
                    } else {
                        normal()
                    }
                }
                None => normal(),
            };
            let color_number = ((lhs << 1) & 0b10) | (rhs & 0b01);
            let tile_type = if is_window {
                TileType::Window
            } else {
                TileType::Background
            };
            self.bg_wn_pixel_indexes[buffer_pos] =
                TilePixel::new(color_number, attr.unwrap_or(TileAttr(0)), tile_type);

            let (r, g, b) = self.rgb.rgb(&self.bg_wn_pixel_indexes[buffer_pos], memory);

            let buffer_pos = buffer_pos * 4; //*4 because of RGBA
            self.screen_buffer[buffer_pos] = r;
            self.screen_buffer[buffer_pos + 1] = g;
            self.screen_buffer[buffer_pos + 2] = b;
            self.screen_buffer[buffer_pos + 3] = 255; //alpha
        }
        memory.write_byte(ioregister::VBK_REGISTER_ADDR, old_vbk);
    }

    fn draw_sprites(&mut self, memory: &mut Memory) {
        // TODO draw sprites based on X priority. (only for Non-CGB)
        if !ioregister::LCDCRegister::is_sprite_display_on(memory) || !self.sprites_on {
            return;
        }

        let curr_line = memory.read_byte(ioregister::LY_REGISTER_ADDR);
        if curr_line >= consts::DISPLAY_HEIGHT_PX || !self.sprites_on {
            return;
        }

        let old_vbk = memory.read_byte(ioregister::VBK_REGISTER_ADDR);

        let mut index = 160; //40*4: 40 sprites that use 4 bytes
        while index != 0 {
            index -= 4;
            let sprite = Sprite(index as u16);
            if sprite.is_not_visible(curr_line as i16, memory) {
                continue;
            }
            let tile_location = consts::SPRITE_PATTERN_TABLE_ADDR_START
                + (sprite.tile_number(memory) as u16 * consts::TILE_SIZE_BYTES as u16);

            let attr = sprite.attr(memory);
            if self.rgb.is_color() {
                memory.write_byte(ioregister::VBK_REGISTER_ADDR, attr.tile_vram_bank());
            }

            let x = sprite.x(memory);
            let endx = if x >= consts::DISPLAY_WIDTH_PX as u8 {
                consts::DISPLAY_WIDTH_PX.wrapping_sub(x - 8)
            } else {
                8
            };

            let y = sprite.y(memory);
            let tile_line = (curr_line as i16 - (y as i16 - 16)) as u8;
            for tile_col in 0..endx {
                let mut buffer_pos = (curr_line as usize * consts::DISPLAY_WIDTH_PX as usize)
                    + ((x.wrapping_add(tile_col) as u16).wrapping_sub(8)) as usize;

                if buffer_pos * 4 > self.screen_buffer.len() - 4 {
                    continue;
                }
                let bg_px = self.bg_wn_pixel_indexes[buffer_pos];

                let mut tile_col = tile_col;
                if attr.h_flip() {
                    tile_col = 7 - tile_col;
                }
                // tile_line*2 because each tile uses 2 bytes per line.
                let normal = || {
                    (
                        memory.read_byte(tile_location + (tile_line as u16 * 2)) >> (7 - tile_col),
                        memory.read_byte(tile_location + (tile_line as u16 * 2) + 1)
                            >> (7 - tile_col),
                    )
                };
                let vflip = || {
                    (
                        memory.read_byte(
                            (tile_location + ((Sprite::height(memory) as u16 * 2) - 1))
                                - (tile_line as u16 * 2) - 1,
                        ) >> (7 - tile_col),
                        memory.read_byte(
                            (tile_location + ((Sprite::height(memory) as u16 * 2) - 1))
                                - (tile_line as u16 * 2),
                        ) >> (7 - tile_col),
                    )
                };
                let (lhs, rhs) = if attr.v_flip() { vflip() } else { normal() };
                let color_number = ((rhs << 1) & 0b10) | (lhs & 0b01);
                if color_number == 0 {
                    continue;
                }

                let sprites_on_top = !ioregister::LCDCRegister::is_bg_window_display_on(memory);
                let oam_priority = bg_px.tile_attr.priority() == TileType::Sprite;
                let sprite_priority = attr.priority() == TileType::Sprite;

                let should_draw = if self.rgb.is_color() {
                    sprites_on_top || (oam_priority && (sprite_priority || bg_px.color_number == 0))
                } else {
                    sprite_priority || bg_px.color_number == 0
                };

                if should_draw {
                    let (r, g, b) = self.rgb.rgb(
                        &TilePixel::new(color_number, attr, TileType::Sprite),
                        memory,
                    );

                    buffer_pos *= 4; // because of RGBA
                    self.screen_buffer[buffer_pos] = r;
                    self.screen_buffer[buffer_pos + 1] = g;
                    self.screen_buffer[buffer_pos + 2] = b;
                    self.screen_buffer[buffer_pos + 3] = 255; //alpha
                }
            }
        }
        memory.write_byte(ioregister::VBK_REGISTER_ADDR, old_vbk);
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
