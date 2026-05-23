/// Native Game Boy screen width in pixels.
pub const SCREEN_WIDTH: usize = 160;
/// Native Game Boy screen height in pixels.
pub const SCREEN_HEIGHT: usize = 144;
/// Base address of VRAM in the memory map.
pub const VRAM_OFFSET: u16 = 0x8000;
/// Base address of OAM in the memory map.
pub const OAM_OFFSET: u16 = 0xFE00;

/// Pixel Processing Unit. Tracks scanline timing and produces framebuffer data.
pub struct Ppu {
    pub ly: u8,
    pub lyc: u8,
    pub stat: u8,
    pub scx: u8,
    pub scy: u8,
    pub bgp: u8,
    pub lcdc: u8,
    pub dot: u16,
    pub vblank: bool,

    pub framebuffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub vram: [u8; Ppu::VRAM_SIZE],
    pub oam: [u8; Ppu::OAM_SIZE],
}

impl Ppu {
    const VRAM_SIZE: usize = 16 * 1024;
    const OAM_SIZE: usize = 4 * 40;
    const MAX_CYCLES: u16  = 456;
    const HORIZONTAL_LINES: u8 = 154;
    const VBLANK: u8 = 144;
    const TILE_SIZE: usize = 8;
    const GRID_SIZE: usize = 32;

    /// Create a new PPU in its initial state.
    pub fn new() -> Self {
        Ppu {
            ly: 0,
            lyc: 0,
            stat: 0,
            scx: 0,
            scy: 0,
            bgp: 0,
            lcdc: 0,
            dot: 0,
            vblank: false,
            framebuffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            vram: [0; Ppu::VRAM_SIZE],
            oam: [0; Ppu::OAM_SIZE],
        }
    }

    /// Advance the PPU by the given number of CPU cycles.
    /// Increments the scanline counter and signals when a frame is complete.
    pub fn tick(&mut self, cycles: u8) {
        self.dot += cycles as u16;
        if self.dot >= Ppu::MAX_CYCLES {
            self.ly = (self.ly + 1) % Ppu::HORIZONTAL_LINES;
            if self.ly == Ppu::VBLANK {
                self.vblank = true;
            } else if self.ly < Ppu::VBLANK {
                self.render_scanline();
            }
            self.dot = self.dot % Ppu::MAX_CYCLES;
        }
    }

    /// Render one scanline of the background layer into the framebuffer.
    fn render_scanline(&mut self) {
        if self.lcdc & 0x80 == 0 { return; }
        if self.lcdc & 0x01 == 0 {
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[self.ly as usize * SCREEN_WIDTH + x] = self.color(0);
            }
            return;
        }

        for x in 0..SCREEN_WIDTH {
            // 1. Find pos in background
            let bx = (x as u8).wrapping_add(self.scx);
            let by = self.ly.wrapping_add(self.scy);

            // 2. Find tile ID
            let tx = bx / Ppu::TILE_SIZE as u8;
            let ty = by / Ppu::TILE_SIZE as u8;
            let tile = self.get_tile(tx, ty);

            // 3. Pixel in tile
            let px = bx % 8;
            let py = by % 8;

            // 4. Find the 2 bytes with the pixel info we need for px.
            // Find the start address of the tile we're in.
            // A tile is 8x8 pixel where a pixel is 2b so in memory
            // a tile is represented as 16B
            // Since a pixel is 2b, a row of 8 pixels in a tile is stored as
            // 2 consecutive u8 vals. eg.
            // val1: 01100101
            // val2: 11010010
            let tile_addr = if self.lcdc & 0x10 != 0 {
                // Unsigned: tile 0 at vram[0x0000]
                (tile as usize) * Ppu::TILE_SIZE * 2
            } else {
                // Signed: tile 0 at vram[0x1000], index is i8
                ((0x1000 as isize) + (tile as i8 as isize) * (Ppu::TILE_SIZE * 2) as isize) as usize
            };

            let byte1 = self.vram[tile_addr + (py as usize) * 2];
            let byte2 = self.vram[tile_addr + (py as usize) * 2 + 1];

            // Pixel render left to right so the bit index is flipped.
            // For example pixel 4 is represented in bit 3.
            // pixel: 0  1  2  3  4  5  6  7
            // bit:   7  6  5  4  3  2  1  0
            let bit = 7 - px;
            let low = (byte1 >> bit) & 1;
            let high = (byte2 >> bit) & 1;
            let color_id = (high << 1) | low;
            self.framebuffer[self.ly as usize * SCREEN_WIDTH + x] = self.color(color_id);
        }
    }

    /// Look up the tile index from the background tile map at grid position (x, y).
    fn get_tile(&self, x: u8, y: u8) -> u8 {
        let offset: usize = if self.lcdc & 0x08 != 0 { 0x1C00 } else { 0x1800 };
        self.vram[offset + (y as usize) * Ppu::GRID_SIZE + (x as usize)]
    }

    /// Map a 2-bit color ID through the BGP palette to an ARGB color value.
    fn color(&self, color_id: u8) -> u32 {
        let shade = (self.bgp >> (color_id * 2)) & 0x03;
        match shade {
            0 => 0xFFFFFFFF,    // white
            1 => 0xFFAAAAAA,    // light gray
            2 => 0xFF555555,    // dark gray
            _ => 0xFF000000,    // black
        }
    }
} 
