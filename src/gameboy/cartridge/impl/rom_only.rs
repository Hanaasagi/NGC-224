use backtrace::Backtrace;
use log::error;
use log::info;

use super::Cartridge;
use super::CartridgeMeta;
use super::IOHandler;

/// Small games of not more than 32KBytes ROM do not require a MBC chip for ROM banking.
/// The ROM is directly mapped to memory at 0000-7FFFh.
/// Optionally up to 8KByte of RAM could be connected at A000-BFFF,
/// even though that could require a tiny MBC-like circuit, but no real MBC chip.
pub struct RomOnly {
    meta: CartridgeMeta,
    rom: Vec<u8>,
}

impl RomOnly {
    /// Returns Rom-Only Cartridge.
    pub fn new(meta: CartridgeMeta, rom: Vec<u8>) -> Self {
        RomOnly { meta, rom }
    }
}

impl IOHandler for RomOnly {
    /// Read a byte from address.
    fn read_byte(&self, a: u16) -> u8 {
        self.rom[a as usize]
    }

    /// Write a byte to address.
    fn write_byte(&mut self, _: u16, _: u8) {
        let bt = Backtrace::new();
        error!("Rom-Only cartridge is not writable {:?}", bt);
    }
}

impl Cartridge for RomOnly {
    fn get_meta(&self) -> CartridgeMeta {
        self.meta.clone()
    }
}

impl Drop for RomOnly {
    fn drop(&mut self) {
        info!("save success when drop the cartridge object.")
    }
}
