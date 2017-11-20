use super::super::mem::Memory;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioDevice, AudioStatus, AudioCallback, AudioSpecDesired};

use super::super::cpu::ioregister::CPU_FREQUENCY_HZ;

// PulseAReg registers
const NR10_REGISTER_ADDR: u16 = 0xFF10;
const NR11_REGISTER_ADDR: u16 = 0xFF11;
const NR12_REGISTER_ADDR: u16 = 0xFF12;
const NR13_REGISTER_ADDR: u16 = 0xFF13;
const NR14_REGISTER_ADDR: u16 = 0xFF14;

const NR21_REGISTER_ADDR: u16 = 0xFF16;
const NR22_REGISTER_ADDR: u16 = 0xFF17;
const NR23_REGISTER_ADDR: u16 = 0xFF18;
const NR24_REGISTER_ADDR: u16 = 0xFF19;

// Global sound registers
const NR50_REGISTER_ADDR: u16 = 0xFF24;
const NR51_REGISTER_ADDR: u16 = 0xFF25;
const NR52_REGISTER_ADDR: u16 = 0xFF26;

const FREQ: i32 = 44100i32;

trait VoiceReg {
    fn control(&self) -> u16;
    fn frequency(&self) -> u16;
    fn volume(&self) -> u16;
    fn length(&self) -> u16;
    fn sweep(&self) -> Option<u16>;
    fn sound_num(&self) -> u8; // 0 to 3

    fn read_control(&self, memory: &Memory) -> u8 {
        memory.read_byte(self.control())
    }
    fn read_frequency(&self, memory: &Memory) -> u8 {
        memory.read_byte(self.frequency())
    }
    fn read_volume(&self, memory: &Memory) -> u8 {
        memory.read_byte(self.volume())
    }
    fn read_length(&self, memory: &Memory) -> u8 {
        memory.read_byte(self.length())
    }
    fn read_sweep(&self, memory: &Memory) -> Option<u8> {
        if let Some(sweep_reg) = self.sweep() {
            Some(memory.read_byte(sweep_reg))
        } else {
            None
        }
    }
}

struct PulseAReg;
impl VoiceReg for PulseAReg {
    fn control(&self) -> u16 {
        NR14_REGISTER_ADDR
    }
    fn frequency(&self) -> u16 {
        NR13_REGISTER_ADDR
    }
    fn volume(&self) -> u16 {
        NR12_REGISTER_ADDR
    }
    fn length(&self) -> u16 {
        NR11_REGISTER_ADDR
    }
    fn sweep(&self) -> Option<u16> {
        Some(NR10_REGISTER_ADDR)
    }
    fn sound_num(&self) -> u8 {
        0
    }
}
struct PulseBReg;
impl VoiceReg for PulseBReg {
    fn control(&self) -> u16 {
        NR21_REGISTER_ADDR
    }
    fn frequency(&self) -> u16 {
        NR22_REGISTER_ADDR
    }
    fn volume(&self) -> u16 {
        NR23_REGISTER_ADDR
    }
    fn length(&self) -> u16 {
        NR24_REGISTER_ADDR
    }
    fn sweep(&self) -> Option<u16> {
        None
    }
    fn sound_num(&self) -> u8 {
        1
    }
}

struct SweepInfo {
    time: f32, // seconds
    increase: bool,
    shift: u8,
}

struct LengthInfo {
    duty_cycles: f32,
    sound_length: f32, // seconds
}

enum EnvelopeDirection {
    Attenuate,
    Amplify,
}

