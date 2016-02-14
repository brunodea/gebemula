use time::Duration;

const NS_PER_CYCLE: u64 = 2384000; //nanoseconds per cycle

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EventType {
    INTERRUPT_V_BLANK,
}

impl EventType {
    pub fn frequency(event_type: EventType) -> Duration {
        match event_type {
            EventType::INTERRUPT_V_BLANK => Duration::nanoseconds(16750419), //~59.7Hz
        }
    }
    pub fn address(event_type: EventType) -> u16 {
        match event_type {
            EventType::INTERRUPT_V_BLANK => 0x0040,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Event {
    time_until: Duration, //amount of time until the event happens.
    pub event_type: EventType,
    address: u16, //place to jump to for handling the event.
}

impl Event {
    pub fn new(event_type: EventType) -> Event {
        Event {
            time_until: EventType::frequency(event_type),
            event_type: event_type,
            address: EventType::address(event_type),
        }
    }

    pub fn offset_time(&self, duration: Duration) -> Option<u64> {
        if duration < self.time_until {
            Some((self.time_until - duration).num_nanoseconds().unwrap() as u64)
        } else {
            None
        }
    }
}

pub struct Timeline {
    event_queue: Vec<Event>,
}

impl Timeline {
    pub fn new() -> Timeline {
        Timeline {
            event_queue: Vec::new(),
        }
    }

    pub fn push_event(&mut self, event: Event) {
        //TODO sort by time_until, priority, etc.
        self.event_queue.push(event);
    }

    pub fn next_event(&mut self) -> Option<Event> {
        if !self.event_queue.is_empty() {
            Some(self.event_queue.remove(0))
        } else {
            None
        }
    }

    pub fn cycles_until_next_event(&self) -> u64 {
        if !self.event_queue.is_empty() {
            let event: Event = self.event_queue[0];
            event.time_until.num_nanoseconds().unwrap() as u64 / NS_PER_CYCLE
        } else {
            0
        }
    }
}
