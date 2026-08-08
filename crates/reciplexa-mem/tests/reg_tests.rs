use reciplexa_mem::reg::{Reg, RegAlloc};

#[test]
fn reg_display() {
    assert_eq!(Reg(0).to_string(), "r0");
    assert_eq!(Reg(42).to_string(), "r42");
}

#[test]
fn reg_invalid_constant() {
    assert_eq!(Reg::INVALID, Reg(u32::MAX));
}

#[test]
fn reg_alloc_fresh_increments() {
    let mut alloc = RegAlloc::default();
    assert_eq!(alloc.fresh(), Reg(0));
    assert_eq!(alloc.fresh(), Reg(1));
    assert_eq!(Reg::new(7), Reg(7));
}

#[test]
fn reg_ordering() {
    assert!(Reg(1) < Reg(2));
    assert_eq!(Reg(5), Reg(5));
}
