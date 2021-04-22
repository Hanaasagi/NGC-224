use std::cell::RefCell;
use std::rc::Rc;

use super::cpu::IntFlag as Flag;
use super::cpu::IntReg;
use super::lcd::LCDControllerRegister;
use super::lcd::LCDMode;
use super::lcd::LCDStatusRegister;
use super::mmu::IOHandler;
use super::tile::{Attr, GBColor, Palette, TileLine};
use crate::gameboy::{SCREEN_H, SCREEN_W};

pub struct GPU {
    updated: bool,
    data: [[[u8; 3]; SCREEN_W]; SCREEN_H],

    lcdc: LCDControllerRegister,
    stat: LCDStatusRegister,
    /// Scroll Y (R/W), Scroll X (R/W)
    /// Specifies the position in the 256x256 pixels BG map (32x32 tiles) which is to be displayed at the upper/left LCD
    /// display position. Values in range from 0-255 may be used for X/Y each, the video controller automatically wraps
    /// back to the upper (left) position in BG map when drawing exceeds the lower (right) border of the BG map area.
    scroll_y: u8,
    scroll_x: u8,

    /// Specifies the upper/left positions of the Window area。
    /// The window becomes visible (if enabled) when positions are set in range WX=0..166, WY=0..143.
    /// A position of WX=7, WY=0 locates the window at upper left, it is then completely covering normal background.
    window_y: u8,
    window_x: u8,

    /// The LY indicates the vertical line to which the present data is transferred to the LCD Driver. The LY can take
    /// on any value between 0 through 153. The values between 144 and 153 indicate the V-Blank period. Writing will
    /// reset the counter.
    ly: u8,
    /// The Gameboy permanently compares the value of the LYC and LY registers. When both values are identical, the
    /// coincident bit in the STAT register becomes set, and (if enabled) a STAT interrupt is requested.
    lc: u8,

    /// This register assigns gray shades to the color numbers of the BG and Window tiles.
    bg_palette: u8,
    /// This register assigns gray shades for sprite palette 0. It works exactly as BGP (FF47), except that the lower
    /// two bits aren't used because sprite data 00 is transparent.
    obj_palette0: u8,
    /// This register assigns gray shades for sprite palette 1. It works exactly as BGP (FF47), except that the lower
    /// two bits aren't used because sprite data 00 is transparent.
    obj_palette1: u8,

    /// Ram
    ram: [u8; 0x4000],
    ram_bank: usize,
    // VRAM Sprite Attribute Table (OAM)
    // Gameboy video controller can display up to 40 sprites either in 8x8 or in 8x16 pixels. Because of a limitation of
    // hardware, only ten sprites can be displayed per scan line. Sprite patterns have the same format as BG tiles, but
    // they are taken from the Sprite Pattern Table located at $8000-8FFF and have unsigned numbering.
    // Sprite attributes reside in the Sprite Attribute Table (OAM - Object Attribute Memory) at $FE00-FE9F. Each of the 40
    // entries consists of four bytes with the following meanings:
    // Byte0 - Y Position
    // Specifies the sprites vertical position on the screen (minus 16). An off-screen value (for example, Y=0 or
    // Y>=160) hides the sprite.
    //
    // Byte1 - X Position
    // Specifies the sprites horizontal position on the screen (minus 8). An off-screen value (X=0 or X>=168) hides the
    // sprite, but the sprite still affects the priority ordering - a better way to hide a sprite is to set its
    // Y-coordinate off-screen.
    //
    // Byte2 - Tile/Pattern Number
    // Specifies the sprites Tile Number (00-FF). This (unsigned) value selects a tile from memory at 8000h-8FFFh. In
    // CGB Mode this could be either in VRAM Bank 0 or 1, depending on Bit 3 of the following byte. In 8x16 mode, the
    // lower bit of the tile number is ignored. IE: the upper 8x8 tile is "NN AND FEh", and the lower 8x8 tile
    // is "NN OR 01h".
    //
    // Byte3 - Attributes/Flags:
    // Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
    //        (Used for both BG and Window. BG color 0 is always behind OBJ)
    // Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
    // Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
    // Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
    // Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
    // Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
    oam: [u8; 0xa0],

    prio: [(bool, usize); SCREEN_W],
    // The LCD controller operates on a 222 Hz = 4.194 MHz dot clock. An entire frame is 154 scanlines, 70224 dots, or
    // 16.74 ms. On scanlines 0 through 143, the LCD controller cycles through modes 2, 3, and 0 once every 456 dots.
    // Scanlines 144 through 153 are mode 1.
    cycles: u32,

