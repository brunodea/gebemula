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
        let mut bootstrap_rom: Memory = Memory::new(data.len());
        bootstrap_rom.data = data;

        let mut gb_ram: Memory = Memory::new(0x10000);
        let mut cpu: Cpu = Cpu::new();
        let instructions = Opcode::fetch_instructions(&bootstrap_rom.data);
        cpu.execute_instructions(&instructions, &mut gb_ram);
    } else {
        println!("Invalid number of arguments.");
    }
}
