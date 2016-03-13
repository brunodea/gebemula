use cpu::consts;
use cpu::interrupt;
use super::super::mem::mem;

pub fn update_stat_reg_coincidence_flag(memory: &mut mem::Memory) {
    let coincidence_flag: u8 = if stat_reg_coincidence_flag(memory) { 0b100 } else { 0b000 };
    let new_stat: u8 =
        (memory.read_byte(consts::STAT_REGISTER_ADDR) & 0b1111_1011) | coincidence_flag;
    memory.write_byte(consts::STAT_REGISTER_ADDR, new_stat);
}

pub fn update_stat_reg_mode_flag(mode_flag: u8, memory: &mut mem::Memory) {
    let new_stat: u8 =
        (memory.read_byte(consts::STAT_REGISTER_ADDR) & 0b1111_1100) | mode_flag;
    memory.write_byte(consts::STAT_REGISTER_ADDR, new_stat);
}

pub fn stat_reg_coincidence_flag(memory: &mem::Memory) -> bool {
    memory.read_byte(consts::LY_REGISTER_ADDR) == memory.read_byte(consts::LYC_REGISTER_ADDR)
}

pub fn lcdc_stat_interrupt(memory: &mut mem::Memory) {
    let reg: u8 = memory.read_byte(consts::STAT_REGISTER_ADDR);
    if  ((((reg >> 6) & 0b1) == 0b1) && ((reg >> 2) & 0b1 == 0b1)) ||
        ((((reg >> 5) & 0b1) == 0b1) && (reg & 0b11 == 0b10)) ||
        ((((reg >> 4) & 0b1) == 0b1) && (reg & 0b11 == 0b01)) ||
        ((((reg >> 3) & 0b1) == 0b1) && (reg & 0b11 == 0b00)) {

        interrupt::request(interrupt::Interrupt::LCDC, memory);
    }
}

//TODO change hex by consts
pub fn dma_transfer(start_address: u8, memory: &mut mem::Memory) {
    let source_address: u16 = (start_address as u16) << 8;
    match source_address {
        //internal rom or ram
        0x0000 ... 0xF19F => {
            //only 0xA0 bytes are transfered, which is the OAM data size.
            for i in 0x0..0xA0 {
                //0XFE00 = start of OAM address.
                let byte: u8 = memory.read_byte(source_address + i);
                memory.write_byte(0xFE00 + i, byte);
            }
        },
        _ => unreachable!(),
    }
}

pub struct LCDCRegister;

impl LCDCRegister {
    fn is_bit_set(bit: u8, memory: &mem::Memory) -> bool {
        (memory.read_byte(consts::LCDC_REGISTER_ADDR) >> bit) & 0b1 == 0b1
    }
    pub fn disable_lcd(memory: &mut mem::Memory) {
        let val: u8 = memory.read_byte(consts::LCDC_REGISTER_ADDR);
        memory.write_byte(consts::LCDC_REGISTER_ADDR, val & (0b0111_1111));
    }
    pub fn is_lcd_display_enable(memory: &mem::Memory) -> bool {
        LCDCRegister::is_bit_set(7, memory)
    }
    pub fn is_window_tile_map_display_normal(memory: &mem::Memory) -> bool {
        !LCDCRegister::is_bit_set(6, memory)
    }
    pub fn is_window_display_on(memory: &mem::Memory) -> bool {
        LCDCRegister::is_bit_set(5, memory)
    }
    pub fn is_tile_data_0(memory: &mem::Memory) -> bool {
        !LCDCRegister::is_bit_set(4, memory)
    }
    pub fn is_bg_tile_map_display_normal(memory: &mem::Memory) -> bool {
        !LCDCRegister::is_bit_set(3, memory)
    }
    pub fn is_sprite_8_16_on(memory: &mem::Memory) -> bool {
        LCDCRegister::is_bit_set(2, memory)
    }
    pub fn is_sprite_display_on(memory: &mem::Memory) -> bool {
        LCDCRegister::is_bit_set(1, memory)
    }
    pub fn is_bg_window_display_on(memory: &mem::Memory) -> bool {
        LCDCRegister::is_bit_set(0, memory)
    }
}

//pixel_data has to have a value from 0 to 3.
pub fn bg_window_palette(pixel_data: u8, memory: &mem::Memory) -> u8 {
    (memory.read_byte(consts::BGP_REGISTER_ADDR) >> (pixel_data * 2)) & 0b11
}
pub fn sprite_palette(obp0: bool, pixel_data: u8, memory: &mem::Memory) -> u8 {
    let addr: u16 =
        if obp0 {
            consts::OBP_0_REGISTER_ADDR
        } else {
            consts::OBP_1_REGISTER_ADDR
        };
    (memory.read_byte(addr) >> (pixel_data * 2)) & 0b11
}

pub fn joypad_buttons_selected(memory: &mem::Memory) -> bool {
    memory.read_byte(consts::JOYPAD_REGISTER_ADDR) & 0b0010_0000 == 0b0
}

pub fn joypad_set_buttons(new_buttons: u8, memory: &mut mem::Memory) {
    let mut buttons: u8 = memory.read_byte(consts::JOYPAD_REGISTER_ADDR);
    buttons = (buttons & 0b0011_0000) | new_buttons;
    memory.write_byte(consts::JOYPAD_REGISTER_ADDR, buttons);
}
