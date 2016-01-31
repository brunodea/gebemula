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
}


