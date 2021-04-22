/// Each tile is stored in memory as 16 bytes.
/// A tile is 8x8 pixels and that in memory each line of the tile requires two bytes to represent,
/// hence the 16 bytes per tile.
/// Each tile is sized 8x8 pixels and has a color depth of 4 colors/gray shades.
pub struct TileLine {
    data: [u8; 2],
}

impl TileLine {
    pub fn new(data: [u8; 2]) -> Self {
        Self { data }
    }

    pub fn get_color_num(&self, bit: u8) -> u8 {
        // pixel 0 in the tile is it 7 of data 1 and data2.
        // Pixel 1 is bit 6 etc..
        let mask = 0b1000_0000 >> bit;
        let color_h = if self.data[1] & mask == mask { 1 } else { 0 };
        let color_l = if self.data[0] & mask == mask { 1 } else { 0 };
        let color = (color_h << 1) | color_l;

        color
    }

    pub fn get_data(&self) -> [u8; 2] {
        self.data
    }
}

// pub struct Tile {
//     data: [TileLine; 8],
// }

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GBColor {
    White = 0xff,
    Light = 0xc0,
    Dark = 0x60,
    Black = 0x00,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Palette {
    OBP0 = 0,
    OBP1 = 1,

    // NOTICE 0xff is special
    BG = 0xff,
}

/// Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
///     (Used for both BG and Window. BG color 0 is always behind OBJ)
/// Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
/// Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
/// Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
/// Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
/// Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
pub struct Attr {
    priority: bool,
    yflip: bool,
    xflip: bool,
    palette: Palette,
    // TODO
}

impl Attr {
    pub fn get_priority(&self) -> bool {
        self.priority
    }
    pub fn has_yflip(&self) -> bool {
        self.yflip
    }

    pub fn has_xflip(&self) -> bool {
        self.xflip
    }

    pub fn get_palette(&self) -> Palette {
        self.palette.clone()
    }
}

impl From<u8> for Attr {
    fn from(u: u8) -> Self {
        Self {
            priority: u & (1 << 7) != 0,
            yflip: u & (1 << 6) != 0,
            xflip: u & (1 << 5) != 0,
            palette: if u & (1 << 4) == 1 {
                Palette::OBP1
            } else {
                Palette::OBP0
            },
        }
    }
}
