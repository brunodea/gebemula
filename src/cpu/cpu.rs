use cpu::opcode::{OpcodeMap,Opcode};
use cpu::rom::Rom;

//should *always* have at least 1 element.
pub type Instruction = Vec<u8>;

pub enum GenReg8 {
    A, F,
    B, C,
    D, E,
    H, L,
}
pub enum GenReg16 {
    AF, BC, 
    DE, HL, 
    SP, PC
}

impl GenReg16 {
    //only the 2bits on the right are evaluated.
    fn pair_from_dd(byte: u8) -> GenReg16 {
        match byte {
            0x00 => GenReg16::BC,
            0x01 => GenReg16::DE,
            0x02 => GenReg16::HL,
            0x03 => GenReg16::SP,
            _ => panic!("Invalid value for GenReg16 convertion."),
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    //AF,BC,DE,HL,SP,PC
    gen_registers: Vec<u16>,
    opcode_map: OpcodeMap,
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
    fn is_reg8_left(reg_name: &GenReg8) -> bool {
        match reg_name {
            &GenReg8::A | 
            &GenReg8::B | 
            &GenReg8::D | 
            &GenReg8::H => true,

            &GenReg8::F | 
            &GenReg8::C | 
            &GenReg8::E | 
            &GenReg8::L => false,
        }
    }

    pub fn reg8(&self, reg_name: GenReg8) -> u8 {
        let reg_value: u16 = self.gen_registers[Cpu::reg_index8(&reg_name)];
        if Cpu::is_reg8_left(&reg_name) {
            (reg_value >> 8) as u8
        } else {
            reg_value as u8
        }
    }

    pub fn reg16(&self, reg_name: GenReg16) -> u16 {
        self.gen_registers[Cpu::reg_index16(&reg_name)]
    }

    pub fn set_reg8(&mut self, value: u8, reg_name: GenReg8) {
        let reg_index = Cpu::reg_index8(&reg_name);
        let reg_value: u16 = self.gen_registers[reg_index];

        if Cpu::is_reg8_left(&reg_name) {
            self.gen_registers[reg_index] = (reg_value & 0x00ff) | ((value as u16) << 8);
        } else {
            self.gen_registers[reg_index] = (reg_value & 0xff00) | value as u16;
        }
    }
    
    pub fn set_reg16(&mut self, value: u16, reg_name: GenReg16) {
        let reg_index = Cpu::reg_index16(&reg_name);
        self.gen_registers[reg_index] = value;
    }

    pub fn fetch_instructions(&self, bytes: &Vec<u8>) -> Vec<Instruction> {
        self.opcode_map.fetch_instructions(bytes)
    }

    pub fn execute_instruction(&mut self, instruction: &Instruction) {
        //TODO
        //get operands
        //perform calculations
        //store result
        //update time
        let opcode: u8 = instruction[0];
        if is_instruction_ld_16(opcode) {
            let rhs: u8 = instruction[1];
            let lhs: u8 = instruction[2];
            let val: u16 = ((lhs as u16) << 8) | rhs as u16;
            let reg16 = GenReg16::pair_from_dd(opcode >> 4);
            self.set_reg16(val, reg16);
        } else {
            panic!("Can't execute instruction with opcode: {:x}", opcode);
        }
    }

    pub fn execute_instructions(&mut self, instructions: &Vec<Instruction>) {
        let regs: Vec<&str> = vec!["AF", "BC", "DE", "HL", "SP", "PC"];
        println!("CPU registers, before execution of rom: {:#?}", self.gen_registers);
        for instruction in instructions { 
            self.execute_instruction(&instruction);
            print!("CPU registers (0x");
            for i in instruction.iter() {
                print!("{:01$x}", i, 2);
            }
            print!("): ");
            let mut i = 0;
            for r in self.gen_registers.iter() {
                print!("0x{}({}),", format!("{:01$x}", r, 2), regs[i]);
                i += 1;
            }
            println!("");
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            gen_registers: vec![0; 6],
            opcode_map: OpcodeMap::new(),
        }
    }
}


fn is_instruction_ld_16(opcode: u8) -> bool {
   let l4: u8 = opcode >> 4;
   let r4: u8 = opcode & 0x0F;

   r4 == 0x1 && l4 <= 0x3
}

