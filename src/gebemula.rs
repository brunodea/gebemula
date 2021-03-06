use crate::peripherals::joypad::{Joypad, JoypadKey};
use crate::peripherals::lcd::LCD;
use crate::peripherals::sound::{AudioController, AUDIO_DESIRED_SPEC};

use crate::cpu::{ioregister, Cpu, EventRequest};
use crate::cpu::timer::Timer;

use crate::graphics;

use crate::mem::Memory;
use crate::debugger::Debugger;

use sdl2;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::keyboard::{Keycode, Scancode};

use time;
use std::{self, thread};
use std::cell::RefCell;
use std::rc::Rc;
use sdl2::audio::AudioQueue;

const GB_MODE_ADDR: u16 = 0x143;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GBMode {
    Mono,
    Color,
}

impl GBMode {
    pub fn get(memory: &Memory) -> Self {
        match memory.read_cartridge(GB_MODE_ADDR) {
            0x80 | 0xC0 => GBMode::Color,
            _ => GBMode::Mono,
        }
    }
}

enum SpeedMode {
    Normal,
    Double,
}

pub struct Gebemula<'a> {
    cpu: Cpu,
    mem: Memory,
    timer: Timer,
    debugger: Debugger,
    cycles_per_sec: u32,
    lcd: LCD,
    joypad: Joypad,
    apu: Rc<RefCell<AudioController>>,
    /// Used to periodically save the battery-backed cartridge SRAM to file.
    battery_save_callback: Option<&'a dyn Fn(&[u8])>,
    speed_mode: SpeedMode,
}

impl<'a> Default for Gebemula<'a> {
    fn default() -> Gebemula<'a> {
        let apu = Rc::new(RefCell::new(AudioController::new()));

        Gebemula {
            cpu: Cpu::default(),
            mem: Memory::new(apu.clone()),
            timer: Timer::default(),
            debugger: Debugger::default(),
            cycles_per_sec: 0,
            lcd: LCD::default(),
            joypad: Joypad::default(),
            apu,
            battery_save_callback: None,
            speed_mode: SpeedMode::Normal,
        }
    }
}

impl<'a> Gebemula<'a> {
    pub fn restart(&mut self) {
        self.cpu.restart();
        self.mem.restart();
        self.lcd.restart(&mut self.mem);
        self.timer = Timer::default();
        self.cycles_per_sec = 0;
        self.joypad = Joypad::default();
    }

    pub fn load_bootstrap_rom(&mut self, bootstrap_rom: &[u8]) {
        self.mem.load_bootstrap_rom(bootstrap_rom);
    }

    pub fn load_cartridge(&mut self, game_rom: &[u8], battery: &[u8]) {
        self.mem.load_cartridge(game_rom, battery);
        if GBMode::get(&self.mem) == GBMode::Color {
            self.lcd.set_color();
        }
    }

    pub fn set_save_battery_callback(&mut self, callback: &'a dyn Fn(&[u8])) {
        self.battery_save_callback = Some(callback);
    }

    fn update_battery(&mut self) {
        if let Some(ref callback) = self.battery_save_callback {
            let data = self.mem.save_battery();
            if !data.is_empty() {
                callback(&data);
            }
        }
    }

    fn step(&mut self) -> u32 {
        let mut extra_cycles = 0;
        let mut cycles = 0;
        while cycles < self.lcd.stat_mode_duration() + extra_cycles {
            //if !ioregister::LCDCRegister::is_lcd_display_enable(&self.mem) {
            //    self.mem.set_access_vram(true);
            //    self.mem.set_access_oam(true);
            //}

            let (instruction, event_request) = self.cpu.run_instruction(&mut self.mem);
            if let Some(e) = event_request {
                match e {
                    EventRequest::BootstrapDisable => {
                        self.mem.disable_bootstrap();
                    }
                    EventRequest::DMATransfer(l_nibble) => {
                        self.mem.set_access_oam(true);
                        let mut dma_cycles = ioregister::dma_transfer(l_nibble, &mut self.mem);

                        dma_cycles = match self.speed_mode {
                            SpeedMode::Normal => dma_cycles,
                            SpeedMode::Double => dma_cycles * 2,
                        };
                        extra_cycles += dma_cycles;
                        self.mem.set_access_oam(false);
                    }
                    EventRequest::HDMATransfer => {
                        self.mem.set_access_oam(true);
                        let hdma5 = self.mem.read_byte(ioregister::HDMA5_REGISTER_ADDR);
                        if hdma5 >> 7 == 0b1 {
                            // if dma transfer mode is h-blank dma we have to use lcd.
                            self.lcd.request_cgb_dma_transfer();
                        } else if let Some(c) = ioregister::cgb_dma_transfer(&mut self.mem) {
                            extra_cycles += c;
                        }
                        self.mem.set_access_oam(false);
                    }
                    EventRequest::JoypadUpdate => {
                        self.joypad.update_joypad_register(&mut self.mem);
                    }
                    EventRequest::SpeedModeSwitch => {
                        let key1 = self.mem.read_byte(ioregister::KEY1_REGISTER_ADDR);
                        let double_speed = key1 >> 7;
                        self.speed_mode = if double_speed == 0b1 {
                            SpeedMode::Double
                        } else {
                            SpeedMode::Normal
                        };
                    }
                }
            }
            let instr_cycles = match self.speed_mode {
                SpeedMode::Normal => instruction.cycles,
                SpeedMode::Double => instruction.cycles / 2,
            };
            self.cpu.handle_interrupts(&mut self.mem);
            self.timer.update(instr_cycles, &mut self.mem);
            self.apu.borrow_mut().run_for(instr_cycles);
            if cfg!(debug_assertions) {
                self.debugger.run(&instruction, &self.cpu, &self.mem);
                if self.debugger.exit {
                    break;
                }
            }
            cycles += instr_cycles;
        }
        cycles += self.lcd.stat_mode_change(&mut self.mem);
        cycles
    }

