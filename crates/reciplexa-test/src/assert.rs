//! Structured equality assertions.

use core::fmt;

/// Difference between two structured values for test output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredDiff {
    pub path: String,
    pub expected: String,
    pub actual: String,
}

impl fmt::Display for StructuredDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "at {}: expected {}, got {}",
            self.path, self.expected, self.actual
        )
    }
}

/// Assert two `Debug` values stringify equally — interim until derive-based diff.
pub fn assert_eq_structured<T: fmt::Debug>(expected: &T, actual: &T) {
    let e = format!("{expected:?}");
    let a = format!("{actual:?}");
    assert_eq!(e, a, "structured values differ");
}

/// Assert diagnostic codes match in order (ignoring instance ids).
pub fn assert_diagnostic_codes(actual_codes: &[String], expected_codes: &[&str]) {
    assert_eq!(
        actual_codes.len(),
        expected_codes.len(),
        "diagnostic count mismatch"
    );
    for (actual, expected) in actual_codes.iter().zip(expected_codes.iter()) {
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_eq_passes_for_identical_values() {
        assert_eq_structured(&vec![1, 2], &vec![1, 2]);
    }

    #[test]
    #[should_panic]
    fn structured_eq_fails_for_different_values() {
        assert_eq_structured(&1, &2);
    }
}
