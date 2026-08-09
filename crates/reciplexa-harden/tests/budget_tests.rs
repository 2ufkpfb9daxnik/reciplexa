//! DecodeBudget tests.

use reciplexa_harden::{BudgetKind, DecodeBudget};

#[test]
fn charges_within_budget() {
    let mut b = DecodeBudget::new(100, 10, 5);
    assert!(b.charge_bytes(40).is_ok());
    assert!(b.charge_bytes(60).is_ok());
    assert_eq!(b.bytes_used(), 100);
    assert_eq!(b.remaining_bytes(), 0);
}

#[test]
fn rejects_byte_overrun() {
    let mut b = DecodeBudget::new(50, 100, 100);
    let err = b.charge_bytes(51).unwrap_err();
    assert_eq!(err.kind, BudgetKind::Bytes);
    assert_eq!(err.limit, 50);
    assert_eq!(err.attempted, 51);
}

#[test]
fn rejects_ops_and_allocs() {
    let mut b = DecodeBudget::new(1000, 2, 1);
    assert!(b.charge_operations(2).is_ok());
    assert_eq!(
        b.charge_operations(1).unwrap_err().kind,
        BudgetKind::Operations
    );
    assert!(b.charge_allocations(1).is_ok());
    assert_eq!(
        b.charge_allocations(1).unwrap_err().kind,
        BudgetKind::Allocations
    );
}

#[test]
fn default_and_reset() {
    let mut b = DecodeBudget::default();
    assert!(b.max_bytes > 0);
    assert!(b.charge_bytes(10).is_ok());
    b.reset_usage();
    assert_eq!(b.bytes_used(), 0);
    assert_eq!(b.operations_used(), 0);
    assert_eq!(b.allocations_used(), 0);
}
