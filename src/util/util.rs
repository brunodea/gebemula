
//TODO make sure both has_carry and has_borrow are correctly implemented.

#[inline]
pub fn has_carry_on_bit(bit: u8, lhs: u8, rhs: u8) -> bool {
    ((lhs >> bit) & 0b1 == 1) && ((rhs >> bit) & 0b1 == 1)
}
#[inline]
pub fn has_carry_on_bit16(bit: u8, lhs: u16, rhs: u16) -> bool {
    ((lhs >> bit) & 0b1 == 1) && ((rhs >> bit) & 0b1 == 1)
}
#[inline]
pub fn has_borrow_on_bit(bit: u8, lhs: u8, rhs: u8) -> bool {
    ((lhs >> bit) & 0b1 == 0) && ((rhs >> bit) & 0b1 == 1)
}
pub fn has_borrow_on_any(lhs: u8, rhs: u8) -> bool {
    let mut has = false;
    for i in 0..8 {
        has = has_borrow_on_bit(i, lhs, rhs);
        if has {
            break;
        }
    }

    has
}

#[inline]
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
