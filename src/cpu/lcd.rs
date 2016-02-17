use super::super::mem::mem;
use cpu::{interrupt, consts, ioregister, timer};
use std::fmt;

pub struct ScreenRefreshEvent {
    screen_refresh: timer::Event,
    vblank_event: timer::Event,
    ly_register: ioregister::LyRegister,
    current_mode: u8,
    current_duration_cycles: u32, //counter for the current mode duration in cycles.
}

impl fmt::Display for ScreenRefreshEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\
               Screen Refresh timer::Event ##########\n\
               {}\n\
               ---------------------\n\
               VBlank:\n{}\n", self.screen_refresh, self.vblank_event)
    }
}

impl ScreenRefreshEvent {
    pub fn new() -> ScreenRefreshEvent {
        ScreenRefreshEvent {
            screen_refresh: timer::Event::new(consts::SCREEN_REFRESH_RATE_CYCLES, Some(consts::SCREEN_REFRESH_DURATION_CYCLES)),
            vblank_event: timer::Event::new(consts::VBLANK_INTERRUPT_RATE_CYCLES, Some(consts::STAT_MODE_1_DURATION_CYCLES)),
            ly_register: ioregister::LyRegister::new(),
            current_mode: 0b0,
            current_duration_cycles: 0,
        }
    }

    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        self.ly_register.update(cycles, memory);
        self.screen_refresh.update(cycles);
        if self.vblank_event.update(cycles) {
            interrupt::request(interrupt::Interrupt::VBlank, memory);
            self.current_duration_cycles = 0;
            ioregister::update_stat_reg_mode_flag(0b01, memory);
            self.current_mode = 0b10; //maybe it is not necessary to do this. If the sync is correct, current_mode should already be 0b10.
        }
        if !self.vblank_event.on_event && self.screen_refresh.on_event {
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
