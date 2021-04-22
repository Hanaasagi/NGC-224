use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use log::info;

use super::BankMode;
use super::Cartridge;
use super::CartridgeMeta;
use super::IOHandler;
use super::MemoryBank;

#[derive(Debug)]
pub struct MBC1 {
    meta: CartridgeMeta,
    rom: Vec<u8>,
    ram: Vec<u8>,
    bank_mode: BankMode, // MBC1 has two different maximum memory modes: 16Mbit ROM/8KByte RAM or 4Mbit ROM/32KByte RAM.

    // Bank Mode   RAM Bank Bits   ROM Bank Bits
    // 1 bit       2 bit           5 bit

    // 当 Bank Mode 置为 0 时, 当前卡带为 ROM Bank Number 模式, 此时

    //    ROM Bank Number = RAM Bank Bits + ROM Bank Bits
    //    RAM Bank Number = 0x00

    // Bank Mode 置为 1 时, 当前卡带为 RAM Banking Mode 模式, 此时

    //    ROM Bank Number = ROM Bank Bits
    //    RAM Bank Number = RAM Bank Bits
    bank_reg: u8,
    ram_enabled: bool,
    sav_path: PathBuf,
}

impl MBC1 {
    /// Returns a new MBC1 chip.
    pub fn new(meta: CartridgeMeta, rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self {
        Self {
            meta,
            rom,
            ram,
            bank_mode: BankMode::Rom,
            bank_reg: 0x01,
            ram_enabled: false,
            sav_path: PathBuf::from(sav.as_ref()),
        }
    }
}

impl MemoryBank for MBC1 {
    fn get_rom_bank_num(&self) -> usize {
        let n = match self.bank_mode {
            BankMode::Rom => self.bank_reg & 0b0111_1111,
            BankMode::Ram => self.bank_reg & 0b0001_1111,
        };
        n as usize
    }

    fn get_ram_bank_num(&self) -> usize {
        let n = match self.bank_mode {
            BankMode::Rom => 0x00,
            BankMode::Ram => (self.bank_reg & 0b0110_0000) >> 5,
        };
        n as usize
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
        if !self.ram_enabled {
            return 0x00;
        }

        let bank_addr = 0x2000 * self.get_ram_bank_num() + (addr as usize - 0xa000);
        if (bank_addr as usize) < self.ram.len() {
            self.ram[bank_addr as usize]
        } else {
            0x00
        }
    }

    fn write_via_ram_bank(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        let bank_addr = 0x2000 * self.get_ram_bank_num() + (addr as usize - 0xa000);
        if (bank_addr as usize) < self.ram.len() {
            self.ram[bank_addr as usize] = value
        }
    }
}

impl IOHandler for MBC1 {
    /// ### 0000-3FFF - ROM Bank 00 (Read Only)
    /// This area always contains the first 16KBytes of the cartridge ROM.
    ///
    /// ### 4000-7FFF - ROM Bank 01-7F (Read Only)
    /// This area may contain any of the further 16KByte banks of the ROM,
    /// allowing to address up to 125 ROM Banks (almost 2MByte).
    /// As described below, bank numbers 20h, 40h, and 60h cannot be used, resulting in the odd amount of 125 banks.
    ///
    /// #### A000-BFFF - RAM Bank 00-03, if any (Read/Write)
    /// This area is used to address external RAM in the cartridge (if any). External RAM is often battery buffered,
    /// allowing to store game positions or high score tables, even if the gameboy is turned off,
    /// or if the cartridge is removed from the gameboy.
    /// Available RAM sizes are: 2KByte (at A000-A7FF), 8KByte (at A000-BFFF), and 32KByte (in form of four 8K banks at A000-BFFF).
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => self.read_via_rom_bank(addr),
            0xa000..=0xbfff => self.read_via_ram_bank(addr),
            _ => 0x00,
        }
    }

    /// ### 0000-1FFF - RAM Enable (Write Only)
    /// Before external RAM can be read or written,
    /// it must be enabled by writing to this address space.
    /// It is recommended to disable external RAM after accessing it,
    /// in order to protect its contents from damage during power down of the gameboy. Usually the following values are used:
    ///    00h  Disable RAM (default)
    ///    0Ah  Enable RAM

    /// ### 2000-3FFF - ROM Bank Number (Write Only)
    /// Writing to this address space selects the lower 5 bits of the ROM Bank Number (in range 01-1Fh).
    /// When 00h is written, the MBC translates that to bank 01h also.
    /// That doesn't harm so far, because ROM Bank 00h can be always directly accessed by reading from 0000-3FFF.
    /// But (when using the register below to specify the upper ROM Bank bits), the same happens for Bank 20h, 40h, and 60h.
    /// Any attempt to address these ROM Banks will select Bank 21h, 41h, and 61h instead.

    /// ### 4000-5FFF - RAM Bank Number - or - Upper Bits of ROM Bank Number (Write Only)
    /// This 2bit register can be used to select a RAM Bank in range from 00-03h,
    /// or to specify the upper two bits (Bit 5-6) of the ROM Bank number, depending on the current ROM/RAM Mode. (See below.)

    /// ### 6000-7FFF - ROM/RAM Mode Select (Write Only)
    /// This 1bit Register selects whether the two bits of the above register should be used as upper two bits of the ROM Bank, or as RAM Bank Number.
    ///     00h = ROM Banking Mode (up to 8KByte RAM, 2MByte ROM) (default)
    ///     01h = RAM Banking Mode (up to 32KByte RAM, 512KByte ROM)
    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1fff => {
                if value & 0x0f == 0x0a {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
                }
            }
            0x2000..=0x3fff => {
                // select lower 5 bits.
                let mut n = value & 0b0001_1111;
                // rewrite the 0x00 to 0x01
                if n == 0x00 {
                    n = 0x01;
                }
                // clean the lower 5 bits and assgin new value.
                self.bank_reg = (self.bank_reg & 0b0110_0000) | n;
            }
            0x4000..=0x5fff => {
                let n = value & 0b0011;
                self.bank_reg = self.bank_reg & 0b1001_1111 | (n << 5);
            }
            0x6000..=0x7fff => match value {
                0x00 => self.bank_mode = BankMode::Rom,
                0x01 => self.bank_mode = BankMode::Ram,
                _ => panic!("Invalid value: {}", value),
            },
            0xa000..=0xbfff => {
                self.write_via_ram_bank(addr, value);
            }
            _ => {}
        }
    }
}

impl Cartridge for MBC1 {
    fn get_meta(&self) -> CartridgeMeta {
        self.meta.clone()
    }
}

impl Drop for MBC1 {
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
