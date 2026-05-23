/// MBC1 cartridge. Handles ROM banking for games larger than 32KB.
pub struct Cartridge {
    rom: Vec<u8>,
    /// Current ROM bank mapped to 0x4000–0x7FFF. Always >= 1.
    bank: u8,
}

impl Cartridge {
    /// Create a new cartridge with the given ROM data.
    pub fn new(data: Vec<u8>) -> Self {
        Cartridge {
            rom: data,
            bank: 1,
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
            0x0000..=0x1FFF => {},                   // RAM enable (ignored)
            0x2000..=0x3FFF => {                     // ROM bank select (lower 5 bits)
                self.bank = byte & 0x1F;
                if self.bank == 0 { self.bank = 1; }
            }
            0x4000..=0x5FFF => {},                   // RAM bank / upper ROM bits (ignored)
            0x6000..=0x7FFF => {},                   // Banking mode select (ignored)
            _ => {},
        }
    }
}
