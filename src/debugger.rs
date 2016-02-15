use cpu::cpu::{Cpu, Instruction};
use cpu::timer;
use cpu::interrupt;
use mem::mem::Memory;
use std::io::{self, Write};

pub struct Debugger {
    break_addr: Option<u16>,
    should_run_cpu: bool,
    run_debug: u8, //0b0000_0000 - bit 0: cpu, bit 1: human;
    break_debug: u8, //same as run_debug
    is_step: bool,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            break_addr: None,
            should_run_cpu: false,
            run_debug: 0x00,
            is_step: false,
            break_debug: 0x00,
        }
    }

    pub fn run(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory) {
        if self.run_debug != 0x00 {
            self.print_cpu_human(self.run_debug, instruction, cpu);
            return;
        }
        if let Some(addr) = self.break_addr {
            if instruction.address >= addr { //>= because the provided address may point to an immediate, in which case == would never be true.
                println!("{}", instruction);
                self.break_addr = None;
                self.read_loop(instruction, cpu, mem);
            } else {
                self.print_cpu_human(self.break_debug, instruction, cpu);
            }
        } else {
            if self.is_step {
                println!("{}", instruction); //prints the instruction run after step.
            }
            self.read_loop(instruction, cpu, mem);
        }
    }
    fn read_loop(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory) {
        loop {
            self.should_run_cpu = false;
            self.is_step = false;
            print!("gbm> "); //gbm: gebemula
            io::stdout().flush().unwrap();
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    input.pop(); //removes the '\n'.
                    self.parse(&input, instruction, cpu, mem);
                },
                Err(error) => println!("error: {}", error),
            }
            if self.should_run_cpu {
                break;
            }
        }
    }

    fn print_cpu_human(&self, mask: u8, instruction: &Instruction, cpu: &Cpu) {
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

    fn parse(&mut self, command: &str, instruction: &Instruction, cpu: &Cpu, mem: &Memory) {
        let words: &mut Vec<&str> = &mut command.split(" ").collect();
        if !words.is_empty() {
            match words[0] {
                "show" => {
                    words.remove(0);
                    Debugger::parse_show(words, cpu, mem);
                },
                "step" => {
                    self.is_step = true;
                    self.should_run_cpu = true;
                },
                "last" => {
                    println!("{}", instruction);
                },
                "break" => {
                    words.remove(0);
                    self.parse_break(words);
                    self.should_run_cpu = true;
                },
                "help" => {
                    Debugger::display_help("");
                },
                "run" => {
                    words.remove(0);
                    self.parse_run(words);
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
            println!("**ERROR: {}", error_msg);
        }
        println!("- show [cpu|ioregs|memory]\n\tShow state of component.");
        println!("- step\n\tRun instruction pointed by PC and print it.");
        println!("- last\n\tPrint last instruction.");
        println!("- break <address in hex> [cpu|human]\n\tRun instructions until the instruction at the provided address is run.\
                 \n\tIf cpu or human (or both) are set, print each instruction run.");
        println!("- run [debug [cpu|human]]\n\tDisable the debugger and run the code.\
                             \n\tIf debug is set, information about cpu state or instruction (human friendly) or both (if both are set) will be printed.");
        println!("- help\n\tShow this.");
    }

    fn parse_run(&mut self, parameters: &[&str]) {
        if parameters.is_empty() || parameters.len() < 2 || parameters.len() > 3 {
            Debugger::display_help(&format!("Invalid number of parameters for run."));
        } else { //1 <= parameters <= 3
            match parameters[0] {
                "debug" => {
                    if let Some(value) = Debugger::cpu_human_in_params(&parameters[1..]) {
                        self.run_debug = value;
                        self.should_run_cpu = true;
                    }
                },
                _ => {
                    Debugger::display_help(&format!("Invalid parameter for run: {}", parameters[0]));
                },
            }
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
                    Debugger::display_help(&format!("***ERROR: Invalid parameter: {}\n", param));
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

    fn parse_show(parameters: &[&str], cpu: &Cpu, mem: &Memory) {
        if parameters.len() != 1 {
            Debugger::display_help(&format!("***ERROR: Invalid number of arguments for 'show'\n"));
        } else {
            match parameters[0] {
                "cpu" => {
                    println!("{}", cpu);
                },
                "ioregs" => {
                    let tima: u8 = mem.read_byte(timer::TIMA_REGISTER_ADDR);
                    let tma: u8 = mem.read_byte(timer::TMA_REGISTER_ADDR);
                    let tac: u8 = mem.read_byte(timer::TAC_REGISTER_ADDR);
                    let div: u8 = mem.read_byte(timer::DIV_REGISTER_ADDR);
                    let if_: u8 = mem.read_byte(interrupt::IF_REGISTER_ADDR);
                    let ie: u8 = mem.read_byte(interrupt::IE_REGISTER_ADDR);

                    println!("IF: {:#x} {:#b}", if_, if_);
                    println!("IE: {:#x} {:#b}", ie, ie);
                    println!("TIMA: {:#x} {:#b}", tima, tima);
                    println!("TMA: {:#x} {:#b}", tma, tma);
                    println!("TAC: {:#x} {:#b}", tac, tac);
                    println!("DIV: {:#x} {:#b}", div, div);
                },
                "memory" => {
                    println!("{}", mem);
                },
                _ => {
                    Debugger::display_help(&format!("***ERROR: Invalid parameter for 'show': {}\n",parameters[0]));
                },
            }
        }
    }

    fn parse_break(&mut self, parameters: &[&str]) {
        if parameters.is_empty() || parameters.len() > 3 {
            Debugger::display_help(&format!("***ERROR: Invalid number of arguments for 'break'\n"));
        } else {
            self.break_addr = match u16::from_str_radix(&parameters[0][2..], 16) {
                Ok(value) => Some(value),
                Err(value) => {
                    Debugger::display_help(&format!("***ERROR: Address is not a valid hex number: {}\n", value));
                    None
                },
            };
            if parameters.len() >= 2 {
                if let Some(value) = Debugger::cpu_human_in_params(&parameters[1..]) {
                    self.break_debug = value;
                    self.should_run_cpu = true;
                } else {
                    //user input incorret value
                    self.break_addr = None;
                }
            }
        }
    }
}
