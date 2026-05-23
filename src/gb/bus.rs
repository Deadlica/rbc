use crate::gb::ppu::{self, Ppu};
use crate::gb::timer::Timer;
use crate::gb::cartridge::Cartridge;
use crate::gb::joypad::Joypad;

pub const MEMORY_SIZE: usize = 64 * 1024;

pub enum Interrupt {
    VBLANK,
    LCD,
    TIMER,
    SERIAL,
    JOYPAD,
}

/// Flat 64KB memory. Will eventually be replaced by a memory bus
/// that dispatches to VRAM, I/O registers, cartridge ROM/RAM, etc.
pub struct Bus {
    memory: [u8; MEMORY_SIZE],
    pub ppu: Ppu,
    pub timer: Timer,
    pub cartridge: Cartridge,
    pub joypad: Joypad,

    r_ie: u8,
    r_if: u8,
}

impl Bus {
    /// Create a new memory instance with all bytes zeroed.
    pub fn new() -> Self {
        Bus {
            memory: [0; MEMORY_SIZE],
            ppu: Ppu::new(),
            timer: Timer::new(),
            cartridge: Cartridge::new(vec![]),
            joypad: Joypad::new(),
            r_ie: 0,
            r_if: 0,
        }
    }

    /// Read a byte from the given address.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.cartridge.read(address),
            0x4000..=0x7FFF => self.cartridge.read(address),
            0x8000..=0x9FFF => self.ppu.vram[(address - ppu::VRAM_OFFSET) as usize],
            0xA000..=0xBFFF => self.memory[address as usize],
            0xC000..=0xCFFF => self.memory[address as usize],
            0xD000..=0xDFFF => self.memory[address as usize],
            0xE000..=0xFDFF => self.memory[address as usize],
            0xFE00..=0xFE9F => self.ppu.oam[(address - ppu::OAM_OFFSET) as usize],
            0xFEA0..=0xFEFF => self.memory[address as usize],
            0xFF00..=0xFF7F => self.read_io_registers(address),
            0xFF80..=0xFFFE => self.memory[address as usize],
            0xFFFF => self.r_ie
        }
    }

    /// Write a byte to the given address.
    pub fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x3FFF => self.cartridge.write(address, byte),
            0x4000..=0x7FFF => self.cartridge.write(address, byte),
            0x8000..=0x9FFF => self.ppu.vram[(address - ppu::VRAM_OFFSET) as usize] = byte,
            0xA000..=0xBFFF => self.memory[address as usize] = byte,
            0xC000..=0xCFFF => self.memory[address as usize] = byte,
            0xD000..=0xDFFF => self.memory[address as usize] = byte,
            0xE000..=0xFDFF => self.memory[address as usize] = byte,
            0xFE00..=0xFE9F => self.ppu.oam[(address - ppu::OAM_OFFSET) as usize] = byte,
            0xFEA0..=0xFEFF => self.memory[address as usize] = byte,
            0xFF00..=0xFF7F => self.write_io_registers(address, byte),
            0xFF80..=0xFFFE => self.memory[address as usize] = byte,
            0xFFFF => self.r_ie = byte,
        };
    }

    /// Read from an I/O register (0xFF00–0xFF7F).
    pub fn read_io_registers(&self, address: u16) -> u8 {
        match address {
            0xFF00 => self.joypad.read(),
            0xFF04 => (self.timer.counter >> 8) as u8,
            0xFF05 => self.timer.tima,
            0xFF06 => self.timer.tma,
            0xFF07 => self.timer.tac,
            0xFF0F => self.r_if,
            0xFF40 => self.ppu.lcdc,
            0xFF41 => self.ppu.stat,
            0xFF42 => self.ppu.scy,
            0xFF43 => self.ppu.scx,
            0xFF44 => self.ppu.ly,
            0xFF45 => self.ppu.lyc,
            0xFF47 => self.ppu.bgp,
            _ => self.memory[address as usize],
        }
    }

    /// Write to an I/O register (0xFF00–0xFF7F). Some registers are read-only.
    pub fn write_io_registers(&mut self, address: u16, byte: u8) {
        match address {
            0xFF00 => self.joypad.write(byte),
            0xFF04 => self.timer.counter = 0,
            0xFF05 => self.timer.tima = byte,
            0xFF06 => self.timer.tma = byte,
            0xFF07 => self.timer.tac = byte,
            0xFF0F => self.r_if = byte,
            0xFF40 => self.ppu.lcdc = byte,
            0xFF41 => self.ppu.stat = byte,
            0xFF42 => self.ppu.scy = byte,
            0xFF43 => self.ppu.scx = byte,
            0xFF44 => {} // LY is reac-only
            0xFF45 => self.ppu.lyc = byte,
            0xFF47 => self.ppu.bgp = byte,
            _ => self.memory[address as usize] = byte,
        };
    }

    /// Write a 16-bit value in little-endian (low byte at address, high byte at address+1).
    pub fn write_u16(&mut self, address: u16, val: u16) {
        self.write(address, val as u8);
        self.write(address.wrapping_add(1), (val >> 8) as u8);
    }

    /// Request an interrupt by setting the corresponding bit in IF.
    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.r_if |= 1 << (interrupt as u8);
    }

    /// Clear an interrupt bit in IF after it has been handled.
    pub fn clear_interrupt(&mut self, bit: u8) {
        self.r_if &= !(1 << bit);
    }

    /// Return the highest-priority pending interrupt (IF & IE), or None.
    pub fn pending_interrupt(&self) -> Option<u8> {
        let pending = self.r_if & self.r_ie & 0x1F;
        if pending == 0 { return None; }
        Some(pending.trailing_zeros() as u8)
    }
}
