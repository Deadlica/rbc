/// SM83 CPU registers for the Game Boy Color.
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    /// Flags register. Only upper 4 bits (Z, N, H, C) are used; lower 4 are always 0.
    pub f: u8,
    pub h: u8,
    pub l: u8,
    /// Stack pointer.
    pub sp: u16,
    /// Program counter.
    pub pc: u16,
}

impl Registers {
    const Z_BIT: u8 = 7;
    const N_BIT: u8 = 6;
    const H_BIT: u8 = 5;
    const C_BIT: u8 = 4;

    /// Initialize registers to CGB post-boot values.
    /// See <https://gbdev.io/pandocs/Power_Up_Sequence.html>
    pub fn new() -> Self {
        Registers {
            a: 0x11,
            b: 0x00,
            c: 0x00,
            d: 0xFF,
            e: 0x56,
            f: 0x80, // Z=1, N=0, H=0, C=0
            h: 0x00,
            l: 0x0D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    //  16 bit register pairs
    //  ┌──────┬──────┬─────┐
    //  │ Pair │ High │ Low │
    //  ├──────┼──────┼─────┤
    //  │ AF   │ A    │ F   │
    //  │ BC   │ B    │ C   │
    //  │ DE   │ D    │ E   │
    //  │ HL   │ H    │ L   │
    //  └──────┴──────┴─────┘

    /// Read the AF register pair.
    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    /// Read the BC register pair.
    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    /// Read the DE register pair.
    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    /// Read the HL register pair.
    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    /// Write to the AF register pair. Lower 4 bits of F are always masked to 0.
    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0xF0) as u8;
    }

    /// Write to the BC register pair.
    pub fn set_bc(&mut self, val: u16) {
      self.b = (val >> 8) as u8;
      self.c = val as u8;
    }

    /// Write to the DE register pair.
    pub fn set_de(&mut self, val: u16) {
      self.d = (val >> 8) as u8;
      self.e = val as u8;
    }

    /// Write to the HL register pair.
    pub fn set_hl(&mut self, val: u16) {
      self.h = (val >> 8) as u8;
      self.l = val as u8;
    }

    /// Get the Zero flag (bit 7 of F).
    pub fn get_z(&self) -> bool {
        (self.f & (1 << Registers::Z_BIT)) != 0
    }

    /// Get the Subtract flag (bit 6 of F).
    pub fn get_n(&self) -> bool {
        (self.f & (1 << Registers::N_BIT)) != 0
    }

    /// Get the Half-carry flag (bit 5 of F).
    pub fn get_h(&self) -> bool {
        (self.f & (1 << Registers::H_BIT)) != 0
    }

    /// Get the Carry flag (bit 4 of F).
    pub fn get_c(&self) -> bool {
        (self.f & (1 << Registers::C_BIT)) != 0
    }

    /// Set the Zero flag (bit 7 of F).
    pub fn set_z(&mut self, val: bool) {
        self.f = (self.f & !(1 << Registers::Z_BIT)) | ((val as u8) << Registers::Z_BIT);
    }

    /// Set the Subtract flag (bit 6 of F).
    pub fn set_n(&mut self, val: bool) {
        self.f = (self.f & !(1 << Registers::N_BIT)) | ((val as u8) << Registers::N_BIT);
    }

    /// Set the Half-carry flag (bit 5 of F).
    pub fn set_h(&mut self, val: bool) {
        self.f = (self.f & !(1 << Registers::H_BIT)) | ((val as u8) << Registers::H_BIT);
    }

    /// Set the Carry flag (bit 4 of F).
    pub fn set_c(&mut self, val: bool) {
        self.f = (self.f & !(1 << Registers::C_BIT)) | ((val as u8) << Registers::C_BIT);
    }
}
