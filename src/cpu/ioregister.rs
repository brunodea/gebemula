use cpu::consts;
use cpu::interrupt;
use super::super::mem::mem;

pub fn update_stat_reg_coincidence_flag(memory: &mut mem::Memory) {
    let coincidence_flag: u8 = if stat_reg_coincidence_flag(memory) { 0b100 } else { 0b000 };
    let new_stat: u8 = 
        (memory.read_byte(consts::STAT_REGISTER_ADDR) & 0b1111_1011) | coincidence_flag;
    memory.write_byte(consts::STAT_REGISTER_ADDR, new_stat);
}

pub fn update_stat_reg_mode_flag(mode_flag: u8, memory: &mut mem::Memory) {
    let new_stat: u8 = 
        (memory.read_byte(consts::STAT_REGISTER_ADDR) & 0b1111_1100) | mode_flag;
    memory.write_byte(consts::STAT_REGISTER_ADDR, new_stat);
}

pub fn stat_reg_coincidence_flag(memory: &mem::Memory) -> bool {
    memory.read_byte(consts::LY_REGISTER_ADDR) == memory.read_byte(consts::LYC_REGISTER_ADDR)
}

pub fn lcdc_stat_interrupt(memory: &mut mem::Memory) {
    update_stat_reg_coincidence_flag(memory);
    let reg: u8 = memory.read_byte(consts::STAT_REGISTER_ADDR);
    if  ((reg >> 6 == 0b1) && stat_reg_coincidence_flag(memory)) ||
        ((reg >> 5 == 0b1) && (reg & 0b11 == 0b10)) ||
        ((reg >> 4 == 0b1) && (reg & 0b11 == 0b01)) ||
        ((reg >> 3 == 0b1) && (reg & 0b11 == 0b00)) {

        interrupt::request(interrupt::Interrupt::LCDC, memory);
    }
}

pub struct LyRegister {
    current_cycles: u32,
}

impl LyRegister {
    pub fn new() -> LyRegister {
        LyRegister {
            current_cycles: 0,
        }
    }

    //LY updates on different rates depending on its current value.
    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        self.current_cycles += cycles;
        let current_value: u8 = memory.read_byte(consts::LY_REGISTER_ADDR);
        let should_update: bool = match current_value {
            0x0 => self.current_cycles >= 856,
            0x1 ... 0x98 => self.current_cycles >= 456,
            0x99 => self.current_cycles >= 56,
            _ => {
                self.current_cycles = 0;
                memory.write_byte(consts::LY_REGISTER_ADDR, 0);
                false
            },
        };
        if should_update {
            self.current_cycles = 0;
            memory.write_byte(consts::LY_REGISTER_ADDR, current_value + 1);
        }
    }
}
