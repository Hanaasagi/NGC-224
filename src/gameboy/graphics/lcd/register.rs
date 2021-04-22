use crate::gameboy::util::clear_bit;
use crate::gameboy::util::set_bit;
use crate::gameboy::util::test_bit;

/// 0: During H-Blank
/// 1: During V-Blank
/// 2: During Searching OAM
/// 3: During Transferring Data to LCD Driver
#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LCDMode {
    HBlank = 0b00,
    VBlank = 0b01,
    OAM = 0b10,
    VRAM = 0b11,
}

impl From<u8> for LCDMode {
    fn from(v: u8) -> Self {
        if v == 0b00 {
            Self::HBlank
        } else if v == 0b01 {
            Self::VBlank
        } else if v == 0b10 {
            Self::OAM
        } else if v == 0b11 {
            Self::VRAM
        } else {
            unreachable!("Invaild u8 value: {} for LCDMode", v);
        }
    }
}

/// LCD Status Register.
/// Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
/// Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
/// Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
/// Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
/// Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
///    0: During H-Blank
///    1: During V-Blank
///    2: During Searching OAM
///    3: During Transferring Data to LCD Driver
#[derive(Debug)]
pub struct LCDStatusRegister {
    mode: u8,
}

impl LCDStatusRegister {
    pub fn new() -> Self {
        Self { mode: 0x00 }
    }

    /// Get the raw value of register.
    pub fn get_value(&self) -> u8 {
        self.mode
    }

    /// Get the mode.
    pub fn get_mode(&self) -> LCDMode {
        LCDMode::from(self.mode & 0b11)
    }

    /// Set the mode
    pub fn set_mode(&mut self, mode: LCDMode) {
        // clean lower 2 bit
        self.mode &= 0b1111_1100;
        // set lower 2 bit
        self.mode |= mode as u8;
    }

    /// Check whether the m0 interrupt is enabled.
    pub fn is_m0_interrupt_enabled(&self) -> bool {
        test_bit(self.mode, 3)
    }

    /// Disable the m0 interrupt
    pub fn disable_m0_interrupt(&mut self) {
        self.mode = clear_bit(self.mode, 3);
    }

    /// Enable the m0 interrupt
    pub fn enable_m0_interrupt(&mut self) {
        self.mode = set_bit(self.mode, 3);
    }

    /// Check whether the m1 interrupt is enabled.
    pub fn is_m1_interrupt_enabled(&self) -> bool {
        test_bit(self.mode, 4)
    }

    /// Disable the m1 interrupt
    pub fn disable_m1_interrupt(&mut self) {
        self.mode = clear_bit(self.mode, 4);
    }

    /// Enable the m1 interrupt
    pub fn enable_m1_interrupt(&mut self) {
        self.mode = set_bit(self.mode, 4);
    }

    /// Check whether the m2 interrupt is enabled.
    pub fn is_m2_interrupt_enabled(&self) -> bool {
        test_bit(self.mode, 5)
    }

    /// Disable the m2 interrupt
    pub fn disable_m2_interrupt(&mut self) {
        self.mode = clear_bit(self.mode, 5);
    }

    /// Enable the m2 interrupt
    pub fn enable_m2_interrupt(&mut self) {
        self.mode = set_bit(self.mode, 5);
    }

    /// Check whether the ly interrupt is enabled.
    pub fn is_ly_interrupt_enabled(&self) -> bool {
        test_bit(self.mode, 6)
    }

    /// Disable the ly interrupt
    pub fn disable_ly_interrupt(&mut self) {
        self.mode = clear_bit(self.mode, 6);
    }

    /// Enable the ly interrupt
    pub fn enable_ly_interrupt(&mut self) {
        self.mode = set_bit(self.mode, 6);
    }
}

/// Reference: https://gbdev.gg8.se/wiki/articles/LCDC
/// LCDC is the main LCD Control register. Its bits toggle what elements are displayed on the screen, and how.
pub struct LCDControllerRegister {
    reg: u8,
}

impl LCDControllerRegister {
    pub fn new() -> Self {
        Self { reg: 0b0100_1000 }
    }

    /// Get the raw value in LCDC register.
    pub fn get_value(&self) -> u8 {
        self.reg
    }

    /// Set the raw value in LCDC register.
    pub fn set_value(&mut self, v: u8) {
        self.reg = v;
    }

    /// LCDC.7 - LCD Display Enable
    /// This bit controls whether the LCD is on and the PPU is active. Setting it to 0 turns both off,
    /// which grants immediate and full access to VRAM, OAM, etc.
    pub fn is_lcd_enabled(&self) -> bool {
        test_bit(self.reg, 7)
    }

    /// LCDC.6 - Window Tile Map Display Select
    /// This bit controls which background map the Window uses for rendering. When it's reset, the $9800 tilemap is used,
    /// otherwise it's the $9C00 one.
    pub fn get_window_tilemap_addr(&self) -> u16 {
        if test_bit(self.reg, 6) {
            0x9c00
        } else {
            0x9800
        }
    }

