use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use log::error;
use log::info;

use super::Cartridge;
use super::CartridgeMeta;
use super::IOHandler;
use super::MemoryBank;
use super::RealTimeClock;

#[derive(Debug)]
pub struct MBC3 {
    meta: CartridgeMeta,
    rom: Vec<u8>,
    ram: Vec<u8>,
    rtc: RealTimeClock,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    sav_path: PathBuf,
}

impl MBC3 {
    /// Retrusn a new MBC3 chip.
    pub fn new(
        meta: CartridgeMeta,
        rom: Vec<u8>,
        ram: Vec<u8>,
        save_path: impl AsRef<Path>,
        rtc_save_path: impl AsRef<Path>,
    ) -> Self {
        MBC3 {
            meta,
            rom,
            ram,
            rtc: RealTimeClock::new(rtc_save_path),
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            sav_path: PathBuf::from(save_path.as_ref()),
        }
    }
}

impl MemoryBank for MBC3 {
    fn get_rom_bank_num(&self) -> usize {
        self.rom_bank
    }

    fn get_ram_bank_num(&self) -> usize {
        self.ram_bank
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
        if self.ram_enabled {
            if self.get_ram_bank_num() <= 0x03 {
                let i = self.get_ram_bank_num() * 0x2000 + addr as usize - 0xa000;
                self.ram[i]
            } else {
                self.rtc.get(self.ram_bank as u16)
            }
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

// https://github.com/HFO4/gameboy.live/blob/657501f18a60c486366cd04b87025a7781db1fd1/gb/memory.go#L93

impl IOHandler for MBC3 {
    /// ### 0000-3FFF - ROM Bank 00 (Read Only)
    /// Same as for MBC1.
    ///
    /// ### 4000-7FFF - ROM Bank 01-7F (Read Only)
    /// Same as for MBC1, except that accessing banks 20h, 40h, and 60h is supported now.
    ///
    /// #### A000-BFFF - RAM Bank 00-03, if any (Read/Write)
    /// #### A000-BFFF - RTC Register 08-0C (Read/Write)
    /// Depending on the current Bank Number/RTC Register selection,
    /// this memory space is used to access an 8KByte external RAM Bank, or a single RTC Register.
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => self.read_via_rom_bank(addr),
            0xa000..=0xbfff => self.read_via_ram_bank(addr),
            _ => 0x00,
        }
    }

    /// ### A000-BFFF - RAM Bank 00-03, if any (Read/Write)
    /// ### A000-BFFF - RTC Register 08-0C (Read/Write)
    /// Depending on the current Bank Number/RTC Register selection (see below),
    /// this memory space is used to access an 8KByte external RAM Bank, or a single RTC Register.
    ///
    /// ### 0000-1FFF - RAM and Timer Enable (Write Only)
    /// Mostly the same as for MBC1, a value of 0Ah will enable reading and writing to external RAM
    /// and to the RTC Registers! A value of 00h will disable either.
    ///
    /// ### 2000-3FFF - ROM Bank Number (Write Only)
    /// Same as for MBC1, except that the whole 7 bits of the RAM Bank Number are written directly to this address.
    /// As for the MBC1, writing a value of 00h, will select Bank 01h instead. All other values 01-7Fh select the corresponding ROM Banks.
    ///
    /// ### 4000-5FFF - RAM Bank Number - or - RTC Register Select (Write Only)
    /// As for the MBC1s RAM Banking Mode, writing a value in range for 00h-03h maps the corresponding external RAM Bank (if any)
    /// into memory at A000-BFFF. When writing a value of 08h-0Ch, this will map the corresponding RTC register into memory at A000-BFFF.
    /// That register could then be read/written by accessing any address in that area, typically that is done by using address A000.
    ///
    /// ### 6000-7FFF - Latch Clock Data (Write Only)
    /// When writing 00h, and then 01h to this register, the current time becomes latched into the RTC registers.
    /// The latched data will not change until it becomes latched again, by repeating the write 00h->01h procedure.
    /// This is supposed for <reading> from the RTC registers. This can be proven by reading the latched (frozen) time from the RTC registers,
    /// and then unlatch the registers to show the clock itself continues to tick in background.
    ///
    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    if self.ram_bank <= 0x03 {
                        let i = self.ram_bank * 0x2000 + addr as usize - 0xa000;
                        self.ram[i] = value;
                    } else {
                        self.rtc.set(self.ram_bank as u16, value)
                    }
                }
            }
            0x0000..=0x1fff => {
                self.ram_enabled = value & 0x0f == 0x0a;
            }
            0x2000..=0x3fff => {
                // select lower 7 bits.
                let mut n = (value & 0b0111_1111) as usize;
                // rewrite the 0x00 to 0x01
                if n == 0x00 {
                    n = 0x01;
                }
                self.rom_bank = n;
            }
            0x4000..=0x5fff => {
                // https://github.com/mvdnes/rboy/blob/a1729c729c504f48c9ec47a5c3f35d16c56a5ee3/src/mbc/mbc3.rs#L151
                self.ram_bank = (value & 0x0f) as usize;
            }
            0x6000..=0x7fff => match value {
                0 => self.rtc.unlock(),
                1 => {
                    if !self.rtc.is_locked() {
                        self.rtc.tick();
                    };
                    self.rtc.lock();
                }
                _ => {
                    error! {"Only support 0|1 to tick, but get the value {}", value}
                }
            },
            _ => {}
        }
    }
}

impl Cartridge for MBC3 {
    fn get_meta(&self) -> CartridgeMeta {
        self.meta.clone()
    }
}

impl Drop for MBC3 {
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
