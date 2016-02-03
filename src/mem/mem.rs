use std::collections::HashMap;

#[derive(Debug)]
pub struct Memory {
    pub data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Memory {
        Memory {
            data: vec![0; size],
        }
    }

    pub fn write(&mut self, position: usize, value: u8) {
        self.data[position] = value;
    }
    pub fn read(&mut self, position: usize) -> u8 {
        self.data[position]
    }
}