    intf: Rc<RefCell<IntReg>>,
}

impl GPU {
    pub fn new(intf: Rc<RefCell<IntReg>>) -> Self {
        Self {
            updated: false,
            data: [[[0xff; 3]; SCREEN_W]; SCREEN_H], // white
            lcdc: LCDControllerRegister::new(),
            stat: LCDStatusRegister::new(),
            scroll_y: 0x00,
            scroll_x: 0x00,
            window_x: 0x00,
            window_y: 0x00,
            ly: 0x00,
            lc: 0x00,
            bg_palette: 0x00,
            obj_palette0: 0x00,
            obj_palette1: 0x01,
            ram: [0x00; 0x4000],
            ram_bank: 0x00,
            oam: [0x00; 0xa0],
            prio: [(true, 0); SCREEN_W],
            cycles: 0,
            intf,
        }
    }

    pub fn reset_updated(&mut self) {
        self.updated = false;
    }

    pub fn should_updated(&self) -> bool {
        self.updated
    }

    /// Clear the screen content, Set all White.
    fn clear_screen(&mut self) {
        self.data = [[[0xff; 3]; SCREEN_W]; SCREEN_H];
    }

    pub fn get_data(&self) -> [[[u8; 3]; SCREEN_W]; SCREEN_H] {
        self.data
    }

