use cpu::interrupt;
use super::super::mem;

pub const CPU_FREQUENCY_HZ: u32 = 4194304; //that is, number of cycles per second.
pub const DMA_DURATION_CYCLES: u32 = CPU_FREQUENCY_HZ / (1000000 / 160);
pub const CGB_DMA_DURATION_CYCLES: u32 = 8; //for a transfer of length 0x10.

// Divider Register
pub const TIMER_INTERNAL_COUNTER_ADDR: u16 = 0xFF03;
pub const DIV_REGISTER_ADDR: u16 = 0xFF04;
/*Timer registers*/
// Timer Counter (incremented at a precise rate -- specified by TAC)
pub const TIMA_REGISTER_ADDR: u16 = 0xFF05;
// Timer Modulo (holds the value to set TIMA for when TIMA overflows)
pub const TMA_REGISTER_ADDR: u16 = 0xFF06;
// Timer Control
pub const TAC_REGISTER_ADDR: u16 = 0xFF07;
// Interrupt registers
pub const IF_REGISTER_ADDR: u16 = 0xFF0F; //interrupt request register
pub const IE_REGISTER_ADDR: u16 = 0xFFFF; //interrupt enable

// LCD registers
pub const STAT_REGISTER_ADDR: u16 = 0xFF41; //LCDC Status
pub const LY_REGISTER_ADDR: u16 = 0xFF44;
pub const LYC_REGISTER_ADDR: u16 = 0xFF45;
pub const LCDC_REGISTER_ADDR: u16 = 0xFF40;
pub const DMA_REGISTER_ADDR: u16 = 0xFF46;

// Graphics registers
pub const BGP_REGISTER_ADDR: u16 = 0xFF47;
pub const SCY_REGISTER_ADDR: u16 = 0xFF42;
pub const SCX_REGISTER_ADDR: u16 = 0xFF43;
pub const WY_REGISTER_ADDR: u16 = 0xFF4A;
pub const WX_REGISTER_ADDR: u16 = 0xFF4B;
pub const OBP_0_REGISTER_ADDR: u16 = 0xFF48;
pub const OBP_1_REGISTER_ADDR: u16 = 0xFF49;

pub const JOYPAD_REGISTER_ADDR: u16 = 0xFF00;
// CGB's graphics registers
pub const BGPI_REGISTER_ADDR: u16 = 0xFF68;
pub const BGPD_REGISTER_ADDR: u16 = 0xFF69;
pub const OBPI_REGISTER_ADDR: u16 = 0xFF6A;
pub const OBPD_REGISTER_ADDR: u16 = 0xFF6B;

// CGB's DMA registers
pub const HDMA1_REGISTER_ADDR: u16 = 0xFF51;
pub const HDMA2_REGISTER_ADDR: u16 = 0xFF52;
pub const HDMA3_REGISTER_ADDR: u16 = 0xFF53;
pub const HDMA4_REGISTER_ADDR: u16 = 0xFF54;
pub const HDMA5_REGISTER_ADDR: u16 = 0xFF55;

pub const VBK_REGISTER_ADDR: u16 = 0xFF4F;
pub const SVBK_REGISTER_ADDR: u16 = 0xFF70;
pub const KEY1_REGISTER_ADDR: u16 = 0xFF4D;

pub fn update_stat_reg_coincidence_flag(memory: &mut mem::Memory) {
    let coincidence_flag = if memory.read_byte(LY_REGISTER_ADDR) == memory.read_byte(LYC_REGISTER_ADDR) {
        0b100
    } else {
        0b000
    };
    let new_stat = (memory.read_byte(STAT_REGISTER_ADDR) & 0b1111_1011) |
                       coincidence_flag;
    memory.write_byte(STAT_REGISTER_ADDR, new_stat);
}

pub fn update_stat_reg_mode_flag(mode_flag: u8, memory: &mut mem::Memory) {
    let new_stat = (memory.read_byte(STAT_REGISTER_ADDR) & 0b1111_1100) |
                       (mode_flag & 0b11);
    memory.write_byte(STAT_REGISTER_ADDR, new_stat);
}

pub fn lcdc_stat_interrupt(memory: &mut mem::Memory) {
    let reg = memory.read_byte(STAT_REGISTER_ADDR);
    if ((((reg >> 6) & 0b1) == 0b1) && ((reg >> 2) & 0b1 == 0b1)) ||
       ((((reg >> 5) & 0b1) == 0b1) && (reg & 0b11 == 0b10)) ||
       ((((reg >> 4) & 0b1) == 0b1) && (reg & 0b11 == 0b01)) ||
       ((((reg >> 3) & 0b1) == 0b1) && (reg & 0b11 == 0b00)) {

        interrupt::request(interrupt::Interrupt::LCDC, memory);
    }
}

