use crate::mem::Memory;
use crate::mem::mapper::{Mapper, NullMapper};
use crate::mem::mapper::rom::RomMapper;
use crate::mem::mapper::mbc1::Mbc1Mapper;
use crate::mem::mapper::mbc2::Mbc2Mapper;
use crate::mem::mapper::mbc3::Mbc3Mapper;
use crate::mem::mapper::mbc5::Mbc5Mapper;
use std::str;
use std::cmp;

pub const GAME_TITLE_ADDR_START: u16 = 0x134;
pub const GAME_TITLE_ADDR_END: u16 = 0x142;
pub const CARTRIDGE_TYPE_ADDR: u16 = 0x147;
const ROM_SIZE_ADDR: u16 = 0x148;
const RAM_SIZE_ADDR: u16 = 0x149;

pub enum MapperType {
    Rom,

    // Standard mappers
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
    Mbc6,
    Mbc7,
    Mmm01,

    // Non-standard mappers & hardware
    Huc1,
    Huc3,
    Tama5,
    PocketCamera,

    Unknown,
}

bitflags! {
    pub struct CartExtraHardware: u32 {
        const NONE_HW       = 0;
        const RAM           = 1 << 0;
        const BATTERY       = 1 << 1;
        const RTC           = 1 << 2;
        const RUMBLE        = 1 << 3;
        const ACCELEROMETER = 1 << 4;
    }
}

pub fn cartridge_type_string(mapper: MapperType, extra_hw: CartExtraHardware) -> String {
    let mut s = match mapper {
        MapperType::Rom => "ROM",

        MapperType::Mbc1 => "MBC1",
        MapperType::Mbc2 => "MBC2",
        MapperType::Mbc3 => "MBC3",
        MapperType::Mbc5 => "MBC5",
        MapperType::Mbc6 => "MBC6",
        MapperType::Mbc7 => "MBC7",
        MapperType::Mmm01 => "MMM01",

        MapperType::Huc1 => "HuC1",
        MapperType::Huc3 => "HuC3",
        MapperType::Tama5 => "TAMA5",
        MapperType::PocketCamera => "Pocket Camera",

        MapperType::Unknown => "???",
    }.to_owned();

    if extra_hw.contains(CartExtraHardware::RAM) {
        s.push_str("+RAM");
    }
    if extra_hw.contains(CartExtraHardware::BATTERY) {
        s.push_str("+BATTERY");
    }
    if extra_hw.contains(CartExtraHardware::RTC) {
        s.push_str("+CartExtraHardware::RTC");
    }
    if extra_hw.contains(CartExtraHardware::RUMBLE) {
        s.push_str("+RUMBLE");
    }

    s
}

pub fn cart_type_from_id(id: u8) -> (MapperType, CartExtraHardware) {
    match id {
        0x00 => (MapperType::Rom, CartExtraHardware::NONE_HW),
        0x01 => (MapperType::Mbc1, CartExtraHardware::NONE_HW),
        0x02 => (MapperType::Mbc1, CartExtraHardware::RAM),
        0x03 => (MapperType::Mbc1, CartExtraHardware::RAM | CartExtraHardware::BATTERY),

        0x05 => (MapperType::Mbc2, CartExtraHardware::NONE_HW),
        0x06 => (MapperType::Mbc2, CartExtraHardware::RAM | CartExtraHardware::BATTERY),

        0x08 => (MapperType::Rom, CartExtraHardware::RAM),
        0x09 => (MapperType::Rom, CartExtraHardware::RAM | CartExtraHardware::BATTERY),

        0x0B => (MapperType::Mmm01, CartExtraHardware::NONE_HW),
        0x0C => (MapperType::Mmm01, CartExtraHardware::RAM),
        0x0D => (MapperType::Mmm01, CartExtraHardware::RAM | CartExtraHardware::BATTERY),

        0x0F => (MapperType::Mbc3, CartExtraHardware::RTC | CartExtraHardware::BATTERY),
        0x10 => (MapperType::Mbc3, CartExtraHardware::RAM | CartExtraHardware::RTC | CartExtraHardware::BATTERY),
        0x11 => (MapperType::Mbc3, CartExtraHardware::NONE_HW),
        0x12 => (MapperType::Mbc3, CartExtraHardware::RAM),
        0x13 => (MapperType::Mbc3, CartExtraHardware::RAM | CartExtraHardware::BATTERY),

        0x19 => (MapperType::Mbc5, CartExtraHardware::NONE_HW),
        0x1A => (MapperType::Mbc5, CartExtraHardware::RAM),
        0x1B => (MapperType::Mbc5, CartExtraHardware::RAM | CartExtraHardware::BATTERY),
        0x1C => (MapperType::Mbc5, CartExtraHardware::RUMBLE),
        0x1D => (MapperType::Mbc5, CartExtraHardware::RAM | CartExtraHardware::RUMBLE),
        0x1E => (MapperType::Mbc5, CartExtraHardware::RAM | CartExtraHardware::BATTERY | CartExtraHardware::RUMBLE),

        0x20 => (MapperType::Mbc6, CartExtraHardware::RAM | CartExtraHardware::BATTERY),

        0x22 => (MapperType::Mbc7, CartExtraHardware::RAM | CartExtraHardware::BATTERY | CartExtraHardware::ACCELEROMETER),

        0xFC => (MapperType::PocketCamera, CartExtraHardware::NONE_HW),
        0xFD => (MapperType::Tama5, CartExtraHardware::NONE_HW),
        0xFE => (MapperType::Huc3, CartExtraHardware::NONE_HW),
        0xFF => (MapperType::Huc1, CartExtraHardware::RAM | CartExtraHardware::BATTERY),

        _ => (MapperType::Unknown, CartExtraHardware::NONE_HW),
    }
}

