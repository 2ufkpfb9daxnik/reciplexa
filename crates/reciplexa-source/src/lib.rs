//! UTF-8 source resources and byte-oriented spans for reciplexa.
//!
//! RPX source spans are half-open UTF-8 byte intervals `[start, end)` per
//! `specification.md` §1.4. Line/column numbers are derived for display only
//! and are not used as normative identity.

#![forbid(unsafe_code)]

pub mod line_index;
pub mod offset;
pub mod range;
pub mod resource;
