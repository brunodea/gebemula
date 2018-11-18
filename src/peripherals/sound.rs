use super::super::cpu::ioregister::CPU_FREQUENCY_HZ;
use blip_buf::BlipBuf;
use sdl2::audio::AudioSpecDesired;
use std::fmt;
use std::fmt::Debug;

const IO_START: u16 = 0xFF10;
const IO_END: u16 = 0xFF3F;

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

// White Noise registers
pub const NR41_REGISTER_ADDR: u16 = 0xFF20;
pub const NR42_REGISTER_ADDR: u16 = 0xFF21;
pub const NR43_REGISTER_ADDR: u16 = 0xFF22;
pub const NR44_REGISTER_ADDR: u16 = 0xFF23;

// Global sound registers
pub const NR50_REGISTER_ADDR: u16 = 0xFF24;
pub const NR51_REGISTER_ADDR: u16 = 0xFF25;
pub const NR52_REGISTER_ADDR: u16 = 0xFF26;

const CUSTOM_WAVE_START_ADDR: u16 = 0xFF30;
const CUSTOM_WAVE_END_ADDR: u16 = 0xFF3F;

/// Sample rate at which sound samples will be generated (before being downsampled to the output device's samplerate)
pub const OUTPUT_FREQUENCY: u32 = CPU_FREQUENCY_HZ;
pub const OUTPUT_CHANNELS: usize = 2;

pub const AUDIO_DESIRED_SPEC: AudioSpecDesired = AudioSpecDesired {
    freq: Some(48000),
    channels: Some(2),
    samples: None, // default sample size
};

fn make_blip_buf(sample_rate: u32) -> BlipBuf {
    let mut buf = BlipBuf::new(sample_rate * 2);
    buf.set_rates(CPU_FREQUENCY_HZ as f64, sample_rate as f64);
    buf
}

const NUM_CHANNELS: usize = 4;

#[derive(Copy, Clone)]
struct SquareVoiceSettings {
    regs: [u8; 5],
}

impl SquareVoiceSettings {
    // Sweep (only present in ch1)
    fn sweep_period(&self) -> u8 {
        (self.regs[0] & 0b0111_0000) >> 4
    }
    fn sweep_negate(&self) -> bool {
        (self.regs[0] & 0b0000_1000) >> 3 != 0
    }
    fn sweep_shift(&self) -> u8 {
        (self.regs[0] & 0b0000_0111) >> 0
    }

    fn duty_cycle(&self) -> u8 {
        (self.regs[1] & 0b1100_0000) >> 6
    }
    fn note_length(&self) -> u8 {
        (self.regs[1] & 0b0011_1111) >> 0
    }

    // Envelope
    fn initial_volume(&self) -> u8 {
        (self.regs[2] & 0b1111_0000) >> 4
    }
    fn env_direction(&self) -> u8 {
        (self.regs[2] & 0b0000_1000) >> 3
    }
    fn env_period(&self) -> u8 {
        (self.regs[2] & 0b0000_0111) >> 0
    }

    fn frequency_lsb(&self) -> u8 {
        self.regs[3]
    }

    fn trigger(&self) -> bool {
        (self.regs[4] & 0b1000_0000) >> 7 != 0
    }
    fn length_enable(&self) -> bool {
        (self.regs[4] & 0b0100_0000) >> 6 != 0
    }
    fn frequency_msb(&self) -> u8 {
        (self.regs[4] & 0b0000_0111) >> 0
    }

    fn frequency(&self) -> u16 {
        let lsb = self.frequency_lsb() as u16;
        let msb = self.frequency_msb() as u16;
        msb << 8 | lsb
    }
}

impl Debug for SquareVoiceSettings {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Regs[raw=[{:02X} {:02X} {:02X} {:02X} {:02X}],\n\
sweep_period={}, sweep_neg={}, sweep_shift={}, duty={}, len={}, vol={}, env_dir={}, env_period={}, freq={}, trigger={}, len_en={}]",
               self.regs[0], self.regs[1], self.regs[2], self.regs[3], self.regs[4],
                self.sweep_period(), self.sweep_negate(), self.sweep_shift(),
                self.duty_cycle(), self.note_length(),
                self.initial_volume(), self.env_direction(), self.env_period(),
                self.frequency(), self.trigger(), self.length_enable())
    }
}