pub fn game_title_str(memory: &Memory) -> String {
    let game_title_u8 = &mut Vec::new();
    for byte in GAME_TITLE_ADDR_START..(GAME_TITLE_ADDR_END + 1) {
        if byte == 0 {
            break;
        }
        game_title_u8.push(memory.read_byte(byte));
    }
    let game_title = match str::from_utf8(&game_title_u8) {
        Ok(v) => v,
        Err(_) => "Undefined",
    };

    game_title.to_owned()
}

pub fn parse_rom_size(id: u8) -> usize {
    match id {
        0x00..=0x08 => (32 * 1024) << id,
        _ => panic!("Unknown ROM size: {:#02X}", id),
    }
}

pub fn parse_ram_size(id: u8) -> usize {
    match id {
        0x00 => 0,
        0x01 => 2 * 1024,
        0x02 => 8 * 1024,
        0x03 => 32 * 1024,
        0x04 => 128 * 1024,
        0x05 => 64 * 1024,
        _ => panic!("Unknown cartridge RAM size: {:#02X}", id),
    }
}

pub fn load_cartridge(rom: &[u8], battery: &[u8]) -> Box<dyn Mapper> {
    if rom.len() == 0 {
        println!("Warning: No cartridge inserted.");
        return Box::new(NullMapper);
    }

    if rom.len() < 0x200 {
        // Files this small aren't even large enough to have a header.
        panic!("Input ROM is too small. ({} bytes)", rom.len());
    }

    let cart_type_id = rom[CARTRIDGE_TYPE_ADDR as usize];
    let (mapper_type, extra_hw) = cart_type_from_id(cart_type_id);
    let rom_size = parse_rom_size(rom[ROM_SIZE_ADDR as usize]);
    let ram_size = match mapper_type {
        MapperType::Mbc2 => 512, // MBC2 always has 512 nibbles of internal SRAM
        _ => parse_ram_size(rom[RAM_SIZE_ADDR as usize]),
    };

    // Copy ROM data from file to backing memory
    let mut rom_data = vec![0xFF; rom_size].into_boxed_slice();
    let copy_len = cmp::min(rom.len(), rom_data.len());
    &rom_data[..copy_len].copy_from_slice(&rom[..copy_len]);

    // Initialize RAM backing memory
    let expected_battery_size = ram_size + if extra_hw.contains(CartExtraHardware::BATTERY) { 48 } else { 0 };
    if !battery.is_empty() && battery.len() != expected_battery_size {
        println!(
            "WARNING: Battery file has unexpected size: {:#X}, expected {:#X}",
            battery.len(),
            expected_battery_size
        );
    }

    let mut ram_data = vec![0xFF; ram_size].into_boxed_slice();
    let copy_len = cmp::min(battery.len(), ram_data.len());
    &ram_data[..copy_len].copy_from_slice(&battery[..copy_len]);

    match mapper_type {
        MapperType::Rom => Box::new(RomMapper::new(
            rom_data,
            ram_data,
            extra_hw.contains(CartExtraHardware::BATTERY),
        )),
        MapperType::Mbc1 => Box::new(Mbc1Mapper::new(
            rom_data,
            ram_data,
            extra_hw.contains(CartExtraHardware::BATTERY),
        )),
        MapperType::Mbc2 => Box::new(Mbc2Mapper::new(
            rom_data,
            ram_data,
            extra_hw.contains(CartExtraHardware::BATTERY),
        )),
        MapperType::Mbc3 => Box::new(Mbc3Mapper::new(
            rom_data,
            ram_data,
            extra_hw.contains(CartExtraHardware::BATTERY),
            extra_hw.contains(CartExtraHardware::RTC),
        )),
        MapperType::Mbc5 => Box::new(Mbc5Mapper::new(
            rom_data,
            ram_data,
            extra_hw.contains(CartExtraHardware::BATTERY),
        )),
        _ => panic!(
            "Cartridges of type {:#X} are not yet supported.",
            cart_type_id
        ),
    }
}
