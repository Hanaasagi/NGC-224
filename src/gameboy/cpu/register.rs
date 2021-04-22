use super::super::Term;

/// # The CPU Registers.
/// Most registers can be accessed either as one 16bit register, or as two separate 8bit registers.
/// ```ignore
/// 16bit Hi   Lo   Name/Function
/// AF    A    F    Accumulator & Flags
/// BC    B    C    BC
/// DE    D    E    DE
/// HL    H    L    HL
/// SP    -    -    Stack Pointer
/// PC    -    -    Program Counter/Pointer
/// ```
///
/// # The Flag Registers.
/// The lower 8bit of AF register is the flag register.
/// ```ignore
/// Bit  Name  Set Clr  Expl.
/// 7    zf    Z   NZ   Zero Flag
/// 6    n     -   -    Add/Sub-Flag (BCD)
/// 5    h     -   -    Half Carry Flag (BCD)
/// 4    cy    C   NC   Carry Flag
/// 3-0  -     -   -    Not used (always zero)
/// ```
///
///
/// ## The Zero Flag (Z)
/// This bit becomes set (1) if the result of an operation has been zero (0). Used for conditional jumps.
///
/// ## The Carry Flag (C, or Cy)
/// Becomes set when the result of an addition became bigger than FFh (8bit) or FFFFh (16bit).
/// Or when the result of a subtraction or comparision became less than zero.
///
/// ## The BCD Flags (N, H)
/// These flags are (rarely) used for the DAA instruction only,
/// N Indicates whether the previous instruction has been an addition or subtraction,
/// and H indicates carry for lower 4bits of the result,
/// also for DAA, the C flag must indicate carry for upper 8bits.
/// After adding/subtracting two BCD numbers,
/// DAA is intended to convert the result into BCD format;
/// BCD numbers are ranged from 00h to 99h rather than 00h to FFh.
/// Because C and H flags must contain carry-outs for each digit,
/// DAA cannot be used for 16bit operations (which have 4 digits),
/// or for INC/DEC operations (which do not affect C-flag).
///
///
/// # Reference:
/// - [CPU Registers and Flags](https://gbdev.gg8.se/wiki/articles/CPU_Registers_and_Flags)
#[derive(Default, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Register {
    A: u8,
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    F: u8,
    H: u8,
    L: u8,
    PC: u16,
    SP: u16,
}

impl Register {
    /// Returns a new CPU register group.

    pub fn new() -> Self {
        Self::default()
    }

    // #[cfg(test)]
    pub fn new_from_debug_string(s: &str) -> Self {
        let mut reg = Self::new();
        // a trick way, but work for now.
        let s = &s[11..s.len() - 2];
        for field in s.split(", ") {
            let tmp: Vec<&str> = field.split(": ").collect();
            let name = tmp[0].to_uppercase();
            let value: u16 = tmp[1].parse().unwrap();
            match &name as &str {
                "A" => reg.A = value as u8,
                "B" => reg.B = value as u8,
                "C" => reg.C = value as u8,
                "D" => reg.D = value as u8,
                "E" => reg.E = value as u8,
                "F" => reg.F = value as u8,
                "H" => reg.H = value as u8,
                "L" => reg.L = value as u8,
                "PC" => reg.PC = value,
                "SP" => reg.SP = value,
                _ => panic!("{} is unknown field", name),
            }
        }

        reg
    }

