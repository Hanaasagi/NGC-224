#![allow(dead_code)]

use std::array::IntoIter;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::string::ToString;

// See
// - https://gbdev.gg8.se/wiki/articles/The_Cartridge_Header
// - http://gameboy.mongenel.com/dmg/asmmemmap.html

// The Header Struct
// 0x0100-0x0103 	NOP / JP 0x0150
// 0x0104-0x0133 	Nintendo Logo
// 0x0134-0x013E 	Game Title (Uppercase ASCII)
// 0x013F-0x0142 	4-byte Game Designation
// 0x0143       	Color Compatibility byte
// 0x0144-0x0145 	New Licensee Code
// 0x0146       	SGB Compatibility byte
// 0x0147 	        Cart Type
// 0x0148       	Cart ROM size
// 0x0149       	Cart RAM size
// 0x014A       	Destination code
// 0x014B       	Old Licensee code
// 0x014C       	Mask ROM version
// 0x014D       	Complement checksum
// 0x014E-0x014F 	Checksum

// All license code here
// 00  none                01  Nintendo R&D1   08  Capcom
// 13  Electronic Arts     18  Hudson Soft     19  b-ai
// 20  kss                 22  pow             24  PCM Complete
// 25  san-x               28  Kemco Japan     29  seta
// 30  Viacom              31  Nintendo        32  Bandai
// 33  Ocean/Acclaim       34  Konami          35  Hector
// 37  Taito               38  Hudson          39  Banpresto
// 41  Ubi Soft            42  Atlus           44  Malibu
// 46  angel               47  Bullet-Proof    49  irem
// 50  Absolute            51  Acclaim         52  Activision
// 53  American sammy      54  Konami          55  Hi tech entertainment
// 56  LJN                 57  Matchbox        58  Mattel
// 59  Milton Bradley      60  Titus           61  Virgin
// 64  LucasArts           67  Ocean           69  Electronic Arts
// 70  Infogrames          71  Interplay       72  Broderbund
// 73  sculptured          75  sci             78  THQ
// 79  Accolade            80  misawa          83  lozc
// 86  tokuma shoten i*    87  tsukuda ori*    91  Chunsoft
// 92  Video system        93  Ocean/Acclaim   95  Varie
// 96  Yonezawa/s'pal      97  Kaneko          99  Pack in soft
// A4  Konami (Yu-Gi-Oh!)
lazy_static! {
    static ref LICENSEE_CODE: HashMap<&'static str, &'static str> = {
        HashMap::<_, _>::from_iter(IntoIter::new([
            ("00", "none"),
            ("01", "Nintendo R&D1"),
            ("08", "Capcom"),
            ("13", "Electronic Arts"),
            ("18", "Hudson Soft"),
            ("19", "b-ai"),
            ("20", "kss"),
            ("22", "pow"),
            ("24", "PCM Complete"),
            ("25", "san-x"),
            ("28", "Kemco Japan"),
            ("29", "seta"),
            ("30", "Viacom"),
            ("31", "Nintendo"),
            ("32", "Bandai"),
            ("33", "Ocean/Acclaim"),
            ("34", "Konami"),
            ("35", "Hector"),
            ("37", "Taito"),
            ("38", "Hudson"),
            ("39", "Banpresto"),
            ("41", "Ubi Soft"),
            ("42", "Atlus"),
            ("44", "Malibu"),
            ("46", "angel"),
            ("47", "Bullet-Proof"),
            ("49", "irem"),
            ("50", "Absolute"),
            ("51", "Acclaim"),
            ("52", "Activision sammy"),
            ("53", "American"),
            ("54", "Konami"),
            ("55", "Hi tech entertainment"),
            ("56", "LJN"),
            ("57", "Matchbox"),
            ("58", "Mattel"),
            ("59", "Milton Bradley"),
            ("60", "Titus"),
            ("61", "Virgin"),
            ("64", "LucasArts"),
            ("67", "Ocean"),
            ("69", "Electronic Arts"),
            ("70", "Infogrames"),
            ("71", "Interplay"),
            ("72", "Broderbund"),
            ("73", "sculptured"),
            ("75", "sci"),
            ("78", "THQ"),
            ("79", "Accolade"),
            ("80", "misawa"),
            ("83", "lozc"),
            ("86", "tokuma shoten i*"),
            ("87", "tsukuda ori*"),
            ("91", "Chunsoft"),
            ("92", "Video system"),
            ("93", "Ocean/Acclaim"),
            ("95", "Varie"),
            ("96", "Yonezawa/s'pal"),
            ("97", "Kaneko"),
            ("99", "Pack in soft"),
            ("A4", "Konami(Yu-Gi-Oh!)"),
        ]))
    };
}

