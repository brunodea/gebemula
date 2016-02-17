use cpu::cpu::{Cpu, Instruction};
use mem::mem::Memory;
use cpu::timer::Timer;
use debugger::Debugger;

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

    fn init(&mut self) {
        self.cpu.reset_registers();
    }

    pub fn run(&mut self) {
        self.init();
        let debugger: &mut Debugger = &mut Debugger::new();
        loop {
            let instruction: &Instruction = &self.cpu.run_instruction(&mut self.mem);
            if cfg!(debug_assertions) {
                debugger.run(instruction, &self.cpu, &self.mem, &self.timer);
            }
            self.timer.update(instruction.cycles, &mut self.mem);
            //Checks for interrupt requests should be made after *every* instruction is
            //run.
            self.cpu.handle_interrupts(&mut self.mem);
        }
    }
}
