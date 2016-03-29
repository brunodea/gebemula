pub mod lcd;
pub mod joypad;

use super::mem::mem::Memory;

pub trait Peripheral {
    fn handle_event(&mut self, memory: &mut Memory);
}
