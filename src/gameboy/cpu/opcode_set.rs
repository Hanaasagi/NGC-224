use std::array::IntoIter;
use std::collections::HashMap;
use std::iter::FromIterator;

use super::cpu::CPU;

pub struct OpCode<'a> {
    name: &'a str,
    clock: u32,
    func: fn(&mut CPU) -> u32,
}

impl<'a> OpCode<'a> {
    pub fn new(name: &'a str, clock: u32, func: fn(&mut CPU) -> u32) -> Self {
        Self { name, clock, func }
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn get_clock(&self) -> u32 {
        self.clock
    }

    pub fn ex(&self, cpu: &mut CPU) -> u32 {
        (self.func)(cpu) + self.clock
    }
}

lazy_static! {
    pub static ref OP_CODE_SET: HashMap<u8, OpCode<'static>> = {
        HashMap::<_, _>::from_iter(IntoIter::new([
            (0x00, OpCode::new("NOP", 4, CPU::op_0x00)),
            (0x01, OpCode::new("LD BC,d16", 12, CPU::op_0x01)),
            (0x02, OpCode::new("LD BC,A", 8, CPU::op_0x02)),
            (0x03, OpCode::new("INC BC", 8, CPU::op_0x03)),
            (0x04, OpCode::new("INC B", 4, CPU::op_0x04)),
            (0x05, OpCode::new("DEC B", 4, CPU::op_0x05)),
            (0x06, OpCode::new("LD B,d8", 8, CPU::op_0x06)),
            (0x07, OpCode::new("RLCA", 4, CPU::op_0x07)),
            (0x08, OpCode::new("LD (a16),SP", 20, CPU::op_0x08)),
            (0x09, OpCode::new("ADD HL,BC", 8, CPU::op_0x09)),
            (0x0A, OpCode::new("LD A,(BC)", 8, CPU::op_0x0A)),
            (0x0B, OpCode::new("DEC BC", 8, CPU::op_0x0B)),
            (0x0C, OpCode::new("INC C", 4, CPU::op_0x0C)),
            (0x0D, OpCode::new("DEC C", 4, CPU::op_0x0D)),
            (0x0E, OpCode::new("LD C,d8", 8, CPU::op_0x0E)),
            (0x0F, OpCode::new("RRCA", 4, CPU::op_0x0F)),
            (0x10, OpCode::new("STOP 0", 4, CPU::op_0x10)),
            (0x11, OpCode::new("LD DE,d16", 12, CPU::op_0x11)),
            (0x12, OpCode::new("LD (DE),A", 8, CPU::op_0x12)),
            (0x13, OpCode::new("INC DE", 8, CPU::op_0x13)),
            (0x14, OpCode::new("INC D", 4, CPU::op_0x14)),
            (0x15, OpCode::new("DEC D", 4, CPU::op_0x15)),
            (0x16, OpCode::new("LD D,d8", 8, CPU::op_0x16)),
            (0x17, OpCode::new("RLA", 4, CPU::op_0x17)),
            (0x18, OpCode::new("JR r8", 12, CPU::op_0x18)),
            (0x19, OpCode::new("ADD HL,DE", 8, CPU::op_0x19)),
            (0x1A, OpCode::new("LD A,(DE)", 8, CPU::op_0x1A)),
            (0x1B, OpCode::new("DEC DE", 8, CPU::op_0x1B)),
            (0x1C, OpCode::new("INC E", 4, CPU::op_0x1C)),
            (0x1D, OpCode::new("DEC E", 4, CPU::op_0x1D)),
            (0x1E, OpCode::new("LD E,d8", 8, CPU::op_0x1E)),
            (0x1F, OpCode::new("RRA", 4, CPU::op_0x1F)),
            (0x20, OpCode::new("JR NZ,r8", 8, CPU::op_0x20)),
            (0x21, OpCode::new("LD HL,d16", 12, CPU::op_0x21)),
            (0x22, OpCode::new("LD (HL+),A", 8, CPU::op_0x22)),
            (0x23, OpCode::new("INC HL", 8, CPU::op_0x23)),
            (0x24, OpCode::new("INC H", 4, CPU::op_0x24)),
            (0x25, OpCode::new("DEC H", 4, CPU::op_0x25)),
            (0x26, OpCode::new("LD H,d8", 8, CPU::op_0x26)),
            (0x27, OpCode::new("DAA", 4, CPU::op_0x27)),
            (0x28, OpCode::new("JR Z,r8", 8, CPU::op_0x28)),
            (0x29, OpCode::new("ADD HL,HL", 8, CPU::op_0x29)),
            (0x2A, OpCode::new("LD A,(HL+)", 8, CPU::op_0x2A)),
            (0x2B, OpCode::new("DEC HL", 8, CPU::op_0x2B)),
            (0x2C, OpCode::new("INC L", 4, CPU::op_0x2C)),
            (0x2D, OpCode::new("DEC L", 4, CPU::op_0x2D)),
            (0x2E, OpCode::new("LD L,d8", 8, CPU::op_0x2E)),
            (0x2F, OpCode::new("CPL", 4, CPU::op_0x2F)),
            (0x30, OpCode::new("JR NC,r8", 8, CPU::op_0x30)),
            (0x31, OpCode::new("LD SP,d16", 12, CPU::op_0x31)),
            (0x32, OpCode::new("LD (HL-),A", 8, CPU::op_0x32)),
            (0x33, OpCode::new("INC SP", 8, CPU::op_0x33)),
            (0x34, OpCode::new("INC (HL)", 12, CPU::op_0x34)),
            (0x35, OpCode::new("DEC (HL)", 12, CPU::op_0x35)),
            (0x36, OpCode::new("LD (HL),d8", 12, CPU::op_0x36)),
            (0x37, OpCode::new("SCF", 4, CPU::op_0x37)),
            (0x38, OpCode::new("JR C,r8", 8, CPU::op_0x38)),
            (0x39, OpCode::new("ADD HL,SP", 8, CPU::op_0x39)),
            (0x3A, OpCode::new("LD A,(HL-)", 8, CPU::op_0x3A)),
            (0x3B, OpCode::new("DEC SP", 8, CPU::op_0x3B)),
            (0x3C, OpCode::new("INC A", 4, CPU::op_0x3C)),
            (0x3D, OpCode::new("DEC A", 4, CPU::op_0x3D)),
            (0x3E, OpCode::new("LD A,d8", 8, CPU::op_0x3E)),
            (0x3F, OpCode::new("CCF", 4, CPU::op_0x3F)),
            (0x40, OpCode::new("LD B,B", 4, CPU::op_0x40)),
            (0x41, OpCode::new("LD B,C", 4, CPU::op_0x41)),
            (0x42, OpCode::new("LD B,D", 4, CPU::op_0x42)),
            (0x43, OpCode::new("LD B,E", 4, CPU::op_0x43)),
            (0x44, OpCode::new("LD B,H", 4, CPU::op_0x44)),
            (0x45, OpCode::new("LD B,L", 4, CPU::op_0x45)),
            (0x46, OpCode::new("LD B,(HL)", 8, CPU::op_0x46)),
            (0x47, OpCode::new("LD B,A", 4, CPU::op_0x47)),
            (0x48, OpCode::new("LD C,B", 4, CPU::op_0x48)),
            (0x49, OpCode::new("LD C,C", 4, CPU::op_0x49)),
            (0x4A, OpCode::new("LD C,D", 4, CPU::op_0x4A)),
            (0x4B, OpCode::new("LD C,E", 4, CPU::op_0x4B)),
            (0x4C, OpCode::new("LD C,H", 4, CPU::op_0x4C)),
            (0x4D, OpCode::new("LD C,L", 4, CPU::op_0x4D)),
            (0x4E, OpCode::new("LD C,(HL)", 8, CPU::op_0x4E)),
            (0x4F, OpCode::new("LD C,A", 4, CPU::op_0x4F)),
            (0x50, OpCode::new("LD D,B", 4, CPU::op_0x50)),
            (0x51, OpCode::new("LD D,C", 4, CPU::op_0x51)),
            (0x52, OpCode::new("LD D,D", 4, CPU::op_0x52)),
            (0x53, OpCode::new("LD D,E", 4, CPU::op_0x53)),
            (0x54, OpCode::new("LD D,H", 4, CPU::op_0x54)),
            (0x55, OpCode::new("LD D,L", 4, CPU::op_0x55)),
            (0x56, OpCode::new("LD D,(HL)", 8, CPU::op_0x56)),
            (0x57, OpCode::new("LD D,A", 4, CPU::op_0x57)),
            (0x58, OpCode::new("LD E,B", 4, CPU::op_0x58)),
            (0x59, OpCode::new("LD E,C", 4, CPU::op_0x59)),
            (0x5A, OpCode::new("LD E,D", 4, CPU::op_0x5A)),
            (0x5B, OpCode::new("LD E,E", 4, CPU::op_0x5B)),
            (0x5C, OpCode::new("LD E,H", 4, CPU::op_0x5C)),
            (0x5D, OpCode::new("LD E,L", 4, CPU::op_0x5D)),
            (0x5E, OpCode::new("LD E,(HL)", 8, CPU::op_0x5E)),
            (0x5F, OpCode::new("LD E,A", 4, CPU::op_0x5F)),
            (0x60, OpCode::new("LD H,B", 4, CPU::op_0x60)),
            (0x61, OpCode::new("LD H,C", 4, CPU::op_0x61)),
            (0x62, OpCode::new("LD H,D", 4, CPU::op_0x62)),
            (0x63, OpCode::new("LD H,E", 4, CPU::op_0x63)),
            (0x64, OpCode::new("LD H,H", 4, CPU::op_0x64)),
            (0x65, OpCode::new("LD H,L", 4, CPU::op_0x65)),
            (0x66, OpCode::new("LD H,(HL)", 8, CPU::op_0x66)),
            (0x67, OpCode::new("LD H,A", 4, CPU::op_0x67)),
            (0x68, OpCode::new("LD L,B", 4, CPU::op_0x68)),
            (0x69, OpCode::new("LD L,C", 4, CPU::op_0x69)),
            (0x6A, OpCode::new("LD L,D", 4, CPU::op_0x6A)),
            (0x6B, OpCode::new("LD L,E", 4, CPU::op_0x6B)),
            (0x6C, OpCode::new("LD L,H", 4, CPU::op_0x6C)),
            (0x6D, OpCode::new("LD L,L", 4, CPU::op_0x6D)),
            (0x6E, OpCode::new("LD L,(HL)", 8, CPU::op_0x6E)),
            (0x6F, OpCode::new("LD L,A", 4, CPU::op_0x6F)),
            (0x70, OpCode::new("LD (HL),B", 8, CPU::op_0x70)),
            (0x71, OpCode::new("LD (HL),C", 8, CPU::op_0x71)),
            (0x72, OpCode::new("LD (HL),D", 8, CPU::op_0x72)),
            (0x73, OpCode::new("LD (HL),E", 8, CPU::op_0x73)),
            (0x74, OpCode::new("LD (HL),H", 8, CPU::op_0x74)),
            (0x75, OpCode::new("LD (HL),L", 8, CPU::op_0x75)),
            (0x76, OpCode::new("HALT", 4, CPU::op_0x76)),
            (0x77, OpCode::new("LD (HL),A", 8, CPU::op_0x77)),
            (0x78, OpCode::new("LD A,B", 4, CPU::op_0x78)),
            (0x79, OpCode::new("LD A,C", 4, CPU::op_0x79)),
            (0x7A, OpCode::new("LD A,D", 4, CPU::op_0x7A)),
            (0x7B, OpCode::new("LD A,E", 4, CPU::op_0x7B)),
            (0x7C, OpCode::new("LD A,H", 4, CPU::op_0x7C)),
            (0x7D, OpCode::new("LD A,L", 4, CPU::op_0x7D)),
            (0x7E, OpCode::new("LD A,(HL)", 8, CPU::op_0x7E)),
            (0x7F, OpCode::new("LD A,A", 4, CPU::op_0x7F)),
            (0x80, OpCode::new("ADD A,B", 4, CPU::op_0x80)),
            (0x81, OpCode::new("ADD A,C", 4, CPU::op_0x81)),
            (0x82, OpCode::new("ADD A,D", 4, CPU::op_0x82)),
            (0x83, OpCode::new("ADD A,E", 4, CPU::op_0x83)),
            (0x84, OpCode::new("ADD A,H", 4, CPU::op_0x84)),
            (0x85, OpCode::new("ADD A,L", 4, CPU::op_0x85)),
            (0x86, OpCode::new("ADD A,(HL)", 8, CPU::op_0x86)),
            (0x87, OpCode::new("ADD A,A", 4, CPU::op_0x87)),
            (0x88, OpCode::new("ADC A,B", 4, CPU::op_0x88)),
            (0x89, OpCode::new("ADC A,C", 4, CPU::op_0x89)),
            (0x8A, OpCode::new("ADC A,D", 4, CPU::op_0x8A)),
            (0x8B, OpCode::new("ADC A,E", 4, CPU::op_0x8B)),
            (0x8C, OpCode::new("ADC A,H", 4, CPU::op_0x8C)),
            (0x8D, OpCode::new("ADC A,L", 4, CPU::op_0x8D)),
            (0x8E, OpCode::new("ADC A,(HL)", 8, CPU::op_0x8E)),
            (0x8F, OpCode::new("ADC A,A", 4, CPU::op_0x8F)),
            (0x90, OpCode::new("SUB B", 4, CPU::op_0x90)),
            (0x91, OpCode::new("SUB C", 4, CPU::op_0x91)),
            (0x92, OpCode::new("SUB D", 4, CPU::op_0x92)),
            (0x93, OpCode::new("SUB E", 4, CPU::op_0x93)),
            (0x94, OpCode::new("SUB H", 4, CPU::op_0x94)),
            (0x95, OpCode::new("SUB L", 4, CPU::op_0x95)),
            (0x96, OpCode::new("SUB (HL)", 8, CPU::op_0x96)),
            (0x97, OpCode::new("SUB A", 4, CPU::op_0x97)),
            (0x98, OpCode::new("SBC A,B", 4, CPU::op_0x98)),
            (0x99, OpCode::new("SBC A,C", 4, CPU::op_0x99)),
            (0x9A, OpCode::new("SBC A,D", 4, CPU::op_0x9A)),
            (0x9B, OpCode::new("SBC A,E", 4, CPU::op_0x9B)),
            (0x9C, OpCode::new("SBC A,H", 4, CPU::op_0x9C)),
            (0x9D, OpCode::new("SBC A,L", 4, CPU::op_0x9D)),
            (0x9E, OpCode::new("SBC A,(HL)", 8, CPU::op_0x9E)),
            (0x9F, OpCode::new("SBC A,A", 4, CPU::op_0x9F)),
            (0xAF, OpCode::new("XOR A", 4, CPU::op_0xAF)),
            (0xA0, OpCode::new("AND B", 4, CPU::op_0xA0)),
            (0xA1, OpCode::new("AND C", 4, CPU::op_0xA1)),
            (0xA2, OpCode::new("AND D", 4, CPU::op_0xA2)),
            (0xA3, OpCode::new("AND E", 4, CPU::op_0xA3)),
            (0xA4, OpCode::new("AND H", 4, CPU::op_0xA4)),
            (0xA5, OpCode::new("AND L", 4, CPU::op_0xA5)),
            (0xA6, OpCode::new("AND (HL)", 8, CPU::op_0xA6)),
            (0xA7, OpCode::new("AND A", 4, CPU::op_0xA7)),
            (0xA8, OpCode::new("XOR B", 4, CPU::op_0xA8)),
            (0xA9, OpCode::new("XOR C", 4, CPU::op_0xA9)),
            (0xAA, OpCode::new("XOR D", 4, CPU::op_0xAA)),
            (0xAB, OpCode::new("XOR E", 4, CPU::op_0xAB)),
            (0xAC, OpCode::new("XOR H", 4, CPU::op_0xAC)),
            (0xAD, OpCode::new("XOR L", 4, CPU::op_0xAD)),
            (0xAE, OpCode::new("XOR (HL)", 8, CPU::op_0xAE)),
            (0xB0, OpCode::new("OR B", 4, CPU::op_0xB0)),
            (0xB1, OpCode::new("OR C", 4, CPU::op_0xB1)),
            (0xB2, OpCode::new("OR D", 4, CPU::op_0xB2)),
            (0xB3, OpCode::new("OR E", 4, CPU::op_0xB3)),
            (0xB4, OpCode::new("OR H", 4, CPU::op_0xB4)),
            (0xB5, OpCode::new("OR L", 4, CPU::op_0xB5)),
            (0xB6, OpCode::new("OR (HL)", 8, CPU::op_0xB6)),
            (0xB7, OpCode::new("OR A", 4, CPU::op_0xB7)),
            (0xB8, OpCode::new("CP B", 4, CPU::op_0xB8)),
            (0xB9, OpCode::new("CP C", 4, CPU::op_0xB9)),
            (0xBA, OpCode::new("CP D", 4, CPU::op_0xBA)),
            (0xBB, OpCode::new("CP E", 4, CPU::op_0xBB)),
            (0xBC, OpCode::new("CP H", 4, CPU::op_0xBC)),
            (0xBD, OpCode::new("CP L", 4, CPU::op_0xBD)),
            (0xBE, OpCode::new("CP (HL)", 8, CPU::op_0xBE)),
            (0xBF, OpCode::new("CP A", 4, CPU::op_0xBF)),
            (0xC0, OpCode::new("RET NZ", 8, CPU::op_0xC0)),
            (0xC1, OpCode::new("POP BC", 12, CPU::op_0xC1)),
            (0xC2, OpCode::new("JP NZ,a16", 12, CPU::op_0xC2)),
            (0xC3, OpCode::new("JP a16", 16, CPU::op_0xC3)),
            (0xC4, OpCode::new("CALL NZ,a16", 12, CPU::op_0xC4)),
            (0xC5, OpCode::new("PUSH BC", 16, CPU::op_0xC5)),
            (0xC6, OpCode::new("ADD A,d8", 8, CPU::op_0xC6)),
            (0xC7, OpCode::new("RST 00H", 16, CPU::op_0xC7)),
            (0xC8, OpCode::new("RET Z", 8, CPU::op_0xC8)),
            (0xC9, OpCode::new("RET", 16, CPU::op_0xC9)),
            (0xCA, OpCode::new("JP Z,a16", 12, CPU::op_0xCA)),
            (0xCB, OpCode::new("PERFIX CB", 4, CPU::op_0xCB)),
            (0xCC, OpCode::new("CALL Z,a16", 12, CPU::op_0xCC)),
            (0xCD, OpCode::new("CALL a16", 24, CPU::op_0xCD)),
            (0xCE, OpCode::new("ADC A,d8", 8, CPU::op_0xCE)),
            (0xCF, OpCode::new("RST 08H", 16, CPU::op_0xCF)),
            (0xD0, OpCode::new("RET NC", 8, CPU::op_0xD0)),
            (0xD1, OpCode::new("POP DE", 12, CPU::op_0xD1)),
            (0xD2, OpCode::new("JP NC,a16", 12, CPU::op_0xD2)),
            (0xD4, OpCode::new("CALL NC,a16", 12, CPU::op_0xD4)),
            (0xD5, OpCode::new("PUSH DE", 16, CPU::op_0xD5)),
            (0xD6, OpCode::new("SUB d8", 8, CPU::op_0xD6)),
            (0xD7, OpCode::new("RST 10H", 16, CPU::op_0xD7)),
            (0xD8, OpCode::new("RET C", 8, CPU::op_0xD8)),
            (0xD9, OpCode::new("RETI", 16, CPU::op_0xD9)),
            (0xDA, OpCode::new("JP C,a16", 12, CPU::op_0xDA)),
            (0xDC, OpCode::new("CALL C,a16", 12, CPU::op_0xDC)),
            (0xDE, OpCode::new("SBC A,d8", 8, CPU::op_0xDE)),
            (0xDF, OpCode::new("RST 18H", 16, CPU::op_0xDF)),
            (0xE0, OpCode::new("LDH (a8),A", 12, CPU::op_0xE0)),
            (0xE1, OpCode::new("POP HL", 12, CPU::op_0xE1)),
            (0xE2, OpCode::new("LD (C),A", 8, CPU::op_0xE2)),
            (0xE5, OpCode::new("PUSH HL", 16, CPU::op_0xE5)),
            (0xE6, OpCode::new("AND d8", 8, CPU::op_0xE6)),
            (0xE7, OpCode::new("RST 20H", 16, CPU::op_0xE7)),
            (0xE8, OpCode::new("ADD SP,r8", 16, CPU::op_0xE8)),
            (0xE9, OpCode::new("JP (HL)", 4, CPU::op_0xE9)),
            (0xEA, OpCode::new("LD (a16),A", 16, CPU::op_0xEA)),
            (0xEE, OpCode::new("XOR d8", 8, CPU::op_0xEE)),
            (0xEF, OpCode::new("RST 28H", 16, CPU::op_0xEF)),
            (0xF0, OpCode::new("LDH A,(a8)", 12, CPU::op_0xF0)),
            (0xF1, OpCode::new("POP AF", 12, CPU::op_0xF1)),
            (0xF2, OpCode::new("LD A,(C)", 8, CPU::op_0xF2)),
            (0xF3, OpCode::new("DI", 4, CPU::op_0xF3)),
            (0xF5, OpCode::new("PUSH AF", 16, CPU::op_0xF5)),
            (0xF6, OpCode::new("OR d8", 8, CPU::op_0xF6)),
            (0xF7, OpCode::new("RST 30H", 16, CPU::op_0xF7)),
            (0xF8, OpCode::new("LD HL,SP+r8", 12, CPU::op_0xF8)),
            (0xF9, OpCode::new("LD SP,HL", 8, CPU::op_0xF9)),
            (0xFA, OpCode::new("LD A,(a16)", 16, CPU::op_0xFA)),
            (0xFB, OpCode::new("EI", 4, CPU::op_0xFB)),
            (0xFE, OpCode::new("CP d8", 8, CPU::op_0xFE)),
            (0xFF, OpCode::new("RST 38H", 16, CPU::op_0xFF)),
        ]))
    };
}
