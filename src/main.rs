mod cpu;
mod mem;

use std::env;
use std::io::Read;
use std::fs::File;

use cpu::cpu::Cpu;
use cpu::opcode::Opcode;
use mem::mem::Memory;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut data: Vec<u8> = Vec::new();
        File::open(&args[1]).unwrap().read_to_end(&mut data).unwrap();

        let mut mem: Memory = Memory::new();
        let mut cpu: Cpu = Cpu::new();

        mem.read_bootstrap_rom(&data);
        //starting point = bootstrap rom's initial position
        cpu.execute_instructions(0x0, &mut mem);
    } else {
        println!("Invalid number of arguments.");
    }
}
