use mem::mapper::Mapper;

pub struct RomMapper {
    /// Mapped to the ROM area. Up to 32 KiB in size.
    rom: Box<[u8]>,
    /// Mapped to the RAM area. Up to 8 KiB in size.
    ram: Box<[u8]>,
}

impl RomMapper {
    pub fn new(rom: Box<[u8]>, ram: Box<[u8]>) -> RomMapper {
        assert!(rom.len() <= 32 << 10);
        assert!(ram.len() <=  8 << 10);
        RomMapper {
            rom: rom,
            ram: ram,
        }
    }
}

impl Mapper for RomMapper {
    fn read_rom(&self, address: u16) -> u8 {
        let offset = (address & 0x7FFF) as usize;
        if offset < self.rom.len() {
            self.rom[offset]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, _address: u16, _data: u8) {
        // Writes to ROM are ignored
    }

    fn read_ram(&self, address: u16) -> u8 {
        let offset = (address & 0x1FFF) as usize;
        if offset < self.ram.len() {
            self.ram[offset]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        let offset = (address & 0x1FFF) as usize;
        if offset < self.ram.len() {
            self.ram[offset] = data;
        }
    }
}
