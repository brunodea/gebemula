use std::fmt;
use super::super::mem::mem;

#[derive(Copy, Clone, PartialEq)]
enum Flag {
    Z, N, H, C,
}

#[derive(Copy, Clone, PartialEq)]
enum Reg {
    A, F,
    B, C,
    D, E,
    H, L,
    AF, BC,
    DE, HL,
    SP, PC
}

impl Reg {
    fn pair_from_ddd(byte: u8) -> Reg {
        match byte & 0b111 {
            0b000 => Reg::B,
            0b001 => Reg::C,
            0b010 => Reg::D,
            0b011 => Reg::E,
            0b100 => Reg::H,
            0b101 => Reg::L,
            0b110 => Reg::HL,
            0b111 => Reg::A,
            _ => unreachable!(),
        }
    }
    fn pair_from_dd(byte: u8) -> Reg {
        match byte & 0b11 {
            0b00 => Reg::BC,
            0b01 => Reg::DE,
            0b10 => Reg::HL,
            0b11 => Reg::SP,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    //[A,F,B,C,D,E,H,L,SP,PC]
    regs: [u8; 12],
} 

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let regs_names = ["A", "F", "B", "C", "D", "E", "H", "L", "SP", "PC"];
        let flags = format!("[{:#01$b} ZNHC]", self.flags(), 4);
        let mut regs: String = "".to_owned();
        for (i, reg) in regs_names.iter().enumerate() {
            match i {
                8 | 10 => {
                    regs = regs + &format!("{:#01$X}", self.regs[i], 4);
                },
                9 | 11 => {
                    let rhs = format!("{:01$X}", self.regs[i], 4);
                    regs = regs + &format!("{}({}), ", rhs, reg);
                }
                _ => {
                    regs = regs + &format!("{}({}), ", format!("{:#01$x}", self.regs[i], 4), reg);
                },
            }
        }
        write!(f, "CPU Registers: {} {}", flags, regs)
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: [0; 12],
        }
    }

    fn reg_index(reg: Reg) -> usize {
        match reg {
            Reg::A | Reg::AF => 0,
            Reg::F => 1,
            Reg::B | Reg::BC => 2,
            Reg::C => 3,
            Reg::D | Reg::DE => 4,
            Reg::E => 5,
            Reg::H | Reg::HL => 6,
            Reg::L => 7,
            Reg::SP => 8,
            Reg::PC => 10,
        }
    }

    fn reg_is8(reg: Reg) -> bool {
        match reg {
            Reg::A | Reg::F |
            Reg::B | Reg::C |
            Reg::D | Reg::E |
            Reg::H | Reg::L => true,
            _ => false,
        }
    }

    fn reg_set16(&mut self, reg: Reg, value: u16) {
        let index: usize = Cpu::reg_index(reg);
        if Cpu::reg_is8(reg) {
            self.regs[index] = value as u8;
        } else {
            self.regs[index] = (value >> 8) as u8;
            self.regs[index+1] = value as u8;
        }
    }

    fn reg_set8(&mut self, reg: Reg, value: u8) {
        self.reg_set16(reg, value as u16);
    }

    fn reg16(&self, reg: Reg) -> u16 {
        let index: usize = Cpu::reg_index(reg);
        if Cpu::reg_is8(reg) {
            self.regs[index] as u16
        } else {
            (self.regs[index] as u16) << 8 | self.regs[index+1] as u16
        }
    }

    fn reg8(&self, reg: Reg) -> u8 {
        self.reg16(reg) as u8
    }

    fn flag_mask(flag: Flag) -> u8 {
        match flag {
            Flag::Z => 0b1000,
            Flag::N => 0b0100,
            Flag::H => 0b0010,
            Flag::C => 0b0001,
        }
    }
    
    fn flag_set(&mut self, set: bool, flag: Flag) {
        let mut flags: u8 = self.reg8(Reg::F);
        let mask: u8 = Cpu::flag_mask(flag);
        if set {
            flags |= mask;
        } else {
            flags &= !mask;
        }
        self.reg_set8(Reg::F, flags); 
    }

    fn flag_is_set(&self, flag: Flag) -> bool {
        let flags: u8 = self.reg8(Reg::F);
        let mask: u8 = Cpu::flag_mask(flag);

        mask & flags == mask
    }

    fn flags(&self) -> u8 {
        self.reg8(Reg::F)
    }

    fn increment_pc(&mut self) {
        let pc: u16 = self.reg16(Reg::PC);
        self.reg_set16(Reg::PC, pc+1);
    }

    fn mem_at_reg(&self, reg: Reg, memory: &mem::Memory) -> u8 {
        let addr: u16 = self.reg16(reg);
        memory.read_byte(addr)
    }

    fn mem_next(&mut self, memory: &mem::Memory) -> u8 {
        let value: u8 = self.mem_at_reg(Reg::PC, memory);
        self.increment_pc();
        value
    }

    pub fn execute_instructions(&mut self, starting_point: u16, memory: &mut mem::Memory) {
        self.reg_set16(Reg::PC, starting_point);

        loop { //TODO: ending point
            let byte: u8 = self.mem_next(memory);

            match byte {
                0x00 ... 0x3F => {
                    //select column in table
                    match byte % 0o10 {
                        0o2 => {
                            //LD (nn), A; LD A, (nn)
                            self.exec_ld_nn_a(byte, memory);
                        },
                        0o6 => {
                            //LD r,n; LD n,r
                            self.exec_ld_r_n(byte, memory);
                        },
                        _ => panic!("No instruction for opcode {:#01$x}", byte, 2),
                    }
                }
                0x40 ... 0x7F => {
                    if byte == 0x76 {
                        //TODO: HALT instruction
                    } else {
                        //LD r,r'
                        self.exec_ld_r_r(byte, memory);
                    }
                },
                0x80 ... 0xBF => {
                    //bit arithmetic
                    self.exec_bit_basic(byte, memory);
                },
                0xC0 ... 0xFF => {
                    panic!("No instruction for opcode {:#01$x}", byte, 2);
                },
                _ => panic!("No opcode defined for {:#01$X}", byte, 2),
            }
        }
    }

    /*Instructions execution codes*/

    fn exec_bit_basic(&mut self, opcode: u8, memory: &mut mem::Memory) {
        //TODO Flag stuff
        let reg_a_val: u8 = self.reg8(Reg::A);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let mut reg_val: u8 = self.reg8(reg);
        if reg == Reg::HL {
            reg_val = self.mem_at_reg(reg, memory);
        }
        let mut result = 0;
        let mut unchange_a: bool = false;

        match opcode {
            0x80 ... 0x87 => {
                //ADD
                result = reg_a_val + reg_val;
                self.flag_set(result == 0, Flag::Z);
            },
            0x88 ... 0x8F => {
                //ADC
                result = reg_a_val + reg_val;
            },
            0x90 ... 0x97 => {
                //SUB
                result = reg_a_val - reg_val;
            },
            0x98 ... 0x9F => {
                //SBC
                result = reg_a_val - reg_val;
            },
            0xA0 ... 0xA7 => {
                //AND
                result = reg_a_val & reg_val;
            },
            0xA8 ... 0xAF => {
                //XOR
                result = reg_a_val^reg_val;
            },
            0xB0 ... 0xB7 => {
                //OR
                result = reg_a_val | reg_val;
            },
            0xB8 ... 0xBF => {
                //CP
                self.flag_set(reg_a_val == reg_val, Flag::Z);
                unchange_a = true;
            },
            _ => unreachable!(),
        }
        if !unchange_a {
            self.reg_set8(Reg::A, result);
        }
    }

    fn exec_ld_nn_a(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let mut reg: Reg = Reg::pair_from_dd(opcode >> 4);
        if reg == Reg::SP {
            reg = Reg::HL;
        }
        let reg_val: u16 = self.reg16(reg);
        
        match opcode & 0b1111 {
            0x2 => {
                let addr: u16 = reg_val;
                let val: u8 = self.reg8(Reg::A);
                memory.write_byte(addr, val);
            },
            0xA => {
                let val: u8 = self.mem_at_reg(reg, memory);
                self.reg_set8(Reg::A, val);
            },
            _ => unreachable!(),
        }
        match opcode {
            0o42 | 0o52 => {
                //HL+
                self.reg_set16(Reg::HL, reg_val + 1);
            },
            0o62 | 0o72 => {
                //HL-
                self.reg_set16(Reg::HL, reg_val - 1);
            },
            _ => unreachable!(),
        }
    }

    fn exec_ld_r_n(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let reg: Reg = Reg::pair_from_ddd(opcode >> 3);
        let immediate: u8 = self.mem_next(memory);

        if reg == Reg::HL {
            let addr: u16 = self.reg16(Reg::HL);
            memory.write_byte(addr, immediate);
        } else {
            self.reg_set8(reg, immediate);
        }
    }

    fn exec_ld_r_r(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let reg_rhs: Reg = Reg::pair_from_ddd(opcode);
        let reg_lhs: Reg = Reg::pair_from_ddd(opcode >> 3);

        let rhs_val: u8 = self.reg8(reg_rhs);
        let lhs_val: u8 = self.reg8(reg_lhs);

        if reg_rhs == Reg::HL {
            let value: u8 = self.mem_at_reg(Reg::HL, memory); 
            self.reg_set8(reg_lhs, value);
        } else if reg_lhs == Reg::HL {
            let addr: u16 = self.reg16(Reg::HL);
            memory.write_byte(addr, rhs_val);
        } else {
            self.reg_set8(reg_rhs, lhs_val);
        }
    }

/*
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
            let mut displacement: i32 = opcode.params[0] as i32;
            if displacement < 0 {
                displacement = -(!displacement - 1);
            }
            let pc: u16 = self.reg16(Reg::PC);
            self.reg_set16(((pc as i32)+displacement) as u16);
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
    */
}
