use super::super::cpu::cpu::{Cpu, Reg, Flag};
use super::super::mem::mem::Memory;
use super::super::util::util;

const PC_DEFAULT: u16 = 0x100;

struct Test {
    pub cpu: Cpu,
    pub mem: Memory,
}

impl Test {
    pub fn new() -> Test {
        Test {
            cpu: Cpu::new(),
            mem: Memory::new()
        }
    }

    fn instr_run(&mut self, opcode: u8) -> u16 {
        self.cpu.reg_set16(Reg::PC, PC_DEFAULT);
        self.mem.write_byte(PC_DEFAULT, opcode);
        self.cpu.run_instruction(&mut self.mem);
        PC_DEFAULT + 1 //PC_DEFAULT + 1 (opcode)
    }

    //returns the value of the next instruction, regardless of the
    //executed instruction.
    fn instr_run8(&mut self, opcode: u8, imm: u8) -> u16 {
        self.cpu.reg_set16(Reg::PC, PC_DEFAULT);
        self.mem.write_byte(PC_DEFAULT, opcode);
        self.mem.write_byte(PC_DEFAULT + 1, imm);
        self.cpu.run_instruction(&mut self.mem);
        PC_DEFAULT + 2 //PC_DEFAULT + 1 (opcode) + 1 (8 bits immediate)
    }

    fn instr_run16(&mut self, opcode: u8, imm: u16) -> u16 {
        self.cpu.reg_set16(Reg::PC, PC_DEFAULT);
        self.mem.write_byte(PC_DEFAULT, opcode);
        self.mem.write_byte(PC_DEFAULT + 1, imm as u8);
        self.mem.write_byte(PC_DEFAULT + 2, (imm >> 8) as u8);
        self.cpu.run_instruction(&mut self.mem);
        PC_DEFAULT + 3 //PC_DEFAULT + 1 (opcode) + 2 (16 bits immediate).
    }
}

#[test]
fn instr_jump() {
    let mut test: &mut Test = &mut Test::new();

    let mut addr: u16;
    //JR
    addr = test.instr_run8(0x18, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 1));
    //JR NZ
    test.cpu.flag_set(false, Flag::Z);
    addr = test.instr_run8(0x20, 0xfb); //-5
    assert!(test.cpu.reg16(Reg::PC) == addr - util::twos_complement(0xfffb));
    //JR Z
    test.cpu.flag_set(true, Flag::Z);
    addr = test.instr_run8(0x28, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 1));
    //JR NC
    test.cpu.flag_set(false, Flag::C);
    addr = test.instr_run8(0x30, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 1));
    //JR C
    test.cpu.flag_set(true, Flag::C);
    addr = test.instr_run8(0x38, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 1));

    //JR NZ - no jump
    test.cpu.flag_set(true, Flag::Z);
    addr = test.instr_run8(0x20, 0xfb); //-5
    assert!(test.cpu.reg16(Reg::PC) == addr);
    //JR Z - no jump
    test.cpu.flag_set(false, Flag::Z);
    addr = test.instr_run8(0x28, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == addr);
    //JR NC - no jump
    test.cpu.flag_set(true, Flag::C);
    addr = test.instr_run8(0x30, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == addr);
    //JR C - no jump
    test.cpu.flag_set(false, Flag::C);
    addr = test.instr_run8(0x38, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == addr);

    /*16 bits jumps*/
    //JP
    addr = test.instr_run16(0xC3, 0x0001);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 0x0001));
    //JP NZ
    test.cpu.flag_set(false, Flag::Z);
    addr = test.instr_run16(0xC2, 0xfffb); //-5
    assert!(test.cpu.reg16(Reg::PC) == addr - util::twos_complement(0xfffb));
    //JP Z
    test.cpu.flag_set(true, Flag::Z);
    addr = test.instr_run16(0xCA, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 1));
    //JP NC
    test.cpu.flag_set(false, Flag::C);
    addr = test.instr_run16(0xD2, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 1));
    //JP C
    test.cpu.flag_set(true, Flag::C);
    addr = test.instr_run16(0xDA, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == (addr + 1));

    //JP NZ - no jump
    test.cpu.flag_set(true, Flag::Z);
    addr = test.instr_run16(0xC2, 0x00fb); //-5
    assert!(test.cpu.reg16(Reg::PC) == addr);
    //JP Z - no jump
    test.cpu.flag_set(false, Flag::Z);
    addr = test.instr_run16(0xCA, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == addr);
    //JP NC - no jump
    test.cpu.flag_set(true, Flag::C);
    addr = test.instr_run16(0xD2, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == addr);
    //JP C - no jump
    test.cpu.flag_set(false, Flag::C);
    addr = test.instr_run16(0xDA, 0x1);
    assert!(test.cpu.reg16(Reg::PC) == addr);
}

