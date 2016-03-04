use cpu;
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

enum EventType {
    SCANLINE_OAM,
    SCANLINE_VRAM,
    H_BLANK,
    VERTICAL_BLANK,
}

pub struct Event {
    rate: u32,
    duration: u32,
    priority: u32,
    event_type: EventType,
}

pub impl Event {
    pub fn new(rate: u32, duration: u32, event_type: EventType) -> Event {
        Event {
            rate: rate,
            duration: duration,
            priority: 0,
            event_type: event_type,
        }
    }
}

pub struct EventTimeline {
    periodic_events: Vec<Event>,
}


impl EventTimeline {
    pub fn new() -> EventTimeline {
        EventTimeline {
            periodic_events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: Event) {
        event.priority += event.rate;
        let position: usize = 0;
        for (i, e) in self.periodic_events.into_iter().enumerate() {
            if e.priority < event.priority {
                position = i;
                break;
            }
        }

        self.periodic_events.insert(position, event);
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.periodic_events.remove(0)
    }
}

pub struct Gebemula {
    cpu: Cpu,
    mem: Memory,
    timer: Timer,
    debugger: Debugger,
    game_rom: Vec<u8>,
    cycles_per_sec: u32,
    graphics: Graphics,
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
        timeline.add_event(Event::new(
                cpu::consts::STAT_MODE_3_DURATION_CYCLES + 
                cpu::consts::STAT_MODE_0_DURATION_CYCLES,
                cpu::consts::STAT_MODE_2_DURATION_CYCLES,
                EventType::SCANLINE_OAM));
        ioregister::update_stat_reg_mode_flag(0b10, &mut self.mem);
    }

    fn run_event(&mut self, event: Event) {
        let gpu_mode_number: Option<u8> = None;
        match event.event_type {
            EventType::SCANLINE_OAM => {
                self.timeline.add_event(Event::new(
                        cpu::consts::STAT_MODE_2_DURATION_CYCLES +
                        cpu::consts::STAT_MODE_0_DURATION_CYCLES,
                        cpu::consts::STAT_MODE_3_DURATION_CYCLES,
                        EventType::SCANLINE_VRAM));
                gpu_mode_number = Some(0b11);
            },
            EventType::SCANLINE_VRAM => {
                self.timeline.add_event(Event::new(
                        cpu::consts::STAT_MODE_2_DURATION_CYCLES +
                        cpu::consts::STAT_MODE_3_DURATION_CYCLES,
                        cpu::consts::STAT_MODE_0_DURATION_CYCLES,
                        EventType::H_BLANK));
                gpu_mode_number = Some(0b00);
            },
            EventType::H_BLANK => {
                    let mut ly: u8 = self.mem.read_byte(cpu::consts::LY_REGISTER_ADDR);
                    ly += 1;
                    if ly == graphics::consts::DISPLAY_HEIGHT_PX {
                        //TODO display buffer
                        gpu_mode_number = Some(0b01);
                        interrupt::request(interrupt::Interrupt::VBlank, memory);
                    } else {
                        timeline.add_event(Event::new(
                                cpu::consts::STAT_MODE_3_DURATION_CYCLES + 
                                cpu::consts::STAT_MODE_0_DURATION_CYCLES,
                                cpu::consts::STAT_MODE_2_DURATION_CYCLES,
                                EventType::SCANLINE_OAM));
                        gpu_mode_number = Some(0b10);
                    }
                    self.mem.write_byte(cpu::consts::LY_REGISTER_ADDR, ly);
            },
            EventType::VERTICAL_BLANK =>,
        }

        if let Some(gpu_mode) = gpu_mode_number {
            ioregister::update_stat_reg_mode_flag(gpu_mode, &mut self.mem);
            //ioregister::lcdc_stat_interrupt(&mut self.mem); //verifies and request LCDC interrupt
        }
    }

    fn step(&mut self) {
        let next_event: Event = timeline.pop_event();
        let cycles: u32 = 0;
        while cycles < next_event.rate {
            let instruction: &Instruction = &self.cpu.run_instruction(&mut self.mem);
            if instruction.address == 0x100 {
                //Disable bootstrap rom.
                self.mem.load_bootstrap_rom(&self.game_rom[0..0x100]);
            }
        }
        self.run_event(event);
        /*
         * cycles_until_next_event = next_event().rate;
         * cycles = 0;
         * while cycles < cycles_until_next_event {
         *     (instruction, non_periodic_event) = self.cpu.run_instruction();
         *     cycles += instruction.cycles;
         *     match non_periodic_event {
         *         DMA => cycles += DMA_DURATION,
         *     }
         * }
         * event = pop_event();
         * run_event(event);
         * sleep(event.duration);
         */
        //let instruction: &Instruction = &self.cpu.run_instruction(&mut self.mem);
        //if instruction.address == 0x100 {
        //    //Disable bootstrap rom.
        //    self.mem.load_bootstrap_rom(&self.game_rom[0..0x100]);
        //}
        //if cfg!(debug_assertions) {
        //    self.debugger.run(instruction, &self.cpu, &self.mem, &self.timer);
        //}
        //self.graphics.update(instruction.cycles, &mut self.mem);
        //self.timer.update(instruction.cycles, &mut self.mem);
        ////Checks for interrupt requests should be made after *every* instruction is
        ////run.
        //self.cpu.handle_interrupts(&mut self.mem);
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
