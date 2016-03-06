use cpu;

#[derive(Copy, Clone, PartialEq)]
pub enum EventType {
    S_OAM,
    S_VRAM,
    H_BLANK,
    V_BLANK,
    DISABLE_BOOTSTRAP,
    DMA_TRANSFER,
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

impl EventTimeline {
    pub fn new() -> EventTimeline {
        let h_blank = Event::new(
                cpu::consts::STAT_MODE_0_DURATION_CYCLES,
                EventType::H_BLANK);
        let v_blank = Event::new(
                cpu::consts::STAT_MODE_1_DURATION_CYCLES,
                EventType::V_BLANK);
        let scanline_oam = Event::new(
                cpu::consts::STAT_MODE_2_DURATION_CYCLES,
                EventType::S_OAM);
        let scanline_vram = Event::new(
                cpu::consts::STAT_MODE_3_DURATION_CYCLES,
                EventType::S_VRAM);
        EventTimeline {
            periodic_events: [scanline_oam, scanline_vram, h_blank, v_blank],
            curr_event_type: EventType::S_OAM,
        }
    }

    pub fn curr_event(&self) -> Option<Event> {
        let mut res: Option<Event> = None;
        for i in 0..self.periodic_events.len() {
            let e: Event = self.periodic_events[i].clone();
            if e.event_type == self.curr_event_type {
                res = Some(e);
                break;
            }
        }
        return res;
    }
}
