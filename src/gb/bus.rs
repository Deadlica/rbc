use crate::gb::ppu::Ppu;

/// Flat 64KB memory. Will eventually be replaced by a memory bus
/// that dispatches to VRAM, I/O registers, cartridge ROM/RAM, etc.
pub struct Bus {
    memory: [u8; 65536],
    pub ppu: Ppu,
}

impl Bus {
    /// Create a new memory instance with all bytes zeroed.
    pub fn new() -> Self {
        Bus {
            memory: [0; 65536],
            ppu: Ppu::new(),
        }
    }

    /// Read a byte from the given address.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.memory[address as usize],
            0x4000..=0x7FFF => self.memory[address as usize],
            0x8000..=0x9FFF => self.memory[address as usize],
            0xA000..=0xBFFF => self.memory[address as usize],
            0xC000..=0xCFFF => self.memory[address as usize],
            0xD000..=0xDFFF => self.memory[address as usize],
            0xE000..=0xFDFF => self.memory[address as usize],
            0xFE00..=0xFE9F => self.memory[address as usize],
            0xFEA0..=0xFEFF => self.memory[address as usize],
            0xFF00..=0xFF7F => self.read_io_registers(address),
            0xFF80..=0xFFFE => self.memory[address as usize],
            0xFFFF..=0xFFFF => self.memory[address as usize],
        }
    }

    /// Write a byte to the given address.
    pub fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x3FFF => self.memory[address as usize] = byte,
            0x4000..=0x7FFF => self.memory[address as usize] = byte,
            0x8000..=0x9FFF => self.memory[address as usize] = byte,
            0xA000..=0xBFFF => self.memory[address as usize] = byte,
            0xC000..=0xCFFF => self.memory[address as usize] = byte,
            0xD000..=0xDFFF => self.memory[address as usize] = byte,
            0xE000..=0xFDFF => self.memory[address as usize] = byte,
            0xFE00..=0xFE9F => self.memory[address as usize] = byte,
            0xFEA0..=0xFEFF => self.memory[address as usize] = byte,
            0xFF00..=0xFF7F => self.write_io_registers(address, byte),
            0xFF80..=0xFFFE => self.memory[address as usize] = byte,
            0xFFFF..=0xFFFF => self.memory[address as usize] = byte,
        };
    }

    pub fn read_io_registers(&self, address: u16) -> u8 {
        match address {
            0xFF44 => self.ppu.ly,
            _ => self.memory[address as usize],
        }
    }

    pub fn write_io_registers(&mut self, address: u16, byte: u8) {
        match address {
            0xFF44 => {}
            _ => self.memory[address as usize] = byte,
        };
    }

    /// Write a 16-bit value in little-endian (low byte at address, high byte at address+1).
    pub fn write_u16(&mut self, address: u16, val: u16) {
        self.write(address, val as u8);
        self.write(address.wrapping_add(1), (val >> 8) as u8);
    }

    /// Load ROM data into memory starting at address 0x0000.
    pub fn load_rom(&mut self, data: &[u8]) {
        self.memory[..data.len()].copy_from_slice(data);
    }
}
