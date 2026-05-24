use std::time::Instant;

/// Cartridge with MBC1/MBC3 support. Handles ROM/RAM banking and RTC.
pub struct Cartridge {
    rom: Vec<u8>,
    pub ram: Vec<u8>,
    /// Current ROM bank mapped to 0x4000–0x7FFF. Always >= 1 for MBC1/3.
    bank: u16,
    /// Current RAM bank or RTC register select mapped to 0xA000–0xBFFF.
    ram_bank: u8,
    /// Whether external RAM/RTC is accessible.
    ram_enabled: bool,
    /// MBC type from cartridge header (0x0147).
    mbc_type: u8,
    pub cgb_mode: bool,
    // RTC registers
    rtc_s: u8,
    rtc_m: u8,
    rtc_h: u8,
    rtc_dl: u8,
    rtc_dh: u8,
    rtc_latch_prev: u8,
    rtc_start: Instant,
}

impl Cartridge {
    /// Create a new cartridge with the given ROM data.
    pub fn new(data: Vec<u8>) -> Self {
        let mbc_type = if data.len() > 0x0147 { data[0x0147] } else { 0 };
        let cgb_flag = if data.len() > 0x0143 { data[0x0143] } else { 0 };
        Cartridge {
            rom: data,
            ram: vec![0; 32 * 1024],
            bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            mbc_type,
            cgb_mode: cgb_flag == 0x80 || cgb_flag == 0xC0,
            rtc_s: 0,
            rtc_m: 0,
            rtc_h: 0,
            rtc_dl: 0,
            rtc_dh: 0,
            rtc_latch_prev: 0xFF,
            rtc_start: Instant::now(),
        }
    }

    /// Read a byte from the cartridge ROM (0x0000–0x7FFF).
    /// Bank 0 is fixed at 0x0000–0x3FFF, switchable bank at 0x4000–0x7FFF.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[(self.bank as usize) * 0x4000 + (address as usize - 0x4000)],
            _ => 0xFF,
        }
    }

    /// Handle writes to the cartridge address space (MBC control registers).
    pub fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = (byte & 0x0F) == 0x0A,
            0x2000..=0x3FFF => self.write_bank(address, byte),
            0x4000..=0x5FFF => self.write_ram_bank(byte),
            0x6000..=0x7FFF => self.write_latch(byte),
            _ => {},
        }
    }

    /// Set the ROM bank number based on MBC type.
    fn write_bank(&mut self, address: u16, byte: u8) {
        match self.mbc_type {
            0x19..=0x1E => {
                if address < 0x3000 {
                    self.bank = (self.bank & 0x100) | byte as u16;
                } else {
                    self.bank = (self.bank & 0xFF) | ((byte as u16 & 0x01) << 8);
                }
            }
            _ => {
                let mask = match self.mbc_type {
                    0x01..=0x03 => 0x1F,
                    0x0F..=0x13 => 0x7F,
                    _ => 0xFF,
                };
                self.bank = (byte & mask) as u16;
                if self.bank == 0 { self.bank = 1; }
            }
        }
    }

    /// Set the RAM bank or RTC register select.
    fn write_ram_bank(&mut self, byte: u8) {
        match self.mbc_type {
            0x19..=0x1E => self.ram_bank = byte & 0x0F,
            _ => self.ram_bank = byte,
        }
    }

    /// Handle RTC latch (MBC3: write 0x00 then 0x01 to latch time).
    fn write_latch(&mut self, byte: u8) {
        if self.rtc_latch_prev == 0x00 && byte == 0x01 {
            self.latch_rtc();
        }
        self.rtc_latch_prev = byte;
    }

    /// Read a byte from external RAM or RTC register (0xA000–0xBFFF).
    pub fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled { return 0xFF; }
        match self.ram_bank {
            0x00..=0x03 => self.ram[(self.ram_bank as usize) * 0x2000 + (address as usize - 0xA000)],
            0x08 => self.rtc_s,
            0x09 => self.rtc_m,
            0x0A => self.rtc_h,
            0x0B => self.rtc_dl,
            0x0C => self.rtc_dh,
            _ => 0xFF,
        }
    }

    /// Write a byte to external RAM or RTC register (0xA000–0xBFFF).
    pub fn write_ram(&mut self, address: u16, byte: u8) {
        if !self.ram_enabled { return; }
        match self.ram_bank {
            0x00..=0x03 => self.ram[(self.ram_bank as usize) * 0x2000 + (address as usize - 0xA000)] = byte,
            0x08 => self.rtc_s = byte,
            0x09 => self.rtc_m = byte,
            0x0A => self.rtc_h = byte,
            0x0B => self.rtc_dl = byte,
            0x0C => self.rtc_dh = byte,
            _ => {},
        }
    }

    /// Latch the current real time into the RTC registers.
    fn latch_rtc(&mut self) {
        // Don't tick if halted
        if self.rtc_dh & 0x40 != 0 { return; }

        let elapsed = self.rtc_start.elapsed().as_secs();
        let mut total_s = elapsed
            + self.rtc_s as u64
            + self.rtc_m as u64 * 60
            + self.rtc_h as u64 * 3600
            + ((self.rtc_dl as u64) | ((self.rtc_dh as u64 & 0x01) << 8)) * 86400;

        let days = total_s / 86400;
        total_s %= 86400;
        self.rtc_h = (total_s / 3600) as u8;
        total_s %= 3600;
        self.rtc_m = (total_s / 60) as u8;
        self.rtc_s = (total_s % 60) as u8;
        self.rtc_dl = (days & 0xFF) as u8;
        self.rtc_dh = (self.rtc_dh & 0xFE) | ((days >> 8) & 0x01) as u8;
        if days > 511 { self.rtc_dh |= 0x80; } // day overflow

        self.rtc_start = Instant::now();
    }
}
