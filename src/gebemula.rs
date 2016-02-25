use cpu;
use cpu::ioregister;
use cpu::lcd::ScreenRefreshEvent;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;

use graphics;
use graphics::graphics::BGWindowLayer;

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
            let bg_on: bool = ioregister::LCDCRegister::is_bg_window_display_on(&self.mem);
            let wn_on: bool = ioregister::LCDCRegister::is_window_display_on(&self.mem);
            if bg_on && self.screen_refresh_event.is_scan_line {
                let bg: BGWindowLayer = BGWindowLayer::new(true, &self.mem);
                bg.update_line_buffer(&mut self.screen_buffer, &self.mem);
            }
            if wn_on {
                let wy: u8 = self.mem.read_byte(cpu::consts::WY_REGISTER_ADDR);
                let wx: u8 = self.mem.read_byte(cpu::consts::WX_REGISTER_ADDR);
                if wy < graphics::consts::DISPLAY_HEIGHT_PX &&
                    wx < graphics::consts::DISPLAY_WIDTH_PX + 7 {

                    let wn: BGWindowLayer = BGWindowLayer::new(false, &self.mem);
                    wn.update_line_buffer(&mut self.screen_buffer, &self.mem);
                }
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
                println!("{}", self.cycles_per_sec);
                self.cycles_per_sec = 0;
            }
            self.step();
        }
    }
}
