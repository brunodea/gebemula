use std::fmt;
use super::super::mem::mem;
use super::super::util::util;

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

    fn push_sp8(&mut self, value: u8, memory: &mut mem::Memory) {
        let sp: u16 = self.reg16(Reg::SP) - 1; //sp auto-decrements when pushing (it goes down in the memory)
        memory.write_byte(sp, value);
        self.reg_set16(Reg::SP, sp);
    }

    //TODO: make sure the order is right
    fn push_sp16(&mut self, value: u16, memory: &mut mem::Memory) {
        self.push_sp8(value as u8, memory);
        self.push_sp8((value >> 8) as u8, memory);
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

            //instr, instruction type
            match ((byte >> 3) as u8, byte % 0o10) {
                (0 ... 7, 2) => {
                    //LD (nn), A; LD A, (nn)
                    self.exec_ld_nn_a(byte, memory);
                },
                (0 ... 7, 4 ... 5) => {
                    //INC; DEC
                    self.exec_inc_dec(byte, memory);
                },
                (0 ... 7, 6) => {
                    //LD r,n; LD n,r
                    self.exec_ld_r_n(byte, memory);
                },
                (16, 6) => {
                    //TODO HALT instruction
                },
                (10 ... 17,_) => {
                    self.exec_ld_r_r(byte, memory);
                },
                (20 ... 27,_)     |
                (30 ... 37, 6) => {
                    //AND,ADC,SUB,SBC,OR,XOR,CP
                    self.exec_bit_alu8(byte, memory);
                },
                (0,1)|(1,1)|(2,1)|(3,1)|
                (34 ... 37, 2) |
                (34, 0) | (36, 0) |
                (37, 0 ... 1) => {
                    //LD (ff00+c), A; LD A, (ff00+c); LDH (a8),A; LDH A,(a8),
                    //LD HL, SP+r8; LD SP, HL; LD (a16),A; LD A,(a16)
                    self.exec_ld_others(byte, memory);
                },
                _ => panic!("No opcode defined for {:#01$X}", byte, 2),
            }
        }
    }

    /*Instructions execution codes*/
    
    fn exec_ld_others(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let addr: u16 = 0xFF00;
        let a_val: u8 = self.reg8(Reg::A);
        match opcode {
            0x01 | 0x11 | 0x21 | 0x31 => {
                //TODO: make sure byte order is correct
                let imm1: u8 = self.mem_next(memory);
                let imm2: u8 = self.mem_next(memory);
                let value: u16 = (imm2 as u16) << 8 | imm1 as u16;
                let reg: Reg = Reg::pair_from_dd(opcode >> 4);
                self.reg_set16(reg, value);
            },
            0xE0 => {
                //LDH (a8), A
                let immediate: u16 = self.mem_next(memory) as u16;

                memory.write_byte(addr+immediate, a_val);
            },
            0xE2 => {
                //LD (C), A
                memory.write_byte(addr+self.reg8(Reg::C) as u16, a_val);
            },
            0xEA => {
                //LD (a16),A
                //TODO: make sure byte order is correct
                let imm1: u8 = self.mem_next(memory);
                let imm2: u8 = self.mem_next(memory);
                let addr: u16 = (imm2 as u16) << 8 | imm1 as u16;

                memory.write_byte(addr, a_val);
            },
            0xF0 => {
                //LDH A, (a8)
                let immediate: u16 = self.mem_next(memory) as u16;
                let value: u8 = memory.read_byte(addr+immediate);
                self.reg_set8(Reg::A, value);
            },
            0xF2 => {
                //LD A,(C)
                let value: u8 = memory.read_byte(self.reg16(Reg::C));
                self.reg_set8(Reg::A, value);
            },
            0xF8 => {
                //LD HL,SP+r8
                let immediate: u16 = self.mem_next(memory) as u16;
                let sp: u16 = self.reg16(Reg::SP);
                self.reg_set16(Reg::HL, immediate+sp);
            },
            0xF9 => {
                //LD SP,HL
                let hl: u16 = self.reg16(Reg::HL);
                self.push_sp16(hl, memory);
                self.flag_set(false, Flag::Z);
                self.flag_set(false, Flag::N);
                //TODO: H and C: "set or reset according to operation"
            },
            0xFA => {
                //LD A, (a16)
                let imm1: u8 = self.mem_next(memory);
                let imm2: u8 = self.mem_next(memory);
                let addr: u16 = (imm2 as u16) << 8 | imm1 as u16;

                self.reg_set8(Reg::A, memory.read_byte(addr));
            },
            _ => panic!("Invalid opcode for ld others: {:#X}", opcode),
        }
    }

    fn exec_inc_dec(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let reg: Reg = Reg::pair_from_ddd(opcode >> 3);
        let reg_val: u8 = self.reg8(reg);
        let mut result: u8 = 0;
        if reg == Reg::HL {
            result = self.mem_at_reg(Reg::HL, memory) + 1;
            memory.write_byte(self.reg16(Reg::HL), result);
        } else {
            match ((opcode >> 3) as u8, opcode % 0o10) {
                (0 ... 7, 4) => {
                    //INC
                    result = reg_val+1;
                    self.flag_set(false, Flag::N);
                    //TODO: set H if carry from bit 3: ????
                    //self.flag_set(util::has_carry_on_bit(3, 
                },
                (0 ... 7, 5) => {
                    //DEC
                    result = reg_val-1;    
                    self.flag_set(true, Flag::N);
                    //TODO: set H if no borrow from bit 4
                },
                _ => panic!("Invalid opcode for inc/dec: {:#X}", opcode),
            }
        }
        self.flag_set(result == 0, Flag::Z);
    }

    fn exec_bit_alu8(&mut self, opcode: u8, memory: &mem::Memory) {
        //TODO Flag stuff
        let reg_a_val: u8 = self.reg8(Reg::A);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let mut value: u8 = 0;
        
        if opcode > 0xBF {
            value = self.mem_next(memory);
        } else if reg == Reg::HL {
            value = self.mem_at_reg(reg, memory);
        } else {
            value = self.reg8(reg);
        }
        let mut result = 0;
        let mut unchange_a: bool = false;

        self.flag_set(result == 0, Flag::Z);
        match ((opcode >> 3) as u8, opcode % 0o10) {
            (20, 0 ... 7) | (30, 6) => {
                //ADD
                result = reg_a_val + value;
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_carry_on_bit(3, reg_a_val, value), Flag::H);
                self.flag_set(util::has_carry_on_bit(7, reg_a_val, value), Flag::C);
            },
            (21, 0 ... 7) | (31, 6) => {
                //ADC
                result = reg_a_val + value;
                if self.flag_is_set(Flag::C) { 
                    result |= 0b1;
                }
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_carry_on_bit(3, reg_a_val, value), Flag::H);
                self.flag_set(util::has_carry_on_bit(7, reg_a_val, value), Flag::C);
            },
            (22, 0 ... 7) | (32, 6) => {
                //SUB
                result = reg_a_val - value;
                self.flag_set(true, Flag::N);
                self.flag_set(!util::has_borrow_on_bit(4, reg_a_val, value), Flag::H);
                self.flag_set(!util::has_borrow_on_any(reg_a_val, value), Flag::C);
            },
            (23, 0 ... 7) | (33, 6) => {
                //SBC
                result = reg_a_val - value;
                self.flag_set(true, Flag::N);
                self.flag_set(!util::has_borrow_on_bit(4, reg_a_val, value), Flag::H);
                self.flag_set(!util::has_borrow_on_any(reg_a_val, value), Flag::C);
            },
            (24, 0 ... 7) | (34, 6) => {
                //AND
                result = reg_a_val & value;
                self.flag_set(false, Flag::N);
                self.flag_set(true, Flag::H);
                self.flag_set(false, Flag::C);
            },
            (25, 0 ... 7) | (35, 6) => {
                //XOR
                result = reg_a_val^value;
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(false, Flag::C);
            },
            (26, 0 ... 7) | (36, 6)  => {
                //OR
                result = reg_a_val | value;
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(false, Flag::C);
            },
            (27, 0 ... 7) | (37, 6) => {
                //CP
                self.flag_set(opcode == 0xFE && reg_a_val == value, Flag::Z);
                self.flag_set(true, Flag::N);
                self.flag_set(!util::has_borrow_on_bit(4, reg_a_val, value), Flag::H);
                let c: bool = opcode == 0xFE && reg_a_val < value;
                self.flag_set(c | !util::has_borrow_on_any(reg_a_val, value), Flag::C);
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
}
