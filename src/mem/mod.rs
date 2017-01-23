mod mapper;
pub mod cartridge;

use mem::mapper::Mapper;
use super::cpu::ioregister::{VBK_REGISTER_ADDR, SVBK_REGISTER_ADDR, BGPI_REGISTER_ADDR, OBPI_REGISTER_ADDR, BGPD_REGISTER_ADDR, OBPD_REGISTER_ADDR};

const VRAM_BANK_SIZE: usize = 0x2000;
const VRAM_BANKS: usize = 2;

const WRAM_BANK_SIZE: usize = 0x1000;
const WRAM_BANKS: usize = 8; // 0-7

const OAM_SIZE: usize = 0xA0;
const IO_SIZE: usize = 0x80;
const HRAM_SIZE: usize = 0x7F;
const PALETTE_SIZE: usize = 2 * 4 * 8; // 2 bytes for each of the 4 colors for each of the 8 palettes.
const BOOT_SIZE: usize = 0x900;

pub struct Memory {
    bootstrap_rom: [u8; BOOT_SIZE],
    vram: [u8; VRAM_BANKS * VRAM_BANK_SIZE], // two vram banks.
    wram: [u8; WRAM_BANKS * WRAM_BANK_SIZE], // eight wram banks
    oam: [u8; OAM_SIZE],
    io_registers: [u8; IO_SIZE],
    hram: [u8; HRAM_SIZE],
    interrupts_enable: u8,
    cartridge: Box<Mapper>,
    bootstrap_enabled: bool,
    can_access_vram: bool,
    can_access_oam: bool,
    // color mode only
    bg_palette_data: [u8; PALETTE_SIZE],
    sprite_palette_data: [u8; PALETTE_SIZE],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            bootstrap_rom: [0; BOOT_SIZE],
            vram: [0; VRAM_BANKS * VRAM_BANK_SIZE],
            wram: [0; WRAM_BANKS * WRAM_BANK_SIZE],
            oam: [0; OAM_SIZE],
            io_registers: [0; IO_SIZE],
            hram: [0; HRAM_SIZE],
            interrupts_enable: 0x0,
            cartridge: Box::new(mapper::NullMapper),
            bootstrap_enabled: true,
            can_access_vram: true,
            can_access_oam: true,
            // all colors are set to white
            bg_palette_data: [255; PALETTE_SIZE],
            sprite_palette_data: [255; PALETTE_SIZE],
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

    fn vbk(&self) -> u8 {
        self.read_byte(VBK_REGISTER_ADDR) & 0b1
    }
    fn svbk(&self) -> u8 {
        self.read_byte(SVBK_REGISTER_ADDR) & 0b111
    }
    fn is_color(&self) -> bool {
        let tmp = self.cartridge.read_rom(0x143);
        tmp == 0x80 || tmp == 0xC0
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x7FFF => self.cartridge.write_rom(address, value),
            0x8000...0x9FFF => {
                if self.can_access_vram {
                    let addr = (address - 0x8000) as usize + (VRAM_BANK_SIZE * self.vbk() as usize);
                    self.vram[addr] = value;
                }
            }
            0xA000...0xBFFF => self.cartridge.write_ram(address, value),
            0xC000...0xCFFF => {
                // always wram bank 0.
                self.wram[address as usize - 0xC000] = value;
            }
            0xD000...0xDFFF => {
                let addr = (address - 0xC000) as usize + (WRAM_BANK_SIZE * self.svbk() as usize);
                self.wram[addr] = value;
            }
            0xE000...0xEFFF => {
                // always wram bank 0.
                self.wram[address as usize - 0xE000] = value;
            }
            0xF000...0xFDFF => {
                let addr = (address - 0xE000) as usize + (WRAM_BANK_SIZE * self.svbk() as usize);
                self.wram[addr] = value;
            }
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
            0x0000...0x00FF if self.bootstrap_enabled && !self.is_color() => self.bootstrap_rom[address as usize],
            0x0000...0x0900 if self.bootstrap_enabled && self.is_color() => self.bootstrap_rom[address as usize],
            0x0000...0x7FFF => self.cartridge.read_rom(address),
            0x8000...0x9FFF => {
                if self.can_access_vram {
                    let addr = (address - 0x8000) as usize + (VRAM_BANK_SIZE * self.vbk() as usize);
                    self.vram[addr]
                } else {
                    0xFF
                }
            }
            0xA000...0xBFFF => self.cartridge.read_ram(address),
            0xC000...0xCFFF => self.wram[address as usize - 0xC000],
            0xD000...0xDFFF => {
                let addr = (address - 0xC000) as usize + (WRAM_BANK_SIZE * self.svbk() as usize);
                self.wram[addr]
            }
            0xE000...0xEFFF => self.wram[address as usize - 0xE000],
            0xF000...0xFDFF => {
                let addr = (address - 0xE000) as usize + (WRAM_BANK_SIZE * self.svbk() as usize);
                self.wram[addr]
            }
            0xFE00...0xFE9F => {
                if self.can_access_oam {
                    self.oam[(address - 0xFE00) as usize]
                } else {
                    0xFF
                }
            }
            0xFEA0...0xFEFF => 0x0,
            0xFF00...0xFF7F => {
                match address {
                    BGPD_REGISTER_ADDR => {
                        if self.is_color() {
                            // read BGPI ioregister
                            let bgpi = self.read_byte(BGPI_REGISTER_ADDR);
                            let palette_addr = bgpi & 0b0011_1111;
                            self.read_bg_palette(palette_addr)
                        }
                        else {
                            self.io_registers[(address - 0xFF00) as usize]
                        }
                    }
                    OBPD_REGISTER_ADDR => {
                        if self.is_color() {
                            // read OBPI ioregister
                            let obpi = self.read_byte(OBPI_REGISTER_ADDR);
                            let palette_addr = obpi & 0b0011_1111;
                            self.read_sprite_palette(palette_addr)
                        }
                        else {
                            self.io_registers[(address - 0xFF00) as usize]
                        }
                    }
                    _ => {
                        self.io_registers[(address - 0xFF00) as usize]
                    }
                }
            }
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
        self.vram = [0; VRAM_BANKS * VRAM_BANK_SIZE];
        self.wram = [0; WRAM_BANKS * WRAM_BANK_SIZE];
        self.oam = [0; OAM_SIZE];
        self.io_registers = [0; IO_SIZE];
        self.hram = [0; HRAM_SIZE];
        self.interrupts_enable = 0x0;
        self.bootstrap_enabled = true;
        self.can_access_vram = true;
        self.bg_palette_data = [255; PALETTE_SIZE];
        self.sprite_palette_data = [255; PALETTE_SIZE];
    }

    pub fn disable_bootstrap(&mut self) {
        self.bootstrap_enabled = false;
        /*self.write_byte(0xFF05, 0x00);
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
        */
    }

    pub fn load_bootstrap_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.bootstrap_rom[i] = *byte;
        }
    }

    pub fn read_cartridge(&self, addr: u16) -> u8 {
        self.cartridge.read_rom(addr)
    }

    pub fn load_cartridge(&mut self, rom: &[u8], battery: &[u8]) {
        self.cartridge = cartridge::load_cartridge(rom, battery);

        for i in 0x100..0x200 {
            self.bootstrap_rom[i] = self.cartridge.read_rom(i as u16);
        }
    }

    pub fn save_battery(&mut self) -> Vec<u8> {
        self.cartridge.save_battery()
    }
}
