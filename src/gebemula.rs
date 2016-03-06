use cpu;
use cpu::ioregister;
use cpu::interrupt;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;

use graphics;
use graphics::graphics::Graphics;

use mem::mem::Memory;
use debugger::Debugger;

use sdl2;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::keyboard::Keycode;

use time;

#[derive(Copy, Clone, PartialEq)]
pub enum EventType {
    S_OAM,
    S_VRAM,
    H_BLANK,
    V_BLANK,
}

#[derive(Copy, Clone)]
pub struct Event {
    duration: u32,
    priority: u32,
    event_type: EventType,
}

impl Event {
    pub fn new(duration: u32, event_type: EventType) -> Event {
        Event {
            duration: duration,
            priority: 0,
            event_type: event_type,
        }
    }
}

pub struct EventTimeline {
    periodic_events: [Event; 4],
    curr_event_type: EventType,
}


impl EventTimeline {
    pub fn new() -> EventTimeline {
        let h_blank = Event::new(
                cpu::consts::STAT_MODE_0_DURATION_CYCLES,
                EventType::H_BLANK);
        let v_blank = Event::new(
                cpu::consts::STAT_MODE_1_DURATION_CYCLES,
                EventType::V_BLANK);
        let scanline_oam = Event::new(
                cpu::consts::STAT_MODE_2_DURATION_CYCLES,
                EventType::S_OAM);
        let scanline_vram = Event::new(
                cpu::consts::STAT_MODE_3_DURATION_CYCLES,
                EventType::S_VRAM);
        EventTimeline {
            periodic_events: [scanline_oam, scanline_vram, h_blank, v_blank],
            curr_event_type: EventType::S_OAM,
        }
    }

    pub fn curr_event(&self) -> Option<Event> {
        let mut res: Option<Event> = None;
        for i in 0..self.periodic_events.len() {
            let e: Event = self.periodic_events[i].clone();
            if e.event_type == self.curr_event_type {
                res = Some(e);
                break;
            }
        }
        return res;
    }

    //pub fn add_event(&mut self, event: Event) {
    //    //event.priority += event.rate;
    //    let position: usize = 0;
    //    for (i, e) in self.periodic_events.into_iter().enumerate() {
    //        position = i;
    //        if e.priority > event.priority {
    //            break;
    //        }
    //    }

    //    self.periodic_events.insert(position, event);
    //}
}

pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
    timer: Timer,
    debugger: Debugger,
    game_rom: Vec<u8>,
    cycles_per_sec: u32,
    graphics: Graphics,
    should_display_screen: bool,
    timeline: EventTimeline,
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
            should_display_screen: false,
            timeline: EventTimeline::new(),
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
        ioregister::update_stat_reg_mode_flag(0b10, &mut self.mem);
    }

    fn run_event(&mut self, event: Event) {
        let mut gpu_mode_number: Option<u8> = None;
        match event.event_type {
            EventType::S_OAM => {
                gpu_mode_number = Some(0b11);
                self.timeline.curr_event_type = EventType::S_VRAM;
            },
            EventType::S_VRAM => {
                gpu_mode_number = Some(0b00);
                self.timeline.curr_event_type = EventType::H_BLANK;
            },
            EventType::H_BLANK => {
                let mut ly: u8 = self.mem.read_byte(cpu::consts::LY_REGISTER_ADDR);
                ly += 1;
                self.graphics.update(&mut self.mem);
                if ly == graphics::consts::DISPLAY_HEIGHT_PX {
                    self.should_display_screen = true;
                    gpu_mode_number = Some(0b01);
                    self.timeline.curr_event_type = EventType::V_BLANK;
                    interrupt::request(interrupt::Interrupt::VBlank, &mut self.mem);
                } else {
                    self.timeline.curr_event_type = EventType::S_OAM;
                    gpu_mode_number = Some(0b10);
                }
                self.mem.write_byte(cpu::consts::LY_REGISTER_ADDR, ly);
            },
            EventType::V_BLANK => {
                let mut ly: u8 = self.mem.read_byte(cpu::consts::LY_REGISTER_ADDR);
                if ly == graphics::consts::DISPLAY_HEIGHT_PX + 10 {
                    self.timeline.curr_event_type = EventType::S_OAM;
                    gpu_mode_number = Some(0b10);
                    ly = 0;
                } else {
                    self.timeline.curr_event_type = EventType::V_BLANK;
                    ly += 1;
                }
                self.mem.write_byte(cpu::consts::LY_REGISTER_ADDR, ly);
            },
        }

        if let Some(gpu_mode) = gpu_mode_number {
            ioregister::update_stat_reg_mode_flag(gpu_mode, &mut self.mem);
            ioregister::lcdc_stat_interrupt(&mut self.mem); //verifies and request LCDC interrupt
        }
    }

    fn step(&mut self) {
        self.should_display_screen = false;
        let event: Event = self.timeline.curr_event().unwrap();
        let mut cycles: u32 = 0;
        while cycles < event.duration {
            let instruction: &Instruction = &self.cpu.run_instruction(&mut self.mem);
            if instruction.address == 0x100 {
                //Disable bootstrap rom.
                self.mem.load_bootstrap_rom(&self.game_rom[0..0x100]);
            }
            self.cpu.handle_interrupts(&mut self.mem);
            self.timer.update(instruction.cycles, &mut self.mem);
            if cfg!(debug_assertions) {
                self.debugger.run(instruction, &self.cpu, &self.mem, &self.timer);
            }
            cycles += instruction.cycles;
            self.cycles_per_sec += instruction.cycles;
        }
        self.run_event(event);
        //self.graphics.update(instruction.cycles, &mut self.mem);
        //
        //
        //self.cycles_per_sec += instruction.cycles;
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
                    sdl2::event::Event::KeyDown { keycode: Some(Keycode::F1), .. } => {
                        self.graphics.toggle_bg();
                    },
                    sdl2::event::Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                        self.graphics.toggle_wn();
                    },
                    sdl2::event::Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                        self.graphics.toggle_sprites();
                    },
                    sdl2::event::Event::Quit {..} |
                    sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }

            if self.should_display_screen {
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
