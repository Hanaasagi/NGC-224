use std::cell::RefCell;
use std::rc::Rc;

use super::cartridge::Cartridge;
use super::cpu::IntReg;
use super::graphics::gpu::GPU;
use super::joypad::Joypad;
use super::timer::Timer;
// use std::fmt::Debug;

pub trait IOHandler {
    /// Read a byte.
    fn read_byte(&self, a: u16) -> u8;

    /// Write a byte.
    fn write_byte(&mut self, a: u16, v: u8);

    /// Read a double byte.
    fn read_word(&self, a: u16) -> u16 {
        u16::from(self.read_byte(a)) | (u16::from(self.read_byte(a + 1)) << 8)
    }

    /// Write a double byte.
    fn write_word(&mut self, a: u16, v: u16) {
        self.write_byte(a, (v & 0xFF) as u8);
        self.write_byte(a + 1, (v >> 8) as u8)
    }
}

///
/// Start       End     Description                     Notes
/// 0000        3FFF    16KB ROM bank 00                From cartridge, usually a fixed bank
/// 4000        7FFF    16KB ROM Bank 01~NN             From cartridge, switchable bank via MBC (if any)
/// 8000        9FFF    8KB Video RAM (VRAM)            Only bank 0 in Non-CGB mode. Switchable bank 0/1 in CGB mode
/// A000        BFFF    8KB External RAM                In cartridge, switchable bank if any
/// C000        CFFF    4KB Work RAM (WRAM) bank 0
/// D000        DFFF    4KB Work RAM (WRAM) bank 1~N    Only bank 1 in Non-CGB mode. Switchable bank 1~7 in CGB mode
/// E000        FDFF    Mirror of C000~DDFF (ECHO RAM)  Typically not used
/// FE00        FE9F    Sprite attribute table (OAM)
/// FEA0        FEFF    Not Usable
/// FF00        FF7F    I/O Registers
/// FF80        FFFE    High RAM (HRAM)
/// FFFF        FFFF    Interrupts Enable Register (IE)
///
pub struct Mmunit {
    pub cartridge: Box<dyn Cartridge>,
    // TODO: apu
    pub gpu: Rc<RefCell<GPU>>,
    pub joypad: Joypad,
    pub timer: Timer,
    // Interrupts Enable Register (IE)
    inte: u8,
    intf: Rc<RefCell<IntReg>>,
    // High ram
    hram: [u8; 0x7f],
    // Work ram
    wram: [u8; 0x8000],
    // CGB wram bank mapping 0xFF70
    wram_bank: usize,
}

impl Mmunit {
    pub fn new(
        cart: Box<dyn Cartridge>,
        gpu: Rc<RefCell<GPU>>,
        joypad: Joypad,
        timer: Timer,
        intf: Rc<RefCell<IntReg>>,
    ) -> Self {
        let mut r = Self {
            cartridge: cart,
            gpu,
            joypad,
            timer,
            intf,
            inte: 0x00,
            hram: [0x00; 0x7f],
            wram: [0x00; 0x8000],
            wram_bank: 0x01,
        };
        r.set_initial();
        r
    }

    // https://gbdev.gg8.se/wiki/articles/Power_Up_Sequence
    // [$FF05] = $00   ; TIMA
    // [$FF06] = $00   ; TMA
    // [$FF07] = $00   ; TAC
    // [$FF10] = $80   ; NR10
    // [$FF11] = $BF   ; NR11
    // [$FF12] = $F3   ; NR12
    // [$FF14] = $BF   ; NR14
    // [$FF16] = $3F   ; NR21
    // [$FF17] = $00   ; NR22
    // [$FF19] = $BF   ; NR24
    // [$FF1A] = $7F   ; NR30
    // [$FF1B] = $FF   ; NR31
    // [$FF1C] = $9F   ; NR32
    // [$FF1E] = $BF   ; NR33
    // [$FF20] = $FF   ; NR41
    // [$FF21] = $00   ; NR42
    // [$FF22] = $00   ; NR43
    // [$FF23] = $BF   ; NR44
    // [$FF24] = $77   ; NR50
    // [$FF25] = $F3   ; NR51
    // [$FF26] = $F1-GB, $F0-SGB ; NR52
    // [$FF40] = $91   ; LCDC
    // [$FF42] = $00   ; SCY
    // [$FF43] = $00   ; SCX
    // [$FF45] = $00   ; LYC
    // [$FF47] = $FC   ; BGP
    // [$FF48] = $FF   ; OBP0
    // [$FF49] = $FF   ; OBP1
    // [$FF4A] = $00   ; WY
    // [$FF4B] = $00   ; WX
    // [$FFFF] = $00   ; IE
    fn set_initial(&mut self) {
        self.write_byte(0xff05, 0x00);
        self.write_byte(0xff06, 0x00);
        self.write_byte(0xff07, 0x00);
        self.write_byte(0xff10, 0x80);
        self.write_byte(0xff11, 0xbf);
        self.write_byte(0xff12, 0xf3);
        self.write_byte(0xff14, 0xbf);
        self.write_byte(0xff16, 0x3f);
        self.write_byte(0xff17, 0x00);
        self.write_byte(0xff19, 0xbf);
        self.write_byte(0xff1a, 0x7f);
        self.write_byte(0xff1b, 0xff);
        self.write_byte(0xff1c, 0x9f);
        self.write_byte(0xff1e, 0xff);
        self.write_byte(0xff20, 0xff);
        self.write_byte(0xff21, 0x00);
        self.write_byte(0xff22, 0x00);
        self.write_byte(0xff23, 0xbf);
        self.write_byte(0xff24, 0x77);
        self.write_byte(0xff25, 0xf3);
        self.write_byte(0xff26, 0xf1);
        self.write_byte(0xff40, 0x91);
        self.write_byte(0xff42, 0x00);
        self.write_byte(0xff43, 0x00);
        self.write_byte(0xff45, 0x00);
        self.write_byte(0xff47, 0xfc);
        self.write_byte(0xff48, 0xff);
        self.write_byte(0xff49, 0xff);
        self.write_byte(0xff4a, 0x00);
        self.write_byte(0xff4b, 0x00);
        // IE is a struct, use it's own init logic.
    }
}

