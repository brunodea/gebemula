mod cpu;

use std::env;
use std::io::Read;
use std::fs::File;
use std::collections::HashMap;

use cpu::cpu::{Cpu, GenReg8, GenReg16};

struct Opcode {
    opcode: u8,
    num_bytes: u8,
    cycles: u8,
    flags: u8,
}

impl Opcode {
    fn new(opcode: u8, num_bytes: u8, cycles: u8, flags: u8) -> Opcode {
        Opcode {
            opcode: opcode,
            num_bytes: num_bytes,
            cycles: cycles,
            flags: flags,
        }
    }
}

struct OpcodeMap {
    map: HashMap<u8, Opcode>,
}

impl Default for OpcodeMap {
    fn default() -> Self {
        let mut op_map: HashMap<u8, Opcode> = HashMap::new();
        for opcode in 0x0..0xFF {
            let mut num_bytes = 0x1;
            let mut cycles = 0x4;
            
            let l4: u8 = opcode >> 4;
            let r4: u8 = opcode & 0x0F;

            //all cases for 2 byte long op.
            if (r4 == 0x0 && ((l4 >= 0x1 && l4 <= 0x3) || (l4 == 0xE || l4 == 0xF))) ||
               (r4 == 0x2 && (l4 == 0xE || l4 == 0xF)) ||
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
}

impl OpcodeMap {


    fn left_4(byte: &u8) -> u8 {
        byte >> 4
    }
    fn right_4(byte: &u8) -> u8 {
        byte & 0x0F
    }
    fn is_ld_8(opcode: &u8) -> bool {
        let l4: u8 = OpcodeMap::left_4(opcode);
        let r4: u8 = OpcodeMap::right_4(opcode);

        //0x76 = HALT
        *opcode != 0x76 && (
            (l4 >= 0x4 && l4 <= 0x7) ||
            (l4 <= 0x3 && (r4 == 0x2 || r4 == 0x6 || r4 == 0xA || r4 == 0xE)) ||
            ((l4 == 0xE || l4 == 0xF) && (r4 == 0x0 || r4 == 0x2 || r4 == 0xA))
        )
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut boostrap_rom = File::open(&args[1]).unwrap();
        let mut data: Vec<u8> = Vec::new();
        boostrap_rom.read_to_end(&mut data).unwrap();

        let mut cpu: Cpu = Cpu::default();
        println!("{:?}", cpu);

        let op_map: OpcodeMap = OpcodeMap::default();

        let mut word = Vec::new();
        let mut data_iter = data.iter();

        let bitop_bytes: u8 = 0x2;
        loop {
            match data_iter.next() {
                Some(opcode) => {
                    let mut num_bytes: &u8 = &op_map.map.get(opcode).unwrap().num_bytes;
                    
                    let mut nbytes: u8 = *num_bytes;
                    //bit operation
                    if *opcode == 0xCB {
                        nbytes = bitop_bytes; //-1 because 1 of the bytes is the opcode, added before the loop.
                    }

                    nbytes = nbytes - 1;
                    word.push(opcode);
                    for n in 0..nbytes {
                        word.push(data_iter.next().unwrap());
                    }

                    for w in &word {
                        print!("{}", format!("{:01$x}", w, 2));
                    }

                    println!("");
                    word.clear();
                },
                None => { break },
            }
        }
    } else {
        println!("Invalid number of arguments.");
    }
}
