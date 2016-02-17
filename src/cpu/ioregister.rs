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

pub fn set_ly_reg(mut value: u8, memory: &mut mem::Memory) {
    value = if value > 0x99 { 0 } else { value };
    memory.write_byte(consts::LY_REGISTER_ADDR, value);
}
