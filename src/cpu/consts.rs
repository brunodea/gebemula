/*Interrupt registers*/
pub const IF_REGISTER_ADDR: u16 = 0xFF0F; //interrupt request register
pub const IE_REGISTER_ADDR: u16 = 0xFFFF; //interrupt enable
/*Timer registers*/
pub const TIMA_REGISTER_ADDR: u16 = 0xFF05; //Timer Counter (incremented at a precise rate -- specified by TAC)
pub const TMA_REGISTER_ADDR: u16 = 0xFF06; //Timer Modulo (holds the value to set TIMA for when TIMA overflows)
pub const TAC_REGISTER_ADDR: u16 = 0xFF07; //Timer Control
pub const DIV_REGISTER_ADDR: u16 = 0xFF04; //Divider Register

/*LCD registers*/
pub const STAT_REGISTER_ADDR: u16 = 0xFF41; //LCDC Status
pub const LY_REGISTER_ADDR: u16 = 0xFF44;
pub const LYC_REGISTER_ADDR: u16 = 0xFF45;

pub const CPU_FREQUENCY_HZ: u32 = 4194304; //that is, number of cycles per second.

pub const DIV_REGISTER_UPDATE_RATE_HZ: u32 = 16384;
pub const DIV_REGISTER_UPDATE_RATE_CYCLES: u32 = CPU_FREQUENCY_HZ / DIV_REGISTER_UPDATE_RATE_HZ;

pub const VBLANK_INTERRUPT_RATE_HZ: u32 = 60;
pub const VBLANK_INTERRUPT_RATE_CYCLES: u32 = CPU_FREQUENCY_HZ / VBLANK_INTERRUPT_RATE_HZ;

pub const STAT_MODE_0_DURATION_CYCLES: u32 = 201; //201~207 - H-Blank period
pub const STAT_MODE_1_DURATION_CYCLES: u32 = 4560; //V-Blank period
pub const STAT_MODE_2_DURATION_CYCLES: u32 = 77; //77~83 oam being used
pub const STAT_MODE_3_DURATION_CYCLES: u32 = 169; //169~175 oam and ram being used

pub const SCREEN_REFRESH_RATE_CYCLES: u32 = 70224;
pub const SCREEN_REFRESH_DURATION_CYCLES: u32 = 
    STAT_MODE_0_DURATION_CYCLES + STAT_MODE_1_DURATION_CYCLES +
    STAT_MODE_2_DURATION_CYCLES + STAT_MODE_3_DURATION_CYCLES;

pub const LY_REGISTER_UPDATE_RATE_CYCLES: u32 = STAT_MODE_1_DURATION_CYCLES / 10; //divide by 10, because it will only update during vblank from 144 to 153 (0x90 to 0x99)