    /// Read byte from the GPU ram.
    fn read_byte_from_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize - 0x8000]
    }

    /// Get the GB Color.
    fn get_color(&self, palette: Palette, i: usize) -> GBColor {
        let mut v = self.bg_palette;
        if palette == Palette::OBP0 {
            v = self.obj_palette0;
        }

        if palette == Palette::OBP1 {
            v = self.obj_palette1;
        }

        // This register assigns gray shades to the color numbers of the BG and Window tiles.
        // Bit 7-6 - Shade for Color Number 3
        // Bit 5-4 - Shade for Color Number 2
        // Bit 3-2 - Shade for Color Number 1
        // Bit 1-0 - Shade for Color Number 0
        // The four possible gray shades are:
        // 0  White
        // 1  Light gray
        // 2  Dark gray
        // 3  Black
        match v >> (2 * i) & 0b11 {
            0x00 => GBColor::White,
            0x01 => GBColor::Light,
            0x02 => GBColor::Dark,
            _ => GBColor::Black,
        }
    }

    /// Render the pixel in current scanline.
    fn render_pixel(&mut self, x: usize, c: GBColor) {
        let c = c as u8;
        self.data[self.ly as usize][x] = [c, c, c];
    }

    /// Call this method every enter new LCD mode!
    fn change_mode(&mut self, mode: LCDMode) {
        self.stat.set_mode(mode);

        match self.stat.get_mode() {
            LCDMode::HBlank => {
                if self.stat.is_m0_interrupt_enabled() {
                    self.intf.borrow_mut().req(Flag::LCDStat);
                }
                // Render scanline
                if self.lcdc.bg_display() {
                    self.render_bg();
                }
                if self.lcdc.is_sprite_enabled() {
                    self.render_sprite();
                }
            }
            LCDMode::VBlank => {
                self.updated = true;
                self.intf.borrow_mut().req(Flag::VBlank);
                if self.stat.is_m1_interrupt_enabled() {
                    self.intf.borrow_mut().req(Flag::LCDStat);
                }
            }
            LCDMode::OAM => {
                if self.stat.is_m2_interrupt_enabled() {
                    self.intf.borrow_mut().req(Flag::LCDStat);
                }
            }
            LCDMode::VRAM => {} // do nothing!
        }
    }

    // The LCD controller operates on a 222 Hz = 4.194 MHz dot clock. An entire frame is 154 scanlines, 70224 dots,
    // or 16.74 ms. On scanlines 0 through 143, the LCD controller cycles through modes 2, 3, and 0 once every 456
    // dots. Scanlines 144 through 153 are mode 1.
    //
    // 1 scanline = 456 dots
    //
    // The following are typical when the display is enabled:
    // Mode 2  2_____2_____2_____2_____2_____2___________________2____
    // Mode 3  _33____33____33____33____33____33__________________3___
    // Mode 0  ___000___000___000___000___000___000________________000
    // Mode 1  ____________________________________11111111111111_____
    pub fn next(&mut self, cycles: u32) {
        if !self.lcdc.is_lcd_enabled() {
            return;
        }

        let mut remaining_cycles = cycles;

        while remaining_cycles > 0 {
            let current_cycles = if remaining_cycles >= 80 {
                80
            } else {
                remaining_cycles
            };
            self.cycles += current_cycles;
            remaining_cycles -= current_cycles;

            // Full line takes 114 ticks
            if self.cycles >= 456 {
                self.cycles -= 456;
                self.ly = (self.ly + 1) % 154;
                if self.stat.is_ly_interrupt_enabled() && self.ly == self.lc {
                    self.intf.borrow_mut().req(Flag::LCDStat);
                }
                // This is a VBlank line
                if self.ly >= 144 && self.stat.get_mode() != LCDMode::VBlank {
                    self.change_mode(LCDMode::VBlank);
                }
            }

            // This is a normal line
            if self.ly < 144 {
                if self.cycles <= 80 {
                    if self.stat.get_mode() != LCDMode::OAM {
                        self.change_mode(LCDMode::OAM);
                    }
                } else if self.cycles <= (80 + 172) {
                    // 252 cycles
                    if self.stat.get_mode() != LCDMode::VRAM {
                        self.change_mode(LCDMode::VRAM);
                    }
                } else {
                    // the remaining 204
                    if self.stat.get_mode() != LCDMode::HBlank {
                        self.change_mode(LCDMode::HBlank);
                    }
                }
            }
        }
    }

    /// Returns true if we should render window instead of the bg.
    fn using_window(&self) -> bool {
        if self.lcdc.is_window_enabled() {
            // is the current scanline we're drawing within the windows Y pos?
            // - window_y in addr 0xFF4A
            // - ly in addr 0xFF44
            if self.window_y <= self.ly {
                return true;
            }
        }
        false
    }

    /// Get the windwo poistion, the result'x axios will minus 7.
    fn get_window_topleft_position(&self) -> (u8, u8) {
        (self.window_x.wrapping_sub(7), self.window_y)
    }

    /// Get the tile position.
    fn get_tile_position(&self, line_offset: u8) -> (u8, u8) {
        let (window_x, window_y) = self.get_window_topleft_position();

        // yPos is used to calculate which of 32 vertical tiles the
        // current scanline is drawing
        let pos_y = if self.using_window() {
            // self.ly + self.scroll_y - (self.scroll_y + self.window_y)
            // 位于 window 中的偏移
            self.ly.wrapping_sub(window_y)
        } else {
            // TODO
            // self.scroll_y.wrapping_add(self.ly)
            // 位于 bg 的偏移
            self.ly.wrapping_add(self.scroll_y)
        };

        let mut pos_x = self.scroll_x.wrapping_add(line_offset);
        if self.using_window() {
            if line_offset >= window_x {
                pos_x = line_offset - window_x;
            }
        }

        (pos_x, pos_y)
    }

    /// Find the tile data address.
    fn find_tile_data_addr(&self, base_addr: u16, row: u16, col: u16) -> u16 {
        let (tile_base_addr, unsig) = self.lcdc.get_tile_data_base_addr();
        // Tile data
        // Each tile is sized 8x8 pixels and has a color depth of 4 colors/gray shades.
        // Each tile occupies 16 bytes, where each 2 bytes represent a line:
        // Byte 0-1  First Line (Upper 8 pixels)
        // Byte 2-3  Next Line
        // etc.
        // 一行 32 个
        let tile_map_addr = base_addr + row * 32 + col;

        let tile_num = if unsig {
            i16::from(self.read_byte_from_ram(tile_map_addr))
        } else {
            i16::from(self.read_byte_from_ram(tile_map_addr) as i8)
        };

        let mut tile_data_addr = tile_base_addr;
        if unsig {
            tile_data_addr += (tile_num * 16) as u16;
        } else {
            tile_data_addr += ((tile_num + 128) * 16) as u16;
        }

        tile_data_addr
    }

    /// Render bg or the window.
    fn render_bg(&mut self) {
        let (window_x, _) = self.get_window_topleft_position();

        // 口袋妖怪红，尼多朗会先跳出来
        // let bg_base = if using_window {
        //     self.lcdc.window_tilemap_addr()
        // } else {
        //     self.lcdc.bg_tilemap_addr()
        // };

        for pixel in 0..SCREEN_W {
            let pixel = pixel as u8;
            let (pos_x, pox_y) = self.get_tile_position(pixel);

            // which of the 8 vertical pixels of the current
            // tile is the scanline on?
            // 计算第多少个 tile
            // 一个 tile 8 * 8 个像素
            let tile_row = u16::from(pox_y / 8);
            let tile_col = u16::from(pos_x / 8);

            // Background memory base addr.
            let bg_base_addr = if self.using_window() && pixel >= window_x {
                self.lcdc.get_window_tilemap_addr()
            } else {
                self.lcdc.get_bg_tilemap_addr()
            };

            // lookup up the tile_data num and return the actual address of tile data.
            let tile_data_addr = self.find_tile_data_addr(bg_base_addr, tile_row, tile_col);

            // find the correct vertical line we're on of the
            // tile to get the tile data
            // from in memory
            let line_in_tile = pox_y % 8;

            // each line takes up two bytes of memory
            let data_1 = self.read_byte_from_ram(tile_data_addr + u16::from(line_in_tile * 2));
            let data_2 = self.read_byte_from_ram(tile_data_addr + u16::from(line_in_tile * 2) + 1);
            // tile_y_data = [data_1, data_2];
            let tile_line = TileLine::new([data_1, data_2]);

            let color_bit = pos_x % 8;
            let color_num = tile_line.get_color_num(color_bit);

            self.prio[pixel as usize] = (false, color_num as usize);
            let color = self.get_color(Palette::BG, color_num as usize);
            self.render_pixel(pixel as usize, color);
        }
    }

    /// Gameboy video controller can display up to 40 sprites either in 8x8 or in 8x16 pixels. Because of a limitation
    /// of hardware, only ten sprites can be displayed per scan line. Sprite patterns have the same format as BG tiles,
    /// but they are taken from the Sprite Pattern Table located at $8000-8FFF and have unsigned numbering.
    ///
    /// Sprite attributes reside in the Sprite Attribute Table (OAM - Object Attribute Memory) at $FE00-FE9F. Each of
    /// the 40 entries consists of four bytes with the following meanings:
    ///   Byte0 - Y Position
    ///   Specifies the sprites vertical position on the screen (minus 16). An off-screen value (for example, Y=0 or
    ///   Y>=160) hides the sprite.
    ///
    ///   Byte1 - X Position
    ///   Specifies the sprites horizontal position on the screen (minus 8). An off-screen value (X=0 or X>=168) hides
    ///   the sprite, but the sprite still affects the priority ordering - a better way to hide a sprite is to set its
    ///   Y-coordinate off-screen.
    ///
    ///   Byte2 - Tile/Pattern Number
    ///   Specifies the sprites Tile Number (00-FF). This (unsigned) value selects a tile from memory at 8000h-8FFFh. In
    ///   CGB Mode this could be either in VRAM Bank 0 or 1, depending on Bit 3 of the following byte. In 8x16 mode, the
    ///   lower bit of the tile number is ignored. IE: the upper 8x8 tile is "NN AND FEh", and the lower 8x8 tile is
    ///   "NN OR 01h".
    ///
    ///   Byte3 - Attributes/Flags:
    ///     Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
    ///           (Used for both BG and Window. BG color 0 is always behind OBJ)
    ///     Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
    ///     Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
    ///     Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
    ///     Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
    ///     Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
    fn render_sprite(&mut self) {
        // Sprite tile size 8x8 or 8x16(2 stacked vertically).
        let (_, sprite_y_size) = self.lcdc.get_sprite_size();
        for i in 0..40 {
            //  sprite occupies 4 bytes in the sprite attributes table
            let index = (i as u16) * 4;
            let sprite_addr = 0xfe00 + index;

            // 0: Sprite Y Position: Position of the sprite on the Y axis of the viewing display minus 16
            // 1: Sprite X Position: Position of the sprite on the X axis of the viewing display minus 8
            let pos_y = self.read_byte(sprite_addr).wrapping_sub(16);
            let pox_x = self.read_byte(sprite_addr + 1).wrapping_sub(8);
            let tile_number = self.read_byte(sprite_addr + 2);
            let tile_attr = Attr::from(self.read_byte(sprite_addr + 3));

            // if !(self.ly > pos_y && self.ly < pos_y.wrapping_add(sprite_y_size)) {
            //     continue;
            // }

            // & if self.lcdc.get_sprite_size() == 16 {
            //     0xfe
            // } else {
            //     0xff
            // };

            if pos_y <= 0xff - sprite_y_size + 1 {
                if self.ly < pos_y || self.ly > pos_y + sprite_y_size - 1 {
                    continue;
                }
            } else {
                if self.ly > pos_y.wrapping_add(sprite_y_size) - 1 {
                    continue;
                }
            }

            if pox_x >= (SCREEN_W as u8) && pox_x <= (0xff - 7) {
                continue;
            }

            let line_in_tile = if tile_attr.has_yflip() {
                sprite_y_size - 1 - self.ly.wrapping_sub(pos_y)
            } else {
                self.ly.wrapping_sub(pos_y)
            };

            let tile_data_addr =
                0x8000u16 + u16::from(tile_number) * 16 + u16::from(line_in_tile) * 2;

            let data_1 = self.read_byte_from_ram(tile_data_addr);
            let data_2 = self.read_byte_from_ram(tile_data_addr + 1);

            let tile_line = TileLine::new([data_1, data_2]);

            // its easier to read in from right to left as pixel 0 is
            // bit 7 in the colour data, pixel 1 is bit 6 etc...
            for x in 0..8 {
                if pox_x.wrapping_add(x) >= (SCREEN_W as u8) {
                    continue;
                }
                let tile_x = if tile_attr.has_xflip() { 7 - x } else { x };
                let color_num = tile_line.get_color_num(tile_x);
                if color_num == 0 {
                    continue;
                }

                // Confirm the priority of background and sprite.
                let prio = self.prio[pox_x.wrapping_add(x) as usize];
                let skip = if prio.0 {
                    prio.1 != 0
                } else {
                    tile_attr.get_priority() && prio.1 != 0
                };
                if skip {
                    continue;
                }

                let palette = tile_attr.get_palette();

                let color = self.get_color(palette, color_num as usize);
                self.render_pixel(pox_x.wrapping_add(x) as usize, color);
            }
        }
    }
}

