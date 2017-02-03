use mem::mapper::Mapper;

pub struct RomMapper {
    /// Mapped to the ROM area. Up to 32 KiB in size.
    rom: Box<[u8]>,
    /// Mapped to the RAM area. Up to 8 KiB in size.
    ram: Box<[u8]>,

    has_battery: bool,
    /// True is SRAM has been written to since the last time it was saved.
    ram_modified: bool,
}

impl RomMapper {
    pub fn new(rom: Box<[u8]>, ram: Box<[u8]>, has_battery: bool) -> RomMapper {
        assert!(rom.len() <= 32 << 10);
        assert!(rom.len().is_power_of_two());
        assert!(ram.len() <= 8 << 10);
        assert!(ram.len() == 0 || ram.len().is_power_of_two());

        RomMapper {
            rom: rom,
            ram: ram,
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

impl Mapper for RomMapper {
    fn read_rom(&self, address: u16) -> u8 {
        let offset = (address & 0x7FFF) as usize;
        self.rom[offset & self.rom_mask()]
    }

    fn write_rom(&mut self, _address: u16, _data: u8) {
        // Writes to ROM are ignored
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram.is_empty() {
            let offset = (address & 0x1FFF) as usize;
            self.ram[offset & self.ram_mask()]
        } else {
            0xFF
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        if !self.ram.is_empty() {
            let offset = (address & 0x1FFF) as usize;
            let mask = self.ram_mask();
            self.ram[offset & mask] = data;
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
