use std::cmp;
use time;

#[derive(Default)]
pub struct Rtc {
    seconds: u8,
    minutes: u8,
    hours: u8,
    /// 8 least significant bits of day counter
    day_counter_lsb: u8,
    /// bit0: MSB of day counter, bit6: stop timer, bit7: day counter overflowed
    misc_bits: u8,
}

impl Rtc {
    pub fn new() -> Rtc {
        let mut rtc : Rtc = Default::default();
        rtc.latch();
        rtc
    }

    pub fn read(&self, address: u8) -> u8 {
        match address {
            0x8 => self.seconds,
            0x9 => self.minutes,
            0xA => self.hours,
            0xB => self.day_counter_lsb,
            0xC => self.misc_bits,
            _   => 0xFF,
        }
    }

    pub fn write(&mut self, address: u8, data: u8) {
        match address {
            0x8 => self.seconds = data,
            0x9 => self.minutes = data,
            0xA => self.hours = data,
            0xB => self.day_counter_lsb = data,
            0xC => self.misc_bits = data,
            _   => (),
        }
    }

    pub fn latch(&mut self) {
        if self.misc_bits & (1 << 6) != 0 {
            // RTC is stopped, so don't update. This will still cause values to jump when it is
            // re-enabled, but a proper solution for that is too complicated.
            return;
        }

        // Since we don't actually count up the time, and just fabricate it from the host time, the
        // day counter will never be above 365, and the carry bit also won't be set on overflow.
        // TODO: Do games rely on being able to adjust the time?
        let now = time::now();
        self.seconds = cmp::min(now.tm_sec, 59) as u8;
        self.minutes = cmp::min(now.tm_min, 59) as u8;
        self.hours = cmp::min(now.tm_hour, 23) as u8;
        self.day_counter_lsb = (now.tm_yday & 0xFF) as u8;
        self.misc_bits = (now.tm_yday & 0x100 >> 8) as u8;
    }
}

