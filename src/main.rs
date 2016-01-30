mod cpu;

use std::env;
use std::io::Read;
use std::fs::File;

use cpu::cpu::{Cpu, GenReg8, GenReg16};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut boostrap_rom = File::open(&args[1]).unwrap();
        let mut data = Vec::new();
        boostrap_rom.read_to_end(&mut data).unwrap();

        let mut cpu: Cpu = Cpu::default();
        cpu.reg8(GenReg8::A);
        cpu.reg16(GenReg16::AF);
        println!("{:?}", cpu);
        //for word in data {
        //    println!("{:x}", word);
        //} 
    } else {
        println!("Invalid number of arguments.");
    }
}