/// Catrtridge Type, see this link https://gbdev.gg8.se/wiki/articles/The_Cartridge_Header.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum CartridgeType {
    ROM_ONLY,
    ROM_MBC1,
    ROM_MBC1_RAM,
    ROM_MBC1_RAM_BATT,
    ROM_MBC2,
    ROM_MBC2_BATT,
    ROM_MMM01,
    ROM_MMM01_RAM,
    ROM_MMM01_RAM_BATT,
    ROM_MBC3_TIMER_BATT,
    ROM_MBC3_TIMER_RAM_BATT,
    ROM_MBC3,
    ROM_MBC3_RAM,
    ROM_MBC3_RAM_BATT,
    ROM_MBC5,
    ROM_MBC5_RAM,
    ROM_MBC5_RAM_BATT,
    ROM_MBC5_RUMBLE,
    ROM_MBC5_RUMBLE_RAM,
    ROM_MBC5_RUMBLE_RAM_BATT,
    ROM_MBC7_BATT,
    GAME_GENIE,
    GAME_SHARK3,
    ROM_POCKET_CAMERA,
    ROM_BANDAI_TAMA5,
    ROM_HUC3,
    ROM_HUC1,
}

// #[derive(Debug, Copy, Clone)]
// pub enum CartridgeFeature {
//     WithRAM,
//     WithBattery,
//     WithTimer,
// }

/// Catrtridge Region, see this link https://gbdev.gg8.se/wiki/articles/The_Cartridge_Header.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum CartridgeRegion {
    JP,
    NON_JP,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum CartridgePlatform {
    // Game Boy Color, works on old gameboys also
    GBC,
    // Game Boy Color
    GBC_ONLY,
    // Super Game Boy
    SGB,
    // Game Boy
    GB,
}

/// Catrtridge Platform, see this link https://gbdev.gg8.se/wiki/articles/The_Cartridge_Header.
#[derive(Debug, Clone)]
pub struct CartridgeMeta {
    title: String,
    rom_size: usize,
    ram_size: usize,
    r#type: CartridgeType,
    region: CartridgeRegion,
    support_sgb: bool,
    licensee: String,
    platform: CartridgePlatform,
}

impl CartridgeMeta {
    /// Parse the type field from the cartridge header.
    /// ```ignore
    /// 00h  ROM ONLY                 19h  MBC5
    /// 01h  MBC1                     1Ah  MBC5+RAM
    /// 02h  MBC1+RAM                 1Bh  MBC5+RAM+BATTERY
    /// 03h  MBC1+RAM+BATTERY         1Ch  MBC5+RUMBLE
    /// 05h  MBC2                     1Dh  MBC5+RUMBLE+RAM
    /// 06h  MBC2+BATTERY             1Eh  MBC5+RUMBLE+RAM+BATTERY
    /// 08h  ROM+RAM                  20h  MBC6
    /// 09h  ROM+RAM+BATTERY          22h  MBC7+SENSOR+RUMBLE+RAM+BATTERY
    /// 0Bh  MMM01
    /// 0Ch  MMM01+RAM
    /// 0Dh  MMM01+RAM+BATTERY
    /// 0Fh  MBC3+TIMER+BATTERY
    /// 10h  MBC3+TIMER+RAM+BATTERY   FCh  POCKET CAMERA
    /// 11h  MBC3                     FDh  BANDAI TAMA5
    /// 12h  MBC3+RAM                 FEh  HuC3
    /// 13h  MBC3+RAM+BATTERY         FFh  HuC1+RAM+BATTERY
    /// ```
    fn parse_type(data: &Vec<u8>) -> CartridgeType {
        let t = data[0x0147];
        match t {
            0x00 => CartridgeType::ROM_ONLY,
            0x01 => CartridgeType::ROM_MBC1,
            0x02 => CartridgeType::ROM_MBC1_RAM,
            0x03 => CartridgeType::ROM_MBC1_RAM_BATT,
            0x05 => CartridgeType::ROM_MBC2,
            0x06 => CartridgeType::ROM_MBC2_BATT,
            0x0b => CartridgeType::ROM_MMM01,
            0x0c => CartridgeType::ROM_MMM01_RAM,
            0x0d => CartridgeType::ROM_MMM01_RAM_BATT,
            0x0f => CartridgeType::ROM_MBC3_TIMER_BATT,
            0x10 => CartridgeType::ROM_MBC3_TIMER_RAM_BATT,
            0x11 => CartridgeType::ROM_MBC3,
            0x12 => CartridgeType::ROM_MBC3_RAM,
            0x13 => CartridgeType::ROM_MBC3_RAM_BATT,
            0x19 => CartridgeType::ROM_MBC5,
            0x1a => CartridgeType::ROM_MBC5_RAM,
            0x1b => CartridgeType::ROM_MBC5_RAM_BATT,
            0x1c => CartridgeType::ROM_MBC5_RUMBLE,
            0x1d => CartridgeType::ROM_MBC5_RUMBLE_RAM,
            0x1e => CartridgeType::ROM_MBC5_RUMBLE_RAM_BATT,
            0x22 => CartridgeType::ROM_MBC7_BATT,
            0x55 => CartridgeType::GAME_GENIE,
            0x56 => CartridgeType::GAME_SHARK3,
            0xfc => CartridgeType::ROM_POCKET_CAMERA,
            0xfd => CartridgeType::ROM_BANDAI_TAMA5,
            0xfe => CartridgeType::ROM_HUC3,
            0xff => CartridgeType::ROM_HUC1,
            _ => panic!("invalie cartridge type {}", t),
        }
    }

