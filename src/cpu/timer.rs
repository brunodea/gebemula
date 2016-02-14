use cpu::interrupt::Interrupt;

const NS_PER_CYCLE: u64 = 2384000; //nanoseconds per cycle

pub struct Timer {
    event_queue: Vec<Interrupt>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            event_queue: Vec::new(),
        }
    }
    
    //TODO handle priority
    pub fn push_event(&mut self, event: Interrupt) {
        self.event_queue.push(event);
    }

    fn first_event(&self) -> Option<Interrupt> {
        if self.event_queue.is_empty() {
            None
        } else {
            Some(self.event_queue[0])
        }
    }

    pub fn next_event(&mut self) -> Option<Interrupt> {
        if self.event_queue.is_empty() {
            None
        } else {
            Some(self.event_queue.remove(0))
        }
    }

    pub fn cycles_until_next_event(&self) -> Option<u64> {
        match self.first_event() {
            Some(event) => {
                Some(event.ns_time_diff_to_now() / NS_PER_CYCLE)
            },
            None => None,
        }
    }
}
