use cpu::opcode::OpcodeMap;

pub struct Rom {
    pub rom_bytes: Vec<u8>,
}

impl Rom {
    pub fn new(rom_bytes: Vec<u8>) -> Rom {
        Rom {
            rom_bytes: rom_bytes,
        }
    }
}