    /// Parse the rom size field from the cartridge header.
    /// ```ignore
    /// 0x00 -  32KByte (no ROM banking)
    /// 0x01 -  64KByte (4 banks)
    /// 0x02 - 128KByte (8 banks)
    /// 0x03 - 256KByte (16 banks)
    /// 0x04 - 512KByte (32 banks)
    /// 0x05 -   1MByte (64 banks)  - only 63 banks used by MBC1
    /// 0x06 -   2MByte (128 banks) - only 125 banks used by MBC1
    /// 0x07 -   4MByte (256 banks)
    /// 0x08 -   8MByte (512 banks)
    /// 0x52 - 1.1MByte (72 banks)
    /// 0x53 - 1.2MByte (80 banks)
    /// 0x54 - 1.5MByte (96 banks)
    /// ```
    fn parse_rom_size(data: &Vec<u8>) -> usize {
        let bank = 16384;
        match data[0x0148] {
            0x00 => bank * 2,
            0x01 => bank * 4,
            0x02 => bank * 8,
            0x03 => bank * 16,
            0x04 => bank * 32,
            0x05 => bank * 64,
            0x06 => bank * 128,
            0x07 => bank * 256,
            0x08 => bank * 512,
            0x52 => bank * 72,
            0x53 => bank * 80,
            0x54 => bank * 96,
            n => panic!("Unsupported rom size: 0x{:02x}", n),
        }
    }

    /// Parse the ram size field from the cartridge header.
    /// ```ignore
    /// 00h - None
    /// 01h - 2 KBytes
    /// 02h - 8 Kbytes
    /// 03h - 32 KBytes (4 banks of 8KBytes each)
    /// 04h - 128 KBytes (16 banks of 8KBytes each)
    /// 05h - 64 KBytes (8 banks of 8KBytes each)
    /// ```
    fn parse_ram_size(data: &Vec<u8>) -> usize {
        match data[0x0149] {
            0x00 => 0,
            0x01 => 1024 * 2,
            0x02 => 1024 * 8,
            0x03 => 1024 * 32,
            0x04 => 1024 * 128,
            0x05 => 1024 * 64,
            n => panic!("Unsupported ram size: 0x{:02x}", n),
        }
    }

    /// Parse the title field from the cartridge header.
    /// ### 0134-0143 - Title
    /// Title of the game in UPPER CASE ASCII.
    /// If it is less than 16 characters then the remaining bytes are filled with 00's.
    /// When inventing the CGB, Nintendo has reduced the length of this area to 15 characters,
    /// and some months later they had the fantastic idea to reduce it to 11 characters only.
    /// The new meaning of the ex-title bytes is described below.
    fn parse_title(data: &Vec<u8>) -> String {
        let mut name = String::new();
        let lower = 0x0134;
        let upper = 0x0143;
        // 这个 0x0143 在旧类型中是 title 的一部分，是右闭区间
        // 新类型中则是 CGB Flag 了
        // 这里直接走新式卡带，不用 0x0143
        // 读到 0 为止，至多读到 0x0142
        for &c in data[lower..upper].iter() {
            if c == 0x00 {
                break;
            }
            name.push(c as char);
        }

        name
    }

