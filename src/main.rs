#![feature(plugin)]
#![plugin(clippy)]

mod cpu;
mod mem;
mod util;

use std::env;
use std::io::Read;
use std::fs::File;

use cpu::Cpu;
use mem::mem::Memory;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 3 {
        let mut bootstrap_data: Vec<u8> = Vec::new();
        File::open(&args[1]).unwrap().read_to_end(&mut bootstrap_data).unwrap();

        let mut game_data: Vec<u8> = Vec::new();
        File::open(&args[2]).unwrap().read_to_end(&mut game_data).unwrap();
        
        let mut mem: Memory = Memory::new();
        let mut cpu: Cpu = Cpu::new();

        mem.load_game_rom(&game_data);
        mem.load_bootstrap_rom(&bootstrap_data);
        mem.write_byte(0xFF44, 0x90); //for bypassing 'waiting for screen frame'.
        //starting point = bootstrap rom's initial position
        cpu.execute_instructions(0x0, &mut mem);
    } else {
        println!("Invalid number of arguments.");
    }
}
