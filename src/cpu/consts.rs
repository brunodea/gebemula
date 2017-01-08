// Interrupt registers
pub const IF_REGISTER_ADDR: u16 = 0xFF0F; //interrupt request register
pub const IE_REGISTER_ADDR: u16 = 0xFFFF; //interrupt enable
/*Timer registers*/
// Timer Counter (incremented at a precise rate -- specified by TAC)
pub const TIMA_REGISTER_ADDR: u16 = 0xFF05;
// Timer Modulo (holds the value to set TIMA for when TIMA overflows)
pub const TMA_REGISTER_ADDR: u16 = 0xFF06;
// Timer Control
pub const TAC_REGISTER_ADDR: u16 = 0xFF07;
// Divider Register
pub const DIV_REGISTER_ADDR: u16 = 0xFF04;
pub const TIMER_INTERNAL_COUNTER_ADDR: u16 = 0xFF03;

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

pub const CPU_FREQUENCY_HZ: u32 = 4194304; //that is, number of cycles per second.

pub const DMA_DURATION_CYCLES: u32 = CPU_FREQUENCY_HZ / (1000000 / 160);
pub const CGB_DMA_DURATION_CYCLES: u32 = 8; //for a transfer of length 0x10.

pub const STAT_MODE_0_DURATION_CYCLES: u32 = 201;
pub const STAT_MODE_1_DURATION_CYCLES: u32 = 456;
pub const STAT_MODE_2_DURATION_CYCLES: u32 = 77;
pub const STAT_MODE_3_DURATION_CYCLES: u32 = 169;

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
