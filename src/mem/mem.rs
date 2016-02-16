use std::fmt;
use util::util;
use mem::consts;

#[derive(Copy, Clone, PartialEq, Debug)]
enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

pub struct Memory {
    rom_bank_00: [u8; 0x4000],
    rom_bank_01_nn: [u8; 0x4000],
    vram: [u8; 0x2000],
    external_ram: [u8; 0x7A1200], // TODO: dinamically allocate size?
    wram_bank_0: [u8; 0x1000],
    wram_bank_1_n: [u8; 0x1000],
    wram_echo: [u8; 0x1E00], 
    oam: [u8; 0xA0],
    unusable: [u8; 0x60],
    io_registers: [u8; 0x80],
    hram: [u8; 0x7F],
    interrupts_enable: u8,
    cartridge: Vec<u8>,
    cartridge_type: CartridgeType,
    current_rom_bank: u16,
    current_ram_bank: u16,
    rom_banking_enabled: bool,
    ram_banking_enabled: bool,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            rom_bank_00: [0; 0x4000],
            rom_bank_01_nn: [0; 0x4000],
            vram: [0; 0x2000],
            external_ram: [0; 0x7A1200],
            wram_bank_0: [0; 0x1000],
            wram_bank_1_n: [0; 0x1000],
            wram_echo: [0; 0x1E00], 
            oam: [0; 0xA0],
            unusable: [0; 0x60],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            interrupts_enable: 0x0,
            cartridge: vec![0; 0x200000],
            cartridge_type: CartridgeType::RomOnly,
            current_rom_bank: 0x1,
            current_ram_bank: 0x0,
            rom_banking_enabled: true,
            ram_banking_enabled: false,
        }
    }

    //returns a string with the memory data from min_addr to max_addr.
    pub fn format(&self, min_addr: Option<u16>, max_addr: Option<u16>) -> String {
        let columns: u8 = 16;

        let mut res: String = "".to_owned();

        let mut to: usize = 0xffff;
        let mut from: usize = 0;

        if let Some(fr) = min_addr {
            from = fr as usize;
        }
        if let Some(t) = max_addr {
            to = t as usize;
        }

        let mut i: usize = from;
        while i >= from && i < to {
            if i as u8 % columns == 0 {
                res = res + &format!("\n{:01$x}: ", i, 8);
            }
            let lhs: u8 = self.read_byte(i as u16);
            i += 1;
            let rhs: u8 = self.read_byte(i as u16);
            res = res + &format!("{:01$x}", lhs, 2);
            res = res + &format!("{:01$x} ", rhs, 2);

            i += 1;
        }
        format!("{}", res)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ... 0x7FFF => self.handle_banking(address, value),
            0x8000 ... 0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xA000 ... 0xBFFF => {
                if self.ram_banking_enabled {
                    self.external_ram[(address as u32 - 0xA000 as u32 + self.current_ram_bank as u32 * consts::RAM_BANK_SIZE as u32) as usize] = value;
                }
            },
            0xC000 ... 0xCFFF => {
                self.wram_bank_0[(address - 0xC000) as usize] = value;
                if address <= 0xCDFF {
                    self.wram_echo[(address - 0xC000) as usize] = value;
                }
            },
            0xD000 ... 0xDFFF => self.wram_bank_1_n[(address - 0xD000) as usize] = value,
            0xE000 ... 0xFDFF => {
                self.wram_echo[(address - 0xE000) as usize] = value;
                self.wram_bank_0[(address - 0xE000) as usize] = value;
            },
            0xFE00 ... 0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFEA0 ... 0xFEFF => self.unusable[(address - 0xFEA0) as usize] = value,
            0xFF00 ... 0xFF7F => self.io_registers[(address - 0xFF00) as usize] = value,
            0xFF80 ... 0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.interrupts_enable = value,
            _ => panic!("Out of bound! Tried to write to {:#X}.", address)
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000 ... 0x3FFF => self.rom_bank_00[address as usize],
            0x4000 ... 0x7FFF => self.cartridge[(address as u32 - 0x4000 as u32 + self.current_rom_bank as u32 * consts::ROM_BANK_SIZE as u32) as usize],
            0x8000 ... 0x9FFF => self.vram[(address - 0x8000) as usize],
            0xA000 ... 0xBFFF => self.external_ram[(address as u32 - 0xA000 as u32 + self.current_ram_bank as u32 * consts::RAM_BANK_SIZE as u32) as usize],
            0xC000 ... 0xCFFF => self.wram_bank_0[(address - 0xC000) as usize],
            0xD000 ... 0xDFFF => self.wram_bank_1_n[(address - 0xD000) as usize],
            0xE000 ... 0xFDFF => self.wram_echo[(address - 0xE000) as usize],
            0xFE00 ... 0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFEA0 ... 0xFEFF => self.unusable[(address - 0xFEA0) as usize],
            0xFF00 ... 0xFF7F => self.io_registers[(address - 0xFF00) as usize],
            0xFF80 ... 0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupts_enable,
            _ => panic!("Out of bound! Tried to read from {:#X}.", address),
        }
    }

    pub fn handle_banking(&mut self, address: u16, byte: u8) {
        match address {
            0x000 ... 0x1FFF => {
                if self.cartridge_type == CartridgeType::Mbc1 || self.cartridge_type == CartridgeType::Mbc2 || self.cartridge_type == CartridgeType::Mbc5 {
                    self.enable_ram_banking(address, byte);
                }
            },
            0x2000 ... 0x2FFF => {
                if self.cartridge_type == CartridgeType::Mbc1 || self.cartridge_type == CartridgeType::Mbc2 {
                    self.change_rom_bank_lower_bits(byte);
                } else if self.cartridge_type == CartridgeType::Mbc5 {
                    self.change_rom_bank_lower_bits_mbc5(byte);
                }
            },
            0x3000 ... 0x3FFF => {
                if self.cartridge_type == CartridgeType::Mbc1 || self.cartridge_type == CartridgeType::Mbc2 {
                    self.change_rom_bank_lower_bits(byte);
                } else if self.cartridge_type == CartridgeType::Mbc5 {
                    self.change_rom_bank_9th_bit_mbc5(byte);
                }
            },
            0x4000 ... 0x5FFF => {
                if self.cartridge_type == CartridgeType::Mbc1 {
                    if self.rom_banking_enabled {
                        self.change_rom_bank_upper_bits(byte);
                    } else {
                        self.change_ram_bank(byte);
                    }
                } else if self.cartridge_type == CartridgeType::Mbc5 { 
                    if self.rom_banking_enabled {
                        self.change_rom_bank_9th_bit_mbc5(byte);
                    } else {
                        self.change_ram_bank(byte);
                    }
                }
            },
            0x6000 ... 0x7FFF => {
                if self.cartridge_type == CartridgeType::Mbc1 {
                    self.handle_mbc1_mode(byte);
                }
            },
            _ => panic!("Address {:#X} is not valid for memory bank handling.", address),
        }
    }

    pub fn enable_ram_banking(&mut self, address: u16, byte: u8) {
        if self.cartridge_type == CartridgeType::Mbc2 && (util::is_bit_one(address, 3)) {
            return;
        }
        let relevant_bits: u8 = byte & 0xF;
        match relevant_bits {
            0x0A => self.ram_banking_enabled = true,
            _ => self.ram_banking_enabled = false,
        }
    }

    pub fn change_rom_bank_lower_bits(&mut self, byte: u8) {
        match self.cartridge_type {
            CartridgeType::Mbc1 => {
                let lower_bits: u16 = byte as u16 & 0x1F;
                self.current_rom_bank &= 0xE0;
                self.current_rom_bank |= lower_bits;
                if self.current_rom_bank == 0x0 || self.current_rom_bank == 0x20 || self.current_rom_bank == 0x40 || self.current_rom_bank == 0x60 {
                    self.current_rom_bank += 0x1;
                }
            }, 
            CartridgeType::Mbc2 => {
                self.current_rom_bank = byte as u16 & 0xF;
                if self.current_rom_bank == 0x0 {
                    self.current_rom_bank = 0x1;
                }
            },
            _ => panic!("Unsupported cartridge type."),
        }
    }

    pub fn change_rom_bank_lower_bits_mbc5(&mut self, byte: u8) {
        if self.cartridge_type == CartridgeType::Mbc5 {
            let lower_bits: u16 = byte as u16 & 0xFF;
            self.current_rom_bank &= 0xF00;
            self.current_rom_bank |= lower_bits;
        } else {
            panic!("Tried to handle cartridge as MBC5.");
        }
    }

    pub fn change_rom_bank_9th_bit_mbc5(&mut self, byte: u8) {
        if self.cartridge_type == CartridgeType::Mbc5 {
            let upper_bit: u16 = byte as u16 & (0x1 << 8);
            self.current_rom_bank &= 0xFF;
            self.current_rom_bank |= upper_bit;
        } else {
            panic!("Tried to handle cartridge as MBC5.");
        }
    }

    pub fn change_rom_bank_upper_bits(&mut self, byte: u8) {
        let upper_bits: u16 = byte as u16 & 0xE0;
        self.current_rom_bank &= 0x1F;
        self.current_rom_bank |= upper_bits;
        if self.current_rom_bank == 0x0 || self.current_rom_bank == 0x20 || self.current_rom_bank == 0x40 || self.current_rom_bank == 0x60 {
            self.current_rom_bank += 0x1;
        }
    }

    pub fn change_ram_bank(&mut self, byte: u8) {
        self.current_ram_bank = byte as u16 & 0x3;
    }
    
    pub fn handle_mbc1_mode(&mut self, byte: u8) {
        let memory_mode_bit: u16 = byte as u16 & 0x1;
        if memory_mode_bit == 0x0 {
            self.rom_banking_enabled = true;
            self.ram_banking_enabled = false;
            self.current_ram_bank = 0x0;
        } else {
            self.rom_banking_enabled = false;
            self.ram_banking_enabled = true;
        }
    }

    pub fn load_bootstrap_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.rom_bank_00[i] = *byte;
        }
    }

    pub fn load_game_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.cartridge[i] = *byte;
            if i < 0x4000 {
                self.rom_bank_00[i] = *byte;
            } else if i < 0x8000 {
                self.rom_bank_01_nn[i - 0x4000] = *byte;
            }
            match self.cartridge[consts::CARTRIDGE_TYPE_ADDR as usize] {
                0x0 => self.cartridge_type = CartridgeType::RomOnly,
                0x1 ... 0x3 => self.cartridge_type = CartridgeType::Mbc1,
                0x5 ... 0x6 => self.cartridge_type = CartridgeType::Mbc2,
                0x19 ... 0x1E => self.cartridge_type = CartridgeType::Mbc5,
                _ => panic!("Cartridges of type {:#X} are not yet supported.", self.cartridge[consts::CARTRIDGE_TYPE_ADDR as usize]),
            }
        }
    }
}
