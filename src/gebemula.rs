use cpu::consts;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;
use mem::mem::Memory;
use graphics;
use graphics::graphics::BackgroundMap;
use debugger::Debugger;

use sdl2;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
    timer: Timer,
    debugger: Debugger,
}

impl Gebemula {
    pub fn new() -> Gebemula {
        Gebemula {
            cpu: Cpu::new(),
            mem: Memory::new(),
            timer: Timer::new(),
            debugger: Debugger::new(),
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

    fn step(&mut self) {
        let instruction: &Instruction = &self.cpu.run_instruction(&mut self.mem);
        if cfg!(debug_assertions) {
            self.debugger.run(instruction, &self.cpu, &self.mem, &self.timer);
        }
        self.timer.update(instruction.cycles, &mut self.mem);
        //Checks for interrupt requests should be made after *every* instruction is
        //run.
        self.cpu.handle_interrupts(&mut self.mem);
    }

    pub fn run(&mut self) {
        self.init();
        loop {
            self.step();
        }
    }

    pub fn run_sdl(&mut self) {
        self.init();

        let sdl_context = sdl2::init().unwrap();
        let vide_subsystem = sdl_context.video().unwrap();

        let window = vide_subsystem.window(
            "Gebemula Emulator",
            graphics::consts::DISPLAY_HEIGHT_PX,
            graphics::consts::DISPLAY_WIDTH_PX)
                .position_centered()
                .opengl()
                .build()
                .unwrap();

        let mut renderer = window.renderer().build().unwrap();
        let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGB24,
            (graphics::consts::DISPLAY_HEIGHT_PX, graphics::consts::DISPLAY_WIDTH_PX)).unwrap();


        renderer.clear();
        renderer.copy(&texture, None, Some(Rect::new_unwrap(
                    0,0,
                    graphics::consts::DISPLAY_HEIGHT_PX, graphics::consts::DISPLAY_WIDTH_PX)));
        renderer.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut bg_map: BackgroundMap = BackgroundMap::new(&self.mem);
        let debugger: &mut Debugger = &mut Debugger::new();
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            self.step();
            //texture.update(None, &BackgroundMap::background_rgb(&self.mem), 3);
            texture.with_lock(None, |buffer: &mut [u8], _| {
                let mut p: usize = 0;
                'bg: while let Some(tile) = bg_map.next_tile(&self.mem) {
                    for pixel in tile.rgb(&self.mem) {
                        buffer[p] = pixel;
                        p += self.mem.read_byte(cpu::ioregister::SCY_REGISTER_ADDR);
                        if p >= (graphics::consts::DISPLAY_HEIGHT_PX*graphics::consts::DISPLAY_WIDTH_PX) as usize {
                            break 'bg
                        }
                    }
                }
            }).unwrap();
        }
    }
}
