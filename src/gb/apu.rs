use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const SAMPLE_RATE: u32 = 44100;
const CPU_CLOCK: u32 = 4194304;
const BUFFER_SIZE: usize = 16384;

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

/// Pulse channel (CH1 has sweep, CH2 doesn't).
struct PulseChannel {
    enabled: bool,
    duty: u8,
    length_timer: u8,
    length_enabled: bool,
    volume: u8,
    volume_initial: u8,
    volume_sweep_pace: u8,
    volume_sweep_dir: bool, // true = increase
    volume_sweep_timer: u8,
    frequency: u16,
    freq_timer: i32,
    duty_pos: u8,
    // Sweep (CH1 only)
    sweep_pace: u8,
    sweep_dir: bool, // true = subtract
    sweep_step: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow: u16,
}

impl PulseChannel {
    /// Create a new pulse channel in its initial state.
    fn new() -> Self {
        PulseChannel {
            enabled: false, duty: 0, length_timer: 0, length_enabled: false,
            volume: 0, volume_initial: 0, volume_sweep_pace: 0,
            volume_sweep_dir: false, volume_sweep_timer: 0,
            frequency: 0, freq_timer: 0, duty_pos: 0,
            sweep_pace: 0, sweep_dir: false, sweep_step: 0,
            sweep_timer: 0, sweep_enabled: false, sweep_shadow: 0,
        }
    }

    /// Advance the frequency timer and update duty position.
    fn tick(&mut self) {
        self.freq_timer -= 1;
        if self.freq_timer <= 0 {
            self.freq_timer = (2048 - self.frequency as i32) * 4;
            self.duty_pos = (self.duty_pos + 1) % 8;
        }
    }

    /// Get the current output sample (0.0-1.0).
    fn sample(&self) -> f32 {
        if !self.enabled || self.volume == 0 { return 0.0; }
        let out = DUTY_TABLE[self.duty as usize][self.duty_pos as usize];
        (out as f32) * (self.volume as f32 / 15.0)
    }

    /// Clock the length timer. Disables channel when it expires.
    fn tick_length(&mut self) {
        if self.length_enabled && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 { self.enabled = false; }
        }
    }

    /// Clock the volume envelope.
    fn tick_volume(&mut self) {
        if self.volume_sweep_pace == 0 { return; }
        self.volume_sweep_timer -= 1;
        if self.volume_sweep_timer == 0 {
            self.volume_sweep_timer = self.volume_sweep_pace;
            if self.volume_sweep_dir && self.volume < 15 {
                self.volume += 1;
            } else if !self.volume_sweep_dir && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    /// Clock the frequency sweep (CH1 only).
    fn tick_sweep(&mut self) {
        if self.sweep_pace == 0 || !self.sweep_enabled { return; }
        self.sweep_timer -= 1;
        if self.sweep_timer == 0 {
            self.sweep_timer = self.sweep_pace;
            let new_freq = self.calc_sweep();
            if new_freq <= 2047 && self.sweep_step > 0 {
                self.frequency = new_freq;
                self.sweep_shadow = new_freq;
                // Overflow check again
                if self.calc_sweep() > 2047 { self.enabled = false; }
            }
        }
    }

    /// Calculate the new frequency after a sweep step.
    fn calc_sweep(&self) -> u16 {
        let shift = self.sweep_shadow >> self.sweep_step;
        if self.sweep_dir {
            self.sweep_shadow.wrapping_sub(shift)
        } else {
            self.sweep_shadow.wrapping_add(shift)
        }
    }

    /// Trigger the channel (restart sound).
    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_timer == 0 { self.length_timer = 64; }
        self.freq_timer = (2048 - self.frequency as i32) * 4;
        self.volume = self.volume_initial;
        self.volume_sweep_timer = if self.volume_sweep_pace == 0 { 8 } else { self.volume_sweep_pace };
        // Sweep
        self.sweep_shadow = self.frequency;
        self.sweep_timer = if self.sweep_pace == 0 { 8 } else { self.sweep_pace };
        self.sweep_enabled = self.sweep_pace > 0 || self.sweep_step > 0;
        if self.sweep_step > 0 && self.calc_sweep() > 2047 {
            self.enabled = false;
        }
    }
}

