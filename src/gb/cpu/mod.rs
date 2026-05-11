use crate::gb::registers::Registers;
use crate::gb::ram::Ram;

mod cb;

/// SM83 CPU. Owns registers and memory, executes instructions via fetch-decode-execute.
pub struct Cpu {
    regs: Registers,
    mem: Ram,
    halted: bool,
    ime: bool,
}

impl Cpu {
    /// Create a new CPU with registers at CGB post-boot values and zeroed memory.
    pub fn new() -> Self {
        Cpu {
            regs: Registers::new(),
            mem: Ram::new(),
            halted: false,
            ime: false,
        }
    }

    /// Load ROM data into memory, delegating to the memory subsystem.
    pub fn load_rom(&mut self, data: &[u8]) {
        self.mem.load_rom(data);
    }

    /// Execute one instruction: fetch opcode, decode, and execute.
    #[allow(dead_code)]
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
            0x40 => self.regs.b = self.regs.b,                          // LD B, B
            0x41 => self.regs.b = self.regs.c,                          // LD B, C
            0x42 => self.regs.b = self.regs.d,                          // LD B, D
            0x43 => self.regs.b = self.regs.e,                          // LD B, E
            0x44 => self.regs.b = self.regs.h,                          // LD B, H
            0x45 => self.regs.b = self.regs.l,                          // LD B, L
            0x46 => self.regs.b = self.mem.read(self.regs.hl()),        // LD B, [HL]
            0x47 => self.regs.b = self.regs.a,                          // LD B, A
            0x48 => self.regs.c = self.regs.b,                          // LD C, B
            0x49 => self.regs.c = self.regs.c,                          // LD C, C
            0x4A => self.regs.c = self.regs.d,                          // LD C, D
            0x4B => self.regs.c = self.regs.e,                          // LD C, E
            0x4C => self.regs.c = self.regs.h,                          // LD C, H
            0x4D => self.regs.c = self.regs.l,                          // LD C, L
            0x4E => self.regs.c = self.mem.read(self.regs.hl()),        // LD C, [HL]
            0x4F => self.regs.c = self.regs.a,                          // LD C, A
            0x50 => self.regs.d = self.regs.b,                          // LD D, B
            0x51 => self.regs.d = self.regs.c,                          // LD D, C
            0x52 => self.regs.d = self.regs.d,                          // LD D, D
            0x53 => self.regs.d = self.regs.e,                          // LD D, E
            0x54 => self.regs.d = self.regs.h,                          // LD D, H
            0x55 => self.regs.d = self.regs.l,                          // LD D, L
            0x56 => self.regs.d = self.mem.read(self.regs.hl()),        // LD D, [HL]
            0x57 => self.regs.d = self.regs.a,                          // LD D, A
            0x58 => self.regs.e = self.regs.b,                          // LD E, B
            0x59 => self.regs.e = self.regs.c,                          // LD E, C
            0x5A => self.regs.e = self.regs.d,                          // LD E, D
            0x5B => self.regs.e = self.regs.e,                          // LD E, E
            0x5C => self.regs.e = self.regs.h,                          // LD E, H
            0x5D => self.regs.e = self.regs.l,                          // LD E, L
            0x5E => self.regs.e = self.mem.read(self.regs.hl()),        // LD E, [HL]
            0x5F => self.regs.e = self.regs.a,                          // LD E, A
            0x60 => self.regs.h = self.regs.b,                          // LD H, B
            0x61 => self.regs.h = self.regs.c,                          // LD H, C
            0x62 => self.regs.h = self.regs.d,                          // LD H, D
            0x63 => self.regs.h = self.regs.e,                          // LD H, E
            0x64 => self.regs.h = self.regs.h,                          // LD H, H
            0x65 => self.regs.h = self.regs.l,                          // LD H, L
            0x66 => self.regs.h = self.mem.read(self.regs.hl()),        // LD H, [HL]
            0x67 => self.regs.h = self.regs.a,                          // LD H, A
            0x68 => self.regs.l = self.regs.b,                          // LD L, B
            0x69 => self.regs.l = self.regs.c,                          // LD L, C
            0x6A => self.regs.l = self.regs.d,                          // LD L, D
            0x6B => self.regs.l = self.regs.e,                          // LD L, E
            0x6C => self.regs.l = self.regs.h,                          // LD L, H
            0x6D => self.regs.l = self.regs.l,                          // LD L, L
            0x6E => self.regs.l = self.mem.read(self.regs.hl()),        // LD L, [HL]
            0x6F => self.regs.l = self.regs.a,                          // LD L, A
            0x70 => self.mem.write(self.regs.hl(), self.regs.b),        // LD [HL], B
            0x71 => self.mem.write(self.regs.hl(), self.regs.c),        // LD [HL], C
            0x72 => self.mem.write(self.regs.hl(), self.regs.d),        // LD [HL], D
            0x73 => self.mem.write(self.regs.hl(), self.regs.e),        // LD [HL], E
            0x74 => self.mem.write(self.regs.hl(), self.regs.h),        // LD [HL], H
            0x75 => self.mem.write(self.regs.hl(), self.regs.l),        // LD [HL], L
            0x76 => self.halted = true,                                 // HALT
            0x77 => self.mem.write(self.regs.hl(), self.regs.a),        // LD [HL], A
            0x78 => self.regs.a = self.regs.b,                          // LD A, B
            0x79 => self.regs.a = self.regs.c,                          // LD A, C
            0x7A => self.regs.a = self.regs.d,                          // LD A, D
            0x7B => self.regs.a = self.regs.e,                          // LD A, E
            0x7C => self.regs.a = self.regs.h,                          // LD A, H
            0x7D => self.regs.a = self.regs.l,                          // LD A, L
            0x7E => self.regs.a = self.mem.read(self.regs.hl()),        // LD A, [HL]
            0x7F => self.regs.a = self.regs.a,                          // LD A, A
            0x80 => self.add(self.regs.b),                              // ADD A, B
            0x81 => self.add(self.regs.c),                              // ADD A, C
            0x82 => self.add(self.regs.d),                              // ADD A, D
            0x83 => self.add(self.regs.e),                              // ADD A, E
            0x84 => self.add(self.regs.h),                              // ADD A, H
            0x85 => self.add(self.regs.l),                              // ADD A, L
            0x86 => self.add(self.mem.read(self.regs.hl())),            // ADD A, [HL]
            0x87 => self.add(self.regs.a),                              // ADD A, A
            0x88 => self.adc(self.regs.b),                              // ADC A, B
            0x89 => self.adc(self.regs.c),                              // ADC A, C
            0x8A => self.adc(self.regs.d),                              // ADC A, D
            0x8B => self.adc(self.regs.e),                              // ADC A, E
            0x8C => self.adc(self.regs.h),                              // ADC A, H
            0x8D => self.adc(self.regs.l),                              // ADC A, L
            0x8E => self.adc(self.mem.read(self.regs.hl())),            // ADC A, [HL]
            0x8F => self.adc(self.regs.a),                              // ADC A, A
            0x90 => self.sub(self.regs.b),                              // SUB A, B
            0x91 => self.sub(self.regs.c),                              // SUB A, C
            0x92 => self.sub(self.regs.d),                              // SUB A, D
            0x93 => self.sub(self.regs.e),                              // SUB A, E
            0x94 => self.sub(self.regs.h),                              // SUB A, H
            0x95 => self.sub(self.regs.l),                              // SUB A, L
            0x96 => self.sub(self.mem.read(self.regs.hl())),            // SUB A, [HL]
            0x97 => self.sub(self.regs.a),                              // SUB A, A
            0x98 => self.sbc(self.regs.b),                              // SBC A, B
            0x99 => self.sbc(self.regs.c),                              // SBC A, C
            0x9A => self.sbc(self.regs.d),                              // SBC A, D
            0x9B => self.sbc(self.regs.e),                              // SBC A, E
            0x9C => self.sbc(self.regs.h),                              // SBC A, H
            0x9D => self.sbc(self.regs.l),                              // SBC A, L
            0x9E => self.sbc(self.mem.read(self.regs.hl())),            // SBC A, [HL]
            0x9F => self.sbc(self.regs.a),                              // SBC A, A
            0xA0 => self.and(self.regs.b),                              // AND A, B
            0xA1 => self.and(self.regs.c),                              // AND A, C
            0xA2 => self.and(self.regs.d),                              // AND A, D
            0xA3 => self.and(self.regs.e),                              // AND A, E
            0xA4 => self.and(self.regs.h),                              // AND A, H
            0xA5 => self.and(self.regs.l),                              // AND A, L
            0xA6 => self.and(self.mem.read(self.regs.hl())),            // AND A, [HL]
            0xA7 => self.and(self.regs.a),                              // AND A, A
            0xA8 => self.xor(self.regs.b),                              // XOR A, B
            0xA9 => self.xor(self.regs.c),                              // XOR A, C
            0xAA => self.xor(self.regs.d),                              // XOR A, D
            0xAB => self.xor(self.regs.e),                              // XOR A, E
            0xAC => self.xor(self.regs.h),                              // XOR A, H
            0xAD => self.xor(self.regs.l),                              // XOR A, L
            0xAE => self.xor(self.mem.read(self.regs.hl())),            // XOR A, [HL]
            0xAF => self.xor(self.regs.a),                              // XOR A, A
            0xB0 => self.or(self.regs.b),                               // OR A, B
            0xB1 => self.or(self.regs.c),                               // OR A, C
            0xB2 => self.or(self.regs.d),                               // OR A, D
            0xB3 => self.or(self.regs.e),                               // OR A, E
            0xB4 => self.or(self.regs.h),                               // OR A, H
            0xB5 => self.or(self.regs.l),                               // OR A, L
            0xB6 => self.or(self.mem.read(self.regs.hl())),             // OR A, [HL]
            0xB7 => self.or(self.regs.a),                               // OR A, A
            0xB8 => self.cp(self.regs.b),                               // CP A, B
            0xB9 => self.cp(self.regs.c),                               // CP A, C
            0xBA => self.cp(self.regs.d),                               // CP A, D
            0xBB => self.cp(self.regs.e),                               // CP A, E
            0xBC => self.cp(self.regs.h),                               // CP A, H
            0xBD => self.cp(self.regs.l),                               // CP A, L
            0xBE => self.cp(self.mem.read(self.regs.hl())),             // CP A, [HL]
            0xBF => self.cp(self.regs.a),                               // CP A, A
            0xC0 => self.ret(!self.regs.get_z()),                       // RET NZ
            0xC1 => { let bc = self.pop(); self.regs.set_bc(bc); }      // POP BC
            0xC2 => self.jp(!self.regs.get_z()),                        // JP NZ, a16
            0xC3 => self.jp(true),                                      // JP a16
            0xC4 => self.call(!self.regs.get_z()),                      // CALL NZ, a16
            0xC5 => self.push(self.regs.bc()),                          // PUSH BC
            0xC6 => { let n8 = self.fetch(); self.add(n8); }            // ADD A, n8
            0xC7 => self.rst(0x00),                                     // RST $00
            0xC8 => self.ret(self.regs.get_z()),                        // RET Z
            0xC9 => self.ret(true),                                     // RET
            0xCA => self.jp(self.regs.get_z()),                         // JP Z, a16
            0xCB => self.step_cb(),                                     // PREFIX
            0xCC => self.call(self.regs.get_z()),                       // CALL Z, a16
            0xCD => self.call(true),                                    // CALL a16
            0xCE => { let n8 = self.fetch(); self.adc(n8); }            // ADC A, n8
            0xCF => self.rst(0x08),                                     // RST $08
            0xD0 => self.ret(!self.regs.get_c()),                       // RET NC
            0xD1 => { let de = self.pop(); self.regs.set_de(de); }      // POP DE
            0xD2 => self.jp(!self.regs.get_c()),                        // JP NC, a16
            0xD3 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xD4 => self.call(!self.regs.get_c()),                      // CALL NC, a16
            0xD5 => self.push(self.regs.de()),                          // PUSH DE
            0xD6 => { let n8 = self.fetch(); self.sub(n8); }            // SUB A, n8
            0xD7 => self.rst(0x10),                                     // RST $10
            0xD8 => self.ret(self.regs.get_c()),                        // RET C
            0xD9 => self.reti(),                                        // RETI
            0xDA => self.jp(self.regs.get_c()),                         // JP C, a16
            0xDB => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xDC => self.call(self.regs.get_c()),                       // CALL C, a16
            0xDD => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xDE => { let n8 = self.fetch(); self.sbc(n8); }            // SBC A, n8
            0xDF => self.rst(0x18),                                     // RST $18
            0xE0 => {                                                   // LDH [a8], A
                let a8 = self.fetch();
                self.mem.write(0xFF00 + a8 as u16, self.regs.a);
            }
            0xE1 => { let hl = self.pop(); self.regs.set_hl(hl); }      // POP HL
            0xE2 => {                                                   // LDH [C], A
                let addr = 0xFF00 + self.regs.c as u16;
                self.mem.write(addr, self.regs.a);
            }
            0xE3 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xE4 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xE5 => self.push(self.regs.hl()),                          // PUSH HL
            0xE6 => { let n8 = self.fetch(); self.and(n8); }            // AND A, n8
            0xE7 => self.rst(0x20),                                     // RST $20
            0xE8 => self.add_sp_e8(),                                   // ADD SP, e8
            0xE9 => self.jp_hl(),                                       // JP HL
            0xEA => {                                                   // LD [a16], A
                let a16 = self.fetch_u16();
                self.mem.write(a16, self.regs.a);
            }
            0xEB => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xEC => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xED => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xEE => { let n8 = self.fetch(); self.xor(n8); }            // XOR A, n8
            0xEF => self.rst(0x28),                                     // RST $28
            0xF0 => {                                                   // LDH A, [a8]
                let a8 = self.fetch();
                self.regs.a = self.mem.read(0xFF00 + a8 as u16);
            }
            0xF1 => { let af = self.pop(); self.regs.set_af(af); }      // POP AF
            0xF2 => {                                                   // LDH A, [C]
                let addr = 0xFF00 + self.regs.c as u16;
                self.regs.a = self.mem.read(addr);
            }
            0xF3 => self.di(),                                          // DI
            0xF4 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xF5 => self.push(self.regs.af()),                          // PUSH AF
            0xF6 => { let n8 = self.fetch(); self.or(n8); }             // OR A, n8
            0xF7 => self.rst(0x30),                                     // RST $30
            0xF8 => self.ld_hl_sp_e8(),                                 // LD HL, SP + e8
            0xF9 => self.regs.sp = self.regs.hl(),                      // LD SP, HL
            0xFA => {                                                   // LD A, [a16]
                let a16 = self.fetch_u16();
                self.regs.a = self.mem.read(a16);
            }
            0xFB => self.ei(),                                          // EI
            0xFC => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xFD => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xFE => { let n8 = self.fetch(); self.cp(n8); }             // CP A, n8
            0xFF => self.rst(0x38),                                     // RST $38
        }
    }

    // For instructions see https://rgbds.gbdev.io/docs/v0.9.1/gbz80.7

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

    /// 8-bit increment.
    pub fn inc(&mut self, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h((val & 0x0F) == 0x0F);
        result
    }

    /// 8-bit decrement.
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

    /// 8-bit add to A.
    pub fn add(&mut self, val: u8) {
        let a = self.regs.a;
        let result = a.wrapping_add(val);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h((a & 0x0F) + (val & 0x0F) > 0x0F);
        self.regs.set_c((a as u16) + (val as u16) > 0xFF);
        self.regs.a = result;
    }

    /// 16-bit add to HL.
    pub fn add_hl(&mut self, val: u16) {
        let hl = self.regs.hl();
        let result = hl.wrapping_add(val);
        self.regs.set_n(false);
        self.regs.set_h((hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF);
        self.regs.set_c((hl as u32) + (val as u32) > 0xFFFF);
        self.regs.set_hl(result);
    }

    /// ADD SP, e8. Adds signed 8-bit offset to SP.
    pub fn add_sp_e8(&mut self) {
        let sp = self.regs.sp;
        let val = self.fetch() as i8;
        let result = sp.wrapping_add(val as u16);
        self.regs.set_z(false);
        self.regs.set_n(false);
        self.regs.set_h((sp as u8) & 0x0F + (val as u8) & 0x0F > 0x0F);
        self.regs.set_c((sp as u8) as u16 + (val as u8) as u16 > 0xFF);
        self.regs.sp = result;
    }

    /// 8-bit add with carry to A.
    pub fn adc(&mut self, val: u8) {
        let a = self.regs.a;
        let c = if self.regs.get_c() { 1u8 } else { 0u8 };
        let result = a.wrapping_add(val).wrapping_add(c);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h((a & 0x0F) + (val & 0x0F) + c > 0x0F);
        self.regs.set_c((a as u16) + (val as u16) + (c as u16) > 0xFF);
        self.regs.a = result;
    }

    /// 8-bit subtract from A.
    pub fn sub(&mut self, val: u8) {
        let a = self.regs.a;
        let result = a.wrapping_sub(val);
        self.regs.set_z(result == 0);
        self.regs.set_n(true);
        self.regs.set_h((val & 0x0F) > (a & 0x0F));
        self.regs.set_c(val > a);
        self.regs.a = result;
    }

    /// 8-bit subtract with carry from A.
    pub fn sbc(&mut self, val: u8) {
        let a = self.regs.a;
        let c = if self.regs.get_c() { 1u8 } else { 0u8 };
        let result = a.wrapping_sub(val).wrapping_sub(c);
        self.regs.set_z(result == 0);
        self.regs.set_n(true);
        self.regs.set_h((val & 0x0F) + c > (a & 0x0F));
        self.regs.set_c((val as u16) + (c as u16) > (a as u16));
        self.regs.a = result;
    }

    /// Bitwise AND with A.
    pub fn and(&mut self, val: u8) {
        let result = self.regs.a & val;
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(true);
        self.regs.set_c(false);
        self.regs.a = result;
    }

    /// Bitwise XOR with A.
    pub fn xor(&mut self, val: u8) {
        let result = self.regs.a ^ val;
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(false);
        self.regs.a = result;
    }

    /// Bitwise OR with A.
    pub fn or(&mut self, val: u8) {
        let result = self.regs.a | val;
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(false);
        self.regs.a = result;
    }

    /// Compare A with val
    pub fn cp(&mut self, val: u8) {
        let a = self.regs.a;
        let result = a.wrapping_sub(val);
        self.regs.set_z(result == 0);
        self.regs.set_n(true);
        self.regs.set_h((val & 0x0F) > (a & 0x0F));
        self.regs.set_c(val > a);
    }

    /// Push a 16-bit value onto the stack (high byte first).
    pub fn push(&mut self, val: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.mem.write(self.regs.sp, (val >> 8) as u8);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.mem.write(self.regs.sp, val as u8);
    }

    /// Pop a 16-bit value from the stack (low byte first).
    pub fn pop(&mut self) -> u16 {
        let lo = self.mem.read(self.regs.sp) as u16;
        self.regs.sp = self.regs.sp.wrapping_add(1);
        let hi = (self.mem.read(self.regs.sp) as u16) << 8;
        self.regs.sp = self.regs.sp.wrapping_add(1);
        hi | lo
    }

    /// Call: push PC and jump to fetched address if condition is met.
    pub fn call(&mut self, condition: bool) {
        let addr = self.fetch_u16();
        if condition {
            self.push(self.regs.pc);
            self.regs.pc = addr;
        }
    }

    /// Return: pop PC from stack if condition is met.
    pub fn ret(&mut self, condition: bool) {
        if !condition { return; }
        self.regs.pc = self.pop();
    }

    /// Return from interrupt: pop PC and enable IME.
    pub fn reti(&mut self) {
        self.ei();
        self.ret(true);
    }

    /// Restart: push PC and jump to fixed address.
    pub fn rst(&mut self, addr: u16) {
        self.push(self.regs.pc);
        self.regs.pc = addr;
    }

    /// Absolute jump to fetched address if condition is met.
    pub fn jp(&mut self, condition: bool) {
        let addr = self.fetch_u16();
        if condition {
            self.regs.pc = addr;
        }
    }

    /// Jump to address in HL.
    pub fn jp_hl(&mut self) {
        self.regs.pc = self.regs.hl();
    }

    /// LD HL, SP+e8. Loads SP + signed 8-bit offset into HL.
    pub fn ld_hl_sp_e8(&mut self) {
        let sp = self.regs.sp;
        let e8 = self.fetch() as i8;
        let result = sp.wrapping_add(e8 as u16);
        self.regs.set_z(false);
        self.regs.set_n(false);
        self.regs.set_h((sp as u8) & 0x0F + (e8 as u8) & 0x0F > 0x0F);
        self.regs.set_c((sp as u8) as u16 + (e8 as u8) as u16 > 0xFF);
        self.regs.set_hl(result);
    }

    /// Enable interrupts (set IME).
    pub fn ei(&mut self) {
        self.ime = true;
    }

    /// Disable interrupts (clear IME).
    pub fn di(&mut self) {
        self.ime = false;
    }
}
