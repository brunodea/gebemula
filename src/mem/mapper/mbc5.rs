use mem::mapper::{Mapper, RAM_BANK_SIZE, ROM_BANK_SIZE};

pub struct Mbc5Mapper {
    /// Mapped to the ROM area. Up to 8 MiB in size.
    rom: Box<[u8]>,
    /// Mapped to the RAM area. Up to 128 KiB in size.
    ram: Box<[u8]>,

    current_rom_bank: u16,
    current_ram_bank: u8,
    ram_enabled: bool,

    has_battery: bool,
    /// True is SRAM has been written to since the last time it was saved.
    ram_modified: bool,

    /// Currently unused.
    rumble_on: bool,
}

impl Mbc5Mapper {
    pub fn new(rom: Box<[u8]>, ram: Box<[u8]>, has_battery: bool) -> Mbc5Mapper {
        assert!(rom.len() <= 8 << 20);
        assert!(rom.len().is_power_of_two());
        assert!(ram.len() <= 128 << 10);
        assert!(ram.is_empty() || ram.len().is_power_of_two());

        Mbc5Mapper {
            rom: rom,
            ram: ram,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
            has_battery: has_battery,
            ram_modified: false,
            rumble_on: false,
        }
    }

    fn rom_mask(&self) -> usize {
        self.rom.len() - 1
    }

    fn ram_mask(&self) -> usize {
        self.ram.len() - 1
    }
}

impl Mapper for Mbc5Mapper {
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
        match (address >> 12) & 0b111 {
            0 | 1 => {
                // RAM enable
                self.ram_enabled = data & 0xF == 0xA;
            }
            2 => {
                // ROM bank low bits
                self.current_rom_bank &= 0xFF00;
                self.current_rom_bank |= data as u16;
            }
            3 => {
                // ROM bank high bits
                self.current_rom_bank &= 0x00FF;
                self.current_rom_bank |= ((data & 0x01) as u16) << 8;
            }
            4 | 5 => {
                // RAM bank
                self.current_ram_bank = data & 0xF;
                self.rumble_on = data & 0x8 != 0; // Yes, this overlaps with the RAM selector
            }
            6 | 7 => {
                // unknown / unused
                //println!("WARNING: write to unknown MBC5 address: {:#04X}, value {:#04X}", address, data);
            }
            _ => unreachable!(),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if self.ram_enabled && !self.ram.is_empty() {
            let offset =
                self.current_ram_bank as usize * RAM_BANK_SIZE + (address & 0x1FFF) as usize;
            self.ram[offset & self.ram_mask()]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        if self.ram_enabled && !self.ram.is_empty() {
            let offset =
                self.current_ram_bank as usize * RAM_BANK_SIZE + (address & 0x1FFF) as usize;
            let ram_mask = self.ram_mask();
            self.ram[offset & ram_mask] = data;
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
