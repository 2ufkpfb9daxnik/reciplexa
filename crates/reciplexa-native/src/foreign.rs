//! Foreign values crossing the trusted boundary.

#[derive(Debug, Clone, PartialEq)]
pub enum ForeignValue {
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),
    String(String),
    Handle(u64),
}

impl ForeignValue {
    pub fn validate_handle(self) -> Result<u64, ForeignValidationError> {
        match self {
            Self::Handle(h) if h != 0 => Ok(h),
            _ => Err(ForeignValidationError::NotAHandle),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignValidationError {
    NotAHandle,
    BorrowedEscape,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
