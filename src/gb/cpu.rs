use crate::gb::registers::Registers;
use crate::gb::ram::Ram;

/// SM83 CPU. Owns registers and memory, executes instructions via fetch-decode-execute.
pub struct Cpu {
    regs: Registers,
    mem: Ram,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs: Registers::new(),
            mem: Ram::new(),
        }
    }

    /// Execute one instruction: fetch opcode, decode, and execute.
    pub fn step(&mut self) {
        let opcode = self.fetch();

        // https://gbdev.io/gb-opcodes/optables/
        match opcode {
            0x00 => {}                                                  // NOP
            0x01 => {                                                   // LD BC, n16
                let val = self.fetch_u16();
                self.regs.set_bc(val);
            }
            0x02 => self.mem.write(self.regs.bc(), self.regs.a),        // LD [BC], A
            0x03 => self.regs.set_bc(self.regs.bc().wrapping_add(1)),   // INC BC
            0x04 => self.regs.b = self.inc(self.regs.b),                // INC B
            0x05 => self.regs.b = self.dec(self.regs.b),                // DEC B
            0x06 => self.regs.b = self.fetch(),                         // LD B, n8
            0x07 => {                                                   // RLCA
                self.regs.a = self.rlc(self.regs.a);
                self.regs.set_z(false);
            }
            0x08 => {                                                   // LD [a16], SP
                let addr = self.fetch_u16();
                self.mem.write_u16(addr, self.regs.sp);
            }
            0x09 => self.add_hl(self.regs.bc()),                        // ADD HL, BC
            _ => panic!("unimplemented opcode: {:#04X}", opcode),
        }
    }

    /// Fetch the byte at PC and advance PC.
    pub fn fetch(&mut self) -> u8 {
        let opcode = self.mem.read(self.regs.pc);
        self.regs.pc += 1;
        opcode
    }

    /// Fetch a little-endian 16-bit value (two bytes, low first).
    pub fn fetch_u16(&mut self) -> u16 {
        let lo = self.fetch();
        let hi = self.fetch();
        (hi as u16) << 8 | lo as u16
    }

    /// 8-bit increment. Sets Z, clears N, sets H on lower-nibble overflow.
    pub fn inc(&mut self, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h((val & 0x0F) == 0x0F);
        result
    }

    /// 8-bit decrement. Sets Z, sets N, sets H on lower-nibble borrow.
    pub fn dec(&mut self, val: u8) -> u8 {
        let result = val.wrapping_sub(1);
        self.regs.set_z(result == 0);
        self.regs.set_n(true);
        self.regs.set_h((val & 0x0F) == 0x00);
        result
    }

    /// Rotate left circular. Bit 7 wraps to bit 0 and into carry flag.
    /// Sets Z based on result (caller overrides for RLCA which always clears Z).
    pub fn rlc(&mut self, val: u8) -> u8 {
        let bit7 = val & 0x80;
        let result = (val << 1) | (bit7 >> 7);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(bit7 != 0);
        result
    }

    /// 16-bit add to HL. Clears N, sets H on bit-11 carry, sets C on bit-15 carry.
    pub fn add_hl(&mut self, val: u16) {
        let hl = self.regs.hl();
        let result = hl.wrapping_add(val);
        self.regs.set_n(false);
        self.regs.set_h((hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF);
        self.regs.set_c((hl as u32) + (val as u32) > 0xFFFF);
        self.regs.set_hl(result);
    }
}
