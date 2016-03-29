bitflags! {
    pub flags JoypadKey: u8 {
        const NONE   = 0,

        const A      = 1 << 0,
        const B      = 1 << 1,
        const SELECT = 1 << 2,
        const START  = 1 << 3,

        const RIGHT  = 1 << 4,
        const LEFT   = 1 << 5,
        const UP     = 1 << 6,
        const DOWN   = 1 << 7,
    }
}

pub struct Joypad {
    keys: JoypadKey,
}

impl Default for Joypad {
    fn default() -> Self {
        Joypad {
            keys: NONE,
        }
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
}