#[derive(Copy, Clone)]
struct WaveVoiceSettings {
    regs: [u8; 5],
}

impl WaveVoiceSettings {
    fn output_on(&self) -> bool {
        (self.regs[0] & 0b1000_0000) >> 7 != 0
    }

    fn sound_length(&self) -> u8 {
        (self.regs[1] & 0b1111_1111) >> 0
    }

    // volume is either 0%, 100%, 50% or 25%
    fn volume(&self) -> u8 {
        (self.regs[2] & 0b0110_0000) >> 5
    }

    fn frequency_lsb(&self) -> u8 {
        self.regs[3]
    }

    fn trigger(&self) -> bool {
        (self.regs[4] & 0b1000_0000) >> 7 != 0
    }
    fn length_enable(&self) -> bool {
        (self.regs[4] & 0b0100_0000) >> 6 != 0
    }
    fn frequency_msb(&self) -> u8 {
        (self.regs[4] & 0b0000_0111) >> 0
    }

    fn frequency(&self) -> u16 {
        let lsb = self.frequency_lsb() as u16;
        let msb = self.frequency_msb() as u16;
        msb << 8 | lsb
    }
}

struct Sequencer {
    // These all step when the value == 0
    length_step: u16, // mod 2
    volume_step: u16, // mod 8
    sweep_step: u16,  // mod 4
}

impl Sequencer {
    fn new() -> Sequencer {
        Sequencer {
            length_step: 0,
            volume_step: 1,
            sweep_step: 2,
        }
    }

    // Clocked at 512 Hz
    fn step(&mut self) {
        self.length_step = (self.length_step + 1) % 2;
        self.volume_step = (self.volume_step + 1) % 8;
        self.sweep_step = (self.sweep_step + 1) % 4;
    }
}

const SQUARE_WAVEFORMS: [u8; 4] = [0b00000001, 0b10000001, 0b10000111, 0b01111110];

struct SquareVoice {
    channel_num: u8,

    has_sweep: bool,
    sweep_active: bool,
    sweep_counter: u8,
    sweep_frequency: u16,

    length_counter: u16,
    frequency: u16, // shadow register
    frequency_counter: u16, // Decremented every 32 clocks
    envelope_counter: u8,
    waveform_index: u8,
    volume: u8,
}

fn adjust_volume_envelope(volume: &mut u8, direction: u8) {
    match direction {
        0 => if *volume > 0 {
            *volume -= 1;
        },
        1 => if *volume < 15 {
            *volume += 1;
        },
        _ => unreachable!(),
    }
}

// Returns (new_frequency, overflow)
fn compute_sweep(old_frequency: u16, regs: SquareVoiceSettings) -> (u16, bool) {
    let abs_delta = old_frequency >> regs.sweep_shift();
    let new_frequency = if regs.sweep_negate() {
        old_frequency - abs_delta
    } else {
        old_frequency + abs_delta
    };

    (new_frequency, new_frequency >= 2048)
}

impl SquareVoice {
    fn new(channel_num: u8) -> SquareVoice {
        SquareVoice {
            channel_num,

            has_sweep: channel_num == 1,
            sweep_active: false,
            sweep_counter: 0,
            sweep_frequency: 0,

            length_counter: 0,
            frequency: 0,
            frequency_counter: 0,
            envelope_counter: 0,
            waveform_index: 0,
            volume: 0,
        }
    }

