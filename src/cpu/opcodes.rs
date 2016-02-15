use cpu::cpu::{Instruction, Reg};

    pub fn instr_to_human(instruction: &Instruction) -> String {
    if let Some(_) = instruction.prefix {
        //CB-prefixed instructions
        let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
        let mut r = format!("{:?}", reg);
        if reg == Reg::HL {
            r = "(HL)".to_owned();
        }
        let bit: u8 = instruction.opcode >> 3 & 0b111;
        match instruction.opcode {
            0x00 ... 0x07 => {
                format!("rlc {}", r)
            },
            0x08 ... 0x0F => {
                format!("rrc {}", r)
            },
            0x10 ... 0x17 => {
                //RL m
                format!("rl {}", r)
            },
            0x18 ... 0x1F => {
                //RR m
                format!("rr {}", r)
            },
            0x20 ... 0x27 => {
                format!("sla {}", r)
            },
            0x28 ... 0x2F => {
                //SRA n
                format!("sra {}", r)
            },
            0x30 ... 0x37 => {
                //SWAP n
                format!("swap {}", r)
            },
            0x38 ... 0x3F => {
                //SRL n
                format!("srl {}", r)
            },
            0x40 ... 0x7F => {
                //BIT b,r; BIT b,(HL)
                format!("bit {},{}", bit, r)
            },
            0x80 ... 0xBF => {
                //RES b,r; RES b,(HL)
                format!("res {},{}", bit, r)
            },
            0xC0 ... 0xFF => {
                //SET b,r; SET b,(HL)
                format!("set {},{}", bit, r)
            },
            _ => unreachable!(),
        }
    } else {
        match instruction.opcode {
            /***************************************/
            /*      Misc/Control instructions      */
            /***************************************/
            0x0 => {
                //NOP
                "nop".to_owned()
            },
            0x10 => {
                //STOP
                "stop".to_owned()
            },
            0x76 => {
                //HALT
                "halt".to_owned()
            },
            0xF3 => {
                //DI
                "di".to_owned()
            },
            0xFB => {
                //EI
                "ei".to_owned()
            },
            /**************************************/
            /*      8 bit rotations/shifts        */
            /**************************************/
            0x07 => {
                "RLCA".to_owned()
            },
            0x17 => {
                "RLA".to_owned()
            },
            0x0F => {
                "RRCA".to_owned()
            }, 
            0x1F => {
                "RRA".to_owned()
            },
            /**************************************/
            /* 8 bit load/store/move instructions */
            /**************************************/
            0x02 | 0x12 => {
                //LD (rr),A;
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("ld ({:?}),A", reg)
            },
            0x22 => {
                //LD (HL+),A
                format!("ld (HL+),A")
            },
            0x32 => {
                //LD (HL-),A
                format!("ld (HL-),A")
            },
            0x0A | 0x1A => {
                //LD A,(rr);
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("ld ({:?}),A", reg)
            },
            0x2A => {
                //LD A,(HL+);
                format!("ld A,(HL+)")
            },
            0x3A => {
                //LD A,(HL-)
                format!("ld A,(HL-)")
            },
            0x06 | 0x16 | 0x26 |
            0x0E | 0x1E | 0x2E |
            0x3E | 0x36 => {
                //LD r,n; LD (HL),n
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode >> 3);
                format!("ld {:?},{:#x}", reg, instruction.imm8.unwrap())
            },
            0x40 ... 0x6F | 0x70 ... 0x75 |
            0x77 ... 0x7F => {
                //LD r,r; LD r,(HL); LD (HL),r
                let reg_rhs: Reg = Reg::pair_from_ddd(instruction.opcode);
                let reg_lhs: Reg = Reg::pair_from_ddd(instruction.opcode >> 3);
                
                let r: String;
                let l: String;
                if reg_rhs == Reg::HL {
                    r = "(HL)".to_owned();
                } else {
                    r = format!("{:?}", reg_rhs);
                }
                if reg_lhs == Reg::HL {
                    l = "(HL)".to_owned();
                } else {
                    l = format!("{:?}", reg_lhs);
                }
                
                format!("ld {},{}", l, r)
            },
            0xE0 => {
                //LDH (n),A
                format!("ldh ({:#x}),A", instruction.imm8.unwrap())
            },
            0xF0 => {
                //LDH A,(n)
                format!("ldh A,({:#x})", instruction.imm8.unwrap())
            },
            0xE2 => {
                //LD (C),A
                format!("ld (0xff00+C), A")
            },
            0xF2 => {
                //LD A,(C)
                format!("ld A,(0xff00+C)")
            },
            0xEA => {
                //LD (nn),A
                format!("ld {:#x},A", instruction.imm16.unwrap())
            },
            0xFA => {
                //LD A,(nn)
                format!("ld A,{:#x}", instruction.imm16.unwrap())
            },
            /***************************************/
            /* 16 bit load/store/move instructions */
            /***************************************/
            0x01 | 0x11 | 0x21 | 0x31 => {
                //LD rr,nn
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("ld {:?},{:#x}", reg, instruction.imm16.unwrap())
            },
            0x08 => {
                //LD (nn), SP
                format!("ld {:#x},SP", instruction.imm16.unwrap())
            },
            0xC1 | 0xD1 | 0xE1 => {
                //POP rr
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("pop {:?}", reg)
            },
            0xC5 | 0xD5 | 0xE5 => {
                //PUSH rr
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("push {:?}", reg)
            },
            0xF8 => {
                //LD HL,SP+n
                format!("ld HL,SP+{:#x}", instruction.imm8.unwrap())
            },
            0xF9 => {
                //LD SP,HL
                format!("ld HL,SP+{:#x}", instruction.imm8.unwrap())
            },
            /*****************************************/
            /* 8 bit arithmetic/logical instructions */
            /*****************************************/
            0x80 ... 0x87 => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("add A,{}", v)
            },
            0x88 ... 0x8F => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("adc A,{}", v)
            },
            0x90 ... 0x97 => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("sub {}", v)
            },
            0x98 ... 0x9F => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("sbc A,{}", v)
            },
            0xA0 ... 0xA7 => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("and {}", v)
            },
            0xA8 ... 0xAF => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("xor {}", v)
            },
            0xB0 ... 0xB7 => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("or {}", v)
            },
            0xB8 ... 0xBF => {
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("cp {}", v)
            },
            0xC6 => {
                format!("add A,{}", instruction.imm8.unwrap())
            },
            0xD6 => {
                format!("sub {}", instruction.imm8.unwrap())
            },
            0xE6 => {
                format!("and {}", instruction.imm8.unwrap())
            },
            0xF6 => {
                format!("or {}", instruction.imm8.unwrap())
            },
            0xCE => {
                format!("adc A,{}", instruction.imm8.unwrap())
            },
            0xDE => {
                format!("sbc A,{}", instruction.imm8.unwrap())
            },
            0xEE => {
                format!("xor {}", instruction.imm8.unwrap())
            },
            0xFE => {
                format!("cp {}", instruction.imm8.unwrap())
            },
            0x04 | 0x14 | 0x24 | 0x34 |
            0x0C | 0x1C | 0x2C | 0x3C => {
                //INC r; INC (HL)
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode >> 3);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("inc {}", v)
            },
            0x05 | 0x15 | 0x25 | 0x35 |
            0x0D | 0x1D | 0x2D | 0x3D => {
                //DEC r; DEC (HL)
                let reg: Reg = Reg::pair_from_ddd(instruction.opcode >> 3);
                let mut v = format!("{:?}", reg);
                if reg == Reg::HL {
                    v = format!("(HL)");
                }
                format!("dec {}", v)
            },
            0x27 => {
                "DAA".to_owned()
            },
            0x37 => {
                "SCF".to_owned()
            },
            0x2F => {
                "CPL".to_owned()
            },
            0x3F => {
                "CCF".to_owned()
            },
            /******************************************/
            /* 16 bit arithmetic/logical instructions */
            /******************************************/
            0x03 | 0x13 | 0x23 | 0x33 => {
                //INC rr
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("inc {:?}", reg)
            },
            0x0B | 0x1B | 0x2B | 0x3B => {
                //DEC rr
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("dec {:?}", reg)
            },
            0x09 | 0x19 | 0x29 | 0x39 => {
                //ADD HL,rr
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("add HL,{:?}", reg)
            },
            0xE8 => {
                //ADD SP,n
                format!("add SP,{:#x}", instruction.imm8.unwrap())
            },
            /******************************************/
            /*            Jumps/Calls                 */
            /******************************************/
            0x18 => {
                //JR n
                format!("jr {:#x}", instruction.imm8.unwrap())
            },
            0x20 => {
                //JR NZ,r8
                format!("jr nz {:#x}", instruction.imm8.unwrap())
            },
            0x28 => {
                //JR Z,r8
                format!("jr z {:#x}", instruction.imm8.unwrap())
            },
            0x30 => {
                //JR NC,r8
                format!("jr nc {:#x}", instruction.imm8.unwrap())
            },
            0x38 => {
                //JR C,r8
                format!("jr c {:#x}", instruction.imm8.unwrap())
            },
            0xC3 => {
                //JP nn
                format!("jp {:#x}", instruction.imm16.unwrap())
            },
            0xC2 => {
                format!("jp nz {:#x}", instruction.imm16.unwrap())
            },
            0xCA => {
                format!("jp z {:#x}", instruction.imm16.unwrap())
            },
            0xD2 => {
                format!("jp nc {:#x}", instruction.imm16.unwrap())
            },
            0xDA => {
                format!("jp c {:#x}", instruction.imm16.unwrap())
            },
            0xE9 => {
                "jp (HL)".to_owned()
            },
            0xC0 => {
                "ret nz".to_owned()
            },
            0xC8 => {
                "ret z".to_owned()
            },
            0xC9 => {
                "ret".to_owned()
            },
            0xD0 => {
                "ret nc".to_owned()
            },
            0xD8 => {
                "ret c".to_owned()
            },
            0xD9 => {
                "reti".to_owned()
            },
            0xC4 => {
                format!("call nz,{:#x}", instruction.imm16.unwrap())
            },
            0xCC => {
                format!("call z,{:#x}", instruction.imm16.unwrap())
            },
            0xCD => {
                format!("call {:#x}", instruction.imm16.unwrap())
            },
            0xD4 => {
                format!("call nc,{:#x}", instruction.imm16.unwrap())
            },
            0xDC => {
                format!("call c,{:#x}", instruction.imm16.unwrap())
            },
            0xC7 | 0xCF | 0xD7 | 0xDF |
            0xE7 | 0xEF | 0xF7 | 0xFF => {
                //RST
                "rst".to_owned()
            },
            _ => panic!("Unknown instruction: {:#x}", instruction.opcode),
        }
    }
}
