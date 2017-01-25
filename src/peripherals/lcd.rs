use super::super::mem::Memory;
use super::super::cpu::{self, ioregister, interrupt};
use super::super::graphics::{self, Graphics};

const STAT_MODE_0_DURATION_CYCLES: u32 = 201;
const STAT_MODE_1_DURATION_CYCLES: u32 = 456;
const STAT_MODE_2_DURATION_CYCLES: u32 = 77;
const STAT_MODE_3_DURATION_CYCLES: u32 = 169;

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
            StatMode::HBlank => STAT_MODE_0_DURATION_CYCLES,
            StatMode::VBlank => STAT_MODE_1_DURATION_CYCLES,
            StatMode::OAM    => STAT_MODE_2_DURATION_CYCLES,
            StatMode::VRam   => STAT_MODE_3_DURATION_CYCLES,
        }
    }
}

pub struct LCD {
    curr_stat_mode: StatMode,
    pub graphics: Graphics,
    cgb_dma_requested: bool,
}

impl Default for LCD {
    fn default() -> Self {
        LCD {
            curr_stat_mode: StatMode::OAM,
            graphics: Graphics::default(),
            cgb_dma_requested: false,
        }
    }
}

impl LCD {
    pub fn has_entered_vblank(&self, memory: &Memory) -> bool {
        self.curr_stat_mode == StatMode::VBlank &&
            memory.read_byte(ioregister::LY_REGISTER_ADDR) == graphics::consts::DISPLAY_HEIGHT_PX
    }
    pub fn request_cgb_dma_transfer(&mut self) {
        self.cgb_dma_requested = true;
    }
    pub fn stat_mode_duration(&self) -> u32 {
        self.curr_stat_mode.duration()
    }
    pub fn restart(&mut self, memory: &mut Memory) {
        self.curr_stat_mode = StatMode::OAM;
        self.cgb_dma_requested = false;
        self.graphics.restart();
        ioregister::update_stat_reg_mode_flag(self.curr_stat_mode.mode_number(), memory);
        memory.set_access_vram(true);
        memory.set_access_oam(false);
    }
    pub fn set_color(&mut self) {
        self.graphics.set_color();
    }

    // return cycles (because of cgb dma transfer). TODO: find a better way.
    pub fn stat_mode_change(&mut self, memory: &mut Memory) -> u32 {
        let mut cycles = 0;
        match self.curr_stat_mode {
            StatMode::HBlank => {
                let mut ly = memory.read_byte(ioregister::LY_REGISTER_ADDR);
                ly += 1;
                if ly == graphics::consts::DISPLAY_HEIGHT_PX {
                    self.curr_stat_mode = StatMode::VBlank;
                    if ioregister::LCDCRegister::is_lcd_display_enable(memory) {
                        interrupt::request(interrupt::Interrupt::VBlank, memory);
                    }
                } else {
                    self.curr_stat_mode = StatMode::OAM;
                }
                memory.write_byte(ioregister::LY_REGISTER_ADDR, ly);
            },
            StatMode::VBlank => {
                let mut ly = memory.read_byte(ioregister::LY_REGISTER_ADDR);
                if ly == graphics::consts::DISPLAY_HEIGHT_PX + 10 {
                    self.curr_stat_mode = StatMode::OAM;
                    ly = 0;
                } else {
                    ly += 1;
                }
                memory.write_byte(ioregister::LY_REGISTER_ADDR, ly);
            },
            StatMode::OAM => {
                self.curr_stat_mode = StatMode::VRam;
                memory.set_access_vram(true);
                memory.set_access_oam(true);
                self.graphics.update(memory);
            },
            StatMode::VRam => {
                self.curr_stat_mode = StatMode::HBlank;
                if self.cgb_dma_requested {
                    if let Some(tmp) = ioregister::cgb_dma_transfer(memory) {
                        cycles = tmp;
                    } else {
                        self.cgb_dma_requested = false;
                    }
                }
            },
        }
        memory.set_access_vram(true);
        memory.set_access_oam(true);
        //FIXME: actually use can_access_vram/oram instead of always setting to true.
        //memory.set_access_vram(self.curr_stat_mode != StatMode::VRam);
        //memory.set_access_oam(self.curr_stat_mode == StatMode::VBlank || self.curr_stat_mode == StatMode::HBlank);


        ioregister::update_stat_reg_mode_flag(self.curr_stat_mode.mode_number(), memory);
        ioregister::update_stat_reg_coincidence_flag(memory);
        ioregister::lcdc_stat_interrupt(memory);

        cycles
    }
}
