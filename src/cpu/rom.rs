use cpu::opcode::OpcodeMap;

pub struct Rom {
    rom_bytes: Vec<u8>,
    curr_byte_position: usize,
    opcode_map: OpcodeMap,
}

//iterate over the words in a Rom.
impl Iterator for Rom {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        if self.curr_byte_position < self.rom_bytes.len() {
            let opcode: &u8 = &self.rom_bytes[self.curr_byte_position];
            let mut num_bytes: &u8 = &self.opcode_map.opcode(opcode).num_bytes;
            let mut nbytes: u8 = *num_bytes;
            //bit operation
            if *opcode == 0xCB {
                nbytes = 0x2; //bit operation always require 2 bytes.
            }

            let mut word: Vec<u8> = Vec::new();
            word.push(*opcode);

            self.curr_byte_position += 1;
            //starts from 1 because the first byte was already added.
            for n in 1..nbytes {
                if self.curr_byte_position >= self.rom_bytes.len() {
                    //TODO better message.
                    panic!("Invalid rom instruction.");
                }
                word.push(self.rom_bytes[self.curr_byte_position]);
                self.curr_byte_position += 1;
            }

            Some(word)
        } else {
            None
        }
    }
}

impl Rom {
    pub fn new(rom_bytes: Vec<u8>, opcode_map: OpcodeMap) -> Rom {
        Rom {
            rom_bytes: rom_bytes,
            curr_byte_position: 0,
            opcode_map: opcode_map,
        }
    }
}
