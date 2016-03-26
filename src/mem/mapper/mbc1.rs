use mem::mapper::{Mapper, ROM_BANK_SIZE, RAM_BANK_SIZE};

pub struct Mbc1Mapper {
    /// Mapped to the ROM area. Up to 2 MiB in size.
    rom: Box<[u8]>,
    /// Mapped to the RAM area. Up to 32 KiB in size.
    ram: Box<[u8]>,

    current_rom_bank: u8,
    current_ram_bank: u8,
    ram_enabled: bool,
    ram_banking_enabled: bool,
}

impl Mbc1Mapper {
    pub fn new(rom: Box<[u8]>, ram: Box<[u8]>) -> Mbc1Mapper {
        assert!(rom.len() <=  2 << 20);
        assert!(ram.len() <= 32 << 10);
        Mbc1Mapper {
            rom: rom,
            ram: ram,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
            ram_banking_enabled: false,
        }
    }
}

impl Mapper for Mbc1Mapper {
    fn read_rom(&self, address: u16) -> u8 {
        let bank = if address & 0x4000 == 0 { 0 } else { self.current_rom_bank };
        let offset = bank as usize * ROM_BANK_SIZE + (address & 0x3FFF) as usize;

        if offset < self.rom.len() {
            self.rom[offset]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, address: u16, data: u8) {
        match address >> 13 & 0b11 {
            0 => { // RAM enable
                self.ram_enabled = data & 0xF == 0xA;
            },
            1 => { // ROM bank
                let mut new_bank = data & 0x1F;
                if new_bank == 0 {
                    new_bank = 1;
                }

                self.current_rom_bank = (self.current_rom_bank & !0x1F) | new_bank;
            },
            2 => { // RAM bank / upper ROM bank
                let new_bank = data & 0x3;
                if self.ram_banking_enabled {
                    self.current_ram_bank = new_bank;
                } else {
                    self.current_rom_bank = (self.current_rom_bank & !0x60) | (new_bank << 5);
                }
            },
            3 => { // ROM/RAM mode
                self.ram_banking_enabled = data & 0x1 == 0x1;
                if self.ram_banking_enabled {
                    self.current_rom_bank &= 0x1F;
                } else {
                    self.current_ram_bank = 0;
                }
            },
            _ => unreachable!(),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        let offset = self.current_ram_bank as usize * RAM_BANK_SIZE + (address & 0x1FFF) as usize;
        if self.ram_enabled && offset < self.ram.len() {
            self.ram[offset]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        let offset = self.current_ram_bank as usize * RAM_BANK_SIZE + (address & 0x1FFF) as usize;
        if self.ram_enabled && offset < self.ram.len() {
            self.ram[offset] = data;
        }
    }
}
