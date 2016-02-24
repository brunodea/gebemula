use super::super::mem::mem;
use super::super::graphics;
use cpu::{consts, ioregister, interrupt};

pub struct ScreenRefreshEvent {
    current_mode: u8,
    current_duration_cycles: u32, //counter for the current mode duration in cycles.
    pub is_scan_line: bool,
    pub is_display_buffer: bool,
}

impl ScreenRefreshEvent {
    pub fn new() -> ScreenRefreshEvent {
        ScreenRefreshEvent {
            current_mode: 0b10,
            current_duration_cycles: 0,
            is_scan_line: false,
            is_display_buffer: false,
        }
    }

    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        self.is_scan_line = false;
        self.is_display_buffer = false;
        self.current_duration_cycles += cycles;
        let mut mode_changed: bool = false;
        match self.current_mode {
            0b00 => {
                if self.current_duration_cycles >= consts::STAT_MODE_0_DURATION_CYCLES {
                    let mut ly: u8 = memory.read_byte(consts::LY_REGISTER_ADDR);
                    ly += 1;
                    if ly == graphics::consts::DISPLAY_HEIGHT_PX {
                        //enter vblank
                        self.current_mode = 0b01;
                        interrupt::request(interrupt::Interrupt::VBlank, memory);
                        self.is_display_buffer = true;
                    } else {
                        self.current_mode = 0b10;
                    }
                    self.current_duration_cycles = 0;
                    mode_changed = true;
                    memory.write_byte(consts::LY_REGISTER_ADDR, ly);
                }
            },
            0b01 => {
                if self.current_duration_cycles >= consts::STAT_MODE_1_DURATION_CYCLES {
                    self.current_duration_cycles = 0;
                    let mut ly: u8 = memory.read_byte(consts::LY_REGISTER_ADDR);
                    ly += 1;
                    if ly > graphics::consts::DISPLAY_HEIGHT_PX + 9 {
                        self.current_mode = 0b10;
                        ly = 0;
                    }
                    mode_changed = true;
                    memory.write_byte(consts::LY_REGISTER_ADDR, ly);
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
                    self.is_scan_line = true;
                }
            },
            _ => unreachable!(),
        }
        if mode_changed {
            ioregister::update_stat_reg_mode_flag(self.current_mode, memory);
            ioregister::lcdc_stat_interrupt(memory); //verifies and request LCDC interrupt
        }
    }
}
