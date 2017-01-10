mod mapper;
pub mod cartridge;

use mem::mapper::Mapper;

pub struct Memory {
    bootstrap_rom: [u8; 0x2000],
    vram: [u8; 0x2000 * 2], // two vram banks.
    wram: [u8; 0x2000],
    oam: [u8; 0xA0],
    io_registers: [u8; 0x80],
    hram: [u8; 0x7F],
    interrupts_enable: u8,
    cartridge: Box<Mapper>,
    bootstrap_enabled: bool,
    can_access_vram: bool,
    can_access_oam: bool,
    // color mode only
    bg_palette_data: [u8; 2 * 4 * 8], // 2 bytes for each of the 4 colors for each of the 8 palettes.
    sprite_palette_data: [u8; 2 * 4 * 8], // 2 bytes for each of the 4 colors for each of the 8 palettes.
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            bootstrap_rom: [0; 0x2000],
            vram: [0; 0x2000 * 2],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            interrupts_enable: 0x0,
            cartridge: Box::new(mapper::NullMapper),
            bootstrap_enabled: true,
            can_access_vram: true,
            can_access_oam: true,
            // all colors are set to white
            bg_palette_data: [255; 2 * 4 * 8],
            sprite_palette_data: [255; 2 * 4 * 8],
        }
    }
}

impl Memory {
    // returns a string with the memory data from min_addr to max_addr.
    pub fn format(&self, min_addr: Option<u16>, max_addr: Option<u16>) -> String {
        let columns = 16;

        let mut res = "".to_owned();

        let mut to = 0xffff;
        let mut from = 0;

        if let Some(fr) = min_addr {
            from = fr as usize;
        }
        if let Some(t) = max_addr {
            to = t as usize;
        }

        let mut i = from;
        while i >= from && i < to {
            if i as u8 % columns == 0 {
                res = res + &format!("\n{:01$x}: ", i, 8);
            }
            let byte = self.read_byte(i as u16);
            res = res + &format!("{:01$x} ", byte, 2);

            i += 1;
        }
        res
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x7FFF => self.cartridge.write_rom(address, value),
            0x8000...0x9FFF => {
                if self.can_access_vram {
                    // TODO: remove hardcoded stuff VBK io register
                    let vbk = self.read_byte(0xFF4F) & 0b1;
                    let addr = (address - 0x8000) as usize + (0x2000 * vbk as usize);
                    self.vram[addr] = value;
                }
            }
            0xA000...0xBFFF => self.cartridge.write_ram(address, value),
            0xC000...0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000...0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            0xFE00...0xFE9F => {
                if self.can_access_oam {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
            }
            0xFEA0...0xFEFF => (),// panic!("writing to unusable ram."),
            0xFF00...0xFF7F => self.io_registers[(address - 0xFF00) as usize] = value,
            0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.interrupts_enable = value,
            _ => panic!("Out of bound! Tried to write to {:#x}.", address),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000...0x00FF if self.bootstrap_enabled => self.bootstrap_rom[address as usize],
            0x0000...0x7FFF => self.cartridge.read_rom(address),
            0x8000...0x9FFF => {
                if self.can_access_vram {
                    // TODO: remove hardcoded stuff VBK io register
                    let vbk = self.read_byte(0xFF4F) & 0b1;
                    let addr = (address - 0x8000) as usize + (0x2000 * vbk as usize);
                    self.vram[addr]
                } else {
                    0xFF
                }
            }
            0xA000...0xBFFF => self.cartridge.read_ram(address),
            0xC000...0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000...0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00...0xFE9F => {
                if self.can_access_oam {
                    self.oam[(address - 0xFE00) as usize]
                } else {
                    0xFF
                }
            }
            0xFEA0...0xFEFF => 0x0,
            0xFF00...0xFF7F => self.io_registers[(address - 0xFF00) as usize],
            0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupts_enable,
            _ => panic!("Out of bound! Tried to read from {:#x}.", address),
        }
    }

    pub fn write_bg_palette(&mut self, addr: u8, value: u8) {
        self.bg_palette_data[addr as usize] = value;
    }
    pub fn read_bg_palette(&self, addr: u8) -> u8 {
        self.bg_palette_data[addr as usize]
    }
    pub fn write_sprite_palette(&mut self, addr: u8, value: u8) {
        self.sprite_palette_data[addr as usize] = value;
    }
    pub fn read_sprite_palette(&self, addr: u8) -> u8 {
        self.sprite_palette_data[addr as usize]
    }

    pub fn set_access_vram(&mut self, can_access: bool) {
        self.can_access_vram = can_access;
    }
    pub fn set_access_oam(&mut self, can_access: bool) {
        self.can_access_oam = can_access;
    }

    pub fn restart(&mut self) {
        self.vram = [0; 0x2000 * 2];
        self.wram = [0; 0x2000];
        self.oam = [0; 0xA0];
        self.io_registers = [0; 0x80];
        self.hram = [0; 0x7F];
        self.interrupts_enable = 0x0;
        self.bootstrap_enabled = true;
        self.can_access_vram = true;
    }

    pub fn disable_bootstrap(&mut self) {
        self.bootstrap_enabled = false;
        self.write_byte(0xFF05, 0x00);
        self.write_byte(0xFF06, 0x00);
        self.write_byte(0xFF07, 0x00);
        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF14, 0xBF);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0x00);
        self.write_byte(0xFF19, 0xBF);
        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1E, 0xBF);
        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0x00);
        self.write_byte(0xFF22, 0x00);
        self.write_byte(0xFF23, 0xBF);
        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
        self.write_byte(0xFF40, 0x91);
        self.write_byte(0xFF42, 0x00);
        self.write_byte(0xFF43, 0x00);
        self.write_byte(0xFF45, 0x00);
        self.write_byte(0xFF47, 0xFC);
        self.write_byte(0xFF48, 0xFF);
        self.write_byte(0xFF49, 0xFF);
        self.write_byte(0xFF4A, 0x00);
        self.write_byte(0xFF4B, 0x00);
        self.write_byte(0xFF4F, 0x00);
        self.write_byte(0xFFFF, 0x00);
    }

    pub fn load_bootstrap_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.bootstrap_rom[i] = *byte;
        }
    }

    pub fn load_cartridge(&mut self, rom: &[u8], battery: &[u8]) {
        self.cartridge = cartridge::load_cartridge(rom, battery);
    }

    pub fn save_battery(&mut self) -> Vec<u8> {
        self.cartridge.save_battery()
    }
}
