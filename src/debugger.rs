use cpu::cpu::{Cpu, Instruction};
use mem::mem::Memory;
use std::io::{self, Write};

pub struct Debugger {
    break_addr: Option<u16>,
    debugging: bool,
    should_run_cpu: bool,
    is_run_debug: bool,
    is_step: bool,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            break_addr: None,
            debugging: true,
            should_run_cpu: false,
            is_run_debug: false,
            is_step: false,
        }
    }

    pub fn run(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory) {
        if !self.debugging {
            return;
        } else if self.is_run_debug {
            print!("{}:\n\t", instruction);
            println!("{}", cpu);
            return;
        }
        if self.is_step {
            println!("{}", instruction); //prints the instruction run after step.
        }
        if let Some(addr) = self.break_addr {
            if instruction.address == addr {
                println!("{}", instruction);
                self.break_addr = None;
            }
        } else {
            loop {
                self.should_run_cpu = false;
                self.is_step = false;
                print!("gbm> "); //gbm: gebemula
                io::stdout().flush();
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
                    println!("- show [cpu|ioregs|memory]\n\tShows state of component.");
                    println!("- step\n\tRuns instruction pointed by PC prints it.");
                    println!("- last\n\tPrints last instruction.");
                    println!("- break <address in hex>\n\tRuns instructions until provided address.");
                    println!("- run [debug]\n\tDisables the debugger and runs the code. If 'debug' is set, the cpu state after every instruction will be printed.");
                    println!("- help\n\tShows this.");
                },
                "run" => {
                    words.remove(0);
                    self.parse_run(words);
                },
                "" => {
                    //does nothing
                },
                _ => println!("Invalid command: {}", words[0]),
            }
        }
    }

    fn parse_run(&mut self, parameters: &[&str]) {
        if parameters.is_empty() {
            self.debugging = false;
            self.should_run_cpu = true;
        } else if parameters.len() > 1 {
            println!("Invalid number of parameters for run.");
        } else {
            match parameters[0] {
                "debug" => {
                    self.is_run_debug = true;
                    self.should_run_cpu = true;
                },
                _ => println!("Invalid parameter for run: {}", parameters[0]),
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
