//! Test harness utilities — structured comparison instead of golden files.
//!
//! Maps tests to specification sections or conformance test IDs.

#![forbid(unsafe_code)]

pub mod assert;
pub mod conformance;
pub mod outcome;

pub use assert::{assert_diagnostic_codes, assert_eq_structured, StructuredDiff};
pub use conformance::{ConformanceId, SpecSection};
pub use outcome::{assert_subject_failure, TestOutcome, TestSubject};
