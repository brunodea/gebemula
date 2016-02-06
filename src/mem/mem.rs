use mem::memregion::MemoryRegion;

// TODO: Implement MBC logic
pub struct Memory {
    mem: Vec<u8>,
    rom_bank_00: MemoryRegion,
    rom_bank_01_nn: MemoryRegion,
    vram: MemoryRegion,
    external_ram: MemoryRegion,
    wram_bank_0: MemoryRegion,
    wram_bank_1_n: MemoryRegion,
    wram_echo: MemoryRegion,
    oam: MemoryRegion,
    unusable: MemoryRegion,
    io_registers: MemoryRegion,
    hram: MemoryRegion,
    interrupts_enable: MemoryRegion,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            mem: vec![0; 0x10000], // 0x0000 to 0xFFFF
            rom_bank_00: MemoryRegion::new(0x0000, 0x3FFF),
            rom_bank_01_nn: MemoryRegion::new(0x4000, 0x7FFF),
            vram: MemoryRegion::new(0x8000, 0x9FFF),
            external_ram: MemoryRegion::new(0xA000, 0xBFFF),
            wram_bank_0: MemoryRegion::new(0xC000, 0xCFFF),
            wram_bank_1_n: MemoryRegion::new(0xD000, 0xDFFF),
            wram_echo: MemoryRegion::new(0xE000, 0xFDFF),
            oam: MemoryRegion::new(0xFE00, 0xFE9F),
            unusable: MemoryRegion::new(0xFEA0, 0xFEFF),
            io_registers: MemoryRegion::new(0xFF00, 0xFF7F),
            hram: MemoryRegion::new(0xFF80, 0xFFFE),
            interrupts_enable: MemoryRegion::new(0xFFFF, 0xFFFF),
        }
    }

    pub fn get_size(&self) -> usize {
        self.mem.len()
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        println!("{:#x} -> mem[{:#x}]", value, address);
        match address {
            0xC000 ... 0xCFFF => self.mem[(address + 0x2000) as usize] = value,
            0xE000 ... 0xFDFF => self.mem[(address - 0x2000) as usize] = value,
            _ => self.mem[address as usize] = value,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        println!("{:#x} <- mem[{:#x}]", self.mem[address as usize], address);
        self.mem[address as usize]
    }

    pub fn read_bootstrap_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.mem[i] = *byte;
        }
    }
}
