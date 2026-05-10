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
            0x0A => self.regs.a = self.mem.read(self.regs.bc()),        // LD A, [BC]
            0x0B => self.regs.set_bc(self.regs.bc().wrapping_sub(1)),   // DEC BC
            0x0C => self.regs.c = self.inc(self.regs.c),                // INC C
            0x0D => self.regs.c = self.dec(self.regs.c),                // DEC C
            0x0E => self.regs.c = self.fetch(),                         // LD C, n8
            0x0F => {                                                   // RRCA
                self.regs.a = self.rrc(self.regs.a);
                self.regs.set_z(false);
            }
            0x10 => { self.fetch(); }                                   // STOP n8, needs to fixed later
            0x11 => {                                                   // LD DE, n16
                let val = self.fetch_u16();
                self.regs.set_de(val);
            }
            0x12 => self.mem.write(self.regs.de(), self.regs.a),        // LD [DE], A
            0x13 => self.regs.set_de(self.regs.de().wrapping_add(1)),   // INC DE
            0x14 => self.regs.d = self.inc(self.regs.d),                // INC D
            0x15 => self.regs.d = self.dec(self.regs.d),                // DEC D
            0x16 => self.regs.d = self.fetch(),                         // LD D, n8
            0x17 => {                                                   // RLA
                self.regs.a = self.rl(self.regs.a);
                self.regs.set_z(false);
            }
            0x18 => self.jr(true),                                      // JR e8
            0x19 => self.add_hl(self.regs.de()),                        // ADD HL, DE
            0x1A => self.regs.a = self.mem.read(self.regs.de()),        // LD A, [DE]
            0x1B => self.regs.set_de(self.regs.de().wrapping_sub(1)),   // DEC DE
            0x1C => self.regs.e = self.inc(self.regs.e),                // INC E
            0x1D => self.regs.e = self.dec(self.regs.e),                // DEC E
            0x1E => self.regs.e = self.fetch(),                         // LD E, n8
            0x1F => {                                                   // RRA
                self.regs.a = self.rr(self.regs.a);
                self.regs.set_z(false);
            }
            0x20 => self.jr(!self.regs.get_z()),                        // JR NZ, e8
            0x21 => {                                                   // LD HL, n16
                let val = self.fetch_u16();
                self.regs.set_hl(val);
            }
            0x22 => {                                                   // LD [HL+], A
                self.mem.write(self.regs.hl(), self.regs.a);
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
            }
            0x23 => self.regs.set_hl(self.regs.hl().wrapping_add(1)),   // INC HL
            0x24 => self.regs.h = self.inc(self.regs.h),                // INC H
            0x25 => self.regs.h = self.dec(self.regs.h),                // DEC H
            0x26 => self.regs.h = self.fetch(),                         // LD H, n8
            0x27 => self.daa(),                                         // DAA
            0x28 => self.jr(self.regs.get_z()),                         // JR Z, e8
            0x29 => self.add_hl(self.regs.hl()),                        // ADD HL, HL
            0x2A => {                                                   // LD A, [HL+]
                self.regs.a = self.mem.read(self.regs.hl());
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
            }
            0x2B => self.regs.set_hl(self.regs.hl().wrapping_sub(1)),   // DEC HL
            0x2C => self.regs.l = self.inc(self.regs.l),                // INC L
            0x2D => self.regs.l = self.dec(self.regs.l),                // DEC L
            0x2E => self.regs.l = self.fetch(),                         // LD L, n8
            0x2F => self.cpl(),                                         // CPL
            0x30 => self.jr(!self.regs.get_c()),                        // JR NC, e8
            0x31 => {                                                   // LD SP, n16
                let val = self.fetch_u16();
                self.regs.sp = val;
            }
            0x32 => {                                                   // LD [HL-], A
                self.mem.write(self.regs.hl(), self.regs.a);
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
            }
            0x33 => self.regs.sp = self.regs.sp.wrapping_add(1),        // INC SP
            0x34 => {                                                   // INC [HL]
                let val = self.mem.read(self.regs.hl());
                let result = self.inc(val);
                self.mem.write(self.regs.hl(), result);
            }
            0x35 => {                                                   // DEC [HL]
                let val = self.mem.read(self.regs.hl());
                let result = self.dec(val);
                self.mem.write(self.regs.hl(), result);
            }
            0x36 => {                                                   // LD [HL], n8
                let n8 = self.fetch();
                self.mem.write(self.regs.hl(), n8);
            }
            0x37 => self.scf(),                                         // SCF
            0x38 => self.jr(self.regs.get_c()),                         // JR C, e8
            0x39 => self.add_hl(self.regs.sp),                          // ADD HL, SP
            0x3A => {                                                   // LD A, [HL-]
                self.regs.a = self.mem.read(self.regs.hl());
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
            }
            0x3B => self.regs.sp = self.regs.sp.wrapping_sub(1),        // DEC SP
            0x3C => self.regs.a = self.inc(self.regs.a),                // INC A
            0x3D => self.regs.a = self.dec(self.regs.a),                // DEC A
            0x3E => self.regs.a = self.fetch(),                         // LD A, n8
            0x3F => self.ccf(),                                         // CCF
            _ => panic!("unimplemented opcode: {:#04X}", opcode),
        }
    }

    /// Fetch the byte at PC and advance PC.
    pub fn fetch(&mut self) -> u8 {
        let opcode = self.mem.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
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

    /// Relative jump. Fetches signed offset and adds to PC if condition is true.
    pub fn jr(&mut self, condition: bool) {
        let offset = self.fetch() as i8;
        if condition {
            self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
        }
    }

    /// Rotate left circular. Bit 7 wraps to bit 0 and into carry flag.
    /// Sets Z based on result
    pub fn rlc(&mut self, val: u8) -> u8 {
        let bit7 = val & 0x80;
        let result = (val << 1) | (bit7 >> 7);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(bit7 != 0);
        result
    }

    /// Rotate right circular. Bit 0 wraps to bit 7 and into carry flag.
    /// Sets Z based on result
    pub fn rrc(&mut self, val: u8) -> u8 {
        let bit0 = val & 0x01;
        let result = (val >> 1) | (bit0 << 7);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(bit0 != 0);
        result
    }

    /// Rotate left through the carry bit.
    pub fn rl(&mut self, val: u8) -> u8 {
        let old_carry = if self.regs.get_c() { 1u8 } else { 0u8 };
        let result = (val << 1) | old_carry;
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c((val & 0x80) != 0);
        result
    }

    /// Rotate right through the carry bit.
    pub fn rr(&mut self, val: u8) -> u8 {
        let old_carry = if self.regs.get_c() { 1u8 } else { 0u8 };
        let result = (val >> 1) | (old_carry << 7);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c((val & 0x01) != 0);
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

    /// Decimal Adjust Accumulator.
    pub fn daa(&mut self) {
        let mut adjust = 0u8;
        let mut carry = false;

        if self.regs.get_n() {
            if self.regs.get_h() { adjust |= 0x06; }
            if self.regs.get_c() { adjust |= 0x60; carry = true; }
            self.regs.a = self.regs.a.wrapping_sub(adjust);
        } else {
            if self.regs.get_h() || (self.regs.a & 0x0F) > 0x09 { adjust |= 0x06; }
            if self.regs.get_c() || self.regs.a > 0x99 { adjust |= 0x60; carry = true; }
            self.regs.a = self.regs.a.wrapping_add(adjust);
        }

        self.regs.set_z(self.regs.a == 0);
        self.regs.set_c(carry);
    }

    /// ComPLement accumulator (A = ~A).
    pub fn cpl(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.set_n(true);
        self.regs.set_h(true);

    }

    /// Set Carry Flag.
    pub fn scf(&mut self) {
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(true);
    }

    /// Complement Carry Flag.
    pub fn ccf(&mut self) {
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(!self.regs.get_c());
    }
}
