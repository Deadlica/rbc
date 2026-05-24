use std::fs;

use crate::gb::{cartridge::Cartridge, joypad::JoypadKey};

pub mod bus;
pub mod cpu;
pub mod registers;
pub mod ppu;
pub mod timer;
pub mod cartridge;
pub mod joypad;
pub mod apu;

/// Top-level Game Boy system. Owns all subsystems (CPU, memory, etc.)
/// and provides the interface for loading ROMs and running emulation.
pub struct Gb {
    cpu: cpu::Cpu,
    bus: bus::Bus,
    frame_done: bool,
}

impl Gb {
    /// Create a new Game Boy instance with all subsystems in their initial state.
    pub fn new() -> Self {
        Gb {
            cpu: cpu::Cpu::new(),
            bus: bus::Bus::new(),
            frame_done: false,
        }
    }

    /// Execute one CPU instruction and advance all subsystems. Returns elapsed T-cycles.
    pub fn step(&mut self) -> u8 {
        let elapsed_cycles = self.cpu.step(&mut self.bus);
        let ppu_cycles = if self.bus.double_speed { elapsed_cycles / 2 } else { elapsed_cycles };
        let scanline_done = self.bus.ppu.tick(ppu_cycles);
        if self.bus.ppu.stat_irq {
            self.bus.request_interrupt(bus::Interrupt::LCD);
            self.bus.ppu.stat_irq = false;
        }
        if scanline_done {
            self.bus.hdma_tick();
        }
        let interrupt = self.bus.timer.tick(elapsed_cycles);
        if interrupt {
            self.bus.request_interrupt(bus::Interrupt::TIMER);
        }
        self.bus.apu.tick(elapsed_cycles);
        if self.bus.ppu.vblank {
            if self.bus.ppu.lcdc & 0x80 != 0 {
                self.bus.request_interrupt(bus::Interrupt::VBLANK);
            }
            self.bus.ppu.vblank = false;
            self.frame_done = true;
        }
        elapsed_cycles
    }

    /// Returns true if a frame was just completed, and clears the flag.
    pub fn frame_ready(&mut self) -> bool {
        let ready = self.frame_done;
        self.frame_done = false;
        ready
    }

    /// Get a reference to the current framebuffer.
    pub fn framebuffer(&self) -> &[u32; ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT] {
        &self.bus.ppu.framebuffer
    }

    /// Press a joypad key.
    pub fn key_down(&mut self, key: JoypadKey) {
        self.bus.joypad.key_down(key);
    }

    /// Reset all joypad keys to unpressed.
    pub fn reset_joypad(&mut self) {
        self.bus.joypad.reset();
    }

    /// Load ROM data into the cartridge.
    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.bus.cartridge = Cartridge::new(data);
        self.bus.ppu.cgb_mode = self.bus.cartridge.cgb_mode;
    }

    /// Load a boot ROM. Resets PC to 0x0000 and enables boot ROM overlay.
    pub fn load_boot_rom(&mut self, data: Vec<u8>) {
        self.bus.boot_rom = Some(data);
        self.bus.boot_rom_enabled = true;
        self.bus.ppu.cgb_mode = true;
        self.cpu.reset_for_boot();
    }

    /// Save cartridge RAM to a file for game persistence.
    pub fn save_game(&self, path: &str) {
        fs::write(path, &self.bus.cartridge.ram).ok();
    }

    /// Load a save file into cartridge RAM if it exists.
    pub fn load_save(&mut self, path: &str) {
        if let Ok(data) = fs::read(path) {
            self.bus.cartridge.ram = data;
        }
    }

    /// Write a byte to a memory address (used by cheats).
    pub fn write_memory(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }

    /// Set master volume (0.0 to 1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.bus.apu.master_volume = volume;
    }

    /// Get current master volume.
    pub fn volume(&self) -> f32 {
        self.bus.apu.master_volume
    }

    /// Set mute state.
    pub fn set_muted(&mut self, muted: bool) {
        self.bus.apu.muted = muted;
    }

    /// Enable/disable audio throttle (disable for fast-forward).
    pub fn set_skip_throttle(&mut self, skip: bool) {
        self.bus.apu.skip_throttle = skip;
    }

    /// Get mute state.
    pub fn muted(&self) -> bool {
        self.bus.apu.muted
    }

    /// Save emulator state to a byte vector.
    pub fn save_state(&self) -> Vec<u8> {
        let mut state = Vec::new();
        // CPU registers
        state.extend_from_slice(&self.cpu.save_state());
        // Bus state (all RAM, PPU, timer, etc.)
        state.extend_from_slice(&self.bus.save_state());
        state
    }

    /// Load emulator state from a byte vector.
    pub fn load_state(&mut self, data: &[u8]) -> bool {
        if data.len() < 12 { return false; }
        let cpu_size = self.cpu.state_size();
        if data.len() < cpu_size { return false; }
        self.cpu.load_state(&data[..cpu_size]);
        self.bus.load_state(&data[cpu_size..]);
        true
    }
}
