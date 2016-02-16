use cpu;
use cpu::cpu::{Cpu, Instruction};
use cpu::timer::Timer;
use mem;
use mem::mem::Memory;
use std::io::{self, Write};
use std::str;

pub struct Debugger {
    break_addr: Option<u16>,
    should_run_cpu: bool,
    run_debug: Option<u8>, //0b0000_0000 - bit 0: cpu, bit 1: human;
    break_debug: u8, //same as run_debug
    num_steps: u32,
    display_header: bool,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            break_addr: None,
            should_run_cpu: false,
            run_debug: None,
            break_debug: 0x00,
            num_steps: 0,
            display_header: true,
        }
    }

    pub fn run(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory, timer: &Timer) {
        if self.display_header {
            println!("##################################");
            println!("#     Gebemula Debug Console     #");
            println!("##################################");
            let game_title_u8: &mut Vec<u8> = &mut Vec::new();
            for byte in mem::consts::GAME_TITLE_ADDR_START..(mem::consts::GAME_TITLE_ADDR_END + 1) {
                if byte == 0 {
                    break;
                }
                game_title_u8.push(mem.read_byte(byte));
            }
            let game_title: &str = match str::from_utf8(&game_title_u8) {
                Ok(v) => v,
                Err(_) => "Undefined",
            };
            println!("Game: {}", game_title);
            println!("Type 'help' for the command list.")
            println!("----------------------------------");
            self.display_header = false;
        }
        if self.run_debug != None {
            self.print_cpu_human(self.run_debug.unwrap(), instruction, cpu);
            return;
        }
        if let Some(addr) = self.break_addr {
            if instruction.address == addr {
                println!("{}", instruction);
                self.break_addr = None;
                self.read_loop(instruction, cpu, mem, timer);
            } else {
                self.print_cpu_human(self.break_debug, instruction, cpu);
            }
        } else {
            let go_to_loop: bool = match self.num_steps {
                0 => true,
                _ => {
                    self.num_steps -= 1;
                    if self.num_steps == 0 {
                        println!("{}", instruction); //prints the instruction run after step.
                        true
                    } else {
                        false
                    }
                },
            };
            if go_to_loop {
                self.read_loop(instruction, cpu, mem, timer);
            }
        }
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
                    self.parse_break(&words[1..]);
                },
                "help" => {
                    Debugger::display_help("");
                },
                "run" => {
                    self.parse_run(&words[1..]);
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
        println!("- show [cpu|ioregs|events|memory [<min_addr_hex> <max_addr_hex>]\n\tShow state of component.");
        println!("- step [num (decimal)]\n\tRun instruction pointed by PC and print it.\
                 \n\tIf a num is set, run step num times and print the last one.");
        println!("- last\n\tPrint last instruction.");
        println!("- break <address in hex> [cpu|human]\n\tRun instructions until the instruction at the provided address is run.\
                 \n\tIf cpu or human (or both) are set, print each instruction run.");
        println!("- run [cpu|human]\n\tDisable the debugger and run the code.\
                             \n\tIf set, information about cpu state or instruction (human friendly) or both (if both are set) will be printed.");
        println!("- help\n\tShow this.");
    }

    fn parse_step(&mut self, parameters: &[&str]) {
        if parameters.is_empty() {
            self.num_steps = 1;
        } else if parameters.len() == 1 {
            let steps = match parameters[0].parse::<u32>() {
                Ok(s) => s,
                Err(e) => {
                    Debugger::display_help(&format!("Couldn't parse number of steps: {}", e));
                    0
                },
            };
            self.num_steps = steps;
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

    fn parse_show(parameters: &[&str], cpu: &Cpu, mem: &Memory, timer: &Timer) {
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

                println!("IF: {:#x} {:#b}", if_, if_);
                println!("IE: {:#x} {:#b}", ie, ie);
                println!("TIMA: {:#x} {:#b}", tima, tima);
                println!("TMA: {:#x} {:#b}", tma, tma);
                println!("TAC: {:#x} {:#b}", tac, tac);
                println!("DIV: {:#x} {:#b}", div, div);
            },
            "memory" => {
                Debugger::parse_show_memory(&parameters[1..], mem);
            },
            "events" => {
                println!("{}", timer.events_to_str());
            }
            _ => {
                Debugger::display_help(&format!("***ERROR: Invalid parameter for 'show': {}\n",parameters[0]));
            },
        }
    }

    fn parse_show_memory(parameters: &[&str], mem: &Memory) {
        if parameters.len() != 2 {
            Debugger::display_help(&format!("***ERROR: Invalid number of arguments for 'show memory'\n"));
        } else {
            let min_addr = Debugger::hex_from_str(parameters[0]);
            let max_addr = Debugger::hex_from_str(parameters[1]);
            if min_addr != None && max_addr != None {
                println!("{}", mem.format(min_addr, max_addr));
            }
        }
    }

    fn parse_break(&mut self, parameters: &[&str]) {
        if parameters.is_empty() || parameters.len() > 3 {
            Debugger::display_help(&format!("***ERROR: Invalid number of arguments for 'break'\n"));
        } else {
            if let Some(addr) = Debugger::hex_from_str(&parameters[0]) {
                self.should_run_cpu = true;
                self.break_addr = Some(addr);
            }

            if parameters.len() >= 2 {
                if let Some(value) = Debugger::cpu_human_in_params(&parameters[1..]) {
                    self.break_debug = value;
                } else {
                    //user input incorret value
                    self.break_addr = None;
                    self.should_run_cpu = false;
                }
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
                Debugger::display_help(&format!("***ERROR: Address is not a valid hex number: {}\n", value));
                None
            },
        }
    }
}
