use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use log::info;

use super::Cartridge;
use super::CartridgeMeta;
use super::IOHandler;
use super::MemoryBank;

#[derive(Debug)]
pub struct MBC2 {
    pub meta: CartridgeMeta,
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_enabled: bool,
    sav_path: PathBuf,
}

impl MBC2 {
    /// Returns a new MBC2 chip.
    pub fn new(meta: CartridgeMeta, rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self {
        Self {
            meta,
            rom,
            ram,
            rom_bank: 1,
            ram_enabled: false,
            sav_path: PathBuf::from(sav.as_ref()),
        }
    }
}

impl MemoryBank for MBC2 {
    fn get_rom_bank_num(&self) -> usize {
        self.rom_bank
    }

    fn get_ram_bank_num(&self) -> usize {
        unimplemented!("MBC2 has not ram_bank")
    }

    fn read_via_rom_bank(&self, addr: u16) -> u8 {
        let bank_addr = 0x4000 * self.get_rom_bank_num() + (addr as usize - 0x4000);
        if (bank_addr as usize) < self.rom.len() {
            self.rom[bank_addr as usize]
        } else {
            0x00
        }
    }

    fn read_via_ram_bank(&self, addr: u16) -> u8 {
        // It's has no ram bank
        if !self.ram_enabled {
            self.ram[(addr - 0xa000) as usize]
        } else {
            0x00
        }
    }

    fn write_via_ram_bank(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        let bank_addr = addr as usize - 0xa000;
        if (bank_addr as usize) < self.ram.len() {
            self.ram[bank_addr as usize] = value
        }
    }
}

impl IOHandler for MBC2 {
    /// ### 0000-3FFF - ROM Bank 00 (Read Only)
    /// Same as for MBC1.
    ///
    /// ### 4000-7FFF - ROM Bank 01-0F (Read Only)
    /// Same as for MBC1, but only a total of 16 ROM banks is supported.
    ///
    /// ### A000-A1FF - 512x4bits RAM, built-in into the MBC2 chip (Read/Write)
    /// The MBC2 doesn't support external RAM,
    /// instead it includes 512x4 bits of built-in RAM (in the MBC2 chip itself).
    /// It still requires an external battery to save data during power-off though.
    /// As the data consists of 4bit values, only the lower 4 bits of the "bytes" in this memory area are used.
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => self.read_via_rom_bank(addr),
            0xa000..=0xa1ff => self.read_via_ram_bank(addr),
            _ => 0x00,
        }
    }

    /// ### 0000-1FFF - RAM Enable (Write Only)
    /// The **least significant bit of the upper address byte must be zero** to enable/disable cart RAM.
    /// For example the following addresses can be used to enable/disable cart RAM: 0000-00FF, 0200-02FF, 0400-04FF, ..., 1E00-1EFF.
    /// The suggested address range to use for MBC2 ram enable/disable is 0000-00FF.
    ///
    /// ### 2000-3FFF - ROM Bank Number (Write Only)
    /// Writing a value (XXXXBBBB - X = Don't cares, B = bank select bits) into 2000-3FFF area will select an appropriate ROM bank at 4000-7FFF.
    /// The least significant bit of the upper address byte must be one to select a ROM bank.
    /// For example the following addresses can be used to select a ROM bank: 2100-21FF, 2300-23FF, 2500-25FF, ..., 3F00-3FFF.
    /// The suggested address range to use for MBC2 rom bank selection is 2100-21FF.
    ///
    /// ### A000-A1FF - 512x4bits RAM, built-in into the MBC2 chip (Read/Write)
    /// The MBC2 doesn't support external RAM,
    /// instead it includes 512x4 bits of built-in RAM (in the MBC2 chip itself).
    /// It still requires an external battery to save data during power-off though.
    /// As the data consists of 4bit values, only the lower 4 bits of the "bytes" in this memory area are used.
    fn write_byte(&mut self, addr: u16, value: u8) {
        // Only the lower 4 bits of the "bytes" in this memory area are used.
        match addr {
            0x0000..=0x1fff => {
                // 高位地址字节的最低有效位为 0 才能启用/禁用
                if addr & 0b0000_0001_0000_0000 == 0 {
                    if value & 0x0f == 0x0a {
                        self.ram_enabled = true;
                    } else {
                        self.ram_enabled = false;
                    }
                }
            }
            0x2000..=0x3fff => {
                // 高位地址字节的最低有效位为 1 才能设置 rom_bank
                if addr & 0b0000_0001_0000_0000 == 1 {
                    // (XXXXBBBB - X = Don't cares, B = bank select bits)
                    self.rom_bank = (value & 0x0f) as usize;
                }
            }
            // 4 bit 区域
            0xa000..=0xa1ff => self.write_via_ram_bank(addr, value & 0b1111),
            _ => {}
        }
    }
}

impl Cartridge for MBC2 {
    fn get_meta(&self) -> CartridgeMeta {
        self.meta.clone()
    }
}

impl Drop for MBC2 {
    fn drop(&mut self) {
        if self.sav_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.sav_path.clone())
            .and_then(|mut f| f.write_all(&self.ram))
            .unwrap();
        info!("save success when drop the cartridge object.")
    }
}
