/// Flat 64KB memory. Will eventually be replaced by a memory bus
/// that dispatches to VRAM, I/O registers, cartridge ROM/RAM, etc.
pub struct Ram {
    memory: [u8; 65536],
}

impl Ram {
    pub fn new() -> Self {
        Ram { memory: [0; 65536] }
    }

    /// Read a byte from the given address.
    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Write a byte to the given address.
    pub fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }

    /// Write a 16-bit value in little-endian (low byte at address, high byte at address+1).
    pub fn write_u16(&mut self, address: u16, val: u16) {
        self.write(address, val as u8);
        self.write(address + 1, (val >> 8) as u8);
    }
}
