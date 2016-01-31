mod cpu;

use std::env;
use std::io::Read;
use std::fs::File;

use cpu::cpu::{Cpu, GenReg8, GenReg16};
use cpu::rom::Rom;
use cpu::opcode::OpcodeMap;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut boostrap_rom = File::open(&args[1]).unwrap();
        let mut data: Vec<u8> = Vec::new();
        boostrap_rom.read_to_end(&mut data).unwrap();

        let mut cpu: Cpu = Cpu::default();
        println!("{:?}", cpu);

        let op_map: OpcodeMap = OpcodeMap::new();
        let mut rom: Rom = Rom::new(data, op_map);

        loop {
            match rom.next() {
                Some(word) => {
                    for w in &word {
                        print!("{}", format!("{:01$x}", w, 2));
                    }
                    println!("");
                },
                None => { break },
            }
        }
    } else {
        println!("Invalid number of arguments.");
    }
}
