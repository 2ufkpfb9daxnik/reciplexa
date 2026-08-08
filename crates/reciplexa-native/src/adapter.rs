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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_contract_metadata() {
        let p = PortableImageDecode;
        let c = p.contract();
        assert_eq!(c.name, "portable-image-decode");
        assert_eq!(c.abi_version, 1);
        assert!(c.pure_replayable);
    }

    #[test]
    fn decode_header_returns_png_magic() {
        let p = PortableImageDecode;
        let out = p.call("decode_header", &[]).unwrap();
        assert_eq!(out, ForeignValue::Bytes(vec![0x89, 0x50, 0x4E, 0x47]));
    }

    #[test]
    fn rejects_unknown_op() {
        let p = PortableImageDecode;
        let err = p.call("resize", &[]).unwrap_err();
        assert!(err.contains("unknown op"));
    }
}
