use mem::mem::Memory;
use mem::consts;
use std::str;

pub fn cartridge_type_str(memory: &Memory) -> String {
    let byte: u8 = memory.read_byte(consts::CARTRIDGE_TYPE_ADDR);
    match byte {
        0x1F => {
            return "Pocket Camera".to_owned();
        },
        0xFD => {
            return "Bandai TAMA5".to_owned();
        },
        0xFE => {
            return "Hudson HuC-3".to_owned();
        },
        0xFF => {
            return "Hudson HuC-1".to_owned();
        },
        _ => (),
    }

    let ram: &str = "+RAM";
    let bat: &str = "+BATTERY";
    let sram: &str = "+SRAM";
    let timer: &str = "+TIMER";
    let rumble: &str = "+RUMBLE";
    let mm01: &str = "+MM01";
    let mbc1: &str = "+MBC1";
    let mbc2: &str = "+MBC2";
    let mbc3: &str = "+MBC3";
    let mbc5: &str = "+MBC5";
    
    let mut res: String = "".to_owned();
    if byte == 0x00 {
        res = " ONLY".to_owned();
    } else if byte <= 0x3 {
        res = mbc1.to_owned();
        if byte >= 0x2 {
            res = res + ram;
        }
        if byte == 0x3 {
            res = res + bat;
        }
    } else if byte <= 0x6 {
        res = mbc2.to_owned();
        if byte == 0x6 {
            res = res + bat;
        }
    } else if byte <= 0x9 {
        res = ram.to_owned();
        if byte == 0x9 {
            res = res + bat;
        }
    } else if byte <= 0xD {
        res = mm01.to_owned();
        if byte >= 0xC {
            res = res + sram;
        }
        if byte == 0xD {
            res = res + bat;
        }
    } else if byte <= 0x13 {
        res = mbc3.to_owned();
        if byte != 0x11 {
            if byte <= 0x10 {
                res = res + timer + bat;
            }
            if byte != 0xF {
                res = res + ram;
            }
            if byte == 0x13 {
                res = res + bat;
            }
        }
    } else if byte <= 0x1E {
        res = mbc5.to_owned();
        if byte <= 0x1B {
            res = res + ram;
        }
        if byte == 0x1B {
            res = res + bat;
        }
        if byte >= 0x1C {
            res = res + rumble;
        }
        if byte >= 0x1D {
            res = res + sram;
        }
        if byte == 0x1E {
            res = res + bat;
        }
    }

    "ROM".to_owned() + &res
}

pub fn game_title_str(memory: &Memory) -> String {
    let game_title_u8: &mut Vec<u8> = &mut Vec::new();
    for byte in consts::GAME_TITLE_ADDR_START..(consts::GAME_TITLE_ADDR_END + 1) {
        if byte == 0 {
            break;
        }
        game_title_u8.push(memory.read_byte(byte));
    }
    let game_title: &str = match str::from_utf8(&game_title_u8) {
        Ok(v) => v,
        Err(_) => "Undefined",
    };

    game_title.to_owned()
}