    /// Init the register, fill the default value.
    /// ```ignore
    /// name value(hex)
    /// AF   01B0
    /// BC 	 0013
    /// DE 	 00D8
    /// HL 	 014D
    /// SP 	 FFFE
    /// PC 	 0100
    /// ```
    /// Find more detail [here](https://mstojcevich.github.io/post/d-gb-emu-registers/)
    pub fn init(&mut self, term: Term) {
        match term {
            Term::GB => {
                self.A = 0x01;
            }
            Term::GBP => {
                self.A = 0xff;
            }
            Term::GBC => {
                self.A = 0x11;
            }
            Term::SGB => {
                self.A = 0x01;
            }
        }

        // self.A = 0x01;
        self.B = 0x00;
        self.C = 0x13;
        self.D = 0x00;
        self.E = 0xD8;
        self.F = 0xB0;
        self.H = 0x01;
        self.L = 0x4D;
        // After displaying the Nintendo Logo,
        // the built-in boot procedure jumps to this address (100h), which should then jump to the actual main program in the cartridge.
        // Usually this 4 byte area contains a NOP instruction, followed by a JP 0150h instruction. But not always.
        self.PC = 0x0100;
        self.SP = 0xFFFE;
    }
}

// All Getter Methods.
#[allow(non_snake_case)]
impl Register {
    /// Returns the value of `A` register.
    #[inline]
    pub fn get_A(&self) -> u8 {
        self.A
    }

    /// Returns the value of `B` register.
    #[inline]
    pub fn get_B(&self) -> u8 {
        self.B
    }

    /// Returns the value of `C` register.
    #[inline]
    pub fn get_C(&self) -> u8 {
        self.C
    }

    /// Returns the value of `D` register.
    #[inline]
    pub fn get_D(&self) -> u8 {
        self.D
    }

    /// Returns the value of `E` register.
    #[inline]
    pub fn get_E(&self) -> u8 {
        self.E
    }

    /// Returns the value of `H` register.
    #[inline]
    pub fn get_H(&self) -> u8 {
        self.H
    }

    /// Returns the value of `L` register.
    #[inline]
    pub fn get_L(&self) -> u8 {
        self.L
    }

    /// Returns the value of `PC` register.
    #[inline]
    pub fn get_PC(&self) -> u16 {
        self.PC
    }

    /// Returns the value of `SP` register.
    #[inline]
    pub fn get_SP(&self) -> u16 {
        self.SP
    }

    /// Incr the value of `PC` register.
    #[inline]
    pub fn incr_PC(&mut self) {
        self.PC += 1;
    }

    /// Returns the value of 16bit `AF` register.
    #[inline]
    pub fn get_AF(&self) -> u16 {
        (u16::from(self.A) << 8) | u16::from(self.F)
    }

    /// Returns the value of 16bit `BC` register.
    #[inline]
    pub fn get_BC(&self) -> u16 {
        (u16::from(self.B) << 8) | u16::from(self.C)
    }

    /// Returns the value of 16bit `DE` register.
    #[inline]
    pub fn get_DE(&self) -> u16 {
        (u16::from(self.D) << 8) | u16::from(self.E)
    }

    /// Returns the value of 16bit `HL` register.
    #[inline]
    pub fn get_HL(&self) -> u16 {
        (u16::from(self.H) << 8) | u16::from(self.L)
    }
}

// All Setter Methods.
#[allow(non_snake_case)]
impl Register {
    /// Set the value of `A` register.
    #[inline]
    pub fn set_A(&mut self, v: u8) {
        self.A = v
    }

    /// Set the value of `B` register.
    #[inline]
    pub fn set_B(&mut self, v: u8) {
        self.B = v
    }

    /// Set the value of `C` register.
    #[inline]
    pub fn set_C(&mut self, v: u8) {
        self.C = v
    }

    /// Set the value of `D` register.
    #[inline]
    pub fn set_D(&mut self, v: u8) {
        self.D = v
    }

    /// Set the value of `E` register.
    #[inline]
    pub fn set_E(&mut self, v: u8) {
        self.E = v
    }

    /// Set the value of `H` register.
    #[inline]
    pub fn set_H(&mut self, v: u8) {
        self.H = v
    }

    /// Set the value of `L` register.
    #[inline]
    pub fn set_L(&mut self, v: u8) {
        self.L = v
    }

    /// Set the value of 16bit `AF` register.
    #[inline]
    pub fn set_AF(&mut self, v: u16) {
        self.A = (v >> 8) as u8;

        self.F = (v & 0x00f0) as u8;
    }