struct VolumeInfo {
    initial_volume: u8,
    env_dir: EnvelopeDirection,
    //env_num_sweep: u8,
    step_len: f32, // length of 1 step in sec.
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum VoiceTrigger {
    On,
    Off,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum VoiceSelection {
    Loop,
    NoLoop,
}

struct ControlInfo {
    frequency: u16,
    frequency_hz: f32,
    trigger: VoiceTrigger,
    selection: VoiceSelection,
}

struct Voice {
    reg: Box<VoiceReg>,
    output_channel_1: bool, // should it output to channel 1/2?
    output_channel_2: bool,
    cycles: u32, // amount of cycles the voice has been on for.
}

impl Voice {
    pub fn new(voice_reg: Box<VoiceReg>) -> Self {
        Voice {
            reg: voice_reg,
            output_channel_1: false,
            output_channel_2: false,
            cycles: 0,
        }
    }

    pub fn update(&mut self, memory: &mut Memory) {
        let output_terminal = memory.read_byte(NR51_REGISTER_ADDR);
        self.output_channel_1 = ((output_terminal >> self.reg.sound_num()) & 0b1) == 0b1;
        self.output_channel_2 = ((output_terminal >> (self.reg.sound_num() + 4)) & 0b1) == 0b1;
        if self.should_play(memory) {
            let sound_onoff = memory.read_byte(NR52_REGISTER_ADDR);
            // update flag of sound ON in io register.
            memory.write_byte(
                NR52_REGISTER_ADDR,
                sound_onoff | (1u8 << self.reg.sound_num()),
            );
        }
    }

    // TODO: make sure VoiceTrigger::On should be here
    pub fn should_play(&self, memory: &Memory) -> bool {
        self.control(memory).trigger == VoiceTrigger::On &&
            (self.output_channel_1 || self.output_channel_2)
    }

    pub fn sweep(&self, memory: &Memory) -> Option<SweepInfo> {
        if let Some(sweep_info) = self.reg.read_sweep(memory) {
            let res = SweepInfo {
                time: ((sweep_info & 0b111) >> 4) as f32 / 128f32,
                increase: (sweep_info & 0b0000_1000) == 0b0000_0000,
                shift: sweep_info & 0b111,
            };
            Some(res)
        } else {
            None
        }
    }

    pub fn length(&self, memory: &Memory) -> LengthInfo {
        let length_info = self.reg.read_length(memory);
        LengthInfo {
            duty_cycles: match length_info >> 6 {
                0b00 => 0.125,
                0b01 => 0.25,
                0b10 => 0.5,
                0b11 => 0.75,
                _ => unreachable!(),
            },
            sound_length: (64f32 - (length_info & 0b0011_1111) as f32) * (1f32 / 256f32),
        }
    }

    pub fn volume(&self, memory: &Memory) -> VolumeInfo {
        let vol_info = self.reg.read_volume(memory);
        let n = vol_info & 0b111;
        VolumeInfo {
            initial_volume: vol_info >> 4,
            env_dir: if (vol_info >> 3) & 0b1 == 0b0 {
                EnvelopeDirection::Attenuate
            } else {
                EnvelopeDirection::Amplify
            },
            //env_num_sweep: n,
            step_len: n as f32 * (1f32 / 64f32),
        }
    }

    pub fn control(&self, memory: &Memory) -> ControlInfo {
        let freq = self.reg.read_frequency(memory);
        let control = self.reg.read_control(memory);
        let frequency = (((control & 0b111) as u16) << 8) | freq as u16;
        ControlInfo {
            frequency: frequency,
            frequency_hz: 131072f32 / (2048f32 - frequency as f32),
            trigger: if (control >> 7) == 0b1 {
                VoiceTrigger::On
            } else {
                VoiceTrigger::Off
            },
            selection: if ((control >> 6) & 0b1) == 0b1 {
                VoiceSelection::Loop
            } else {
                VoiceSelection::NoLoop
            },
        }
    }

    pub fn reset(&mut self, memory: &mut Memory) {
        // reset sound trigger.
        let control = self.reg.read_control(memory);
        memory.write_byte(self.reg.control(), control & 0b0111_1111);
        // reset ON flag.
        let sound_onoff = memory.read_byte(NR52_REGISTER_ADDR);
        memory.write_byte(
            NR52_REGISTER_ADDR,
            sound_onoff & !(1 << self.reg.sound_num()),
        );
        self.cycles = 0;
    }
}

// works as a PulseController for now
struct VoiceController {
    voice: Voice,
    device: AudioDevice<SquareWave>,
}

impl VoiceController {
    pub fn new(voice_reg: Box<VoiceReg>, device: AudioDevice<SquareWave>) -> Self {
        VoiceController {
            voice: Voice::new(voice_reg),
            device: device,
        }
    }

    fn adjust_device(&mut self, memory: &mut Memory) {
        let len = self.voice.length(memory);
        let ctrl = self.voice.control(memory);
        let vol = self.voice.volume(memory);
        let mut lock = self.device.lock();
        (*lock).volume = vol.initial_volume as f32;
        (*lock).phase_inc = ctrl.frequency_hz / FREQ as f32;
        (*lock).duty = len.duty_cycles;
    }

    fn reset(&mut self, memory: &mut Memory) {
        if self.device.status() == AudioStatus::Playing {
            self.device.pause();
            self.voice.reset(memory);
            let mut lock = self.device.lock();
            (*lock).phase = 0.0;
        }
    }
    pub fn run(&mut self, cycles: u32, memory: &mut Memory) {
        self.voice.update(memory);
        if self.voice.should_play(memory) {
            self.adjust_device(memory);

            let vol = self.voice.volume(memory);
            let ctrl = self.voice.control(memory);
            let len = self.voice.length(memory);
            // handle envelope
            if vol.step_len > 0f32 {
                let step_duration_cycles = (CPU_FREQUENCY_HZ as f32 * vol.step_len) as u32;
                let last_step = self.voice.cycles / step_duration_cycles;
                let curr_step = (self.voice.cycles + cycles) / step_duration_cycles;
                if curr_step > last_step {
                    let mult = match vol.env_dir {
                        EnvelopeDirection::Amplify => 1f32,
                        EnvelopeDirection::Attenuate => -1f32,
                    };
                    // TODO: make sure the logic below is correct
                    // my assumption that this addition will go up from 1 to 1 may be wrong!
                    let new_volume = vol.initial_volume as f32 + (mult * curr_step as f32) as f32;
                    if new_volume >= 0f32 && new_volume <= 15f32 {
                        let mut lock = self.device.lock();
                        (*lock).volume = new_volume;
                    }
                }
            }

            let mut play_sound = true;
            // handle sweep
            if let Some(sweep) = self.voice.sweep(memory) {
                // only apply sweep if the sweep duration is > 0 and the shift number ain't
                // zero.
                if sweep.time > 0f32 && sweep.shift > 0 {
                    let sweep_duration_cycles = (CPU_FREQUENCY_HZ as f32 * sweep.time) as u32;
                    let last_sweep = self.voice.cycles / sweep_duration_cycles;
                    let curr_sweep = (self.voice.cycles + cycles) / sweep_duration_cycles;
                    if curr_sweep > last_sweep {
                        let mult = if sweep.increase { 1f32 } else { -1f32 };
                        let new_freq = ctrl.frequency as u32 +
                            ((mult * ctrl.frequency as f32) / 2f32.powf(sweep.shift as f32)) as u32;
                        // if new frequency has more than 11bits, the sound should be stopped.
                        if new_freq > 0b0111_1111_1111 {
                            play_sound = false;
                        } else {
                            // adjust frequency
                            memory.write_byte(NR13_REGISTER_ADDR, new_freq as u8);
                            let nr14 = memory.read_byte(NR14_REGISTER_ADDR) & 0b1111_1000;
                            memory.write_byte(NR14_REGISTER_ADDR, nr14 | (new_freq >> 8) as u8);

                            let mut lock = self.device.lock();
                            (*lock).phase_inc = self.voice.control(memory).frequency_hz /
                                FREQ as f32;
                        }
                    }
                }
            }

            // TODO: maybe this can overflow, should wrap add or do something else?
            self.voice.cycles += cycles;
            // if the sound shouldn't loop, we have to count the elapsed time in order to stop
            // it after its total length.
            if ctrl.selection == VoiceSelection::NoLoop {
                let duration_cycles = (CPU_FREQUENCY_HZ as f32 * len.sound_length) as u32;
                if self.voice.cycles >= duration_cycles {
                    play_sound = false;
                }
            }

            if play_sound {// && self.device.status() != AudioStatus::Playing {
                // play the sound
                self.device.resume();
            } else {
                self.reset(memory);
            }
        } else {
            self.reset(memory);
        }
    }
}

pub struct SoundController {
    sound_is_on: bool,
    channel_1_volume: u8,
    channel_2_volume: u8,
    pulse_a: VoiceController,
    pulse_b: VoiceController,
}

impl SoundController {
    pub fn new(audio_subsystem: &AudioSubsystem) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(FREQ),
            channels: Some(2), // mono
            samples: None, // default sample size
        };
        SoundController {
            sound_is_on: false,
            channel_1_volume: 0,
            channel_2_volume: 0,
            pulse_a: VoiceController::new(
                Box::new(PulseAReg),
                audio_subsystem.open_playback(None, &desired_spec, |_| {
                    SquareWave {
                        phase_inc: 0f32,
                        phase: 0f32,
                        volume: 0f32,
                        duty: 0f32,
                    }
                }).unwrap(),
            ),
            pulse_b: VoiceController::new(
                Box::new(PulseBReg),
                audio_subsystem.open_playback(None, &desired_spec, |_| {
                    SquareWave {
                        phase_inc: 0f32,
                        phase: 0f32,
                        volume: 0f32,
                        duty: 0f32,
                    }
                }).unwrap(),
            ),
        }
    }
    pub fn reset(&mut self, memory: &mut Memory) {
        self.pulse_a.reset(memory);
        self.pulse_b.reset(memory);
    }

    pub fn run(&mut self, cycles: u32, memory: &mut Memory) {
        let sound_onoff = memory.read_byte(NR52_REGISTER_ADDR);

        self.sound_is_on = (sound_onoff >> 7) == 0b1;
        if self.sound_is_on {
            let channel_ctrl = memory.read_byte(NR50_REGISTER_ADDR);
            self.channel_1_volume = channel_ctrl & 0b111;
            self.channel_2_volume = (channel_ctrl >> 4) & 0b111;

            self.pulse_a.run(cycles, memory);
            self.pulse_b.run(cycles, memory);
        } else {
            self.reset(memory);
        }
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
    duty: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= self.duty {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
