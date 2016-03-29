#[derive(Copy, Clone, PartialEq)]
pub enum EventType {
    BootstrapFinished,
    DMATransfer,
    JoypadPressed,
}

#[derive(Copy, Clone)]
pub struct Event {
    pub duration: u32,
    pub event_type: EventType,
    pub additional_value: u8,
}

impl Event {
    pub fn new(duration: u32, event_type: EventType) -> Event {
        Event {
            duration: duration,
            event_type: event_type,
            additional_value: 0,
        }
    }
}