    fn step_sweep(&mut self, regs: SquareVoiceSettings) {
        if !self.has_sweep {
            unreachable!();
        }

        if self.sweep_active && regs.sweep_period() != 0 {
            if self.sweep_counter == 0 {
                let (new_frequency, overflow) = compute_sweep(self.sweep_frequency, regs);
                let (_, overflow2) = compute_sweep(new_frequency, regs);

                if !overflow {
                    self.sweep_frequency = new_frequency;
                    self.frequency = new_frequency;
                }
                if overflow || overflow2 {
                    // TODO disable channel
                }

                self.sweep_counter = regs.sweep_period();
            } else {
                self.sweep_counter -= 1;
            }
        }
    }

    fn step_envelope(&mut self, regs: SquareVoiceSettings) {
        if regs.env_period() != 0 {
            if self.envelope_counter == 0 {
                adjust_volume_envelope(&mut self.volume, regs.env_direction());
                self.envelope_counter = regs.env_period();
            } else {
                self.envelope_counter -= 1;
            }
        }
    }

    /// Returns true if the sound should stop based on its length.
    fn step_length(&mut self, regs: SquareVoiceSettings) -> bool {
        if regs.length_enable() {
            if self.length_counter > 0 {
                self.length_counter -= 1;
            } else {
                return true;
            }
        }
        false
    }

    fn get_frequency_period(&self) -> u16 {
        (32 / 8) * (2048 - self.frequency)
    }

    fn trigger(&mut self, regs: SquareVoiceSettings) {
        self.volume = regs.initial_volume();
        self.frequency_counter = self.get_frequency_period();
        self.envelope_counter = regs.env_period();

        /*println!(
            "trigger ch{}: regs={:?}\nvolume={} freq={} len={}",
            self.channel_num, regs, self.volume, self.frequency_counter, self.length_counter
        );*/
        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        if self.has_sweep {
            self.sweep_frequency = self.frequency;
            self.sweep_counter = regs.sweep_period();
            self.sweep_active = regs.sweep_shift() != 0 || regs.sweep_period() != 0;
            if regs.sweep_shift() != 0 {
                let (new_frequency, overflow) = compute_sweep(self.sweep_frequency, regs);
                if overflow {
                    // TODO: Disable channel
                }
            }
        }
    }

    fn step(&mut self, regs: SquareVoiceSettings) {
        if self.frequency_counter > 0 {
            self.frequency_counter -= 1;
        } else {
            self.frequency_counter = self.get_frequency_period();
            self.waveform_index = (self.waveform_index + 1) % 8;
        }
    }

    // 4-bit output
    fn sample(&self, regs: SquareVoiceSettings) -> u8 {
        let waveform = SQUARE_WAVEFORMS[regs.duty_cycle() as usize];
        if waveform >> self.waveform_index & 1 != 0 {
            self.volume
        } else {
            0
        }
    }
}

const CUSTOM_WAVE_SIZE: usize = (CUSTOM_WAVE_END_ADDR-CUSTOM_WAVE_START_ADDR + 1) as usize;

struct WaveVoice {
    custom_wave: [u8; CUSTOM_WAVE_SIZE],
    pos_counter: u16,
    length_counter: u16,
    frequency: u16,
    frequency_counter: u16,
}

impl WaveVoice {
    fn new() -> Self {
        WaveVoice {
            custom_wave: [0; CUSTOM_WAVE_SIZE],
            pos_counter: 0,
            length_counter: 0,
            frequency: 0,
            frequency_counter: 0,
        }
    }

    /// Returns true if the sound should stop based on its length.
    fn step_length(&mut self, regs: WaveVoiceSettings) -> bool {
        if regs.length_enable() {
            if self.length_counter > 0 {
                self.length_counter -= 1;
            } else {
                return true;
            }
        }
        false
    }

    fn get_frequency_period(&self) -> u16 {
        (2048 - self.frequency) * 2
    }

    fn trigger(&mut self, regs: WaveVoiceSettings) {
        self.pos_counter = 0;
        self.frequency_counter = self.get_frequency_period();
        
        if self.length_counter == 0 {
            self.length_counter = 256;
        }
    }