    fn set_joypad_key(&mut self, key: JoypadKey, code: Scancode, event_pump: &sdl2::EventPump) {
        if event_pump.keyboard_state().is_scancode_pressed(code) {
            self.joypad.press_key(key);
        } else {
            self.joypad.release_key(key);
        }
    }

    fn adjust_joypad_keys(&mut self, event_pump: &sdl2::EventPump) {
        self.set_joypad_key(JoypadKey::A, Scancode::Z, event_pump);
        self.set_joypad_key(JoypadKey::B, Scancode::X, event_pump);
        self.set_joypad_key(JoypadKey::SELECT, Scancode::LShift, event_pump);
        self.set_joypad_key(JoypadKey::START, Scancode::LCtrl, event_pump);

        self.set_joypad_key(JoypadKey::RIGHT, Scancode::Right, event_pump);
        self.set_joypad_key(JoypadKey::LEFT, Scancode::Left, event_pump);
        self.set_joypad_key(JoypadKey::UP, Scancode::Up, event_pump);
        self.set_joypad_key(JoypadKey::DOWN, Scancode::Down, event_pump);
    }

    fn print_buttons() {
        println!(" Gameboy | Keyboard");
        println!("---------+------------");
        println!("   dir   |  arrows");
        println!("    A    |    Z");
        println!("    B    |    X");
        println!("  start  | left ctrl");
        println!("  select | left shift");
        println!("---------+------------");
        println!("  U: increase speed");
        println!("  I: decrease speed");
        println!("  R: restart");
        println!("  B: bypass nintendo logo");
        println!(" F1: toggle background");
        println!(" F2: toggle window");
        println!(" F3: toggle sprites");
        println!(" F4: toggle pulse A");
        println!(" F5: toggle pulse B");
        println!(" F6: toggle custom wave");
        println!(" F7: toggle white noise");
        println!("Tab: speed up while being held down");
        println!("Esc: quit");
        println!("######################");
    }

    pub fn run_sdl(&mut self) {
        Gebemula::print_buttons();

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let audio_device = audio_subsystem.open_queue(None, &AUDIO_DESIRED_SPEC).unwrap();
        let audio_spec = audio_device.spec();
        let mut audio_buffer = Vec::new();

        self.apu.borrow_mut().set_sample_rate(audio_spec.freq as u32);

        let window = video_subsystem
            .window(
                "Gebemula Emulator",
                graphics::consts::DISPLAY_WIDTH_PX as u32 * 2,
                graphics::consts::DISPLAY_HEIGHT_PX as u32 * 2,
            )
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        let texture_creator = canvas.texture_creator();

        let mut texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::ABGR8888,
                graphics::consts::DISPLAY_WIDTH_PX as u32,
                graphics::consts::DISPLAY_HEIGHT_PX as u32,
            )
            .unwrap();

        canvas.present();
        audio_device.resume();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut last_time_seconds = time::now();
        let mut last_time = time::now();
        let mut frame_time_err = 0;

