use cpu::cpu::{Cpu, Instruction};
use mem::mem::Memory;
use std::io;

pub struct Debugger {
    break_addr: Option<u16>,
    step: bool,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            break_addr: None,
            step: false,
        }
    }

    pub fn run(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory) {
        if self.step {
            println!("{}", instruction);
            self.step = false;
        }
        if let Some(addr) = self.break_addr {
            if instruction.address == addr {
                println!("{}", instruction);
                self.break_addr = None;
            }
        } else {
            print!("gbm> "); //gbm: gebemula
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    self.parse(&input, cpu, mem);
                },
                Err(error) => println!("error: {}", error),
            }
        }
    }

    fn parse(&mut self, command: &str, cpu: &Cpu, mem: &Memory) {
        let words: &mut Vec<&str> = &mut command.split(" ").collect();
        if !words.is_empty() {
            match words[0] {
                "show" => {
                    words.pop();
                    Debugger::parse_show(words, cpu, mem);
                },
                "step" => {
                    self.step = true;
                },
                "break" => {
                    words.pop();
                    self.parse_break(words);
                },
                "help" => {
                    println!("Commands available:");
                    println!("show [cpu|ioregs|memory]");
                    println!("step");
                    println!("break <address>");
                    println!("help");
                },
                _ => println!("Invalid command: {}", words[0]),
            }
        }
    }

    fn parse_show(parameters: &[&str], cpu: &Cpu, mem: &Memory) {
        if parameters.len() != 1 {
            println!("Invalid number of arguments for 'show'");
        } else {
            match parameters[0] {
                "cpu" => {
                    println!("{}", cpu);
                },
                "ioregs" => {
                    println!("Not implemented in Debugger yet!");
                },
                "memory" => {
                    println!("{}", mem);
                },
                _ => {
                    println!("Invalid parameter for 'show'.");
                },
            }
        }
    }

    fn parse_break(&mut self, parameters: &[&str]) {
        if parameters.len() != 1 {
            println!("Invalid number of arguments for 'break'");
        } else {
            self.break_addr = match parameters[0].parse::<u16>() {
                Ok(value) => Some(value),
                Err(value) => {
                    println!("Address is not a number: {}", value);
                    None
                },
            };
        }
    }
}
