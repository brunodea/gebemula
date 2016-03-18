use cpu;

#[derive(Copy, Clone, PartialEq)]
pub enum EventType {
    OAM,
    Vram,
    HorizontalBlank,
    VerticalBlank,
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

pub struct EventTimeline {
    periodic_events: [Event; 4],
    pub curr_event_type: EventType,
}

impl Default for EventTimeline {
    fn default() -> EventTimeline {
        let h_blank = Event::new(cpu::consts::STAT_MODE_0_DURATION_CYCLES, EventType::HorizontalBlank);
        let v_blank = Event::new(cpu::consts::STAT_MODE_1_DURATION_CYCLES, EventType::VerticalBlank);
        let scanline_oam = Event::new(cpu::consts::STAT_MODE_2_DURATION_CYCLES, EventType::OAM);
        let scanline_vram = Event::new(cpu::consts::STAT_MODE_3_DURATION_CYCLES, EventType::Vram);
        EventTimeline {
            periodic_events: [scanline_oam, scanline_vram, h_blank, v_blank],
            curr_event_type: EventType::OAM,
        }
    }
}

impl EventTimeline {
    pub fn curr_event(&self) -> Option<Event> {
        let mut res: Option<Event> = None;
        for i in 0..self.periodic_events.len() {
            let e: Event = self.periodic_events[i];
            if e.event_type == self.curr_event_type {
                res = Some(e);
                break;
            }
        }
        res
    }
}
