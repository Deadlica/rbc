pub mod bus;
pub mod cpu;
pub mod registers;
pub mod ppu;
pub mod timer;
pub mod display;

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
            self.bus.ppu.tick(elapsed_cycles);
            let interrupt = self.bus.timer.tick(elapsed_cycles);
            if interrupt {
                self.bus.request_interrupt(bus::Interrupt::TIMER);
            }
            if self.bus.ppu.vblank {
                self.bus.request_interrupt(bus::Interrupt::VBLANK);
                self.display.update(&self.bus.ppu.framebuffer);
                self.bus.ppu.vblank = false;
            }
        }
    }

    /// Load ROM data into memory starting at address 0x0000.
    pub fn load_rom(&mut self, data: &[u8]) {
        self.cpu.load_rom(data, &mut self.bus);
    }
}
