#[derive(Debug)]
pub enum BankMode {
    Rom,
    Ram,
}

pub trait MemoryBank {
    /// Returns the current num of rom bank reg.
    fn get_rom_bank_num(&self) -> usize;

    /// Returns the current num of ram bank reg.
    fn get_ram_bank_num(&self) -> usize;

    /// Read a byte from given address via bank controller.
    fn read_via_rom_bank(&self, addr: u16) -> u8;

    /// Read a byte from given address via bank controller.
    fn read_via_ram_bank(&self, addr: u16) -> u8;

    /// Write a byte from given address via bank controller.
    fn write_via_ram_bank(&mut self, addr: u16, value: u8);
}