    /// LCDC.5 - Window Display Enable
    /// This bit controls whether the window shall be displayed or not. (TODO : what happens when toggling this
    /// mid-scanline ?) This bit is overridden on DMG by bit 0 if that bit is reset.
    /// Note that on CGB models, setting this bit to 0 then back to 1 mid-frame may cause the second write to be ignored.
    pub fn is_window_enabled(&self) -> bool {
        test_bit(self.reg, 5)
    }

    /// LCDC.4 - BG & Window Tile Data Select
    /// This bit controls which addressing mode the BG and Window use to pick tiles.
    /// Sprites aren't affected by this, and will always use $8000 addressing mode.
    pub fn get_tile_data_base_addr(&self) -> (u16, bool) {
        if test_bit(self.reg, 4) {
            (0x8000, true) // unsigned
        } else {
            (0x8800, false) // This memory region uses signed bytes as tile identifiers
        }
    }

    /// LCDC.3 - BG Tile Map Display Select
    /// This bit works similarly to bit 6: if the bit is reset, the BG uses tilemap $9800, otherwise tilemap $9C00.
    pub fn get_bg_tilemap_addr(&self) -> u16 {
        // Each byte in the memory region is a tile identification number of what needs to be drawn.
        // This identification number is used to lookup the tile data in video ram so we know how to draw it.
        if test_bit(self.reg, 3) {
            0x9c00
        } else {
            0x9800
        }
    }

    /// LCDC.2 - OBJ Size
    /// This bit controls the sprite size (1 tile or 2 stacked vertically).
    /// Be cautious when changing this mid-frame from 8x8 to 8x16 : "remnants" of the sprites intended for 8x8 could
    /// "leak" into the 8x16 zone and cause artifacts.
    pub fn get_sprite_size(&self) -> (u8, u8) {
        if test_bit(self.reg, 2) {
            (8, 16) // 8 * 16
        } else {
            (8, 8) // 8 * 8
        }
    }

    /// LCDC.1 - OBJ Display Enable
    /// This bit toggles whether sprites are displayed or not.
    /// This can be toggled mid-frame, for example to avoid sprites being displayed on top of a status bar or text box.
    /// (Note: toggling mid-scanline might have funky results on DMG? Investigation needed.)
    pub fn is_sprite_enabled(&self) -> bool {
        test_bit(self.reg, 1)
    }

    /// LCDC.0 - BG/Window Display/Priority
    /// LCDC.0 has different meanings depending on Gameboy type and Mode:
    /// Monochrome Gameboy, SGB and CGB in Non-CGB Mode: BG Display
    /// When Bit 0 is cleared, both background and window become blank (white), and the Window Display Bit is ignored in
    /// that case. Only Sprites may still be displayed (if enabled in Bit 1).
    /// CGB in CGB Mode: BG and Window Master Priority
    /// When Bit 0 is cleared, the background and window lose their priority - the sprites will be always displayed on
    /// top of background and window, independently of the priority flags in OAM and BG Map attributes.
    pub fn bg_display(&self) -> bool {
        test_bit(self.reg, 0)
    }
}

// TODO:
// **** CGB only, currently not using ****
/// This register is used to address a byte in the CGBs Background Palette Memory.
/// Each two byte in that memory define a
/// color value. The first 8 bytes define Color 0-3 of Palette 0 (BGP0), and so on for BGP1-7.
///  Bit 0-5   Index (00-3F)
///  Bit 7     Auto Increment  (0=Disabled, 1=Increment after Writing)
/// Data can be read/written to/from the specified index address through Register FF69.
/// When the Auto Increment bit is set then the index is automatically incremented after each <write> to FF69.
/// Auto Increment has no effect when <reading> from FF69, so the index must be manually incremented in that case.
/// Writing to FF69 during rendering still causes auto-increment to occur.
/// Unlike the following, this register can be accessed outside V-Blank and H-Blank.
pub struct BGPI {
    reg: u8,
}

impl BGPI {
    pub fn new() -> Self {
        BGPI { reg: 0 }
    }

    pub fn get_value(&self) -> u8 {
        self.reg
    }

    pub fn get_index(&self) -> u8 {
        self.reg & 0b0011_1111
    }

    pub fn select(&mut self, val: u8) {
        self.reg = val;
    }

    fn is_auto_incr(&self) -> bool {
        test_bit(self.reg, 7)
    }

    pub fn on_write(&mut self) {
        if self.is_auto_incr() {
            let mut index = self.reg & 0b0011_1111;
            index = (index + 1) % 0x40;
            self.reg = self.reg & 0b1100_0000;
            self.reg |= index;
        }
    }
}