    /// Set the value of 16bit `BC` register.
    #[inline]
    pub fn set_BC(&mut self, v: u16) {
        self.B = (v >> 8) as u8;

        self.C = (v & 0x00ff) as u8;
    }

    /// Set the value of 16bit `DE` register.
    #[inline]
    pub fn set_DE(&mut self, v: u16) {
        self.D = (v >> 8) as u8;

        self.E = (v & 0x00ff) as u8;
    }

    /// Set the value of 16bit `HL` register.
    #[inline]
    pub fn set_HL(&mut self, v: u16) {
        self.H = (v >> 8) as u8;

        self.L = (v & 0x00ff) as u8;
    }

    /// Incr the value of 16bit `HL` register.
    #[inline]
    pub fn incr_HL(&mut self) {
        self.set_HL(self.get_HL().wrapping_add(1));
    }

    /// Set the value of 16bit `PC` register.
    #[inline]
    pub fn set_PC(&mut self, v: u16) {
        self.PC = v
    }

    /// Set the value of 16bit `SP` register.
    #[inline]
    pub fn set_SP(&mut self, v: u16) {
        self.SP = v
    }
}

// All Flag methods.
impl Register {
    /// Returns `true` if the `Flag` is set.
    #[inline]
    pub fn is_flag_set(&self, flag: Flag) -> bool {
        // use the mask to get the real flag here.
        (self.F & flag.value()) != 0
    }

    /// Set the Flag.
    pub fn set_flag(&mut self, flag: Flag) {
        self.F = self.F | (flag.value());
    }

    /// Unset the Flag.
    pub fn unset_flag(&mut self, flag: Flag) {
        self.F = self.F & !(flag.value())
    }

    /// Reverse the Flag.
    pub fn reverse_flag(&mut self, flag: Flag) {
        if self.is_flag_set(flag.clone()) {
            self.unset_flag(flag);
        } else {
            self.set_flag(flag);
        }
    }
}

// The Flag Register consists of the following bits: Z, N, H, C, 0, 0, 0, 0.
#[derive(Clone, Debug)]
pub enum Flag {
    // Zero Flag. This bit is set when the result of a math operationis zero or two values match when using the CP
    // instruction.
    Zero = 0b1000_0000,
    // Subtract Flag. This bit is set if a subtraction was performed in the last math instruction.
    Sub = 0b0100_0000,
    // Half Carry Flag. This bit is set if a carry occurred from the lowernibble in the last math operation.
    HalfCarry = 0b0010_0000,
    // Carry Flag. This bit is set if a carry occurred from the last math operation or if register A is the smaller
    // valuewhen executing the CP instruction.
    Carry = 0b0001_0000,
}

impl Flag {
    /// Returns the `u8` type value of flag. Actually it is using for mask.
    pub fn into_value(self) -> u8 {
        self as u8
    }

    /// Returns the `u8` type value of flag. Actually it is using for mask.
    pub fn value(&self) -> u8 {
        self.clone() as u8
    }
}

// FF0F - IF - Interrupt Flag (R/W)
// Bit 0: V-Blank  Interrupt Request (INT 40h)  (1=Request)
// Bit 1: LCD STAT Interrupt Request (INT 48h)  (1=Request)
// Bit 2: Timer    Interrupt Request (INT 50h)  (1=Request)
// Bit 3: Serial   Interrupt Request (INT 58h)  (1=Request)
// Bit 4: Joypad   Interrupt Request (INT 60h)  (1=Request)
#[rustfmt::skip]
#[derive(Clone)]
pub enum IntFlag {
    VBlank  = 0b0000,
    LCDStat = 0b0001,
    Timer   = 0b0010,
    Serial  = 0b0011,
    Joypad  = 0b0100,
}

pub struct IntReg {
    pub data: u8,
}

impl IntReg {
    pub fn new() -> Self {
        Self { data: 0x00 }
    }

    pub fn req(&mut self, flag: IntFlag) {
        self.data |= 1 << flag as u8;
    }
}
