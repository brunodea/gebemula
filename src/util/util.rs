
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

#[inline(always)]
pub fn is_neg16(value: u16) -> bool {
    ((value >> 15) & 0b1) == 0b1
}

pub fn twos_complement(mut value: u16) -> u16 {
    if is_neg16(value) {
        value = (!value) + 1;
    }

    value
}

pub fn sign_extend(value: u8) -> u16 {
    let mut res: u16 = value as u16;
    if (value >> 7) & 0b1 == 0b1 {
        res = 0xFF00 | res;
    }
    res
}
