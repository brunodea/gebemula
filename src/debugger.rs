use cpu::cpu::{Cpu, Instruction};
use mem::mem::Memory;
use std::io::{self, Write};

pub struct Debugger {
    break_addr: Option<u16>,
    step: bool,
    debugging: bool,
    should_run_cpu: bool,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            break_addr: None,
            step: false,
            debugging: true,
            should_run_cpu: false,
        }
    }

    pub fn run(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory) {
        if !self.debugging {
            return;
        }
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
            loop {
                print!("gbm> "); //gbm: gebemula
                io::stdout().flush();
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        input.pop(); //removes the '\n'.
                        self.parse(&input, cpu, mem);
                    },
                    Err(error) => println!("error: {}", error),
                }
                if self.should_run_cpu {
                    break;
                }
            }
        }
    }

    fn parse(&mut self, command: &str, cpu: &Cpu, mem: &Memory) {
        let words: &mut Vec<&str> = &mut command.split(" ").collect();
        if !words.is_empty() {
            match words[0] {
                "show" => {
                    words.remove(0);
                    Debugger::parse_show(words, cpu, mem);
                },
                "step" => {
                    self.step = true;
                    self.should_run_cpu = true;
                },
                "break" => {
                    words.remove(0);
                    self.parse_break(words);
                    self.should_run_cpu = true;
                },
                "help" => {
                    println!("- show [cpu|ioregs|memory]");
                    println!("- step");
                    println!("- break <address in hex>");
                    println!("- run");
                    println!("- help");
                },
                "run" => {
                    self.debugging = false;
                    self.should_run_cpu = true;
                },
                "" => {
                    //does nothing
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
                    println!("Invalid parameter for 'show': {}", parameters[0]);
                },
            }
        }
    }

    fn parse_break(&mut self, parameters: &[&str]) {
        if parameters.len() != 1 {
            println!("Invalid number of arguments for 'break'");
        } else {
            self.break_addr = match u16::from_str_radix(&parameters[0][2..], 16) {
                Ok(value) => Some(value),
                Err(value) => {
                    println!("Address is not a valid hex number: {}", value);
                    None
                },
            };
        }
    }
}
