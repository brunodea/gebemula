pub mod rom;

const ROM_BANK_SIZE: u16 = 0x4000;
const RAM_BANK_SIZE: u16 = 0x2000;

pub trait Mapper {
    /// Handles a read from the 0x0000-0x7FFF ROM/MBC area.
    fn read_rom(&self, address: u16) -> u8;
    /// Handles a write to the 0x0000-0x7FFF ROM/MBC area.
    fn write_rom(&mut self, address: u16, data: u8);

    /// Handles a read from the 0xA000-0xBFFF SRAM/IO area.
    fn read_ram(&self, address: u16) -> u8;
    /// Handles a write to the 0xA000-0xBFFF SRAM/IO area.
    fn write_ram(&mut self, address: u16, data: u8);
}

/// Mapper that simulates having no cartridge inserted.
pub struct NullMapper;

impl Mapper for NullMapper {
    fn read_rom(&self, _address: u16) -> u8 { 0xFF }
    fn write_rom(&mut self, _address: u16, _data: u8) {}

    fn read_ram(&self, _address: u16) -> u8 { 0xFF }
    fn write_ram(&mut self, _address: u16, _data: u8) {}
}