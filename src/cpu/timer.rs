use super::super::mem::mem;

const CPU_FREQUENCY_HZ: u32 = 4194304; //that is, number of cycles per second.

/*Timer registers*/
const TIMA_REGISTER_ADDR: u16 = 0xFF05; //Timer Counter (incremented at a precise rate -- specified by TAC)
const TMA_REGISTER_ADDR: u16 = 0xFF06; //Timer Modulo (holds the value to set TIMA for when TIMA overflows)
const TAC_REGISTER_ADDR: u16 = 0xFF07; //Timer Control

pub fn increment_timer_counter(memory: &mut mem::Memory) {
    let mut tima: u8 = memory.read_byte(TIMA_REGISTER_ADDR);
    if tima == 0xFF {
        //overflows
        tima = memory.read_byte(TMA_REGISTER_ADDR);
    } else {
        tima += 1;
    }
    memory.write_byte(TIMA_REGISTER_ADDR, tima);
}

pub fn timer_stop(memory: &mem::Memory) -> bool {
    (memory.read_byte(TAC_REGISTER_ADDR) >> 2) & 0b1 == 0b0
}

pub fn input_clock(memory: &mem::Memory) -> u32 {
    match memory.read_byte(TAC_REGISTER_ADDR) & 0b11 {
        0b00 => 4096,
        0b01 => 262144,
        0b10 => 65536,
        0b11 => 16384,
        _ => unreachable!(),
    }
}

pub fn cycles_from_hz(rate_hz: u32) -> u32 {
    CPU_FREQUENCY_HZ / rate_hz
}
