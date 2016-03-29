use super::super::mem::mem::Memory;
use super::super::cpu::{self, ioregister, interrupt};

use super::super::graphics;
use super::super::graphics::graphics::Graphics;

use peripherals::Peripheral;

#[derive(Copy, Clone, PartialEq)]
enum StatMode {
    HBlank,
    VBlank,
    OAM,
    VRam,
}

impl StatMode {
    fn mode_number(&self) -> u8 {
        match *self {
            StatMode::HBlank => 0b00,
            StatMode::VBlank => 0b01,
            StatMode::OAM    => 0b10,
            StatMode::VRam   => 0b11,
        }
    }

    fn duration(&self) -> u32 {
        match *self {
            StatMode::HBlank => cpu::consts::STAT_MODE_0_DURATION_CYCLES,
            StatMode::VBlank => cpu::consts::STAT_MODE_1_DURATION_CYCLES,
            StatMode::OAM    => cpu::consts::STAT_MODE_2_DURATION_CYCLES,
            StatMode::VRam   => cpu::consts::STAT_MODE_3_DURATION_CYCLES,
        }
    }
}

pub struct LCD {
    curr_stat_mode: StatMode,
    pub graphics: Graphics,
}

impl Default for LCD {
    fn default() -> Self {
        LCD {
            curr_stat_mode: StatMode::OAM,
            graphics: Graphics::default(),
        }
    }
}

impl Peripheral for LCD {
    fn handle_event(&mut self, memory: &mut Memory) {
        match self.curr_stat_mode {
            StatMode::HBlank => {
                let mut ly: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
                ly += 1;
                if ly == graphics::consts::DISPLAY_HEIGHT_PX {
                    self.curr_stat_mode = StatMode::VBlank;
                    if ioregister::LCDCRegister::is_lcd_display_enable(memory) {
                        interrupt::request(interrupt::Interrupt::VBlank, memory);
                    }
                } else {
                    self.curr_stat_mode = StatMode::OAM;
                }
                memory.write_byte(cpu::consts::LY_REGISTER_ADDR, ly);
            },
            StatMode::VBlank => {
                let mut ly: u8 = memory.read_byte(cpu::consts::LY_REGISTER_ADDR);
                if ly == graphics::consts::DISPLAY_HEIGHT_PX + 10 {
                    self.curr_stat_mode = StatMode::OAM;
                    ly = 0;
                } else {
                    ly += 1;
                }
                memory.write_byte(cpu::consts::LY_REGISTER_ADDR, ly);
            },
            StatMode::OAM => {
                self.curr_stat_mode = StatMode::VRam;
                memory.set_access_vram(true);
                memory.set_access_oam(true);
                self.graphics.update(memory);
            },
            StatMode::VRam => {
                self.curr_stat_mode = StatMode::HBlank;
            },
        }
        memory.set_access_vram(true);
        memory.set_access_oam(true);
        //FIXME: actually use can_access_vram/oram instead of always setting to true.
        // self.mem.set_access_vram(gpu_mode <= 2);
        // self.mem.set_access_oam(gpu_mode <= 1);

        ioregister::update_stat_reg_mode_flag(self.curr_stat_mode.mode_number(), memory);
        ioregister::update_stat_reg_coincidence_flag(memory);
        ioregister::lcdc_stat_interrupt(memory);
    }
}

impl LCD {
    pub fn has_entered_vblank(&self, memory: &Memory) -> bool {
        self.curr_stat_mode == StatMode::VBlank &&
            memory.read_byte(cpu::consts::LY_REGISTER_ADDR) == graphics::consts::DISPLAY_HEIGHT_PX
    }
    pub fn stat_mode_duration(&self) -> u32 {
        self.curr_stat_mode.duration()
    }
    pub fn restart(&mut self, memory: &mut Memory) {
        self.curr_stat_mode = StatMode::OAM;
        self.graphics.restart();
        ioregister::update_stat_reg_mode_flag(self.curr_stat_mode.mode_number(), memory);
        memory.set_access_vram(true);
        memory.set_access_oam(false);
    }
}
