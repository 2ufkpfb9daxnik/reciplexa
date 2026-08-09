//! Fingerprint unit tests.

use reciplexa_opt::Fingerprint;

#[test]
fn same_input_same_fingerprint() {
    let a = Fingerprint::of(&("ir", 42u64));
    let b = Fingerprint::of(&("ir", 42u64));
    assert_eq!(a, b);
    assert!(!a.is_zero());
}

#[test]
fn different_input_diverges() {
    let a = Fingerprint::of(&"layout");
    let b = Fingerprint::of(&"render");
    assert_ne!(a, b);
}

#[test]
fn combine_is_order_sensitive() {
    let x = Fingerprint::of(&1u32);
    let y = Fingerprint::of(&2u32);
    assert_ne!(Fingerprint::combine(&[x, y]), Fingerprint::combine(&[y, x]));
}

#[test]
fn display_and_zero() {
    let z = Fingerprint::zero();
    assert!(z.is_zero());
    assert_eq!(z.as_u64(), 0);
    assert_eq!(format!("{z}"), "fp:0000000000000000");
    let from = Fingerprint::from(0xabcdu64);
    assert_eq!(from.as_u64(), 0xabcd);
}
