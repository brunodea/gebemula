use cpu;
use cpu::cpu::{Cpu, Reg, Instruction};
use cpu::timer::Timer;
use mem;
use mem::mem::Memory;
use std::io::{self, Write};

struct BreakCommand {
    break_addr: Option<u16>,
    break_reg: Option<Reg>,
    break_reg_value: u16,
    break_debug: u8,
}

impl BreakCommand {
    fn new() -> BreakCommand {
        BreakCommand {
            break_addr: None,
            break_reg: None,
            break_reg_value: 0,
            break_debug: 0,
        }
    }

    //true if should go to read loop;
    fn run(&mut self, instruction: &Instruction, cpu: &Cpu) -> bool {
        let mut go_to_read_loop: bool = false;
        if let Some(addr) = self.break_addr {
            if instruction.address == addr {
                println!("{}", instruction);
                self.break_addr = None;
                self.break_reg = None;
                self.break_reg_value = 0;
                go_to_read_loop = true;
            } else {
                Debugger::print_cpu_human(self.break_debug, instruction, cpu);
            }
        } else if let Some(reg) = self.break_reg {
            if cpu.reg16(reg) == self.break_reg_value {
                println!("{}", instruction);
                self.break_addr = None;
                self.break_reg = None;
                self.break_reg_value = 0;
                go_to_read_loop = true;
            }
        } else {
            go_to_read_loop = true;
        }

        go_to_read_loop
    }

    //true if should_run_cpu
    fn parse(&mut self, params: &[&str]) -> bool {
        if params.is_empty() {
            Debugger::display_help("Invalid number of arguments for 'break'\n");
            return false;
        }
        let mut should_run_cpu: bool = false;
        let mut has_cpu_human: bool = false;
        let mut cpu_human_param_index: usize = 1;
        if params.len() == 1 || params[1] == "cpu" || params[1] == "human" {
            if let Some(addr) = Debugger::hex_from_str(&params[0]) {
                self.break_addr = Some(addr);
                self.break_reg = None;
                self.break_reg_value = 0;
                has_cpu_human = params.len() >= 2;
                should_run_cpu = true;
                cpu_human_param_index = 1;
            }
        } else {
            should_run_cpu = true;
            let reg: Reg = match params[0] {
                "A" => Reg::A,
                "F" => Reg::F,
                "B" => Reg::B,
                "C" => Reg::C,
                "D" => Reg::D,
                "E" => Reg::E,
                "H" => Reg::H,
                "L" => Reg::L,
                "AF" => Reg::AF,
                "BC" => Reg::BC,
                "HL" => Reg::HL,
                "SP" => Reg::SP,
                "PC" => Reg::PC,
                _ => {
                    Debugger::display_help(&format!("Invalid register: {}", params[0]));
                    should_run_cpu = false;
                    Reg::A
                }
            };
            if should_run_cpu {
                if let Some(value) = Debugger::hex_from_str(&params[1]) {
                    self.break_addr = None;
                    self.break_reg = Some(reg);
                    self.break_reg_value = value as u16;
                    cpu_human_param_index = 2;
                    has_cpu_human = params.len() >= 3;
                } else {
                    Debugger::display_help(&format!("Invalid register value: {}", params[1]));
                    should_run_cpu = false;
                }

            }
        }

        if has_cpu_human {
            if let Some(value) = Debugger::cpu_human_in_params(&params[cpu_human_param_index..]) {
                self.break_debug = value;
            } else {
                //user has input some incorret value
                self.break_addr = None;
                self.break_reg = None;
                self.break_reg_value = 0;
                should_run_cpu = false;
            }
        }
        should_run_cpu
    }
}

