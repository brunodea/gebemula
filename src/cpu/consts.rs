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
pub const LCDC_REGISTER_ADDR: u16 = 0xFF40;
pub const DMA_REGISTER_ADDR: u16 = 0xFF46;

/*Graphics registers*/
pub const BGP_REGISTER_ADDR: u16 = 0xFF47;
pub const SCY_REGISTER_ADDR: u16 = 0xFF42;
pub const SCX_REGISTER_ADDR: u16 = 0xFF43;
pub const WY_REGISTER_ADDR: u16 = 0xFF4A;
pub const WX_REGISTER_ADDR: u16 = 0xFF4B;
pub const OBP_0_REGISTER_ADDR: u16 = 0xFF48;
pub const OBP_1_REGISTER_ADDR: u16 = 0xFF49;

pub const CPU_FREQUENCY_HZ: u32 = 4194304; //that is, number of cycles per second.

pub const DIV_REGISTER_UPDATE_RATE_HZ: u32 = 16384;
pub const DIV_REGISTER_UPDATE_RATE_CYCLES: u32 = CPU_FREQUENCY_HZ / DIV_REGISTER_UPDATE_RATE_HZ;

pub const DMA_DURATION_CYCLES: u32 = CPU_FREQUENCY_HZ / (1000000/160);

pub const STAT_MODE_0_DURATION_CYCLES: u32 = 201;
pub const STAT_MODE_1_DURATION_CYCLES: u32 = 456;
pub const STAT_MODE_2_DURATION_CYCLES: u32 = 77;
pub const STAT_MODE_3_DURATION_CYCLES: u32 = 169;
