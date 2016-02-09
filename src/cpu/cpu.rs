use std::fmt;
use super::super::mem::mem;
use super::super::util::util;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Flag {
    Z, N, H, C,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Reg {
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
    pub regs: [u8; 12],
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

    pub fn reg_set16(&mut self, reg: Reg, value: u16) {
        let index: usize = Cpu::reg_index(reg);
        if Cpu::reg_is8(reg) {
            self.regs[index] = value as u8;
        } else {
            self.regs[index] = (value >> 8) as u8;
            self.regs[index+1] = value as u8;
        }
    }

    pub fn reg_set8(&mut self, reg: Reg, value: u8) {
        self.reg_set16(reg, value as u16);
    }

    pub fn reg16(&self, reg: Reg) -> u16 {
        let index: usize = Cpu::reg_index(reg);
        if Cpu::reg_is8(reg) {
            self.regs[index] as u16
        } else {
            (self.regs[index] as u16) << 8 | self.regs[index+1] as u16
        }
    }

    pub fn reg8(&self, reg: Reg) -> u8 {
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

    pub fn flag_set(&mut self, set: bool, flag: Flag) {
        let mut flags: u8 = self.reg8(Reg::F);
        let mask: u8 = Cpu::flag_mask(flag);
        if set {
            flags |= mask;
        } else {
            flags &= !mask;
        }
        self.reg_set8(Reg::F, flags);
    }

    pub fn flag_is_set(&self, flag: Flag) -> bool {
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

    //TODO: make sure the order is right
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
        loop {
            self.run_instruction(memory);
        }
    }

    pub fn run_instruction(&mut self, memory: &mut mem::Memory) {
        /*if self.reg16(Reg::PC) > 0xfe {
            panic!("End of Bootstrap ROM.");
        }*/
        let byte: u8 = self.mem_next(memory);

        //instr, instruction type
        match ((byte >> 3) as u8, byte % 0o10) {
            (0o3 ... 0o7, 0o0) |
            (0o30, 0o3) | (0o30 ... 0o33, 0o2) => {
                //JR r8; JR NZ,r8; JR Z,r8; JR NC,r8; JR C,r8
                //JP r16; JP NZ,r16; JP Z,r16; JP NC,r16; JP C,r16
                self.exec_jump(byte, memory);
            },
            (0o0 ... 0o7, 0o2) => {
                //LD (nn), A; LD A, (nn)
                self.exec_ld_nn_a(byte, memory);
            },
            (0o0 ... 0o7, 0o3) => {
                //INC nn; DEC nn
                self.exec_inc_dec16(byte);
            },
            (0o0 ... 0o7, 0o4 ... 0o5) => {
                //INC n; DEC n
                self.exec_inc_dec(byte, memory);
            },
            (0o1,0o1) | (0o3,0o1) | (0o5,0o1) | (0o7,0o1) => {
                //ADD HL,ss
                self.exec_add_hl_ss(byte);
            },
            (0o0 ... 0o7, 0o6) => {
                //LD r,n; LD n,r
                self.exec_ld_r_n(byte, memory);
            },
            (0o0 ... 0o3, 0o7) => {
                //RLCA, RRCA, RLA, RRA
                self.exec_rotates_shifts(byte);
            },
            (0o16, 0o6) => {
                //TODO HALT instruction
                panic!("HALT!");
            },
            (0o10 ... 0o17,_) => {
                //LD r,r
                self.exec_ld_r_r(byte, memory);
            },
            (0o30 ... 0o33, 0o0) | (0o31,0o1) | (0o33,0o1) => {
                //RET
                self.exec_ret(byte, memory);
            },
            (0o30,0o1) | (0o32,0o1) | (0o34,0o1) | (0o36,0o1) |
            (0o30,0o5) | (0o32,0o5) | (0o34,0o5) | (0o36,0o5) => {
                //PUSH pp, POP pp
                self.exec_push_pop(byte, memory);
            },
            (0o30 ... 0o31, 0o4) | (0o31, 0o5) |
            (0o32 ... 0o33, 0o4) => {
                //CALL
                self.exec_call(byte, memory);
            },
            (0o20 ... 0o27, _) |
            (0o30 ... 0o37, 0o6) => {
                //AND,ADC,SUB,SBC,OR,XOR,CP
                self.exec_bit_alu8(byte, memory);
            },
            (0o31, 0o3) => {
                //CB-Prefixed
                self.exec_cb_prefixed(memory);
            },
            (0o0,0o1)|(0o2,0o1)|(0o4,0o1)|(0o6,0o1) |
            (0o34 ... 0o37, 0o2) |
            (0o34, 0o0) | (0o36, 0o0) |
            (0o37, 0o0 ... 0o1) => {
                //LD BC,d16; LD DE,d16; LD HL,d16; LD SP,d16
                //LD (ff00+c), A; LD A, (ff00+c);
                //LD (a16),A; LD A,(a16)
                //LDH (a8),A; LDH A,(a8),
                //LD HL, SP+r8; LD SP, HL;
                self.exec_ld_others(byte, memory);
            },
            _ => panic!("No opcode defined for {:#01$x}", byte, 2),
        }
        if cfg!(debug_assertions) {
            println!("opcode {}: {}", format!("{:#01$x}", byte, 4), self);
        }
    }

    /*Instructions execution codes*/

    fn exec_ret(&mut self, opcode: u8, memory: &mem::Memory) {
        let should_return: bool;
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
            },
            _ => unreachable!(),
        }

        if should_return {
            let addr: u16 = self.pop_sp16(memory);
            self.reg_set16(Reg::PC, addr);
        }
    }

    fn exec_rotates_shifts(&mut self, opcode: u8) {
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
    }

    fn exec_push_pop(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let reg: Reg = Reg::pair_from_dd(opcode >> 4);
        match opcode & 0b1111 {
            0x1 => {
                //POP
                let sp_val: u16 = self.pop_sp16(memory);
                self.reg_set16(reg, sp_val);
            },
            0x5 => {
                //PUSH
                let val = self.reg16(reg);
                self.push_sp16(val, memory);
            },
            _ => unreachable!(),
        }
    }

    fn exec_call(&mut self, opcode: u8, memory: &mut mem::Memory) {
        //push next instruction onto stack
        let imm1: u8 = self.mem_next(memory);
        let imm2: u8 = self.mem_next(memory);
        let immediate: u16 = ((imm2 as u16) << 8) | imm1 as u16;
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

        if should_jump {
            let pc: u16 = self.reg16(Reg::PC);
            self.push_sp16(pc, memory);
            self.reg_set16(Reg::PC, immediate);
        }
    }

    fn exec_cb_prefixed(&mut self, memory: &mut mem::Memory) {
        let opcode = self.mem_next(memory);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let mut value: u8;
        if reg == Reg::HL {
            value = memory.read_byte(self.reg16(Reg::HL));
        } else {
            value = self.reg8(reg);
        }
        let bit: u8 = opcode >> 3 & 0b111;
        let mut should_change_reg: bool = true;
        match ((opcode >> 3) as u8, opcode % 0o10) {
            (0o0, 0o0 ... 0o7) => {
                //RLC b
                let bit_7: u8 = value >> 7 & 0b1;

                value = value << 1 | value >> 7;
                self.flag_set(bit_7 == 1, Flag::C);
            },
            (0o1, 0o0 ... 0o7) => {
                //RRC m
                let bit_0: u8 = value & 0b1;

                value = value >> 1 | value << 7;
                self.flag_set(bit_0 == 1, Flag::C);
            },
            (0o2, 0o0 ... 0o7) => {
                //RL m
                let bit_7: u8 = value >> 7 & 0b1;

                value = value << 1 | self.flag_bit(Flag::C);
                self.flag_set(bit_7 == 1, Flag::C);
            },
            (0o3, 0o0 ... 0o7) => {
                //RR m
                let bit_c: u8 = self.flag_bit(Flag::C);
                let bit_0: u8 = value & 0b1;

                value = value >> 1 | (bit_c << 7);
                self.flag_set(bit_0 == 1, Flag::C);
            },
            (0o4, 0o0 ... 0o7) => {
                //SLA n
                let bit_7: u8 = (value >> 7) & 0b1;
                value = value << 1;

                self.flag_set(bit_7 == 1, Flag::C);
            },
            (0o5, 0o0 ... 0o7) => {
                //SRA n
                let bit_7: u8 = (value >> 7) & 0b1;
                let bit_0: u8 = value & 0b1;
                value = value >> 1 | (bit_7 << 7);

                self.flag_set(bit_0 == 1, Flag::C);
            },
            (0o6, 0o0 ... 0o7) => {
                //SWAP n
                value = value << 4 | value >> 4;
                self.flag_set(false, Flag::C);
            },
            (0o7, 0o0 ... 0o7) => {
                //SRL n
                let bit_0: u8 = value & 0b1;
                value = value >> 1;

                self.flag_set(bit_0 == 1, Flag::C);
            },
            (0o10 ... 0o17, 0o0 ... 0o7) => {
                //BIT b,r; BIT b,(HL)
                self.flag_set((value >> bit) & 0b1 == 0b0, Flag::Z);
                self.flag_set(false, Flag::N);
                self.flag_set(true, Flag::H);

                should_change_reg = false;
            },
            (0o20 ... 0o27, 0o0 ... 0o7) => {
                //RES b,r; RES b,(HL)
                value = value & !(1 << bit);
            },
            (0o30 ... 0o37, 0o0 ... 0o7) => {
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
    }

    fn exec_jump(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let should_jump: bool;
        match opcode {
            0x18 | 0xC3 => {
                //JR n; JP nn
                should_jump = true;
            },
            0x20 | 0xC2 => {
                //JR NZ,r8; JP NZ,nn
                should_jump = !self.flag_is_set(Flag::Z);
            },
            0x28 | 0xCA => {
                //JR Z,r8; JP Z,nn
                should_jump = self.flag_is_set(Flag::Z);
            },
            0x30 | 0xD2 => {
                //JR NC,r8; JP NC,nn
                should_jump = !self.flag_is_set(Flag::C);
            },
            0x38 | 0xDA => {
                //JR C,r8; JP C,nn
                should_jump = self.flag_is_set(Flag::C);
            },
            _ => unreachable!(),
        }

        //16 bits immediate (JP ... instruction)
        let is_imm16: bool = opcode > 0x38;
        if should_jump {
            let mut imm: u16 = util::sign_extend(self.mem_next(memory));
            if is_imm16 {
                imm = ((self.mem_next(memory) as u16) << 8) | imm;
            }

            let mut addr: u16 = self.reg16(Reg::PC);
            if util::is_neg16(imm) {
                addr = addr - util::twos_complement(imm);
            } else {
                addr = addr + imm;
            }

            self.reg_set16(Reg::PC, addr);
        } else {
            self.increment_pc();
            if is_imm16 {
                self.increment_pc();
            }
        }
    }

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
                memory.write_byte(addr + self.reg8(Reg::C) as u16, a_val);
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
                let value: u8 = memory.read_byte(self.reg8(Reg::C) as u16);
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

    fn exec_add_hl_ss(&mut self, opcode: u8) {
        let reg: Reg = Reg::pair_from_dd(opcode >> 4);
        let value: u16 = self.reg16(reg);

        let hl: u16 = self.reg16(Reg::HL);
        self.reg_set16(Reg::HL, hl.wrapping_add(value));

        self.flag_set(false, Flag::N);
        self.flag_set(util::has_carry_on_bit16(11, hl, value), Flag::H);
        self.flag_set(util::has_carry_on_bit16(15, hl, value), Flag::C);
    }

    fn exec_inc_dec16(&mut self, opcode: u8) {
        let reg: Reg = Reg::pair_from_dd(opcode >> 4);
        let mut value: u16 = self.reg16(reg);
        match opcode & 0b1111 {
            0x3 =>{
                //INC nn
                value = value.wrapping_add(1);
            },
            0xB =>{
                //DEC nn
                value = (value as i32 - 1) as u16;
            },
            _ => unreachable!(),
        }
        self.reg_set16(reg, value);
    }

    fn exec_inc_dec(&mut self, opcode: u8, memory: &mut mem::Memory) {
        let reg: Reg = Reg::pair_from_ddd(opcode >> 3);
        let mut reg_val: u8 = self.reg8(reg);
        let result: u8;
        if reg == Reg::HL {
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
    }

    fn exec_bit_alu8(&mut self, opcode: u8, memory: &mem::Memory) {
        let reg_a_val: u8 = self.reg8(Reg::A);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let value: u8;

        if opcode > 0xBF {
            value = self.mem_next(memory);
        } else if reg == Reg::HL {
            value = self.mem_at_reg(reg, memory);
        } else {
            value = self.reg8(reg);
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
        if reg == Reg::HL {
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

        if reg_rhs == Reg::HL {
            let value: u8 = self.mem_at_reg(Reg::HL, memory);
            self.reg_set8(reg_lhs, value);
        } else if reg_lhs == Reg::HL {
            let addr: u16 = self.reg16(Reg::HL);
            memory.write_byte(addr, rhs_val);
        } else {
            self.reg_set8(reg_lhs, rhs_val);
        }
    }
}
