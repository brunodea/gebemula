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

    rtc: Option<Rtc>,
}

impl Mbc3Mapper {
    pub fn new(rom: Box<[u8]>, ram: Box<[u8]>, has_rtc: bool) -> Mbc3Mapper {
        assert!(rom.len() <=  2 << 20);
        assert!(ram.len() <= 64 << 10);
        Mbc3Mapper {
            rom: rom,
            ram: ram,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
            rtc: if has_rtc { Some(Rtc::new()) } else { None },
        }
    }
}

impl Mapper for Mbc3Mapper {
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
        match (address >> 13) & 0b11 {
            0 => { // RAM enable
                self.ram_enabled = data & 0xF == 0xA;
            },
            1 => { // ROM bank
                let mut new_bank = data & 0x7F;
                if new_bank == 0 {
                    new_bank = 1;
                }

                self.current_rom_bank = new_bank;
            },
            2 => { // RAM bank / RTC register
                self.current_ram_bank = data & 0xF;
            },
            3 => { // latch RTC
                if let Some(ref mut rtc) = self.rtc {
                    if data & 1 == 1 {
                        rtc.latch();
                    }
                }
            },
            _ => unreachable!(),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        if self.current_ram_bank < 8 {
            let offset = self.current_ram_bank as usize * RAM_BANK_SIZE +
                         (address & 0x1FFF) as usize;
            if offset < self.ram.len() {
                self.ram[offset]
            } else {
                0xFF
            }
        } else if let Some(ref rtc) = self.rtc {
            rtc.read(self.current_ram_bank)
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        if !self.ram_enabled {
            return;
        }

        if self.current_ram_bank < 8 {
            let offset = self.current_ram_bank as usize * RAM_BANK_SIZE +
                         (address & 0x1FFF) as usize;
            if offset < self.ram.len() {
                self.ram[offset] = data;
            }
        } else if let Some(ref mut rtc) = self.rtc {
            rtc.write(self.current_ram_bank, data);
        }
    }
}
