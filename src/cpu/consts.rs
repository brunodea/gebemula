/*Interrupt registers*/
pub const IF_REGISTER_ADDR: u16 = 0xFF0F; //interrupt request register
pub const IE_REGISTER_ADDR: u16 = 0xFFFF; //interrupt enable

/*Timer registers*/
pub const TIMA_REGISTER_ADDR: u16 = 0xFF05; //Timer Counter (incremented at a precise rate -- specified by TAC)
pub const TMA_REGISTER_ADDR: u16 = 0xFF06; //Timer Modulo (holds the value to set TIMA for when TIMA overflows)
pub const TAC_REGISTER_ADDR: u16 = 0xFF07; //Timer Control

pub const DIV_REGISTER_ADDR: u16 = 0xFF04; //Divider Register


pub const CPU_FREQUENCY_HZ: u32 = 4194304; //that is, number of cycles per second.

pub const DIV_REGISTER_UPDATE_RATE_HZ: u32 = 16384;
pub const DIV_REGISTER_UPDATE_RATE_CYCLES: u32 = CPU_FREQUENCY_HZ / DIV_REGISTER_UPDATE_RATE_HZ;

pub const VBLANK_INTERRUPT_RATE_HZ: u32 = 60;
pub const VBLANK_INTERRUPT_RATE_CYCLES: u32 = CPU_FREQUENCY_HZ / VBLANK_INTERRUPT_RATE_HZ;
pub const VBLANK_DURATION_CYCLES: u32 = 4560;
