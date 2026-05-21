use crate::gb::registers::Registers;
use crate::gb::bus::{Bus, Interrupt};

mod cb;

/// SM83 CPU. Owns registers and memory, executes instructions via fetch-decode-execute.
pub struct Cpu {
    regs: Registers,
    halted: bool,
    ime: bool,
}

impl Cpu {
    /// Base cycle counts (not-taken for conditional branches).
    const CYCLES: [u8; 256] = [
    //  0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        4, 12,  8,  8,  4,  4,  8,  4, 20,  8,  8,  8,  4,  4,  8,  4, // 0x
        4, 12,  8,  8,  4,  4,  8,  4, 12,  8,  8,  8,  4,  4,  8,  4, // 1x
        8, 12,  8,  8,  4,  4,  8,  4,  8,  8,  8,  8,  4,  4,  8,  4, // 2x
        8, 12,  8,  8, 12, 12, 12,  4,  8,  8,  8,  8,  4,  4,  8,  4, // 3x
        4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 4x
        4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 5x
        4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 6x
        8,  8,  8,  8,  8,  8,  4,  8,  4,  4,  4,  4,  4,  4,  8,  4, // 7x
        4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 8x
        4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // 9x
        4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // Ax
        4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4, // Bx
        8, 12, 12, 16, 12, 16,  8, 16,  8, 16, 12,  4, 12, 24,  8, 16, // Cx
        8, 12, 12,  0, 12, 16,  8, 16,  8, 16, 12,  0, 12,  0,  8, 16, // Dx
       12, 12,  8,  0,  0, 16,  8, 16, 16,  4, 16,  0,  0,  0,  8, 16, // Ex
       12, 12,  8,  4,  0, 16,  8, 16, 12,  8, 16,  4,  0,  0,  8, 16, // Fx
    ];
    const CYCLES_BRANCHED: [u8; 256] = [
    //  0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 0x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 1x
       12,  0,  0,  0,  0,  0,  0,  0, 12,  0,  0,  0,  0,  0,  0,  0, // 2x
       12,  0,  0,  0,  0,  0,  0,  0, 12,  0,  0,  0,  0,  0,  0,  0, // 3x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 5x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 7x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9x
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // Ax
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // Bx
       20,  0, 16,  0, 24,  0,  0,  0, 20,  0, 16,  0, 24,  0,  0,  0, // Cx
       20,  0, 16,  0, 24,  0,  0,  0, 20,  0, 16,  0, 24,  0,  0,  0, // Dx
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // Ex
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // Fx
    ];

    /// Create a new CPU with registers at CGB post-boot values and zeroed memory.
    pub fn new() -> Self {
        Cpu {
            regs: Registers::new(),
            halted: false,
            ime: false,
        }
    }

    /// Load ROM data into memory, delegating to the memory subsystem.
    pub fn load_rom(&mut self, data: &[u8], bus: &mut Bus) {
        bus.load_rom(data);
    }

