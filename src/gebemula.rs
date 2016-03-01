use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;

use graphics;
use graphics::graphics::Graphics;

use mem::mem::Memory;
use debugger::Debugger;

use sdl2;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time;

//display width * display height * 4 (rgba)
pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
    timer: Timer,
    debugger: Debugger,
    game_rom: Vec<u8>,
    cycles_per_sec: u32,
    graphics: Graphics,
}

impl Gebemula {
    pub fn new() -> Gebemula {
        Gebemula {
            cpu: Cpu::new(),
            mem: Memory::new(),
            timer: Timer::new(),
            debugger: Debugger::new(),
            game_rom: Vec::new(),
            cycles_per_sec: 0,
            graphics: Graphics::new(),
        }
    }

    pub fn load_bootstrap_rom(&mut self, bootstrap_rom: &[u8]) {
        self.mem.load_bootstrap_rom(bootstrap_rom);
    }

    pub fn load_game_rom(&mut self, game_rom: &[u8]) {
        for byte in game_rom {
            self.game_rom.push(*byte);
        }
        self.mem.load_game_rom(game_rom);
    }

    fn init(&mut self) {
        self.cpu.reset_registers();
    }

    fn step(&mut self) {
        let instruction: &Instruction = &self.cpu.run_instruction(&mut self.mem);
        if instruction.address == 0x100 {
            //Disable bootstrap rom.
            self.mem.load_bootstrap_rom(&self.game_rom[0..0x100]);
        }
        if cfg!(debug_assertions) {
            self.debugger.run(instruction, &self.cpu, &self.mem, &self.timer);
        }
        self.graphics.update(instruction.cycles, &mut self.mem);
        self.timer.update(instruction.cycles, &mut self.mem);
        //Checks for interrupt requests should be made after *every* instruction is
        //run.
        self.cpu.handle_interrupts(&mut self.mem);
        self.cycles_per_sec += instruction.cycles;
    }

    pub fn run_sdl(&mut self) {
        self.init();

        let sdl_context = sdl2::init().unwrap();
        let vide_subsystem = sdl_context.video().unwrap();

        let window = vide_subsystem.window(
            "Gebemula Emulator",
            graphics::consts::DISPLAY_WIDTH_PX as u32 * 2,
            graphics::consts::DISPLAY_HEIGHT_PX as u32 * 2)
            .opengl()
            .build()
            .unwrap();

        let mut renderer = window.renderer().build().unwrap();
        renderer.set_draw_color(Color::RGBA(0,0,0,255));

        let mut texture = renderer.create_texture_streaming(
            PixelFormatEnum::ABGR8888,
            (graphics::consts::DISPLAY_WIDTH_PX as u32,
             graphics::consts::DISPLAY_HEIGHT_PX as u32)).unwrap();


        renderer.clear();
        renderer.present();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut last_time = time::now();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::KeyDown { keycode: Some(Keycode::F1), .. } => {
                        self.graphics.toggle_bg();
                    },
                    Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                        self.graphics.toggle_wn();
                    },
                    Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                        self.graphics.toggle_sprites();
                    },
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }

            if self.graphics.update_screen() {
                renderer.clear();
                texture.update(None, &self.graphics.screen_buffer,
                               graphics::consts::DISPLAY_WIDTH_PX as usize * 4).unwrap();
                renderer.copy(&texture, None, None);
                renderer.present();
            }

            let now = time::now();
            if now - last_time >= time::Duration::seconds(1) {
                last_time = now;
                renderer.window_mut().unwrap().set_title(&format!("Gebemula - {}", self.cycles_per_sec));
                self.cycles_per_sec = 0;
            }
            self.step();
        }
    }
}
