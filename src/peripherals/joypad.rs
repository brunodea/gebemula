use super::super::mem::Memory;
use super::super::cpu::{interrupt, ioregister};

bitflags! {
    pub struct JoypadKey: u8 {
        const NONE   = 0;

        const A      = 1 << 0;
        const B      = 1 << 1;
        const SELECT = 1 << 2;
        const START  = 1 << 3;

        const RIGHT  = 1 << 4;
        const LEFT   = 1 << 5;
        const UP     = 1 << 6;
        const DOWN   = 1 << 7;
    }
}

pub struct Joypad {
    keys: JoypadKey,
}

impl Default for Joypad {
    fn default() -> Self {
        Joypad { keys: JoypadKey::NONE }
    }
}

impl Joypad {
    pub fn press_key(&mut self, key: JoypadKey) {
        self.keys = self.keys & !key;
    }
    pub fn release_key(&mut self, key: JoypadKey) {
        self.keys = self.keys | key;
    }
    pub fn keys(&self, dir_keys: bool) -> u8 {
        if dir_keys {
            self.keys.bits as u8
        } else {
            self.keys.bits >> 4
        }
    }
    pub fn update_joypad_register(&mut self, memory: &mut Memory) {
        let buttons = self.keys(ioregister::joypad_buttons_selected(memory));
        // old buttons & !new_buttons != 0 -> true if there was a change from 1 to 0.
        // new_buttons < 0b1111 -> make sure at least 1 button was pressed.
        if ioregister::joypad_buttons(memory) & !buttons != 0 && buttons < 0b1111 {
            // interrupt is requested when a button goes from 1 to 0.
            interrupt::request(interrupt::Interrupt::Joypad, memory);
        }

        ioregister::joypad_set_buttons(buttons, memory);
    }
}
