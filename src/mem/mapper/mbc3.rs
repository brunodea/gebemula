use mem::mapper::rtc::Rtc;
use mem::mapper::{Mapper, ROM_BANK_SIZE, RAM_BANK_SIZE};

pub struct Mbc3Mapper {
    /// Mapped to the ROM area. Up to 2 MiB in size.
    rom: Box<[u8]>,
    /// Mapped to the RAM area. Up to 64 KiB in size.
    ram: Box<[u8]>,

    current_rom_bank: u8,
    current_ram_bank: u8,
    ram_enabled: bool,

    has_battery: bool,
    /// True is SRAM has been written to since the last time it was saved.
    ram_modified: bool,

    rtc: Option<Rtc>,
}

impl Mbc3Mapper {
    pub fn new(rom: Box<[u8]>, ram: Box<[u8]>, has_battery: bool, has_rtc: bool) -> Mbc3Mapper {
        assert!(rom.len() <= 2 << 20);
        assert!(rom.len().is_power_of_two());
        assert!(ram.len() <= 64 << 10);
        assert!(ram.is_empty() || ram.len().is_power_of_two());

        Mbc3Mapper {
            rom: rom,
            ram: ram,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
            has_battery: has_battery,
            ram_modified: false,
            rtc: if has_rtc { Some(Rtc::new()) } else { None },
        }
    }

    fn rom_mask(&self) -> usize {
        self.rom.len() - 1
    }

    fn ram_mask(&self) -> usize {
        self.ram.len() - 1
    }
}

impl Mapper for Mbc3Mapper {
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
        match (address >> 13) & 0b11 {
            0 => {
                // RAM enable
                self.ram_enabled = data & 0xF == 0xA;
            }
            1 => {
                // ROM bank
                let mut new_bank = data & 0x7F;
                if new_bank == 0 {
                    new_bank = 1;
                }

                self.current_rom_bank = new_bank;
            }
            2 => {
                // RAM bank / RTC register
                self.current_ram_bank = data & 0xF;
            }
            3 => {
                // latch RTC
                if let Some(ref mut rtc) = self.rtc {
                    if data & 1 == 1 {
                        rtc.latch();
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        match self.current_ram_bank {
            0x0...0x7 if !self.ram.is_empty() => {
                let offset = self.current_ram_bank as usize * RAM_BANK_SIZE +
                             (address & 0x1FFF) as usize;
                self.ram[offset & self.ram_mask()]
            }
            0x8...0xF if self.rtc.is_some() => {
                self.rtc.as_ref().unwrap().read(self.current_ram_bank)
            }
            _ => 0xFF,
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        if !self.ram_enabled {
            return;
        }

        match self.current_ram_bank {
            0x0...0x7 if !self.ram.is_empty() => {
                let offset = self.current_ram_bank as usize * RAM_BANK_SIZE +
                             (address & 0x1FFF) as usize;
                let ram_mask = self.ram_mask();
                self.ram[offset & ram_mask] = data;
                self.ram_modified = true;
            }
            0x8...0xF if self.rtc.is_some() => {
                self.rtc.as_mut().unwrap().write(self.current_ram_bank, data);
            }
            _ => (),
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
