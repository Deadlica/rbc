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
            0x8000..=0x9FFF => self.ppu.vram[(self.ppu.vram_bank as u16 * 0x2000 + (address - ppu::VRAM_OFFSET)) as usize],
            0xA000..=0xBFFF => self.cartridge.read_ram(address),
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
            0x8000..=0x9FFF => self.ppu.vram[(self.ppu.vram_bank as u16 * 0x2000 + (address - ppu::VRAM_OFFSET)) as usize] = byte,
            0xA000..=0xBFFF => self.cartridge.write_ram(address, byte),
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
            0xFF48 => self.ppu.obp0,
            0xFF49 => self.ppu.obp1,
            0xFF4A => self.ppu.wy,
            0xFF4B => self.ppu.wx,
            0xFF4F => self.ppu.vram_bank,
            0xFF51 => (self.ppu.hdma_src >> 8) as u8,
            0xFF52 => self.ppu.hdma_src as u8,
            0xFF53 => (self.ppu.hdma_dst >> 8) as u8,
            0xFF54 => self.ppu.hdma_dst as u8,
            0xFF55 => if self.ppu.hdma_active { self.ppu.hdma_len } else { 0xFF },
            0xFF68 => self.ppu.bg_palette_index,
            0xFF69 => self.ppu.bg_palette_ram[(self.ppu.bg_palette_index & 0x3F) as usize],
            0xFF6A => self.ppu.obj_palette_index,
            0xFF6B => self.ppu.obj_palette_ram[(self.ppu.obj_palette_index & 0x3F) as usize],
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
            0xFF46 => {
                // OAM DMA: copy 160 bytes from (byte << 8) into OAM
                let src = (byte as u16) << 8;
                for i in 0..160 {
                    self.ppu.oam[i] = self.read(src + i as u16);
                }
            }
            0xFF47 => self.ppu.bgp = byte,
            0xFF48 => self.ppu.obp0 = byte,
            0xFF49 => self.ppu.obp1 = byte,
            0xFF4A => self.ppu.wy = byte,
            0xFF4B => self.ppu.wx = byte,
            0xFF4F => self.ppu.vram_bank = byte & 0x01,
            0xFF51 => self.ppu.hdma_src = (self.ppu.hdma_src & 0x00FF) | ((byte as u16) << 8),
            0xFF52 => self.ppu.hdma_src = (self.ppu.hdma_src & 0xFF00) | ((byte & 0xF0) as u16),
            0xFF53 => self.ppu.hdma_dst = (self.ppu.hdma_dst & 0x00FF) | (((byte & 0x1F) as u16) << 8),
            0xFF54 => self.ppu.hdma_dst = (self.ppu.hdma_dst & 0xFF00) | ((byte & 0xF0) as u16),
            0xFF55 => {
                let len = (byte & 0x7F) + 1;
                if byte & 0x80 == 0 {
                    // GDMA: immediate copy
                    let src = self.ppu.hdma_src;
                    let dst = 0x8000 | self.ppu.hdma_dst;
                    for i in 0..(len as u16 * 16) {
                        let b = self.read(src + i);
                        self.write(dst + i, b);
                    }
                    self.ppu.hdma_active = false;
                    self.ppu.hdma_len = 0xFF;
                } else {
                    // HBlank DMA: set up, copy happens per scanline
                    self.ppu.hdma_active = true;
                    self.ppu.hdma_len = byte & 0x7F;
                }
            }
            0xFF68 => self.ppu.bg_palette_index = byte,
            0xFF69 => {
                self.ppu.bg_palette_ram[(self.ppu.bg_palette_index & 0x3F) as usize] = byte;
                if self.ppu.bg_palette_index & 0x80 != 0 {
                    self.ppu.bg_palette_index = (self.ppu.bg_palette_index & 0x80) | ((self.ppu.bg_palette_index + 1) & 0x3F);
                }
            }
            0xFF6A => self.ppu.obj_palette_index = byte,
            0xFF6B => {
                self.ppu.obj_palette_ram[(self.ppu.obj_palette_index & 0x3F) as usize] = byte;
                if self.ppu.obj_palette_index & 0x80 != 0 {
                    self.ppu.obj_palette_index = (self.ppu.obj_palette_index & 0x80) | ((self.ppu.obj_palette_index + 1) & 0x3F);
                }
            }
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

    /// Perform one HBlank DMA step: copy 16 bytes from source to VRAM.
    pub fn hdma_tick(&mut self) {
        if !self.ppu.hdma_active { return; }
        let src = self.ppu.hdma_src;
        let dst = 0x8000 | self.ppu.hdma_dst;
        for i in 0..16 {
            let b = self.read(src + i);
            self.write(dst + i, b);
        }
        self.ppu.hdma_src += 16;
        self.ppu.hdma_dst += 16;
        if self.ppu.hdma_len == 0 {
            self.ppu.hdma_active = false;
        } else {
            self.ppu.hdma_len -= 1;
        }
  }
}
