use cpu::cpu::{Cpu, Instruction};
use mem::mem::Memory;
use std::io::{self, Write};

pub struct Debugger {
    break_addr: Option<u16>,
    debugging: bool,
    should_run_cpu: bool,
    run_debug: u8, //0b0000_0000 - bit 0: cpu, bit 1: human;
    is_step: bool,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            break_addr: None,
            debugging: true,
            should_run_cpu: false,
            run_debug: 0x00,
            is_step: false,
        }
    }

    pub fn run(&mut self, instruction: &Instruction, cpu: &Cpu, mem: &Memory) {
        if !self.debugging {
            return;
        } else if self.run_debug != 0x00 {
            let debug_cpu: bool = self.run_debug & 0b1 == 0b1;
            let debug_human: bool = (self.run_debug >> 1) & 0b1 == 0b1;

            if debug_human {
                let v: &str = if debug_cpu { ":\n\t" } else { "\n" };
                print!("{}{}", instruction, v);
            }
            if debug_cpu {
                println!("{}", cpu);
            }

            return;
        }
        if self.is_step {
            println!("{}", instruction); //prints the instruction run after step.
        }
        if let Some(addr) = self.break_addr {
            if instruction.address >= addr { //>= because the provided address may point to an immediate, in which case == would never be true.
                println!("{}", instruction);
                self.break_addr = None;
                self.read_loop(instruction, cpu, mem);
            }
        } else {
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
                    println!("- show [cpu|ioregs|memory]\n\tShow state of component.");
                    println!("- step\n\tRun instruction pointed by PC and print it.");
                    println!("- last\n\tPrint last instruction.");
                    println!("- break <address in hex>\n\tRun instructions until the instruction at the provided address is run.");
                    println!("- run [debug [cpu|human]]\n\tDisable the debugger and run the code.\
                             \n\tIf debug is set, information about cpu state or instruction (human friendly) or both (if both are set) will be print.");
                    println!("- help\n\tShow this.");
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
        } else if parameters.len() > 3 {
            println!("Invalid number of parameters for run.");
        } else { //1 <= parameters <= 3
            match parameters[0] {
                "debug" => {
                    let mut cpu: bool = false;
                    let mut human: bool = false;
                    for param in &parameters[1..] {
                        match *param {
                            "cpu" => {
                                cpu = true;
                            },
                            "human" => {
                                human = true;
                            },
                            _ => {
                                println!("Invalid parameter for `run debug`: {}", param);
                                break;
                            }
                        }
                    }
                    if cpu || human {
                        self.run_debug = if human { 0b10 } else { 0b00 };
                        self.run_debug = if cpu { self.run_debug | 0b01 } else { self.run_debug };
                        self.should_run_cpu = true;
                    }
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
