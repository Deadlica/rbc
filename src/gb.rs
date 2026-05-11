pub mod ram;
pub mod cpu;
pub mod registers;

/// Top-level Game Boy system. Owns all subsystems (CPU, memory, etc.)
/// and provides the interface for loading ROMs and running emulation.
pub struct Gb {
    cpu: cpu::Cpu,
}

impl Gb {
    /// Create a new Game Boy instance with all subsystems in their initial state.
    pub fn new() -> Self {
        Gb { cpu: cpu::Cpu::new() }
    }

    /// Run the emulation loop indefinitely.
    pub fn run(&mut self) {
        loop {
            self.cpu.step();
        }
    }

    /// Load ROM data into memory starting at address 0x0000.
    pub fn load_rom(&mut self, data: &[u8]) {
        self.cpu.load_rom(data);
    }
}
