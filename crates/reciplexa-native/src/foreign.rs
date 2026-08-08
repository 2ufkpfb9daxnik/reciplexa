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
