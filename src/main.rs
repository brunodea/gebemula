mod cpu;
mod mem;

use std::env;
use std::io::Read;
use std::fs::File;

use cpu::cpu::Cpu;
use cpu::opcode::Opcode;
use mem::rom::Rom;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut boostrap_rom = File::open(&args[1]).unwrap();
        let mut data: Vec<u8> = Vec::new();
        boostrap_rom.read_to_end(&mut data).unwrap();

        let mut cpu: Cpu = Cpu::new();
        let rom: Rom = Rom::new(data);
        let instructions = Opcode::fetch_instructions(&rom.rom_bytes);
        cpu.execute_instructions(&instructions);
    } else {
        println!("Invalid number of arguments.");
    }
}
