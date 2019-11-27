use crate::mem::mapper::{Mapper, ROM_BANK_SIZE};

pub struct Mbc2Mapper {
    /// Mapped to the ROM area. Up to 256 KiB in size.
    rom: Box<[u8]>,
    /// Mapped to the RAM area. Exactly 512 bytes in size. Only the 4 lower bits are used, but we
    /// just waste the top 4 bits instead of packing the nibbles.
    ram: Box<[u8]>,

    current_rom_bank: u8,
    ram_enabled: bool,

    has_battery: bool,
    /// True is SRAM has been written to since the last time it was saved.
    ram_modified: bool,
}

impl Mbc2Mapper {
    pub fn new(rom: Box<[u8]>, ram: Box<[u8]>, has_battery: bool) -> Mbc2Mapper {
        assert!(rom.len() <= 256 << 10);
        assert!(rom.len().is_power_of_two());
        assert!(ram.len() == 512);

        Mbc2Mapper {
            rom: rom,
            ram: ram,
            current_rom_bank: 1,
            ram_enabled: false,
            has_battery: has_battery,
            ram_modified: false,
        }
    }

    fn rom_mask(&self) -> usize {
        self.rom.len() - 1
    }

    fn ram_mask(&self) -> usize {
        self.ram.len() - 1
    }
}

impl Mapper for Mbc2Mapper {
    fn read_rom(&self, address: u16) -> u8 {
        let bank = if address & 0x4000 == 0 {
            0
        } else {
            self.current_rom_bank
        };
        let offset = bank as usize * ROM_BANK_SIZE + (address & 0x3FFF) as usize;

        self.rom[offset & self.rom_mask()]
    }

    fn write_rom(&mut self, address: u16, data: u8) {
        match address >> 13 & 0b11 {
            0 => {
                // RAM enable
                if address & 0x0100 == 0 {
                    self.ram_enabled = data & 0xF == 0xA;
                }
            }
            1 => {
                // ROM bank
                if address & 0x0100 != 0 {
                    let mut new_bank = data & 0xF;
                    if new_bank == 0 {
                        new_bank = 1;
                    }

                    self.current_rom_bank = new_bank;
                }
            }
            2 | 3 => {
                println!("WARNING: write to unknown MBC2 address: {:#04X}", address);
            }
            _ => unreachable!(),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if self.ram_enabled && !self.ram.is_empty() {
            let offset = (address & 0x1FFF) as usize;
            self.ram[offset & self.ram_mask()] | 0xF0
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        if self.ram_enabled && !self.ram.is_empty() {
            let offset = (address & 0x1FFF) as usize;
            let ram_mask = self.ram_mask();
            self.ram[offset & ram_mask] = data | 0xF0;
            self.ram_modified = true;
        }
    }

    fn save_battery(&mut self) -> Vec<u8> {
        if self.has_battery && self.ram_modified {
            self.ram_modified = false;
            Vec::from(&*self.ram)
        } else {
            Vec::new()
        }
    }
}
