use super::super::mem;
use cpu::{interrupt, ioregister};

pub struct Timer {
    /// The timer overflow behavior is delayed.
    timer_overflow: bool,
    tima_cycles_counter: u32,
}

impl Default for Timer {
    fn default() -> Timer {
        Timer {
            timer_overflow: false,
            tima_cycles_counter: 0,
        }
    }
}

impl Timer {
    pub fn update(&mut self, cycles: u32, memory: &mut mem::Memory) {
        let internal_counter = memory.read_byte(ioregister::TIMER_INTERNAL_COUNTER_ADDR);
        let div = memory.read_byte(ioregister::DIV_REGISTER_ADDR);

        let old_internal_timer = ((div as u16) << 8) | internal_counter as u16;
        let internal_timer = old_internal_timer.wrapping_add(cycles as u16);

        memory.write_byte(ioregister::TIMER_INTERNAL_COUNTER_ADDR,
                          internal_timer as u8);
        memory.write_byte(ioregister::DIV_REGISTER_ADDR, (internal_timer >> 8) as u8);

        if self.timer_overflow {
            self.timer_overflow = false;
            let tima = memory.read_byte(ioregister::TMA_REGISTER_ADDR);
            memory.write_byte(ioregister::TIMA_REGISTER_ADDR, tima);
            interrupt::request(interrupt::Interrupt::TimerOverflow, memory);
        } else {
            let tac = memory.read_byte(ioregister::TAC_REGISTER_ADDR);
            // timer start bit is on
            if (tac >> 2) & 0b1 == 0b1 {
                // TODO: these numbers never change, move them out to some static const.
                let fc = |hz| -> u32 {
                    ioregister::CPU_FREQUENCY_HZ / (1_000_000 / hz)
                };
                let freq_cycles = match tac & 0b11 {
                    0 => fc(4096u32),
                    1 => fc(262144u32),
                    2 => fc(65536u32),
                    3 => fc(16384u32),
                    _ => unreachable!(),
                };

                //TODO implement glitch?
                self.tima_cycles_counter = self.tima_cycles_counter.wrapping_add(cycles);
                if self.tima_cycles_counter > freq_cycles {
                    let tima = memory.read_byte(ioregister::TIMA_REGISTER_ADDR).wrapping_add(1);
                    if tima == 0 {
                        // overflows
                        self.timer_overflow = true;
                    }
                    memory.write_byte(ioregister::TIMA_REGISTER_ADDR, tima);
                    self.tima_cycles_counter = 0;
                }
            }

        }
    }
}
