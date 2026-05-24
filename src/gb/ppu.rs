/// Native Game Boy screen width in pixels.
pub const SCREEN_WIDTH: usize = 160;
/// Native Game Boy screen height in pixels.
pub const SCREEN_HEIGHT: usize = 144;
/// Base address of VRAM in the memory map.
pub const VRAM_OFFSET: u16 = 0x8000;
/// Base address of OAM in the memory map.
pub const OAM_OFFSET: u16 = 0xFE00;

// Modes
#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Hblank, Vblank, OamScan, PixelTransfer,
}


/// Pixel Processing Unit. Tracks scanline timing and produces framebuffer data.
pub struct Ppu {
    pub ly: u8,
    pub lyc: u8,
    pub stat: u8,
    pub scx: u8,
    pub scy: u8,
    pub wx: u8,
    pub wy: u8,
    pub bgp: u8,
    pub lcdc: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub window_line: u8,
    pub dot: u16,
    pub vblank: bool,
    pub cgb_mode: bool,

    pub framebuffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub vram: [u8; Ppu::VRAM_SIZE],
    pub oam: [u8; Ppu::OAM_SIZE],

    pub bg_palette_ram: [u8; 64],
    pub obj_palette_ram: [u8; 64],
    pub bg_palette_index: u8,
    pub obj_palette_index: u8,
    pub vram_bank: u8,

    pub hdma_src: u16,
    pub hdma_dst: u16,
    pub hdma_len: u8,
    pub hdma_active: bool,

    mode: Mode,
    pub stat_irq: bool,
}

impl Ppu {
    const VRAM_SIZE: usize = 16 * 1024;
    const OAM_SIZE: usize = 4 * 40;
    const MAX_CYCLES: u16  = 456;
    const HORIZONTAL_LINES: u8 = 154;

    // LCDC register bits
    const LCDC_BG_ENABLE: u8 = 0x01;
    const LCDC_OBJ_ENABLE: u8 = 0x02;
    const LCDC_OBJ_SIZE: u8 = 0x04;
    const LCDC_BG_MAP: u8 = 0x08;
    const LCDC_TILE_DATA: u8 = 0x10;
    const LCDC_WIN_ENABLE: u8 = 0x20;
    const LCDC_WIN_MAP: u8 = 0x40;
    const LCDC_LCD_ENABLE: u8 = 0x80;

    // VRAM offsets
    const TILE_MAP_0: usize = 0x1800;
    const TILE_MAP_1: usize = 0x1C00;
    const VRAM_BANK_SIZE: usize = 0x2000;

