mod cpu;

use std::env;
use std::io::Read;
use std::fs::File;

use cpu::cpu::{Cpu, GenReg8, GenReg16};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut boostrap_rom = File::open(&args[1]).unwrap();
        let mut data: Vec<u8> = Vec::new();
        boostrap_rom.read_to_end(&mut data).unwrap();

        let mut cpu: Cpu = Cpu::default();
        println!("{:?}", cpu);

        let mut word = Vec::new();
        let mut data_iter = data.iter();
        let l4_fn = |opcode: &u8| -> u8 { opcode >> 4 };
        let r4_fn = |opcode: &u8| -> u8 { opcode & 0x0f };

        loop {
            match data_iter.next() {
                Some(opcode) => {
                    word.push(opcode);
                    //word.push(data_iter.next().unwrap());
                    for w in &word {
                        print!("{:x}", w);
                    }
                    println!("");
                    word.clear();
                },
                None => { break },
        }
    } else {
        println!("Invalid number of arguments.");
    }
}
