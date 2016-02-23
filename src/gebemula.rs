use cpu;
use cpu::ioregister;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;

use graphics;
use graphics::graphics::BGWindowLayer;

use mem::mem::Memory;
use debugger::Debugger;

use sdl2;
use sdl2::rect::Rect;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time;

pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
    timer: Timer,
    debugger: Debugger,
    game_rom: Vec<u8>,
    cycles_per_sec: u64,
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
        self.timer.update(instruction.cycles, &mut self.mem);
        //Checks for interrupt requests should be made after *every* instruction is
        //run.
        self.cpu.handle_interrupts(&mut self.mem);
        self.cycles_per_sec += instruction.cycles as u64;
    }

    pub fn run_sdl(&mut self) {
        self.init();

        let sdl_context = sdl2::init().unwrap();
        let vide_subsystem = sdl_context.video().unwrap();

        let window = vide_subsystem.window(
            "Gebemula Emulator",
            graphics::consts::DISPLAY_WIDTH_PX as u32,
            graphics::consts::DISPLAY_HEIGHT_PX as u32)
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

        //*4 to support RGBA.
        let mut buffer: &mut [u8] = &mut [0; graphics::consts::DISPLAY_WIDTH_PX as usize * 4];
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

                let mut texture_updated: bool = false;
                if bg_on {
                    let bg: BGWindowLayer = BGWindowLayer::new(true, &self.mem);
                    if let Some(curr_line) = bg.update_line_buffer(buffer, &self.mem) {
                        texture.update(Rect::new(
                                0, curr_line as i32,
                                graphics::consts::DISPLAY_WIDTH_PX as u32, 1
                                ).unwrap(),
                            buffer, buffer.len()).unwrap();
                        texture_updated = true;
                    }
                }
                if wn_on {
                    let wy: u8 = self.mem.read_byte(cpu::consts::WY_REGISTER_ADDR);
                    let wx: u8 = self.mem.read_byte(cpu::consts::WX_REGISTER_ADDR);
                    if wy < graphics::consts::DISPLAY_HEIGHT_PX &&
                        wx < graphics::consts::DISPLAY_WIDTH_PX + 7 {
                        let wn: BGWindowLayer = BGWindowLayer::new(false, &self.mem);
                        if let Some(curr_line) = wn.update_line_buffer(buffer, &self.mem) {
                            texture.update(Rect::new(
                                    0, curr_line as i32,
                                    graphics::consts::DISPLAY_WIDTH_PX as u32, 1
                                    ).unwrap(),
                                buffer, buffer.len()).unwrap();
                            texture_updated = true;
                        }
                    }
                }
                if texture_updated {
                    renderer.clear();
                    renderer.copy(&texture, None, None);
                    renderer.present();
                }
            }
            let now = time::now();
            if now - last_time >= time::Duration::seconds(1) {
                last_time = now;
                renderer.window_mut().unwrap().set_title(&format!("{}", self.cycles_per_sec));
            }
            self.step();
        }
    }
}
