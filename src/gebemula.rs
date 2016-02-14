use cpu::cpu::Cpu;
use mem::mem::Memory;
use cpu::interrupt::{Interrupt, InterruptType};
use cpu::timer::Timer;

pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
    timer: Timer,
}

impl Gebemula {
    pub fn new() -> Gebemula {
        Gebemula {
            cpu: Cpu::new(),
            mem: Memory::new(),
            timer: Timer::new(),
        }
    }

    pub fn load_bootstrap_rom(&mut self, bootstrap_rom: &[u8]) {
        self.mem.load_bootstrap_rom(bootstrap_rom);
    }

    pub fn load_game_rom(&mut self, game_rom: &[u8]) {
        self.mem.load_game_rom(game_rom);
    }

    pub fn run(&mut self) {
        self.cpu.reset_registers();
        self.mem.write_byte(0xFF44, 0x90); //for bypassing 'waiting for screen frame'.

        let v_blank: &mut Interrupt = &mut Interrupt::new(InterruptType::V_BLANK);
        loop {
            if v_blank.interrupt() {
                self.timer.push_event(v_blank.clone());
            }

            let max_cycles: u64 = match self.timer.cycles_until_next_event() {
                Some(cycles) => cycles,
                None => 0,
            };

            //only will work if there is at least one event in the queue.
            let mut cycles: u64 = 0;
            while cycles < max_cycles {
                cycles += self.cpu.run_instruction(&mut self.mem) as u64;
            }
            
            //next_events removes the event from the queue
            if let Some(interrupt) = self.timer.next_event() {
                //TODO
            }
            //TODO sleep until next event time is reached.
        }
    }
}
