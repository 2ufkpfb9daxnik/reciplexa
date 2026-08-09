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
        f.write_str(&format!("source-resource:{}", self.0))
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
        let msg = match self {
            Self::InvalidUtf8 { offset } => {
                format!("invalid UTF-8 at byte offset {offset}")
            }
            Self::BomAndShebang => {
                "UTF-8 BOM and shebang cannot both appear at offset 0".to_string()
            }
        };
        f.write_str(&msg)
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
            text.strip_prefix('\u{feff}')
                .expect("BOM flag implies prefix")
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
