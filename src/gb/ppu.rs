/// Native Game Boy screen width in pixels.
pub const SCREEN_WIDTH: usize = 160;
/// Native Game Boy screen height in pixels.
pub const SCREEN_HEIGHT: usize = 144;

/// Pixel Processing Unit. Tracks scanline timing and produces framebuffer data.
pub struct Ppu {
    pub ly: u8,
    pub lyc: u8,
    pub dot: u16,
    pub frame_ready: bool,
    pub framebuffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Ppu {
    const MAX_CYCLES: u16  = 456;
    const HORIZONTAL_LINES: u8 = 154;
    const VBLANK: u8 = 144;

    /// Create a new PPU in its initial state.
    pub fn new() -> Self {
        Ppu {
            ly: 0,
            lyc: 0,
            dot: 0,
            frame_ready: false,
            framebuffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    /// Advance the PPU by the given number of CPU cycles.
    /// Increments the scanline counter and signals when a frame is complete.
    pub fn tick(&mut self, cycles: u8) {
        self.dot += cycles as u16;
        if self.dot >= Ppu::MAX_CYCLES {
            self.ly = (self.ly + 1) % Ppu::HORIZONTAL_LINES;
            if self.ly == Ppu::VBLANK {
                self.frame_ready = true;
            }
            self.dot = self.dot % Ppu::MAX_CYCLES;
        }
    }
}