    // CGB tile attribute bits
    const ATTR_PALETTE: u8 = 0x07;
    const ATTR_VRAM_BANK: u8 = 0x08;
    const ATTR_X_FLIP: u8 = 0x20;
    const ATTR_Y_FLIP: u8 = 0x40;
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
            wx: 0,
            wy: 0,
            bgp: 0,
            lcdc: 0,
            obp0: 0,
            obp1: 0,
            window_line: 0,
            dot: 0,
            vblank: false,
            cgb_mode: false,
            framebuffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            vram: [0; Ppu::VRAM_SIZE],
            oam: [0; Ppu::OAM_SIZE],
            bg_palette_ram: [0; 64],
            obj_palette_ram: [0; 64],
            bg_palette_index: 0,
            obj_palette_index: 0,
            vram_bank: 0,
            hdma_src: 0,
            hdma_dst: 0,
            hdma_len: 0,
            hdma_active: false,
            mode: Mode::OamScan,
            stat_irq: false,
        }
    }

    /// Advance the PPU by the given number of CPU cycles.
    /// Increments the scanline counter and signals when a frame is complete.
    pub fn tick(&mut self, cycles: u8) -> bool {
        // OLD SOLUTION
        /*
        self.dot += cycles as u16;
        if self.dot >= Ppu::MAX_CYCLES {
            self.ly = (self.ly + 1) % Ppu::HORIZONTAL_LINES;
            if self.ly == Ppu::VBLANK {
                self.vblank = true;
                self.window_line = 0;
            } else if self.ly < Ppu::VBLANK {
                self.render_scanline();
                self.render_sprites();
            }
            self.dot = self.dot % Ppu::MAX_CYCLES;
            return true;
        }
        false
        */
        self.dot += cycles as u16;
        let old_mode = self.mode;
        self.mode = self.update_mode();
        if self.mode != old_mode {
            self.update_stat();
        }
        if self.dot >= Ppu::MAX_CYCLES {
            self.ly = (self.ly + 1) % Ppu::HORIZONTAL_LINES;
            self.dot = self.dot % Ppu::MAX_CYCLES;
            if self.ly == self.lyc {
                self.stat |= 1 << 2;
                if self.stat & 0x40 != 0 { self.stat_irq = true; }
            } else {
                self.stat &= !(1 << 2);
            }
            if self.ly == Ppu::VBLANK {
                self.vblank = true;
                self.window_line = 0;
            } else if self.ly < Ppu::VBLANK {
                self.render_scanline();
                self.render_sprites();
            }
            return true;
        }
        false
    }

    /// Render one scanline of the background layer into the framebuffer.
    fn render_scanline(&mut self) {
        if self.lcdc & Ppu::LCDC_LCD_ENABLE == 0 { return; }
        if self.lcdc & Ppu::LCDC_BG_ENABLE == 0 {
            for x in 0..SCREEN_WIDTH {
                self.framebuffer[self.ly as usize * SCREEN_WIDTH + x] = self.bg_color(0);
            }
            return;
        }

        for x in 0..SCREEN_WIDTH {
            let color = self.window_pixel(x as u8).unwrap_or_else(|| self.bg_pixel(x as u8));
            self.framebuffer[self.ly as usize * SCREEN_WIDTH + x] = color;
        }
        if self.lcdc & Ppu::LCDC_WIN_ENABLE != 0 && self.ly >= self.wy {
            self.window_line += 1;
        }
    }

    /// Render sprites for the current scanline.
    fn render_sprites(&mut self) {
        if self.lcdc & Ppu::LCDC_OBJ_ENABLE == 0 { return; }
        let sprite_height: u8 = if self.lcdc & Ppu::LCDC_OBJ_SIZE != 0 { 16 } else { 8 };

        for i in 0..40 {
            let offset = i * 4;
            let sy = self.oam[offset].wrapping_sub(16);
            if self.ly < sy || self.ly >= sy.wrapping_add(sprite_height) {
                continue;
            }
            self.draw_sprite(offset, sy, sprite_height);
        }
    }

    /// Draw a single sprite's pixels for the current scanline.
    fn draw_sprite(&mut self, offset: usize, sy: u8, sprite_height: u8) {
        let sx = self.oam[offset + 1].wrapping_sub(8);
        let tile = self.oam[offset + 2];
        let attrs = self.oam[offset + 3];

        let y_flip = attrs & Ppu::ATTR_Y_FLIP != 0;
        let x_flip = attrs & Ppu::ATTR_X_FLIP != 0;
        let palette_num = attrs & Ppu::ATTR_PALETTE;
        let palette = if attrs & 0x10 != 0 { self.obp1 } else { self.obp0 };
        let behind_bg = attrs & 0x80 != 0;

        let row = if y_flip { sprite_height - 1 - (self.ly - sy) } else { self.ly - sy };
        let tile_addr = (tile as usize) * 16 + (row as usize) * 2;
        let byte1 = self.vram[tile_addr];
        let byte2 = self.vram[tile_addr + 1];

        for px in 0..8u8 {
            let bit = if x_flip { px } else { 7 - px };
            let color_id = ((byte2 >> bit) & 1) << 1 | ((byte1 >> bit) & 1);
            if color_id == 0 { continue; }

            let screen_x = sx.wrapping_add(px) as usize;
            if screen_x >= SCREEN_WIDTH { continue; }

            let fb_idx = self.ly as usize * SCREEN_WIDTH + screen_x;
            if behind_bg && self.framebuffer[fb_idx] != self.bg_color(0) {
                continue;
            }
            self.framebuffer[fb_idx] = self.obj_color(color_id, palette_num, palette);
        }
    }

    /// Get the 2-bit color ID for a background pixel at screen x position.
    fn bg_pixel(&self, x: u8) -> u32 {
        let bx = x.wrapping_add(self.scx);
        let by = self.ly.wrapping_add(self.scy);

        let tx = bx / Ppu::TILE_SIZE as u8;
        let ty = by / Ppu::TILE_SIZE as u8;

        let map_offset: usize = if self.lcdc & Ppu::LCDC_BG_MAP != 0 { Ppu::TILE_MAP_1 } else { Ppu::TILE_MAP_0 };
        let map_index = ty as usize * Ppu::GRID_SIZE + tx as usize;

        let tile = self.vram[map_offset + map_index];

        let (palette_num, tile_bank, x_flip, y_flip) = if self.cgb_mode {
            let attr = self.vram[Ppu::VRAM_BANK_SIZE + map_offset + map_index];
            (attr & Ppu::ATTR_PALETTE, (attr & Ppu::ATTR_VRAM_BANK) >> 3, attr & Ppu::ATTR_X_FLIP != 0, attr & Ppu::ATTR_Y_FLIP != 0)
        } else {
            (0, 0, false, false)
        };

        let py = if y_flip { 7 - (by % 8) } else { by % 8 };
        let tile_addr = (tile_bank as usize) * Ppu::VRAM_BANK_SIZE + self.tile_data_addr(tile);
        let byte1 = self.vram[tile_addr + (py as usize) * 2];
        let byte2 = self.vram[tile_addr + (py as usize) * 2 + 1];

        let bit = if x_flip { bx % 8 } else { 7 - (bx % 8) };
        let low = (byte1 >> bit) & 1;
        let high = (byte2 >> bit) & 1;
        let color_id = (high << 1) | low;

        if self.cgb_mode {
            self.cgb_color(&self.bg_palette_ram, palette_num, color_id)
        } else {
            self.shade_to_color(color_id, self.bgp)
        }
    }

    /// Get the 2-bit color ID for a window pixel at screen x position, or None if window doesn't cover this pixel.
    fn window_pixel(&self, x: u8) -> Option<u32> {
        if self.lcdc & Ppu::LCDC_WIN_ENABLE == 0 { return None; }
        if self.ly < self.wy { return None; }
        if x < self.wx.wrapping_sub(7) { return None; }

        let wx_offset = x - self.wx.wrapping_sub(7);
        let wy_offset = self.window_line;

        let tx = wx_offset / 8;
        let ty = wy_offset / 8;
        let map_offset: usize = if self.lcdc & Ppu::LCDC_WIN_MAP != 0 { Ppu::TILE_MAP_1 } else { Ppu::TILE_MAP_0 };
        let tile = self.vram[map_offset + (ty as usize) * Ppu::GRID_SIZE + (tx as usize)];

        let tile_addr = self.tile_data_addr(tile);
        let row = (wy_offset % 8) as usize;
        let byte1 = self.vram[tile_addr + row * 2];
        let byte2 = self.vram[tile_addr + row * 2 + 1];
        let bit = 7 - (wx_offset % 8);
        let low = (byte1 >> bit) & 1;
        let high = (byte2 >> bit) & 1;
        let color_id = (high << 1) | low;
        if self.cgb_mode {
            Some(self.cgb_color(&self.bg_palette_ram, 0, color_id))
        } else {
            Some(self.shade_to_color(color_id, self.bgp))
        }
    }

    /// Resolve tile data address based on LCDC bit 4 addressing mode.
    fn tile_data_addr(&self, tile: u8) -> usize {
        if self.lcdc & Ppu::LCDC_TILE_DATA != 0 {
            (tile as usize) * 16
        } else {
            ((0x1000 as isize) + (tile as i8 as isize) * 16) as usize
        }
    }

    /// Get the final BG color, dispatching to CGB or DMG palette.
    fn bg_color(&self, color_id: u8) -> u32 {
        if self.cgb_mode {
            self.cgb_color(&self.bg_palette_ram, 0, color_id)
        } else {
            self.shade_to_color(color_id, self.bgp)
        }
    }

    /// Get the final color for a sprite pixel, using CGB or DMG palette.
    fn obj_color(&self, color_id: u8, palette_num: u8, dmg_palette: u8) -> u32 {
        if self.cgb_mode {
            self.cgb_color(&self.obj_palette_ram, palette_num, color_id)
        } else {
            self.shade_to_color(color_id, dmg_palette)
        }
    }

    /// Map a 2-bit color ID through a palette to an ARGB color value.
    fn shade_to_color(&self, color_id: u8, palette: u8) -> u32 {
        let shade = (palette >> (color_id * 2)) & 0x03;
        match shade {
            0 => 0xFFFFFFFF,
            1 => 0xFFAAAAAA,
            2 => 0xFF555555,
            _ => 0xFF000000,
        }
    }

    /// Convert a CGB palette color to ARGB. Reads 2 bytes from palette RAM.
    fn cgb_color(&self, palette_ram: &[u8], palette_num: u8, color_id: u8) -> u32 {
        let index = (palette_num as usize) * 8 + (color_id as usize) * 2;
        let lo = palette_ram[index] as u16;
        let hi = palette_ram[index + 1] as u16;
        let rgb555 = (hi << 8) | lo;
        let r = ((rgb555 & 0x1F) << 3) as u8;
        let g = (((rgb555 >> 5) & 0x1F) << 3) as u8;
        let b = (((rgb555 >> 10) & 0x1F) << 3) as u8;
        0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
    }

    /// Determine the current PPU mode based on ly and dot position.
    fn update_mode(&self) -> Mode {
        if self.ly >= Ppu::VBLANK {
            Mode::Vblank
        } else if self.dot < 80 {
            Mode::OamScan
        } else if self.dot < 252 {
            Mode::PixelTransfer
        } else {
            Mode::Hblank
        }
    }

    /// Fire STAT interrupt if the current mode's enable bit is set.
    fn update_stat(&mut self) {
        match self.mode {
            Mode::Hblank if self.stat & 0x08 != 0 => self.stat_irq = true,
            Mode::Vblank if self.stat & 0x10 != 0 => self.stat_irq = true,
            Mode::OamScan if self.stat & 0x20 != 0 => self.stat_irq = true,
            _ => {}
        }
    }

    /// Return the current PPU mode as a 2-bit value for the STAT register.
    pub fn mode_bits(&self) -> u8 {
        match self.mode {
            Mode::Hblank => 0,
            Mode::Vblank => 1,
            Mode::OamScan => 2,
            Mode::PixelTransfer => 3,
        }
    }
}
