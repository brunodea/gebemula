use super::super::mem::mem;
use cpu::{interrupt, consts};
use std::fmt;

pub struct Event {
    cycles_counter: u32,
    cycles_rate: u32, //rate at which the event should happen
    cycles_duration: Option<u32>, //duration of the event, that is, number of cycles until the cycles counter starts again.
    cycles_duration_counter: u32,
    pub on_event: bool,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "cycles counter: {}\
               \ncycles rate: {}\
               \ncycles duration: {:?}\
               \ncycles duration counter: {}\
               \non event: {:?}", self.cycles_counter, self.cycles_rate, self.cycles_duration,
               self.cycles_duration_counter, self.on_event)
    }
}

impl Event {
    pub fn new(cycles_rate: u32, cycles_duration: Option<u32>) -> Event {
        Event {
            cycles_counter: 0,
            cycles_rate: cycles_rate,
            cycles_duration: cycles_duration,
            cycles_duration_counter: 0,
            on_event: false,
        }
    }

    //return true if event happened.
    pub fn update(&mut self, cycles: u32) -> bool {
        if let Some(duration) = self.cycles_duration {
            if self.on_event {
                self.cycles_duration_counter += cycles;
                if self.cycles_duration_counter >= duration {
                    self.cycles_duration_counter = 0;
                    self.on_event = false;
                }
            }
        } else {
            self.on_event = false
        }

        if !self.on_event {
            self.cycles_counter += cycles;
            if self.cycles_counter >= self.cycles_rate {
                self.cycles_counter = 0;
                self.on_event = true;
                return true;
            }
        }
        false
    }
}

pub struct Timer {
    div_event: Event,
    tima_event: Event,
    timer_started: bool,
}

impl Timer {
    pub fn new() -> Timer {
        //TODO fix durations.
        Timer {
            div_event: Event::new(consts::DIV_REGISTER_UPDATE_RATE_CYCLES, None),
            tima_event: Event::new(0, None),
            timer_started: false,
        }
    }

    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        if self.div_event.update(cycles) {
            let div: u8 = memory.read_byte(consts::DIV_REGISTER_ADDR);
            memory.write_byte(consts::DIV_REGISTER_ADDR, div.wrapping_add(1));
        }

        if !timer_stop(memory) {
            if !self.timer_started {
                self.tima_event.cycles_rate = cycles_from_hz(input_clock(memory));
                self.timer_started = true;
            }
            if self.tima_event.update(cycles) {
                let mut tima: u8 = memory.read_byte(consts::TIMA_REGISTER_ADDR);
                if tima == 0xFF {
                    //overflows
                    tima = memory.read_byte(consts::TMA_REGISTER_ADDR);
                } else {
                    tima += 1;
                }
                memory.write_byte(consts::TIMA_REGISTER_ADDR, tima);
                interrupt::request(interrupt::Interrupt::TimerOverflow, memory);
            }
        } else {
            self.timer_started = false;
        }
    }

    pub fn events_to_str(&self) -> String {
        let line = "---------------------\n";
        let div = format!("DIV #########\n{}\n", self.div_event);
        let tima = format!("TIMA #########\n{}\n", self.tima_event);

        (div + line + &tima).to_owned()
    }
}

#[inline]
pub fn timer_stop(memory: &mem::Memory) -> bool {
    (memory.read_byte(consts::TAC_REGISTER_ADDR) >> 2) & 0b1 == 0b0
}

pub fn input_clock(memory: &mem::Memory) -> u32 {
    match memory.read_byte(consts::TAC_REGISTER_ADDR) & 0b11 {
        0b00 => 4096,
        0b01 => 262144,
        0b10 => 65536,
        0b11 => 16384,
        _ => unreachable!(),
    }
}

#[inline]
pub fn cycles_from_hz(rate_hz: u32) -> u32 {
    consts::CPU_FREQUENCY_HZ / rate_hz
}
