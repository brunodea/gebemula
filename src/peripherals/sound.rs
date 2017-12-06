use super::super::mem::Memory;
use super::super::cpu::ioregister::CPU_FREQUENCY_HZ;
use sdl2::AudioSubsystem;
use sdl2::audio::{AudioStatus, AudioDevice, AudioQueue, AudioCallback, AudioSpecDesired};

// PulseAVoice registers
pub const NR10_REGISTER_ADDR: u16 = 0xFF10;
pub const NR11_REGISTER_ADDR: u16 = 0xFF11;
pub const NR12_REGISTER_ADDR: u16 = 0xFF12;
pub const NR13_REGISTER_ADDR: u16 = 0xFF13;
pub const NR14_REGISTER_ADDR: u16 = 0xFF14;

// PulseBReg registers
pub const NR21_REGISTER_ADDR: u16 = 0xFF16;
pub const NR22_REGISTER_ADDR: u16 = 0xFF17;
pub const NR23_REGISTER_ADDR: u16 = 0xFF18;
pub const NR24_REGISTER_ADDR: u16 = 0xFF19;

// Wave registers
pub const NR30_REGISTER_ADDR: u16 = 0xFF1A;
pub const NR31_REGISTER_ADDR: u16 = 0xFF1B;
pub const NR32_REGISTER_ADDR: u16 = 0xFF1C;
pub const NR33_REGISTER_ADDR: u16 = 0xFF1D;
pub const NR34_REGISTER_ADDR: u16 = 0xFF1E;

// Global sound registers
pub const NR50_REGISTER_ADDR: u16 = 0xFF24;
pub const NR51_REGISTER_ADDR: u16 = 0xFF25;
pub const NR52_REGISTER_ADDR: u16 = 0xFF26;

const CUSTOM_WAVE_START_ADDR: u16 = 0xFF30;
const CUSTOM_WAVE_END_ADDR: u16 = 0xFF3F;

const SWEEP_CYCLES_PER_STEP: u32 = CPU_FREQUENCY_HZ / 128;
const ENVELOPE_CYCLES_PER_STEP: u32 = CPU_FREQUENCY_HZ / 64; // every 'envelope step' has cycles.

// TODO make sure it is 44100 and not some other thing such as 48000.
const FREQ: i32 = 44_100i32;
const SQUARE_DESIRED_SPEC: AudioSpecDesired = AudioSpecDesired {
    freq: Some(FREQ),
    channels: Some(1), // mono
    samples: None, // default sample size
};

#[derive(Copy, Clone, PartialEq, Debug)]
enum SweepFunc {
    Addition,
    Subtraction,
}

struct Sweep {
    shift_number: u8, // 0-7
    func: SweepFunc,
    sweep_time: u8,
    addr: u16,
    start_time_cycles: Option<u32>,
}

impl Sweep {
    fn new(addr: u16, memory: &Memory) -> Self {
        let sweep_raw = memory.read_byte(addr);
        let shift_number = sweep_raw & 0b111;
        let func = if ((sweep_raw >> 3) & 0b1) == 0b0 {
            SweepFunc::Addition
        } else {
            SweepFunc::Subtraction
        };
        let sweep_time = (sweep_raw >> 4) & 0b111;

        Sweep {
            shift_number: shift_number,
            func: func,
            sweep_time: sweep_time,
            addr: addr,
            start_time_cycles: None,
        }
    }