#[test]
fn instr_inc() {
    let mut test: &mut Test = &mut Test::new();
    test.cpu.regs = [0; 12];
    test.cpu.reg_set8(Reg::A, 0xFF);
    //INC A
    test.instr_run(0x3C);
    assert!(test.cpu.reg8(Reg::A) == 0x0);
    assert!(test.cpu.flag_is_set(Flag::H));
    //INC B
    test.instr_run(0x04);
    assert!(test.cpu.reg8(Reg::B) == 0x1);
    assert!(!test.cpu.flag_is_set(Flag::H));
    //INC C
    test.instr_run(0x0C);
    assert!(test.cpu.reg8(Reg::C) == 0x1);
    //INC D
    test.instr_run(0x14);
    assert!(test.cpu.reg8(Reg::D) == 0x1);
    //INC E
    test.instr_run(0x1C);
    assert!(test.cpu.reg8(Reg::E) == 0x1);
    //INC H
    test.instr_run(0x24);
    assert!(test.cpu.reg8(Reg::H) == 0x1);
    //INC L
    test.instr_run(0x2C);
    assert!(test.cpu.reg8(Reg::L) == 0x1);
    //INC (HL)
    let addr: u16 = 0x555;
    test.cpu.reg_set16(Reg::HL, addr);
    let val: u8 = test.mem.read_byte(addr);
    test.instr_run(0x34);
    assert!(test.mem.read_byte(addr) == val+1);

    assert!(!test.cpu.flag_is_set(Flag::N));
}

#[test]
fn instr_dec() {
    let mut test: &mut Test = &mut Test::new();
    test.cpu.regs = [1; 12];
    //DEC A
    test.instr_run(0x3D);
    assert!(test.cpu.reg8(Reg::A) == 0x0);
    assert!(test.cpu.flag_is_set(Flag::Z));
    //DEC B
    test.cpu.reg_set8(Reg::B, 0x0);
    test.instr_run(0x05);
    assert!(test.cpu.reg8(Reg::B) == 0xFF);
    assert!(!test.cpu.flag_is_set(Flag::H));
    //DEC C
    test.instr_run(0x0D);
    assert!(test.cpu.reg8(Reg::C) == 0x0);
    //DEC D
    test.instr_run(0x15);
    assert!(test.cpu.reg8(Reg::D) == 0x0);
    //DEC E
    test.instr_run(0x1D);
    assert!(test.cpu.reg8(Reg::E) == 0x0);
    //DEC H
    test.instr_run(0x25);
    assert!(test.cpu.reg8(Reg::H) == 0x0);
    //DEC L
    test.instr_run(0x2D);
    assert!(test.cpu.reg8(Reg::L) == 0x0);
    //DEC (HL)
    let addr: u16 = 0x555;
    test.cpu.reg_set16(Reg::HL, addr);
    test.mem.write_byte(addr, 0x9);
    test.instr_run(0x35);
    assert!(test.mem.read_byte(addr) == 0x8);

    assert!(test.cpu.flag_is_set(Flag::N));
}

