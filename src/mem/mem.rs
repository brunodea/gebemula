use util::util;
use mem::consts;
use time;

#[derive(Copy, Clone, PartialEq, Debug)]
enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

struct Rtc {
    seconds_reg: Option<u16>,
    minutes_reg: Option<u16>,
    hours_reg: Option<u16>,
    day_lower_bits_reg: Option<u16>,
    day_upper_bits_reg: Option<u16>,
    latch_clock_data: Option<u8>,
}

impl Default for Rtc {
    fn default() -> Rtc {
        Rtc {
            seconds_reg: None,
            minutes_reg: None,
            hours_reg: None,
            day_lower_bits_reg: None,
            day_upper_bits_reg: None,
            latch_clock_data: None,
        }
    }
}

pub struct Memory {
    bootstrap_rom: [u8; 0x100],
    vram: [u8; 0x2000],
    external_ram: [u8; 0x7A1200], // TODO: dinamically allocate size?
    wram: [u8; 0x2000],
    oam: [u8; 0xA0],
    io_registers: [u8; 0x80],
    hram: [u8; 0x7F],
    interrupts_enable: u8,
    cartridge: Vec<u8>,
    cartridge_type: CartridgeType,
    current_rom_bank: u16,
    current_ram_bank: u16,
    rom_banking_enabled: bool,
    external_ram_enabled: bool,
    bootstrap_enabled: bool,
    can_access_vram: bool,
    can_access_oam: bool,
    rtc: Rtc,
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            bootstrap_rom: [0; 0x100],
            vram: [0; 0x2000],
            external_ram: [0; 0x7A1200],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            interrupts_enable: 0x0,
            cartridge: vec![0; 0x200000],
            cartridge_type: CartridgeType::RomOnly,
            current_rom_bank: 0x1,
            current_ram_bank: 0x0,
            rom_banking_enabled: true,
            external_ram_enabled: false,
            bootstrap_enabled: true,
            can_access_vram: true,
            can_access_oam: true,
            rtc: Rtc::default(),
        }
    }
}

impl Memory {
    // returns a string with the memory data from min_addr to max_addr.
    pub fn format(&self, min_addr: Option<u16>, max_addr: Option<u16>) -> String {
        let columns: u8 = 16;

        let mut res: String = "".to_owned();

        let mut to: usize = 0xffff;
        let mut from: usize = 0;

        if let Some(fr) = min_addr {
            from = fr as usize;
        }
        if let Some(t) = max_addr {
            to = t as usize;
        }

        let mut i: usize = from;
        while i >= from && i < to {
            if i as u8 % columns == 0 {
                res = res + &format!("\n{:01$x}: ", i, 8);
            }
            let byte: u8 = self.read_byte(i as u16);
            res = res + &format!("{:01$x} ", byte, 2);

            i += 1;
        }
        res
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000...0x7FFF => {
                if self.cartridge_type == CartridgeType::RomOnly {
                    // self.cartridge[address as usize] = value;
                } else {
                    self.handle_banking(address, value);
                }
            }
            0x8000...0x9FFF => {
                if self.can_access_vram {
                    self.vram[(address - 0x8000) as usize] = value;
                }
            }
            0xA000...0xBFFF => {
                // TODO && if battery powered?
                if self.external_ram_enabled {
                    self.external_ram[address as usize - 0xA000 +
                                      (self.current_ram_bank as usize *
                                       consts::RAM_BANK_SIZE as usize)] = value;
                    // self.external_ram_enabled = false;
                }
            }
            0xC000...0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000...0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            0xFE00...0xFE9F => {
                if self.can_access_oam {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
            }
            0xFEA0...0xFEFF => (),// panic!("writing to unusable ram."),
            0xFF00...0xFF7F => self.io_registers[(address - 0xFF00) as usize] = value,
            0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.interrupts_enable = value,
            _ => panic!("Out of bound! Tried to write to {:#x}.", address),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000...0x00FF => {
                if self.bootstrap_enabled {
                    self.bootstrap_rom[address as usize]
                } else {
                    self.cartridge[address as usize]
                }
            }
            0x0100...0x3FFF => self.cartridge[address as usize],
            0x4000...0x7FFF => {
                self.cartridge[address as usize - 0x4000 +
                               (self.current_rom_bank as usize * consts::ROM_BANK_SIZE as usize)]
            }
            0x8000...0x9FFF => {
                if self.can_access_vram {
                    self.vram[(address - 0x8000) as usize]
                } else {
                    0xFF
                }
            }
            0xA000...0xBFFF => {
                let bank_addr: usize = self.current_ram_bank as usize *
                                       consts::RAM_BANK_SIZE as usize;
                self.external_ram[address as usize - 0xA000 + bank_addr]
            }
            0xC000...0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000...0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00...0xFE9F => {
                if self.can_access_oam {
                    self.oam[(address - 0xFE00) as usize]
                } else {
                    0xFF
                }
            }
            0xFF00...0xFF7F => self.io_registers[(address - 0xFF00) as usize],
            0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupts_enable,
            _ => panic!("Out of bound! Tried to read from {:#x}.", address),
        }
    }

