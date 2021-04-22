use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time;

use super::super::get_global_term;
use super::super::mmu::IOHandler;
use super::opcode_set::OP_CODE_SET;
use super::register::Flag;
use super::register::Register;
use crate::gameboy::debug::insert_cpu_record;
use crate::gameboy::debug::CPUDebugInfo;
use crate::gameboy::spec::{STEP_CYCLES, STEP_TIME};

pub struct CPU {
    pub reg: Register,
    // flag: FlagRegister,
    is_halt: bool,
    data_bus: Rc<RefCell<dyn IOHandler>>,

    /// The IME flag is used to disable all interrupts,
    /// overriding any enabled bits in the IE Register.
    /// It isn't possible to access the IME flag by using a I/O address,
    /// instead IME is accessed directly from the CPU, by the following opcodes/operations:
    /// ```ignore
    /// EI     ;Enable Interrupts  (ie. IME=1)
    /// DI     ;Disable Interrupts (ie. IME=0)
    /// RETI   ;Enable Ints & Return (same as the opcode combination EI, RET)
    /// <INT>  ;Disable Ints & Call to Interrupt Vector
    /// ```
    /// Whereas <INT> means the operation which is automatically executed by the CPU when it executes an interrupt.
    /// The effect of EI is delayed by one instruction. This means that EI followed immediately by DI does not allow interrupts between the EI and the DI.

    /// IME flag could be following value:
    /// 0 - Disable all Interrupts
    /// 1 - Enable all Interrupts that are enabled in IE Register (FFFF)
    ime_flag: bool,

    step_cycles: u32,
    step_zero: time::Instant,
    step_flip: bool,
    speed_simulation: bool,
}

impl CPU {
    pub fn new(data_bus: Rc<RefCell<dyn IOHandler>>, speed_simulation: bool) -> Self {
        let mut reg = Register::new();
        let term = get_global_term();
        reg.init(term);

        Self {
            reg,
            is_halt: false,
            data_bus,
            ime_flag: true,

            step_cycles: 0,
            step_zero: time::Instant::now(),
            step_flip: false,
            speed_simulation,
        }
    }

    pub fn get_reg_snapshot(&self) -> Register {
        self.reg.clone()
    }

    pub fn is_ime_enabled(&self) -> bool {
        self.ime_flag == true
    }

    pub fn enable_ime(&mut self) {
        self.ime_flag = true;
    }

    pub fn disable_ime(&mut self) {
        self.ime_flag = false;
    }

    pub fn is_halt(&self) -> bool {
        self.is_halt
    }
    fn imm(&mut self) -> u8 {
        let v = self.read_byte_from_memory(self.reg.get_PC());
        self.reg.incr_PC();
        v
    }

    fn imm_freeze(&self) -> u8 {
        self.read_byte_from_memory(self.reg.get_PC())
    }

    fn imm_word(&mut self) -> u16 {
        let v = self.read_word_from_memory(self.reg.get_PC());
        self.reg.incr_PC();
        self.reg.incr_PC();
        v
    }

    // pub fn get_reg(&self) -> Box<Register {
    //     Box::new(self.reg)
    // }

    // only for unittest!!
    // #[cfg(test)]
    pub fn set_reg(&mut self, reg: Register) {
        self.reg = reg;
    }
}

#[allow(non_snake_case)]
impl CPU {
    /*
        Execute given OPCode and return used CPU clock
    */
    // func (core *Core) ExecuteOPCode(code byte) int {
    // 	if OPCodeFunctionMap[code].Clock != 0 {
    // 		if core.DebugControl == core.CPU.Registers.PC-1 && core.Debug {
    // 			core.Break(code)
    // 			_, err := fmt.Scanf("%X", &core.DebugControl)
    // 			if err != nil {
    // 				core.CPU.Registers.PC = 0
    // 			}
    // 		}
    // 		var extCycles int
    // 		extCycles = OPCodeFunctionMap[code].Func(core)
    // 		return OPCodeFunctionMap[code].Clock + extCycles
    // 	} else {
    // 		if core.Debug {
    // 			core.Break(code)
    // 		}
    // 		log.Fatalf("Unable to resolve OPCode:%X   PC:%X\n", code, core.CPU.Registers.PC-1)
    // 		return 0
    // 	}
    // }

    fn hi(&mut self) -> u32 {
        if !self.is_halt && !self.is_ime_enabled() {
            return 0;
        }
        let intf = self.read_byte_from_memory(0xff0f);
        let inte = self.read_byte_from_memory(0xffff);
        let ii = intf & inte;
        if ii == 0x00 {
            return 0;
        }
        self.is_halt = false;
        if !self.is_ime_enabled() {
            return 0;
        }
        self.disable_ime();

        // Consumer an interrupter, the rest is written back to the register
        let n = ii.trailing_zeros();
        let intf = intf & !(1 << n);
        self.write_byte_to_memory(0xff0f, intf);

        self._stack_push(self.reg.get_PC());
        // Set the PC to correspond interrupt process program:
        // V-Blank: 0x40
        // LCD: 0x48
        // TIMER: 0x50
        // JOYPAD: 0x60
        // Serial: 0x58
        self.reg.set_PC(0x0040 | ((n as u16) << 3));
        4
    }

    pub fn _next(&mut self) -> u32 {
        let cycles = {
            let c = self.hi();
            if c != 0 {
                c * 4
            } else if self.is_halt {
                4
            } else {
                self.execute_opcode()
            }
        };
        cycles
    }

    fn down_frequency(&mut self) {
        self.step_flip = true;
        self.step_cycles -= STEP_CYCLES;
        let now = time::Instant::now();
        let d = now.duration_since(self.step_zero);
        let s = u64::from(STEP_TIME.saturating_sub(d.as_millis() as u32));
        thread::sleep(time::Duration::from_millis(s));
        self.step_zero = self
            .step_zero
            .checked_add(time::Duration::from_millis(u64::from(STEP_TIME)))
            .unwrap();

        if now.checked_duration_since(self.step_zero).is_some() {
            self.step_zero = now;
        }
    }

    pub fn next(&mut self) -> u32 {
        if self.speed_simulation {
            if self.step_cycles > STEP_CYCLES {
                self.down_frequency();
            }
            let cycles = self._next();
            self.step_cycles += cycles;
            cycles
        } else {
            self._next()
        }
    }

    pub fn flip(&mut self) -> bool {
        let r = self.step_flip;
        if r {
            self.step_flip = false;
        }
        r
    }

    pub fn get_current_opcode(&self) -> u8 {
        self.imm_freeze()
    }
    pub fn execute_opcode(&mut self) -> u32 {
        let opcode = self.imm();

        // TODO: 时钟周期这里有问题
        // if opcode != 0xCB {
        //     println!("cpu opcode is {:?}", opcode);
        //     println!("cpu reg is {:?}", format!("{:?}", self.reg).to_lowercase());
        // }
        if opcode != 0xcb {
            insert_cpu_record(CPUDebugInfo::new(self.reg.clone(), opcode, false));
        }

        OP_CODE_SET
            .get(&opcode)
            .expect(&format!("unknown opcode is {}", opcode))
            .ex(self)
    }

    pub fn get_opcode(&self) {}

    pub fn exexute_forever(&mut self) {}

    pub fn read_byte_from_memory(&self, addr: u16) -> u8 {
        let data = self.data_bus.borrow().read_byte(addr);
        // println!("fuck read byte {}:{:02x}", addr, data);
        data
    }

    pub fn read_word_from_memory(&self, addr: u16) -> u16 {
        let data = self.data_bus.borrow().read_word(addr);
        // println!("!!!! read byte {}:{:02x}", addr, data);
        data
    }

