use std::collections::HashMap;
use cpu::cpu::instruction::Instruction;

//TODO verify if all opcodes are in the correct AddressingMode.

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ImmediateExt,
    Relative,
    Extended,
    Register,
    RegisterIndirect,
    Implied, //no specific Addressing Mode: instruction uses only 1 byte.
    NoAddr16, //for opcodes that don't rely on Addressing Mode (e.g. STOP)
    
    Invalid, //for opcodes not available (e.g. 0xCB by itself, 0xE3, 0xDB, 0xED, etc)
}

impl AddressingMode {
    fn is_immediate(opcode: u8) -> bool {
        let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;

        //from 8-bit Arithmetic and Logic
        (l4 >= 0xC && (r4 == 0xE || r4 == 0x6)) ||
        //GB Exclusive
        (l4 <= 0x3 && (r4 == 0x6 || r4 == 0xE)) ||
        (l4 >= 0xE && (r4 == 0x0 || r4 == 0x2)) ||
        opcode == 0xF8
    }
    fn is_immediate_ext(opcode: u8) -> bool {
        let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;
        //from LD 16-bit
        (l4 <= 0x3 && r4 == 0x1) ||
        //from Jump,Call and Return
        opcode == 0xC3 || opcode == 0xDA || opcode == 0xD2 || opcode == 0xCA || opcode == 0xC2 ||
        opcode == 0xCD || opcode == 0xDC || opcode == 0xD4 || opcode == 0xCC || opcode == 0xC4 ||
        //GB Exclusive
        (l4 >= 0xE && r4 == 0xA)
    }
    fn is_relative(opcode: u8) -> bool {
        //from Jump,Call and Return
        opcode == 0x18 || opcode == 0x38 || opcode == 0x30 || opcode == 0x28 || opcode == 0x20
    }
    fn is_extended(opcode: u8) -> bool {
        //GB Exclusive
        opcode == 0x08
    }
    fn is_register(opcode: u8) -> bool {
        let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;

        //from LD 8-bit
        (l4 >= 0x4 && l4 <= 0x6 && r4 != 0xE && r4 != 0x6) ||
        (l4 == 0x7 && r4 >= 0x8 && r4 != 0xE) || 
        //from LD 16-bit
        opcode == 0xF9 ||
        //from 8-bit Arithmetic and Logic
        (l4 >= 0x8 && l4 <= 0xB && r4 != 0x6 && r4 != 0xE) ||
        (l4 <= 0x2 && (r4 == 0xC || r4 == 0xD || r4 == 0x4 || r4 == 0x5)) ||
        opcode == 0x3C || opcode == 0x3D ||
        //GB Exclusive
        opcode == 0xE8
    }
    fn is_register_indirect(opcode: u8) -> bool {
        let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;

        //from LD 8-bit
        (l4 >= 0x4 && l4 <= 0x7 && (r4 == 0xE || r4 == 0x6)) || (opcode >= 0x70 && opcode <= 0x77) ||
        opcode == 0x02 || opcode == 0x12 ||
        //from LD 16-bit
        (l4 >= 0xC && (r4 == 0x1 || r4 == 0x5)) ||
        //from 8-bit Arithmetic and Logic
        ((l4 >= 0x8 && l4 <= 0xB) && (r4 == 0x6 || r4 == 0xE)) ||
        opcode == 0x34 || opcode == 0x35 || opcode == 0x74 || 
        //from Jump,Call and Return
        opcode == 0xC9 || opcode == 0xD8 || opcode == 0xD0 || opcode == 0xC8 || opcode == 0xC0 ||
        opcode == 0xE9 ||
        //GB Exclusive
        (l4 <= 0x3 && (r4 == 0x2 || r4 == 0xA)) ||
        opcode == 0x32 || opcode == 0xD9
    }
    fn is_implied(opcode: u8) -> bool {
        let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;
        //from General-Purpose AF Operation
        opcode == 0x27 || opcode == 0x2F || opcode == 0x3F || opcode == 0x37 ||
        //from Bit Arithmetic (INC and DEC)
        (l4 <= 0x3 && (r4 == 0x3 || r4 == 0xB)) ||
        //from Jump,Call and Return
        (l4 >= 0xC && (r4 == 0x7 || r4 == 0xF)) ||
        //from Miscellaneous CPU Control
        opcode == 0x00 || opcode == 0xF3 || opcode == 0xFB ||
        //from Rotate and Shift
        (l4 <= 0x1 && (r4 == 0x7 || r4 == 0xF)) ||
        //from 16-bit Arithmetic
        (l4 <= 0x3 && r4 == 0x9)
    }
    fn is_noaddr16(opcode: u8) -> bool {
        //GB Exclusive
        opcode == 0x10
    }