    /// Execute one instruction: fetch opcode, decode, and execute.
    #[allow(dead_code)]
    pub fn step(&mut self, bus: &mut Bus) -> u8 {
        if self.is_halt(bus) { return 4; }
        if self.is_interrupt(bus) { return 20; }

        let opcode = self.fetch(bus);
        let mut branched = false;

        // https://gbdev.io/gb-opcodes/optables/
        match opcode {
            0x00 => {}                                                  // NOP
            0x01 => {                                                   // LD BC, n16
                let val = self.fetch_u16(bus);
                self.regs.set_bc(val);
            }
            0x02 => bus.write(self.regs.bc(), self.regs.a),        // LD [BC], A
            0x03 => self.regs.set_bc(self.regs.bc().wrapping_add(1)),   // INC BC
            0x04 => self.regs.b = self.inc(self.regs.b),                // INC B
            0x05 => self.regs.b = self.dec(self.regs.b),                // DEC B
            0x06 => self.regs.b = self.fetch(bus),                         // LD B, n8
            0x07 => {                                                   // RLCA
                self.regs.a = self.rlc(self.regs.a);
                self.regs.set_z(false);
            }
            0x08 => {                                                   // LD [a16], SP
                let addr = self.fetch_u16(bus);
                bus.write_u16(addr, self.regs.sp);
            }
            0x09 => self.add_hl(self.regs.bc()),                        // ADD HL, BC
            0x0A => self.regs.a = bus.read(self.regs.bc()),        // LD A, [BC]
            0x0B => self.regs.set_bc(self.regs.bc().wrapping_sub(1)),   // DEC BC
            0x0C => self.regs.c = self.inc(self.regs.c),                // INC C
            0x0D => self.regs.c = self.dec(self.regs.c),                // DEC C
            0x0E => self.regs.c = self.fetch(bus),                         // LD C, n8
            0x0F => {                                                   // RRCA
                self.regs.a = self.rrc(self.regs.a);
                self.regs.set_z(false);
            }
            0x10 => { self.fetch(bus); }                                   // STOP n8, needs to fixed later
            0x11 => {                                                   // LD DE, n16
                let val = self.fetch_u16(bus);
                self.regs.set_de(val);
            }
            0x12 => bus.write(self.regs.de(), self.regs.a),        // LD [DE], A
            0x13 => self.regs.set_de(self.regs.de().wrapping_add(1)),   // INC DE
            0x14 => self.regs.d = self.inc(self.regs.d),                // INC D
            0x15 => self.regs.d = self.dec(self.regs.d),                // DEC D
            0x16 => self.regs.d = self.fetch(bus),                         // LD D, n8
            0x17 => {                                                   // RLA
                self.regs.a = self.rl(self.regs.a);
                self.regs.set_z(false);
            }
            0x18 => { self.jr(true, bus); }                                      // JR e8
            0x19 => self.add_hl(self.regs.de()),                        // ADD HL, DE
            0x1A => self.regs.a = bus.read(self.regs.de()),        // LD A, [DE]
            0x1B => self.regs.set_de(self.regs.de().wrapping_sub(1)),   // DEC DE
            0x1C => self.regs.e = self.inc(self.regs.e),                // INC E
            0x1D => self.regs.e = self.dec(self.regs.e),                // DEC E
            0x1E => self.regs.e = self.fetch(bus),                         // LD E, n8
            0x1F => {                                                   // RRA
                self.regs.a = self.rr(self.regs.a);
                self.regs.set_z(false);
            }
            0x20 => branched = self.jr(!self.regs.get_z(), bus),                        // JR NZ, e8
            0x21 => {                                                   // LD HL, n16
                let val = self.fetch_u16(bus);
                self.regs.set_hl(val);
            }
            0x22 => {                                                   // LD [HL+], A
                bus.write(self.regs.hl(), self.regs.a);
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
            }
            0x23 => self.regs.set_hl(self.regs.hl().wrapping_add(1)),   // INC HL
            0x24 => self.regs.h = self.inc(self.regs.h),                // INC H
            0x25 => self.regs.h = self.dec(self.regs.h),                // DEC H
            0x26 => self.regs.h = self.fetch(bus),                         // LD H, n8
            0x27 => self.daa(),                                         // DAA
            0x28 => branched = self.jr(self.regs.get_z(), bus),                         // JR Z, e8
            0x29 => self.add_hl(self.regs.hl()),                        // ADD HL, HL
            0x2A => {                                                   // LD A, [HL+]
                self.regs.a = bus.read(self.regs.hl());
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
            }
            0x2B => self.regs.set_hl(self.regs.hl().wrapping_sub(1)),   // DEC HL
            0x2C => self.regs.l = self.inc(self.regs.l),                // INC L
            0x2D => self.regs.l = self.dec(self.regs.l),                // DEC L
            0x2E => self.regs.l = self.fetch(bus),                         // LD L, n8
            0x2F => self.cpl(),                                         // CPL
            0x30 => branched = self.jr(!self.regs.get_c(), bus),                        // JR NC, e8
            0x31 => {                                                   // LD SP, n16
                let val = self.fetch_u16(bus);
                self.regs.sp = val;
            }
            0x32 => {                                                   // LD [HL-], A
                bus.write(self.regs.hl(), self.regs.a);
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
            }
            0x33 => self.regs.sp = self.regs.sp.wrapping_add(1),        // INC SP
            0x34 => {                                                   // INC [HL]
                let val = bus.read(self.regs.hl());
                let result = self.inc(val);
                bus.write(self.regs.hl(), result);
            }
            0x35 => {                                                   // DEC [HL]
                let val = bus.read(self.regs.hl());
                let result = self.dec(val);
                bus.write(self.regs.hl(), result);
            }
            0x36 => {                                                   // LD [HL], n8
                let n8 = self.fetch(bus);
                bus.write(self.regs.hl(), n8);
            }
            0x37 => self.scf(),                                         // SCF
            0x38 => branched = self.jr(self.regs.get_c(), bus),                         // JR C, e8
            0x39 => self.add_hl(self.regs.sp),                          // ADD HL, SP
            0x3A => {                                                   // LD A, [HL-]
                self.regs.a = bus.read(self.regs.hl());
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
            }
            0x3B => self.regs.sp = self.regs.sp.wrapping_sub(1),        // DEC SP
            0x3C => self.regs.a = self.inc(self.regs.a),                // INC A
            0x3D => self.regs.a = self.dec(self.regs.a),                // DEC A
            0x3E => self.regs.a = self.fetch(bus),                         // LD A, n8
            0x3F => self.ccf(),                                         // CCF
            0x40 => self.regs.b = self.regs.b,                          // LD B, B
            0x41 => self.regs.b = self.regs.c,                          // LD B, C
            0x42 => self.regs.b = self.regs.d,                          // LD B, D
            0x43 => self.regs.b = self.regs.e,                          // LD B, E
            0x44 => self.regs.b = self.regs.h,                          // LD B, H
            0x45 => self.regs.b = self.regs.l,                          // LD B, L
            0x46 => self.regs.b = bus.read(self.regs.hl()),        // LD B, [HL]
            0x47 => self.regs.b = self.regs.a,                          // LD B, A
            0x48 => self.regs.c = self.regs.b,                          // LD C, B
            0x49 => self.regs.c = self.regs.c,                          // LD C, C
            0x4A => self.regs.c = self.regs.d,                          // LD C, D
            0x4B => self.regs.c = self.regs.e,                          // LD C, E
            0x4C => self.regs.c = self.regs.h,                          // LD C, H
            0x4D => self.regs.c = self.regs.l,                          // LD C, L
            0x4E => self.regs.c = bus.read(self.regs.hl()),        // LD C, [HL]
            0x4F => self.regs.c = self.regs.a,                          // LD C, A
            0x50 => self.regs.d = self.regs.b,                          // LD D, B
            0x51 => self.regs.d = self.regs.c,                          // LD D, C
            0x52 => self.regs.d = self.regs.d,                          // LD D, D
            0x53 => self.regs.d = self.regs.e,                          // LD D, E
            0x54 => self.regs.d = self.regs.h,                          // LD D, H
            0x55 => self.regs.d = self.regs.l,                          // LD D, L
            0x56 => self.regs.d = bus.read(self.regs.hl()),        // LD D, [HL]
            0x57 => self.regs.d = self.regs.a,                          // LD D, A
            0x58 => self.regs.e = self.regs.b,                          // LD E, B
            0x59 => self.regs.e = self.regs.c,                          // LD E, C
            0x5A => self.regs.e = self.regs.d,                          // LD E, D
            0x5B => self.regs.e = self.regs.e,                          // LD E, E
            0x5C => self.regs.e = self.regs.h,                          // LD E, H
            0x5D => self.regs.e = self.regs.l,                          // LD E, L
            0x5E => self.regs.e = bus.read(self.regs.hl()),        // LD E, [HL]
            0x5F => self.regs.e = self.regs.a,                          // LD E, A
            0x60 => self.regs.h = self.regs.b,                          // LD H, B
            0x61 => self.regs.h = self.regs.c,                          // LD H, C
            0x62 => self.regs.h = self.regs.d,                          // LD H, D
            0x63 => self.regs.h = self.regs.e,                          // LD H, E
            0x64 => self.regs.h = self.regs.h,                          // LD H, H
            0x65 => self.regs.h = self.regs.l,                          // LD H, L
            0x66 => self.regs.h = bus.read(self.regs.hl()),        // LD H, [HL]
            0x67 => self.regs.h = self.regs.a,                          // LD H, A
            0x68 => self.regs.l = self.regs.b,                          // LD L, B
            0x69 => self.regs.l = self.regs.c,                          // LD L, C
            0x6A => self.regs.l = self.regs.d,                          // LD L, D
            0x6B => self.regs.l = self.regs.e,                          // LD L, E
            0x6C => self.regs.l = self.regs.h,                          // LD L, H
            0x6D => self.regs.l = self.regs.l,                          // LD L, L
            0x6E => self.regs.l = bus.read(self.regs.hl()),        // LD L, [HL]
            0x6F => self.regs.l = self.regs.a,                          // LD L, A
            0x70 => bus.write(self.regs.hl(), self.regs.b),        // LD [HL], B
            0x71 => bus.write(self.regs.hl(), self.regs.c),        // LD [HL], C
            0x72 => bus.write(self.regs.hl(), self.regs.d),        // LD [HL], D
            0x73 => bus.write(self.regs.hl(), self.regs.e),        // LD [HL], E
            0x74 => bus.write(self.regs.hl(), self.regs.h),        // LD [HL], H
            0x75 => bus.write(self.regs.hl(), self.regs.l),        // LD [HL], L
            0x76 => self.halted = true,                                 // HALT
            0x77 => bus.write(self.regs.hl(), self.regs.a),        // LD [HL], A
            0x78 => self.regs.a = self.regs.b,                          // LD A, B
            0x79 => self.regs.a = self.regs.c,                          // LD A, C
            0x7A => self.regs.a = self.regs.d,                          // LD A, D
            0x7B => self.regs.a = self.regs.e,                          // LD A, E
            0x7C => self.regs.a = self.regs.h,                          // LD A, H
            0x7D => self.regs.a = self.regs.l,                          // LD A, L
            0x7E => self.regs.a = bus.read(self.regs.hl()),        // LD A, [HL]
            0x7F => self.regs.a = self.regs.a,                          // LD A, A
            0x80 => self.add(self.regs.b),                              // ADD A, B
            0x81 => self.add(self.regs.c),                              // ADD A, C
            0x82 => self.add(self.regs.d),                              // ADD A, D
            0x83 => self.add(self.regs.e),                              // ADD A, E
            0x84 => self.add(self.regs.h),                              // ADD A, H
            0x85 => self.add(self.regs.l),                              // ADD A, L
            0x86 => self.add(bus.read(self.regs.hl())),            // ADD A, [HL]
            0x87 => self.add(self.regs.a),                              // ADD A, A
            0x88 => self.adc(self.regs.b),                              // ADC A, B
            0x89 => self.adc(self.regs.c),                              // ADC A, C
            0x8A => self.adc(self.regs.d),                              // ADC A, D
            0x8B => self.adc(self.regs.e),                              // ADC A, E
            0x8C => self.adc(self.regs.h),                              // ADC A, H
            0x8D => self.adc(self.regs.l),                              // ADC A, L
            0x8E => self.adc(bus.read(self.regs.hl())),            // ADC A, [HL]
            0x8F => self.adc(self.regs.a),                              // ADC A, A
            0x90 => self.sub(self.regs.b),                              // SUB A, B
            0x91 => self.sub(self.regs.c),                              // SUB A, C
            0x92 => self.sub(self.regs.d),                              // SUB A, D
            0x93 => self.sub(self.regs.e),                              // SUB A, E
            0x94 => self.sub(self.regs.h),                              // SUB A, H
            0x95 => self.sub(self.regs.l),                              // SUB A, L
            0x96 => self.sub(bus.read(self.regs.hl())),            // SUB A, [HL]
            0x97 => self.sub(self.regs.a),                              // SUB A, A
            0x98 => self.sbc(self.regs.b),                              // SBC A, B
            0x99 => self.sbc(self.regs.c),                              // SBC A, C
            0x9A => self.sbc(self.regs.d),                              // SBC A, D
            0x9B => self.sbc(self.regs.e),                              // SBC A, E
            0x9C => self.sbc(self.regs.h),                              // SBC A, H
            0x9D => self.sbc(self.regs.l),                              // SBC A, L
            0x9E => self.sbc(bus.read(self.regs.hl())),            // SBC A, [HL]
            0x9F => self.sbc(self.regs.a),                              // SBC A, A
            0xA0 => self.and(self.regs.b),                              // AND A, B
            0xA1 => self.and(self.regs.c),                              // AND A, C
            0xA2 => self.and(self.regs.d),                              // AND A, D
            0xA3 => self.and(self.regs.e),                              // AND A, E
            0xA4 => self.and(self.regs.h),                              // AND A, H
            0xA5 => self.and(self.regs.l),                              // AND A, L
            0xA6 => self.and(bus.read(self.regs.hl())),            // AND A, [HL]
            0xA7 => self.and(self.regs.a),                              // AND A, A
            0xA8 => self.xor(self.regs.b),                              // XOR A, B
            0xA9 => self.xor(self.regs.c),                              // XOR A, C
            0xAA => self.xor(self.regs.d),                              // XOR A, D
            0xAB => self.xor(self.regs.e),                              // XOR A, E
            0xAC => self.xor(self.regs.h),                              // XOR A, H
            0xAD => self.xor(self.regs.l),                              // XOR A, L
            0xAE => self.xor(bus.read(self.regs.hl())),            // XOR A, [HL]
            0xAF => self.xor(self.regs.a),                              // XOR A, A
            0xB0 => self.or(self.regs.b),                               // OR A, B
            0xB1 => self.or(self.regs.c),                               // OR A, C
            0xB2 => self.or(self.regs.d),                               // OR A, D
            0xB3 => self.or(self.regs.e),                               // OR A, E
            0xB4 => self.or(self.regs.h),                               // OR A, H
            0xB5 => self.or(self.regs.l),                               // OR A, L
            0xB6 => self.or(bus.read(self.regs.hl())),             // OR A, [HL]
            0xB7 => self.or(self.regs.a),                               // OR A, A
            0xB8 => self.cp(self.regs.b),                               // CP A, B
            0xB9 => self.cp(self.regs.c),                               // CP A, C
            0xBA => self.cp(self.regs.d),                               // CP A, D
            0xBB => self.cp(self.regs.e),                               // CP A, E
            0xBC => self.cp(self.regs.h),                               // CP A, H
            0xBD => self.cp(self.regs.l),                               // CP A, L
            0xBE => self.cp(bus.read(self.regs.hl())),             // CP A, [HL]
            0xBF => self.cp(self.regs.a),                               // CP A, A
            0xC0 => branched = self.ret(!self.regs.get_z(), bus),                       // RET NZ
            0xC1 => { let bc = self.pop(bus); self.regs.set_bc(bc); }      // POP BC
            0xC2 => branched = self.jp(!self.regs.get_z(), bus),                        // JP NZ, a16
            0xC3 => { self.jp(true, bus); }                                      // JP a16
            0xC4 => branched = self.call(!self.regs.get_z(), bus),                      // CALL NZ, a16
            0xC5 => self.push(self.regs.bc(), bus),                          // PUSH BC
            0xC6 => { let n8 = self.fetch(bus); self.add(n8); }            // ADD A, n8
            0xC7 => self.rst(0x00, bus),                                     // RST $00
            0xC8 => branched = self.ret(self.regs.get_z(), bus),                        // RET Z
            0xC9 => { self.ret(true, bus); }                                     // RET
            0xCA => branched = self.jp(self.regs.get_z(), bus),                         // JP Z, a16
            0xCB => return self.step_cb(bus),                                     // PREFIX
            0xCC => branched = self.call(self.regs.get_z(), bus),                       // CALL Z, a16
            0xCD => { self.call(true, bus); }                                    // CALL a16
            0xCE => { let n8 = self.fetch(bus); self.adc(n8); }            // ADC A, n8
            0xCF => self.rst(0x08, bus),                                     // RST $08
            0xD0 => branched = self.ret(!self.regs.get_c(), bus),                       // RET NC
            0xD1 => { let de = self.pop(bus); self.regs.set_de(de); }      // POP DE
            0xD2 => branched = self.jp(!self.regs.get_c(), bus),                        // JP NC, a16
            0xD3 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xD4 => branched = self.call(!self.regs.get_c(), bus),                      // CALL NC, a16
            0xD5 => self.push(self.regs.de(), bus),                          // PUSH DE
            0xD6 => { let n8 = self.fetch(bus); self.sub(n8); }            // SUB A, n8
            0xD7 => self.rst(0x10, bus),                                     // RST $10
            0xD8 => branched = self.ret(self.regs.get_c(), bus),                        // RET C
            0xD9 => self.reti(bus),                                        // RETI
            0xDA => branched = self.jp(self.regs.get_c(), bus),                         // JP C, a16
            0xDB => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xDC => branched = self.call(self.regs.get_c(), bus),                       // CALL C, a16
            0xDD => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xDE => { let n8 = self.fetch(bus); self.sbc(n8); }            // SBC A, n8
            0xDF => self.rst(0x18, bus),                                     // RST $18
            0xE0 => {                                                   // LDH [a8], A
                let a8 = self.fetch(bus);
                bus.write(0xFF00 + a8 as u16, self.regs.a);
            }
            0xE1 => { let hl = self.pop(bus); self.regs.set_hl(hl); }      // POP HL
            0xE2 => {                                                   // LDH [C], A
                let addr = 0xFF00 + self.regs.c as u16;
                bus.write(addr, self.regs.a);
            }
            0xE3 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xE4 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xE5 => self.push(self.regs.hl(), bus),                          // PUSH HL
            0xE6 => { let n8 = self.fetch(bus); self.and(n8); }            // AND A, n8
            0xE7 => self.rst(0x20, bus),                                     // RST $20
            0xE8 => self.add_sp_e8(bus),                                   // ADD SP, e8
            0xE9 => self.jp_hl(),                                       // JP HL
            0xEA => {                                                   // LD [a16], A
                let a16 = self.fetch_u16(bus);
                bus.write(a16, self.regs.a);
            }
            0xEB => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xEC => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xED => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xEE => { let n8 = self.fetch(bus); self.xor(n8); }            // XOR A, n8
            0xEF => self.rst(0x28, bus),                                     // RST $28
            0xF0 => {                                                   // LDH A, [a8]
                let a8 = self.fetch(bus);
                self.regs.a = bus.read(0xFF00 + a8 as u16);
            }
            0xF1 => { let af = self.pop(bus); self.regs.set_af(af); }      // POP AF
            0xF2 => {                                                   // LDH A, [C]
                let addr = 0xFF00 + self.regs.c as u16;
                self.regs.a = bus.read(addr);
            }
            0xF3 => self.di(),                                          // DI
            0xF4 => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xF5 => self.push(self.regs.af(), bus),                          // PUSH AF
            0xF6 => { let n8 = self.fetch(bus); self.or(n8); }             // OR A, n8
            0xF7 => self.rst(0x30, bus),                                     // RST $30
            0xF8 => self.ld_hl_sp_e8(bus),                                 // LD HL, SP + e8
            0xF9 => self.regs.sp = self.regs.hl(),                      // LD SP, HL
            0xFA => {                                                   // LD A, [a16]
                let a16 = self.fetch_u16(bus);
                self.regs.a = bus.read(a16);
            }
            0xFB => self.ei(),                                          // EI
            0xFC => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xFD => panic!("illegal opcode: {:#04X}", opcode),          // —
            0xFE => { let n8 = self.fetch(bus); self.cp(n8); }             // CP A, n8
            0xFF => self.rst(0x38, bus),                                     // RST $38
        }

        if branched {
            Cpu::CYCLES_BRANCHED[opcode as usize]
        } else {
            Cpu::CYCLES[opcode as usize]
        }
    }

