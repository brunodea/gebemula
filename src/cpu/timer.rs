use super::super::mem;
use cpu::{interrupt, ioregister};

pub struct Timer {
    /// The timer overflow behavior is delayed.
    timer_overflow: bool,
}

impl Default for Timer {
    fn default() -> Timer {
        Timer {
            timer_overflow: false,
        }
    }
}

impl Timer {
    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        let internal_counter = memory.read_byte(ioregister::TIMER_INTERNAL_COUNTER_ADDR);
        let div = memory.read_byte(ioregister::DIV_REGISTER_ADDR);

        let old_internal_timer = ((div as u16) << 8) | internal_counter as u16;
        let internal_timer = old_internal_timer.wrapping_add(cycles as u16);

        memory.write_byte(ioregister::TIMER_INTERNAL_COUNTER_ADDR, internal_timer as u8);
        memory.write_byte(ioregister::DIV_REGISTER_ADDR, (internal_timer >> 8) as u8);

        if self.timer_overflow {
            self.timer_overflow = false;
            let tima = memory.read_byte(ioregister::TMA_REGISTER_ADDR);
            memory.write_byte(ioregister::TIMA_REGISTER_ADDR, tima);
            interrupt::request(interrupt::Interrupt::TimerOverflow, memory);
        } else {
            let tac = memory.read_byte(ioregister::TAC_REGISTER_ADDR);
            let timer_bit = match tac & 0b11 {
                0 => 9,
                1 => 3,
                2 => 5,
                3 => 7,
                _ => unreachable!(),
            };

            //TODO implement glitch?
            let old_bit = ((old_internal_timer >> timer_bit) & 0b1) as u8;
            let new_bit = ((internal_timer >> timer_bit) & 0b1) as u8;
            // timer start bit & timer bit from 1 to 0.
            if ((tac >> 2) & 0b1) & (old_bit & !new_bit) == 0b1 {
                let tima = memory.read_byte(ioregister::TIMA_REGISTER_ADDR).wrapping_add(1);
                if tima == 0 {
                    // overflows
                    self.timer_overflow = true;
                }
                memory.write_byte(ioregister::TIMA_REGISTER_ADDR, tima);
            }

        }
    }
}
