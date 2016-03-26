fn has_carry_on_bit(bit: u8, lhs: u16, rhs: u16) -> bool {
    let res_b: bool = ((lhs as u32 + rhs as u32) >> bit) & 0b1 == 1;
    let lhs_b: bool = (lhs >> bit) & 0b1 == 1;
    let rhs_b: bool = (rhs >> bit) & 0b1 == 1;

    (rhs_b & !res_b) | (lhs_b & !res_b) | (lhs_b & rhs_b)
}

pub fn has_carry(lhs: u8, rhs: u8) -> bool {
    has_carry_on_bit(7, lhs as u16, rhs as u16)
}

pub fn has_half_carry(lhs: u8, rhs: u8) -> bool {
    has_carry_on_bit(3, lhs as u16, rhs as u16)
}

pub fn has_carry16(lhs: u16, rhs: u16) -> bool {
    has_carry_on_bit(15, lhs, rhs)
}

pub fn has_half_carry16(lhs: u16, rhs: u16) -> bool {
    has_carry_on_bit(11, lhs, rhs)
}

#[inline]
pub fn has_borrow(lhs: u8, rhs: u8) -> bool {
    lhs & 0xF < rhs & 0xF
}

#[inline]
pub fn is_neg16(value: u16) -> bool {
    ((value >> 15) & 0b1) == 0b1
}

pub fn twos_complement(mut value: u16) -> u16 {
    if is_neg16(value) {
        value = !value + 1;
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

pub fn is_bit_one(value: u16, bit: u8) -> bool {
    (value >> bit) & 0b1 == 0b1
}