    fn handle_banking(&mut self, address: u16, byte: u8) {
        match address {
            0x0000...0x1FFF => {
                if self.cartridge_type == CartridgeType::Mbc2 {
                    self.external_ram_enabled = util::is_bit_one(address, 8);
                } else if self.cartridge_type != CartridgeType::RomOnly {
                    self.external_ram_enabled = (byte & 0x0F) == 0x0A;
                }
            }
            0x2000...0x2FFF => {
                match self.cartridge_type {
                    CartridgeType::Mbc1 | CartridgeType::Mbc2 => {
                        self.change_rom_bank_lower_bits(address, byte);
                    }
                    CartridgeType::Mbc3 => {
                        self.change_rom_bank_mbc3(byte);
                    }
                    CartridgeType::Mbc5 => {
                        self.change_rom_bank_lower_bits_mbc5(byte);
                    }
                    _ => unreachable!(),
                }
            }
            0x3000...0x3FFF => {
                match self.cartridge_type {
                    CartridgeType::Mbc1 | CartridgeType::Mbc2 => {
                        self.change_rom_bank_lower_bits(address, byte);
                    }
                    CartridgeType::Mbc3 => {
                        self.change_rom_bank_mbc3(byte);
                    }
                    CartridgeType::Mbc5 => {
                        self.change_rom_bank_9th_bit_mbc5(byte);
                    }
                    _ => unreachable!(),
                }
            }
            0x4000...0x5FFF => {
                match self.cartridge_type {
                    CartridgeType::Mbc1 => {
                        if self.rom_banking_enabled {
                            self.current_rom_bank = (self.current_rom_bank & 0b0001_1111) |
                                                    ((byte as u16 & 0b11) << 5);
                            if self.current_rom_bank == 0x0 || self.current_rom_bank == 0x20 ||
                               self.current_rom_bank == 0x40 ||
                               self.current_rom_bank == 0x60 {

                                self.current_rom_bank += 0x1;
                            }
                        } else {
                            self.change_ram_bank(byte);
                        }
                    }
                    CartridgeType::Mbc3 => {
                        match byte {
                            0x0...0x3 => {
                                self.change_ram_bank(byte);
                            }
                            0x8 => {
                                self.rtc.seconds_reg = Some(address);
                            }
                            0x9 => {
                                self.rtc.minutes_reg = Some(address);
                            }
                            0xA => {
                                self.rtc.hours_reg = Some(address);
                            }
                            0xB => {
                                self.rtc.day_lower_bits_reg = Some(address);
                            }
                            0xC => {
                                self.rtc.day_upper_bits_reg = Some(address);
                            }
                            _ => (),
                        }
                    }
                    CartridgeType::Mbc5 => {
                        if self.rom_banking_enabled {
                            self.change_rom_bank_9th_bit_mbc5(byte);
                        } else {
                            self.change_ram_bank(byte);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            0x6000...0x7FFF => {
                match self.cartridge_type {
                    CartridgeType::Mbc1 => {
                        self.rom_banking_enabled = byte & 0b1 == 0;
                        if self.rom_banking_enabled {
                            self.current_ram_bank = 0;
                        } else {
                            self.current_rom_bank &= 0b0001_1111;
                        }
                    }
                    CartridgeType::Mbc3 => {
                        if self.rtc.latch_clock_data.is_none() {
                            self.rtc.latch_clock_data = Some(byte);
                        } else {
                            if self.rtc.latch_clock_data.unwrap() == 0 && byte == 1 {
                                let now = time::now();
                                if let Some(addr) = self.rtc.seconds_reg {
                                    let s: u8 = if now.tm_sec < 60 {
                                        now.tm_sec as u8
                                    } else {
                                        0
                                    };
                                    self.write_byte(addr, s);
                                }
                                if let Some(addr) = self.rtc.minutes_reg {
                                    self.write_byte(addr, now.tm_min as u8);
                                }
                                if let Some(addr) = self.rtc.hours_reg {
                                    self.write_byte(addr, now.tm_hour as u8);
                                }
                                if let Some(addr) = self.rtc.day_lower_bits_reg {
                                    self.write_byte(addr, now.tm_yday as u8);
                                }
                                if let Some(addr) = self.rtc.day_upper_bits_reg {
                                    // tm_yday will limit the Rtc to '365' days and not
                                    // 511 as it should be. Also, the day counter carry bit
                                    // will never be set.
                                    // However, it doesn't really matter for this emulator.
                                    let day_ms_bit: u8 = ((now.tm_yday & 0x100) >> 8) as u8;
                                    self.write_byte(addr, day_ms_bit);
                                }
                            }
                            self.rtc.latch_clock_data = Some(byte);
                        }
                    }
                    _ => (),
                }
            }
            _ => unreachable!(),
        }
    }

    fn change_rom_bank_lower_bits(&mut self, address: u16, byte: u8) {
        match self.cartridge_type {
            CartridgeType::Mbc1 => {
                let lower_bits: u16 = byte as u16 & 0x1F;
                self.current_rom_bank &= 0b1110_0000;
                self.current_rom_bank |= lower_bits;
                if self.current_rom_bank == 0x0 || self.current_rom_bank == 0x20 ||
                   self.current_rom_bank == 0x40 ||
                   self.current_rom_bank == 0x60 {
                    self.current_rom_bank += 0x1;
                }
            }
            CartridgeType::Mbc2 => {
                if util::is_bit_one(address, 8) {
                    self.current_rom_bank = byte as u16 & 0xF;
                    if self.current_rom_bank == 0x0 {
                        self.current_rom_bank = 0x1;
                    }
                }
            }
            _ => panic!("Unsupported cartridge type."),
        }
    }

    fn change_rom_bank_lower_bits_mbc5(&mut self, byte: u8) {
        if self.cartridge_type == CartridgeType::Mbc5 {
            let lower_bits: u16 = byte as u16 & 0xFF;
            self.current_rom_bank &= 0xF00;
            self.current_rom_bank |= lower_bits;
        } else {
            panic!("Tried to handle cartridge as MBC5.");
        }
    }

    fn change_rom_bank_9th_bit_mbc5(&mut self, byte: u8) {
        if self.cartridge_type == CartridgeType::Mbc5 {
            let upper_bit: u16 = byte as u16 & (0x1 << 8);
            self.current_rom_bank &= 0xFF;
            self.current_rom_bank |= upper_bit;
        } else {
            panic!("Tried to handle cartridge as MBC5.");
        }
    }

    fn change_rom_bank_mbc3(&mut self, byte: u8) {
        self.current_rom_bank = byte as u16 & 0x7F;
        if self.current_rom_bank == 0x0 {
            self.current_rom_bank = 0x1;
        }
    }

    fn change_ram_bank(&mut self, byte: u8) {
        self.current_ram_bank = (byte & 0b11) as u16;
    }

    pub fn set_access_vram(&mut self, can_access: bool) {
        self.can_access_vram = can_access;
    }
    pub fn set_access_oam(&mut self, can_access: bool) {
        self.can_access_oam = can_access;
    }

    pub fn restart(&mut self) {
        self.vram = [0; 0x2000];
        self.wram = [0; 0x2000];
        self.oam = [0; 0xA0];
        self.io_registers = [0; 0x80];
        self.hram = [0; 0x7F];
        self.interrupts_enable = 0x0;
        self.current_rom_bank = 0x1;
        self.current_ram_bank = 0x0;
        self.rom_banking_enabled = true;
        self.external_ram_enabled = false;
        self.bootstrap_enabled = true;
        self.can_access_vram = true;
        self.rtc = Rtc::default();
    }

    pub fn disable_bootstrap(&mut self) {
        self.bootstrap_enabled = false;
        self.write_byte(0xFF05, 0x00);
        self.write_byte(0xFF06, 0x00);
        self.write_byte(0xFF07, 0x00);
        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF14, 0xBF);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0x00);
        self.write_byte(0xFF19, 0xBF);
        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1E, 0xBF);
        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0x00);
        self.write_byte(0xFF22, 0x00);
        self.write_byte(0xFF23, 0xBF);
        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
        self.write_byte(0xFF40, 0x91);
        self.write_byte(0xFF42, 0x00);
        self.write_byte(0xFF43, 0x00);
        self.write_byte(0xFF45, 0x00);
        self.write_byte(0xFF47, 0xFC);
        self.write_byte(0xFF48, 0xFF);
        self.write_byte(0xFF49, 0xFF);
        self.write_byte(0xFF4A, 0x00);
        self.write_byte(0xFF4B, 0x00);
        self.write_byte(0xFFFF, 0x00);
    }

    pub fn load_bootstrap_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.bootstrap_rom[i] = *byte;
        }
    }

    pub fn load_game_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            self.cartridge[i] = *byte;
        }
        match self.cartridge[consts::CARTRIDGE_TYPE_ADDR as usize] {
            0x0 => self.cartridge_type = CartridgeType::RomOnly,
            0x1...0x3 => self.cartridge_type = CartridgeType::Mbc1,
            0x5...0x6 => self.cartridge_type = CartridgeType::Mbc2,
            0x11...0x13 => self.cartridge_type = CartridgeType::Mbc3,
            0x19...0x1E => self.cartridge_type = CartridgeType::Mbc5,
            _ => {
                panic!("Cartridges of type {:#X} are not yet supported.",
                       self.cartridge[consts::CARTRIDGE_TYPE_ADDR as usize])
            }
        }
    }
}
