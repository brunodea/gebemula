use std::thread;
use std::time::Duration;

pub const CPU_CLOCK_SPEED_NS: f32 = 238.4; //nano seconds per cycle.
pub const H_SYNC_LINE_CYCLES: u32 = 456; //per line
pub const V_SYNC_FRAME_CYCLES: u32 = 70224; //per frame

pub fn wait_cycles(cycles: u16) {
    thread::sleep(Duration::new(0, (CPU_CLOCK_SPEED_NS * cycles as f32) as u32))
}

