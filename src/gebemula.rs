use cpu::cpu::Cpu;
use mem::mem::Memory;
use time::{Duration, PreciseTime};
use std;

pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
}

impl Gebemula {
    pub fn new() -> Gebemula {
        Gebemula {
            cpu: Cpu::new(),
            mem: Memory::new(),
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
    }

    pub fn run(&mut self) {
        self.init();
        loop {
            self.cpu.run_instruction(&mut self.mem) as u64;
            self.cpu.handle_interrupts(&mut self.mem);
        }
    }
}
