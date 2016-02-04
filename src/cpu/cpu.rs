use std::fmt;
use cpu::opcode;
use super::super::mem::mem;

enum Flag {
    Z, N, H, C,
}

enum GenReg8 {
    A, F,
    B, C,
    D, E,
    H, L,
}
enum GenReg16 {
    AF, BC, 
    DE, HL, 
    SP, PC
}

impl GenReg8 {
    //only the 3bits on the right are evaluated.
    fn pair_from_ddd(byte: u8) -> GenReg8 {
        match byte & 0b111 {
            0b000 => GenReg8::B,
            0b001 => GenReg8::C,
            0b010 => GenReg8::D,
            0b011 => GenReg8::E,
            0b100 => GenReg8::H,
            0b101 => GenReg8::L,
            0b111 => GenReg8::A,
            _ => panic!("Invalid value for GenReg8 conversion. Value: {:#01$b}", byte, 4),
        }
    }
}

impl GenReg16 {
    //only the 2bits on the right are evaluated.
    fn pair_from_dd(byte: u8) -> GenReg16 {
        match byte & 0b11 {
            0b00 => GenReg16::BC,
            0b01 => GenReg16::DE,
            0b10 => GenReg16::HL,
            0b11 => GenReg16::SP,
            _ => panic!("Invalid value for GenReg16 conversion."),
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    //AF,BC,DE,HL,SP,PC
    gen_registers: Vec<u16>,
    flags: u8,
} impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let regs_names: Vec<&str> = vec!["AF", "BC", "DE", "HL", "SP", "PC"];
        let flags = format!("[{:#01$b} ZNHC]", self.flags, 4);
        let mut regs: String = "".to_string();
        let mut i = 0;
        for r in self.gen_registers.iter() {
            regs = regs + &format!("{}({}), ", format!("{:#01$x}", r, 4), regs_names[i]);
            i += 1;
        }
        write!(f, "CPU Registers: {} {}", flags, regs)
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            gen_registers: vec![0; 6],
            flags: 0b0000,
        }
    }

    fn flag_mask(flag: &Flag) -> u8 {
        match flag {
            &Flag::Z => 0b1000,
            &Flag::N => 0b0100,
            &Flag::H => 0b0010,
            &Flag::C => 0b0001,
        }
    }
    
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

    fn flag_set(&mut self, set: bool, flag: &Flag) {
        if set {
            self.flags |= Cpu::flag_mask(flag);
        } else {
            self.flags &= !Cpu::flag_mask(flag);
        }
    }

    fn flag_is_set(&self, flag: &Flag) -> bool {
        (Cpu::flag_mask(flag) & self.flags) == Cpu::flag_mask(flag)
    }

    
    fn reg8(&self, reg_name: GenReg8) -> u8 {
        let reg_value: u16 = self.gen_registers[Cpu::reg_index8(&reg_name)];
        if Cpu::is_reg8_left(&reg_name) {
            (reg_value >> 8) as u8
        } else {
            reg_value as u8
        }
    }

    fn reg16(&self, reg_name: GenReg16) -> u16 {
        self.gen_registers[Cpu::reg_index16(&reg_name)]
    }

    fn set_reg8(&mut self, value: u8, reg_name: GenReg8) {
        let reg_index = Cpu::reg_index8(&reg_name);
        let reg_value: u16 = self.gen_registers[reg_index];

        if Cpu::is_reg8_left(&reg_name) {
            self.gen_registers[reg_index] = (reg_value & 0x00ff) | ((value as u16) << 8);
        } else {
            self.gen_registers[reg_index] = (reg_value & 0xff00) | value as u16;
        }
    }
    
    fn set_reg16(&mut self, value: u16, reg_name: GenReg16) {
        let reg_index = Cpu::reg_index16(&reg_name);
        self.gen_registers[reg_index] = value;
    }

    //PC+value; returns PC's new value.
    fn increment_pc(&mut self, value: i16) -> u16 {
        let pc: u16 = self.reg16(GenReg16::PC);
        self.set_reg16(((pc as i32)+(value as i32)) as u16, GenReg16::PC);
        self.reg16(GenReg16::PC)
    }

    //false if this function didn't handle PC. It handles PC when an instruction sets its
    //value (e.g. via jumps).
    pub fn execute_instruction(&mut self, opcode: &opcode::Opcode, memory: &mut mem::Memory) -> bool {
        //TODO
        //get operands
        //perform calculations
        //store result
        //update time
        let mut no_instr: bool = false;
        let mut handle_pc: bool = false;

        if opcode.prefix == 0xCB {
            if opcode::Opcode::is_cb_bit_s(opcode.opcode) {
                self.exec_cb_bit_s(opcode);
            } else {
                no_instr = true;
            }
        } else {
            if opcode::Opcode::is_ld_dd_nn(opcode.opcode) {
                self.exec_ld_dd_nn(opcode);
            } else if opcode::Opcode::is_ld_r_n(opcode.opcode) {
                self.exec_ld_r_n(opcode);
            } else if opcode::Opcode::is_xor_r(opcode.opcode) {
                self.exec_xor_r(opcode);
            } else if opcode::Opcode::is_ldd_hl_a(opcode.opcode) {
                self.exec_ldd_hl_a(memory);
            } else if opcode::Opcode::is_jr_nz_e(opcode.opcode) {
                handle_pc = self.exec_jr_nz_e(opcode);
            } else if opcode::Opcode::is_ld_f000c_a(opcode.opcode) {
                self.exec_ld_f000c_a(memory);
            } else if opcode::Opcode::is_ld_a_f000c(opcode.opcode) {
                self.exec_ld_a_f000c(memory);
            } else if opcode::Opcode::is_di(opcode.opcode) {
                self.exec_di(); 
            } else if opcode::Opcode::is_ld_hl_r(opcode.opcode) {
                self.exec_ld_hl_r(opcode, memory);
            } else if opcode::Opcode::is_ld_r_hl(opcode.opcode) {
                self.exec_ld_r_hl(opcode, memory);
            } else if opcode::Opcode::is_ldh_n_a(opcode.opcode) {
                self.exec_ldh_n_a(opcode, memory);
            } else if opcode::Opcode::is_ldh_a_n(opcode.opcode) {
                self.exec_ldh_a_n(opcode, memory);
            } else if opcode::Opcode::is_ld_a_de(opcode.opcode) {
                self.exec_ld_a_de(memory);
            } else if opcode::Opcode::is_ld_de_a(opcode.opcode) {
                self.exec_ld_de_a(memory);
            } else if opcode::Opcode::is_call_nn(opcode.opcode) {
                handle_pc = self.exec_call_nn(opcode, memory);
            } else if opcode::Opcode::is_ld_r_r(opcode.opcode) {
                self.exec_ld_r_r(opcode);
            } else {
                no_instr = true;
            }
        }
        if no_instr {
            panic!("Can't execute instruction with opcode: {}", opcode);
        }

        handle_pc
    }

    pub fn execute_instructions(&mut self, starting_point: u16, memory: &mut mem::Memory) {
        self.set_reg16(starting_point, GenReg16::PC);
        let mut opcode: opcode::Opcode::new(0,0,0,0);
        loop { //TODO: finish point
            let mut pc: u16 = self.reg16(GenReg16::PC);
            let mut byte: u8 = memory.read_byte(pc);
            let mut n_params: u8 = 0x0;
            let mut prefix: u8 = 0x0;
            if opcode::Opcode::is_prefix(byte) {
                opcode.prefix = byte;
                prefix = byte;
                self.increment_pc(1);
                pc = self.reg16(GenReg16::PC);
                byte = memory.read_byte(pc);
            } else {
                n_params = opcode::Opcode::num_params(byte);
            }
            let mut opcode = Box::new(opcode::Opcode::new(prefix, byte, 0, n_params as usize));

            for n in 0..n_params {
                self.increment_pc(1);
                let addr = self.reg16(GenReg16::PC);
                opcode.params[n as usize] = memory.read_byte(addr);
            }

            if !self.execute_instruction(&opcode, memory) {
                self.increment_pc(1);
            }
            println!("{} {}", opcode, self);
        }
    }

    /*Instructions execution codes*/
    fn exec_cb_bit_s(&mut self, opcode: &opcode::Opcode) {
        let bit: u8 = (opcode.opcode >> 3) & 0b111;
        let reg: u8 = self.reg8(GenReg8::pair_from_ddd(opcode.opcode));
        
        self.flag_set((reg >> bit) & 0b1 == 0b0, &Flag::Z);
        self.flag_set(false, &Flag::N);
        self.flag_set(true, &Flag::H);
    }
    fn exec_ld_dd_nn(&mut self, opcode: &opcode::Opcode) {
        let rhs: u8 = opcode.params[0];
        let lhs: u8 = opcode.params[1];
        let val: u16 = ((lhs as u16) << 8) | rhs as u16;
        let reg16: GenReg16 = GenReg16::pair_from_dd(opcode.opcode >> 4);
        self.set_reg16(val, reg16);
    }
    fn exec_xor_r(&mut self, opcode: &opcode::Opcode) {
        let reg8: GenReg8 = GenReg8::pair_from_ddd(opcode.opcode);
        let res: u8 = self.reg8(GenReg8::A)^self.reg8(reg8);
        self.flags &= Cpu::flag_mask(&Flag::Z);
        if res == 0x0 {
            self.flag_set(true, &Flag::Z);
        }
        self.set_reg8(res, GenReg8::A);
    }
    fn exec_ldd_hl_a(&mut self, memory: &mut mem::Memory) {
        let val_a = self.reg8(GenReg8::A);
        let val_hl = self.reg16(GenReg16::HL);

        memory.write_byte(val_hl, val_a);
        self.set_reg16(val_hl-1, GenReg16::HL); 
    }
    fn exec_jr_nz_e(&mut self, opcode: &opcode::Opcode) -> bool {
        if self.flag_is_set(&Flag::Z) == false {
            //Two's complement
            let mut displacement: i8 = opcode.params[0] as i8;
            if displacement < 0 {
                displacement = -(!displacement - 1);
            }
            self.increment_pc(displacement as i16);
            return true
        }
        false
    }
    fn exec_ld_r_n(&mut self, opcode: &opcode::Opcode) {
        let r: GenReg8 = GenReg8::pair_from_ddd(opcode.opcode >> 3);
        let n: u8 = opcode.params[0];
        self.set_reg8(n, r);
    }
    fn exec_ld_f000c_a(&mut self, memory: &mut mem::Memory) {
        let addr: u16 = 0xFF00 + self.reg8(GenReg8::C) as u16;
        let reg: u8 = self.reg8(GenReg8::A);
        memory.write_byte(addr, reg);
    }
    fn exec_ld_a_f000c(&mut self, memory: &mem::Memory) {
        let addr: u16 = 0xFF00 + self.reg8(GenReg8::C) as u16;
        let value = memory.read_byte(addr);
        self.set_reg8(value, GenReg8::A);
    }
    fn exec_di(&mut self) {
        //TODO: disables interrupts but not emmediately. Interrupts are disabled after instruction
        //after DI is executed.
    }
    fn exec_ld_hl_r(&self, opcode: &opcode::Opcode, memory: &mut mem::Memory) {
        let r: GenReg8 = GenReg8::pair_from_ddd(opcode.opcode);
        let val_hl: u16 = self.reg16(GenReg16::HL);
        let val_r: u8 = self.reg8(r);
        memory.write_byte(val_hl, val_r);
    }
    fn exec_ld_r_hl(&mut self, opcode: &opcode::Opcode, memory: &mem::Memory) {
        let addr: u16 = self.reg16(GenReg16::HL);
        let reg: GenReg8 = GenReg8::pair_from_ddd(opcode.opcode >> 3); 
        let value: u8 = memory.read_byte(addr);
        self.set_reg8(value, reg);
    }
    fn exec_ldh_n_a(&self, opcode: &opcode::Opcode, memory: &mut mem::Memory) {
        let val_a: u8 = self.reg8(GenReg8::A);
        let addr: u16 = 0xFF00 + opcode.params[0] as u16;
        memory.write_byte(addr, val_a);
    }
    fn exec_ldh_a_n(&mut self, opcode: &opcode::Opcode, memory: &mem::Memory) {
        let addr: u16 = 0xFF00 + opcode.params[0] as u16;
        let value: u8 = memory.read_byte(addr);
        self.set_reg8(value, GenReg8::A);
    }
    fn exec_ld_a_de(&mut self, memory: &mem::Memory) {
        let addr: u16 = self.reg16(GenReg16::DE);
        let value: u8 = memory.read_byte(addr);
        self.set_reg8(value, GenReg8::A);
    }
    fn exec_ld_de_a(&self, memory: &mut mem::Memory){
        let addr: u16 = self.reg16(GenReg16::DE);
        let value: u8 = self.reg8(GenReg8::A);
        memory.write_byte(addr, value);
    }
    fn exec_call_nn(&mut self, opcode: &opcode::Opcode, memory: &mut mem::Memory) -> bool {
        let next_addr: u16 = self.reg16(GenReg16::PC) + opcode.len() as u16;
        let mut sp: u16 = self.reg16(GenReg16::SP)-1;
        //From the GB CPU Manual:
        //"The Stack Pointer automatically decrements before it puts something onto the stack."
        memory.write_byte(sp, next_addr as u8);
        sp -= 1;
        memory.write_byte(sp, (next_addr >> 8) as u8);
        self.set_reg16(sp, GenReg16::SP);

        let jump_to_addr: u16 = (opcode.params[1] as u16) << 8 | opcode.params[0] as u16;
        self.set_reg16(jump_to_addr, GenReg16::PC);

        true
    }
    fn exec_ld_r_r(&mut self, opcode: &opcode::Opcode) {
        let reg_from: GenReg8 = GenReg8::pair_from_ddd(opcode.opcode);
        let reg_to: GenReg8 = GenReg8::pair_from_ddd(opcode.opcode >> 3);
        let value: u8 = self.reg8(reg_from);
        self.set_reg8(value, reg_to);
    }
}