impl Mmunit {
    pub fn next(&mut self, cycles: u32) -> u32 {
        self.timer.next(cycles);
        self.gpu.borrow_mut().next(cycles);
        cycles
    }
}

impl IOHandler for Mmunit {
    fn read_byte(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x7fff => self.cartridge.read_byte(a),
            0x8000..=0x9fff => self.gpu.borrow().read_byte(a),
            0xa000..=0xbfff => self.cartridge.read_byte(a),
            0xc000..=0xcfff => self.wram[a as usize - 0xc000],
            0xd000..=0xdfff => self.wram[a as usize - 0xd000 + 0x1000 * self.wram_bank],
            0xe000..=0xefff => self.wram[a as usize - 0xe000],
            0xf000..=0xfdff => self.wram[a as usize - 0xf000 + 0x1000 * self.wram_bank],
            0xfe00..=0xfe9f => self.gpu.borrow().read_byte(a),
            0xfea0..=0xfeff => 0x00,
            0xff00 => self.joypad.read_byte(a),
            0xff01..=0xff02 => 0x00, // TODO: serial
            0xff04..=0xff07 => self.timer.get(a),
            0xff0f => self.intf.borrow().data,
            0xff10..=0xff3f => 0x00, // TODO: APU
            0xff4d => 0x00,          // FF4D - KEY1 - CGB Mode Only - Prepare Speed Switch
            0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f => self.gpu.borrow().read_byte(a),
            0xff51..=0xff55 => 0x00, // HDMA CGB
            0xff68..=0xff6b => self.gpu.borrow().read_byte(a),
            0xff70 => self.wram_bank as u8,
            0xff80..=0xfffe => self.hram[a as usize - 0xff80],
            0xffff => self.inte,
            _ => 0x00,
        }
    }

    fn write_byte(&mut self, a: u16, v: u8) {
        // if a == 65348 {
        //     debug!("mmu write byte hook 65348 => {}", v);
        // }
        match a {
            0x0000..=0x7fff => self.cartridge.write_byte(a, v),
            0x8000..=0x9fff => {
                self.gpu.borrow_mut().write_byte(a, v);
            }
            0xa000..=0xbfff => self.cartridge.write_byte(a, v),
            0xc000..=0xcfff => self.wram[a as usize - 0xc000] = v,
            0xd000..=0xdfff => self.wram[a as usize - 0xd000 + 0x1000 * self.wram_bank] = v,
            0xe000..=0xefff => self.wram[a as usize - 0xe000] = v,
            0xf000..=0xfdff => self.wram[a as usize - 0xf000 + 0x1000 * self.wram_bank] = v,
            0xfe00..=0xfe9f => self.gpu.borrow_mut().write_byte(a, v),
            0xfea0..=0xfeff => {}
            0xff00 => self.joypad.write_byte(a, v),
            0xff01..=0xff02 => {} // TODO: serial
            0xff04..=0xff07 => self.timer.set(a, v),
            0xff10..=0xff3f => {} // TODO: apu
            0xff46 => {
                // DMA
                // http://www.codeslinger.co.uk/pages/projects/gameboy/dma.html
                // See: http://gbdev.gg8.se/wiki/articles/Video_Display#FF46_-_DMA_-_DMA_Transfer_and_Start_Address_.28R.2FW.29
                let base_addr = u16::from(v) << 8;
                for i in 0..0xa0 {
                    let b = self.read_byte(base_addr + i);
                    self.write_byte(0xfe00 + i, b);
                }
            }
            0xff4d => {} // FF4D - KEY1 - CGB Mode Only - Prepare Speed Switch
            0xff40..=0xff45 | 0xff47..=0xff4b | 0xff4f => self.gpu.borrow_mut().write_byte(a, v),
            0xff51..=0xff55 => {} //
            0xff68..=0xff6b => self.gpu.borrow_mut().write_byte(a, v),
            0xff0f => self.intf.borrow_mut().data = v,
            0xff70 => {
                // In CGB Mode 32 KBytes internal RAM are available.
                // This memory is divided into 8 banks of 4 KBytes each.
                // Bank 0 is always available in memory at C000-CFFF,
                // Bank 1-7 can be selected into the address space at D000-DFFF.
                // Writing a value of 01h-07h will select Bank 1-7, writing a value of 00h will select Bank 1 either.
                self.wram_bank = match v & 0x7 {
                    0 => 1,
                    n => n as usize,
                };
            }
            0xff80..=0xfffe => self.hram[a as usize - 0xff80] = v,
            0xffff => self.inte = v,
            _ => {}
        }
    }
}
