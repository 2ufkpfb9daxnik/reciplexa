use reciplexa_native::foreign::*;

#[test]
fn validate_handle_accepts_nonzero() {
    assert_eq!(ForeignValue::Handle(42).validate_handle(), Ok(42));
}

#[test]
fn validate_handle_rejects_zero() {
    assert_eq!(
        ForeignValue::Handle(0).validate_handle(),
        Err(ForeignValidationError::NotAHandle)
    );
}

#[test]
fn validate_handle_rejects_non_handle() {
    assert_eq!(
        ForeignValue::Int(1).validate_handle(),
        Err(ForeignValidationError::NotAHandle)
    );
    assert_eq!(
        ForeignValue::Float(1.5).validate_handle(),
        Err(ForeignValidationError::NotAHandle)
    );
    assert_eq!(
        ForeignValue::String("h".into()).validate_handle(),
        Err(ForeignValidationError::NotAHandle)
    );
    assert_eq!(
        ForeignValue::Bytes(vec![1]).validate_handle(),
        Err(ForeignValidationError::NotAHandle)
    );
}

#[test]
fn borrowed_escape_variant_is_documented() {
    let err = ForeignValidationError::BorrowedEscape;
    assert!(matches!(err, ForeignValidationError::BorrowedEscape));
}
