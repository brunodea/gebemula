use cpu::ioregister;
use cpu::lcd::ScreenRefreshEvent;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;

use graphics;

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
    screen_refresh_event: ScreenRefreshEvent,
    screen_buffer: [u8; 160*144*4],
    game_rom: Vec<u8>,
    cycles_per_sec: u32,
    show_bg: bool,
    show_wn: bool,
    show_sprites: bool,
}

impl Gebemula {
    pub fn new() -> Gebemula {
        Gebemula {
            cpu: Cpu::new(),
            mem: Memory::new(),
            timer: Timer::new(),
            debugger: Debugger::new(),
            screen_refresh_event: ScreenRefreshEvent::new(),
            screen_buffer: [255; 160*144*4],
            game_rom: Vec::new(),
            cycles_per_sec: 0,
            show_bg: true,
            show_wn: true,
            show_sprites: true,
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
        if ioregister::LCDCRegister::is_lcd_display_enable(&self.mem) {
            self.screen_refresh_event.update(instruction.cycles, &mut self.mem);
            let bg_on: bool = ioregister::LCDCRegister::is_bg_window_display_on(&self.mem)
                && self.show_bg;
            let wn_on: bool = ioregister::LCDCRegister::is_window_display_on(&self.mem)
                && self.show_wn;
            if self.screen_refresh_event.is_scan_line {
                graphics::graphics::update_line_buffer(bg_on, wn_on, &mut self.screen_buffer, &self.mem);
            }
            if ioregister::LCDCRegister::is_sprite_display_on(&self.mem) && self.show_sprites &&
                self.screen_refresh_event.is_scan_line {
                graphics::graphics::draw_sprites(&mut self.screen_buffer, &self.mem);
            }
        }
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
                        self.show_bg = !self.show_bg;
                        println!("bg: {}", self.show_bg);
                    },
                    Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                        self.show_wn = !self.show_wn;
                        println!("wn: {}", self.show_wn);
                    },
                    Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                        self.show_sprites = !self.show_sprites;
                        println!("sprites: {}", self.show_sprites);
                    },
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }

            if self.screen_refresh_event.is_display_buffer {
                renderer.clear();
                texture.update(None, &self.screen_buffer, 
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
