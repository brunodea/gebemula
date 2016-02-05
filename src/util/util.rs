
//TODO make sure both has_carry and has_borrow are correct implemented.

pub fn has_carry_on_bit(bit: u8, byte_1: u8, byte_2: u8) -> bool {
    let mask: u8 = 1 << bit;
    byte_1 & mask == byte_2 & mask
}

pub fn has_borrow_on_bit(bit: u8, byte_1: u8, byte_2: u8) -> bool {
    let mask: u8 = 1 << bit;
    byte_1 & mask == 0b0 && byte_2 & mask == 0b1
}

pub fn has_borrow_on_any(byte_1: u8, byte_2: u8) -> bool {
    let mut has = false;
    for i in 0..8 {
        has = has_borrow_on_bit(i, byte_1, byte_2);
        if has {
            break;
        }
    }

    has
}

pub fn twos_complement(value: u8) -> i16 {
    let mut value_i: i16 = value as i16;
    if value_i < 0 {
        value_i= -(!value_i - 1);
    }

    value_i
}
