use std::collections::HashMap;

struct Opcode {
    pub opcode: u8,
    pub num_bytes: u8,
    pub cycles: u8,
    pub flags: u8,
}

impl Opcode {
    pub fn new(opcode: u8, num_bytes: u8, cycles: u8, flags: u8) -> Opcode {
        Opcode {
            opcode: opcode,
            num_bytes: num_bytes,
            cycles: cycles,
            flags: flags,
        }
    }
}

//TODO: use tuple struct with one element instead.
pub struct OpcodeMap {
    map: HashMap<u8, Opcode>,
}

impl OpcodeMap {
    pub fn new() -> OpcodeMap {
        let mut op_map: HashMap<u8, Opcode> = HashMap::new();
        for opcode in 0x0..0xFF {
            let mut num_bytes = 0x1;
            let mut cycles = 0x4;
            
            let l4: u8 = opcode >> 4;
            let r4: u8 = opcode & 0x0F;

            //all cases for 2 byte long op.
            if (r4 == 0x0 && ((l4 >= 0x1 && l4 <= 0x3) || (l4 == 0xE || l4 == 0xF))) ||
               (r4 == 0x6 && (l4 <= 0x3 || l4 >= 0xC)) ||
               (r4 == 0x8 && ((l4 >= 0x1 && l4 <= 0x3) || (l4 == 0xE || l4 == 0xF))) ||
               (r4 == 0xE && (l4 <= 0x4 || l4 >= 0xC)) {

               num_bytes = 0x2;
            } else 

            //all cases for 3 byte long op.
            if (r4 == 0x1 && l4 <= 0x3) ||
               (r4 == 0x2 && (l4 == 0xC || l4 == 0xD)) ||
               (r4 == 0x3 && l4 == 0xC) ||
               (r4 == 0x4 && (l4 == 0xC || l4 == 0xD)) ||
               (r4 == 0x8 && l4 == 0x0) ||
               (r4 == 0xA && l4 >= 0xC) ||
               (r4 == 0xC && (l4 == 0xC || l4 == 0xD)) ||
               (r4 == 0xD && l4 == 0xC) {

               num_bytes = 0x3;
            } 
               
            let opcode_obj: Opcode = Opcode::new(opcode, num_bytes, cycles, 0x0);
            op_map.insert(opcode, opcode_obj);
        }
        OpcodeMap {
            map: op_map,
        }
    }

    pub fn opcode(&self, opcode: &u8) -> &Opcode {
        match self.map.get(opcode) {
            Some(opcode_obj) => opcode_obj,
            None => panic!("Non existing opcode: {}", opcode),
        }
    }

    pub fn fetch_instructions(&self, bytes: &Vec<u8>) -> Vec<Vec<u8>> {
        let mut data_iter = bytes.iter();
        let mut all_instructions = Vec::new();
        loop {
            match data_iter.next() {
                Some(opcode_byte) => {
                    let mut nbytes: u8 = 0x0;
                    //0xCB is the prefix for bit operations
                    if *opcode_byte == 0xCB {
                        nbytes = 0x2; //bit operation always require 2 bytes.
                    } else {
                        nbytes = self.opcode(opcode_byte).num_bytes;
                    }

                    let mut instruction: Vec<u8> = Vec::new();
                    instruction.push(*opcode_byte);

                    //starts from 1 because the first byte was already added.
                    for n in 1..nbytes {
                        match data_iter.next() {
                            Some(byte) => {
                                instruction.push(*byte);
                            },
                            None => panic!("Invalid opcode instruction size."),
                        }
                    }

                    all_instructions.push(instruction);
                },
                None => break,
            }
        }
        all_instructions
    }
}