/// Wave channel (CH3).
/// Wave channel (CH3). Plays custom 4-bit waveform from wave RAM.
struct WaveChannel {
    enabled: bool,
    dac_enabled: bool,
    length_timer: u16,
    length_enabled: bool,
    volume_shift: u8,
    frequency: u16,
    freq_timer: i32,
    sample_pos: u8,
    wave_ram: [u8; 16],
}

impl WaveChannel {
    /// Create a new wave channel in its initial state.
    fn new() -> Self {
        WaveChannel {
            enabled: false, dac_enabled: false, length_timer: 0,
            length_enabled: false, volume_shift: 0, frequency: 0,
            freq_timer: 0, sample_pos: 0, wave_ram: [0; 16],
        }
    }

    /// Advance the channel by one T-cycle.
    fn tick(&mut self) {
        self.freq_timer -= 1;
        if self.freq_timer <= 0 {
            self.freq_timer = (2048 - self.frequency as i32) * 2;
            self.sample_pos = (self.sample_pos + 1) % 32;
        }
    }

    /// Get the current output sample value.
    fn sample(&self) -> f32 {
        if !self.enabled || !self.dac_enabled { return 0.0; }
        let byte = self.wave_ram[(self.sample_pos / 2) as usize];
        let nibble = if self.sample_pos % 2 == 0 { byte >> 4 } else { byte & 0x0F };
        let shifted = match self.volume_shift {
            0 => 0,
            1 => nibble,
            2 => nibble >> 1,
            3 => nibble >> 2,
            _ => 0,
        };
        shifted as f32 / 15.0
    }

    /// Clock the length timer. Disables channel when expired.
    fn tick_length(&mut self) {
        if self.length_enabled && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 { self.enabled = false; }
        }
    }

    /// Trigger the channel (restart sound).
    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_timer == 0 { self.length_timer = 256; }
        self.freq_timer = (2048 - self.frequency as i32) * 2;
        self.sample_pos = 0;
    }
}

/// Noise channel (CH4).
/// Noise channel (CH4). Uses LFSR for pseudo-random output.
struct NoiseChannel {
    enabled: bool,
    length_timer: u8,
    length_enabled: bool,
    volume: u8,
    volume_initial: u8,
    volume_sweep_pace: u8,
    volume_sweep_dir: bool,
    volume_sweep_timer: u8,
    clock_shift: u8,
    width_mode: bool, // true = 7-bit, false = 15-bit
    divisor_code: u8,
    freq_timer: i32,
    lfsr: u16,
}

impl NoiseChannel {
    /// Create a new noise channel in its initial state.
    fn new() -> Self {
        NoiseChannel {
            enabled: false, length_timer: 0, length_enabled: false,
            volume: 0, volume_initial: 0, volume_sweep_pace: 0,
            volume_sweep_dir: false, volume_sweep_timer: 0,
            clock_shift: 0, width_mode: false, divisor_code: 0,
            freq_timer: 0, lfsr: 0x7FFF,
        }
    }

    /// Advance the channel by one T-cycle.
    fn tick(&mut self) {
        self.freq_timer -= 1;
        if self.freq_timer <= 0 {
            let divisor = if self.divisor_code == 0 { 8 } else { (self.divisor_code as i32) * 16 };
            self.freq_timer = divisor << self.clock_shift;
            let xor = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
            self.lfsr = (self.lfsr >> 1) | (xor << 14);
            if self.width_mode {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor << 6;
            }
        }
    }

    /// Get the current output sample value.
    fn sample(&self) -> f32 {
        if !self.enabled || self.volume == 0 { return 0.0; }
        let out = (!self.lfsr & 1) as f32;
        out * (self.volume as f32 / 15.0)
    }

