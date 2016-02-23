use super::super::mem::mem;
use cpu::{consts, ioregister};

pub struct ScreenRefreshEvent {
    ly_register: ioregister::LYRegister,
    current_mode: u8,
    current_duration_cycles: u32, //counter for the current mode duration in cycles.
}

impl ScreenRefreshEvent {
    pub fn new() -> ScreenRefreshEvent {
        ScreenRefreshEvent {
            ly_register: ioregister::LYRegister::new(),
            current_mode: 0b10,
            current_duration_cycles: 0,
        }
    }

    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        if ioregister::LCDCRegister::is_lcd_display_enable(memory) {
            self.current_duration_cycles += cycles;
            self.ly_register.update(cycles, memory);
            let mut mode_changed: bool = false;
            if ioregister::LYRegister::value(memory) >= 0x90 {
                self.current_mode = 0b01;
                mode_changed = true;
            }
            match self.current_mode {
                0b00 => {
                    if self.current_duration_cycles >= consts::STAT_MODE_0_DURATION_CYCLES {
                        self.current_mode = 0b10;
                        self.current_duration_cycles = 0;
                        mode_changed = true;
                    }
                },
                0b01 => {
                    if ioregister::LYRegister::value(memory) < 0x90 {
                        self.current_mode = 0b10;
                        self.current_duration_cycles = 0;
                        mode_changed = true;
                    }
                },
                0b10 => {
                    if self.current_duration_cycles >= consts::STAT_MODE_2_DURATION_CYCLES {
                        self.current_mode = 0b11;
                        self.current_duration_cycles = 0;
                        mode_changed = true;
                    }
                },
                0b11 => {
                    if self.current_duration_cycles >= consts::STAT_MODE_3_DURATION_CYCLES {
                        self.current_mode = 0b00;
                        self.current_duration_cycles = 0;
                        mode_changed = true;
                    }
                },
                _ => unreachable!(),
            }
            if mode_changed {
                ioregister::update_stat_reg_mode_flag(self.current_mode, memory);
            }
        }
    }
}