#[test]
fn instr_inc_dec16() {
    let mut test: &mut Test = &mut Test::new();
    test.cpu.regs = [0; 12];
    //INC BC
    test.cpu.reg_set16(Reg::BC, 0xffff);
    test.instr_run(0x03);
    assert!(test.cpu.reg8(Reg::BC) == 0x0);
    //INC DE
    test.instr_run(0x13);
    assert!(test.cpu.reg8(Reg::DE) == 0x1);
    //INC HL
    test.instr_run(0x23);
    assert!(test.cpu.reg8(Reg::HL) == 0x1);
    //INC SP
    test.instr_run(0x33);
    assert!(test.cpu.reg8(Reg::SP) == 0x1);

    //DEC BC
    test.cpu.reg_set16(Reg::BC, 0x0);
    test.instr_run(0x0B);
    assert!(test.cpu.reg8(Reg::BC) == 0xffff);
    //DEC DE
    test.instr_run(0x1B);
    assert!(test.cpu.reg8(Reg::DE) == 0x0);
    //DEC HL
    test.instr_run(0x2B);
    assert!(test.cpu.reg8(Reg::HL) == 0x0);
    //DEC SP
    test.instr_run(0x3B);
    assert!(test.cpu.reg8(Reg::SP) == 0x0);
}

#[test]
fn instr_add_hl_ss() {
    let mut test: &mut Test = &mut Test::new();
    let z: bool = test.cpu.flag_is_set(Flag::Z);
    test.cpu.reg_set16(Reg::HL, 0x1);
    //ADD HL BC
    test.cpu.reg_set16(Reg::BC, 0xffff);
    test.instr_run(0x09);
    assert!(test.cpu.reg16(Reg::HL) == 0x0);
    assert!(test.cpu.flag_is_set(Flag::C));
    //ADD HL DE
    test.cpu.reg_set16(Reg::HL, 0x1);
    test.cpu.reg_set16(Reg::DE, 0x0fff);
    test.instr_run(0x19);
    assert!(test.cpu.reg16(Reg::HL) == 0x1000);
    assert!(test.cpu.flag_is_set(Flag::H));
    //ADD HL HL
    test.cpu.reg_set16(Reg::HL, 0x3);
    test.instr_run(0x29);
    assert!(test.cpu.reg16(Reg::HL) == 0x6);
    //ADD HL SP
    test.cpu.reg_set16(Reg::SP, 0x4);
    test.instr_run(0x39);
    assert!(test.cpu.reg16(Reg::HL) == 0xA);

    assert!(z == test.cpu.flag_is_set(Flag::Z));
    assert!(!test.cpu.flag_is_set(Flag::N));
}

#[test]
fn instr_ld_nn_a() {
    let mut test: &mut Test = &mut Test::new();

    //LD (BC), A
    test.cpu.reg_set8(Reg::A, 0x10);
    test.instr_run(0x02);
    assert!(test.mem.read_byte(test.cpu.reg16(Reg::BC)) == 0x10);

    //LD (DE), A
    test.cpu.reg_set8(Reg::A, 0x11);
    test.instr_run(0x12);
    assert!(test.mem.read_byte(test.cpu.reg16(Reg::DE)) == 0x11);

    //LD (HL+), A
    test.cpu.reg_set8(Reg::A, 0x12);
    let hl: u16 = test.cpu.reg16(Reg::HL);
    test.instr_run(0x22);
    assert!(test.mem.read_byte(hl) == 0x12);
    assert!(test.cpu.reg16(Reg::HL) == (hl + 1));

    //LD (HL-), A
    test.cpu.reg_set8(Reg::A, 0x13);
    let hl: u16 = test.cpu.reg16(Reg::HL);
    test.instr_run(0x32);
    assert!(test.mem.read_byte(hl) == 0x13);
    assert!(test.cpu.reg16(Reg::HL) == (hl - 1));
}