    // For instructions see https://rgbds.gbdev.io/docs/v0.9.1/gbz80.7

    /// Fetch the byte at PC and advance PC.
    pub fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let opcode = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        opcode
    }

    /// Fetch a little-endian 16-bit value (two bytes, low first).
    pub fn fetch_u16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.fetch(bus);
        let hi = self.fetch(bus);
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
    pub fn jr(&mut self, condition: bool, bus: &mut Bus) -> bool {
        let offset = self.fetch(bus) as i8;
        if condition {
            self.regs.pc = self.regs.pc.wrapping_add(offset as u16);
        }
        condition
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
    pub fn add_sp_e8(&mut self, bus: &mut Bus) {
        let sp = self.regs.sp;
        let val = self.fetch(bus) as i8;
        let result = sp.wrapping_add(val as u16);
        self.regs.set_z(false);
        self.regs.set_n(false);
        self.regs.set_h(((sp as u8) & 0x0F) + ((val as u8) & 0x0F) > 0x0F);
        self.regs.set_c(((sp as u8) as u16) + ((val as u8) as u16) > 0xFF);
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
    pub fn push(&mut self, val: u16, bus: &mut Bus) {
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, (val >> 8) as u8);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        bus.write(self.regs.sp, val as u8);
    }

    /// Pop a 16-bit value from the stack (low byte first).
    pub fn pop(&mut self, bus: &mut Bus) -> u16 {
        let lo = bus.read(self.regs.sp) as u16;
        self.regs.sp = self.regs.sp.wrapping_add(1);
        let hi = (bus.read(self.regs.sp) as u16) << 8;
        self.regs.sp = self.regs.sp.wrapping_add(1);
        hi | lo
    }

    /// Call: push PC and jump to fetched address if condition is met.
    pub fn call(&mut self, condition: bool, bus: &mut Bus) -> bool {
        let addr = self.fetch_u16(bus);
        if condition {
            self.push(self.regs.pc, bus);
            self.regs.pc = addr;
        }
        condition
    }

    /// Return: pop PC from stack if condition is met.
    pub fn ret(&mut self, condition: bool, bus: &mut Bus) -> bool{
        if condition {
            self.regs.pc = self.pop(bus);
        }
        condition
    }

    /// Return from interrupt: pop PC and enable IME.
    pub fn reti(&mut self, bus: &mut Bus) {
        self.ei();
        self.ret(true, bus);
    }

    /// Restart: push PC and jump to fixed address.
    pub fn rst(&mut self, addr: u16, bus: &mut Bus) {
        self.push(self.regs.pc, bus);
        self.regs.pc = addr;
    }

    /// Absolute jump to fetched address if condition is met.
    pub fn jp(&mut self, condition: bool, bus: &mut Bus) -> bool {
        let addr = self.fetch_u16(bus);
        if condition {
            self.regs.pc = addr;
        }
        condition
    }

    /// Jump to address in HL.
    pub fn jp_hl(&mut self) {
        self.regs.pc = self.regs.hl();
    }

    /// LD HL, SP+e8. Loads SP + signed 8-bit offset into HL.
    pub fn ld_hl_sp_e8(&mut self, bus: &mut Bus) {
        let sp = self.regs.sp;
        let e8 = self.fetch(bus) as i8;
        let result = sp.wrapping_add(e8 as u16);
        self.regs.set_z(false);
        self.regs.set_n(false);
        self.regs.set_h(((sp as u8) & 0x0F) + ((e8 as u8) & 0x0F) > 0x0F);
        self.regs.set_c(((sp as u8) as u16) + ((e8 as u8) as u16) > 0xFF);
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

    /// Check if CPU is halted. Returns true if still halted (caller should return early).
    /// Wakes up if any interrupt is pending.
    fn is_halt(&mut self, bus: &mut Bus) -> bool {
        if !self.halted { return false; }
        if bus.pending_interrupt().is_some() {
            self.halted = false;
            return false;
        }
        true
    }

    /// Handle pending interrupt if IME is set. Disables IME, clears IF bit,
    /// pushes PC, and jumps to the interrupt vector. Returns true if handled.
    fn is_interrupt(&mut self, bus: &mut Bus) -> bool {
        let interrupt = bus.pending_interrupt();
        if self.ime && interrupt.is_some() {
            let bit = interrupt.unwrap();
            self.di();
            bus.clear_interrupt(bit);
            self.push(self.regs.pc, bus);
            self.regs.pc = 0x40 + (bit as u16) * 8;
            return true;
        }
        false
    }
}