    fn step(&mut self, regs: WaveVoiceSettings) {
        if self.frequency_counter > 0 {
            self.frequency_counter -= 1;
        } else {
            self.frequency_counter = self.get_frequency_period();
            self.pos_counter = (self.pos_counter + 1) % (self.custom_wave.len() as u16 * 2); //*2 because each nibble is a sample
        }
    }

    //4 bit output
    fn sample(&self, regs: WaveVoiceSettings) -> u8 {
        if regs.volume() == 0 {
            0
        } else {
            let pos = (self.pos_counter / 2) as usize;
            let nibble = self.pos_counter % 2;
            let sample = if nibble == 0 {
                self.custom_wave[pos] >> 4
            } else {
                self.custom_wave[pos] & 0x0F
            };
            sample >> (regs.volume() - 1)
        }
    }
}

struct NoiseVoice {}

pub struct AudioController {
    /// All register values as written.
    /// 0x00-0x04: NR1x (Square 1)
    /// 0x05-0x09: NR2x (Square 2)
    /// 0x0A-0x0E: NR3x (Wave)
    /// 0x0F-0x13: NR4x (Noise)
    /// 0x14-0x16: NR5x (Control/Status)
    /// 0x20-0x2F: Wave table (packed, 2 4-bit samples per byte)
    regs: [u8; 0x30],

    buf_l: BlipBuf,
    buf_r: BlipBuf,
    previous_l: i32,
    previous_r: i32,

    /// Cycles since last output to buf_l/buf_r
    cur_cycle: u32,

    sequencer: Sequencer,
    sequencer_counter: u16,

    apu_enabled: bool,                      // global APU power-on status
    enabled_channels: [bool; NUM_CHANNELS], // 1 bit per channel
    ch1: SquareVoice,
    ch2: SquareVoice,
    ch3: WaveVoice,
    ch4: NoiseVoice,

    debug_enabled_channels: [bool; NUM_CHANNELS],
}

impl AudioController {
    pub fn new() -> AudioController {
        AudioController {
            regs: [0; 0x30],
            buf_l: make_blip_buf(0),
            buf_r: make_blip_buf(0),
            previous_l: 0,
            previous_r: 0,

            cur_cycle: 0,

            sequencer: Sequencer::new(),
            sequencer_counter: 0,

            apu_enabled: false,
            enabled_channels: [false; NUM_CHANNELS],
            ch1: SquareVoice::new(1),
            ch2: SquareVoice::new(2),
            ch3: WaveVoice::new(),
            ch4: NoiseVoice {},

            debug_enabled_channels: [true; NUM_CHANNELS],
        }
    }

    pub fn set_sample_rate(&mut self, output_sample_rate: u32) {
        self.buf_l = make_blip_buf(output_sample_rate);
        self.buf_r = make_blip_buf(output_sample_rate);
    }

