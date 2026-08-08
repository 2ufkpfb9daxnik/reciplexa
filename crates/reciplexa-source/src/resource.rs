//! Source resource identity and validated UTF-8 buffers.

use core::fmt;

use crate::offset::ByteOffset;
use crate::range::{TextRange, TextRangeError};

/// Stable identifier for a loaded source resource within a package instance.
///
/// File path strings are not used as the canonical identity (`specification.md`
/// §28 Source Origin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceResourceId(pub u64);

impl SourceResourceId {
    pub const INVALID: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl fmt::Display for SourceResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source-resource:{}", self.0)
    }
}

/// Error decoding a source buffer before lexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceDecodeError {
    InvalidUtf8 { offset: usize },
    BomAndShebang,
}

impl fmt::Display for SourceDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 { offset } => {
                write!(f, "invalid UTF-8 at byte offset {offset}")
            }
            Self::BomAndShebang => {
                write!(f, "UTF-8 BOM and shebang cannot both appear at offset 0")
            }
        }
    }
}

/// A validated UTF-8 source buffer with a stable resource identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceResource {
    id: SourceResourceId,
    text: String,
    has_bom: bool,
    has_shebang: bool,
}

impl SourceResource {
    /// Decode a UTF-8 byte buffer into a source resource.
    pub fn from_bytes(id: SourceResourceId, bytes: &[u8]) -> Result<Self, SourceDecodeError> {
        let text = std::str::from_utf8(bytes).map_err(|e| SourceDecodeError::InvalidUtf8 {
            offset: e.valid_up_to(),
        })?;
        Self::from_utf8_str(id, text)
    }

    /// Wrap an already-validated UTF-8 string.
    pub fn from_utf8_str(id: SourceResourceId, text: &str) -> Result<Self, SourceDecodeError> {
        let has_bom = text.starts_with('\u{feff}');
        let body = if has_bom {
            text.strip_prefix('\u{feff}').unwrap_or(text)
        } else {
            text
        };
        let has_shebang = body.starts_with("#!");
        if has_bom && has_shebang {
            return Err(SourceDecodeError::BomAndShebang);
        }
        Ok(Self {
            id,
            text: text.to_owned(),
            has_bom,
            has_shebang,
        })
    }

    /// Convenience when the caller already owns a `String`.
    pub fn from_utf8(
        id: SourceResourceId,
        text: impl Into<String>,
    ) -> Result<Self, SourceDecodeError> {
        let text = text.into();
        Self::from_utf8_str(id, &text)
    }

    pub const fn id(&self) -> SourceResourceId {
        self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len_bytes(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub const fn has_bom(&self) -> bool {
        self.has_bom
    }

    pub const fn has_shebang(&self) -> bool {
        self.has_shebang
    }

    pub fn range_for(&self, start: u32, end: u32) -> Result<TextRange, TextRangeError> {
        TextRange::try_new(ByteOffset::new(start), ByteOffset::new(end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_utf8() {
        let src = SourceResource::from_utf8(SourceResourceId::new(1), "(page a4)").unwrap();
        assert_eq!(src.text(), "(page a4)");
        assert_eq!(src.id(), SourceResourceId::new(1));
        assert!(!src.has_bom());
    }

    #[test]
    fn rejects_invalid_utf8() {
        let bytes = b"(page \xff)";
        let err = SourceResource::from_bytes(SourceResourceId::new(1), bytes).unwrap_err();
        assert!(matches!(err, SourceDecodeError::InvalidUtf8 { .. }));
    }

    #[test]
    fn bom_at_start_is_recorded() {
        let src = SourceResource::from_utf8(SourceResourceId::new(1), "\u{feff}(doc)").unwrap();
        assert!(src.has_bom());
    }

    #[test]
    fn bom_and_shebang_conflict() {
        let err = SourceResource::from_utf8(SourceResourceId::new(1), "\u{feff}#!/usr/bin/env rpx")
            .unwrap_err();
        assert_eq!(err, SourceDecodeError::BomAndShebang);
    }

    #[test]
    fn shebang_without_bom_is_ok() {
        let src =
            SourceResource::from_utf8(SourceResourceId::new(2), "#!/usr/bin/env rpx\n(page a4)")
                .unwrap();
        assert!(src.has_shebang());
        assert!(!src.has_bom());
    }

    #[test]
    fn resource_id_display_and_validity() {
        let id = SourceResourceId::new(42);
        assert_eq!(id.to_string(), "source-resource:42");
        assert!(id.is_valid());
        assert!(!SourceResourceId::INVALID.is_valid());
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn range_for_validates_offsets() {
        let src = SourceResource::from_utf8(SourceResourceId::new(1), "abcdef").unwrap();
        let range = src.range_for(1, 4).unwrap();
        assert_eq!(range.start().get(), 1);
        assert_eq!(range.end().get(), 4);
    }

    #[test]
    fn from_bytes_roundtrip() {
        let bytes = b"(page a4)";
        let src = SourceResource::from_bytes(SourceResourceId::new(3), bytes).unwrap();
        assert_eq!(src.text(), "(page a4)");
        assert_eq!(src.len_bytes(), 9);
        assert!(!src.is_empty());
    }

    #[test]
    fn decode_error_display() {
        let err = SourceDecodeError::InvalidUtf8 { offset: 7 };
        assert!(err.to_string().contains("7"));
        assert!(SourceDecodeError::BomAndShebang.to_string().contains("BOM"));
    }

    #[test]
    fn range_for_rejects_inverted_offsets() {
        let src = SourceResource::from_utf8(SourceResourceId::new(1), "abcdef").unwrap();
        assert!(src.range_for(4, 2).is_err());
    }
}