        let mut speed_mul = 1;
        let target_fps = 60;
        let mut desired_frametime_ns = 1_000_000_000 / target_fps;
        let mut fps = 0;
        if !cfg!(debug_assertions) {
            self.debugger.display_info(&self.mem);
        }
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::B),
                        ..
                    } => {
                        self.cpu.bypass_nintendo_logo(&mut self.mem);
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::F1),
                        ..
                    } => {
                        self.lcd.graphics.toggle_bg();
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::F2),
                        ..
                    } => {
                        self.lcd.graphics.toggle_wn();
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::F3),
                        ..
                    } => {
                        self.lcd.graphics.toggle_sprites();
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::F4),
                        ..
                    } => {
                        println!("Pulse A: {:?}",
                            self.apu.borrow_mut().debug_toggle_channel(0)
                        );
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::F5),
                        ..
                    } => {
                        println!("Pulse B: {:?}",
                            self.apu.borrow_mut().debug_toggle_channel(1)
                        );
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::F6),
                        ..
                    } => {
                        println!("Custom Wave: {:?}",
                            self.apu.borrow_mut().debug_toggle_channel(2)
                        );
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::F7),
                        ..
                    } => {
                        //sound_controller.whitenoise_toggle();
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => {
                        self.debugger.cancel_run();
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => {
                        self.restart();
                        //sound_controller.reset(&mut self.mem);
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::Tab),
                        repeat: false,
                        ..
                    } => {
                        speed_mul += 1;
                        println!("speed x{}", speed_mul);
                        desired_frametime_ns = 1_000_000_000 / (target_fps * speed_mul);
                    }
                    sdl2::event::Event::KeyUp {
                        keycode: Some(Keycode::Tab),
                        repeat: false,
                        ..
                    } => {
                        speed_mul -= 1;
                        println!("speed x{}", speed_mul);
                        desired_frametime_ns = 1_000_000_000 / (target_fps * speed_mul);
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::U),
                        ..
                    } => {
                        speed_mul += 1;
                        if speed_mul >= 15 {
                            speed_mul = 15;
                        }
                        println!("speed x{}", speed_mul);
                        desired_frametime_ns = 1_000_000_000 / (target_fps * speed_mul);
                    }
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::I),
                        ..
                    } => {
                        speed_mul -= 1;
                        if speed_mul == 0 {
                            speed_mul = 1;
                        }
                        println!("speed x{}", speed_mul);
                        desired_frametime_ns = 1_000_000_000 / (target_fps * speed_mul);
                    }
                    sdl2::event::Event::Quit { .. }
                    | sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            self.adjust_joypad_keys(&event_pump);
            let cycles_ran = self.step();
            self.cycles_per_sec += cycles_ran;

            self.feed_audio(&audio_device, &mut audio_buffer);

            if !cfg!(debug_assertions) {
                if self.debugger.exit {
                    break 'running;
                }
            }

            /*
             * Yuri Kunde Schlesner:
             * it's just the way you do it (fps checking)  seems brittle and
             * you'll get error depending on your timing
             * instead of counting "each >= 1 second check how many frames
             * were rendered and show that as fps", you should either do
             * "each >= 1 second check how many frame were rendered / *actual*
             * elapsed time since last reset of fps"
             * or "each N frames, check elapsed time since last fps update and
             * calculate based on that" fps is just 1 / frametime, so you should
             * just try to average frametime over time to calculate it imo
             *
             * https://github.com/yuriks/super-match-5-dx/blob/master/src/main.cpp#L224
             */
            if self.lcd.has_entered_vblank(&self.mem) {
                texture
                    .update(
                        None,
                        &self.lcd.graphics.screen_buffer,
                        graphics::consts::DISPLAY_WIDTH_PX as usize * 4,
                    )
                    .unwrap();
                canvas.clear();
                if let Err(_) = canvas.copy(&texture, None, None) {
                    println!("Unable to draw texture to canvas!");
                    return;
                }
                canvas.present();

                //clear buffer
                for p in self.lcd.graphics.screen_buffer.chunks_mut(4) {
                    // This actually makes the code faster by skipping redundant bound checking:
                    assert_eq!(p.len(), 4);

                    let color = match GBMode::get(&self.mem) {
                        GBMode::Color => {
                            //TODO: remove hardcoded stuff?
                            (255, 255, 255) //all white
                        }
                        GBMode::Mono => graphics::consts::DMG_PALETTE[0],
                    };
                    p[0] = color.0;
                    p[1] = color.1;
                    p[2] = color.2;
                    p[3] = 255;
                }

                frame_time_err += desired_frametime_ns;
                let now = time::now();
                let elapsed = (now - last_time).num_nanoseconds().unwrap() as i64;
                frame_time_err -= elapsed;
                if frame_time_err > 0 {
                    thread::sleep(std::time::Duration::new(0, frame_time_err as u32));
                }
                last_time = now;
                fps += 1;
            }

            let now = time::now();
            if now - last_time_seconds >= time::Duration::seconds(1) {
                last_time_seconds = now;
                let title = &format!("{} Gebemula - {}", fps, self.cycles_per_sec);
                canvas.window_mut().set_title(title).unwrap();
                self.cycles_per_sec = 0;
                fps = 0;

                self.update_battery();
            }
        }
        self.update_battery();
    }

    fn feed_audio(&mut self, audio_device: &AudioQueue<i16>, audio_buffer: &mut Vec<i16>) {
        self.apu.borrow_mut().generate_audio(audio_buffer);
        let current_audio_buf = audio_device.size();
        audio_device.queue(audio_buffer.as_ref());
        if current_audio_buf < 128 {
            //println!("Audio buffer underrun: {} (adding {})", current_audio_buf, audio_buffer.len());
        }
        audio_buffer.clear();
    }
}
