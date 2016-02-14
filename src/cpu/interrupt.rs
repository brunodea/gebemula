use time::{Duration, PreciseTime};

#[derive(Copy, Clone)]
pub enum InterruptType {
    V_BLANK,
}

impl InterruptType {
    fn frequency(interrupt: InterruptType) -> Duration {
        match interrupt {
            InterruptType::V_BLANK => Duration::nanoseconds(16750419) //~59.7Hz
        }
    }
}

#[derive(Copy, Clone)]
pub struct Interrupt {
    itype: InterruptType,
    time: PreciseTime,
}

impl Interrupt {
    pub fn new(itype: InterruptType) -> Interrupt {
        Interrupt {
            itype: itype,
            time: PreciseTime::now(),
        }
    }

    //returns opcode if should interrupt, otherwise returns None.
    pub fn interrupt(&mut self) -> bool {
        let now: PreciseTime = PreciseTime::now();
        if self.time.to(now) >= InterruptType::frequency(self.itype) {
            self.time = now;
            return true;
        }
        false
    }

    pub fn ns_time_diff_to_now(&self) -> u64 {
        self.time.to(PreciseTime::now()).num_nanoseconds().unwrap() as u64
    }
}