// returns the number of cycles.
pub fn dma_transfer(start_address: u8, memory: &mut mem::Memory) -> u32 {
    let source_address = (start_address as u16) << 8;
    // only 0xA0 bytes are transfered, which is the OAM data size.
    for i in 0x0..0xA0 {
        // 0XFE00 = start of OAM address.
        let byte = memory.read_byte(source_address + i);
        memory.write_byte(0xFE00 + i, byte);
    }

    DMA_DURATION_CYCLES
}

// returns None if finished the transfer, otherwise, returns the number of cycles.
pub fn cgb_dma_transfer(memory: &mut mem::Memory) -> Option<u32> {
    let hdma1 = memory.read_byte(HDMA1_REGISTER_ADDR) as u16;
    let hdma2 = memory.read_byte(HDMA2_REGISTER_ADDR) as u16 & 0b0000; //ignore lower 4 bits.
    let hdma3 = ((memory.read_byte(HDMA3_REGISTER_ADDR) & 0b0001_1111) | 0b1000_0000) as u16; //ignore upper 3 bits (always VRAM).
    let hdma4 = memory.read_byte(HDMA4_REGISTER_ADDR) as u16 & 0b0000; //ignore lower 4 bits.
    let hdma5 = memory.read_byte(HDMA5_REGISTER_ADDR);

    let src_start = (hdma1 << 8) | hdma2;
    let dst_start = (hdma3 << 8) | hdma4;
    //let mut len = ((hdma5 & 0b0111_1111) / 0x10).wrapping_sub(1);
    let mut len = ((hdma5 & 0b0111_1111) << 4) + 0x10;
    // TODO: make sure it is len that has to be 0xFF and not the 7 bits of hdma5.
    if len == 0xFF {
        memory.write_byte(HDMA5_REGISTER_ADDR, len);
        None
    } else {
        let mut mode = hdma5 >> 7;

        if mode == 0b1 {
            //H-Blank DMA
            len = len.wrapping_sub(0x10);
            mode = if len == 0 { 0 } else { 1 };
            //TODO: make sure it is okay to change the length from here.
            memory.write_byte(HDMA5_REGISTER_ADDR, (mode << 7) | len);
            len = 0x10;
        }

        let mut cycles = 0;
        for i in 0x0..(len as u16) {
            let byte = memory.read_byte(src_start + i);
            memory.write_byte(dst_start + i, byte);
            // add 8 cycles every 0x10 addresses.
            if (i + 1) % 0x10 == 0 {
                cycles += CGB_DMA_DURATION_CYCLES;
            }
        }

        Some(cycles)
    }
}

pub struct LCDCRegister;

impl LCDCRegister {
    fn is_bit_set(bit: u8, memory: &mem::Memory) -> bool {
        (memory.read_byte(LCDC_REGISTER_ADDR) >> bit) & 0b1 == 0b1
    }
    pub fn disable_lcd(memory: &mut mem::Memory) {
        let val = memory.read_byte(LCDC_REGISTER_ADDR);
        memory.write_byte(LCDC_REGISTER_ADDR, val & (0b0111_1111));
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

// pixel_data has to have a value from 0 to 3.
pub fn bg_window_palette(pixel_data: u8, memory: &mem::Memory) -> u8 {
    (memory.read_byte(BGP_REGISTER_ADDR) >> (pixel_data * 2)) & 0b11
}
pub fn sprite_palette(obp0: bool, pixel_data: u8, memory: &mem::Memory) -> u8 {
    let addr = if obp0 {
        OBP_0_REGISTER_ADDR
    } else {
        OBP_1_REGISTER_ADDR
    };
    (memory.read_byte(addr) >> (pixel_data * 2)) & 0b11
}

pub fn joypad_buttons(memory: &mem::Memory) -> u8 {
    memory.read_byte(JOYPAD_REGISTER_ADDR) & 0x0F
}

pub fn joypad_buttons_selected(memory: &mem::Memory) -> bool {
    memory.read_byte(JOYPAD_REGISTER_ADDR) & 0b0010_0000 == 0b0
}

pub fn joypad_set_buttons(new_buttons: u8, memory: &mut mem::Memory) {
    let mut buttons = memory.read_byte(JOYPAD_REGISTER_ADDR);
    buttons = (buttons & 0b0011_0000) | new_buttons;
    memory.write_byte(JOYPAD_REGISTER_ADDR, buttons);
}
