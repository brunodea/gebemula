use cpu;
use cpu::ioregister;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;
use mem::mem::Memory;
use graphics;
use graphics::graphics::{BGWindowLayer, apply_palette};
use debugger::Debugger;

use sdl2;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time;

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
            .opengl()
            .build()
            .unwrap();


        let mut renderer = window.renderer().build().unwrap();
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 255));
        renderer.clear();
        renderer.present();
        let mut texture = renderer.create_texture_streaming(
            PixelFormatEnum::ABGR8888,
            (graphics::consts::DISPLAY_WIDTH_PX,
             graphics::consts::DISPLAY_HEIGHT_PX)).unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut fps: u32 = 0;
        let mut last_time = time::now();
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            if ioregister::LCDCRegister::is_lcd_display_enable(&self.mem) {
                let bg_on: bool = ioregister::LCDCRegister::is_bg_window_display_on(&self.mem);
                let wn_on: bool = ioregister::LCDCRegister::is_window_display_on(&self.mem);

                if bg_on || wn_on {
                    texture.with_lock(None, |buffer: &mut [u8], _| {
                        if bg_on {
                            let mut bg_map: BGWindowLayer =
                                BGWindowLayer::new(true, &self.mem);
                            for (i, value) in apply_palette(
                                &bg_map.resize_to_display(&self.mem)).iter().enumerate() {
                                buffer[i] = *value;
                            }
                        }
                        if wn_on {
                            let mut window: BGWindowLayer =
                                BGWindowLayer::new(false, &self.mem);
                            let mut x: u32 = (self.mem.read_byte(cpu::consts::WX_REGISTER_ADDR)-7) as u32;
                            let mut y: u32 = self.mem.read_byte(cpu::consts::WY_REGISTER_ADDR) as u32;
                            if x < graphics::consts::DISPLAY_WIDTH_PX+7 &&
                                y < graphics::consts::DISPLAY_HEIGHT_PX {
                                for (_, value) in apply_palette(
                                    &window.resize_to_display(&self.mem)).iter().enumerate() {
                                    let pos: usize = ((y*graphics::consts::DISPLAY_WIDTH_PX) + x) as usize;
                                    buffer[pos] = *value;
                                    x += 1;
                                    if x > graphics::consts::DISPLAY_WIDTH_PX {
                                        x = 0;
                                        y += 1;
                                        if y > graphics::consts::DISPLAY_HEIGHT_PX {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }).unwrap();
                }
                renderer.clear();
                renderer.copy(&texture, None, None);
                renderer.present();
                fps += 1;
            }
            let now = time::now();
            if now - last_time >= time::Duration::seconds(1) {
                last_time = now;
                renderer.window_mut().unwrap().set_title(&format!("{}", fps));
                fps = 0;
            }
            self.step();
        }
    }
}
