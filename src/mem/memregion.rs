pub struct MemoryRegion {
    address_start: u16,
    address_end: u16,
}

impl MemoryRegion {
    pub fn new(start: u16, end: u16) -> MemoryRegion {
        MemoryRegion {
            address_start: start,
            address_end: end,
        }
    }

    pub fn start(&self) -> u16 {
        self.address_start
    }

    pub fn end(&self) -> u16 {
        self.address_end
    }
}