    /// Clock the length timer. Disables channel when expired.
    fn tick_length(&mut self) {
        if self.length_enabled && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 { self.enabled = false; }
        }
    }

    /// Clock the volume envelope.
    fn tick_volume(&mut self) {
        if self.volume_sweep_pace == 0 { return; }
        self.volume_sweep_timer -= 1;
        if self.volume_sweep_timer == 0 {
            self.volume_sweep_timer = self.volume_sweep_pace;
            if self.volume_sweep_dir && self.volume < 15 {
                self.volume += 1;
            } else if !self.volume_sweep_dir && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    /// Trigger the channel (restart sound).
    fn trigger(&mut self) {
        self.enabled = true;
        if self.length_timer == 0 { self.length_timer = 64; }
        self.volume = self.volume_initial;
        self.volume_sweep_timer = if self.volume_sweep_pace == 0 { 8 } else { self.volume_sweep_pace };
        self.lfsr = 0x7FFF;
        let divisor = if self.divisor_code == 0 { 8 } else { (self.divisor_code as i32) * 16 };
        self.freq_timer = divisor << self.clock_shift;
    }
}

/// Audio Processing Unit.
pub struct Apu {
    ch1: PulseChannel,
    ch2: PulseChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    master_enabled: bool,
    left_volume: u8,
    right_volume: u8,
    panning: u8, // NR51
    frame_seq_timer: u32,
    frame_seq_step: u8,
    sample_timer: f64,
    sample_rate: u32,
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
    _stream: Option<cpal::Stream>,
}

impl Apu {
    /// Create a new APU and initialize the audio output stream.
    pub fn new() -> Self {
        let buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(BUFFER_SIZE)));
        let (stream, sample_rate) = Self::init_audio(buffer.clone());

        Apu {
            ch1: PulseChannel::new(),
            ch2: PulseChannel::new(),
            ch3: WaveChannel::new(),
            ch4: NoiseChannel::new(),
            master_enabled: false,
            left_volume: 7,
            right_volume: 7,
            panning: 0xFF,
            frame_seq_timer: 0,
            frame_seq_step: 0,
            sample_timer: 0.0,
            sample_rate,
            sample_buffer: buffer,
            _stream: stream,
        }
    }

    /// Initialize the audio output stream using the system default device.
    fn init_audio(buffer: Arc<Mutex<VecDeque<f32>>>) -> (Option<cpal::Stream>, u32) {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => return (None, SAMPLE_RATE),
        };
        let supported_config = match device.default_output_config() {
            Ok(c) => c,
            Err(_) => return (None, SAMPLE_RATE),
        };
        let rate = supported_config.sample_rate().0;
        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: supported_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut buf = buffer.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = buf.pop_front().unwrap_or(0.0);
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        ).ok();

        if let Some(ref s) = stream { s.play().ok(); }
        (stream, rate)
    }

    /// Returns true if audio output is available.
    pub fn has_audio(&self) -> bool {
        self._stream.is_some()
    }

    /// Advance the APU by the given number of T-cycles.
    pub fn tick(&mut self, cycles: u8) {
        if !self.master_enabled || self._stream.is_none() { return; }

        for _ in 0..cycles {
            self.ch1.tick();
            self.ch2.tick();
            self.ch3.tick();
            self.ch4.tick();

            // Frame sequencer (512 Hz = every 8192 T-cycles)
            self.frame_seq_timer += 1;
            if self.frame_seq_timer >= 8192 {
                self.frame_seq_timer = 0;
                self.clock_frame_sequencer();
            }

            // Sample generation
            self.sample_timer += self.sample_rate as f64 / CPU_CLOCK as f64;
            if self.sample_timer >= 1.0 {
                self.sample_timer -= 1.0;
                self.generate_sample();
            }
        }
    }

    /// Clock the frame sequencer (length, volume envelope, sweep).
    fn clock_frame_sequencer(&mut self) {
        match self.frame_seq_step {
            0 => { self.ch1.tick_length(); self.ch2.tick_length(); self.ch3.tick_length(); self.ch4.tick_length(); }
            2 => { self.ch1.tick_length(); self.ch2.tick_length(); self.ch3.tick_length(); self.ch4.tick_length(); self.ch1.tick_sweep(); }
            4 => { self.ch1.tick_length(); self.ch2.tick_length(); self.ch3.tick_length(); self.ch4.tick_length(); }
            6 => { self.ch1.tick_length(); self.ch2.tick_length(); self.ch3.tick_length(); self.ch4.tick_length(); self.ch1.tick_sweep(); }
            7 => { self.ch1.tick_volume(); self.ch2.tick_volume(); self.ch4.tick_volume(); }
            _ => {}
        }
        self.frame_seq_step = (self.frame_seq_step + 1) % 8;
    }

    /// Mix all channels and push a stereo sample to the output buffer.
    fn generate_sample(&mut self) {
        let ch1 = self.ch1.sample();
        let ch2 = self.ch2.sample();
        let ch3 = self.ch3.sample();
        let ch4 = self.ch4.sample();

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        if self.panning & 0x10 != 0 { left += ch1; }
        if self.panning & 0x20 != 0 { left += ch2; }
        if self.panning & 0x40 != 0 { left += ch3; }
        if self.panning & 0x80 != 0 { left += ch4; }
        if self.panning & 0x01 != 0 { right += ch1; }
        if self.panning & 0x02 != 0 { right += ch2; }
        if self.panning & 0x04 != 0 { right += ch3; }
        if self.panning & 0x08 != 0 { right += ch4; }

        left *= (self.left_volume as f32 + 1.0) / 32.0;
        right *= (self.right_volume as f32 + 1.0) / 32.0;

        let mut buf = self.sample_buffer.lock().unwrap();
        if buf.len() < BUFFER_SIZE {
            buf.push_back(left);
            buf.push_back(right);
        } else {
            drop(buf);
            // Buffer full — wait for audio to drain (throttles emulation to real-time)
            std::thread::sleep(std::time::Duration::from_micros(100));
            let mut buf = self.sample_buffer.lock().unwrap();
            buf.push_back(left);
            buf.push_back(right);
        }
    }

    /// Read an APU register.
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => (self.ch1.sweep_pace << 4) | (if self.ch1.sweep_dir { 0x08 } else { 0 }) | self.ch1.sweep_step,
            0xFF11 => self.ch1.duty << 6,
            0xFF12 => (self.ch1.volume_initial << 4) | (if self.ch1.volume_sweep_dir { 0x08 } else { 0 }) | self.ch1.volume_sweep_pace,
            0xFF14 => if self.ch1.length_enabled { 0x40 } else { 0 },
            0xFF16 => self.ch2.duty << 6,
            0xFF17 => (self.ch2.volume_initial << 4) | (if self.ch2.volume_sweep_dir { 0x08 } else { 0 }) | self.ch2.volume_sweep_pace,
            0xFF19 => if self.ch2.length_enabled { 0x40 } else { 0 },
            0xFF1A => if self.ch3.dac_enabled { 0x80 } else { 0 },
            0xFF1C => self.ch3.volume_shift << 5,
            0xFF1E => if self.ch3.length_enabled { 0x40 } else { 0 },
            0xFF21 => (self.ch4.volume_initial << 4) | (if self.ch4.volume_sweep_dir { 0x08 } else { 0 }) | self.ch4.volume_sweep_pace,
            0xFF22 => (self.ch4.clock_shift << 4) | (if self.ch4.width_mode { 0x08 } else { 0 }) | self.ch4.divisor_code,
            0xFF23 => if self.ch4.length_enabled { 0x40 } else { 0 },
            0xFF24 => (self.left_volume << 4) | self.right_volume,
            0xFF25 => self.panning,
            0xFF26 => {
                let mut val = if self.master_enabled { 0x80 } else { 0 };
                if self.ch1.enabled { val |= 0x01; }
                if self.ch2.enabled { val |= 0x02; }
                if self.ch3.enabled { val |= 0x04; }
                if self.ch4.enabled { val |= 0x08; }
                val
            }
            0xFF30..=0xFF3F => self.ch3.wave_ram[(addr - 0xFF30) as usize],
            _ => 0xFF,
        }
    }

    /// Write an APU register.
    pub fn write(&mut self, addr: u16, byte: u8) {
        match addr {
            0xFF10 => {
                self.ch1.sweep_pace = (byte >> 4) & 0x07;
                self.ch1.sweep_dir = byte & 0x08 != 0;
                self.ch1.sweep_step = byte & 0x07;
            }
            0xFF11 => {
                self.ch1.duty = (byte >> 6) & 0x03;
                self.ch1.length_timer = 64 - (byte & 0x3F);
            }
            0xFF12 => {
                self.ch1.volume_initial = byte >> 4;
                self.ch1.volume_sweep_dir = byte & 0x08 != 0;
                self.ch1.volume_sweep_pace = byte & 0x07;
                if byte & 0xF8 == 0 { self.ch1.enabled = false; }
            }
            0xFF13 => { self.ch1.frequency = (self.ch1.frequency & 0x700) | byte as u16; }
            0xFF14 => {
                self.ch1.frequency = (self.ch1.frequency & 0xFF) | (((byte & 0x07) as u16) << 8);
                self.ch1.length_enabled = byte & 0x40 != 0;
                if byte & 0x80 != 0 { self.ch1.trigger(); }
            }
            0xFF16 => {
                self.ch2.duty = (byte >> 6) & 0x03;
                self.ch2.length_timer = 64 - (byte & 0x3F);
            }
            0xFF17 => {
                self.ch2.volume_initial = byte >> 4;
                self.ch2.volume_sweep_dir = byte & 0x08 != 0;
                self.ch2.volume_sweep_pace = byte & 0x07;
                if byte & 0xF8 == 0 { self.ch2.enabled = false; }
            }
            0xFF18 => { self.ch2.frequency = (self.ch2.frequency & 0x700) | byte as u16; }
            0xFF19 => {
                self.ch2.frequency = (self.ch2.frequency & 0xFF) | (((byte & 0x07) as u16) << 8);
                self.ch2.length_enabled = byte & 0x40 != 0;
                if byte & 0x80 != 0 { self.ch2.trigger(); }
            }
            0xFF1A => { self.ch3.dac_enabled = byte & 0x80 != 0; if !self.ch3.dac_enabled { self.ch3.enabled = false; } }
            0xFF1B => { self.ch3.length_timer = 256 - byte as u16; }
            0xFF1C => { self.ch3.volume_shift = (byte >> 5) & 0x03; }
            0xFF1D => { self.ch3.frequency = (self.ch3.frequency & 0x700) | byte as u16; }
            0xFF1E => {
                self.ch3.frequency = (self.ch3.frequency & 0xFF) | (((byte & 0x07) as u16) << 8);
                self.ch3.length_enabled = byte & 0x40 != 0;
                if byte & 0x80 != 0 { self.ch3.trigger(); }
            }
            0xFF20 => { self.ch4.length_timer = 64 - (byte & 0x3F); }
            0xFF21 => {
                self.ch4.volume_initial = byte >> 4;
                self.ch4.volume_sweep_dir = byte & 0x08 != 0;
                self.ch4.volume_sweep_pace = byte & 0x07;
                if byte & 0xF8 == 0 { self.ch4.enabled = false; }
            }
            0xFF22 => {
                self.ch4.clock_shift = byte >> 4;
                self.ch4.width_mode = byte & 0x08 != 0;
                self.ch4.divisor_code = byte & 0x07;
            }
            0xFF23 => {
                self.ch4.length_enabled = byte & 0x40 != 0;
                if byte & 0x80 != 0 { self.ch4.trigger(); }
            }
            0xFF24 => {
                self.left_volume = (byte >> 4) & 0x07;
                self.right_volume = byte & 0x07;
            }
            0xFF25 => { self.panning = byte; }
            0xFF26 => {
                self.master_enabled = byte & 0x80 != 0;
                if !self.master_enabled {
                    self.ch1.enabled = false;
                    self.ch2.enabled = false;
                    self.ch3.enabled = false;
                    self.ch4.enabled = false;
                }
            }
            0xFF30..=0xFF3F => { self.ch3.wave_ram[(addr - 0xFF30) as usize] = byte; }
            _ => {}
        }
    }
}