    pub fn write_reg(&mut self, addr: u16, val: u8) {
        match addr {
            NR52_REGISTER_ADDR => {
                let enable = val & 0b1000_0000 != 0;
                println!("APU power={}", enable);
                if self.apu_enabled && !enable {
                    self.power_down();
                }
                self.apu_enabled = enable;
                return;
            }
            // Check if register write is allowed by current power-on status
            _ if self.apu_enabled => {}
            CUSTOM_WAVE_START_ADDR...CUSTOM_WAVE_END_ADDR => {
                self.ch3.custom_wave[(addr - CUSTOM_WAVE_START_ADDR) as usize] = val;
            }
            _ => return,
        }

        self.regs[(addr - IO_START) as usize] = val;
        match addr {
            NR11_REGISTER_ADDR => {
                let regs = self.nr1x();
                self.ch1.length_counter = 64 - regs.note_length() as u16;
            }
            NR13_REGISTER_ADDR => {
                let regs = self.nr1x();
                self.ch1.frequency &= !0xFF;
                self.ch1.frequency |= regs.frequency_lsb() as u16;
            }
            NR14_REGISTER_ADDR => {
                let regs = self.nr1x();
                self.ch1.frequency &= !0x700;
                self.ch1.frequency |= (regs.frequency_msb() as u16) << 8;

                if regs.trigger() {
                    self.enabled_channels[0] = true;
                    self.ch1.trigger(regs);
                }
            },
            NR21_REGISTER_ADDR => {
                let regs = self.nr2x();
                self.ch2.length_counter = 64 - regs.note_length() as u16;
            }
            NR23_REGISTER_ADDR => {
                let regs = self.nr2x();
                self.ch2.frequency &= !0xFF;
                self.ch2.frequency |= regs.frequency_lsb() as u16;
            }
            NR24_REGISTER_ADDR => {
                let regs = self.nr2x();
                self.ch2.frequency &= !0x700;
                self.ch2.frequency |= (regs.frequency_msb() as u16) << 8;

                if regs.trigger() {
                    self.enabled_channels[1] = true;
                    self.ch2.trigger(regs);
                }
            }
            NR31_REGISTER_ADDR => {
                let regs = self.nr3x();
                self.ch3.length_counter = 256 - regs.sound_length() as u16;
            }
            NR33_REGISTER_ADDR => {
                let regs = self.nr3x();
                self.ch3.frequency &= !0xFF;
                self.ch3.frequency |= regs.frequency_lsb() as u16;
            }
            NR34_REGISTER_ADDR => {
                let regs = self.nr3x();
                self.ch3.frequency &= !0x700;
                self.ch3.frequency |= (regs.frequency_msb() as u16) << 8;
                if regs.trigger() {
                    self.enabled_channels[2] = true;
                    self.ch3.trigger(regs);
                }
            }
            //NR44_REGISTER_ADDR => self.trigger_ch4(),
            _ => {}
        }
    }

    fn power_down(&mut self) {
        self.apu_enabled = false;
        for x in self.enabled_channels.iter_mut() {
            *x = false;
        }
        // Clear all configuration registers except for wave table
        for x in &mut self.regs[0x00..=0x16] {
            *x = 0
        }
    }

