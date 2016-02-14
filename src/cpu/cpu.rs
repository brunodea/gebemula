use std::fmt;
use super::super::mem::mem;
use super::super::util::util;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Flag {
    Z, N, H, C,
}

#[derive(Copy, Clone, PartialEq, Debug)]
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
        let regs_names = ["AF", "BC", "DE", "HL", "SP", "PC"];
        let flags = format!("[{:#01$b} ZNHC]", self.flags() >> 4, 6);
        let mut regs: String = "".to_owned();

        let mut i: usize = 0;
        while i < 12 {
            let value: u16 = (self.regs[i] as u16) << 8 | self.regs[i+1] as u16;
            let value_fmt = format!("{:#01$x}", value, 6);
            regs = regs + &format!("{}({}) ", value_fmt, regs_names[i/2]);

            i += 2;
        }
        write!(f, "{} {}", flags, regs)
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

    pub fn reset_registers(&mut self) {
        self.regs = [0; 12];
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
        if !Cpu::reg_is8(reg) {
            panic!("Trying to get 8 bits from 16-bit register: {:?}", reg)
        }
        let index: usize = Cpu::reg_index(reg);
        self.regs[index]
    }

    fn flag_mask(flag: Flag) -> u8 {
        match flag {
            Flag::Z => 0b1000_0000,
            Flag::N => 0b0100_0000,
            Flag::H => 0b0010_0000,
            Flag::C => 0b0001_0000,
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
        self.flag_bit(flag) == 0b1
    }

    fn flag_bit(&self, flag: Flag) -> u8 {
        let m: u8;
        match flag {
            Flag::Z =>  {
                m = 7;
            },
            Flag::N => {
                m = 6;
            },
            Flag::H => {
                m = 5;
            },
            Flag::C => {
                m = 4;
            },
        }
        (self.flags() >> m) & 0b1
    }

    fn flags(&self) -> u8 {
        self.reg8(Reg::F)
    }

    fn push_sp8(&mut self, value: u8, memory: &mut mem::Memory) {
        let sp: u16 = self.reg16(Reg::SP) - 1; //sp auto-decrements when pushing (it goes down in the memory)
        memory.write_byte(sp, value);
        self.reg_set16(Reg::SP, sp);
    }

    fn push_sp16(&mut self, value: u16, memory: &mut mem::Memory) {
        self.push_sp8((value >> 8) as u8, memory);
        self.push_sp8(value as u8, memory);
    }

    fn pop_sp8(&mut self, memory: &mem::Memory) -> u8 {
        let sp: u16 = self.reg16(Reg::SP);
        self.reg_set16(Reg::SP, sp + 1);
        memory.read_byte(sp)
    }

    fn pop_sp16(&mut self, memory: &mem::Memory) -> u16 {
        let lo: u8 = self.pop_sp8(memory);
        let hi: u8 = self.pop_sp8(memory);
        ((hi as u16) << 8) | lo as u16
    }

    fn increment_reg(&mut self, reg: Reg) {
        if Cpu::reg_is8(reg) {
            let val: u8 = self.reg8(reg).wrapping_add(1);
            self.reg_set8(reg, val);
        } else {
            let val: u16 = self.reg16(reg).wrapping_add(1);
            self.reg_set16(reg, val);
        }
    }
    fn decrement_reg(&mut self, reg: Reg) {
        if Cpu::reg_is8(reg) {
            let val: u8 = self.reg8(reg);
            self.reg_set8(reg, val - 1);
        } else {
            let val: u16 = self.reg16(reg);
            self.reg_set16(reg, val - 1);
        }
    }

    fn mem_at_reg(&self, reg: Reg, memory: &mem::Memory) -> u8 {
        let addr: u16 = self.reg16(reg);
        memory.read_byte(addr)
    }

    fn mem_next8(&mut self, memory: &mem::Memory) -> u8 {
        let value: u8 = self.mem_at_reg(Reg::PC, memory);
        self.increment_reg(Reg::PC);
        value
    }

    //next 2 bytes.
    fn mem_next16(&mut self, memory: &mem::Memory) -> u16 {
        let n1: u16 = self.mem_next8(memory) as u16;
        let n2: u16 = self.mem_next8(memory) as u16;

        (n2 << 8) | n1
    }

    pub fn run_instruction(&mut self, memory: &mut mem::Memory) -> u16 {
        //if self.reg16(Reg::PC) > 0xfe {
        //println!("{}", memory);
        //panic!("End of Bootstrap ROM.");
        // }
        let byte: u8 = self.mem_next8(memory);
        let mut cycles: u16 = 0;
        //instr, instruction type
        match byte {
            /***************************************/
            /*      Misc/Control instructions      */ 
            /***************************************/
            0x0 => {
                //NOP
                cycles = 4;
            },
            0x10 => {
                //STOP
            },
            0x76 => {
                //HALT
            },
            0xF3 => {
                //DI
            },
            0xFB => {
                //EI
            },
            0xCB => {
                //CB-prefixed
                cycles = self.exec_cb_prefixed(memory);
            },
            /**************************************/
            /*      8 bit rotations/shifts        */
            /**************************************/
            0x07 | 0x17 | 0x0F | 0x1F => {
                //RLCA; RLA; RRCA; RRA
                cycles = self.exec_rotates_shifts(byte);
            },
            /**************************************/
            /* 8 bit load/store/move instructions */
            /**************************************/
            0x02 | 0x12 => {
                //LD (rr),A;
                cycles = self.exec_ld_nn_a(byte, memory);
            },
            0x22 => {
                //LD (HL+),A
                cycles = self.exec_ld_nn_a(byte, memory);
                self.increment_reg(Reg::HL);
            },
            0x32 => {
                //LD (HL-),A
                cycles = self.exec_ld_nn_a(byte, memory);
                self.decrement_reg(Reg::HL);
            },
            0x0A | 0x1A => {
                //LD A,(rr);
                cycles = self.exec_ld_a_nn(byte, memory);
            },
            0x2A => {
                //LD A,(HL+);
                cycles = self.exec_ld_a_nn(byte, memory);
                self.increment_reg(Reg::HL);
            },
            0x3A => {
                //LD A,(HL-)
                cycles = self.exec_ld_a_nn(byte, memory);
                self.decrement_reg(Reg::HL);
            },
            0x06 | 0x16 | 0x26 |
            0x0E | 0x1E | 0x2E |
            0x3E | 0x36 => {
                //LD r,n; LD (HL),n
                cycles = self.exec_ld_r_n(byte, memory);
            },
            0x40 ... 0x6F | 0x70 ... 0x75 |
            0x77 ... 0x7F => {
                //LD r,r; LD r,(HL); LD (HL),r
                cycles = self.exec_ld_r_r(byte, memory);
            },
            0xE0 => {
                //LDH (n),A
                let immediate: u16 = self.mem_next8(memory) as u16;
                memory.write_byte(0xFF00+immediate,
                                  self.reg8(Reg::A));
                cycles = 12;
            },
            0xF0 => {
                //LDH A,(n)
                let immediate: u16 = self.mem_next8(memory) as u16;
                let value: u8 = memory.read_byte(0xFF00+immediate);
                self.reg_set8(Reg::A, value);
                cycles = 12;
            },
            0xE2 => {
                //LD (C),A
                memory.write_byte(0xFF00 + self.reg8(Reg::C) as u16, 
                                  self.reg8(Reg::A));
                cycles = 8;
            },
            0xF2 => {
                //LD A,(C)
                let value: u8 = memory.read_byte(self.reg8(Reg::C) as u16);
                self.reg_set8(Reg::A, value);
                cycles = 8;
            },
            0xEA => {
                //LD (nn),A
                let val: u16 = self.mem_next16(memory);
                memory.write_byte(val, self.reg8(Reg::A));
                cycles = 16;
            },
            0xFA => {
                //LD A,(nn)
                let addr: u16 = self.mem_next16(memory);
                let val: u8 = memory.read_byte(addr);
                self.reg_set8(Reg::A, val);
                cycles = 16;
            },
            /***************************************/
            /* 16 bit load/store/move instructions */
            /***************************************/
            0x01 | 0x11 | 0x21 | 0x31 => {
                //LD rr,nn
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                let val: u16 = self.mem_next16(memory);
                self.reg_set16(reg, val);
                cycles = 12;
            },
            0x08 => {
                //LD (nn), SP
                let addr: u16 = self.mem_next16(memory);
                let val: u16 = self.reg16(Reg::SP);
                memory.write_byte(addr, val as u8);
                memory.write_byte(addr+1, (val >> 8) as u8);
                cycles = 20;
            },
            0xC1 | 0xD1 | 0xE1 => {
                //POP rr
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                let sp_val: u16 = self.pop_sp16(memory);
                self.reg_set16(reg, sp_val);
                cycles = 12;
            },
            0xC5 | 0xD5 | 0xE5 => {
                //PUSH rr
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                let val = self.reg16(reg);
                self.push_sp16(val, memory);
                cycles = 16
            },
            0xF8 => {
                //LD HL,SP+n
                let immediate: u16 = self.mem_next8(memory) as u16;
                let sp: u16 = self.reg16(Reg::SP);
                self.reg_set16(Reg::HL, immediate+sp);
                cycles = 12;
            },
            0xF9 => {
                //LD SP,HL
                let hl: u16 = self.reg16(Reg::HL);
                self.push_sp16(hl, memory);
                cycles = 8;
            },
            /*****************************************/
            /* 8 bit arithmetic/logical instructions */
            /*****************************************/
            0x80 ... 0xBF |
            0xC6 | 0xD6 | 0xE6 | 0xF6 |
            0xCE | 0xDE | 0xEE | 0xFE => {
                //ADD A,r; ADD A,(HL)
                //ADC A,r; ADC A,(HL)
                //SUB r; SUB (HL); SBC A,r; SBC A,(HL)
                //AND r; AND (HL)
                //XOR r; XOR (HL)
                //ADD A,n; ADC A,n; SUB n; SBC A,n; AND n; XOR n; OR n; CP n;
                cycles = self.exec_bit_alu8(byte, memory);
            },
            0x04 | 0x14 | 0x24 | 0x34 |
            0x0C | 0x1C | 0x2C | 0x3C |
            0x05 | 0x15 | 0x25 | 0x35 |
            0x0D | 0x1D | 0x2D | 0x3D => {
                //INC r; INC (HL)
                //DEC r; DEC (HL)
                cycles = self.exec_inc_dec(byte, memory);
            },
            0x27 => {
                //DAA
            },
            0x37 => {
                //SCF
            },
            0x2F => {
                //CPL
            },
            0x3F => {
                //CCF
            },
            /******************************************/
            /* 16 bit arithmetic/logical instructions */
            /******************************************/
            0x03 | 0x13 | 0x23 | 0x33 => {
                //INC rr
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                self.increment_reg(reg);
                cycles = 8;
            },
            0x0B | 0x1B | 0x2B | 0x3B => {
                //DEC rr
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                self.decrement_reg(reg);
                cycles = 8;
            },
            0x09 | 0x19 | 0x29 | 0x39 => {
                //ADD HL,rr
                self.exec_add_hl_rr(byte);
            },
            0xE8 => {
                //ADD SP,n
                let imm: u16 = self.mem_next8(memory) as u16;
                let val: u16 = self.reg16(Reg::SP);
                self.reg_set16(Reg::SP, val + imm);
                self.flag_set(false, Flag::Z);
                self.flag_set(false, Flag::N);
                //TODO: make sure flags are right
                self.flag_set(util::has_carry_on_bit16(7, val, imm), Flag::H);
                self.flag_set(util::has_carry_on_bit16(15, val, imm), Flag::C);
                cycles = 16;
            },
            /******************************************/
            /*            Jumps/Calls                 */
            /******************************************/
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                //JR n; JR c,n
                cycles = self.exec_jr(byte, memory);
            },
            0xC2 | 0xC3 | 0xCA |
            0xD2 | 0xDA | 0xE9 => {
                //JP nn; JP c,nn; JP (HL)
                cycles = self.exec_jp(byte, memory);
            },
            0xC0 | 0xC8 | 0xC9 |
            0xD0 | 0xD8 | 0xD9 => {
                //RET; RET c; RETI
                cycles = self.exec_ret(byte, memory);
            },
            0xC4 | 0xCC | 0xCD |
            0xD4 | 0xDC => {
                //CALL nn; CALL c,nn
                cycles = self.exec_call(byte, memory);
            },
            0xC7 | 0xCF | 0xD7 | 0xDF |
            0xE7 | 0xEF | 0xF7 | 0xFF => {
                //RST
            },
            _ => panic!("Unknown instruction: {:#x}", byte),
        }
        if cycles == 0 {
            panic!("Unknown instruction: {:#x}", byte);
        }
        if cfg!(debug_assertions) {
            println!("opcode {}: {}", format!("{:#01$x}", byte, 4), self);
        }

        cycles
    }

    /*Instructions execution codes*/

    fn exec_ret(&mut self, opcode: u8, memory: &mem::Memory) -> u16 {
        let should_return: bool;
        let mut cycles: u16 = 20;
        match opcode {
            0xC0 => {
                //RET NZ
                should_return = !self.flag_is_set(Flag::Z);
            },
            0xC8 => {
                //RET Z
                should_return = self.flag_is_set(Flag::Z);
            },
            0xC9 => {
                //RET
                should_return = true;
                cycles = 16;
            },
            0xD0 => {
                //RET NC
                should_return = !self.flag_is_set(Flag::C);
            },
            0xD8 => {
                //RET C
                should_return = self.flag_is_set(Flag::C);
            },
            0xD9 => {
                //TODO: enable interrupts
                //RETI
                should_return = true;
                cycles = 16;
            },
            _ => unreachable!(),
        }

        if should_return {
            let addr: u16 = self.pop_sp16(memory);
            self.reg_set16(Reg::PC, addr);
        } else {
            cycles = 8;
        }
        cycles
    }

    fn exec_rotates_shifts(&mut self, opcode: u8) -> u16 {
        let mut value: u8 = self.reg8(Reg::A);

        let bit_7: u8 = (value >> 7) & 0b1;
        let bit_0: u8 = value & 0b1;
        let bit: u8;
        match opcode {
            0x07 => {
                //RLCA
                value = value << 1 | bit_7;
                bit = bit_7;
            },
            0x0F => {
                //RRCA
                value = value >> 1 | (bit_0 << 7);
                bit = bit_0;
            },
            0x17 => {
                //RLA
                value = value << 1 | self.flag_bit(Flag::C);
                bit = bit_7;
            },
            0x1F => {
                //RRA
                value = value >> 1 | (self.flag_bit(Flag::C) << 7);
                bit = bit_0
            },
            _ => unreachable!(),
        }

        self.reg_set8(Reg::A, value);

        self.flag_set(bit == 1, Flag::C);
        self.flag_set(value == 0, Flag::Z);
        self.flag_set(false, Flag::N);
        self.flag_set(false, Flag::H);

        4
    }

    fn exec_call(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        //push next instruction onto stack
        let immediate: u16 = self.mem_next16(memory);
        let should_jump: bool;
        match opcode {
            0xC4 => {
                //CALL NZ,a16
                should_jump = !self.flag_is_set(Flag::Z);
            },
            0xCC => {
                //CALL Z,a16
                should_jump = self.flag_is_set(Flag::Z);
            },
            0xCD => {
                //CALL a16
                should_jump = true;
            },
            0xD4 => {
                //CALL NC,a16
                should_jump = !self.flag_is_set(Flag::C);
            },
            0xDC => {
                //CALL C,a16
                should_jump = self.flag_is_set(Flag::C);
            },
            _ => unreachable!(),
        }

        let mut cycles: u16 = 12;
        if should_jump {
            let pc: u16 = self.reg16(Reg::PC);
            self.push_sp16(pc, memory);
            self.reg_set16(Reg::PC, immediate);
            cycles = 24;
        }
        cycles
    }

    fn exec_cb_prefixed(&mut self, memory: &mut mem::Memory) -> u16 {
        let opcode = self.mem_next8(memory);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let mut value: u8;
        if reg == Reg::HL {
            value = memory.read_byte(self.reg16(Reg::HL));
            } else {
            value = self.reg8(reg);
        }
        let bit: u8 = opcode >> 3 & 0b111;
        let mut should_change_reg: bool = true;

        let mut cycles: u16 = if reg == Reg::HL { 16 } else { 8 };
        match opcode {
            0x00 ... 0x07 => {
                //RLC b
                let bit_7: u8 = value >> 7 & 0b1;

                value = value << 1 | value >> 7;
                self.flag_set(bit_7 == 1, Flag::C);
            },
            0x08 ... 0x0F => {
                //RRC m
                let bit_0: u8 = value & 0b1;

                value = value >> 1 | value << 7;
                self.flag_set(bit_0 == 1, Flag::C);
            },
            0x10 ... 0x17 => {
                //RL m
                let bit_7: u8 = value >> 7 & 0b1;

                value = value << 1 | self.flag_bit(Flag::C);
                self.flag_set(bit_7 == 1, Flag::C);
            },
            0x18 ... 0x1F => {
                //RR m
                let bit_c: u8 = self.flag_bit(Flag::C);
                let bit_0: u8 = value & 0b1;

                value = value >> 1 | (bit_c << 7);
                self.flag_set(bit_0 == 1, Flag::C);
            },
            0x20 ... 0x27 => {
                //SLA n
                let bit_7: u8 = (value >> 7) & 0b1;
                value = value << 1;

                self.flag_set(bit_7 == 1, Flag::C);
            },
            0x28 ... 0x2F => {
                //SRA n
                let bit_7: u8 = (value >> 7) & 0b1;
                let bit_0: u8 = value & 0b1;
                value = value >> 1 | (bit_7 << 7);

                self.flag_set(bit_0 == 1, Flag::C);
            },
            0x30 ... 0x37 => {
                //SWAP n
                value = value << 4 | value >> 4;
                self.flag_set(false, Flag::C);
            },
            0x38 ... 0x3F => {
                //SRL n
                let bit_0: u8 = value & 0b1;
                value = value >> 1;

                self.flag_set(bit_0 == 1, Flag::C);
            },
            0x40 ... 0x7F => {
                //BIT b,r; BIT b,(HL)
                self.flag_set((value >> bit) & 0b1 == 0b0, Flag::Z);
                self.flag_set(false, Flag::N);
                self.flag_set(true, Flag::H);

                should_change_reg = false;
                if reg == Reg::HL {
                    cycles = 12;
                }
            },
            0x80 ... 0xBF => {
                //RES b,r; RES b,(HL)
                value = value & !(1 << bit);
            },
            0xC0 ... 0xFF => {
                //SET b,r; SET b,(HL)
                value = value | 1 << bit;
            },
            _ => panic!("CB-prefixed opcode not yet implemented: {:#01$x}", opcode, 2),
        }

        if should_change_reg {
            if reg == Reg::HL {
                memory.write_byte(self.reg16(Reg::HL), value);
            } else {
                self.reg_set8(reg, value);
            }
        }

        if opcode <= 0o77 {
            self.flag_set(value == 0, Flag::Z);
            self.flag_set(false, Flag::N);
            self.flag_set(false, Flag::H);
        }

        cycles
    }

    fn exec_jp(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        let should_jump: bool;
        let mut jump_to_hl: bool = false;
        match opcode {
            0xC3 => {
                //JP nn
                should_jump = true;
            },
            0xC2 => {
                //JP NZ,nn
                should_jump = !self.flag_is_set(Flag::Z);
            },
            0xCA => {
                //JP Z,nn
                should_jump = self.flag_is_set(Flag::Z);
            },
            0xD2 => {
                //JP NC,nn
                should_jump = !self.flag_is_set(Flag::C);
            },
            0xDA => {
                //JP C,nn
                should_jump = self.flag_is_set(Flag::C);
            },
            0xE9 => {
                //JP (HL)
                should_jump = true;
                jump_to_hl = true;
            },
            _ => unreachable!(),
        }

        let cycles: u16;
        if should_jump {
            cycles = 16;
            let val: u16;
            if jump_to_hl {
                val = self.reg16(Reg::HL);
            } else {
                val = self.mem_next16(memory);
            }
            self.reg_set16(Reg::PC, val);
        } else {
            self.increment_reg(Reg::PC);
            self.increment_reg(Reg::PC);
            cycles = 12;
        }

        cycles
    }

    fn exec_jr(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        let should_jump: bool;
        match opcode {
            0x18 => {
                //JR n
                should_jump = true;
            },
            0x20 => {
                //JR NZ,r8
                should_jump = !self.flag_is_set(Flag::Z);
            },
            0x28 => {
                //JR Z,r8
                should_jump = self.flag_is_set(Flag::Z);
            },
            0x30 => {
                //JR NC,r8
                should_jump = !self.flag_is_set(Flag::C);
            },
            0x38 => {
                //JR C,r8
                should_jump = self.flag_is_set(Flag::C);
            },
            _ => unreachable!(),
        }

        let cycles: u16;
        if should_jump {
            let imm: u16 = util::sign_extend(self.mem_next8(memory));
            cycles = 12;

            let mut addr: u16 = self.reg16(Reg::PC);
            if util::is_neg16(imm) {
                addr = addr - util::twos_complement(imm);
            } else {
                addr = addr + imm;
            }

            self.reg_set16(Reg::PC, addr);
        } else {
            self.increment_reg(Reg::PC);
            cycles = 8;
        }

        cycles
    }

    fn exec_add_hl_rr(&mut self, opcode: u8) -> u16 {
        let reg: Reg = Reg::pair_from_dd(opcode >> 4);
        let value: u16 = self.reg16(reg);

        let hl: u16 = self.reg16(Reg::HL);
        self.reg_set16(Reg::HL, hl.wrapping_add(value));

        self.flag_set(false, Flag::N);
        self.flag_set(util::has_carry_on_bit16(11, hl, value), Flag::H);
        self.flag_set(util::has_carry_on_bit16(15, hl, value), Flag::C);

        8
    }

    fn exec_inc_dec(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        let reg: Reg = Reg::pair_from_ddd(opcode >> 3);
        let mut reg_val: u8 = self.reg8(reg);
        let result: u8;
        let mut cycles: u16 = 4;
        if reg == Reg::HL {
            cycles = 12;
            reg_val = self.mem_at_reg(Reg::HL, memory);
        }
        match ((opcode >> 3) as u8, opcode % 0o10) {
            (0o0 ... 0o7, 0o4) => {
                //INC
                result = reg_val.wrapping_add(1);
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_carry_on_bit(3, reg_val, 1), Flag::H);
            },
            (0o0 ... 0o7, 0o5) => {
                //DEC
                result = (reg_val as i16 - 1) as u8;
                self.flag_set(true, Flag::N);
                let minus_one: i8 = -1;
                self.flag_set(!util::has_borrow_on_bit(4, reg_val, minus_one as u8), Flag::H);
            },
            _ => unreachable!(),
        }
        self.flag_set(result == 0, Flag::Z);

        if reg == Reg::HL {
            memory.write_byte(self.reg16(Reg::HL), result);
        } else {
            self.reg_set8(reg, result);
        }

        cycles
    }

    fn exec_bit_alu8(&mut self, opcode: u8, memory: &mem::Memory) -> u16 {
        let reg_a_val: u8 = self.reg8(Reg::A);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let value: u8;

        let mut cycles: u16 = 8;
        if opcode > 0xBF {
            value = self.mem_next8(memory);
        } else if reg == Reg::HL {
            value = self.mem_at_reg(reg, memory);
        } else {
            value = self.reg8(reg);
            cycles = 4;
        }
        let mut result: u8;
        let mut unchange_a: bool = false;

        match ((opcode >> 3) as u8, opcode % 0o10) {
            (0o20, 0o0 ... 0o7) | (0o30, 0o6) => {
                //ADD
                result = reg_a_val.wrapping_add(value);
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_carry_on_bit(3, reg_a_val, value), Flag::H);
                self.flag_set(util::has_carry_on_bit(7, reg_a_val, value), Flag::C);
            },
            (0o21, 0o0 ... 0o7) | (0o31, 0o6) => {
                //ADC
                result = reg_a_val.wrapping_add(value);
                if self.flag_is_set(Flag::C) {
                    result |= 0b1;
                }
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_carry_on_bit(3, reg_a_val, value), Flag::H);
                self.flag_set(util::has_carry_on_bit(7, reg_a_val, value), Flag::C);
            },
            (0o22, 0o0 ... 0o7) | (0o32, 0o6) => {
                //SUB
                result = (reg_a_val as i16 - value as i16) as u8;
                self.flag_set(true, Flag::N);
                self.flag_set(!util::has_borrow_on_bit(4, reg_a_val, value), Flag::H);
                self.flag_set(!util::has_borrow_on_any(reg_a_val, value), Flag::C);
            },
            (0o23, 0o0 ... 0o7) | (0o33, 0o6) => {
                //SBC
                result = (reg_a_val as i16 - value as i16) as u8;
                self.flag_set(true, Flag::N);
                self.flag_set(!util::has_borrow_on_bit(4, reg_a_val, value), Flag::H);
                self.flag_set(!util::has_borrow_on_any(reg_a_val, value), Flag::C);
            },
            (0o24, 0o0 ... 0o7) | (0o34, 0o6) => {
                //AND
                result = reg_a_val & value;
                self.flag_set(false, Flag::N);
                self.flag_set(true, Flag::H);
                self.flag_set(false, Flag::C);
            },
            (0o25, 0o0 ... 0o7) | (0o35, 0o6) => {
                //XOR
                result = reg_a_val^value;
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(false, Flag::C);
            },
            (0o26, 0o0 ... 0o7) | (0o36, 0o6)  => {
                //OR
                result = reg_a_val | value;
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(false, Flag::C);
            },
            (0o27, 0o0 ... 0o7) | (0o37, 0o6) => {
                //CP
                result = if reg_a_val == value { 0x0 } else { 0x1 };
                self.flag_set(true, Flag::N);
                self.flag_set(!util::has_borrow_on_bit(4, reg_a_val, value), Flag::H);
                self.flag_set(!util::has_borrow_on_any(reg_a_val, value), Flag::C);
                unchange_a = true;
            },

            _ => unreachable!(),
        }
        self.flag_set(result == 0, Flag::Z);
        if !unchange_a {
            self.reg_set8(Reg::A, result);
        }

        cycles
    }
    
    fn exec_ld_a_nn(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        let mut reg: Reg = Reg::pair_from_dd(opcode >> 4);
        if reg == Reg::SP {
            reg = Reg::HL;
        }
        let val: u8 = self.mem_at_reg(reg, memory);
        self.reg_set8(Reg::A, val);
        8
    }

    fn exec_ld_nn_a(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        let mut reg: Reg = Reg::pair_from_dd(opcode >> 4);
        if reg == Reg::SP {
            reg = Reg::HL;
        }
        let addr: u16 = self.reg16(reg);
        let val: u8 = self.reg8(Reg::A);
        memory.write_byte(addr, val);
        8
    }

    fn exec_ld_r_n(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        let reg: Reg = Reg::pair_from_ddd(opcode >> 3);
        let immediate: u8 = self.mem_next8(memory);

        let cycles: u16;
        if reg == Reg::HL {
            //LD (HL),n
            let addr: u16 = self.reg16(Reg::HL);
            memory.write_byte(addr, immediate);
            cycles = 12;
        } else {
            //LD r,n
            self.reg_set8(reg, immediate);
            cycles = 8
        }
        cycles
    }

    fn exec_ld_r_r(&mut self, opcode: u8, memory: &mut mem::Memory) -> u16 {
        let reg_rhs: Reg = Reg::pair_from_ddd(opcode);
        let reg_lhs: Reg = Reg::pair_from_ddd(opcode >> 3);

        let rhs_val: u8 = self.reg8(reg_rhs);

        let cycles: u16;
        if reg_rhs == Reg::HL {
            let value: u8 = self.mem_at_reg(Reg::HL, memory);
            self.reg_set8(reg_lhs, value);
            cycles = 8;
        } else if reg_lhs == Reg::HL {
            let addr: u16 = self.reg16(Reg::HL);
            memory.write_byte(addr, rhs_val);
            cycles = 8;
        } else {
            self.reg_set8(reg_lhs, rhs_val);
            cycles = 4;
        }
        cycles
    }
}
