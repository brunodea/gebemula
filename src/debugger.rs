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
                go_to_read_loop = true;
            } else {
                Debugger::print_cpu_human(self.break_debug, instruction, cpu);
            }
        } else if let Some(reg) = self.break_reg {
            if cpu.reg16(reg) == self.break_reg_value {
                println!("{}", instruction);
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
            print!("gdc> "); //gbm: gebemula
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
                },
                "step" => {
                    self.parse_step(&words[1..]);
                    self.should_run_cpu = self.num_steps > 0;
                },
                "last" => {
                    println!("{}", instruction);
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
            \tIf cpu, human or both are set, every instruction until the break point will be displayed.\
            \tAvailable regs: A,F,B,C,D,E,H,L,AF,BC,DE,HL,SP,PC");
        println!("- run [cpu|human]\n\tDisable the debugger and run the code.\
                             \n\tIf set, information about cpu state or instruction (human friendly) or both will be printed.");
        println!("- info\n\tDisplay information about the game rom.");
        println!("- help\n\tShow this.");
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

                println!("IF: {:#x} {:#b}", if_, if_);
                println!("IE: {:#x} {:#b}", ie, ie);
                println!("DIV: {:#x} {:#b}", div, div);
                println!("LY: {:#x} {:#b}", ly, ly);
                println!("LCDC: {:#x} {:#b}", lcdc, lcdc);
                println!("TIMA: {:#x} {:#b}", tima, tima);
                println!("TMA: {:#x} {:#b}", tma, tma);
                println!("TAC: {:#x} {:#b}", tac, tac);
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
