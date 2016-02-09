use std::fmt;

// TODO: Implement MBC logic
pub struct Memory {
    rom_bank_00: [u8; 0x4000],
    rom_bank_01_nn: [u8; 0x4000],
    vram: [u8; 0x2000],
    external_ram: [u8; 0x2000],
    wram_bank_0: [u8; 0x1000],
    wram_bank_1_n: [u8; 0x1000],
    wram_echo: [u8; 0x1E00], // mirror c000 to ddff
    oam: [u8; 0xA0],
    unusable: [u8; 0x60],
    io_registers: [u8; 0x80],
    hram: [u8; 0x7F],
    interrupts_enable: u8,
    cartridge: Vec<u8>,
}


// TODO: Rewrite to work with new memory structure
//impl fmt::Display for Memory {
//fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //let columns: u8 = 16;

        //let mut res: String = "".to_owned();

        //let mut to: usize = self.mem.len();
        //let mut from: usize = 0;

        //if let Some(fr) = f.width() {
        //    from = fr;
        //}
        //if let Some(t) = f.precision() {
        //    to = t;
        //}

       // let mut i: usize = 0;
        //while i >= from && i < to {
            //if i as u8 % columns == 0 {
            //    res = res + &format!("\n{:01$x}: ", i, 8);
            //}
            //let lhs: u8 = self.mem[i];
            //i += 1;
            //let rhs: u8 = self.mem[i];
            //res = res + &format!("{:01$x}", lhs, 2);
            //res = res + &format!("{:01$x} ", rhs, 2);

            //i += 1;
        //}
       // write!(f, "{}", res)
//    }
//}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            rom_bank_00: [0; 0x4000],
            rom_bank_01_nn: [0; 0x4000],
            vram: [0; 0x2000],
            external_ram: [0; 0x2000],
            wram_bank_0: [0; 0x1000],
            wram_bank_1_n: [0; 0x1000],
            wram_echo: [0; 0x1E00], // mirror C000 to DDFF
            oam: [0; 0xA0],
            unusable: [0; 0x60],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            interrupts_enable: 0,
            cartridge: vec![0; 0x8000], // assuming 32kb cartridge
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        if cfg!(debug_assertions) {
            println!("{:#x} -> mem[{:#x}]", value, address);
        }
        match address {
            0x0000 ... 0x3FFF => self.rom_bank_00[address as usize] = value,
            0x4000 ... 0x7FFF => self.rom_bank_01_nn[(address - 0x4000) as usize] = value,
            0x8000 ... 0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xA000 ... 0xBFFF => self.external_ram[(address - 0xA000) as usize] = value,
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
            0x4000 ... 0x7FFF => self.rom_bank_01_nn[(address - 0x4000) as usize],
            0x8000 ... 0x9FFF => self.vram[(address - 0x8000) as usize],
            0xA000 ... 0xBFFF => self.external_ram[(address - 0xA000) as usize],
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
    
    pub fn load_bootstrap_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.rom_bank_00[i] = *byte;
        }
    }

    pub fn load_game_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.cartridge[i] = *byte;
            self.write_byte(i as u16, *byte);
        }
    }
}
