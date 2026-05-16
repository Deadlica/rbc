use super::Cpu;
use super::Bus;

impl Cpu {
    const CYCLES_CB: [u8; 256] = [
    //  0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 0x
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 1x
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 2x
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 3x
        8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 4x
        8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 5x
        8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 6x
        8,  8,  8,  8,  8,  8, 12,  8,  8,  8,  8,  8,  8,  8, 12,  8, // 7x
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 8x
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // 9x
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // Ax
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // Bx
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // Cx
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // Dx
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // Ex
        8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8, // Fx
    ];
    
    /// Execute a CB-prefixed instruction: fetch the second byte and dispatch.
    pub fn step_cb(&mut self, bus: &mut Bus) -> u8 {
        let cb_op = self.fetch(bus);

        match cb_op {
            0x00 => self.regs.b = self.rlc(self.regs.b),                // RLC B
            0x01 => self.regs.c = self.rlc(self.regs.c),                // RLC C
            0x02 => self.regs.d = self.rlc(self.regs.d),                // RLC D
            0x03 => self.regs.e = self.rlc(self.regs.e),                // RLC E
            0x04 => self.regs.h = self.rlc(self.regs.h),                // RLC H
            0x05 => self.regs.l = self.rlc(self.regs.l),                // RLC L
            0x06 => {                                                   // RLC [HL]
                let addr = self.regs.hl();
                let val = self.rlc(bus.read(addr));
                bus.write(addr, val);
            }
            0x07 => self.regs.a = self.rlc(self.regs.a),                // RLC A
            0x08 => self.regs.b = self.rrc(self.regs.b),                // RRC B
            0x09 => self.regs.c = self.rrc(self.regs.c),                // RRC C
            0x0A => self.regs.d = self.rrc(self.regs.d),                // RRC D
            0x0B => self.regs.e = self.rrc(self.regs.e),                // RRC E
            0x0C => self.regs.h = self.rrc(self.regs.h),                // RRC H
            0x0D => self.regs.l = self.rrc(self.regs.l),                // RRC L
            0x0E => {                                                   // RRC [HL]
                let addr = self.regs.hl();
                let val = self.rrc(bus.read(addr));
                bus.write(addr, val);
            }
            0x0F => self.regs.a = self.rrc(self.regs.a),                // RRC A
            0x10 => self.regs.b = self.rl(self.regs.b),                 // RL B
            0x11 => self.regs.c = self.rl(self.regs.c),                 // RL C
            0x12 => self.regs.d = self.rl(self.regs.d),                 // RL D
            0x13 => self.regs.e = self.rl(self.regs.e),                 // RL E
            0x14 => self.regs.h = self.rl(self.regs.h),                 // RL H
            0x15 => self.regs.l = self.rl(self.regs.l),                 // RL L
            0x16 => {                                                   // RL [HL]
                let addr = self.regs.hl();
                let val = self.rl(bus.read(addr));
                bus.write(addr, val);
            }
            0x17 => self.regs.a = self.rl(self.regs.a),                 // RL A
            0x18 => self.regs.b = self.rr(self.regs.b),                 // RR B
            0x19 => self.regs.c = self.rr(self.regs.c),                 // RR C
            0x1A => self.regs.d = self.rr(self.regs.d),                 // RR D
            0x1B => self.regs.e = self.rr(self.regs.e),                 // RR E
            0x1C => self.regs.h = self.rr(self.regs.h),                 // RR H
            0x1D => self.regs.l = self.rr(self.regs.l),                 // RR L
            0x1E => {                                                   // RR [HL]
                let addr = self.regs.hl();
                let val = self.rr(bus.read(addr));
                bus.write(addr, val);
            }
            0x1F => self.regs.a = self.rr(self.regs.a),                 // RR A
            0x20 => self.regs.b = self.sla(self.regs.b),                // SLA B
            0x21 => self.regs.c = self.sla(self.regs.c),                // SLA C
            0x22 => self.regs.d = self.sla(self.regs.d),                // SLA D
            0x23 => self.regs.e = self.sla(self.regs.e),                // SLA E
            0x24 => self.regs.h = self.sla(self.regs.h),                // SLA H
            0x25 => self.regs.l = self.sla(self.regs.l),                // SLA L
            0x26 => {                                                   // SLA [HL]
                let addr = self.regs.hl();
                let val = self.sla(bus.read(addr));
                bus.write(addr, val);
            }
            0x27 => self.regs.a = self.sla(self.regs.a),                // SLA A
            0x28 => self.regs.b = self.sra(self.regs.b),                // SRA B
            0x29 => self.regs.c = self.sra(self.regs.c),                // SRA C
            0x2A => self.regs.d = self.sra(self.regs.d),                // SRA D
            0x2B => self.regs.e = self.sra(self.regs.e),                // SRA E
            0x2C => self.regs.h = self.sra(self.regs.h),                // SRA H
            0x2D => self.regs.l = self.sra(self.regs.l),                // SRA L
            0x2E => {                                                   // SRA [HL]
                let addr = self.regs.hl();
                let val = self.sra(bus.read(addr));
                bus.write(addr, val);
            }
            0x2F => self.regs.a = self.sra(self.regs.a),                // SRA A
            0x30 => self.regs.b = self.swap(self.regs.b),               // SWAP B
            0x31 => self.regs.c = self.swap(self.regs.c),               // SWAP C
            0x32 => self.regs.d = self.swap(self.regs.d),               // SWAP D
            0x33 => self.regs.e = self.swap(self.regs.e),               // SWAP E
            0x34 => self.regs.h = self.swap(self.regs.h),               // SWAP H
            0x35 => self.regs.l = self.swap(self.regs.l),               // SWAP L
            0x36 => {                                                   // SWAP [HL]
                let addr = self.regs.hl();
                let val = self.swap(bus.read(addr));
                bus.write(addr, val);
            }
            0x37 => self.regs.a = self.swap(self.regs.a),               // SWAP A
            0x38 => self.regs.b = self.srl(self.regs.b),                // SRL B
            0x39 => self.regs.c = self.srl(self.regs.c),                // SRL C
            0x3A => self.regs.d = self.srl(self.regs.d),                // SRL D
            0x3B => self.regs.e = self.srl(self.regs.e),                // SRL E
            0x3C => self.regs.h = self.srl(self.regs.h),                // SRL H
            0x3D => self.regs.l = self.srl(self.regs.l),                // SRL L
            0x3E => {                                                   // SRL [HL]
                let addr = self.regs.hl();
                let val = self.srl(bus.read(addr));
                bus.write(addr, val);
            }
            0x3F => self.regs.a = self.srl(self.regs.a),                // SRL A
            0x40 => self.bit(0, self.regs.b),                           // BIT 0, B
            0x41 => self.bit(0, self.regs.c),                           // BIT 0, C
            0x42 => self.bit(0, self.regs.d),                           // BIT 0, D
            0x43 => self.bit(0, self.regs.e),                           // BIT 0, E
            0x44 => self.bit(0, self.regs.h),                           // BIT 0, H
            0x45 => self.bit(0, self.regs.l),                           // BIT 0, L
            0x46 => self.bit(0, bus.read(self.regs.hl())),              // BIT 0, [HL]
            0x47 => self.bit(0, self.regs.a),                           // BIT 0, A
            0x48 => self.bit(1, self.regs.b),                           // BIT 1, B
            0x49 => self.bit(1, self.regs.c),                           // BIT 1, C
            0x4A => self.bit(1, self.regs.d),                           // BIT 1, D
            0x4B => self.bit(1, self.regs.e),                           // BIT 1, E
            0x4C => self.bit(1, self.regs.h),                           // BIT 1, H
            0x4D => self.bit(1, self.regs.l),                           // BIT 1, L
            0x4E => self.bit(1, bus.read(self.regs.hl())),              // BIT 1, [HL]
            0x4F => self.bit(1, self.regs.a),                           // BIT 1, A
            0x50 => self.bit(2, self.regs.b),                           // BIT 2, B
            0x51 => self.bit(2, self.regs.c),                           // BIT 2, C
            0x52 => self.bit(2, self.regs.d),                           // BIT 2, D
            0x53 => self.bit(2, self.regs.e),                           // BIT 2, E
            0x54 => self.bit(2, self.regs.h),                           // BIT 2, H
            0x55 => self.bit(2, self.regs.l),                           // BIT 2, L
            0x56 => self.bit(2, bus.read(self.regs.hl())),              // BIT 2, [HL]
            0x57 => self.bit(2, self.regs.a),                           // BIT 2, A
            0x58 => self.bit(3, self.regs.b),                           // BIT 3, B
            0x59 => self.bit(3, self.regs.c),                           // BIT 3, C
            0x5A => self.bit(3, self.regs.d),                           // BIT 3, D
            0x5B => self.bit(3, self.regs.e),                           // BIT 3, E
            0x5C => self.bit(3, self.regs.h),                           // BIT 3, H
            0x5D => self.bit(3, self.regs.l),                           // BIT 3, L
            0x5E => self.bit(3, bus.read(self.regs.hl())),              // BIT 3, [HL]
            0x5F => self.bit(3, self.regs.a),                           // BIT 3, A
            0x60 => self.bit(4, self.regs.b),                           // BIT 4, B
            0x61 => self.bit(4, self.regs.c),                           // BIT 4, C
            0x62 => self.bit(4, self.regs.d),                           // BIT 4, D
            0x63 => self.bit(4, self.regs.e),                           // BIT 4, E
            0x64 => self.bit(4, self.regs.h),                           // BIT 4, H
            0x65 => self.bit(4, self.regs.l),                           // BIT 4, L
            0x66 => self.bit(4, bus.read(self.regs.hl())),              // BIT 4, [HL]
            0x67 => self.bit(4, self.regs.a),                           // BIT 4, A
            0x68 => self.bit(5, self.regs.b),                           // BIT 5, B
            0x69 => self.bit(5, self.regs.c),                           // BIT 5, C
            0x6A => self.bit(5, self.regs.d),                           // BIT 5, D
            0x6B => self.bit(5, self.regs.e),                           // BIT 5, E
            0x6C => self.bit(5, self.regs.h),                           // BIT 5, H
            0x6D => self.bit(5, self.regs.l),                           // BIT 5, L
            0x6E => self.bit(5, bus.read(self.regs.hl())),              // BIT 5, [HL]
            0x6F => self.bit(5, self.regs.a),                           // BIT 5, A
            0x70 => self.bit(6, self.regs.b),                           // BIT 6, B
            0x71 => self.bit(6, self.regs.c),                           // BIT 6, C
            0x72 => self.bit(6, self.regs.d),                           // BIT 6, D
            0x73 => self.bit(6, self.regs.e),                           // BIT 6, E
            0x74 => self.bit(6, self.regs.h),                           // BIT 6, H
            0x75 => self.bit(6, self.regs.l),                           // BIT 6, L
            0x76 => self.bit(6, bus.read(self.regs.hl())),              // BIT 6, [HL]
            0x77 => self.bit(6, self.regs.a),                           // BIT 6, A
            0x78 => self.bit(7, self.regs.b),                           // BIT 7, B
            0x79 => self.bit(7, self.regs.c),                           // BIT 7, C
            0x7A => self.bit(7, self.regs.d),                           // BIT 7, D
            0x7B => self.bit(7, self.regs.e),                           // BIT 7, E
            0x7C => self.bit(7, self.regs.h),                           // BIT 7, H
            0x7D => self.bit(7, self.regs.l),                           // BIT 7, L
            0x7E => self.bit(7, bus.read(self.regs.hl())),              // BIT 7, [HL]
            0x7F => self.bit(7, self.regs.a),                           // BIT 7, A
            0x80 => self.regs.b = self.res(0, self.regs.b),             // RES 0, B
            0x81 => self.regs.c = self.res(0, self.regs.c),             // RES 0, C
            0x82 => self.regs.d = self.res(0, self.regs.d),             // RES 0, D
            0x83 => self.regs.e = self.res(0, self.regs.e),             // RES 0, E
            0x84 => self.regs.h = self.res(0, self.regs.h),             // RES 0, H
            0x85 => self.regs.l = self.res(0, self.regs.l),             // RES 0, L
            0x86 => {                                                   // RES 0, [HL]
                let addr = self.regs.hl();
                let val = self.res(0, bus.read(addr));
                bus.write(addr, val);
            }
            0x87 => self.regs.a = self.res(0, self.regs.a),             // RES 0, A
            0x88 => self.regs.b = self.res(1, self.regs.b),             // RES 1, B
            0x89 => self.regs.c = self.res(1, self.regs.c),             // RES 1, C
            0x8A => self.regs.d = self.res(1, self.regs.d),             // RES 1, D
            0x8B => self.regs.e = self.res(1, self.regs.e),             // RES 1, E
            0x8C => self.regs.h = self.res(1, self.regs.h),             // RES 1, H
            0x8D => self.regs.l = self.res(1, self.regs.l),             // RES 1, L
            0x8E => {                                                   // RES 1, [HL]
                let addr = self.regs.hl();
                let val = self.res(1, bus.read(addr));
                bus.write(addr, val);
            }
            0x8F => self.regs.a = self.res(1, self.regs.a),             // RES 1, A
            0x90 => self.regs.b = self.res(2, self.regs.b),             // RES 2, B
            0x91 => self.regs.c = self.res(2, self.regs.c),             // RES 2, C
            0x92 => self.regs.d = self.res(2, self.regs.d),             // RES 2, D
            0x93 => self.regs.e = self.res(2, self.regs.e),             // RES 2, E
            0x94 => self.regs.h = self.res(2, self.regs.h),             // RES 2, H
            0x95 => self.regs.l = self.res(2, self.regs.l),             // RES 2, L
            0x96 => {                                                   // RES 2, [HL]
                let addr = self.regs.hl();
                let val = self.res(2, bus.read(addr));
                bus.write(addr, val);
            }
            0x97 => self.regs.a = self.res(2, self.regs.a),             // RES 2, A
            0x98 => self.regs.b = self.res(3, self.regs.b),             // RES 3, B
            0x99 => self.regs.c = self.res(3, self.regs.c),             // RES 3, C
            0x9A => self.regs.d = self.res(3, self.regs.d),             // RES 3, D
            0x9B => self.regs.e = self.res(3, self.regs.e),             // RES 3, E
            0x9C => self.regs.h = self.res(3, self.regs.h),             // RES 3, H
            0x9D => self.regs.l = self.res(3, self.regs.l),             // RES 3, L
            0x9E => {                                                   // RES 3, [HL]
                let addr = self.regs.hl();
                let val = self.res(3, bus.read(addr));
                bus.write(addr, val);
            }
            0x9F => self.regs.a = self.res(3, self.regs.a),             // RES 3, A
            0xA0 => self.regs.b = self.res(4, self.regs.b),             // RES 4, B
            0xA1 => self.regs.c = self.res(4, self.regs.c),             // RES 4, C
            0xA2 => self.regs.d = self.res(4, self.regs.d),             // RES 4, D
            0xA3 => self.regs.e = self.res(4, self.regs.e),             // RES 4, E
            0xA4 => self.regs.h = self.res(4, self.regs.h),             // RES 4, H
            0xA5 => self.regs.l = self.res(4, self.regs.l),             // RES 4, L
            0xA6 => {                                                   // RES 4, [HL]
                let addr = self.regs.hl();
                let val = self.res(4, bus.read(addr));
                bus.write(addr, val);
            }
            0xA7 => self.regs.a = self.res(4, self.regs.a),             // RES 4, A
            0xA8 => self.regs.b = self.res(5, self.regs.b),             // RES 5, B
            0xA9 => self.regs.c = self.res(5, self.regs.c),             // RES 5, C
            0xAA => self.regs.d = self.res(5, self.regs.d),             // RES 5, D
            0xAB => self.regs.e = self.res(5, self.regs.e),             // RES 5, E
            0xAC => self.regs.h = self.res(5, self.regs.h),             // RES 5, H
            0xAD => self.regs.l = self.res(5, self.regs.l),             // RES 5, L
            0xAE => {                                                   // RES 5, [HL]
                let addr = self.regs.hl();
                let val = self.res(5, bus.read(addr));
                bus.write(addr, val);
            }
            0xAF => self.regs.a = self.res(5, self.regs.a),             // RES 5, A
            0xB0 => self.regs.b = self.res(6, self.regs.b),             // RES 6, B
            0xB1 => self.regs.c = self.res(6, self.regs.c),             // RES 6, C
            0xB2 => self.regs.d = self.res(6, self.regs.d),             // RES 6, D
            0xB3 => self.regs.e = self.res(6, self.regs.e),             // RES 6, E
            0xB4 => self.regs.h = self.res(6, self.regs.h),             // RES 6, H
            0xB5 => self.regs.l = self.res(6, self.regs.l),             // RES 6, L
            0xB6 => {                                                   // RES 6, [HL]
                let addr = self.regs.hl();
                let val = self.res(6, bus.read(addr));
                bus.write(addr, val);
            }
            0xB7 => self.regs.a = self.res(6, self.regs.a),             // RES 6, A
            0xB8 => self.regs.b = self.res(7, self.regs.b),             // RES 7, B
            0xB9 => self.regs.c = self.res(7, self.regs.c),             // RES 7, C
            0xBA => self.regs.d = self.res(7, self.regs.d),             // RES 7, D
            0xBB => self.regs.e = self.res(7, self.regs.e),             // RES 7, E
            0xBC => self.regs.h = self.res(7, self.regs.h),             // RES 7, H
            0xBD => self.regs.l = self.res(7, self.regs.l),             // RES 7, L
            0xBE => {                                                   // RES 7, [HL]
                let addr = self.regs.hl();
                let val = self.res(7, bus.read(addr));
                bus.write(addr, val);
            }
            0xBF => self.regs.a = self.res(7, self.regs.a),             // RES 7, A
            0xC0 => self.regs.b = self.set(0, self.regs.b),             // SET 0, B
            0xC1 => self.regs.c = self.set(0, self.regs.c),             // SET 0, C
            0xC2 => self.regs.d = self.set(0, self.regs.d),             // SET 0, D
            0xC3 => self.regs.e = self.set(0, self.regs.e),             // SET 0, E
            0xC4 => self.regs.h = self.set(0, self.regs.h),             // SET 0, H
            0xC5 => self.regs.l = self.set(0, self.regs.l),             // SET 0, L
            0xC6 => {                                                   // SET 0, [HL]
                let addr = self.regs.hl();
                let val = self.set(0, bus.read(addr));
                bus.write(addr, val);
            }
            0xC7 => self.regs.a = self.set(0, self.regs.a),             // SET 0, A
            0xC8 => self.regs.b = self.set(1, self.regs.b),             // SET 1, B
            0xC9 => self.regs.c = self.set(1, self.regs.c),             // SET 1, C
            0xCA => self.regs.d = self.set(1, self.regs.d),             // SET 1, D
            0xCB => self.regs.e = self.set(1, self.regs.e),             // SET 1, E
            0xCC => self.regs.h = self.set(1, self.regs.h),             // SET 1, H
            0xCD => self.regs.l = self.set(1, self.regs.l),             // SET 1, L
            0xCE => {                                                   // SET 1, [HL]
                let addr = self.regs.hl();
                let val = self.set(1, bus.read(addr));
                bus.write(addr, val);
            }
            0xCF => self.regs.a = self.set(1, self.regs.a),             // SET 1, A
            0xD0 => self.regs.b = self.set(2, self.regs.b),             // SET 2, B
            0xD1 => self.regs.c = self.set(2, self.regs.c),             // SET 2, C
            0xD2 => self.regs.d = self.set(2, self.regs.d),             // SET 2, D
            0xD3 => self.regs.e = self.set(2, self.regs.e),             // SET 2, E
            0xD4 => self.regs.h = self.set(2, self.regs.h),             // SET 2, H
            0xD5 => self.regs.l = self.set(2, self.regs.l),             // SET 2, L
            0xD6 => {                                                   // SET 2, [HL]
                let addr = self.regs.hl();
                let val = self.set(2, bus.read(addr));
                bus.write(addr, val);
            }
            0xD7 => self.regs.a = self.set(2, self.regs.a),             // SET 2, A
            0xD8 => self.regs.b = self.set(3, self.regs.b),             // SET 3, B
            0xD9 => self.regs.c = self.set(3, self.regs.c),             // SET 3, C
            0xDA => self.regs.d = self.set(3, self.regs.d),             // SET 3, D
            0xDB => self.regs.e = self.set(3, self.regs.e),             // SET 3, E
            0xDC => self.regs.h = self.set(3, self.regs.h),             // SET 3, H
            0xDD => self.regs.l = self.set(3, self.regs.l),             // SET 3, L
            0xDE => {                                                   // SET 3, [HL]
                let addr = self.regs.hl();
                let val = self.set(3, bus.read(addr));
                bus.write(addr, val);
            }
            0xDF => self.regs.a = self.set(3, self.regs.a),             // SET 3, A
            0xE0 => self.regs.b = self.set(4, self.regs.b),             // SET 4, B
            0xE1 => self.regs.c = self.set(4, self.regs.c),             // SET 4, C
            0xE2 => self.regs.d = self.set(4, self.regs.d),             // SET 4, D
            0xE3 => self.regs.e = self.set(4, self.regs.e),             // SET 4, E
            0xE4 => self.regs.h = self.set(4, self.regs.h),             // SET 4, H
            0xE5 => self.regs.l = self.set(4, self.regs.l),             // SET 4, L
            0xE6 => {                                                   // SET 4, [HL]
                let addr = self.regs.hl();
                let val = self.set(4, bus.read(addr));
                bus.write(addr, val);
            }
            0xE7 => self.regs.a = self.set(4, self.regs.a),             // SET 4, A
            0xE8 => self.regs.b = self.set(5, self.regs.b),             // SET 5, B
            0xE9 => self.regs.c = self.set(5, self.regs.c),             // SET 5, C
            0xEA => self.regs.d = self.set(5, self.regs.d),             // SET 5, D
            0xEB => self.regs.e = self.set(5, self.regs.e),             // SET 5, E
            0xEC => self.regs.h = self.set(5, self.regs.h),             // SET 5, H
            0xED => self.regs.l = self.set(5, self.regs.l),             // SET 5, L
            0xEE => {                                                   // SET 5, [HL]
                let addr = self.regs.hl();
                let val = self.set(5, bus.read(addr));
                bus.write(addr, val);
            }
            0xEF => self.regs.a = self.set(5, self.regs.a),             // SET 5, A
            0xF0 => self.regs.b = self.set(6, self.regs.b),             // SET 6, B
            0xF1 => self.regs.c = self.set(6, self.regs.c),             // SET 6, C
            0xF2 => self.regs.d = self.set(6, self.regs.d),             // SET 6, D
            0xF3 => self.regs.e = self.set(6, self.regs.e),             // SET 6, E
            0xF4 => self.regs.h = self.set(6, self.regs.h),             // SET 6, H
            0xF5 => self.regs.l = self.set(6, self.regs.l),             // SET 6, L
            0xF6 => {                                                   // SET 6, [HL]
                let addr = self.regs.hl();
                let val = self.set(6, bus.read(addr));
                bus.write(addr, val);
            }
            0xF7 => self.regs.a = self.set(6, self.regs.a),             // SET 6, A
            0xF8 => self.regs.b = self.set(7, self.regs.b),             // SET 7, B
            0xF9 => self.regs.c = self.set(7, self.regs.c),             // SET 7, C
            0xFA => self.regs.d = self.set(7, self.regs.d),             // SET 7, D
            0xFB => self.regs.e = self.set(7, self.regs.e),             // SET 7, E
            0xFC => self.regs.h = self.set(7, self.regs.h),             // SET 7, H
            0xFD => self.regs.l = self.set(7, self.regs.l),             // SET 7, L
            0xFE => {                                                   // SET 7, [HL]
                let addr = self.regs.hl();
                let val = self.set(7, bus.read(addr));
                bus.write(addr, val);
            }
            0xFF => self.regs.a = self.set(7, self.regs.a),             // SET 7, A
        }
        Cpu::CYCLES_CB[cb_op as usize]
    }

