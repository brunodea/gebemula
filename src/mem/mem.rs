use std::collections::HashMap;
use std::hash::Hash;

//TODO: should be Box<T>?
#[derive(Debug)]
pub struct Memory<A: Hash + Eq, T> { //A: address type, T: data type.
    map: HashMap<A, T>,
}

impl<A: Hash + Eq, T> Memory<A, T> {
    pub fn new() -> Memory<A, T> {
        Memory {
            map: HashMap::new(),
        }
    }

    pub fn write(&mut self, addr: A, value: Box<T>) {
        self.map.insert(addr, *value);
    }
    pub fn read(&self, addr: A) -> Option<&T> {
        self.map.get(&addr)
    }
}