    // returns new frequency value or the old one or None if it is off. Err is returned if the
    // sound output should stop.
    fn update(&mut self, cycles: u32, old_frequency: u16, memory: &Memory) -> Result<Option<u16>, ()> {
        let sweep_raw = memory.read_byte(self.addr);
        self.shift_number = sweep_raw & 0b111;
        self.func = if ((sweep_raw >> 3) & 0b1) == 0b0 {
            SweepFunc::Addition
        } else {
            SweepFunc::Subtraction
        };
        self.sweep_time = (sweep_raw >> 4) & 0b111;

        let mut new_freq_result = Ok(Some(old_frequency));
        self.start_time_cycles = if self.sweep_time > 0 {
            if self.start_time_cycles.is_none() {
                Some(cycles)
            } else {
                self.start_time_cycles
            }
        } else {
            new_freq_result = None;
            None
        };

        if let Some(sweep_start_time_cycles) = self.start_time_cycles {
            let steps = (cycles - sweep_start_time_cycles) / SWEEP_CYCLES_PER_STEP;
            if steps >= self.sweep_time as u32 {
                self.start_time_cycles = Some(cycles);
                let rhs = old_frequency as f32 / 2f32.powi(self.shift_number as i32);
                let new_freq = match self.func {
                    SweepFunc::Addition => old_frequency + rhs as u16,
                    SweepFunc::Subtraction => {
                        // TODO: maybe we should wrapping_sub instead?
                        if rhs > old_frequency as f32 {
                            0u16
                        } else {
                            old_frequency - rhs as u16
                        }
                    }
                };
                if new_freq > 0b0000_0111_1111_1111 {
                    return Err(());
                } else {
                    new_freq_result = Ok(Some(new_freq));
                }
            }
        }

        new_freq_result
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum EnvelopeFunc {
    Attenuate,
    Amplify,
}

struct Envelope {
    step_length: u8, // 0-7
    func: EnvelopeFunc,
    default_value: u8, // 0x0-0xF
    addr: u16,
    start_time_cycles: Option<u32>,
}

impl Envelope {
    fn new(addr: u16, memory: &Memory) -> Self {
        let envelope_raw = memory.read_byte(addr);
        let step_length = envelope_raw & 0b111;
        let func = if ((envelope_raw >> 3) & 0b1) == 0b0 {
            EnvelopeFunc::Attenuate
        } else {
            EnvelopeFunc::Amplify
        };
        let default_value = envelope_raw >> 4;

        Envelope {
            step_length: step_length,
            func: func,
            default_value: default_value,
            addr: addr,
            start_time_cycles: None,
        }
    }
    fn update(&mut self, cycles: u32, memory: &mut Memory) {
        let envelope_raw = memory.read_byte(self.addr);
        self.step_length = envelope_raw & 0b111;
        self.func = if ((envelope_raw >> 3) & 0b1) == 0b0 {
            EnvelopeFunc::Attenuate
        } else {
            EnvelopeFunc::Amplify
        };
        self.default_value = envelope_raw >> 4;
        self.start_time_cycles = if self.step_length > 0 {
            if self.start_time_cycles.is_none() {
                Some(cycles)
            } else {
                self.start_time_cycles
            }
        } else {
            None
        };

        if let Some(start_time_cycles) = self.start_time_cycles {
            let steps = (cycles - start_time_cycles) / ENVELOPE_CYCLES_PER_STEP;
            if steps >= self.step_length as u32 + 1 {
                self.start_time_cycles = Some(cycles);
                self.default_value = match self.func {
                    EnvelopeFunc::Amplify => {
                        if self.default_value == 0xF {
                            self.default_value 
                        } else {
                            self.default_value + 1
                        }
                        //self.default_value.wrapping_add(1)
                    },
                    EnvelopeFunc::Attenuate => {
                        if self.default_value == 0 {
                            self.default_value
                        } else {
                            self.default_value - 1
                        }
                        //self.default_value.wrapping_sub(1)
                    },
                };

                let nr2 = memory.read_byte(self.addr);
                memory.write_byte(self.addr, (nr2 & 0b0000_1111) | (self.default_value << 4));// & self.step_length);
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum SoundLoop {
    Loop,
    NoLoop,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum SoundTrigger {
    On,
    Off,
}

struct PulseVoice {
    sweep: Option<Sweep>,
    sound_length: u8, // 0-63
    waveform_duty_cycles: f32,
    envelope: Envelope,
    frequency: u16, // 11bits
    sound_loop: SoundLoop,
    sound_trigger: SoundTrigger,
    start_time_cycles: Option<u32>,
    device: AudioDevice<SquareWave>,
    voice_type: VoiceType,
    nr1_reg: u16,
    nr3_reg: u16,
    nr4_reg: u16,
}

impl PulseVoice {
    fn new(voice_type: VoiceType, audio_subsystem: &AudioSubsystem, memory: &Memory) -> Self {
        let (sweep, nr1_reg, nr2_reg, nr3_reg, nr4_reg) = match voice_type {
            VoiceType::PulseA => {
                (
                    Some(Sweep::new(NR10_REGISTER_ADDR, memory)),
                    NR11_REGISTER_ADDR,
                    NR12_REGISTER_ADDR,
                    NR13_REGISTER_ADDR,
                    NR14_REGISTER_ADDR,
                )
            }
            VoiceType::PulseB => {
                (
                    None,
                    NR21_REGISTER_ADDR,
                    NR22_REGISTER_ADDR,
                    NR23_REGISTER_ADDR,
                    NR24_REGISTER_ADDR,
                )
            }
            _ => panic!(),
        };

        let nr1 = memory.read_byte(nr1_reg);
        let nr3 = memory.read_byte(nr3_reg);
        let nr4 = memory.read_byte(nr4_reg);

        let sound_length = nr1 & 0b0011_1111;
        let waveform_duty_cycles = match nr1 >> 6 {
            0b00 => 0.125f32,
            0b01 => 0.25f32,
            0b10 => 0.50f32,
            0b11 => 0.75f32,
            _ => unreachable!(),
        };
        let frequency = ((nr4 as u16 & 0b111) << 8) | nr3 as u16;
        let sound_loop = if ((nr4 >> 6) & 0b1) == 0b0 {
            SoundLoop::Loop
        } else {
            SoundLoop::NoLoop
        };
        let sound_trigger = if ((nr4 >> 7) & 0b1) == 0b0 {
            SoundTrigger::Off
        } else {
            SoundTrigger::On
        };

        PulseVoice {
            sweep: sweep,
            sound_length: sound_length,
            waveform_duty_cycles: waveform_duty_cycles,
            envelope: Envelope::new(nr2_reg, memory),
            frequency: frequency,
            sound_loop: sound_loop,
            sound_trigger: sound_trigger,
            start_time_cycles: None,
            device: audio_subsystem
                .open_playback(None, &SQUARE_DESIRED_SPEC, |_| {
                    SquareWave {
                        phase_inc: 0f32,
                        phase: 0f32,
                        volume: 0f32,
                        duty: 0f32,
                    }
                })
                .unwrap(),
            voice_type: voice_type,
            nr1_reg: nr1_reg,
            nr3_reg: nr3_reg,
            nr4_reg: nr4_reg,
        }
    }

    fn update(&mut self, memory: &mut Memory) {
        let nr1 = memory.read_byte(self.nr1_reg);
        let nr3 = memory.read_byte(self.nr3_reg);
        let nr4 = memory.read_byte(self.nr4_reg);

        self.sound_length = nr1 & 0b0011_1111;
        self.waveform_duty_cycles = match (nr1 >> 6) & 0b11 {
            0b00 => 0.125f32,
            0b01 => 0.25f32,
            0b10 => 0.50f32,
            0b11 => 0.75f32,
            _ => unreachable!(),
        };
        self.frequency = ((nr4 as u16 & 0b111) << 8) | nr3 as u16;
        self.sound_loop = if ((nr4 >> 6) & 0b1) == 0b0 {
            SoundLoop::Loop
        } else {
            SoundLoop::NoLoop
        };

        self.sound_trigger = if ((nr4 >> 7) & 0b1) == 0b1 {
            SoundTrigger::On
        } else {
            SoundTrigger::Off
        };
    }

    fn update_device(&mut self, memory: &Memory) {
        let frequency_hz = 131072f32 / (2048f32 - self.frequency as f32);
        let mut volume =
            if GlobalReg::should_output(self.voice_type, ChannelNum::ChannelA, memory) {
                GlobalReg::output_level(ChannelNum::ChannelA, memory) as f32
            } else if GlobalReg::should_output(self.voice_type, ChannelNum::ChannelB, memory) {
                GlobalReg::output_level(ChannelNum::ChannelB, memory) as f32
            } else {
                0f32
            };
        if self.envelope.step_length > 0 {
            volume *= self.envelope.default_value as f32 / 0xF as f32;
        }
        let mut lock = self.device.lock();
        (*lock).phase_inc = frequency_hz / FREQ as f32;
        (*lock).volume = volume;
        (*lock).duty = self.waveform_duty_cycles;
    }

    fn run(&mut self, cycles: u32, memory: &mut Memory) {
        self.update(memory);
        if self.sound_trigger == SoundTrigger::On {
            // update ON flag all the time.
            GlobalReg::set_voice_flag(self.voice_type, memory);

            if self.start_time_cycles.is_none() {
                // first loop with sound on.
                // things here should be run only once when the sound is on.
                self.update_device(memory);
                self.device.resume();
                self.start_time_cycles = Some(cycles);
            }

            self.envelope.update(cycles, memory);

            let mut should_stop = false;
            // handle sweep
            if let Some(ref mut sweep) = self.sweep {
                let s = sweep.update(cycles, self.frequency, memory);
                match s {
                    Ok(opt) => {
                        if let Some(new_freq) = opt {
                            let nr14 = memory.read_byte(self.nr4_reg);
                            memory.write_byte(self.nr3_reg, new_freq as u8);
                            memory.write_byte(self.nr4_reg, nr14 | (new_freq >> 8) as u8);
                        }
                    },
                    Err(_) => {
                        should_stop = true;
                    },
                }
            }

            if self.sound_loop == SoundLoop::NoLoop {
                let sound_length_cycles = CPU_FREQUENCY_HZ * (64 - self.sound_length as u32) / 256;
                if cycles - self.start_time_cycles.unwrap() >= sound_length_cycles {
                    should_stop = true;
                }
            }

            if should_stop {
                self.stop(memory);
            }

            self.update_device(memory);
        } else {
            self.stop(memory);
        }
    }

    fn stop(&mut self, memory: &mut Memory) {
        // this if is for extra safety
        if self.device.status() == AudioStatus::Playing {
            let mut lock = self.device.lock();
            (*lock).volume = 0.0;
            (*lock).duty = 0.0;

            self.start_time_cycles = None;
            GlobalReg::reset_voice_flag(self.voice_type, memory);
            // reset initialize (trigger) flag
            let nr4 = memory.read_byte(self.nr4_reg);
            memory.write_byte(self.nr4_reg, nr4 & 0b0111_1111);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum SoundEnable {
    Enabled,
    Disabled,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum OutputLevelSelection {
    Mute,
    Unmodified,
    Shifted1, // waveform shifted 1bit to the right
    Shifted2, // waveform shifted 2bits to the right
}

struct WaveVoice {
    sound_enable: SoundEnable,
    sound_length: u8,
    frequency: u16, // 11bits
    output_level: OutputLevelSelection,
    sound_loop: SoundLoop,
    sound_trigger: SoundTrigger,
    device: AudioQueue<u8>,
    start_time_cycles: Option<u32>,
    curr_phase: f32,
    wave: Vec<u8>,
}

impl WaveVoice {
    fn new(audio_subsystem: &AudioSubsystem, memory: &Memory) -> Self {
        let nr30 = memory.read_byte(NR30_REGISTER_ADDR);
        let nr32 = memory.read_byte(NR32_REGISTER_ADDR);
        let nr33 = memory.read_byte(NR33_REGISTER_ADDR);
        let nr34 = memory.read_byte(NR34_REGISTER_ADDR);
        let sound_enable = match nr30 >> 7 {
            0b0 => SoundEnable::Disabled,
            0b1 => SoundEnable::Enabled,
            _ => unreachable!(),
        };
        let sound_length = memory.read_byte(NR31_REGISTER_ADDR);
        let output_level = match (nr32 >> 5) & 0b11 {
            0b00 => OutputLevelSelection::Mute,
            0b01 => OutputLevelSelection::Unmodified,
            0b10 => OutputLevelSelection::Shifted1,
            0b11 => OutputLevelSelection::Shifted2,
            _ => unreachable!(),
        };
        let frequency = ((nr34 as u16 & 0b111) << 8) | nr33 as u16;
        let sound_loop = if ((nr34 >> 6) & 0b1) == 0b0 {
            SoundLoop::Loop
        } else {
            SoundLoop::NoLoop
        };
        let sound_trigger = if ((nr34 >> 7) & 0b1) == 0b0 {
            SoundTrigger::Off
        } else {
            SoundTrigger::On
        };

        WaveVoice {
            sound_enable: sound_enable,
            sound_length: sound_length,
            frequency: frequency,
            output_level: output_level,
            sound_loop: sound_loop,
            sound_trigger: sound_trigger,
            device: audio_subsystem
                .open_queue(None, &SQUARE_DESIRED_SPEC)
                .unwrap(),
            start_time_cycles: None,
            curr_phase: 0.0,
            wave: vec![0; FREQ as usize],
        }
    }

    fn update(&mut self, memory: &mut Memory) {
        let nr30 = memory.read_byte(NR30_REGISTER_ADDR);
        let nr32 = memory.read_byte(NR32_REGISTER_ADDR);
        let nr33 = memory.read_byte(NR33_REGISTER_ADDR);
        let nr34 = memory.read_byte(NR34_REGISTER_ADDR);
        self.sound_enable = match nr30 >> 7 {
            0b0 => SoundEnable::Disabled,
            0b1 => SoundEnable::Enabled,
            _ => unreachable!(),
        };
        self.sound_length = memory.read_byte(NR31_REGISTER_ADDR);
        self.output_level = match (nr32 >> 5) & 0b11 {
            0b00 => OutputLevelSelection::Mute,
            0b01 => OutputLevelSelection::Unmodified,
            0b10 => OutputLevelSelection::Shifted1,
            0b11 => OutputLevelSelection::Shifted2,
            _ => unreachable!(),
        };
        self.frequency = ((nr34 as u16 & 0b111) << 8) | nr33 as u16;
        self.sound_loop = if ((nr34 >> 6) & 0b1) == 0b0 {
            SoundLoop::Loop
        } else {
            SoundLoop::NoLoop
        };
        self.sound_trigger = if ((nr34 >> 7) & 0b1) == 0b0 {
            SoundTrigger::Off
        } else {
            SoundTrigger::On
        };
    }

    fn update_wave(&mut self, memory: &Memory) {
        let frequency_hz = 65536f32 / (2048f32 - self.frequency as f32);
        let channel_volume =
            if GlobalReg::should_output(VoiceType::Wave, ChannelNum::ChannelA, memory) {
                GlobalReg::output_level(ChannelNum::ChannelA, memory)
            } else if GlobalReg::should_output(VoiceType::Wave, ChannelNum::ChannelB, memory) {
                GlobalReg::output_level(ChannelNum::ChannelB, memory)
            } else {
                0
            };
        let phase_inc = frequency_hz / FREQ as f32;

        let out_level = self.output_level;
        let volume = |slot| -> u8 {
            match out_level {
                OutputLevelSelection::Unmodified => slot * channel_volume, // 100%
                OutputLevelSelection::Shifted1 => (slot >> 1) * channel_volume, // 50%
                OutputLevelSelection::Shifted2 => (slot >> 2) * channel_volume, // 25%
                OutputLevelSelection::Mute => 0, // 0%
            }
        };

        let mut addr = CUSTOM_WAVE_START_ADDR;
        for i in 0..self.wave.len() {
            let entry = memory.read_byte(addr);
            let s = if i % 2 == 0 {
                entry >> 4
            } else {
                if addr < CUSTOM_WAVE_END_ADDR {
                    addr += 1;
                }
                entry & 0b1111
            };

            self.wave[i] = volume(s);

            self.curr_phase = self.curr_phase + phase_inc;
            if self.curr_phase > 1.0 {
                // duty finished
                self.curr_phase = self.curr_phase % 1.0;
                addr = CUSTOM_WAVE_START_ADDR;
            }
        }
        self.device.clear();
        self.device.queue(self.wave.as_slice());
    }

    fn run(&mut self, cycles: u32, memory: &mut Memory) {
        self.update(memory);
        // TODO: should we also check if at least one output channel is on?
        //(GlobalReg::should_output(VoiceType::PulseA, ChannelNum::ChannelA) ||
        // GlobalReg::should_output(VoiceType::PulseA, ChannelNum::ChannelB))
        if self.sound_trigger == SoundTrigger::On && self.sound_enable == SoundEnable::Enabled {
            GlobalReg::set_voice_flag(VoiceType::Wave, memory);
            if self.start_time_cycles.is_none() {
                // first loop with sound on.
                // things here should be run only once when the sound is on.
                self.update_wave(memory);
                self.device.resume();
                self.start_time_cycles = Some(cycles);
            }
            if self.sound_loop == SoundLoop::NoLoop {
                let sound_length_cycles = CPU_FREQUENCY_HZ * (256 - self.sound_length as u32) / 256;
                if cycles - self.start_time_cycles.unwrap() >= sound_length_cycles {
                    self.stop(memory);
                }
            }

        } else {
            self.stop(memory);
        }
    }
    fn stop(&mut self, memory: &mut Memory) {
        if self.start_time_cycles.is_some() {
            self.device.clear();
            self.wave = vec![0; FREQ as usize];
            self.device.queue(self.wave.as_slice());

            self.start_time_cycles = None;
            GlobalReg::reset_voice_flag(VoiceType::Wave, memory);
            // reset initialize (trigger) flag
            let nr34 = memory.read_byte(NR34_REGISTER_ADDR);
            memory.write_byte(NR34_REGISTER_ADDR, nr34 & 0b0111_1111);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum VoiceType {
    PulseA,
    PulseB,
    Wave,
    WhiteNoise,
}

impl VoiceType {
    fn global_mask(&self) -> u8 {
        // bit position in the global registers
        let global_position: u8 = match *self {
            VoiceType::PulseA => 0,
            VoiceType::PulseB => 1,
            VoiceType::Wave => 2,
            VoiceType::WhiteNoise => 3,
        };
        1 << global_position
    }
}

enum ChannelNum {
    ChannelA,
    ChannelB,
}

struct GlobalReg;

impl GlobalReg {
    fn should_output(voice_type: VoiceType, channel_num: ChannelNum, memory: &Memory) -> bool {
        let channel_nibble = match channel_num {
            ChannelNum::ChannelA => 0,
            ChannelNum::ChannelB => 1,
        };
        let nr51 = memory.read_byte(NR51_REGISTER_ADDR);
        (nr51 >> (4 * channel_nibble) & voice_type.global_mask()) == voice_type.global_mask()
    }
    fn output_level(channel_num: ChannelNum, memory: &Memory) -> u8 {
        let nr50 = memory.read_byte(NR50_REGISTER_ADDR);
        match channel_num {
            ChannelNum::ChannelA => nr50 & 0b111,
            ChannelNum::ChannelB => (nr50 >> 4) & 0b111,
        }
    }
    fn reset_voice_flag(voice_type: VoiceType, memory: &mut Memory) {
        let nr52 = memory.read_byte(NR52_REGISTER_ADDR);
        memory.write_byte(NR52_REGISTER_ADDR, nr52 & !(voice_type.global_mask()));
    }
    fn set_voice_flag(voice_type: VoiceType, memory: &mut Memory) {
        let nr52 = memory.read_byte(NR52_REGISTER_ADDR);
        memory.write_byte(NR52_REGISTER_ADDR, nr52 | voice_type.global_mask());
    }
}

pub struct SoundController {
    sound_is_on: bool,
    channel_1_volume: u8,
    channel_2_volume: u8,
    pulse_a: PulseVoice,
    pulse_b: PulseVoice,
    wave: WaveVoice,
    pulse_a_enabled: bool,
    pulse_b_enabled: bool,
    wave_enabled: bool,
}

impl SoundController {
    pub fn new(audio_subsystem: &AudioSubsystem, memory: &Memory) -> Self {
        SoundController {
            sound_is_on: false,
            channel_1_volume: 0,
            channel_2_volume: 0,
            pulse_a: PulseVoice::new(VoiceType::PulseA, audio_subsystem, memory),
            pulse_b: PulseVoice::new(VoiceType::PulseB, audio_subsystem, memory),
            wave: WaveVoice::new(audio_subsystem, memory),
            pulse_a_enabled: true,
            pulse_b_enabled: true,
            wave_enabled: true,
        }
    }
    pub fn reset(&mut self, memory: &mut Memory) {
        self.pulse_a.stop(memory);
        self.pulse_b.stop(memory);
        self.wave.stop(memory);
    }

    pub fn pulse_a_toggle(&mut self) {
        self.pulse_a_enabled = !self.pulse_a_enabled;
        println!("pulse a: {}", self.pulse_a_enabled);
    }
    pub fn pulse_b_toggle(&mut self) {
        self.pulse_b_enabled = !self.pulse_b_enabled;
        println!("pulse b: {}", self.pulse_b_enabled);
    }
    pub fn wave_toggle(&mut self) {
        self.wave_enabled = !self.wave_enabled;
        println!("wave: {}", self.wave_enabled);
    }

    pub fn run(&mut self, cycles: u32, memory: &mut Memory) {
        let sound_onoff = memory.read_byte(NR52_REGISTER_ADDR);

        self.sound_is_on = (sound_onoff >> 7) == 0b1;
        if self.sound_is_on {
            let channel_ctrl = memory.read_byte(NR50_REGISTER_ADDR);
            self.channel_1_volume = channel_ctrl & 0b111;
            self.channel_2_volume = (channel_ctrl >> 4) & 0b111;

            if self.pulse_a_enabled {
                self.pulse_a.run(cycles, memory);
            } else {
                self.pulse_a.stop(memory);
            }

            if self.pulse_b_enabled {
                self.pulse_b.run(cycles, memory);
            } else {
                self.pulse_b.stop(memory);
            }

            if self.wave_enabled {
                self.wave.run(cycles, memory);
            } else {
                self.wave.stop(memory);
            }
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
        for sample_value in out.iter_mut() {
            *sample_value = if self.phase <= self.duty {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
