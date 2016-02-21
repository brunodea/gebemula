use cpu::ioregister;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;
use mem::mem::Memory;
use graphics;
use graphics::graphics::{BackgroundMap, apply_palette};
use debugger::Debugger;

use sdl2;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::render::Renderer;

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

    pub fn run_sdl(&mut self) {
        self.init();

        let sdl_context = sdl2::init().unwrap();
        let vide_subsystem = sdl_context.video().unwrap();

        let window = vide_subsystem.window(
            "Gebemula Emulator",
            graphics::consts::DISPLAY_WIDTH_PX,
            graphics::consts::DISPLAY_HEIGHT_PX)
            .position_centered()
            .opengl()
            .build()
            .unwrap();


        let mut renderer = window.renderer().build().unwrap();
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 255));
        renderer.clear();
        renderer.present();
        let mut texture = renderer.create_texture_streaming(PixelFormatEnum::ABGR8888,
                                                            (graphics::consts::DISPLAY_WIDTH_PX,
                                                             graphics::consts::DISPLAY_HEIGHT_PX)).unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            if ioregister::LCDCRegister::is_lcd_display_enable(&self.mem) &&
                ioregister::LCDCRegister::is_bg_window_display_on(&self.mem) {

                    texture.with_lock(None, |buffer: &mut [u8], _| {
                        let bg_map: BackgroundMap = BackgroundMap::new(&self.mem);
                        for (i, value) in apply_palette(&bg_map.display_rgb(&self.mem)).iter().enumerate() {
                            buffer[i] = *value;
                        }
                    }).unwrap();
                    renderer.clear();
                    renderer.copy(&texture, None, None);
                    renderer.present();
                }
            self.step();
        }
    }
}
