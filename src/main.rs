use std::env;
use std::io::Read;
use std::fs::File;

enum GenReg8 {
    A, F,
    B, C,
    D, E,
    H, L,
}
enum GenReg16 {
    AF, BC, DE, HL, SP, PC
}

#[derive(Debug)]
struct Cpu {
    //AF,BC,DE,HL,SP,PC
    gen_registers: Vec<u16>,
}

impl Cpu {
    fn reg_index8(reg_name: &GenReg8) -> usize {
        match reg_name {
            &GenReg8::A | &GenReg8::F => 0,
            &GenReg8::B | &GenReg8::C => 1,
            &GenReg8::D | &GenReg8::E => 2,
            &GenReg8::H | &GenReg8::L => 3,
        }
    }
    fn reg_index16(reg_name: &GenReg16) -> usize {
        match reg_name {
            &GenReg16::AF => 0,
            &GenReg16::BC => 1,
            &GenReg16::DE => 2,
            &GenReg16::HL => 3,
            &GenReg16::SP => 4,
            &GenReg16::PC => 5,
        }
    }
    fn reg8(&self, reg_name: GenReg8) -> u8 {
        let reg_value: u16 = self.gen_registers[Cpu::reg_index8(&reg_name)];
        let left_byte: u8  = (reg_value >> 8) as u8;
        let right_byte: u8 = reg_value as u8;

        match reg_name {
            GenReg8::A | 
            GenReg8::B | 
            GenReg8::D | 
            GenReg8::H => left_byte,

            GenReg8::F | 
            GenReg8::C | 
            GenReg8::E | 
            GenReg8::L => right_byte,
        }
    }

    fn reg16(&self, reg_name: GenReg16) -> u16 {
        self.gen_registers[Cpu::reg_index16(&reg_name)]
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            gen_registers: vec![0; 6],
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        let mut boostrap_rom = File::open(&args[1]).unwrap();
        let mut data = Vec::new();
        boostrap_rom.read_to_end(&mut data).unwrap();

        let cpu: Cpu = Cpu::default();
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
