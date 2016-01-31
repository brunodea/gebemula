use std::collections::HashMap;
use cpu::cpu::Instruction;

#[derive(Debug)]
pub struct Opcode {
    pub opcode: u8,
    pub num_bytes: u8,
    pub cycles: u8,
}

impl Opcode {
    pub fn new(opcode: u8, num_bytes: u8, cycles: u8) -> Opcode {
        Opcode {
            opcode: opcode,
            num_bytes: num_bytes,
            cycles: cycles,
        }
    }
}

//TODO: use tuple struct with one element instead.
#[derive(Debug)]
pub struct OpcodeMap {
    map: HashMap<u8, Opcode>,
}

impl OpcodeMap {
    pub fn new() -> OpcodeMap {
        let mut op_map: HashMap<u8, Opcode> = HashMap::new();
        let mut is_cb: bool = false;
        for opcode in 0x0..0xFF {
            let num_bytes = OpcodeMap::opcode_num_bytes(&opcode);
            let mut cycles = OpcodeMap::opcode_cycles(&opcode, is_cb);
            is_cb = opcode == 0xCB;

            let opcode_obj: Opcode = Opcode::new(opcode, num_bytes, cycles);
            op_map.insert(opcode, opcode_obj);
        }
        OpcodeMap {
            map: op_map,
        }
    }

    //*CAREFUL* -> some values from http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    //are different than in the gameboy manual. Preference to the manual.
    fn opcode_cycles(opcode: &u8, is_bit_op: bool) -> u8 {
        let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;

        if is_bit_op {
            if r4 == 0x6 || r4 == 0xE {
                return 16 as u8
            } else {
                return 8 as u8 
            }
        }

        //all cases for cycle 8
        if (r4 == 0x0 && (l4 == 0x3 || l4 == 0x2 || l4 == 0x7 || l4 == 0xC || l4 == 0xD)) ||
           (r4 == 0x1 && l4 == 0x7) ||
           (r4 == 0x2 && (l4 <= 0x3 || l4 == 0x7 || l4 >= 0xE)) ||
           (r4 == 0x3 && (l4 <= 0x3 || l4 == 0x7)) ||
           (r4 == 0x4 && l4 == 0x7) ||
           (r4 == 0x5 && l4 == 0x7) ||
           (r4 == 0x6 && l4 != 0x3 && l4 != 0x7) ||
           (r4 == 0x7 && l4 == 0x7) ||
           (r4 == 0x8 && (l4 == 0x2 || l4 == 0x3 || l4 == 0xC || l4 == 0xD || l4 == 0x1)) ||
           (r4 == 0x9 && (l4 <= 0x3 || l4 == 0xF || l4 == 0xC || l4 == 0xD)) ||
           (r4 == 0xA && l4 <= 0x3) ||
           (r4 == 0xB && l4 <= 0x3) ||
           (r4 == 0xE) {
           
           8 as u8
        }
        //all cases for cycle 12
        else if *opcode == 0xE0 || *opcode == 0xF0 ||
            (r4 == 0x1 && (l4 <= 0x3 || l4 >= 0xC)) ||
            *opcode == 0xC2 || *opcode == 0xD2 ||
            *opcode == 0xC4 || *opcode == 0xD4 ||
            *opcode == 0xF8 ||
            *opcode == 0xCA || *opcode == 0xDA ||
            *opcode == 0xCC || *opcode == 0xDC ||
            *opcode == 0xCD {

            12 as u8
        }
        //all cases for cycle 16
        else if *opcode == 0xC3  ||
            (r4 == 0x5 && l4 >= 0xC) ||
            *opcode == 0xE8 || 
            *opcode == 0xFA || *opcode == 0xEA {
            
            16 as u8
        } else if *opcode == 0x08 {

            20 as u8
        } else if (r4 == 0x7 && l4 >= 0xC) ||
            (r4 == 0xF && l4 >= 0xC) {

            32 as u8
        } else {

            4 as u8
        }
    }
    
    fn opcode_num_bytes(opcode: &u8) -> u8 {
        let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;

        //all cases for 2 byte long op.
        if (r4 == 0x0 && ((l4 >= 0x1 && l4 <= 0x3) || (l4 == 0xE || l4 == 0xF))) ||
           (r4 == 0x6 && (l4 <= 0x3 || l4 >= 0xC)) ||
           (r4 == 0x8 && ((l4 >= 0x1 && l4 <= 0x3) || (l4 == 0xE || l4 == 0xF))) ||
           (r4 == 0xE && (l4 <= 0x4 || l4 >= 0xC)) ||
           (r4 == 0xB && l4 == 0xC) {

           2 as u8
        }
        //all cases for 3 byte long op.
        else if (r4 == 0x1 && l4 <= 0x3) ||
            (r4 == 0x2 && (l4 == 0xC || l4 == 0xD)) ||
            (r4 == 0x3 && l4 == 0xC) ||
            (r4 == 0x4 && (l4 == 0xC || l4 == 0xD)) ||
            (r4 == 0x8 && l4 == 0x0) ||
            (r4 == 0xA && l4 >= 0xC) ||
            (r4 == 0xC && (l4 == 0xC || l4 == 0xD)) ||
            (r4 == 0xD && l4 == 0xC) {

            3 as u8
        } else {
            1 as u8
        }
    }

    pub fn opcode(&self, opcode: &u8) -> &Opcode {
        match self.map.get(opcode) {
            Some(opcode_obj) => opcode_obj,
            None => panic!("Non existing opcode: {}", opcode),
        }
    }

    pub fn fetch_instructions(&self, bytes: &Vec<u8>) -> Vec<Instruction> {
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

                    let mut instruction: Instruction = Instruction::new();
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


