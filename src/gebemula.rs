use cpu::cpu::Cpu;
use mem::mem::Memory;
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

    fn init(&mut self) {
        self.cpu.reset_registers();
        self.mem.write_byte(0xFF44, 0x90); //for bypassing 'waiting for screen frame'.
        self.timer.init(&self.mem);
    }

    pub fn run(&mut self) {
        self.init();
        loop {
            let cycles: u32 = self.cpu.run_instruction(&mut self.mem);
            self.timer.update(cycles, &mut self.mem);
            //Checks for interrupt requests should be made after *every* instruction is
            //run.
            self.cpu.handle_interrupts(&mut self.mem);
            //TODO before requesting an interrupt, we *have* to check if the interrupts
            //are enabled. This way, only an interrupt code would allow interrupts. 
            //The problem is that an interrupt may happen during the execution code of
            //some other interrupt, which could be a problem (unless the interrupt code
            //executes EI).
        }
    }
}
