use super::super::mem::mem;
use cpu::interrupt;
use cpu::consts;
use cpu::ioregister;
use std::thread;
use std::{time, fmt};

struct Event {
    cycles_counter: u32,
    cycles_rate: u32, //rate at which the event should happen
    cycles_duration: Option<u32>, //duration of the event, that is, number of cycles until the cycles counter starts again.
    cycles_duration_counter: u32,
    on_event: bool,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "cycles counter: {}\
               \ncycles rate {}\
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

struct ScreenRefreshEvent {
    screen_refresh: Event,
    vblank_event: Event,
    current_mode: u8,
    current_duration_cycles: u32, //counter for the current mode duration in cycles.
}

impl fmt::Display for ScreenRefreshEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\
               Screen Refresh Event ##########\n\
               {}\n\
               ---------------------\n\
               VBlank:\n{}\n", self.screen_refresh, self.vblank_event)
    }
}

impl ScreenRefreshEvent {
    pub fn new() -> ScreenRefreshEvent {
        ScreenRefreshEvent {
            screen_refresh: Event::new(consts::SCREEN_REFRESH_RATE_CYCLES, Some(consts::SCREEN_REFRESH_DURATION_CYCLES)),
            vblank_event: Event::new(consts::VBLANK_INTERRUPT_RATE_CYCLES, Some(consts::STAT_MODE_1_DURATION_CYCLES)),
            current_mode: 0b0,
            current_duration_cycles: 0,
        }
    }

    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        if self.vblank_event.update(cycles) {
            interrupt::request(interrupt::Interrupt::VBlank, memory);
            self.current_duration_cycles = 0;
            ioregister::update_stat_reg_mode_flag(0b01, memory);
            self.current_mode = 0b10; //maybe it is not necessary to do this. If the sync is correct, current_mode should already be 0b10.
        }
        self.screen_refresh.update(cycles);
        if self.screen_refresh.on_event && !self.vblank_event.on_event {
            self.current_duration_cycles += cycles;
            match self.current_mode {
                0b00 => {
                    if self.current_duration_cycles >= consts::STAT_MODE_0_DURATION_CYCLES {
                        self.current_mode = 0b10;
                        self.current_duration_cycles = 0;
                    }
                },
                0b10 => {
                    if self.current_duration_cycles >= consts::STAT_MODE_2_DURATION_CYCLES {
                        self.current_mode = 0b11;
                        self.current_duration_cycles = 0;
                    }
                },
                0b11 => {
                    if self.current_duration_cycles >= consts::STAT_MODE_3_DURATION_CYCLES {
                        self.current_mode = 0b00;
                        self.current_duration_cycles = 0;
                    }
                },
                _ => unreachable!(),
            }
            ioregister::update_stat_reg_mode_flag(self.current_mode, memory);
        }
    }
}

pub struct Timer {
    div_event: Event,
    tima_event: Event,
    screen_refresh_event: ScreenRefreshEvent,
    frame_rate_event: Event,
    timer_started: bool,
}

impl Timer {
    pub fn new() -> Timer {
        //TODO fix durations.
        Timer {
            div_event: Event::new(consts::DIV_REGISTER_UPDATE_RATE_CYCLES, None),
            tima_event: Event::new(0, None),
            screen_refresh_event: ScreenRefreshEvent::new(),
            frame_rate_event: Event::new(cycles_from_hz(60), None),
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

        self.screen_refresh_event.update(cycles, memory);

        ioregister::lcdc_stat_interrupt(memory); //verifies and request LCDC interrupt

        if self.frame_rate_event.update(cycles) {
            //TODO adjust duration to consider elapsed time since the last frame.
            thread::sleep(time::Duration::new(0, 16666666)); //~1/60 seconds
        }
    }

    pub fn events_to_str(&self) -> String {
        let line = "---------------------\n";
        let div = format!("DIV #########\n{}\n", self.div_event);
        let tima = format!("TIMA #########\n{}\n", self.tima_event);
        let screen = format!("{}", self.screen_refresh_event);
        let frame_rate = format!("FrameRate #########\n{}", self.frame_rate_event);

        (div + line + &tima + line + &screen + line + &frame_rate).to_owned()
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
