use super::super::mem::mem;
/*Interrupt registers*/
const IF_REGISTER_ADDR: u16 = 0xFF0F; //interrupt request register
const IE_REGISTER_ADDR: u16 = 0xFFFF; //interrupt enable

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Interrupt {
    VBlank, LCDC, TimerOverflow,
    SerialIO, TransitionHighLow
}

pub fn bit(interrupt: Interrupt) -> u8 {
    match interrupt {
        Interrupt::VBlank => 0,
        Interrupt::LCDC => 1,
        Interrupt::TimerOverflow => 2,
        Interrupt::SerialIO => 3,
        Interrupt::TransitionHighLow => 4,
    }
}

pub fn from_bit(bit: u8) -> Interrupt {
    match bit {
        0 => Interrupt::VBlank,
        1 => Interrupt::LCDC,
        2 => Interrupt::TimerOverflow,
        3 => Interrupt::SerialIO,
        4 => Interrupt::TransitionHighLow,
        _ => unreachable!(),
    }
}

pub fn address(interrupt: Interrupt) -> u16 {
    match interrupt {
        Interrupt::VBlank => 0x40,
        Interrupt::LCDC => 0x48,
        Interrupt::TimerOverflow => 0x50,
        Interrupt::SerialIO => 0x58,
        Interrupt::TransitionHighLow => 0x60,
    }
}

fn is_set_bit(bit: u8, addr: u16, memory: &mem::Memory) -> bool {
    let reg: u8 = memory.read_byte(addr);
    (reg >> bit) & 0b1 == 0b1
}

fn set_bit(bit: u8, addr: u16, memory: &mut mem::Memory) {
    let reg: u8 = memory.read_byte(addr);
    let new: u8 = reg | (1 << bit);
    memory.write_byte(addr, new);
}

fn unset_bit(bit: u8, addr: u16, memory: &mut mem::Memory) {
    let reg: u8 = memory.read_byte(addr);
    let new: u8 = reg & !(1 << bit);
    memory.write_byte(addr, new);
}

pub fn is_requested(interrupt: Interrupt, memory: &mem::Memory) -> bool {
    is_set_bit(bit(interrupt), IF_REGISTER_ADDR, memory)
}

pub fn request(interrupt: Interrupt, memory: &mut mem::Memory) {
    set_bit(bit(interrupt), IF_REGISTER_ADDR, memory);
}

pub fn remove_request(interrupt: Interrupt, memory: &mut mem::Memory) {
    unset_bit(bit(interrupt), IF_REGISTER_ADDR, memory);
}

pub fn next_request(memory: &mem::Memory) -> Option<Interrupt> {
    //order of priority
    for bit in 0..5 {
        let interrupt: Interrupt = from_bit(bit);
        if is_requested(interrupt, memory) {
            return Some(interrupt);
        }
    }
    None
}

pub fn is_enabled(interrupt: Interrupt, memory: &mem::Memory) -> bool {
    is_set_bit(bit(interrupt), IE_REGISTER_ADDR, memory)
}

pub fn enable(interrupt: Interrupt, memory: &mut mem::Memory) {
    set_bit(bit(interrupt), IE_REGISTER_ADDR, memory);
}

pub fn disable(interrupt: Interrupt, memory: &mut mem::Memory) {
    unset_bit(bit(interrupt), IE_REGISTER_ADDR, memory);
}

pub fn disable_all(memory: &mut mem::Memory) {
    memory.write_byte(IE_REGISTER_ADDR, 0x00);
}

pub fn enable_all(memory: &mut mem::Memory) {
    memory.write_byte(IE_REGISTER_ADDR, 0x01);
}
