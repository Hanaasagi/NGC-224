use std::cell::RefCell;
use std::convert::From;
use std::rc::Rc;

use super::cpu::IntFlag;
use super::cpu::IntReg;
use super::IOHandler;

const SELECT_FUNC_KEY_MASK: u8 = 0b0010_0000;
const SELECT_DIRECTION_KEY_MASK: u8 = 0b0001_0000;

#[derive(Clone, PartialEq, Debug)]
pub enum JoypadKey {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

bitflags! {
    /// The eight gameboy buttons/direction keys are arranged in form of a 2x4 matrix. Select either button or direction
    /// keys by writing to this register, then read-out bit 0-3.
    ///
    /// FF00 - P1/JOYP - Joypad (R/W)
    ///
    /// Bit 7 - Not used
    /// Bit 6 - Not used
    /// Bit 5 - P15 Select Button Keys      (0=Select)
    /// Bit 4 - P14 Select Direction Keys   (0=Select)
    /// Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
    /// Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
    /// Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
    /// Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
    ///
    /// Bits 0-3 are set by the emulator to show the state of the joypad.
    /// As you can see the directional buttons and the standard buttons share this range of bits
    /// so how would the game know if bit 3 was set whether it was the directional down button
    /// or the stanadrd start button? The way this works is that the game sets bit 4 and 5 depending on
    /// whether it wants to check on the directional buttons or the standard buttons.
    pub struct JoypadKeyMask: u8 {
        const RIGHT  = 0b0000_0001;
        const LEFT   = 0b0000_0010;
        const UP     = 0b0000_0100;
        const DOWN   = 0b0000_1000;
        const A      = 0b0000_0001;
        const B      = 0b0000_0010;
        const SELECT = 0b0000_0100;
        const START  = 0b0000_1000;
    }
}

impl From<JoypadKey> for JoypadKeyMask {
    fn from(key: JoypadKey) -> Self {
        match key {
            JoypadKey::Right => Self::RIGHT,
            JoypadKey::Left => Self::LEFT,
            JoypadKey::Up => Self::UP,
            JoypadKey::Down => Self::DOWN,
            JoypadKey::A => Self::A,
            JoypadKey::B => Self::B,
            JoypadKey::Select => Self::SELECT,
            JoypadKey::Start => Self::START,
        }
    }
}

pub struct Joypad {
    intf: Rc<RefCell<IntReg>>,
    reg: u8,
    // The cpu tell us what should be select, direction key or func key.
    select_mask: u8,
}

impl Joypad {
    pub fn new(intf: Rc<RefCell<IntReg>>) -> Self {
        Self {
            intf,
            reg: 0xff,
            select_mask: 0xff,
        }
    }
}

impl Joypad {
    pub fn keydown(&mut self, key: JoypadKey) {
        let keys: [JoypadKey; 4] = [
            JoypadKey::Right,
            JoypadKey::Left,
            JoypadKey::Up,
            JoypadKey::Down,
        ];
        if keys.contains(&key) {
            self.reg &= !SELECT_DIRECTION_KEY_MASK;
        } else {
            self.reg &= !SELECT_FUNC_KEY_MASK;
        }

        self.reg &= !((JoypadKeyMask::from(key.clone())).bits());
        self.intf.borrow_mut().req(IntFlag::Joypad);
    }

    pub fn keyup(&mut self, key: JoypadKey) {
        let keys: [JoypadKey; 4] = [
            JoypadKey::Right,
            JoypadKey::Left,
            JoypadKey::Up,
            JoypadKey::Down,
        ];
        if keys.contains(&key) {
            self.reg |= SELECT_DIRECTION_KEY_MASK;
        } else {
            self.reg |= SELECT_FUNC_KEY_MASK;
        }

        self.reg |= (JoypadKeyMask::from(key)).bits();
    }
}

impl IOHandler for Joypad {
    fn read_byte(&self, _: u16) -> u8 {
        if (self.select_mask & SELECT_DIRECTION_KEY_MASK) == 0 {
            if self.reg & SELECT_DIRECTION_KEY_MASK == 0 {
                return self.reg;
            } else {
                return 0xff;
            }
        }
        if (self.select_mask & SELECT_FUNC_KEY_MASK) == 0 {
            if self.reg & SELECT_FUNC_KEY_MASK == 0 {
                return self.reg;
            } else {
                return 0xff;
            }
        }

        0xff
    }

    // Reference: http://www.codeslinger.co.uk/pages/projects/gameboy/joypad.html
    // The way I believe this works in the original gameboy hardware is the game writes to memory 0xFF00 with bit 4 or 5 set (never both).
    // It then reads from memory 0xFF00 and instead of reading back what it just wrote what is returned is
    // the state of the joypad based on whether bit 4 or bit 5 was set.
    // For example if the game wanted to check which directional buttons was pressed it would set bit 4 to 1
    // and then it would do a read memory on 0xFF00.
    // If the up key is pressed then when reading 0xFF00 bit 2 would be set to 0 to signal
    // that the directional button up is pressed (0 means pressed, 1 unpressed).
    // However if up was not pressed but the select button was then bit 2 would be left at 1 meaning nothing is pressed,
    // even though the select button is pressed which maps on to bit 2.
    // The reason why bit 2 would be set to 1 signalling it is not pressed
    // even when it is is because bit 4 was set to 1 meaning the game is only interested in the state of the directional buttons.
    fn write_byte(&mut self, _: u16, v: u8) {
        // 0b0010_0000 (32)
        // 0b0001_0000 (16)
        self.select_mask = v;
    }
}
