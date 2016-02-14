use cpu::cpu::Cpu;
use mem::mem::Memory;
use time::{Duration, PreciseTime};
use std::time;
use cpu::timer::{EventType, Event, Timeline};
use std::thread;

pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
    timeline: Timeline,
}

impl Gebemula {
    pub fn new() -> Gebemula {
        Gebemula {
            cpu: Cpu::new(),
            mem: Memory::new(),
            timeline: Timeline::new(),
        }
    }

    pub fn load_bootstrap_rom(&mut self, bootstrap_rom: &[u8]) {
        self.mem.load_bootstrap_rom(bootstrap_rom);
    }

    pub fn load_game_rom(&mut self, game_rom: &[u8]) {
        self.mem.load_game_rom(game_rom);
    }

    fn init(&mut self) {
        self.cpu.reset_registers();
        self.mem.write_byte(0xFF44, 0x90); //for bypassing 'waiting for screen frame'.

        //periodic events should be put in the queue beforehand.
        self.timeline.push_event(Event::new(EventType::INTERRUPT_V_BLANK));
    }

    pub fn run(&mut self) {
        self.init();
        loop {
            let start: PreciseTime = PreciseTime::now();
            let max_cycles: u64 = self.timeline.cycles_until_next_event();

            let mut cycles: u64 = 0;
            while cycles < max_cycles {
                cycles += self.cpu.run_instruction(&mut self.mem) as u64;
            }

            //next_events removes the event from the queue
            if let Some(event) = self.timeline.next_event() {
                //TODO event code

                //sleeps for the amount of time needed until the event actually
                //should happen.
                let duration: Duration = start.to(PreciseTime::now());
                if let Some(offset_time) = event.offset_time(duration) {
                    thread::sleep(time::Duration::new(0, offset_time as u32));
                }
                //put periodic events in the queue again.
                match event.event_type {
                    EventType::INTERRUPT_V_BLANK => {
                        self.timeline.push_event(event);
                    },
                }
            }
        }
    }
}