    /// Test bit `b` of `val`.
    pub fn bit(&mut self, b: u8, val: u8) {
        self.regs.set_z(val & (1 << b) == 0);
        self.regs.set_n(false);
        self.regs.set_h(true);
    }

    /// Swap upper and lower nibbles of `val`.
    pub fn swap(&mut self, val: u8) -> u8 {
        let result = (val << 4) | (val >> 4);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(false);
        result
    }

    /// Shift left arithmetic.
    pub fn sla(&mut self, val: u8) -> u8 {
        let result = val << 1;
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(val & 0x80 != 0);
        result
    }

    /// Shift right arithmetic.
    pub fn sra(&mut self, val: u8) -> u8 {
        let result = (val >> 1) | (val & 0x80);
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(val & 0x01 != 0);
        result
    }

    /// Shift right logical.
    pub fn srl(&mut self, val: u8) -> u8 {
        let result = val >> 1;
        self.regs.set_z(result == 0);
        self.regs.set_n(false);
        self.regs.set_h(false);
        self.regs.set_c(val & 0x01 != 0);
        result
    }

    /// Set bit `b` of `val`.
    pub fn set(&mut self, b: u8, val: u8) -> u8 {
        val | (1 << b)
    }

    /// Reset (clear) bit `b` of `val`.
    pub fn res(&mut self, b: u8, val: u8) -> u8 {
        val & !(1 << b)
    }
}
