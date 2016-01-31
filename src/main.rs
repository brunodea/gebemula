mod cpu;

use std::env;
use std::io::Read;
use std::fs::File;

use cpu::cpu::Cpu;
use cpu::rom::Rom;
use cpu::opcode::OpcodeMap;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut boostrap_rom = File::open(&args[1]).unwrap();
        let mut data: Vec<u8> = Vec::new();
        boostrap_rom.read_to_end(&mut data).unwrap();

        let cpu: Cpu = Cpu::default();
        println!("{:#?}", cpu);

        let opcode_map = OpcodeMap::new();
        let rom: Rom = Rom::new(data);

        for instruction in opcode_map.fetch_instructions(&rom.rom_bytes) {
            for w in instruction {
                print!("{}", format!("{:01$x}", w, 2));
            }
            println!("");
        }
    } else {
        println!("Invalid number of arguments.");
    }
}