    pub fn from_opcode(prefix: u8, opcode: u8) -> AddressingMode {
        //let l4: u8 = opcode >> 4;
        let r4: u8 = opcode & 0x0F;
        if prefix == 0xCB {
            //Rotate and Shifts
            if r4 != 0x6 && r4 != 0xE {
                return AddressingMode::Register
            } else {
                return AddressingMode::RegisterIndirect
            }
        } else {
            if AddressingMode::is_immediate(opcode) {
                return AddressingMode::Immediate
            } else if AddressingMode::is_immediate_ext(opcode) {
                return AddressingMode::ImmediateExt
            } else if AddressingMode::is_relative(opcode) {
                return AddressingMode::Relative
            } else if AddressingMode::is_extended(opcode) {
                return AddressingMode::Extended
            } else if AddressingMode::is_register(opcode) {
                return AddressingMode::Register
            } else if AddressingMode::is_register_indirect(opcode) {
                return AddressingMode::RegisterIndirect
            } else if AddressingMode::is_implied(opcode) {
                return AddressingMode::Implied
            } else if AddressingMode::is_noaddr16(opcode) {
                return AddressingMode::NoAddr16
            } else {
                return AddressingMode::Invalid
            }
        }
        panic!("No addressing mode for opcode 0x{} and prefix 0x{}", format!("{:01$x}", opcode, 2), format!("{:01$x}", prefix, 2));
    }
}

#[derive(Debug)]
pub struct Opcode {
    pub opcode: u8,
    pub prefix: u8,
    pub num_bytes: u8,
    pub cycles: u8,
}

impl Opcode {
    pub fn new(prefix: u8, opcode: u8, cycles: u8) -> Opcode {
        let mut nb = 0;
        if prefix == 0xCB {
            nb = 2;
        } else {
            nb = OpcodeMap::opcode_num_bytes(AddressingMode::from_opcode(prefix, opcode));
        }
        Opcode {
            opcode: opcode,
            prefix: prefix,
            num_bytes: nb,
            cycles: cycles,
        }
    }
}

//TODO: use tuple struct with one element instead.
#[derive(Debug)]
pub struct OpcodeMap {
    map: HashMap<u16, Opcode>,
}

impl OpcodeMap {
    pub fn new() -> OpcodeMap {
        let mut op_map: HashMap<u16, Opcode> = HashMap::new();
        for opcode in 0x0..0xFF {
            let cycles: u8 = OpcodeMap::opcode_cycles(opcode, false);
            let opcode_obj: Opcode = Opcode::new(0x0, opcode, cycles);
            op_map.insert(opcode as u16, opcode_obj);
        }
        //for cb-prefixed code.
        for opcode in 0x0..0xFF {
            let cycles: u8 = OpcodeMap::opcode_cycles(opcode, true);
            let opcode_obj: Opcode = Opcode::new(0xCB, opcode, cycles);
            let key: u16 = 0xCB00 | opcode as u16;
            op_map.insert(key, opcode_obj);
        }
        OpcodeMap {
            map: op_map,
        }
    }

    fn opcode_num_bytes(addr_mode: AddressingMode) -> u8 {
        match addr_mode {
            AddressingMode::Register         |
            AddressingMode::RegisterIndirect |
            AddressingMode::Implied => 1,

            AddressingMode::Immediate |
            AddressingMode::Relative  |
            AddressingMode::NoAddr16 => 2,

            AddressingMode::ImmediateExt |
            AddressingMode::Extended => 3,

            AddressingMode::Invalid => 0,
        }
    }

    //*CAREFUL* -> some values from http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
    //are different than in the gameboy manual. Preference to the manual.
    fn opcode_cycles(opcode: u8, is_bit_op: bool) -> u8 {
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
        else if opcode == 0xE0 || opcode == 0xF0 ||
            (r4 == 0x1 && (l4 <= 0x3 || l4 >= 0xC)) ||
            opcode == 0xC2 || opcode == 0xD2 ||
            opcode == 0xC4 || opcode == 0xD4 ||
            opcode == 0xF8 ||
            opcode == 0xCA || opcode == 0xDA ||
            opcode == 0xCC || opcode == 0xDC ||
            opcode == 0xCD {

            12 as u8
        }
        //all cases for cycle 16
        else if opcode == 0xC3  ||
            (r4 == 0x5 && l4 >= 0xC) ||
            opcode == 0xE8 || 
            opcode == 0xFA || opcode == 0xEA {
            
            16 as u8
        } else if opcode == 0x08 {

            20 as u8
        } else if (r4 == 0x7 && l4 >= 0xC) ||
            (r4 == 0xF && l4 >= 0xC) {

            32 as u8
        } else {

            4 as u8
        }
    }

    pub fn opcode(&self, opcode: u8) -> &Opcode {
        match self.map.get(&(opcode as u16)) {
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
                    let opcode_obj: &Opcode = self.opcode(*opcode_byte);
                    let mut nbytes = opcode_obj.num_bytes;
                    if *opcode_byte == 0xCB {
                        nbytes = 2; //1 for supporting the CB prefix + 1 for the CB-prefixed instruction.
                    } else if nbytes == 0 {
                        println!("While fetching instructions, found invalid with 0 bytes: {:01$x}", *opcode_byte, 2);
                        continue;
                    }

                    let mut instruction: Instruction = vec![0; nbytes as usize];
                    instruction[0] = *opcode_byte;

                    //starts from 1 because the first byte was already added.
                    for n in 1..nbytes {
                        match data_iter.next() {
                            Some(byte) => {
                                instruction[n as usize] = *byte;
                            },
                            None => panic!("Invalid opcode instruction size."),
                        }
                    }

                    print!("0x");
                    for i in instruction.iter() {
                        print!("{:01$x}", i, 2);
                    }
                    println!("");
                    all_instructions.push(instruction);
                },
                None => break,
            }
        }
        all_instructions
    }
}