impl IOHandler for GPU {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9fff => self.ram[self.ram_bank * 0x2000 + addr as usize - 0x8000],
            0xfe00..=0xfe9f => self.oam[addr as usize - 0xfe00],
            0xff40 => self.lcdc.get_value(),
            0xff41 => self.stat.get_value(),
            0xff42 => self.scroll_y,
            0xff43 => self.scroll_x,
            0xff44 => self.ly,
            0xff45 => self.lc,
            0xff47 => self.bg_palette,
            0xff48 => self.obj_palette0,
            0xff49 => self.obj_palette1,
            0xff4a => self.window_y,
            0xff4b => self.window_x,
            _ => unreachable!(
                "GPU should not handle the {:0x} address read operation",
                addr
            ),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000..=0x9fff => self.ram[self.ram_bank * 0x2000 + addr as usize - 0x8000] = val,
            0xfe00..=0xfe9f => self.oam[addr as usize - 0xfe00] = val,
            0xff40 => {
                self.lcdc.set_value(val);
                if !self.lcdc.is_lcd_enabled() {
                    self.cycles = 0;
                    self.ly = 0;
                    self.stat.set_mode(LCDMode::HBlank);
                    self.clear_screen();
                    self.updated = true;
                }
            }
            0xff41 => {
                if val & 0x40 != 0x00 {
                    self.stat.enable_ly_interrupt();
                } else {
                    self.stat.disable_ly_interrupt();
                }

                if 0x20 != 0x00 {
                    self.stat.enable_m2_interrupt();
                } else {
                    self.stat.disable_m2_interrupt();
                }
                if val & 0x10 != 0x00 {
                    self.stat.enable_m1_interrupt();
                } else {
                    self.stat.disable_m1_interrupt();
                }
                if val & 0x08 != 0x00 {
                    self.stat.enable_m0_interrupt();
                } else {
                    self.stat.disable_m0_interrupt();
                }
            }
            0xff42 => self.scroll_y = val,
            0xff43 => self.scroll_x = val,
            0xff44 => {}
            0xff45 => self.lc = val,
            0xff47 => self.bg_palette = val,
            0xff48 => self.obj_palette0 = val,
            0xff49 => self.obj_palette1 = val,
            0xff4a => self.window_y = val,
            0xff4b => self.window_x = val,
            _ => panic!(
                "GPU should not handle the {:0x} address write operation, value is {:0x}",
                addr, val
            ),
        }
    }
}
