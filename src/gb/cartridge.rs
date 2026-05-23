/// Cartridge with MBC1/MBC3 support. Handles ROM/RAM banking.
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    /// Current ROM bank mapped to 0x4000–0x7FFF. Always >= 1.
    bank: u8,
    /// Current RAM bank mapped to 0xA000–0xBFFF.
    ram_bank: u8,
    /// Whether external RAM is accessible.
    ram_enabled: bool,
    /// MBC type from cartridge header (0x0147).
    mbc_type: u8,
}

impl Cartridge {
    /// Create a new cartridge with the given ROM data.
    pub fn new(data: Vec<u8>) -> Self {
        let mbc_type = if data.len() > 0x0147 { data[0x0147] } else { 0 };
        Cartridge {
            rom: data,
            ram: vec![0; 32 * 1024],
            bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            mbc_type: mbc_type,
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

    /// Handle writes to the cartridge address space (MBC1 control registers).
    pub fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = (byte & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                self.bank = match self.mbc_type {
                    0x01..=0x03 => byte & 0x1F,
                    0x0F..=0x13 => byte & 0x7F,
                    _ => byte,
                };
                if self.bank == 0 { self.bank = 1; }
            }
            0x4000..=0x5FFF => self.ram_bank = byte & 0x03,
            0x6000..=0x7FFF => {},                   // Banking mode select (ignored for now)
            _ => {},
        }
    }

    /// Read a byte from external RAM (0xA000–0xBFFF). Returns 0xFF if RAM is disabled.
    pub fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled { return 0xFF; }
        self.ram[(self.ram_bank as usize) * 0x2000 + (address as usize - 0xA000)]
    }

    /// Write a byte to external RAM (0xA000–0xBFFF). Ignored if RAM is disabled.
    pub fn write_ram(&mut self, address: u16, byte: u8) {
        if !self.ram_enabled { return; }
        self.ram[(self.ram_bank as usize) * 0x2000 + (address as usize - 0xA000)] = byte;
    }
}