pub struct Debugger {
    should_run_cpu: bool,
    run_debug: Option<u8>, //0b0000_0000 - bit 0: cpu, bit 1: human;
    break_command: BreakCommand,
    steps_debug: u8, //same as run_debug
    num_steps: u32,
    display_header: bool,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            should_run_cpu: false,
            run_debug: None,
            break_command: BreakCommand::new(),
            steps_debug: 0b0,
            num_steps: 0,
            display_header: true,
        }
    }

    pub fn run(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory, timer: &Timer) {
        if self.display_header {
            println!("##################################");
            println!("#     Gebemula Debug Console     #");
            println!("##################################");
            self.display_info(mem);
            println!("Type 'help' for the command list.");
            println!("----------------------------------");
            self.display_header = false;
        }
        if self.run_debug != None {
            Debugger::print_cpu_human(self.run_debug.unwrap(), instruction, cpu);
            return;
        }
        let mut go_to_loop: bool = self.break_command.run(instruction, cpu);
        if go_to_loop && self.num_steps > 0 {
            self.num_steps -= 1;
            Debugger::print_cpu_human(self.steps_debug, instruction, cpu);
            if self.num_steps == 0 &&  self.steps_debug == 0 {
                println!("{}", instruction);
            } else {
                go_to_loop = false;
            }
        };
        if go_to_loop {
            self.read_loop(instruction, cpu, mem, timer);
        }
    }
    fn display_info(&self, mem: &Memory) {
        println!("Game: {}", mem::cartridge::game_title_str(mem));
        println!("Cartridge Type: {}", mem::cartridge::cartridge_type_str(mem));
    }
    fn read_loop(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory, timer: &Timer) {
        loop {
            self.should_run_cpu = false;
            print!("gdc> "); //gdc: gebemula debugger console
            io::stdout().flush().unwrap();
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    input.pop(); //removes the '\n'.
                    self.parse(&input, instruction, cpu, mem, timer);
                },
                Err(error) => println!("error: {}", error),
            }
            if self.should_run_cpu {
                break;
            }
        }
    }

    fn print_cpu_human(mask: u8, instruction: &Instruction, cpu: &Cpu) {
        let debug_cpu: bool = mask & 0b1 == 0b1;
        let debug_human: bool = (mask >> 1) & 0b1 == 0b1;

        if debug_human {
            let v: &str = if debug_cpu { ":\n\t" } else { "\n" };
            print!("{}{}", instruction, v);
        }
        if debug_cpu {
            println!("{}", cpu);
        }
    }

    fn parse(&mut self, command: &str, instruction: &Instruction, cpu: &Cpu, mem: &Memory, timer: &Timer) {
        let aux: &mut Vec<&str> = &mut command.trim().split(" ").collect();
        let mut words: Vec<&str> = Vec::new();
        for w in aux.iter().filter(|x| *x.to_owned() != "") {
            words.push(w.trim());
        }

        if !words.is_empty() {
            match words[0] {
                "show" => {
                    Debugger::parse_show(&words[1..], cpu, mem, timer);
                    self.should_run_cpu = false;
                },
                "step" => {
                    self.parse_step(&words[1..]);
                    self.should_run_cpu = self.num_steps > 0;
                },
                "last" => {
                    println!("{}", instruction);
                    self.should_run_cpu = false;
                },
                "break" => {
                    self.should_run_cpu = self.break_command.parse(&words[1..]);
                },
                "help" => {
                    Debugger::display_help("");
                },
                "run" => {
                    self.parse_run(&words[1..]);
                },
                "info" => {
                    self.display_info(mem);
                },
                "" => {
                    //does nothing
                },
                _ => {
                    Debugger::display_help(&format!("Invalid command: {}", words[0]));
                },
            }
        }
    }

    fn display_help(error_msg: &str) {
        if error_msg != "" {
            println!("***ERROR: {}", error_msg);
        }
        println!("- show [cpu|ioregs|events|memory [<min_addr_hex> <max_addr_hex>]\n\tShow state of component.");
        println!("- step [decimal] [cpu|human]\n\tRun instruction pointed by PC and print it.\
                 \n\tIf a number is set, run step num times and print the last one.\
                 \n\tIf a number is set and cpu or human or both, then it will print all the instructions until the n'th instruction.");
        println!("- last\n\tPrint last instruction.");
        println!("- break [<0xaddr>|<reg> <0xvalue>] [cpu|human]\n\
            \tBreak when addr is hit or reg has value.\n\
            \tIf cpu, human or both are set, every instruction until the break point will be displayed.\n\
            \ttAvailable regs: A,F,B,C,D,E,H,L,AF,BC,DE,HL,SP,PC");
        println!("- run [cpu|human]\n\tDisable the debugger and run the code.\
                             \n\tIf set, information about cpu state or instruction (human friendly) or both will be printed.");
        println!("- info\n\tDisplay information about the game rom.");
        println!("- help\n\tShow this.");
        println!("Tip: when running 'run', 'step' or 'break' press 'Q' to stop it and go back to the debugger.");
    }

    fn parse_step(&mut self, parameters: &[&str]) {
        if parameters.is_empty() {
            self.num_steps = 1;
        } else if parameters.len() >= 1 {
            if let Ok(s) = parameters[0].parse::<u32>() {
                self.num_steps = s;
            } else {
                Debugger::display_help(&format!("Couldn't parse number of steps."));
                self.num_steps = 0;
                return;
            }
            if let Some(value) = Debugger::cpu_human_in_params(&parameters[1..]) {
                self.steps_debug = value;
            }
        } else {
            Debugger::display_help("Too many parameters for the command `step`.");
        }
    }

    pub fn cancel_run(&mut self) {
        self.run_debug = None;
        self.should_run_cpu = false;
        self.steps_debug = 0b0;
        self.num_steps = 0;
        self.break_command = BreakCommand::new();
    }

    fn parse_run(&mut self, parameters: &[&str]) {
        if parameters.is_empty() {
            self.run_debug = Some(0);
            self.should_run_cpu = true;
        } else if parameters.len() > 2 {
            Debugger::display_help(&format!("Invalid number of parameters for run."));
        } else if let Some(value) = Debugger::cpu_human_in_params(&parameters) {
            self.run_debug = Some(value);
            self.should_run_cpu = true;
        }
    }

    fn cpu_human_in_params(params: &[&str]) -> Option<u8> {
        let mut cpu: bool = false;
        let mut human: bool = false;
        for param in params {
            match *param {
                "cpu" => {
                    cpu = true;
                },
                "human" => {
                    human = true;
                },
                _ => {
                    Debugger::display_help(&format!("Invalid parameter for cpu|human: {}\n", param));
                    return None;
                }
            }
        }
        let mut res: u8 = 0;
        if cpu || human {
            res = if human { 0b10 } else { 0b00 };
            res = if cpu { res | 0b01 } else { res };
        }

        Some(res)
    }

    fn parse_show(parameters: &[&str], cpu: &Cpu, mem: &Memory, timer: &Timer) {
        if parameters.is_empty() {
            Debugger::display_help("Invalid number of parameters for 'show'.");
            return;
        }
        match parameters[0] {
            "cpu" => {
                println!("{}", cpu);
            },
            "ioregs" => {
                let tima: u8 = mem.read_byte(cpu::consts::TIMA_REGISTER_ADDR);
                let tma: u8 = mem.read_byte(cpu::consts::TMA_REGISTER_ADDR);
                let tac: u8 = mem.read_byte(cpu::consts::TAC_REGISTER_ADDR);
                let div: u8 = mem.read_byte(cpu::consts::DIV_REGISTER_ADDR);
                let if_: u8 = mem.read_byte(cpu::consts::IF_REGISTER_ADDR);
                let ie: u8 = mem.read_byte(cpu::consts::IE_REGISTER_ADDR);
                let ly: u8 = mem.read_byte(cpu::consts::LY_REGISTER_ADDR);
                let lcdc: u8 = mem.read_byte(cpu::consts::LCDC_REGISTER_ADDR);
                let scx: u8 = mem.read_byte(cpu::consts::SCX_REGISTER_ADDR);
                let scy: u8 = mem.read_byte(cpu::consts::SCY_REGISTER_ADDR);
                let stat: u8 = mem.read_byte(cpu::consts::STAT_REGISTER_ADDR);
                let lyc: u8 = mem.read_byte(cpu::consts::LYC_REGISTER_ADDR);

                println!("IF: {:#x} {:#b}", if_, if_);
                println!("IE: {:#x} {:#b}", ie, ie);
                println!("DIV: {:#x} {:#b}", div, div);
                println!("LY: {:#x} {:#b}", ly, ly);
                println!("LYC: {:#x} {:#b}", lyc, lyc);
                println!("LCDC: {:#x} {:#b}", lcdc, lcdc);
                println!("STAT: {:#x} {:#b}", stat, stat);
                println!("TIMA: {:#x} {:#b}", tima, tima);
                println!("TMA: {:#x} {:#b}", tma, tma);
                println!("TAC: {:#x} {:#b}", tac, tac);
                println!("SCX: {:#x} {:#b}", scx, scx);
                println!("SCY: {:#x} {:#b}", scy, scy);
            },
            "memory" => {
                Debugger::parse_show_memory(&parameters[1..], mem);
            },
            "events" => {
                println!("{}", timer.events_to_str());
            }
            _ => {
                Debugger::display_help(&format!("Invalid parameter for 'show': {}\n",parameters[0]));
            },
        }
    }

    fn parse_show_memory(parameters: &[&str], mem: &Memory) {
        if parameters.len() != 2 {
            Debugger::display_help(&format!("Invalid number of arguments for 'show memory'\n"));
        } else {
            let min_addr = Debugger::hex_from_str(parameters[0]);
            let max_addr = Debugger::hex_from_str(parameters[1]);
            if min_addr != None && max_addr != None {
                println!("{}", mem.format(min_addr, max_addr));
            }
        }
    }

    fn hex_from_str(mut str_hex: &str) -> Option<u16> {
        if str_hex.len() > 2 && str_hex[..2].to_owned() == "0x" {
            str_hex = &str_hex[2..];
        }
        match u16::from_str_radix(str_hex, 16) {
            Ok(value) => {
                Some(value)
            },
            Err(value) => {
                Debugger::display_help(&format!("Address is not a valid hex number: {}\n", value));
                None
            },
        }
    }
}

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
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                //POP rr
                let reg: Reg = Reg::pair_from_dd(instruction.opcode >> 4);
                format!("pop {:?}", reg)
            },
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
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
                format!("ld SP,HL")
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
                format!("add A,{:#x}", instruction.imm8.unwrap())
            },
            0xD6 => {
                format!("sub {:#x}", instruction.imm8.unwrap())
            },
            0xE6 => {
                format!("and {:#x}", instruction.imm8.unwrap())
            },
            0xF6 => {
                format!("or {:#x}", instruction.imm8.unwrap())
            },
            0xCE => {
                format!("adc A,{:#x}", instruction.imm8.unwrap())
            },
            0xDE => {
                format!("sbc A,{:#x}", instruction.imm8.unwrap())
            },
            0xEE => {
                format!("xor {:#x}", instruction.imm8.unwrap())
            },
            0xFE => {
                format!("cp {:#x}", instruction.imm8.unwrap())
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
                format!("add HL,{:?}", reg) },
            0xE8 => {
                //ADD SP,n
                format!("add SP,{:#x}", instruction.imm8.unwrap())
            },
            /*****************************************/
            /*            Jumps/Calls                */
            /*****************************************/
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