    pub fn write_byte_to_memory(&mut self, addr: u16, data: u8) {
        self.data_bus.borrow_mut().write_byte(addr, data);
    }

    pub fn write_word_to_memory(&mut self, addr: u16, data: u16) {
        self.data_bus.borrow_mut().write_word(addr, data);
    }
}

// all opcodes here
#[allow(non_snake_case)]
impl CPU {
    pub fn op_0x00(&mut self) -> u32 {
        // just nop here
        0
    }

    /// Read a word from the memory which PC pointed and assign to BC register.
    pub fn op_0x01(&mut self) -> u32 {
        let v = self.imm_word();
        self.reg.set_BC(v);
        0
    }

    /// Write the value in A register to the memory which BC pointed.
    pub fn op_0x02(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_BC(), self.reg.get_A());
        0
    }

    /// Incr the value in BC register.
    pub fn op_0x03(&mut self) -> u32 {
        let v = self.reg.get_BC();
        self.reg.set_BC(v.wrapping_add(1));
        0
    }

    // TODO why
    pub fn op_0x04(&mut self) -> u32 {
        let v = self.reg.get_B();
        let new_v = v.wrapping_add(1);
        self.reg.set_B(new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        0
    }

    pub fn op_0x05(&mut self) -> u32 {
        let v = self.reg.get_B();
        let new_v = v.wrapping_sub(1);
        self.reg.set_B(new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x06(&mut self) -> u32 {
        let v = self.imm();
        self.reg.set_B(v);

        0
    }

    // 	OP:0x07 RLCA
    pub fn op_0x07(&mut self) -> u32 {
        let v = self.reg.get_A();

        // TODO 这个逻辑貌似有问题
        self.reg.unset_flag(Flag::Zero);
        self.reg.unset_flag(Flag::Sub);
        self.reg.unset_flag(Flag::HalfCarry);
        if v > 0x7F {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        // TODO why
        self.reg.set_A(v << 1 | v >> 7);

        // origin := core.CPU.Registers.A

        // core.CPU.Registers.A = byte(core.CPU.Registers.A<<1) | (core.CPU.Registers.A >> 7)
        // core.CPU.Flags.Zero = false
        // core.CPU.Flags.Sub = false
        // core.CPU.Flags.HalfCarry = false
        // core.CPU.Flags.Carry = origin > 0x7F

        0
    }

    pub fn op_0x08(&mut self) -> u32 {
        let addr = self.imm_word();
        self.write_word_to_memory(addr, self.reg.get_SP());
        0
    }

    pub fn op_0x09(&mut self) -> u32 {
        let old_hl = self.reg.get_HL();
        let old_bc = self.reg.get_BC();

        let new_v = old_hl.wrapping_add(old_bc);
        self.reg.set_HL(new_v);

        if old_hl > 0xffff - old_bc {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }
        self.reg.unset_flag(Flag::Sub);

        if (old_hl & 0x0fff) + (old_bc & 0x0fff) > 0x0fff {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry)
        }

        0
    }

    pub fn op_0x0A(&mut self) -> u32 {
        self.reg
            .set_A(self.read_byte_from_memory(self.reg.get_BC()));
        0
    }

    pub fn op_0x0B(&mut self) -> u32 {
        let v = self.reg.get_BC();
        self.reg.set_BC(v.wrapping_sub(1));
        0
    }

    pub fn op_0x0C(&mut self) -> u32 {
        let v = self.reg.get_C();
        let new_v = v.wrapping_add(1);
        self.reg.set_C(new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        0
    }

    pub fn op_0x0D(&mut self) -> u32 {
        let v = self.reg.get_C();
        let new_v = v.wrapping_sub(1);
        self.reg.set_C(new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x0E(&mut self) -> u32 {
        let v = self.imm();
        self.reg.set_C(v);

        0
    }

    // 	OP:0x0F RRCA
    pub fn op_0x0F(&mut self) -> u32 {
        let v = self.reg.get_A();
        if v & 0x01 == 0x01 {
            self.reg.set_A(v >> 1 | 0x80);
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.set_A(v >> 1);
            self.reg.unset_flag(Flag::Carry);
        }

        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Zero);
        self.reg.unset_flag(Flag::Sub);

        0
    }

    pub fn op_0x10(&mut self) -> u32 {
        // TODO: Stop op code
        0
    }

    pub fn op_0x11(&mut self) -> u32 {
        let v = self.imm_word();
        self.reg.set_DE(v);
        0
    }

    pub fn op_0x12(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_DE(), self.reg.get_A());
        0
    }

    pub fn op_0x13(&mut self) -> u32 {
        let v = self.reg.get_DE();
        self.reg.set_DE(v.wrapping_add(1));
        0
    }

    pub fn op_0x14(&mut self) -> u32 {
        let v = self.reg.get_D();
        let new_v = v.wrapping_add(1);
        self.reg.set_D(new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        0
    }

    pub fn op_0x15(&mut self) -> u32 {
        let v = self.reg.get_D();
        let new_v = v.wrapping_sub(1);
        self.reg.set_D(new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x16(&mut self) -> u32 {
        let v = self.imm();
        self.reg.set_D(v);

        0
    }

    pub fn op_0x17(&mut self) -> u32 {
        let is_carry_flag_set = self.reg.is_flag_set(Flag::Carry);
        let mut carry = 0;
        if is_carry_flag_set {
            carry = 1;
        }
        let v = self.reg.get_A();

        self.reg.unset_flag(Flag::Zero);
        self.reg.unset_flag(Flag::Sub);
        self.reg.unset_flag(Flag::HalfCarry);

        if v & 0x80 == 0x80 {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        self.reg.set_A((v << 1 & 0xFF) | carry);

        0
    }

    pub fn op_0x18(&mut self) -> u32 {
        // address := int32(core.CPU.Registers.PC) + int32(int8(core.getParameter8()))
        // core.CPU.Registers.PC = uint16(address)

        let v = self.imm();
        // 这个地方强制转换可能有问题
        //let address = (self.reg.get_PC() as i32 + v as i32) as u16;
        //self.reg.set_PC(address);
        self.alu_jr(v);
        0
    }

    pub fn op_0x19(&mut self) -> u32 {
        let old_hl = self.reg.get_HL();
        let old_de = self.reg.get_DE();

        let new_v = old_hl.wrapping_add(old_de);
        self.reg.set_HL(new_v);
        self.reg.unset_flag(Flag::Sub);

        if old_hl > 0xffff - old_de {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        if (old_hl & 0x0fff) + (old_de & 0x0fff) > 0x0fff {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry)
        }

        0
    }

    pub fn op_0x1A(&mut self) -> u32 {
        self.reg
            .set_A(self.read_byte_from_memory(self.reg.get_DE()));
        0
    }

    pub fn op_0x1B(&mut self) -> u32 {
        let v = self.reg.get_DE();
        self.reg.set_DE(v.wrapping_sub(1));
        0
    }

    pub fn op_0x1C(&mut self) -> u32 {
        let v = self.reg.get_E();
        let new_v = v.wrapping_add(1);
        self.reg.set_E(new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x1D(&mut self) -> u32 {
        let v = self.reg.get_E();
        let new_v = v.wrapping_sub(1);
        self.reg.set_E(new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x1E(&mut self) -> u32 {
        let v = self.imm();
        self.reg.set_E(v);
        0
    }

    pub fn op_0x1F(&mut self) -> u32 {
        let v = self.reg.get_A();
        let is_carry_flag_set = self.reg.is_flag_set(Flag::Carry);
        let mut carry = 0;
        if is_carry_flag_set {
            carry = 0x80;
        }

        self.reg.unset_flag(Flag::Sub);
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Zero);
        self.reg.set_A((v >> 1) | carry);
        if v & 0x01 == 0x01 {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        0
    }
    fn alu_jr(&mut self, n: u8) {
        let n = n as i8;
        self.reg
            .set_PC(((u32::from(self.reg.get_PC()) as i32) + i32::from(n)) as u16);
    }

    // // JR IF
    // fn alu_jr(&mut self, n: u8) {
    //     let n = n as i8;
    //     self.reg.set_PC(((u32::from(self.reg.get_PC()) as i32) + i32::from(n)) as u16);
    // }

    pub fn op_0x20(&mut self) -> u32 {
        let v = self.imm();
        // let n = self.imm();
        if !self.reg.is_flag_set(Flag::Zero) {
            // let address = (self.reg.get_PC() as i32 + v as i32) as u16;
            // 这里必须先 u32 然后 i32 为什么
            // let address = ((u32::from(self.reg.get_PC()) as i32) + i32::from(v as i8)) as u16;
            // self.reg.set_PC(address);
            //
            self.alu_jr(v);
            return 4;
        }

        0
    }

    pub fn op_0x21(&mut self) -> u32 {
        let v = self.imm_word();
        self.reg.set_HL(v);
        0
    }

    pub fn op_0x22(&mut self) -> u32 {
        let addr = self.reg.get_HL();
        let v = self.reg.get_A();

        self.write_byte_to_memory(addr, v);
        self.reg.incr_HL();

        0
    }

    pub fn op_0x23(&mut self) -> u32 {
        self.reg.incr_HL();
        0
    }

    pub fn op_0x24(&mut self) -> u32 {
        let v = self.reg.get_H();
        let new_v = v.wrapping_add(1);
        self.reg.set_H(new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        0
    }

    pub fn op_0x25(&mut self) -> u32 {
        let v = self.reg.get_H();
        let new_v = v.wrapping_sub(1);
        self.reg.set_H(new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x26(&mut self) -> u32 {
        // https://users.rust-lang.org/t/mutable-borrows-in-nested-function-calls/28028/2
        // https://rustc-dev-guide.rust-lang.org/borrow_check/two_phase_borrows.html

        let v = self.imm();
        self.reg.set_H(v);
        0
    }

    pub fn op_0x27(&mut self) -> u32 {
        let mut v = self.reg.get_A();
        let mut adjust = if self.reg.is_flag_set(Flag::Carry) {
            0x60
        } else {
            0x00
        };
        if self.reg.is_flag_set(Flag::HalfCarry) {
            adjust |= 0x06;
        }
        if !self.reg.is_flag_set(Flag::Sub) {
            if v & 0x0f > 0x09 {
                adjust |= 0x06;
            };
            if v > 0x99 {
                adjust |= 0x60;
            };
            v = v.wrapping_add(adjust);
        } else {
            v = v.wrapping_sub(adjust);
        }

        if adjust >= 0x60 {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        // TODO https://github.com/HFO4/gameboy.live/blob/657501f18a60c486366cd04b87025a7781db1fd1/gb/opcodes.go#L1354
        self.reg.unset_flag(Flag::HalfCarry);
        if v == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
        self.reg.set_A(v);

        0
    }

    pub fn op_0x28(&mut self) -> u32 {
        let v = self.imm();

        if self.reg.is_flag_set(Flag::Zero) {
            // let n = n as i8;
            // self.reg.pc = ((u32::from(self.reg.pc) as i32) + i32::from(n)) as u16;

            // let address = (self.reg.get_PC() as i32 + i32::from(v)) as u16;
            // self.reg.set_PC(address);
            self.alu_jr(v);
            return 4;
        }
        0
    }

    pub fn op_0x29(&mut self) -> u32 {
        let v = self.reg.get_HL();

        let new_v = v.wrapping_add(v);
        self.reg.set_HL(new_v);
        self.reg.unset_flag(Flag::Sub);

        if v > 0xffff - v {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        if (v & 0x0fff) + (v & 0x0fff) > 0x0fff {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry)
        }

        0
    }

    pub fn op_0x2A(&mut self) -> u32 {
        let addr = self.reg.get_HL();
        let data = self.read_byte_from_memory(addr);
        self.reg.set_A(data);
        self.reg.incr_HL();
        0
    }

    pub fn op_0x2B(&mut self) -> u32 {
        let v = self.reg.get_HL();
        self.reg.set_HL(v.wrapping_sub(1));
        0
    }

    pub fn op_0x2C(&mut self) -> u32 {
        let v = self.reg.get_L();
        let new_v = v.wrapping_add(1);
        self.reg.set_L(new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x2D(&mut self) -> u32 {
        let v = self.reg.get_L();
        let new_v = v.wrapping_sub(1);
        self.reg.set_L(new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x2E(&mut self) -> u32 {
        let v = self.imm();
        self.reg.set_L(v);
        0
    }

    // 	OP:0x2F CPL
    pub fn op_0x2F(&mut self) -> u32 {
        // core.CPU.Registers.A = 0XFF ^ core.CPU.Registers.A
        // core.CPU.Flags.Sub = true
        // core.CPU.Flags.HalfCarry = true

        self.reg.set_A(0xff ^ self.reg.get_A());
        self.reg.set_flag(Flag::Sub);
        self.reg.set_flag(Flag::HalfCarry);

        0
    }

    pub fn op_0x30(&mut self) -> u32 {
        let v = self.imm(); // as i8;
        if !self.reg.is_flag_set(Flag::Carry) {
            // let address = (self.reg.get_PC() as i32 + v as i32) as u16;
            // self.reg.set_PC(address);

            self.alu_jr(v);
            return 4;
        }

        0
    }

    pub fn op_0x31(&mut self) -> u32 {
        let v = self.imm_word();
        self.reg.set_SP(v);
        0
    }

    pub fn op_0x32(&mut self) -> u32 {
        let addr = self.reg.get_HL();
        let data = self.reg.get_A();
        self.write_byte_to_memory(addr, data);
        self.reg.set_HL(addr - 1);
        0
    }

    pub fn op_0x33(&mut self) -> u32 {
        let v = self.reg.get_SP();
        self.reg.set_SP(v.wrapping_add(1));
        0
    }

    pub fn op_0x34(&mut self) -> u32 {
        let addr = self.reg.get_HL();
        let v = self.read_byte_from_memory(addr);
        let new_v = v.wrapping_add(1);
        self.write_byte_to_memory(addr, new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x35(&mut self) -> u32 {
        let addr = self.reg.get_HL();
        let v = self.read_byte_from_memory(addr);
        let new_v = v.wrapping_sub(1);
        self.write_byte_to_memory(addr, new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        0
    }

    pub fn op_0x36(&mut self) -> u32 {
        let addr = self.reg.get_HL();
        let v = self.imm();
        self.write_byte_to_memory(addr, v);
        0
    }

    pub fn op_0x37(&mut self) -> u32 {
        self.reg.set_flag(Flag::Carry);
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);
        0
    }

    pub fn op_0x38(&mut self) -> u32 {
        let v = self.imm(); // as i8;
        if self.reg.is_flag_set(Flag::Carry) {
            // let address = (self.reg.get_PC() as i32 + v as i32) as u16;
            // self.reg.set_PC(address);
            self.alu_jr(v);
            return 4;
        }
        0
    }

    pub fn op_0x39(&mut self) -> u32 {
        let v = self.reg.get_SP();
        let a = self.reg.get_HL();

        let new_v = a.wrapping_add(v);
        self.reg.set_HL(new_v);
        self.reg.unset_flag(Flag::Sub);

        if a > 0xffff - v {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        if (a & 0x0fff) + (v & 0x0fff) > 0x0fff {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry)
        }

        0
    }

    pub fn op_0x3A(&mut self) -> u32 {
        let addr = self.reg.get_HL();
        let value = self.read_byte_from_memory(addr);
        self.reg.set_A(value);
        self.reg.set_HL(addr.wrapping_sub(1));
        0
    }

    pub fn op_0x3B(&mut self) -> u32 {
        let v = self.reg.get_SP();
        self.reg.set_SP(v.wrapping_sub(1));
        0
    }

    pub fn op_0x3C(&mut self) -> u32 {
        let v = self.reg.get_A();
        let new_v = v.wrapping_add(1);
        self.reg.set_A(new_v);

        self.reg.unset_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if (v & 0x0f) + 0x01 > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        0
    }

    pub fn op_0x3D(&mut self) -> u32 {
        let v = self.reg.get_A();
        let new_v = v.wrapping_sub(1);
        self.reg.set_A(new_v);

        self.reg.set_flag(Flag::Sub);
        if new_v == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v & 0x0F == 0 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
        0
    }

    pub fn op_0x3E(&mut self) -> u32 {
        let v = self.imm();
        self.reg.set_A(v);
        0
    }

    pub fn op_0x3F(&mut self) -> u32 {
        self.reg.reverse_flag(Flag::Carry);
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);
        0
    }

    pub fn op_0x40(&mut self) -> u32 {
        self.reg.set_B(self.reg.get_B());
        0
    }

    pub fn op_0x41(&mut self) -> u32 {
        self.reg.set_B(self.reg.get_C());
        0
    }

    pub fn op_0x42(&mut self) -> u32 {
        self.reg.set_B(self.reg.get_D());
        0
    }

    pub fn op_0x43(&mut self) -> u32 {
        self.reg.set_B(self.reg.get_E());

        0
    }

    pub fn op_0x44(&mut self) -> u32 {
        self.reg.set_B(self.reg.get_H());

        0
    }

    pub fn op_0x45(&mut self) -> u32 {
        self.reg.set_B(self.reg.get_L());

        0
    }

    pub fn op_0x46(&mut self) -> u32 {
        self.reg
            .set_B(self.read_byte_from_memory(self.reg.get_HL()));
        0
    }

    pub fn op_0x47(&mut self) -> u32 {
        self.reg.set_B(self.reg.get_A());

        0
    }

    pub fn op_0x48(&mut self) -> u32 {
        self.reg.set_C(self.reg.get_B());

        0
    }

    pub fn op_0x49(&mut self) -> u32 {
        self.reg.set_C(self.reg.get_C());

        0
    }

    pub fn op_0x4A(&mut self) -> u32 {
        self.reg.set_C(self.reg.get_D());

        0
    }

    pub fn op_0x4B(&mut self) -> u32 {
        self.reg.set_C(self.reg.get_E());

        0
    }

    pub fn op_0x4C(&mut self) -> u32 {
        self.reg.set_C(self.reg.get_H());

        0
    }

    pub fn op_0x4D(&mut self) -> u32 {
        self.reg.set_C(self.reg.get_L());

        0
    }

    pub fn op_0x4E(&mut self) -> u32 {
        self.reg
            .set_C(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x4F(&mut self) -> u32 {
        self.reg.set_C(self.reg.get_A());

        0
    }

    pub fn op_0x50(&mut self) -> u32 {
        self.reg.set_D(self.reg.get_B());

        0
    }

    pub fn op_0x51(&mut self) -> u32 {
        self.reg.set_D(self.reg.get_C());

        0
    }

    pub fn op_0x52(&mut self) -> u32 {
        self.reg.set_D(self.reg.get_D());

        0
    }

    pub fn op_0x53(&mut self) -> u32 {
        self.reg.set_D(self.reg.get_E());

        0
    }

    pub fn op_0x54(&mut self) -> u32 {
        self.reg.set_D(self.reg.get_H());

        0
    }

    pub fn op_0x55(&mut self) -> u32 {
        self.reg.set_D(self.reg.get_L());

        0
    }

    pub fn op_0x56(&mut self) -> u32 {
        self.reg
            .set_D(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x57(&mut self) -> u32 {
        self.reg.set_D(self.reg.get_A());

        0
    }

    pub fn op_0x58(&mut self) -> u32 {
        self.reg.set_E(self.reg.get_B());

        0
    }

    pub fn op_0x59(&mut self) -> u32 {
        self.reg.set_E(self.reg.get_C());

        0
    }

    pub fn op_0x5A(&mut self) -> u32 {
        self.reg.set_E(self.reg.get_D());

        0
    }

    pub fn op_0x5B(&mut self) -> u32 {
        self.reg.set_E(self.reg.get_E());

        0
    }

    pub fn op_0x5C(&mut self) -> u32 {
        self.reg.set_E(self.reg.get_H());

        0
    }

    pub fn op_0x5D(&mut self) -> u32 {
        self.reg.set_E(self.reg.get_L());

        0
    }

    pub fn op_0x5E(&mut self) -> u32 {
        self.reg
            .set_E(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x5F(&mut self) -> u32 {
        self.reg.set_E(self.reg.get_A());

        0
    }

    pub fn op_0x60(&mut self) -> u32 {
        self.reg.set_H(self.reg.get_B());

        0
    }

    pub fn op_0x61(&mut self) -> u32 {
        self.reg.set_H(self.reg.get_C());

        0
    }

    pub fn op_0x62(&mut self) -> u32 {
        self.reg.set_H(self.reg.get_D());

        0
    }

    pub fn op_0x63(&mut self) -> u32 {
        self.reg.set_H(self.reg.get_E());

        0
    }

    pub fn op_0x64(&mut self) -> u32 {
        self.reg.set_H(self.reg.get_H());

        0
    }

    pub fn op_0x65(&mut self) -> u32 {
        self.reg.set_H(self.reg.get_L());

        0
    }

    pub fn op_0x66(&mut self) -> u32 {
        self.reg
            .set_H(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x67(&mut self) -> u32 {
        self.reg.set_H(self.reg.get_A());

        0
    }

    pub fn op_0x68(&mut self) -> u32 {
        self.reg.set_L(self.reg.get_B());

        0
    }

    pub fn op_0x69(&mut self) -> u32 {
        self.reg.set_L(self.reg.get_C());

        0
    }

    pub fn op_0x6A(&mut self) -> u32 {
        self.reg.set_L(self.reg.get_D());

        0
    }

    pub fn op_0x6B(&mut self) -> u32 {
        self.reg.set_L(self.reg.get_E());

        0
    }

    pub fn op_0x6C(&mut self) -> u32 {
        self.reg.set_L(self.reg.get_H());

        0
    }

    pub fn op_0x6D(&mut self) -> u32 {
        self.reg.set_L(self.reg.get_L());

        0
    }

    pub fn op_0x6E(&mut self) -> u32 {
        self.reg
            .set_L(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x6F(&mut self) -> u32 {
        self.reg.set_L(self.reg.get_A());

        0
    }

    pub fn op_0x70(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_HL(), self.reg.get_B());
        0
    }

    pub fn op_0x71(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_HL(), self.reg.get_C());

        0
    }

    pub fn op_0x72(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_HL(), self.reg.get_D());

        0
    }

    pub fn op_0x73(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_HL(), self.reg.get_E());

        0
    }

    pub fn op_0x74(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_HL(), self.reg.get_H());

        0
    }

    pub fn op_0x75(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_HL(), self.reg.get_L());

        0
    }

    pub fn op_0x76(&mut self) -> u32 {
        self.is_halt = true;
        // info!("halt opcode!!");
        0
    }

    pub fn op_0x77(&mut self) -> u32 {
        self.write_byte_to_memory(self.reg.get_HL(), self.reg.get_A());

        0
    }

    pub fn op_0x78(&mut self) -> u32 {
        self.reg.set_A(self.reg.get_B());

        0
    }

    pub fn op_0x79(&mut self) -> u32 {
        self.reg.set_A(self.reg.get_C());

        0
    }

    pub fn op_0x7A(&mut self) -> u32 {
        self.reg.set_A(self.reg.get_D());

        0
    }

    pub fn op_0x7B(&mut self) -> u32 {
        self.reg.set_A(self.reg.get_E());

        0
    }

    pub fn op_0x7C(&mut self) -> u32 {
        self.reg.set_A(self.reg.get_H());

        0
    }

    pub fn op_0x7D(&mut self) -> u32 {
        self.reg.set_A(self.reg.get_L());

        0
    }

    pub fn op_0x7E(&mut self) -> u32 {
        self.reg
            .set_A(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x7F(&mut self) -> u32 {
        self.reg.set_A(self.reg.get_A());

        0
    }

    fn _op_add(&mut self, v: u8) {
        let v1 = self.reg.get_A();
        let v2 = v;
        let res = v1.wrapping_add(v2);

        self.reg.set_A(res);
        if res == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
        self.reg.unset_flag(Flag::Sub);
        if (v1 & 0x0F) + (v2 & 0x0F) > 0x0F {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        if u16::from(v1) + u16::from(v2) > 0xFF {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }
    }
    pub fn op_0x80(&mut self) -> u32 {
        self._op_add(self.reg.get_B());
        0
    }

    pub fn op_0x81(&mut self) -> u32 {
        self._op_add(self.reg.get_C());

        0
    }

    pub fn op_0x82(&mut self) -> u32 {
        self._op_add(self.reg.get_D());

        0
    }

    pub fn op_0x83(&mut self) -> u32 {
        self._op_add(self.reg.get_E());

        0
    }

    pub fn op_0x84(&mut self) -> u32 {
        self._op_add(self.reg.get_H());

        0
    }

    pub fn op_0x85(&mut self) -> u32 {
        self._op_add(self.reg.get_L());

        0
    }

    pub fn op_0x86(&mut self) -> u32 {
        self._op_add(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x87(&mut self) -> u32 {
        self._op_add(self.reg.get_A());

        0
    }

    // ADC
    fn _op_adc(&mut self, v: u8) {
        let carry = if self.reg.is_flag_set(Flag::Carry) {
            1
        } else {
            0
        };
        let v1 = self.reg.get_A();
        let v2 = v;

        let res = v1.wrapping_add(v2).wrapping_add(carry);
        self.reg.set_A(res);

        self.reg.unset_flag(Flag::Sub);
        if res == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if u16::from(v1) + u16::from(v2) + u16::from(carry) > 0xFF {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        if (v1 & 0x0f) + (v2 & 0x0f) + (carry & 0x0f) > 0x0f {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
    }

    pub fn op_0x88(&mut self) -> u32 {
        self._op_adc(self.reg.get_B());
        0
    }

    pub fn op_0x89(&mut self) -> u32 {
        self._op_adc(self.reg.get_C());

        0
    }

    pub fn op_0x8A(&mut self) -> u32 {
        self._op_adc(self.reg.get_D());

        0
    }

    pub fn op_0x8B(&mut self) -> u32 {
        self._op_adc(self.reg.get_E());

        0
    }

    pub fn op_0x8C(&mut self) -> u32 {
        self._op_adc(self.reg.get_H());

        0
    }

    pub fn op_0x8D(&mut self) -> u32 {
        self._op_adc(self.reg.get_L());

        0
    }

    pub fn op_0x8E(&mut self) -> u32 {
        self._op_adc(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x8F(&mut self) -> u32 {
        self._op_adc(self.reg.get_A());

        0
    }

    fn _op_sub(&mut self, v: u8) {
        let v1 = self.reg.get_A();
        let v2 = v;
        let res = v1.wrapping_sub(v2);
        self.reg.set_A(res);

        self.reg.set_flag(Flag::Sub);
        if res == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if u16::from(v1) < u16::from(v2) {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        if u16::from(v1 & 0x0F) < u16::from(v2 & 0x0F) {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
    }
    pub fn op_0x90(&mut self) -> u32 {
        self._op_sub(self.reg.get_B());
        0
    }

    pub fn op_0x91(&mut self) -> u32 {
        self._op_sub(self.reg.get_C());

        0
    }

    pub fn op_0x92(&mut self) -> u32 {
        self._op_sub(self.reg.get_D());

        0
    }

    pub fn op_0x93(&mut self) -> u32 {
        self._op_sub(self.reg.get_E());

        0
    }

    pub fn op_0x94(&mut self) -> u32 {
        self._op_sub(self.reg.get_H());

        0
    }

    pub fn op_0x95(&mut self) -> u32 {
        self._op_sub(self.reg.get_L());

        0
    }

    pub fn op_0x96(&mut self) -> u32 {
        self._op_sub(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x97(&mut self) -> u32 {
        self._op_sub(self.reg.get_A());

        0
    }

    fn _op_sbc(&mut self, v: u8) {
        let v1 = self.reg.get_A();
        let carry = if self.reg.is_flag_set(Flag::Carry) {
            1
        } else {
            0
        };
        let v2 = v;
        let res = v1.wrapping_sub(v2).wrapping_sub(carry);
        self.reg.set_A(res);

        if res == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
        self.reg.set_flag(Flag::Sub);

        if u16::from(v1) < u16::from(v2) + u16::from(carry) {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        if (v1 & 0x0F) < (v2 & 0x0F) + (carry & 0x0F) {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
    }

    pub fn op_0x98(&mut self) -> u32 {
        self._op_sbc(self.reg.get_B());
        0
    }

    pub fn op_0x99(&mut self) -> u32 {
        self._op_sbc(self.reg.get_C());

        0
    }

    pub fn op_0x9A(&mut self) -> u32 {
        self._op_sbc(self.reg.get_D());

        0
    }

    pub fn op_0x9B(&mut self) -> u32 {
        self._op_sbc(self.reg.get_E());

        0
    }

    pub fn op_0x9C(&mut self) -> u32 {
        self._op_sbc(self.reg.get_H());

        0
    }

    pub fn op_0x9D(&mut self) -> u32 {
        self._op_sbc(self.reg.get_L());

        0
    }

    pub fn op_0x9E(&mut self) -> u32 {
        self._op_sbc(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0x9F(&mut self) -> u32 {
        self._op_sbc(self.reg.get_A());

        0
    }

    fn _op_and(&mut self, v: u8) {
        let v1 = self.reg.get_A();
        let v2 = v;
        let res = v1 & v2;
        self.reg.set_A(res);

        self.reg.unset_flag(Flag::Carry);
        self.reg.set_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);
        if res == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
    }

    pub fn op_0xA0(&mut self) -> u32 {
        self._op_and(self.reg.get_B());
        0
    }

    pub fn op_0xA1(&mut self) -> u32 {
        self._op_and(self.reg.get_C());

        0
    }

    pub fn op_0xA2(&mut self) -> u32 {
        self._op_and(self.reg.get_D());

        0
    }

    pub fn op_0xA3(&mut self) -> u32 {
        self._op_and(self.reg.get_E());

        0
    }

    pub fn op_0xA4(&mut self) -> u32 {
        self._op_and(self.reg.get_H());

        0
    }

    pub fn op_0xA5(&mut self) -> u32 {
        self._op_and(self.reg.get_L());

        0
    }

    pub fn op_0xA6(&mut self) -> u32 {
        self._op_and(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0xA7(&mut self) -> u32 {
        self._op_and(self.reg.get_A());

        0
    }

    fn _op_xor(&mut self, v: u8) {
        let v1 = self.reg.get_A();
        let v2 = v;
        let res = v1 ^ v2;
        self.reg.set_A(res);

        if res == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
        self.reg.unset_flag(Flag::Sub);
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Carry);
    }

    pub fn op_0xA8(&mut self) -> u32 {
        self._op_xor(self.reg.get_B());

        0
    }

    pub fn op_0xA9(&mut self) -> u32 {
        self._op_xor(self.reg.get_C());

        0
    }

    pub fn op_0xAA(&mut self) -> u32 {
        self._op_xor(self.reg.get_D());

        0
    }

    pub fn op_0xAB(&mut self) -> u32 {
        self._op_xor(self.reg.get_E());

        0
    }

    pub fn op_0xAC(&mut self) -> u32 {
        self._op_xor(self.reg.get_H());

        0
    }

    pub fn op_0xAD(&mut self) -> u32 {
        self._op_xor(self.reg.get_L());

        0
    }

    pub fn op_0xAE(&mut self) -> u32 {
        self._op_xor(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0xAF(&mut self) -> u32 {
        self._op_xor(self.reg.get_A());

        0
    }

    fn _op_or(&mut self, v: u8) {
        let v1 = self.reg.get_A();
        let v2 = v;
        let res = v1 | v2;
        self.reg.set_A(res);

        if res == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
        self.reg.unset_flag(Flag::Sub);
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Carry);
    }

    pub fn op_0xB0(&mut self) -> u32 {
        self._op_or(self.reg.get_B());

        0
    }

    pub fn op_0xB1(&mut self) -> u32 {
        self._op_or(self.reg.get_C());

        0
    }

    pub fn op_0xB2(&mut self) -> u32 {
        self._op_or(self.reg.get_D());

        0
    }

    pub fn op_0xB3(&mut self) -> u32 {
        self._op_or(self.reg.get_E());

        0
    }

    pub fn op_0xB4(&mut self) -> u32 {
        self._op_or(self.reg.get_H());

        0
    }

    pub fn op_0xB5(&mut self) -> u32 {
        self._op_or(self.reg.get_L());

        0
    }

    pub fn op_0xB6(&mut self) -> u32 {
        self._op_or(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0xB7(&mut self) -> u32 {
        self._op_or(self.reg.get_A());

        0
    }

    fn _op_compare(&mut self, v: u8) {
        let v1 = v;
        let v2 = self.reg.get_A();

        self.reg.set_flag(Flag::Sub);
        if v1 == v2 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        if v1 > v2 {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        if v1 & 0x0F > v2 & 0x0F {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }
    }

    pub fn op_0xB8(&mut self) -> u32 {
        self._op_compare(self.reg.get_B());
        0
    }

    pub fn op_0xB9(&mut self) -> u32 {
        self._op_compare(self.reg.get_C());

        0
    }

    pub fn op_0xBA(&mut self) -> u32 {
        self._op_compare(self.reg.get_D());

        0
    }

    pub fn op_0xBB(&mut self) -> u32 {
        self._op_compare(self.reg.get_E());

        0
    }

    pub fn op_0xBC(&mut self) -> u32 {
        self._op_compare(self.reg.get_H());

        0
    }

    pub fn op_0xBD(&mut self) -> u32 {
        self._op_compare(self.reg.get_L());

        0
    }

    pub fn op_0xBE(&mut self) -> u32 {
        self._op_compare(self.read_byte_from_memory(self.reg.get_HL()));

        0
    }

    pub fn op_0xBF(&mut self) -> u32 {
        self._op_compare(self.reg.get_A());

        0
    }

    fn _stack_pop(&mut self) -> u16 {
        let addr = self.reg.get_SP();
        let data = self.read_word_from_memory(addr);
        self.reg.set_SP(addr + 2);
        data
    }

    pub fn op_0xC0(&mut self) -> u32 {
        if !self.reg.is_flag_set(Flag::Zero) {
            let v = self._stack_pop();
            self.reg.set_PC(v);
            return 12;
        }
        0
    }

    pub fn op_0xC1(&mut self) -> u32 {
        let v = self._stack_pop();
        self.reg.set_BC(v);
        0
    }

    pub fn op_0xC2(&mut self) -> u32 {
        let addr = self.imm_word();
        if !self.reg.is_flag_set(Flag::Zero) {
            self.reg.set_PC(addr);
            return 4;
        }
        0
    }

    pub fn op_0xC3(&mut self) -> u32 {
        let v = self.imm_word();
        self.reg.set_PC(v);
        0
    }

    fn _stack_push(&mut self, data: u16) {
        let sp = self.reg.get_SP();
        let new_sp = sp - 2;
        self.reg.set_SP(new_sp);
        self.write_word_to_memory(new_sp, data);
    }

    pub fn op_0xC4(&mut self) -> u32 {
        let v = self.imm_word();
        if !self.reg.is_flag_set(Flag::Zero) {
            self._stack_push(self.reg.get_PC());
            self.reg.set_PC(v);
            return 14;
        }
        0
    }

    pub fn op_0xC5(&mut self) -> u32 {
        self._stack_push(self.reg.get_BC());
        0
    }

    pub fn op_0xC6(&mut self) -> u32 {
        let v = self.imm();
        self._op_add(v);

        0
    }

    pub fn op_0xC7(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x0000);
        0
    }

    pub fn op_0xC8(&mut self) -> u32 {
        if self.reg.is_flag_set(Flag::Zero) {
            let v = self._stack_pop();
            self.reg.set_PC(v);
            return 12;
        }
        0
    }

    pub fn op_0xC9(&mut self) -> u32 {
        let v = self._stack_pop();
        self.reg.set_PC(v);
        0
    }

    pub fn op_0xCA(&mut self) -> u32 {
        let addr = self.imm_word();
        if self.reg.is_flag_set(Flag::Zero) {
            self.reg.set_PC(addr);
            return 4;
        }

        0
    }

    pub fn op_0xCC(&mut self) -> u32 {
        let v = self.imm_word();
        if self.reg.is_flag_set(Flag::Zero) {
            self._stack_push(self.reg.get_PC());
            self.reg.set_PC(v);
            return 12;
        }

        0
    }

    pub fn op_0xCD(&mut self) -> u32 {
        let v = self.imm_word();
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(v);

        0
    }

    pub fn op_0xCE(&mut self) -> u32 {
        let v = self.imm();
        self._op_adc(v);

        0
    }

    pub fn op_0xCF(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x0008);
        0
    }

    pub fn op_0xD0(&mut self) -> u32 {
        if !self.reg.is_flag_set(Flag::Carry) {
            let v = self._stack_pop();
            self.reg.set_PC(v);
            return 12;
        }
        0
    }

    pub fn op_0xD1(&mut self) -> u32 {
        // self.reg.set_DE(self._stack_pop());

        let v = self._stack_pop();
        self.reg.set_DE(v);

        0
    }

    pub fn op_0xD2(&mut self) -> u32 {
        let addr = self.imm_word();
        if !self.reg.is_flag_set(Flag::Carry) {
            self.reg.set_PC(addr);
            return 4;
        }
        0
    }

    // pub fn op_0xD3(&mut self) -> u32 {

    //     0
    // }

    pub fn op_0xD4(&mut self) -> u32 {
        let v = self.imm_word();
        if !self.reg.is_flag_set(Flag::Carry) {
            self._stack_push(self.reg.get_PC());
            self.reg.set_PC(v);
            return 14;
        }
        0
    }

    pub fn op_0xD5(&mut self) -> u32 {
        self._stack_push(self.reg.get_DE());

        0
    }

    pub fn op_0xD6(&mut self) -> u32 {
        let v = self.imm();
        self._op_sub(v);
        0
    }

    pub fn op_0xD7(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x0010);
        0
    }

    pub fn op_0xD8(&mut self) -> u32 {
        if self.reg.is_flag_set(Flag::Carry) {
            let v = self._stack_pop();
            self.reg.set_PC(v);
            return 12;
        }
        0
    }

    pub fn op_0xD9(&mut self) -> u32 {
        let v = self._stack_pop();
        self.reg.set_PC(v);
        self.enable_ime();
        0
    }

    pub fn op_0xDA(&mut self) -> u32 {
        let addr = self.imm_word();
        if self.reg.is_flag_set(Flag::Carry) {
            self.reg.set_PC(addr);
            return 4;
        }
        0
    }

    pub fn op_0xDB(&mut self) -> u32 {
        // address := core.getParameter16()
        // if core.CPU.Flags.Carry {
        //     core.StackPush(core.CPU.Registers.PC)
        //     core.CPU.Registers.PC = address
        //     return 12
        // }
        // return 0

        let addr = self.imm_word();
        if self.reg.is_flag_set(Flag::Carry) {
            self._stack_push(self.reg.get_PC());
            self.reg.set_PC(addr);
            return 12;
        }

        0
    }

    pub fn op_0xDC(&mut self) -> u32 {
        let v = self.imm_word();
        if self.reg.is_flag_set(Flag::Carry) {
            self._stack_push(self.reg.get_PC());
            self.reg.set_PC(v);
            return 14;
        }
        0
    }

    // pub fn op_0xDd(&mut self) -> u32 {

    //     0
    // }

    pub fn op_0xDE(&mut self) -> u32 {
        let v = self.imm();
        self._op_sbc(v);
        0
    }

    pub fn op_0xDF(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x18);
        0
    }

    pub fn op_0xE0(&mut self) -> u32 {
        // NOTE
        let addr = 0xFF00 | u16::from(self.imm());
        let data = self.reg.get_A();
        self.write_byte_to_memory(addr, data);

        // let a = 0xff00 | u16::from(self.imm());
        // self.mem.borrow_mut().set(a, self.reg.a);

        // core.WriteMemory(0xFF00+uint16(core.getParameter8()), core.CPU.Registers.A)
        0
    }

    pub fn op_0xE1(&mut self) -> u32 {
        let v = self._stack_pop();
        self.reg.set_HL(v);
        0
    }

    pub fn op_0xE2(&mut self) -> u32 {
        let addr = 0xFF00 | u16::from(self.reg.get_C());
        let data = self.reg.get_A();
        self.write_byte_to_memory(addr, data);
        0
    }

    // pub fn op_0xe3(&mut self) -> u32 {
    //     0
    // }

    // pub fn op_0xe4(&mut self) -> u32 {
    //     0
    // }

    pub fn op_0xE5(&mut self) -> u32 {
        self._stack_push(self.reg.get_HL());
        0
    }

    pub fn op_0xE6(&mut self) -> u32 {
        let v = self.imm();
        self._op_and(v);
        0
    }

    pub fn op_0xE7(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x0020);
        0
    }

    pub fn op_0xE8(&mut self) -> u32 {
        // origin1 := core.CPU.Registers.SP
        // origin2 := int8(core.getParameter8())
        // res := uint16(int32(core.CPU.Registers.SP) + int32(origin2))
        // tmpVal := origin1 ^ uint16(origin2) ^ res
        // core.CPU.Registers.SP = res

        // core.CPU.Flags.Zero = false
        // core.CPU.Flags.Sub = false
        // core.CPU.Flags.HalfCarry = (tmpVal & 0x10) == 0x10
        // core.CPU.Flags.Carry = ((tmpVal & 0x100) == 0x100)

        let v1 = self.reg.get_SP();
        let v2 = i16::from(self.imm() as i8) as u16;
        let res = v1.wrapping_add(v2);
        let tmp = v1 ^ v2 ^ res;

        self.reg.set_SP(res);

        self.reg.unset_flag(Flag::Zero);
        self.reg.unset_flag(Flag::Sub);
        if tmp & 0x10 == 0x10 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        if tmp & 0x100 == 0x100 {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        0
    }

    pub fn op_0xE9(&mut self) -> u32 {
        self.reg.set_PC(self.reg.get_HL());
        0
    }

    pub fn op_0xEA(&mut self) -> u32 {
        let addr = self.imm_word();
        self.write_byte_to_memory(addr, self.reg.get_A());
        0
    }

    // pub fn op_0xeb(&mut self) -> u32 {

    //     0
    // }

    // pub fn op_0xec(&mut self) -> u32 {
    //     0
    // }

    // pub fn op_0xed(&mut self) -> u32 {
    //     0
    // }

    pub fn op_0xEE(&mut self) -> u32 {
        let v = self.imm();
        self._op_xor(v);
        0
    }

    pub fn op_0xEF(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x0028);
        0
    }

    pub fn op_0xF0(&mut self) -> u32 {
        let addr = 0xFF00 | u16::from(self.imm());
        // println!(
        //     "!!!!OXF0 addr is {} => {}",
        //     addr,
        //     self.read_byte_from_memory(addr)
        // );
        self.reg.set_A(self.read_byte_from_memory(addr));

        0
    }

    pub fn op_0xF1(&mut self) -> u32 {
        let v = self._stack_pop();
        // 这个地方需要注意
        self.reg.set_AF(v);
        0
    }

    pub fn op_0xF2(&mut self) -> u32 {
        let addr = 0xFF00 + u16::from(self.reg.get_C());
        self.reg.set_A(self.read_byte_from_memory(addr));
        0
    }

    pub fn op_0xF3(&mut self) -> u32 {
        self.disable_ime();
        0
    }

    // pub fn op_0xf4(&mut self) -> u32 {
    //     0
    // }

    pub fn op_0xF5(&mut self) -> u32 {
        self._stack_push(self.reg.get_AF());
        0
    }

    pub fn op_0xF6(&mut self) -> u32 {
        let v = self.imm();

        self._op_or(v);
        0
    }

    pub fn op_0xF7(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x0030);
        0
    }

    pub fn op_0xF8(&mut self) -> u32 {
        let v1 = self.reg.get_SP();
        // NOTICE TODO:
        // u8 to i8 then to i32
        // 将 SP 寄存器 + 有符号 8 位立即参数的结果写入寄存器 HL.
        let v2 = self.imm() as i8;

        let res = v1.wrapping_add(i16::from(v2 as i8) as u16);

        self.reg.set_HL(res);

        let tmp = (v1 as i32) ^ (v2 as i32) ^ (res as i32);
        self.reg.unset_flag(Flag::Zero);
        self.reg.unset_flag(Flag::Sub);

        if tmp & 0x10 == 0x10 {
            self.reg.set_flag(Flag::HalfCarry);
        } else {
            self.reg.unset_flag(Flag::HalfCarry);
        }

        if tmp & 0x100 == 0x100 {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        0
    }

    pub fn op_0xF9(&mut self) -> u32 {
        self.reg.set_SP(self.reg.get_HL());
        0
    }

    pub fn op_0xFA(&mut self) -> u32 {
        let addr = self.imm_word();
        self.reg.set_A(self.read_byte_from_memory(addr));
        0
    }

    pub fn op_0xFB(&mut self) -> u32 {
        self.enable_ime();
        0
    }

    // pub fn op_0xfc(&mut self) -> u32 {

    //     0
    // }

    // pub fn op_0xfd(&mut self) -> u32 {
    //     0
    // }

    pub fn op_0xFE(&mut self) -> u32 {
        let v = self.imm();
        self._op_compare(v);

        0
    }

    pub fn op_0xFF(&mut self) -> u32 {
        self._stack_push(self.reg.get_PC());
        self.reg.set_PC(0x0038);
        0
    }
}

// Extend OpCodes

#[allow(non_snake_case)]
impl<'a> CPU {
    fn alu_rlc(&mut self, a: u8) -> u8 {
        let c = (a & 0x80) >> 7 == 0x01;
        let r = (a << 1) | u8::from(c);

        if c {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);
        if r == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        r
    }

    fn alu_rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = if c { 0x80 | (a >> 1) } else { a >> 1 };

        if c {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);
        if r == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        r
    }

    fn alu_rl(&mut self, a: u8) -> u8 {
        let c = (a & 0x80) >> 7 == 0x01;

        let r = (a << 1)
            + if self.reg.is_flag_set(Flag::Carry) {
                1
            } else {
                0
            };

        if c {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);

        if r == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        r
    }

    fn alu_rr(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = if self.reg.is_flag_set(Flag::Carry) {
            0x80 | (a >> 1)
        } else {
            a >> 1
        };

        if c {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);
        if r == 0x00 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        r
    }

    fn alu_sla(&mut self, a: u8) -> u8 {
        let c = (a & 0x80) >> 7 == 0x01;
        let r = a << 1;
        if c {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }

        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);

        if r == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        r
    }

    fn alu_sra(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (a & 0x80);

        if c {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);

        if r == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }

        r
    }

    fn alu_swap(&mut self, a: u8) -> u8 {
        self.reg.unset_flag(Flag::Carry);
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);
        if a == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
        (a >> 4) | (a << 4)
    }

    fn alu_srl(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = a >> 1;

        if c {
            self.reg.set_flag(Flag::Carry);
        } else {
            self.reg.unset_flag(Flag::Carry);
        }
        self.reg.unset_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);

        if r == 0 {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
        r
    }

    fn alu_bit(&mut self, a: u8, b: u8) {
        // println!("alu bit op {} and {}", a, b);
        let r = a & (1 << b) == 0x00;
        self.reg.set_flag(Flag::HalfCarry);
        self.reg.unset_flag(Flag::Sub);

        if r {
            self.reg.set_flag(Flag::Zero);
        } else {
            self.reg.unset_flag(Flag::Zero);
        }
    }

    fn alu_res(&mut self, a: u8, b: u8) -> u8 {
        a & !(1 << b)
    }

    fn alu_set(&mut self, a: u8, b: u8) -> u8 {
        a | (1 << b)
    }

    fn get_setter(&'a mut self, i: u8) -> Box<dyn FnMut(u8) + 'a> {
        match i {
            0 => Box::new(move |v: u8| self.reg.set_B(v)),
            1 => Box::new(move |v: u8| self.reg.set_C(v)),
            2 => Box::new(move |v: u8| self.reg.set_D(v)),
            3 => Box::new(move |v: u8| self.reg.set_E(v)),
            4 => Box::new(move |v: u8| self.reg.set_H(v)),
            5 => Box::new(move |v: u8| self.reg.set_L(v)),
            6 => Box::new(move |v: u8| self.write_byte_to_memory(self.reg.get_HL(), v)),

            7 => Box::new(move |v: u8| self.reg.set_A(v)),
            _ => unreachable!(),
        }
    }

    // 	OP:0xCB PREFIX CB
    pub fn op_0xCB(&mut self) -> u32 {
        // nextIns := core.getParameter8()
        // if core.cbMap[nextIns] != nil {
        //     core.cbMap[nextIns]()
        //     return CBCycles[nextIns] * 4
        // } else {
        //     log.Fatalf("Undefined CB Opcode: %X \n", nextIns)
        // }
        // return 0

        // 0
        let next_op = self.read_byte_from_memory(self.reg.get_PC());
        // println!("fuck cb opcode is {}", next_op);
        self.reg.incr_PC();

        insert_cpu_record(CPUDebugInfo::new(self.reg.clone(), next_op, true));

        #[allow(unused_assignments)] // it will be orverwirte
        let mut v = 0;

        let row = next_op / 8;
        let col = next_op % 8;

        {
            let getters: Vec<Box<dyn Fn() -> u8>> = vec![
                Box::new(|| self.reg.get_B()),
                Box::new(|| self.reg.get_C()),
                Box::new(|| self.reg.get_D()),
                Box::new(|| self.reg.get_E()),
                Box::new(|| self.reg.get_H()),
                Box::new(|| self.reg.get_L()),
                Box::new(|| self.read_byte_from_memory(self.reg.get_HL())),
                Box::new(|| self.reg.get_A()),
            ];
            v = getters[col as usize]();
        }

        // let mut setter: Box<dyn FnMut(u8)> = match col {
        //     0 => Box::new(|v: u8| self.reg.set_B(v)),
        //     1 => Box::new(|v: u8| self.reg.set_C(v)),
        //     2 => Box::new(|v: u8| self.reg.set_D(v)),
        //     3 => Box::new(|v: u8| self.reg.set_E(v)),
        //     4 => Box::new(|v: u8| self.reg.set_H(v)),
        //     5 => Box::new(|v: u8| self.reg.set_L(v)),
        //     6 => Box::new(|v: u8| self.write_byte_to_memory(self.reg.get_HL(), v)),

        //     7 => Box::new(|v: u8| self.reg.set_A(v)),
        //     _ => unreachable!(),
        // };

        match row {
            0x00 => {
                let v = self.alu_rlc(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x01 => {
                let v = self.alu_rrc(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x02 => {
                let v = self.alu_rl(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x03 => {
                let v = self.alu_rr(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x04 => {
                let v = self.alu_sla(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }

            0x05 => {
                let v = self.alu_sra(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x06 => {
                let v = self.alu_swap(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x07 => {
                let v = self.alu_srl(v);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x08 => {
                self.alu_bit(v, 0);
            }
            0x09 => {
                self.alu_bit(v, 1);
            }
            0x0A => {
                self.alu_bit(v, 2);
            }
            0x0B => {
                self.alu_bit(v, 3);
            }
            0x0C => {
                self.alu_bit(v, 4);
            }
            0x0D => {
                self.alu_bit(v, 5);
            }
            0x0E => {
                self.alu_bit(v, 6);
            }
            0x0F => {
                self.alu_bit(v, 7);
            }
            0x10 => {
                let v = self.alu_res(v, 0);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x11 => {
                let v = self.alu_res(v, 1);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x12 => {
                let v = self.alu_res(v, 2);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x13 => {
                let v = self.alu_res(v, 3);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x14 => {
                let v = self.alu_res(v, 4);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x15 => {
                let v = self.alu_res(v, 5);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x16 => {
                let v = self.alu_res(v, 6);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x17 => {
                let v = self.alu_res(v, 7);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x18 => {
                let v = self.alu_set(v, 0);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x19 => {
                let v = self.alu_set(v, 1);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x1A => {
                let v = self.alu_set(v, 2);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x1B => {
                let v = self.alu_set(v, 3);
                let mut setter = self.get_setter(col);
                setter(v);
            }

            0x1C => {
                let v = self.alu_set(v, 4);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x1D => {
                let v = self.alu_set(v, 5);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x1E => {
                let v = self.alu_set(v, 6);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            0x1F => {
                let v = self.alu_set(v, 7);
                let mut setter = self.get_setter(col);
                setter(v);
            }
            _ => {
                unreachable!("{:#02x}", row);
            }
        }

        let ex_op_cycles = vec![
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 1
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 2
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 3
            2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 4
            2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 5
            2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 6
            2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 7
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 8
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 9
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // A
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // B
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // C
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // D
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // E
            2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // F
        ]; //0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f

        // TODO
        return ex_op_cycles[next_op as usize] * 2;
    }
}