    pub fn read_reg(&self, addr: u16) -> u8 {
        const UNUSED_BITS: [u8; 0x30] = [
            // NR1x
            0x80,
            0x3F,
            0x00,
            0xFF,
            0xBF,
            // NR2x
            0xFF,
            0x3F,
            0x00,
            0xFF,
            0xBF,
            // NR3x
            0x7F,
            0xFF,
            0x9F,
            0xFF,
            0xBF,
            // NR4x
            0xFF,
            0xFF,
            0x00,
            0x00,
            0xBF,
            // NR5x
            0x00,
            0x00,
            0x70,
            // Unused
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            // Wave table
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        match addr {
            NR52_REGISTER_ADDR => {
                let mut value = UNUSED_BITS[(NR52_REGISTER_ADDR - IO_START) as usize];
                if self.apu_enabled {
                    value |= 0b1000_0000;
                }
                for (i, &enabled) in self.enabled_channels.iter().enumerate() {
                    if enabled {
                        value |= 1 << i;
                    }
                }
                value
            }
            _ => {
                let index = (addr - IO_START) as usize;
                self.regs[index] | UNUSED_BITS[index]
            }
        }
    }

    fn nr1x(&self) -> SquareVoiceSettings {
        SquareVoiceSettings {
            regs: *array_ref![self.regs, 0, 5],
        }
    }

    fn nr2x(&self) -> SquareVoiceSettings {
        SquareVoiceSettings {
            regs: *array_ref![self.regs, 5, 5],
        }
    }

    fn nr3x(&self) -> WaveVoiceSettings {
        WaveVoiceSettings {
            regs: *array_ref![self.regs, 10, 5],
        }
    }

    fn nr4x(&self) -> &[u8; 5] {
        array_ref![self.regs, 15, 5]
    }

    fn nr5x(&self) -> &[u8; 3] {
        array_ref![self.regs, 20, 3]
    }

    fn wave_table(&self) -> &[u8] {
        &self.regs[0x20..0x2F]
    }

    pub fn run_for(&mut self, num_cycles: u32) {
        for _ in 0..num_cycles {
            let (l, r) = self.step();

            self.buf_l.add_delta(self.cur_cycle, l - self.previous_l);
            self.previous_l = l;
            self.buf_r.add_delta(self.cur_cycle, r - self.previous_r);
            self.previous_r = r;

            self.cur_cycle += 1;
        }
    }

    pub fn generate_audio(&mut self, output: &mut Vec<i16>) {
        // Not enough queued audio to generate yet
        if self.cur_cycle < CPU_FREQUENCY_HZ / 200 {
            return;
        }

        self.buf_l.end_frame(self.cur_cycle);
        self.buf_r.end_frame(self.cur_cycle);
        self.cur_cycle = 0;

        let samples_available = self.buf_l.samples_avail() as usize;
        assert_eq!(samples_available, self.buf_r.samples_avail() as usize);
        if samples_available <= 0 {
            return;
        }

        let previous_len = output.len();
        // The blip_buf crate incorrectly computes the output array size when using stereo, add an extra item to prevent that
        output.resize(previous_len + samples_available * 2 + 1, 0);
        {
            let new_output = &mut output[previous_len..];
            let samples_read_l = self.buf_l
                .read_samples(&mut new_output[0..samples_available * 2], true);
            let samples_read_r = self.buf_r
                .read_samples(&mut new_output[1..samples_available * 2 + 1], true);
            assert_eq!(samples_read_l, samples_read_r);
            assert_eq!(samples_read_l, samples_available);
        }
        // Remove the item used for the workaround above
        output.pop();
    }

    pub fn debug_toggle_channel(&mut self, ch: usize) -> bool {
        //assert 0 <= ch <= 3
        self.debug_enabled_channels[ch] = !self.debug_enabled_channels[ch];
        self.debug_enabled_channels[ch]
    }

    fn step(&mut self) -> (i32, i32) {
        if !self.apu_enabled {
            return (0, 0);
        }

        let nr1x = self.nr1x();
        let nr2x = self.nr2x();
        let nr3x = self.nr3x();

        self.sequencer_counter = (self.sequencer_counter + 1) % 8192;
        if self.sequencer_counter == 0 {
            if self.sequencer.length_step == 0 {
                if self.ch1.step_length(nr1x) {
                    self.enabled_channels[0] = false;
                }
                if self.ch2.step_length(nr2x) {
                    self.enabled_channels[1] = false;
                }
                if self.ch3.step_length(nr3x) {
                    self.enabled_channels[2] = false;
                }
            }

            if self.sequencer.volume_step == 0 {
                self.ch1.step_envelope(nr1x);
                self.ch2.step_envelope(nr2x);
                // TODO self.ch3.step_envelope();
                // TODO self.ch4.step_envelope();
            }

            if self.sequencer.sweep_step == 0 {
                self.ch1.step_sweep(nr1x);
            }

            // TODO: at 512Hz: self.sequencer.step();
            self.sequencer.step();
        }

        if self.enabled_channels[0] {
            self.ch1.step(nr1x);
        }
        if self.enabled_channels[1] {
            self.ch2.step(nr2x);
        }
        if self.enabled_channels[2] {
            self.ch3.step(nr3x);
        }
        // TODO ch4

        let mut mixed = 0;
        let ch1_val = self.ch1.sample(nr1x) as i32;
        let ch2_val = self.ch2.sample(nr2x) as i32;
        let ch3_val = self.ch3.sample(nr3x) as i32;

        if self.debug_enabled_channels[0] && self.enabled_channels[0] {
            mixed += (ch1_val - 7) * 0x200;
        }
        if self.debug_enabled_channels[1] && self.enabled_channels[1] {
            mixed += (ch2_val - 7) * 0x200;
        }
        if self.debug_enabled_channels[2] && self.enabled_channels[2] {
            if ch3_val != 0 {
                println!("ch3: {}", ch3_val);
            }
            mixed += (ch3_val - 7) * 0x200;
        }

        (mixed, mixed)
    }
}
