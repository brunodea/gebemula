use super::super::util::util;

#[test]
fn carry_borrow() {
    assert!(util::has_carry_on_bit(2, 0x4, 0x4));
    assert!(!util::has_carry_on_bit(4, 0x4, 0x4));

    assert!(util::has_carry_on_bit16(8, 0x104, 0x104));
    assert!(!util::has_carry_on_bit16(1, 0x104, 0x104));

    assert!(util::has_borrow_on_bit(2, 0x0, 0x4));
    assert!(!util::has_borrow_on_bit(1, 0x0, 0x4));

    assert!(util::has_borrow_on_any(0x10, 0x3));
    assert!(!util::has_borrow_on_any(0x12, 0x2));
}

#[test]
fn misc() {
    let val_pos: u8 = 0x05;
    let val_neg: u8 = 0xfb; //-5
    let val_pos16: u16 = util::sign_extend(val_pos);
    let val_neg16: u16 = util::sign_extend(val_neg);

    assert!(val_pos16 == 0x0005);
    assert!(val_neg16 == 0xfffb);
    assert!(!util::is_neg16(val_pos16));
    assert!(util::is_neg16(val_neg16));
    assert!(util::twos_complement(val_neg16) == 0x5);
}
