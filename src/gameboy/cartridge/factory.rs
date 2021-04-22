use std::fs::File;
use std::io::Read;
use std::path::Path;

use log::info;

use super::meta::CartridgeMeta;
use super::meta::CartridgeType;
use super::r#impl::mbc1::MBC1;
use super::r#impl::mbc2::MBC2;
use super::r#impl::mbc3::MBC3;
use super::r#impl::rom_only::RomOnly;
use super::Cartridge;

/// The Factroy of Cartridge.
pub struct CartridgeFactory {}

// check rom cartridge type here https://ladecadence.net/trastero/listado%20juegos%20gameboy.html
impl CartridgeFactory {
    /// Returns the differrent catridge entity according to the type from rom metadata.
    pub fn new_catridge(path: impl AsRef<Path>) -> Box<dyn Cartridge> {
        let mut rom = Vec::new();
        let mut f = File::open(&path).unwrap();
        f.read_to_end(&mut rom).unwrap();

        let meta = CartridgeMeta::new(&rom);
        let save_path = path.as_ref().to_path_buf().with_extension("sav");
        let rtc_save_path = path.as_ref().to_path_buf().with_extension("rtc");

        info!("cartridge metadata is {:?}", meta);

        let cart: Box<dyn Cartridge> = match meta.get_type() {
            CartridgeType::ROM_ONLY => Box::new(RomOnly::new(meta, rom.to_owned())),
            CartridgeType::ROM_MBC1 => Box::new(MBC1::new(meta, rom.to_owned(), vec![], "")),
            CartridgeType::ROM_MBC1_RAM => {
                let ram = vec![0; meta.get_ram_size()];
                Box::new(MBC1::new(meta, rom.to_owned(), ram, ""))
            }
            CartridgeType::ROM_MBC1_RAM_BATT => {
                let ram = match File::open(&save_path) {
                    Ok(mut ok) => {
                        let mut ram = Vec::new();
                        ok.read_to_end(&mut ram).unwrap();
                        ram
                    }
                    Err(_) => vec![0; meta.get_ram_size()],
                };

                Box::new(MBC1::new(meta, rom.to_owned(), ram, save_path))
            }
            CartridgeType::ROM_MBC2 => Box::new(MBC2::new(meta, rom.to_owned(), vec![0; 512], "")),
            CartridgeType::ROM_MBC2_BATT => {
                let ram = match File::open(&save_path) {
                    Ok(mut ok) => {
                        let mut ram = Vec::new();
                        ok.read_to_end(&mut ram).unwrap();
                        ram
                    }
                    Err(_) => vec![0; 512],
                };

                Box::new(MBC2::new(meta, rom.to_owned(), ram, save_path))
            }
            CartridgeType::ROM_MBC3_TIMER_BATT => Box::new(MBC3::new(
                meta,
                rom.to_owned(),
                vec![],
                save_path,
                rtc_save_path,
            )),
            CartridgeType::ROM_MBC3_TIMER_RAM_BATT => {
                let ram = match File::open(&save_path) {
                    Ok(mut ok) => {
                        let mut ram = Vec::new();
                        ok.read_to_end(&mut ram).unwrap();
                        ram
                    }
                    Err(_) => vec![0; meta.get_ram_size()],
                };
                Box::new(MBC3::new(
                    meta,
                    rom.to_owned(),
                    ram,
                    save_path,
                    rtc_save_path,
                ))
            }
            CartridgeType::ROM_MBC3 => Box::new(MBC3::new(meta, rom.to_owned(), vec![], "", "")),
            CartridgeType::ROM_MBC3_RAM => {
                let ram = vec![0; meta.get_ram_size()];
                Box::new(MBC3::new(meta, rom.to_owned(), ram, "", ""))
            }
            CartridgeType::ROM_MBC3_RAM_BATT => {
                let ram = match File::open(&save_path) {
                    Ok(mut ok) => {
                        let mut ram = Vec::new();
                        ok.read_to_end(&mut ram).unwrap();
                        ram
                    }
                    Err(_) => vec![0; meta.get_ram_size()],
                };
                Box::new(MBC3::new(meta, rom.to_owned(), ram, save_path, ""))
            }
            n => panic!("Sorry, this cartridge type: {:?} is not implemented...", n),
        };

        cart
    }
}
