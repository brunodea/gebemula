use super::super::mem::mem;
use cpu::interrupt;
use std::thread;
use std::time;

const CPU_FREQUENCY_HZ: u32 = 4194304; //that is, number of cycles per second.

/*Timer registers*/
const TIMA_REGISTER_ADDR: u16 = 0xFF05; //Timer Counter (incremented at a precise rate -- specified by TAC)
const TMA_REGISTER_ADDR: u16 = 0xFF06; //Timer Modulo (holds the value to set TIMA for when TIMA overflows)
const TAC_REGISTER_ADDR: u16 = 0xFF07; //Timer Control

pub const DIV_REGISTER_ADDR: u16 = 0xFF04; //Divider Register
const DIV_REGISTER_UPDATE_RATE_HZ: u32 = 16384;
const DIV_REGISTER_UPDATE_RATE_CYCLES: u32 = CPU_FREQUENCY_HZ / DIV_REGISTER_UPDATE_RATE_HZ;

const VBLANK_INTERRUPT_RATE_HZ: u32 = 60;
const VBLANK_INTERRUPT_RATE_CYCLES: u32 = CPU_FREQUENCY_HZ / VBLANK_INTERRUPT_RATE_HZ;

pub struct Timer {
    div_cycles_counter: u32,
    tima_cycles_counter: u32,
    tima_rate_cycles: u32,
    vblank_interrupt_cycles_counter: u32,
    frame_rate_cycles: u32,
    frame_rate_cycles_counter: u32,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div_cycles_counter: 0,
            tima_cycles_counter: 0,
            tima_rate_cycles: 0,
            vblank_interrupt_cycles_counter: 0,
            frame_rate_cycles_counter: 0,
            frame_rate_cycles: cycles_from_hz(60), //default: 60hz
        }
    }

    pub fn init(&mut self, memory: &mem::Memory) {
        self.tima_rate_cycles = cycles_from_hz(input_clock(memory));
    }

    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        self.div_cycles_counter += cycles;
        if self.div_cycles_counter >= DIV_REGISTER_UPDATE_RATE_CYCLES {
            let div: u8 = memory.read_byte(DIV_REGISTER_ADDR);
            memory.write_byte(DIV_REGISTER_ADDR, div.wrapping_add(1));
            self.div_cycles_counter = 0;
        }

        self.tima_cycles_counter += cycles;
        if self.tima_cycles_counter >= self.tima_rate_cycles {
            let mut tima: u8 = memory.read_byte(TIMA_REGISTER_ADDR);
            if tima == 0xFF {
                //overflows
                tima = memory.read_byte(TMA_REGISTER_ADDR);
            } else {
                tima += 1;
            }
            memory.write_byte(TIMA_REGISTER_ADDR, tima);
            interrupt::request(interrupt::Interrupt::TimerOverflow, memory);
            self.tima_cycles_counter = 0;
        }

        self.vblank_interrupt_cycles_counter += cycles;
        if self.vblank_interrupt_cycles_counter >= VBLANK_INTERRUPT_RATE_CYCLES {
            interrupt::request(interrupt::Interrupt::VBlank, memory);
            self.vblank_interrupt_cycles_counter = 0;
        }

        self.frame_rate_cycles_counter += cycles;
        if self.frame_rate_cycles_counter >= self.frame_rate_cycles {
            //TODO adjust duration to consider elapsed time since the last frame.
            thread::sleep(time::Duration::new(0, 16666666)); //~1/60 seconds
            self.frame_rate_cycles_counter = 0;
        }
    }
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
