use std::fmt;
use super::super::mem::mem;
use super::super::util::util;
use super::super::debugger;
use cpu::{ioregister, interrupt, consts};
use super::super::timeline::{Event, EventType};

#[derive(Copy, Clone, PartialEq, Debug)]
enum Flag {
    Z,
    N,
    H,
    C,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Reg {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl Reg {
    #[inline]
    pub fn pair_from_ddd(byte: u8) -> Reg {
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

    #[inline]
    pub fn pair_from_dd(byte: u8) -> Reg {
        match byte & 0b11 {
            0b00 => Reg::BC,
            0b01 => Reg::DE,
            0b10 => Reg::HL,
            0b11 => Reg::SP,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction {
    pub prefix: Option<u8>,
    pub opcode: u8,
    pub imm8: Option<u8>,
    pub imm16: Option<u16>,
    pub address: u16,
    pub cycles: u32,
}

impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            prefix: None,
            opcode: 0x0,
            imm8: None,
            imm16: None,
            address: 0x0,
            cycles: 0,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = match self.prefix {
            Some(val) => format!("{:#x}", val),
            None => "".to_owned(),
        };
        let imm8 = match self.imm8 {
            Some(val) => format!("{:#x}", val),
            None => "".to_owned(),
        };
        let imm16 = match self.imm16 {
            Some(val) => format!("{:#01$x}", val, 6),
            None => "".to_owned(),
        };
        let mut opcode = format!("{:#x}", self.opcode);
        if prefix != "" {
            opcode = format!("{}{:x}", prefix, self.opcode);
        }
        let addr = format!("{:#01$x}", self.address, 6);
        if imm8 == "" && imm16 == "" {
            write!(f,
                   "{}: {} - ({})",
                   addr,
                   debugger::instr_to_human(&self),
                   opcode)
        } else {
            write!(f,
                   "{}: {} - ({} {}{})",
                   addr,
                   debugger::instr_to_human(&self),
                   opcode,
                   imm8,
                   imm16)
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    // [A,F,B,C,D,E,H,L,SP,PC]
    regs: [u8; 12],
    ime_flag: bool, // interrupt master enable flag
    halt_flag: bool, // cpu doesn't run until an interrupt occurs.
    last_instruction: Option<Instruction>,
    disable_interrupts: bool,
    enable_interrupts: bool,
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let regs_names = ["AF", "BC", "DE", "HL", "SP", "PC"];
        let flags = format!("[{:#01$b} ZNHC]", self.flags() >> 4, 6);
        let mut regs: String = "".to_owned();

        let mut i: usize = 0;
        while i < 12 {
            let value: u16 = (self.regs[i] as u16) << 8 | self.regs[i + 1] as u16;
            let value_fmt = format!("{:#01$x}", value, 6);
            regs = regs + &format!("{}({}) ", value_fmt, regs_names[i / 2]);

            i += 2;
        }
        write!(f, "{} {}", flags, regs)
    }
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            regs: [0; 12],
            ime_flag: true,
            halt_flag: false,
            last_instruction: None,
            disable_interrupts: false,
            enable_interrupts: false,
        }
    }
}

impl Cpu {
    #[inline]
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

    #[inline]
    fn reg_is8(reg: Reg) -> bool {
        match reg {
            Reg::A |
            Reg::F |
            Reg::B |
            Reg::C |
            Reg::D |
            Reg::E |
            Reg::H |
            Reg::L => true,
            _ => false,
        }
    }

    pub fn restart(&mut self) {
        self.regs = [0; 12];
        self.ime_flag = true;
        self.halt_flag = false;
        self.last_instruction = None;
        self.disable_interrupts = false;
        self.enable_interrupts = false;
    }

    #[inline]
    fn reg_set16(&mut self, reg: Reg, value: u16) {
        let index: usize = Cpu::reg_index(reg);
        if Cpu::reg_is8(reg) {
            self.regs[index] = value as u8;
        } else {
            self.regs[index] = (value >> 8) as u8;
            self.regs[index + 1] = value as u8;
        }
    }

    #[inline]
    fn reg_set8(&mut self, reg: Reg, value: u8) {
        self.reg_set16(reg, value as u16);
    }

    #[inline]
    pub fn reg16(&self, reg: Reg) -> u16 {
        let index: usize = Cpu::reg_index(reg);
        if Cpu::reg_is8(reg) {
            self.regs[index] as u16
        } else {
            ((self.regs[index] as u16) << 8) | self.regs[index + 1] as u16
        }
    }

    #[inline]
    fn reg8(&self, reg: Reg) -> u8 {
        if !Cpu::reg_is8(reg) {
            panic!("Trying to get 8 bits from 16-bit register: {:?}", reg)
        }
        let index: usize = Cpu::reg_index(reg);
        self.regs[index]
    }

    #[inline]
    fn flag_mask(flag: Flag) -> u8 {
        match flag {
            Flag::Z => 0b1000_0000,
            Flag::N => 0b0100_0000,
            Flag::H => 0b0010_0000,
            Flag::C => 0b0001_0000,
        }
    }

    #[inline]
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

    #[inline]
    fn flag_is_set(&self, flag: Flag) -> bool {
        self.flag_bit(flag) == 0b1
    }

    #[inline]
    fn flag_bit(&self, flag: Flag) -> u8 {
        let m: u8;
        match flag {
            Flag::Z => {
                m = 7;
            }
            Flag::N => {
                m = 6;
            }
            Flag::H => {
                m = 5;
            }
            Flag::C => {
                m = 4;
            }
        }
        (self.flags() >> m) & 0b1
    }

    #[inline]
    fn flags(&self) -> u8 {
        self.reg8(Reg::F)
    }

    #[inline]
    fn push_sp8(&mut self, value: u8, memory: &mut mem::Memory) {
        let sp: u16 = self.reg16(Reg::SP) - 1;
        self.mem_write(sp, value, memory);
        self.reg_set16(Reg::SP, sp);
    }

    #[inline]
    fn push_sp16(&mut self, value: u16, memory: &mut mem::Memory) {
        self.push_sp8((value >> 8) as u8, memory);
        self.push_sp8(value as u8, memory);
    }

    #[inline]
    fn pop_sp8(&mut self, memory: &mem::Memory) -> u8 {
        let sp: u16 = self.reg16(Reg::SP);
        self.reg_set16(Reg::SP, sp + 1);
        memory.read_byte(sp)
    }

    #[inline]
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
            let val: u8 = self.reg8(reg).wrapping_sub(1);
            self.reg_set8(reg, val);
        } else {
            let val: u16 = self.reg16(reg).wrapping_sub(1);
            self.reg_set16(reg, val);
        }
    }

    #[inline]
    fn mem_at_reg(&self, reg: Reg, memory: &mem::Memory) -> u8 {
        let addr: u16 = self.reg16(reg);
        memory.read_byte(addr)
    }

    #[inline]
    fn mem_next8(&mut self, memory: &mem::Memory) -> u8 {
        let value: u8 = self.mem_at_reg(Reg::PC, memory);
        self.increment_reg(Reg::PC);
        value
    }

    // next 2 bytes.
    #[inline]
    fn mem_next16(&mut self, memory: &mem::Memory) -> u16 {
        let n1: u16 = self.mem_next8(memory) as u16;
        let n2: u16 = self.mem_next8(memory) as u16;

        (n2 << 8) | n1
    }

    // function for having control of memory writes
    #[inline]
    fn mem_write(&self, address: u16, value: u8, memory: &mut mem::Memory) {
        let value: u8 = match address {
            consts::DIV_REGISTER_ADDR | consts::LY_REGISTER_ADDR => 0,
            _ => value,
        };
        memory.write_byte(address, value);
    }

    pub fn handle_interrupts(&mut self, memory: &mut mem::Memory) {
        if self.ime_flag {
            if let Some(interrupt) = interrupt::next_request(memory) {
                self.halt_flag = false;
                self.ime_flag = false;
                let pc: u16 = self.reg16(Reg::PC);
                self.push_sp16(pc, memory);
                self.reg_set16(Reg::PC, interrupt::address(interrupt));
                interrupt::remove_request(interrupt, memory);
                // since the interrupt request is removed and interrupts are disabled,
                // simply returning to the main loop seems correct.
            }
        }
    }

    pub fn run_instruction(&mut self, memory: &mut mem::Memory) -> (Instruction, Option<Event>) {

        if self.halt_flag {
            return (self.last_instruction.unwrap(), None);
        }

        // Actually performs DI and EI at the right time.
        // The order of these if's *has* to be like this.
        if self.disable_interrupts {
            self.ime_flag = false;
            self.disable_interrupts = false;
        } else if self.enable_interrupts {
            self.ime_flag = true;
            self.enable_interrupts = false;
        }
        if let Some(ref last_instr) = self.last_instruction {
            match last_instr.opcode {
                0xF3 => {
                    // DI
                    self.disable_interrupts = true;
                }
                0xFB => {
                    // EI
                    self.enable_interrupts = true;
                }
                _ => (),
            }
        }
        // *********************************************

        let mut event: Option<Event> = None;
        let addr: u16 = self.reg16(Reg::PC);
        let byte: u8 = self.mem_next8(memory);
        let mut instruction: Instruction = Instruction::default();
        instruction.opcode = byte;
        match byte {
            /***************************************/
            /*      Misc/Control instructions      */
            /***************************************/
            0x0 => {
                //NOP
                instruction.cycles = 4;
                if addr == 0x100 {
                    event = Some(Event::new(0, EventType::BootstrapFinished));
                }
            },
            0x10 => {
                //STOP
                self.halt_flag = true;
                instruction.cycles = 4;
                ioregister::LCDCRegister::disable_lcd(memory);
            },
            0x76 => {
                //HALT
                instruction.cycles = 4;
                self.halt_flag = true;
            },
            0xF3 | 0xFB => {
                //DI | EI
                instruction.cycles = 4;
            },
            0xCB => {
                //CB-prefixed
                instruction = self.exec_cb_prefixed(memory);
            },
            /**************************************/
            /*      8 bit rotations/shifts        */
            /**************************************/
            0x07 | 0x17 | 0x0F | 0x1F => {
                //RLCA; RLA; RRCA; RRA
                instruction = self.exec_rotates_shifts(byte);
            },
            /**************************************/
            /* 8 bit load/store/move instructions */
            /**************************************/
            0x02 | 0x12 => {
                //LD (rr),A;
                instruction = self.exec_ld_nn_a(byte, memory);
            },
            0x22 => {
                //LD (HL+),A
                instruction = self.exec_ld_nn_a(byte, memory);
                self.increment_reg(Reg::HL);
            },
            0x32 => {
                //LD (HL-),A
                instruction = self.exec_ld_nn_a(byte, memory);
                self.decrement_reg(Reg::HL);
            },
            0x0A | 0x1A => {
                //LD A,(rr);
                instruction = self.exec_ld_a_nn(byte, memory);
            },
            0x2A => {
                //LD A,(HL+);
                instruction = self.exec_ld_a_nn(byte, memory);
                self.increment_reg(Reg::HL);
            },
            0x3A => {
                //LD A,(HL-)
                instruction = self.exec_ld_a_nn(byte, memory);
                self.decrement_reg(Reg::HL);
            },
            0x06 | 0x16 | 0x26 |
            0x0E | 0x1E | 0x2E |
            0x3E | 0x36 => {
                //LD r,n; LD (HL),n
                let reg: Reg = Reg::pair_from_ddd(byte >> 3);
                let immediate: u8 = self.mem_next8(memory);

                let cycles: u32;
                if reg == Reg::HL {
                    // LD (HL),n
                    let addr: u16 = self.reg16(Reg::HL);
                    self.mem_write(addr, immediate, memory);
                    cycles = 12;
                } else {
                    // LD r,n
                    self.reg_set8(reg, immediate);
                    cycles = 8
                }

                instruction.cycles = cycles;
                instruction.imm8 = Some(immediate);
            },
            0x40 ... 0x75 |
            0x77 ... 0x7F => {
                //LD r,r; LD r,(HL); LD (HL),r
                let reg_rhs: Reg = Reg::pair_from_ddd(byte);
                let reg_lhs: Reg = Reg::pair_from_ddd(byte >> 3);

                let cycles: u32;
                if reg_rhs == Reg::HL {
                    let value: u8 = self.mem_at_reg(Reg::HL, memory);
                    self.reg_set8(reg_lhs, value);
                    cycles = 8;
                } else if reg_lhs == Reg::HL {
                    let addr: u16 = self.reg16(Reg::HL);
                    let rhs_val: u8 = self.reg8(reg_rhs);
                    self.mem_write(addr, rhs_val, memory);
                    cycles = 8;
                } else {
                    let rhs_val: u8 = self.reg8(reg_rhs);
                    self.reg_set8(reg_lhs, rhs_val);
                    cycles = 4;
                }

                instruction.cycles = cycles;
            },
            0xE0 => {
                //LDH (n),A
                let immediate: u16 = 0xFF00 + (self.mem_next8(memory) as u16);
                if immediate == consts::DMA_REGISTER_ADDR {
                    let mut e: Event = Event::new(
                        consts::DMA_DURATION_CYCLES,
                        EventType::DMATransfer);
                    e.additional_value = self.reg8(Reg::A);
                    event = Some(e);
                } else if immediate == consts::JOYPAD_REGISTER_ADDR {
                    let e: Event = Event::new(
                        0, EventType::JoypadPressed);
                    event = Some(e);
                }
                self.mem_write(immediate, self.reg8(Reg::A), memory);
                instruction.cycles = 12;
                instruction.imm8 = Some(immediate as u8);
            },
            0xF0 => {
                //LDH A,(n)
                let immediate: u8 = self.mem_next8(memory);
                let value: u8 = memory.read_byte(0xFF00 + (immediate as u16));
                self.reg_set8(Reg::A, value);
                instruction.cycles = 12;
                instruction.imm8 = Some(immediate);
            },
            0xE2 => {
                //LD (C),A
                let addr: u16 = 0xFF00 + (self.reg8(Reg::C) as u16);
                if addr == consts::DMA_REGISTER_ADDR {
                    let mut e: Event = Event::new(
                        consts::DMA_DURATION_CYCLES,
                        EventType::DMATransfer);
                    e.additional_value = self.reg8(Reg::A);
                    event = Some(e);
                } else if addr == consts::JOYPAD_REGISTER_ADDR {
                    let e: Event = Event::new(
                        0, EventType::JoypadPressed);
                    event = Some(e);
                }
                self.mem_write(addr, self.reg8(Reg::A), memory);
                instruction.cycles = 8
            },
            0xF2 => {
                //LD A,(C)
                let value: u8 = memory.read_byte(0xFF00 + (self.reg8(Reg::C) as u16));
                self.reg_set8(Reg::A, value);
                instruction.cycles = 8
            },
            0xEA => {
                //LD (nn),A
                let val: u16 = self.mem_next16(memory);
                self.mem_write(val, self.reg8(Reg::A), memory);
                instruction.cycles = 16;
                instruction.imm16 = Some(val);
            },
            0xFA => {
                //LD A,(nn)
                let addr: u16 = self.mem_next16(memory);
                let val: u8 = memory.read_byte(addr);
                self.reg_set8(Reg::A, val);
                instruction.cycles = 16;
                instruction.imm16 = Some(addr);
            },
            /***************************************/
            /* 16 bit load/store/move instructions */
            /***************************************/
            0x01 | 0x11 | 0x21 | 0x31 => {
                //LD rr,nn
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                let val: u16 = self.mem_next16(memory);
                self.reg_set16(reg, val);
                instruction.cycles = 12;
                instruction.imm16 = Some(val);
            },
            0x08 => {
                //LD (nn), SP
                let addr: u16 = self.mem_next16(memory);
                let val: u16 = self.reg16(Reg::SP);
                self.mem_write(addr, val as u8, memory);
                self.mem_write(addr+1, (val >> 8) as u8, memory);
                instruction.cycles = 20;
                instruction.imm16 = Some(addr);
            },
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                //POP rr
                let mut reg: Reg = Reg::pair_from_dd(byte >> 4);
                if reg == Reg::SP {
                    reg = Reg::AF;
                }
                let sp_val: u16 = self.pop_sp16(memory);
                self.reg_set16(reg, sp_val);
                instruction.cycles = 12;
            },
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                //PUSH rr
                let mut reg: Reg = Reg::pair_from_dd(byte >> 4);
                if reg == Reg::SP {
                    reg = Reg::AF;
                }
                let val: u16 = self.reg16(reg);
                self.push_sp16(val, memory);
                instruction.cycles = 16;
            },
            0xF8 => {
                //LD HL,SP+n
                let immediate: u16 = util::sign_extend(self.mem_next8(memory));
                let sp: u16 = self.reg16(Reg::SP);
                if util::is_neg16(immediate) {
                    let res: u16 = sp.wrapping_sub(util::twos_complement(immediate));
                    self.reg_set16(Reg::HL, res);
                    self.flag_set((res & 0xff) <= (sp & 0xff), Flag::C);
                    self.flag_set((res & 0xf) <= (sp & 0xf), Flag::H);
                } else {
                    let res: u16 = sp.wrapping_add(immediate);
                    self.reg_set16(Reg::HL, res);
                    self.flag_set((sp & 0xff) as u32 + immediate as u32 > 0xff, Flag::C);
                    self.flag_set((sp & 0xf) + (immediate & 0xf) > 0xf, Flag::H);
                }
                self.flag_set(false, Flag::Z);
                self.flag_set(false, Flag::N);
                instruction.cycles = 12;
                instruction.imm8 = Some(immediate as u8);
            },
            0xF9 => {
                //LD SP,HL
                let hl: u16 = self.reg16(Reg::HL);
                self.reg_set16(Reg::SP, hl);
                instruction.cycles = 8;
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
                instruction = self.exec_bit_alu8(byte, memory);
            },
            0x04 | 0x14 | 0x24 | 0x34 |
            0x0C | 0x1C | 0x2C | 0x3C |
            0x05 | 0x15 | 0x25 | 0x35 |
            0x0D | 0x1D | 0x2D | 0x3D => {
                //INC r; INC (HL)
                //DEC r; DEC (HL)
                instruction = self.exec_inc_dec(byte, memory);
            },
            0x27 => {
                //DAA
                let reg_a_val: u8 = self.reg8(Reg::A);
                let upper_nibble: u8 = reg_a_val >> 4;
                let lower_nibble: u8 = reg_a_val & 0b0000_1111;
                let c_flag: bool = self.flag_is_set(Flag::C);
                let h_flag: bool = self.flag_is_set(Flag::H);
                let n_flag: bool = self.flag_is_set(Flag::N);
                let mut as_nop: bool = false;
                //the N flag isn't strictly necessary here, so it can be removed in the future.
                let (add_value, new_c_flag) =
                    match (n_flag, c_flag, upper_nibble, h_flag, lower_nibble) {
                    (false, false, 0x0 ... 0x9, false, 0x0 ... 0x9) |
                    (true, false, 0x0 ... 0x9, false, 0x0 ... 0x9) => (0x00, false),
                    (false, false, 0x0 ... 0x8, false, 0xA ... 0xF) |
                    (false, false, 0x0 ... 0x9, true, 0x0 ... 0x3) => (0x06, false),
                    (false, false, 0xA ... 0xF, false, 0x0 ... 0x9) |
                    (false, true, 0x0 ... 0x2, false, 0x0 ... 0x9) => (0x60, true),
                    (false, false, 0x9 ... 0xF, false, 0xA ... 0xF) |
                    (false, false, 0xA ... 0xF, true, 0x0 ... 0x3) |
                    (false, true, 0x0 ... 0x2, false, 0xA ... 0xF) |
                    (false, true, 0x0 ... 0x3, true, 0x0 ... 0x3) => (0x66, true),
                    (true, false, 0x0 ... 0x8, true, 0x6 ... 0xF) => (0xFA, false),
                    (true, true, 0x7 ... 0xF, false, 0x0 ... 0x9) => (0xA0, true),
                    (true, true, 0x6 ... 0xF, true, 0x6 ... 0xF) => (0x9A, true),
                    _ => {
                        as_nop = true;
                        (0, false)
                    },
                };
                if !as_nop {
                    let res: u8 = reg_a_val.wrapping_add(add_value);
                    self.reg_set8(Reg::A, res);
                    self.flag_set(res == 0, Flag::Z);
                    self.flag_set(false, Flag::H);
                    self.flag_set(new_c_flag, Flag::C);
                }

                instruction.cycles = 4;
            },
            0x37 => {
                //SCF
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(true, Flag::C);
                instruction.cycles = 4;
            },
            0x2F => {
                //CPL
                let val: u8 = self.reg8(Reg::A);
                self.reg_set8(Reg::A, !val);
                self.flag_set(true, Flag::N);
                self.flag_set(true, Flag::H);
                instruction.cycles = 4;
            },
            0x3F => {
                //CCF
                let c: bool = self.flag_is_set(Flag::C);
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(!c, Flag::C);
                instruction.cycles = 4;
            },
            /******************************************/
            /* 16 bit arithmetic/logical instructions */
            /******************************************/
            0x03 | 0x13 | 0x23 | 0x33 => {
                //INC rr
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                self.increment_reg(reg);
                instruction.cycles = 8;
            },
            0x0B | 0x1B | 0x2B | 0x3B => {
                //DEC rr
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                self.decrement_reg(reg);
                instruction.cycles = 8;
            },
            0x09 | 0x19 | 0x29 | 0x39 => {
                //ADD HL,rr
                let reg: Reg = Reg::pair_from_dd(byte >> 4);
                let value: u16 = self.reg16(reg);

                let hl: u16 = self.reg16(Reg::HL);
                self.reg_set16(Reg::HL, hl.wrapping_add(value));

                self.flag_set(false, Flag::N);
                self.flag_set(util::has_half_carry16(hl, value), Flag::H);
                self.flag_set(util::has_carry16(hl, value), Flag::C);

                instruction.cycles = 8;
            },
            0xE8 => {
                //ADD SP,n
                let imm: u16 = util::sign_extend(self.mem_next8(memory));
                let sp: u16 = self.reg16(Reg::SP);
                if util::is_neg16(imm) {
                    let res: u16 = sp.wrapping_sub(util::twos_complement(imm));
                    self.reg_set16(Reg::SP, res);
                    self.flag_set((res & 0xff) <= (sp & 0xff), Flag::C);
                    self.flag_set((res & 0xf) <= (sp & 0xf), Flag::H);
                } else {
                    let res: u16 = sp.wrapping_add(imm);
                    self.reg_set16(Reg::SP, res);
                    self.flag_set((sp & 0xff) as u32 + imm as u32 > 0xff, Flag::C);
                    self.flag_set((sp & 0xf) + (imm & 0xf) > 0xf, Flag::H);
                }
                self.flag_set(false, Flag::Z);
                self.flag_set(false, Flag::N);
                instruction.cycles = 16;
                instruction.imm8 = Some(imm as u8);
            },
            /******************************************/
            /*            Jumps/Calls                 */
            /******************************************/
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                //JR n; JR c,n
                instruction = self.exec_jr(byte, memory);
            },
            0xC2 | 0xC3 | 0xCA |
            0xD2 | 0xDA | 0xE9 => {
                //JP nn; JP c,nn; JP (HL)
                instruction = self.exec_jp(byte, memory);
            },
            0xC0 | 0xC8 | 0xC9 |
            0xD0 | 0xD8 | 0xD9 => {
                //RET; RET c; RETI
                instruction = self.exec_ret(byte, memory);
            },
            0xC4 | 0xCC | 0xCD |
            0xD4 | 0xDC => {
                //CALL nn; CALL c,nn
                instruction = self.exec_call(byte, memory);
            },
            0xC7 | 0xCF | 0xD7 | 0xDF |
            0xE7 | 0xEF | 0xF7 | 0xFF => {
                //RST
                let pc: u16 = self.reg16(Reg::PC);
                self.push_sp16(pc, memory);
                let addr: u16 = byte as u16 & 0b0011_1000;
                self.reg_set16(Reg::PC, addr);
                instruction.cycles = 32;
            },
            _ => panic!("Unknown instruction: {:#x}", byte),
        }

        if instruction.prefix.is_none() {
            instruction.opcode = byte;
        }
        instruction.address = addr;
        self.last_instruction = Some(instruction);
        (instruction, event)
    }

    // Instructions execution codes

    fn exec_ret(&mut self, opcode: u8, memory: &mem::Memory) -> Instruction {
        let should_return: bool;
        let mut cycles: u32 = 20;
        match opcode {
            0xC0 => {
                // RET NZ
                should_return = !self.flag_is_set(Flag::Z);
            }
            0xC8 => {
                // RET Z
                should_return = self.flag_is_set(Flag::Z);
            }
            0xC9 => {
                // RET
                should_return = true;
                cycles = 16;
            }
            0xD0 => {
                // RET NC
                should_return = !self.flag_is_set(Flag::C);
            }
            0xD8 => {
                // RET C
                should_return = self.flag_is_set(Flag::C);
            }
            0xD9 => {
                // RETI
                should_return = true;
                cycles = 16;
                self.ime_flag = true;
            }
            _ => unreachable!(),
        }

        if should_return {
            let addr: u16 = self.pop_sp16(memory);
            self.reg_set16(Reg::PC, addr);
        } else {
            cycles = 8;
        }
        let mut instr: Instruction = Instruction::default();
        instr.cycles = cycles;

        instr
    }

    fn exec_rotates_shifts(&mut self, opcode: u8) -> Instruction {
        let mut value: u8 = self.reg8(Reg::A);

        let bit_7: u8 = (value >> 7) & 0b1;
        let bit_0: u8 = value & 0b1;
        let bit: u8;
        match opcode {
            0x07 => {
                // RLCA
                value = (value << 1) | bit_7;
                bit = bit_7;
            }
            0x0F => {
                // RRCA
                value = (value >> 1) | (bit_0 << 7);
                bit = bit_0;
            }
            0x17 => {
                // RLA
                value = (value << 1) | self.flag_bit(Flag::C);
                bit = bit_7;
            }
            0x1F => {
                // RRA
                value = (value >> 1) | (self.flag_bit(Flag::C) << 7);
                bit = bit_0
            }
            _ => unreachable!(),
        }

        self.reg_set8(Reg::A, value);

        self.flag_set(bit == 1, Flag::C);
        // TODO: what to believe?
        // Z80 manual says Z flag is not affected;
        // Gameboy manual says it is.
        // self.flag_set(value == 0, Flag::Z);
        self.flag_set(false, Flag::N);
        self.flag_set(false, Flag::H);

        let mut instr: Instruction = Instruction::default();
        instr.cycles = 4;

        instr
    }

    fn exec_call(&mut self, opcode: u8, memory: &mut mem::Memory) -> Instruction {
        // push next instruction onto stack
        let immediate: u16 = self.mem_next16(memory);
        let should_jump: bool;
        match opcode {
            0xC4 => {
                // CALL NZ,a16
                should_jump = !self.flag_is_set(Flag::Z);
            }
            0xCC => {
                // CALL Z,a16
                should_jump = self.flag_is_set(Flag::Z);
            }
            0xCD => {
                // CALL a16
                should_jump = true;
            }
            0xD4 => {
                // CALL NC,a16
                should_jump = !self.flag_is_set(Flag::C);
            }
            0xDC => {
                // CALL C,a16
                should_jump = self.flag_is_set(Flag::C);
            }
            _ => unreachable!(),
        }

        let mut cycles: u32 = 12;
        if should_jump {
            let pc: u16 = self.reg16(Reg::PC);
            self.push_sp16(pc, memory);
            self.reg_set16(Reg::PC, immediate);
            cycles = 24;
        }
        let mut instr: Instruction = Instruction::default();
        instr.cycles = cycles;
        instr.imm16 = Some(immediate);

        instr
    }

    fn exec_cb_prefixed(&mut self, memory: &mut mem::Memory) -> Instruction {
        let opcode = self.mem_next8(memory);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let mut value: u8;
        if reg == Reg::HL {
            value = memory.read_byte(self.reg16(Reg::HL));
        } else {
            value = self.reg8(reg);
        }
        let bit: u8 = (opcode >> 3) & 0b111;
        let mut should_change_reg: bool = true;

        let cycles: u32 = if reg == Reg::HL {
            16
        } else {
            8
        };
        match opcode {
            0x00...0x07 => {
                // RLC b
                let bit_7: u8 = (value >> 7) & 0b1;

                value = (value << 1) | bit_7;
                self.flag_set(bit_7 == 1, Flag::C);
            }
            0x08...0x0F => {
                // RRC m
                let bit_0: u8 = value & 0b1;

                value = (value >> 1) | (bit_0 << 7);
                self.flag_set(bit_0 == 1, Flag::C);
            }
            0x10...0x17 => {
                // RL m
                let bit_7: u8 = (value >> 7) & 0b1;

                value = (value << 1) | self.flag_bit(Flag::C);
                self.flag_set(bit_7 == 1, Flag::C);
            }
            0x18...0x1F => {
                // RR m
                let bit_c: u8 = self.flag_bit(Flag::C);
                let bit_0: u8 = value & 0b1;

                value = (value >> 1) | (bit_c << 7);
                self.flag_set(bit_0 == 1, Flag::C);
            }
            0x20...0x27 => {
                // SLA n
                let bit_7: u8 = (value >> 7) & 0b1;
                value = value << 1;

                self.flag_set(bit_7 == 1, Flag::C);
            }
            0x28...0x2F => {
                // SRA n
                let bit_7: u8 = value & 0b1000_0000;
                let bit_0: u8 = value & 0b1;
                value = (value >> 1) | bit_7;

                self.flag_set(bit_0 == 1, Flag::C);
            }
            0x30...0x37 => {
                // SWAP n
                value = (value << 4) | (value >> 4);
                self.flag_set(false, Flag::C);
            }
            0x38...0x3F => {
                // SRL n
                let bit_0: u8 = value & 0b1;
                value = value >> 1;

                self.flag_set(bit_0 == 1, Flag::C);
            }
            0x40...0x7F => {
                // BIT b,r; BIT b,(HL)
                self.flag_set(((value >> bit) & 0b1) == 0b0, Flag::Z);
                self.flag_set(false, Flag::N);
                self.flag_set(true, Flag::H);

                should_change_reg = false;
            }
            0x80...0xBF => {
                // RES b,r; RES b,(HL)
                value = value & !(1 << bit);
            }
            0xC0...0xFF => {
                // SET b,r; SET b,(HL)
                value = value | (1 << bit);
            }
            _ => {
                panic!("CB-prefixed opcode not yet implemented: {:#01$x}",
                       opcode,
                       2)
            }
        }

        if should_change_reg {
            if reg == Reg::HL {
                self.mem_write(self.reg16(Reg::HL), value, memory)
            } else {
                self.reg_set8(reg, value);
            }
        }

        if opcode <= 0x3F {
            self.flag_set(value == 0, Flag::Z);
            self.flag_set(false, Flag::N);
            self.flag_set(false, Flag::H);
        }

        let mut instr: Instruction = Instruction::default();
        instr.prefix = Some(0xCB);
        instr.opcode = opcode;
        instr.cycles = cycles;

        instr
    }

    fn exec_jp(&mut self, opcode: u8, memory: &mut mem::Memory) -> Instruction {
        let should_jump: bool;
        let mut jump_to_hl: bool = false;
        match opcode {
            0xC3 => {
                // JP nn
                should_jump = true;
            }
            0xC2 => {
                // JP NZ,nn
                should_jump = !self.flag_is_set(Flag::Z);
            }
            0xCA => {
                // JP Z,nn
                should_jump = self.flag_is_set(Flag::Z);
            }
            0xD2 => {
                // JP NC,nn
                should_jump = !self.flag_is_set(Flag::C);
            }
            0xDA => {
                // JP C,nn
                should_jump = self.flag_is_set(Flag::C);
            }
            0xE9 => {
                // JP (HL)
                should_jump = true;
                jump_to_hl = true;
            }
            _ => unreachable!(),
        }

        let cycles: u32;
        let mut imm16: Option<u16> = None;
        if should_jump {
            cycles = 16;
            let val: u16 = if jump_to_hl {
                self.reg16(Reg::HL)
            } else {
                let imm: u16 = self.mem_next16(memory);
                imm16 = Some(imm);
                imm
            };
            self.reg_set16(Reg::PC, val);
        } else {
            if !jump_to_hl {
                imm16 = Some(self.mem_next16(memory)); //mem_next increments PC twice.
            }
            cycles = 12;
        }

        let mut instr: Instruction = Instruction::default();
        instr.cycles = cycles;
        instr.imm16 = imm16;

        instr
    }

    fn exec_jr(&mut self, opcode: u8, memory: &mut mem::Memory) -> Instruction {
        let should_jump: bool;
        match opcode {
            0x18 => {
                // JR n
                should_jump = true;
            }
            0x20 => {
                // JR NZ,r8
                should_jump = !self.flag_is_set(Flag::Z);
            }
            0x28 => {
                // JR Z,r8
                should_jump = self.flag_is_set(Flag::Z);
            }
            0x30 => {
                // JR NC,r8
                should_jump = !self.flag_is_set(Flag::C);
            }
            0x38 => {
                // JR C,r8
                should_jump = self.flag_is_set(Flag::C);
            }
            _ => unreachable!(),
        }

        let cycles: u32;
        let imm8: u8 = self.mem_next8(memory);
        if should_jump {
            let imm: u16 = util::sign_extend(imm8);
            cycles = 12;

            let mut addr: u16 = self.reg16(Reg::PC);
            if util::is_neg16(imm) {
                addr = addr - util::twos_complement(imm);
            } else {
                addr = addr + imm;
            }

            self.reg_set16(Reg::PC, addr);
        } else {
            cycles = 8;
        }

        let mut instr: Instruction = Instruction::default();
        instr.cycles = cycles;
        instr.imm8 = Some(imm8);

        instr
    }

    fn exec_inc_dec(&mut self, opcode: u8, memory: &mut mem::Memory) -> Instruction {
        let reg: Reg = Reg::pair_from_ddd(opcode >> 3);
        let result: u8;
        let mut cycles: u32 = 4;
        let reg_val: u8;
        if reg == Reg::HL {
            cycles = 12;
            reg_val = self.mem_at_reg(Reg::HL, memory);
        } else {
            reg_val = self.reg8(reg);
        }
        match opcode {
            0x04 |
            0x14 |
            0x24 |
            0x34 |
            0x0C |
            0x1C |
            0x2C |
            0x3C => {
                // INC
                result = reg_val.wrapping_add(1);
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_half_carry(reg_val, 1), Flag::H);
            }
            0x05 |
            0x15 |
            0x25 |
            0x35 |
            0x0D |
            0x1D |
            0x2D |
            0x3D => {
                // DEC
                result = reg_val.wrapping_sub(1);
                self.flag_set(true, Flag::N);
                self.flag_set(util::has_borrow(reg_val, result), Flag::H);
            }
            _ => unreachable!(),
        }
        self.flag_set(result == 0, Flag::Z);

        if reg == Reg::HL {
            self.mem_write(self.reg16(Reg::HL), result, memory);
        } else {
            self.reg_set8(reg, result);
        }

        let mut instr: Instruction = Instruction::default();
        instr.cycles = cycles;

        instr
    }

    fn exec_bit_alu8(&mut self, opcode: u8, memory: &mem::Memory) -> Instruction {
        let reg_a_val: u8 = self.reg8(Reg::A);
        let reg: Reg = Reg::pair_from_ddd(opcode);
        let value: u8;

        let mut cycles: u32 = 8;
        let mut imm8: Option<u8> = None;
        if opcode > 0xBF {
            value = self.mem_next8(memory);
            imm8 = Some(value);
        } else if reg == Reg::HL {
            value = self.mem_at_reg(reg, memory);
        } else {
            value = self.reg8(reg);
            cycles = 4;
        }
        let result: u8;
        let mut unchange_a: bool = false;

        match opcode {
            0x80...0x87 | 0xC6 => {
                // ADD
                result = reg_a_val.wrapping_add(value);
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_half_carry(reg_a_val, value), Flag::H);
                self.flag_set(util::has_carry(reg_a_val, value), Flag::C);
            }
            0x88...0x8F | 0xCE => {
                // ADC
                let value: u8 = value.wrapping_add(self.flag_bit(Flag::C));
                result = reg_a_val.wrapping_add(value);
                self.flag_set(false, Flag::N);
                self.flag_set(util::has_half_carry(reg_a_val, value), Flag::H);
                self.flag_set(util::has_carry(reg_a_val, value), Flag::C);
            }
            0x90...0x97 | 0xD6 => {
                // SUB
                result = reg_a_val.wrapping_sub(value);
                self.flag_set(true, Flag::N);
                self.flag_set(util::has_borrow(reg_a_val, value), Flag::H);
                self.flag_set(value > reg_a_val, Flag::C);
            }
            0x98...0x9F | 0xDE => {
                // SBC
                //result = reg_a_val.wrapping_sub(value.wrapping_add(self.flag_bit(Flag::C)));
                let value: u8 = value.wrapping_add(self.flag_bit(Flag::C));
                result = reg_a_val.wrapping_sub(value);
                self.flag_set(true, Flag::N);
                self.flag_set(util::has_borrow(reg_a_val, value), Flag::H);
                self.flag_set(value > reg_a_val, Flag::C);
            }
            0xA0...0xA7 | 0xE6 => {
                // AND
                result = reg_a_val & value;
                self.flag_set(false, Flag::N);
                self.flag_set(true, Flag::H);
                self.flag_set(false, Flag::C);
            }
            0xA8...0xAF | 0xEE => {
                // XOR
                result = reg_a_val ^ value;
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(false, Flag::C);
            }
            0xB0...0xB7 | 0xF6 => {
                // OR
                result = reg_a_val | value;
                self.flag_set(false, Flag::N);
                self.flag_set(false, Flag::H);
                self.flag_set(false, Flag::C);
            }
            0xB8...0xBF | 0xFE => {
                // CP
                result = if reg_a_val == value {
                    0x0
                } else {
                    0x1
                };
                self.flag_set(true, Flag::N);
                self.flag_set(util::has_borrow(reg_a_val, value), Flag::H);
                self.flag_set(reg_a_val < value, Flag::C);
                unchange_a = true;
            }

            _ => unreachable!(),
        }
        self.flag_set(result == 0, Flag::Z);
        if !unchange_a {
            self.reg_set8(Reg::A, result);
        }

        let mut instr: Instruction = Instruction::default();
        instr.cycles = cycles;
        instr.imm8 = imm8;

        instr
    }

    fn exec_ld_a_nn(&mut self, opcode: u8, memory: &mut mem::Memory) -> Instruction {
        let mut reg: Reg = Reg::pair_from_dd(opcode >> 4);
        if reg == Reg::SP {
            reg = Reg::HL;
        }
        let val: u8 = self.mem_at_reg(reg, memory);
        self.reg_set8(Reg::A, val);

        let mut instr: Instruction = Instruction::default();
        instr.cycles = 8;

        instr
    }

    fn exec_ld_nn_a(&mut self, opcode: u8, memory: &mut mem::Memory) -> Instruction {
        let mut reg: Reg = Reg::pair_from_dd(opcode >> 4);
        if reg == Reg::SP {
            reg = Reg::HL;
        }
        let addr: u16 = self.reg16(reg);
        let val: u8 = self.reg8(Reg::A);
        self.mem_write(addr, val, memory);

        let mut instr: Instruction = Instruction::default();
        instr.cycles = 8;

        instr
    }
}
