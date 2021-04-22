mod bank;
mod factory;
mod r#impl;
mod rtc;

mod meta;

use std::path::Path;

use factory::CartridgeFactory;
use log::info;
pub use meta::*;

use crate::gameboy::mmu::IOHandler;

pub trait Cartridge: IOHandler + Send + Drop {
    fn get_meta(&self) -> meta::CartridgeMeta;
}

// https://github.com/StarlitGhost/GBOxide

pub fn load_cartridge_from_file(file_path: impl AsRef<Path>) -> Box<dyn Cartridge> {
    info!("Loading cartridge from {:?}", file_path.as_ref().to_str());
    CartridgeFactory::new_catridge(file_path)
}
