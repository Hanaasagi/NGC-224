#![allow(non_snake_case)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use NGC224::gameboy::IOHandler;
use NGC224::gameboy::Register;
use NGC224::gameboy::CPU;

struct FakeMemory<'a> {
    records: Vec<&'a str>,
    data: HashMap<u16, u16>,
}

impl<'a> FakeMemory<'a> {
    fn new() -> FakeMemory<'a> {
        Self {
            records: vec![],
            data: HashMap::new(),
        }
    }

    fn fake_data(&mut self, a: u16, v: u16) {
        self.data.insert(a, v);
    }
}

impl IOHandler for FakeMemory<'_> {
    fn read_byte(&self, a: u16) -> u8 {
        *self.data.get(&a).unwrap() as u8
    }

    fn write_byte(&mut self, a: u16, v: u8) {
        self.data.insert(a, v as u16);
    }

    fn read_word(&self, a: u16) -> u16 {
        *self.data.get(&a).unwrap()
    }

    fn write_word(&mut self, a: u16, v: u16) {
        self.data.insert(a, v);
    }
}

#[test]
fn test_opcode_0X00() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 176, h: 1, l: 77, pc: 257, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(257, 195);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x00();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 176, h: 1, l: 77, pc: 257, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0X01() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 192, l: 0, pc: 8062, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(8062, 8192);
    mem.borrow_mut().fake_data(8064, 54);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x01();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 160, h: 192, l: 0, pc: 8064, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0X04() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 1, c: 104, d: 0, e: 0, f: 128, h: 69, l: 56, pc: 6419, sp: 57327 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(6419, 33);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x04();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 2, c: 104, d: 0, e: 0, f: 0, h: 69, l: 56, pc: 6419, sp: 57327 }"
    );
}
#[test]
fn test_opcode_0X05() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 160, c: 0, d: 0, e: 216, f: 128, h: 195, l: 1, pc: 138, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(138, 32);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x05();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 159, c: 0, d: 0, e: 216, f: 96, h: 195, l: 1, pc: 138, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X06() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 0, d: 0, e: 216, f: 128, h: 195, l: 0, pc: 135, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(135, 160);
    mem.borrow_mut().fake_data(136, 34);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x06();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 160, c: 0, d: 0, e: 216, f: 128, h: 195, l: 0, pc: 136, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X09() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 2, b: 0, c: 0, d: 0, e: 4, f: 192, h: 192, l: 38, pc: 20747, sp: 57323 }",
    );
    mem.borrow_mut().fake_data(20747, 126);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x09();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 2, b: 0, c: 0, d: 0, e: 4, f: 128, h: 192, l: 38, pc: 20747, sp: 57323 }"
    );
}
#[test]
fn test_opcode_0X0B() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 160, h: 192, l: 1, pc: 8068, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(8068, 120);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x0B();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 31, c: 255, d: 0, e: 216, f: 160, h: 192, l: 1, pc: 8068, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0X0C() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 62, b: 10, c: 128, d: 0, e: 216, f: 192, h: 75, l: 252, pc: 19447, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(19447, 5);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x0C();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 62, b: 10, c: 129, d: 0, e: 216, f: 0, h: 75, l: 252, pc: 19447, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X0D() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 9, d: 0, e: 0, f: 192, h: 96, l: 139, pc: 24710, sp: 57335 }",
    );
    mem.borrow_mut().fake_data(24710, 32);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x0D();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 8, d: 0, e: 0, f: 64, h: 96, l: 139, pc: 24710, sp: 57335 }"
    );
}
#[test]
fn test_opcode_0X0E() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 0, d: 0, e: 216, f: 192, h: 195, l: 160, pc: 19438, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(19438, 128);
    mem.borrow_mut().fake_data(19439, 6);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x0E();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 128, d: 0, e: 216, f: 192, h: 195, l: 160, pc: 19439, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X11() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 127, b: 0, c: 138, d: 0, e: 216, f: 128, h: 152, l: 5, pc: 7414, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(7414, 1024);
    mem.borrow_mut().fake_data(7416, 107);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x11();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 127, b: 0, c: 138, d: 4, e: 0, f: 128, h: 152, l: 5, pc: 7416, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X12() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 128, c: 16, d: 136, e: 0, f: 32, h: 111, l: 233, pc: 24974, sp: 57331 }",
    );
    mem.borrow_mut().fake_data(24974, 19);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x12();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 128, c: 16, d: 136, e: 0, f: 32, h: 111, l: 233, pc: 24974, sp: 57331 }"
    );
}
#[test]
fn test_opcode_0X13() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 28, b: 0, c: 138, d: 127, e: 57, f: 0, h: 127, l: 57, pc: 32371, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32371, 26);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x13();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 28, b: 0, c: 138, d: 127, e: 58, f: 0, h: 127, l: 57, pc: 32371, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X15() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 127, b: 0, c: 138, d: 4, e: 0, f: 192, h: 153, l: 0, pc: 7422, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(7422, 32);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x15();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 127, b: 0, c: 138, d: 3, e: 0, f: 64, h: 153, l: 0, pc: 7422, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X16() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 255, c: 138, d: 0, e: 0, f: 128, h: 160, l: 0, pc: 23148, sp: 57333 }",
    );
    mem.borrow_mut().fake_data(23148, 160);
    mem.borrow_mut().fake_data(23149, 33);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x16();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 255, c: 138, d: 160, e: 0, f: 128, h: 160, l: 0, pc: 23149, sp: 57333 }"
    );
}
#[test]
fn test_opcode_0X18() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 342, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(342, 2);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(345, 234);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x18();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 345, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0X19() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 192, b: 0, c: 138, d: 0, e: 192, f: 0, h: 126, l: 121, pc: 32364, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32364, 84);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x19();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 192, b: 0, c: 138, d: 0, e: 192, f: 0, h: 127, l: 57, pc: 32364, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X1A() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 192, b: 0, c: 138, d: 127, e: 57, f: 0, h: 127, l: 57, pc: 32367, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(32569, 28);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32367, 234);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x1A();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 28, b: 0, c: 138, d: 127, e: 57, f: 0, h: 127, l: 57, pc: 32367, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X1B() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 138, d: 27, e: 88, f: 128, h: 101, l: 8, pc: 24913, sp: 57329 }",
    );
    mem.borrow_mut().fake_data(24913, 122);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x1B();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 138, d: 27, e: 87, f: 128, h: 101, l: 8, pc: 24913, sp: 57329 }"
    );
}
#[test]
fn test_opcode_0X1D() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 127, b: 0, c: 138, d: 4, e: 0, f: 128, h: 152, l: 1, pc: 7419, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(7419, 32);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x1D();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 127, b: 0, c: 138, d: 4, e: 255, f: 96, h: 152, l: 1, pc: 7419, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X1E() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 48, b: 16, c: 138, d: 62, e: 141, f: 128, h: 100, l: 248, pc: 24575, sp: 57331 }",
    );
    mem.borrow_mut().fake_data(24575, 8);
    mem.borrow_mut().fake_data(24576, 42);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x1E();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 48, b: 16, c: 138, d: 62, e: 8, f: 128, h: 100, l: 248, pc: 24576, sp: 57331 }"
    );
}
#[test]
fn test_opcode_0X20() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 112, h: 1, l: 77, pc: 112, sp: 65532 }",
    );
    mem.borrow_mut().fake_data(112, 250);
    mem.borrow_mut().fake_data(107, 240);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x20();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 112, h: 1, l: 77, pc: 107, sp: 65532 }"
    );
}
#[test]
fn test_opcode_0X21() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 8059, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(8059, 49152);
    mem.borrow_mut().fake_data(8061, 1);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x21();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 192, l: 0, pc: 8061, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0X22() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 0, pc: 14052, sp: 57339 }",
    );
    mem.borrow_mut().fake_data(14052, 11);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x22();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 1, pc: 14052, sp: 57339 }"
    );
}
#[test]
fn test_opcode_0X23() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 160, h: 192, l: 0, pc: 8067, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(8067, 11);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x23();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 160, h: 192, l: 1, pc: 8067, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0X24() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 5, c: 0, d: 0, e: 0, f: 176, h: 156, l: 0, pc: 7638, sp: 50240 }",
    );
    mem.borrow_mut().fake_data(7638, 5);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x24();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 5, c: 0, d: 0, e: 0, f: 16, h: 157, l: 0, pc: 7638, sp: 50240 }"
    );
}
#[test]
fn test_opcode_0X26() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 255, b: 0, c: 138, d: 0, e: 216, f: 128, h: 76, l: 5, pc: 8127, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(8127, 152);
    mem.borrow_mut().fake_data(8128, 205);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x26();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 255, b: 0, c: 138, d: 0, e: 216, f: 128, h: 152, l: 5, pc: 8128, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0X28() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 80, h: 1, l: 77, pc: 339, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(339, 3);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(340, 175);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x28();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 80, h: 1, l: 77, pc: 340, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0X29() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 12, b: 12, c: 0, d: 62, e: 141, f: 80, h: 0, l: 12, pc: 24051, sp: 57329 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(24051, 17);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x29();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 12, b: 12, c: 0, d: 62, e: 141, f: 0, h: 0, l: 24, pc: 24051, sp: 57329 }"
    );
}
#[test]
fn test_opcode_0X2A() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 10, c: 128, d: 0, e: 216, f: 192, h: 75, l: 251, pc: 19445, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(19451, 62);
    mem.borrow_mut().fake_data(19445, 226);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x2A();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 62, b: 10, c: 128, d: 0, e: 216, f: 192, h: 75, l: 252, pc: 19445, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X2C() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 6, c: 0, d: 0, e: 0, f: 160, h: 156, l: 0, pc: 7585, sp: 50082 }",
    );
    mem.borrow_mut().fake_data(7585, 114);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x2C();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 6, c: 0, d: 0, e: 0, f: 0, h: 156, l: 1, pc: 7585, sp: 50082 }"
    );
}
#[test]
fn test_opcode_0X2F() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 47, b: 2, c: 0, d: 25, e: 108, f: 160, h: 77, l: 238, pc: 370, sp: 57315 }",
    );
    mem.borrow_mut().fake_data(370, 230);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x2F();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 208, b: 2, c: 0, d: 25, e: 108, f: 224, h: 77, l: 238, pc: 370, sp: 57315 }"
    );
}
#[test]
fn test_opcode_0X30() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 192, b: 0, c: 138, d: 0, e: 192, f: 0, h: 126, l: 121, pc: 32361, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(32361, 1);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32363, 25);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x30();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 192, b: 0, c: 138, d: 0, e: 192, f: 0, h: 126, l: 121, pc: 32363, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X31() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 8056, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(8056, 57343);
    mem.borrow_mut().fake_data(8058, 33);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x31();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 8058, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0X36() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 160, h: 192, l: 0, pc: 8065, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(8065, 0);
    mem.borrow_mut().fake_data(8066, 35);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x36();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 160, h: 192, l: 0, pc: 8066, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0X37() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 128, h: 100, l: 248, pc: 24833, sp: 57335 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(24833, 201);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x37();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 144, h: 100, l: 248, pc: 24833, sp: 57335 }"
    );
}
#[test]
fn test_opcode_0X3C() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 128, b: 20, c: 13, d: 0, e: 12, f: 192, h: 152, l: 1, pc: 24886, sp: 57333 }",
    );
    mem.borrow_mut().fake_data(24886, 5);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x3C();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 129, b: 20, c: 13, d: 0, e: 12, f: 0, h: 152, l: 1, pc: 24886, sp: 57333 }"
    );
}
#[test]
fn test_opcode_0X3D() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 128, h: 160, l: 0, pc: 8225, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(8225, 195);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x3D();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 255, b: 0, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 8225, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X3E() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 8049, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(8049, 128);
    mem.borrow_mut().fake_data(8050, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x3E();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 128, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 8050, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0X42() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 255, c: 138, d: 160, e: 0, f: 128, h: 192, l: 6, pc: 23178, sp: 57331 }",
    );
    mem.borrow_mut().fake_data(23178, 34);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x42();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 160, c: 138, d: 160, e: 0, f: 128, h: 192, l: 6, pc: 23178, sp: 57331 }"
    );
}
#[test]
fn test_opcode_0X44() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 80, b: 0, c: 40, d: 69, e: 135, f: 192, h: 196, l: 143, pc: 6492, sp: 57327 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(6492, 77);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x44();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 80, b: 196, c: 40, d: 69, e: 135, f: 192, h: 196, l: 143, pc: 6492, sp: 57327 }"
    );
}
#[test]
fn test_opcode_0X47() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 103, sp: 65532 }",
    );
    mem.borrow_mut().fake_data(103, 203);
    mem.borrow_mut().fake_data(104, 135);
    mem.borrow_mut().fake_data(105, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x47();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 103, sp: 65532 }"
    );
}
#[test]
fn test_opcode_0X4D() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 80, b: 196, c: 40, d: 69, e: 135, f: 192, h: 196, l: 143, pc: 6493, sp: 57327 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(6493, 225);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x4D();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 80, b: 196, c: 143, d: 69, e: 135, f: 192, h: 196, l: 143, pc: 6493, sp: 57327 }"
    );
}
#[test]
fn test_opcode_0X4F() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 12, c: 0, d: 62, e: 141, f: 0, h: 86, l: 130, pc: 16044, sp: 57327 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(16044, 201);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x4F();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 12, c: 0, d: 62, e: 141, f: 0, h: 86, l: 130, pc: 16044, sp: 57327 }"
    );
}
#[test]
fn test_opcode_0X54() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 192, b: 0, c: 138, d: 0, e: 192, f: 0, h: 127, l: 57, pc: 32365, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32365, 93);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x54();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 192, b: 0, c: 138, d: 127, e: 192, f: 0, h: 127, l: 57, pc: 32365, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X57() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 0, pc: 14050, sp: 57339 }",
    );
    mem.borrow_mut().fake_data(14050, 122);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x57();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 0, pc: 14050, sp: 57339 }"
    );
}
#[test]
fn test_opcode_0X5D() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 192, b: 0, c: 138, d: 127, e: 192, f: 0, h: 127, l: 57, pc: 32366, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32366, 26);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x5D();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 192, b: 0, c: 138, d: 127, e: 57, f: 0, h: 127, l: 57, pc: 32366, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X5F() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 64, b: 0, c: 138, d: 0, e: 0, f: 192, h: 126, l: 121, pc: 32357, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32357, 135);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x5F();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 64, b: 0, c: 138, d: 0, e: 64, f: 192, h: 126, l: 121, pc: 32357, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X66() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 40, b: 0, c: 9, d: 0, e: 0, f: 160, h: 96, l: 138, pc: 24702, sp: 57331 }",
    );
    mem.borrow_mut().fake_data(24714, 101);
    mem.borrow_mut().fake_data(24702, 111);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x66();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 40, b: 0, c: 9, d: 0, e: 0, f: 160, h: 101, l: 138, pc: 24702, sp: 57331 }"
    );
}
#[test]
fn test_opcode_0X67() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 96, b: 0, c: 138, d: 127, e: 59, f: 0, h: 127, l: 43, pc: 32376, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32376, 201);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x67();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 96, b: 0, c: 138, d: 127, e: 59, f: 0, h: 96, l: 43, pc: 32376, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X6B() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 127, b: 0, c: 138, d: 4, e: 0, f: 128, h: 152, l: 5, pc: 7417, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(7417, 34);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x6B();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 127, b: 0, c: 138, d: 4, e: 0, f: 128, h: 152, l: 0, pc: 7417, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0X6F() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 43, b: 0, c: 138, d: 127, e: 58, f: 0, h: 127, l: 57, pc: 32373, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32373, 19);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x6F();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 43, b: 0, c: 138, d: 127, e: 58, f: 0, h: 127, l: 43, pc: 32373, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X71() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 192, h: 204, l: 84, pc: 32347, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32347, 33);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x71();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 192, h: 204, l: 84, pc: 32347, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X72() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 6, c: 0, d: 0, e: 0, f: 0, h: 156, l: 1, pc: 7586, sp: 50082 }",
    );
    mem.borrow_mut().fake_data(7586, 44);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x72();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 6, c: 0, d: 0, e: 0, f: 0, h: 156, l: 1, pc: 7586, sp: 50082 }"
    );
}
#[test]
fn test_opcode_0X73() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 6, c: 0, d: 0, e: 0, f: 160, h: 156, l: 0, pc: 7584, sp: 50082 }",
    );
    mem.borrow_mut().fake_data(7584, 44);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x73();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 6, c: 0, d: 0, e: 0, f: 160, h: 156, l: 0, pc: 7584, sp: 50082 }"
    );
}
#[test]
fn test_opcode_0X76() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 3, d: 0, e: 0, f: 192, h: 197, l: 8, pc: 8372, sp: 57325 }",
    );

    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x76();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 3, d: 0, e: 0, f: 192, h: 197, l: 8, pc: 8372, sp: 57325 }"
    );
    assert_eq!(cpu.is_halt(), true);
}
#[test]
fn test_opcode_0X77() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 160, b: 40, c: 138, d: 0, e: 4, f: 192, h: 195, l: 0, pc: 152, sp: 57323 }",
    );
    mem.borrow_mut().fake_data(152, 25);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x77();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 160, b: 40, c: 138, d: 0, e: 4, f: 192, h: 195, l: 0, pc: 152, sp: 57323 }"
    );
}
#[test]
fn test_opcode_0X78() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 120, sp: 65532 }",
    );
    mem.borrow_mut().fake_data(120, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x78();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 120, sp: 65532 }"
    );
}
#[test]
fn test_opcode_0X79() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 0, d: 0, e: 4, f: 160, h: 192, l: 38, pc: 20785, sp: 57323 }",
    );
    mem.borrow_mut().fake_data(20786, 12);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x79();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 0, d: 0, e: 4, f: 160, h: 192, l: 38, pc: 20785, sp: 57323 }"
    );
}
#[test]
fn test_opcode_0X7A() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 0, pc: 14051, sp: 57339 }",
    );
    mem.borrow_mut().fake_data(14051, 34);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x7A();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 0, pc: 14051, sp: 57339 }"
    );
}
#[test]
fn test_opcode_0X7B() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 192, h: 204, l: 82, pc: 32343, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32343, 34);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x7B();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 192, h: 204, l: 82, pc: 32343, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X7C() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 19, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 32330, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32330, 234);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x7C();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 160, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 32330, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X7D() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 160, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 32334, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32334, 234);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x7D();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 32334, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X7E() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 138, d: 62, e: 141, f: 128, h: 100, l: 248, pc: 24556, sp: 57333 }",
    );
    mem.borrow_mut().fake_data(25848, 137);
    mem.borrow_mut().fake_data(24556, 230);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x7E();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 137, b: 0, c: 138, d: 62, e: 141, f: 128, h: 100, l: 248, pc: 24556, sp: 57333 }"
    );
}
#[test]
fn test_opcode_0X83() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 128, b: 0, c: 138, d: 0, e: 64, f: 0, h: 126, l: 121, pc: 32359, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32359, 95);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x83();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 192, b: 0, c: 138, d: 0, e: 64, f: 0, h: 126, l: 121, pc: 32359, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X85() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 13, b: 6, c: 0, d: 0, e: 0, f: 0, h: 156, l: 19, pc: 7634, sp: 50100 }",
    );
    mem.borrow_mut().fake_data(7634, 111);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x85();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 32, b: 6, c: 0, d: 0, e: 0, f: 32, h: 156, l: 19, pc: 7634, sp: 50100 }"
    );
}
#[test]
fn test_opcode_0X87() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 64, b: 0, c: 138, d: 0, e: 64, f: 192, h: 126, l: 121, pc: 32358, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(32358, 131);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x87();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 128, b: 0, c: 138, d: 0, e: 64, f: 0, h: 126, l: 121, pc: 32358, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0X88() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 130, c: 228, d: 0, e: 4, f: 192, h: 122, l: 143, pc: 31381, sp: 57311 }",
    );
    mem.borrow_mut().fake_data(31381, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x88();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 130, b: 130, c: 228, d: 0, e: 4, f: 0, h: 122, l: 143, pc: 31381, sp: 57311 }"
    );
}
#[test]
fn test_opcode_0X98() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 130, c: 228, d: 0, e: 4, f: 0, h: 122, l: 143, pc: 31389, sp: 57311 }",
    );
    mem.borrow_mut().fake_data(31389, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0x98();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 126, b: 130, c: 228, d: 0, e: 4, f: 112, h: 122, l: 143, pc: 31389, sp: 57311 }"
    );
}
#[test]
fn test_opcode_0XA7() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 255, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 9145, sp: 57335 }",
    );
    mem.borrow_mut().fake_data(9145, 40);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xA7();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 255, c: 138, d: 0, e: 0, f: 160, h: 160, l: 0, pc: 9145, sp: 57335 }"
    );
}
#[test]
fn test_opcode_0XAF() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 80, h: 1, l: 77, pc: 341, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(341, 24);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xAF();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 341, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0XB0() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 0, d: 25, e: 108, f: 160, h: 77, l: 238, pc: 403, sp: 57315 }",
    );
    mem.borrow_mut().fake_data(403, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xB0();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 0, d: 25, e: 108, f: 128, h: 77, l: 238, pc: 403, sp: 57315 }"
    );
}
#[test]
fn test_opcode_0XB1() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 31, b: 31, c: 255, d: 0, e: 216, f: 160, h: 192, l: 1, pc: 8070, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(8070, 32);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xB1();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 255, b: 31, c: 255, d: 0, e: 216, f: 0, h: 192, l: 1, pc: 8070, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0XB3() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 27, b: 0, c: 138, d: 27, e: 87, f: 128, h: 101, l: 8, pc: 24915, sp: 57329 }",
    );
    mem.borrow_mut().fake_data(24915, 32);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xB3();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 95, b: 0, c: 138, d: 27, e: 87, f: 0, h: 101, l: 8, pc: 24915, sp: 57329 }"
    );
}
#[test]
fn test_opcode_0XC0() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 255, b: 0, c: 138, d: 0, e: 0, f: 192, h: 101, l: 8, pc: 19224, sp: 57323 }",
    );
    mem.borrow_mut().fake_data(19224, 234);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xC0();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 255, b: 0, c: 138, d: 0, e: 0, f: 192, h: 101, l: 8, pc: 19224, sp: 57323 }"
    );
}
#[test]
fn test_opcode_0XC1() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 138, d: 24, e: 0, f: 192, h: 192, l: 206, pc: 9254, sp: 57335 }",
    );
    mem.borrow_mut().fake_data(57335, 138);
    mem.borrow_mut().fake_data(9254, 209);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xC1();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 138, d: 24, e: 0, f: 192, h: 192, l: 206, pc: 9254, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0XC3() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 176, h: 1, l: 77, pc: 258, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(258, 336);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(336, 254);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xC3();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 176, h: 1, l: 77, pc: 336, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0XC5() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 255, b: 0, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 9140, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(9140, 71);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xC5();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 255, b: 0, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 9140, sp: 57335 }"
    );
}
#[test]
fn test_opcode_0XC8() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 138, d: 62, e: 141, f: 32, h: 100, l: 248, pc: 24559, sp: 57333 }",
    );
    mem.borrow_mut().fake_data(24559, 71);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xC8();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 138, d: 62, e: 141, f: 32, h: 100, l: 248, pc: 24559, sp: 57333 }"
    );
}
#[test]
fn test_opcode_0XC9() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 123, sp: 65532 }",
    );
    mem.borrow_mut().fake_data(65532, 8055);
    mem.borrow_mut().fake_data(8055, 49);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xC9();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 8055, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0XCA() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 255, b: 255, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 22652, sp: 57333 }",
    );
    mem.borrow_mut().fake_data(22652, 23092);
    mem.borrow_mut().fake_data(23092, 62);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xCA();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 255, b: 255, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 23092, sp: 57333 }"
    );
}
#[test]
fn test_opcode_0XCC() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 2, c: 192, d: 0, e: 4, f: 32, h: 77, l: 238, pc: 8352, sp: 57325 }",
    );
    mem.borrow_mut().fake_data(8352, 351);
    mem.borrow_mut().fake_data(8354, 250);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xCC();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 2, c: 192, d: 0, e: 4, f: 32, h: 77, l: 238, pc: 8354, sp: 57325 }"
    );
}
#[test]
fn test_opcode_0XCD() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 128, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 8053, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(8053, 97);
    mem.borrow_mut().fake_data(97, 175);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xCD();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 128, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 97, sp: 65532 }"
    );
}
#[test]
fn test_opcode_0XD0() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 144, h: 100, l: 248, pc: 24627, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(24627, 62);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xD0();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 138, d: 0, e: 0, f: 144, h: 100, l: 248, pc: 24627, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0XD1() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 0, d: 0, e: 216, f: 128, h: 160, l: 0, pc: 14058, sp: 57339 }",
    );
    mem.borrow_mut().fake_data(57339, 216);
    mem.borrow_mut().fake_data(14058, 201);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xD1();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 0, d: 0, e: 216, f: 128, h: 160, l: 0, pc: 14058, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0XD5() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 0, pc: 14049, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(14049, 87);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xD5();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 32, c: 0, d: 0, e: 216, f: 128, h: 128, l: 0, pc: 14049, sp: 57339 }"
    );
}
#[test]
fn test_opcode_0XD6() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 32, b: 4, c: 32, d: 98, e: 136, f: 160, h: 150, l: 0, pc: 6274, sp: 57325 }",
    );
    mem.borrow_mut().fake_data(6274, 8);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(6275, 79);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xD6();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 24, b: 4, c: 32, d: 98, e: 136, f: 96, h: 150, l: 0, pc: 6275, sp: 57325 }"
    );
}
#[test]
fn test_opcode_0XD9() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 101, l: 8, pc: 8368, sp: 57333 }",
    );
    mem.borrow_mut().fake_data(57333, 24743);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(24743, 205);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xD9();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 101, l: 8, pc: 24743, sp: 57335 }"
    );
}
#[test]
fn test_opcode_0XE0() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 8023, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(8023, 15);
    mem.borrow_mut().fake_data(8024, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xE0();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 8024, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0XE1() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 192, l: 206, pc: 9256, sp: 57339 }",
    );
    mem.borrow_mut().fake_data(57339, 40960);
    mem.borrow_mut().fake_data(9256, 201);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xE1();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 9256, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0XE2() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 62, b: 10, c: 128, d: 0, e: 216, f: 192, h: 75, l: 252, pc: 19446, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(19446, 12);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xE2();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 62, b: 10, c: 128, d: 0, e: 216, f: 192, h: 75, l: 252, pc: 19446, sp: 57341 }"
    );
}
#[test]
fn test_opcode_0XE5() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 255, b: 0, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 9138, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(9138, 213);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xE5();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 255, b: 0, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 9138, sp: 57339 }"
    );
}
#[test]
fn test_opcode_0XE6() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 128, b: 0, c: 19, d: 0, e: 216, f: 192, h: 1, l: 77, pc: 116, sp: 65532 }",
    );
    mem.borrow_mut().fake_data(116, 127);
    mem.borrow_mut().fake_data(117, 224);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xE6();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 160, h: 1, l: 77, pc: 117, sp: 65532 }"
    );
}
#[test]
fn test_opcode_0XE9() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 28, b: 0, c: 138, d: 62, e: 141, f: 0, h: 96, l: 43, pc: 16013, sp: 57337 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(24619, 175);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xE9();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 28, b: 0, c: 138, d: 62, e: 141, f: 0, h: 96, l: 43, pc: 24619, sp: 57337 }"
    );
}
#[test]
fn test_opcode_0XEA() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 346, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(346, 53018);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(348, 195);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xEA();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 348, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0XF0() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 101, sp: 65532 }",
    );
    mem.borrow_mut().fake_data(101, 255);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(102, 71);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xF0();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 102, sp: 65532 }"
    );
}
#[test]
fn test_opcode_0XF1() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 28, b: 0, c: 138, d: 0, e: 0, f: 32, h: 101, l: 8, pc: 8366, sp: 57331 }",
    );
    mem.borrow_mut().fake_data(57331, 448);
    mem.borrow_mut().fake_data(8366, 217);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xF1();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 101, l: 8, pc: 8366, sp: 57333 }"
    );
}
#[test]
fn test_opcode_0XF3() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 8021, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(8021, 175);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xF3();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 19, d: 0, e: 216, f: 128, h: 1, l: 77, pc: 8021, sp: 65534 }"
    );
}
#[test]
fn test_opcode_0XF5() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 15990, sp: 57341 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(15990, 62);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xF5();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 15990, sp: 57339 }"
    );
}
#[test]
fn test_opcode_0XF8() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 0, d: 25, e: 170, f: 32, h: 100, l: 216, pc: 7516, sp: 57309 }",
    );
    mem.borrow_mut().fake_data(7516, 0);
    mem.borrow_mut().fake_data(7517, 124);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xF8();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 0, d: 25, e: 170, f: 0, h: 223, l: 221, pc: 7517, sp: 57309 }"
    );
}
#[test]
fn test_opcode_0XF9() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 0, b: 0, c: 0, d: 25, e: 170, f: 160, h: 195, l: 160, pc: 7552, sp: 57309 }",
    );
    mem.borrow_mut().fake_data(7552, 240);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xF9();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 0, c: 0, d: 25, e: 170, f: 160, h: 195, l: 160, pc: 7552, sp: 50080 }"
    );
}
#[test]
fn test_opcode_0XFA() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 255, b: 255, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 9142, sp: 57335 }",
    );
    mem.borrow_mut().fake_data(9142, 49390);
    mem.borrow_mut().fake_data(49390, 0);
    mem.borrow_mut().fake_data(9144, 167);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xFA();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 0, b: 255, c: 138, d: 0, e: 0, f: 96, h: 160, l: 0, pc: 9144, sp: 57335 }"
    );
}
#[test]
fn test_opcode_0XFB() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 8148, sp: 57343 }",
    );
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 13);
    mem.borrow_mut().fake_data(8148, 62);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xFB();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 138, d: 0, e: 0, f: 192, h: 160, l: 0, pc: 8148, sp: 57343 }"
    );
}
#[test]
fn test_opcode_0XFE() {
    let mem = Rc::new(RefCell::new(FakeMemory::new()));
    let reg = Register::new_from_debug_string(
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 176, h: 1, l: 77, pc: 337, sp: 65534 }",
    );
    mem.borrow_mut().fake_data(337, 17);
    mem.borrow_mut().fake_data(65295, 0);
    mem.borrow_mut().fake_data(65535, 0);
    mem.borrow_mut().fake_data(338, 40);
    let mut cpu = CPU::new(mem, false);
    cpu.set_reg(reg);
    cpu.op_0xFE();
    assert_eq!(
        format!("{:?}", cpu.get_reg_snapshot()).to_lowercase(),
        "register { a: 1, b: 0, c: 19, d: 0, e: 216, f: 80, h: 1, l: 77, pc: 338, sp: 65534 }"
    );
}