    /// Parse the CGB and SGB field from the cartridge header.
    /// ### 0143 - CGB Flag
    /// In older cartridges this byte has been part of the Title (see above).
    /// In CGB cartridges the upper bit is used to enable CGB functions.
    /// This is required, otherwise the CGB switches itself into Non-CGB-Mode. Typical values are:
    /// ```ignore
    /// 80h - Game supports CGB functions, but works on old gameboys also.
    /// C0h - Game works on CGB only (physically the same as 80h).
    /// ```
    ///
    /// ### 0146 - SGB Flag
    /// Specifies whether the game supports SGB functions, common values are:
    /// ```ignore
    /// 00h = No SGB functions (Normal Gameboy or CGB only game)
    /// 03h = Game supports SGB functions
    /// ```
    /// The SGB disables its SGB functions if this byte is set to another value than 03h.
    fn parse_platform(data: &Vec<u8>) -> CartridgePlatform {
        if data[0x0146] == 0x03 {
            return CartridgePlatform::SGB;
        } else if data[0x0143] == 0x80 {
            return CartridgePlatform::GBC;
        } else if data[0x0143] == 0xC0 {
            return CartridgePlatform::GBC_ONLY;
        } else {
            CartridgePlatform::GB
        }
    }

    /// Parse the destination code field from the cartridge header.
    /// Specifies if this version of the game is supposed to be sold in Japan, or anywhere else. Only two values are defined.
    /// 00h - Japanese
    /// 01h - Non-Japanese
    fn parse_region(data: &Vec<u8>) -> CartridgeRegion {
        if data[0x014A] == 0 {
            CartridgeRegion::JP
        } else {
            CartridgeRegion::NON_JP
        }
    }

    /// Parse the SGB Flag.
    fn parse_sgb_flag(data: &Vec<u8>) -> bool {
        data[0x0146] != 0
    }

    /// Parse the licensee field from the cartridge header.
    ///
    /// Specifies a two character ASCII licensee code, indicating the company or publisher of the game.
    /// These two bytes are used in newer games only (games that have been released after the SGB has been invented).
    /// Older games are using the header entry at 014B instead.
    fn parse_licensee(data: &Vec<u8>) -> String {
        // See the Python Code: https://github.com/garbear/pyrominfo/blob/9c3b94482c2eed335858535633c22bfa71e14c45/pyrominfo/gameboy.py
        // # 0144-0145 - New Licenseee Code, two character ASCII licenseee code
        // # 014B - Old Licenseee Code in range 00-FF, value of 33h signals New Licensee Code is used instead
        // ```
        // if data[0x14b] == 0x33:
        //     pub = data[0x144 : 0x144 + 2].decode("ascii", "ignore")
        // else:
        //     pub = "%02X" % data[0x14b]
        // ```

        if data[0x014b] == 0x33 {
            let mut code = String::new();
            for c in data[0x0144..=0x0145].iter() {
                code.push(*c as char);
            }
            LICENSEE_CODE
                .get(&code as &str)
                .map(|s| s.to_string())
                .unwrap_or(format!("unknown licensee code {}", code))
        } else {
            format!("{:02x}", data[0x014b])
        }
    }
}

impl CartridgeMeta {
    /// Retruns the rom size in byte.
    pub fn get_rom_size(&self) -> usize {
        self.rom_size
    }

    /// Returns the ram size in byte.
    pub fn get_ram_size(&self) -> usize {
        self.ram_size
    }

    /// Returns the type of cartridge.
    pub fn get_type(&self) -> CartridgeType {
        self.r#type
    }

    /// Returns the title.
    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    /// Returns the region.
    pub fn get_region(&self) -> CartridgeRegion {
        self.region
    }

    /// Returns the licensee.
    pub fn get_licensee(&self) -> String {
        self.licensee.clone()
    }

    /// Returns whether support SGB.
    pub fn support_sgb(&self) -> bool {
        self.support_sgb
    }

    /// Returns the platform.
    pub fn get_platform(&self) -> CartridgePlatform {
        self.platform
    }
}

impl CartridgeMeta {
    /// Parse the cartridge header and return the meta info struct.
    pub fn new(data: &Vec<u8>) -> Self {
        let title = Self::parse_title(data);
        let rom_size = Self::parse_rom_size(data);
        let ram_size = Self::parse_ram_size(data);
        let r#type = Self::parse_type(data);
        let region = Self::parse_region(data);
        let support_sgb = Self::parse_sgb_flag(data);
        let licensee = Self::parse_licensee(data);
        let platform = Self::parse_platform(data);

        Self {
            title,
            rom_size,
            ram_size,
            r#type,
            region,
            support_sgb,
            licensee,
            platform,
        }
    }
}
