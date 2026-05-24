use std::fs;

use crate::gb::{cartridge::Cartridge, joypad::JoypadKey};

pub mod bus;
pub mod cpu;
pub mod registers;
pub mod ppu;
pub mod timer;
pub mod display;
pub mod cartridge;
pub mod joypad;
pub mod apu;

/// Top-level Game Boy system. Owns all subsystems (CPU, memory, etc.)
/// and provides the interface for loading ROMs and running emulation.
pub struct Gb {
    cpu: cpu::Cpu,
    bus: bus::Bus,
    display: display::Display,
}

impl Gb {
    /// Create a new Game Boy instance with all subsystems in their initial state.
    pub fn new() -> Self {
        Gb {
            cpu: cpu::Cpu::new(),
            bus: bus::Bus::new(),
            display: display::Display::new(),
        }
    }

    /// Run the emulation loop indefinitely.
    pub fn run(&mut self) {
        while self.display.is_open() {
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
                self.poll_keys();
                if self.bus.ppu.lcdc & 0x80 != 0 {
                    self.bus.request_interrupt(bus::Interrupt::VBLANK);
                }
                self.display.update(&self.bus.ppu.framebuffer);
                self.bus.ppu.vblank = false;
                // Fallback frame limiter when no audio device is available
                if !self.bus.apu.has_audio() {
                    const FRAME_DURATION_US: u64 = 1_000_000 / 60;
                    std::thread::sleep(std::time::Duration::from_micros(FRAME_DURATION_US));
                }
            }
        }
    }

    /// Poll keyboard input and update joypad state.
    pub fn poll_keys(&mut self) {
        self.bus.joypad.reset();
        for key in self.display.get_keys() {
            match key {
                minifb::Key::Right => self.bus.joypad.key_down(JoypadKey::Right),
                minifb::Key::Left => self.bus.joypad.key_down(JoypadKey::Left),
                minifb::Key::Up => self.bus.joypad.key_down(JoypadKey::Up),
                minifb::Key::Down => self.bus.joypad.key_down(JoypadKey::Down),
                minifb::Key::Z => self.bus.joypad.key_down(JoypadKey::A),
                minifb::Key::X => self.bus.joypad.key_down(JoypadKey::B),
                minifb::Key::Enter => self.bus.joypad.key_down(JoypadKey::Start),
                minifb::Key::Backspace => self.bus.joypad.key_down(JoypadKey::Select),
                _ => {}
            }
        }
    }

    /// Load ROM data into memory starting at address 0x0000.
    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.bus.cartridge = Cartridge::new(data);
        self.bus.ppu.cgb_mode = self.bus.cartridge.cgb_mode;
    }

    /// Load a boot ROM. Resets PC to 0x0000 and enables boot ROM overlay.
    pub fn load_boot_rom(&mut self, data: Vec<u8>) {
        self.bus.boot_rom = Some(data);
        self.bus.boot_rom_enabled = true;
        self.bus.ppu.cgb_mode = true; // Boot ROM is always CGB code
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
}
