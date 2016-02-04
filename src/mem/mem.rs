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
        if self.check_echo(address) {
            self.mem[(address + 0x2000) as usize] = value;
        }
        self.mem[address as usize] = value;
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    pub fn read_bootstrap_rom(&mut self, rom: &Vec<u8>) {
        for byte in 0x000..rom.len() as usize {
            self.mem[byte] = rom[byte];
        }
    }

    fn check_echo(&self, address: u16) -> bool {
        if address >= self.rom_bank_00.start() && address <= self.rom_bank_00.end() {
            true
        } else if address >= 0xE000 && address < 0xFE00 {
            panic!("Tried to write to a reserved memory area ({:#X}, Internal RAM Echo).", address);
        } else {
            false
        }
    }
}
