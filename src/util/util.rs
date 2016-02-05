
//TODO make sure both has_carry and has_borrow are correct implemented.

pub fn has_carry_on_bit(bit: u8, byte_1: u8, byte_2: u8) -> bool {
    let mask: u8 = 1 << bit;
    byte_1 & mask == byte_2 & mask
}

pub fn has_carry_on_bit16(bit: u8, lhs: u16, rhs: u16) -> bool {
    let mask: u16 = 1 << bit as u16;
    lhs & mask == rhs & mask
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

//adds value to twos_complement value
pub fn twos_complement(twos_complement: u8, value: u16) -> u16 {
    let mut value_i: i8 = twos_complement as i8;
    if value_i < 0 {
        value_i= -(!value_i - 1);
    }

    ((value as i32) + (value_i as i32)) as u16
}
