//! Trusted adapter contracts.

use crate::foreign::ForeignValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterContract {
    pub name: String,
    pub abi_version: u32,
    pub pure_replayable: bool,
}

pub trait NativeProvider: Send + Sync {
    fn contract(&self) -> AdapterContract;
    fn call(&self, op: &str, args: &[ForeignValue]) -> Result<ForeignValue, String>;
}

/// Portable image decode fallback (always available).
#[derive(Debug, Clone, Default)]
pub struct PortableImageDecode;

impl NativeProvider for PortableImageDecode {
    fn contract(&self) -> AdapterContract {
        AdapterContract {
            name: "portable-image-decode".into(),
            abi_version: 1,
            pure_replayable: true,
        }
    }

    fn call(&self, op: &str, _args: &[ForeignValue]) -> Result<ForeignValue, String> {
        if op != "decode_header" {
            return Err(format!("unknown op {op}"));
        }
        Ok(ForeignValue::Bytes(vec![0x89, 0x50, 0x4E, 0x47]))
    }
}
