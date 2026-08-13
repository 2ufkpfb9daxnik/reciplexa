//! Thin bridges from package domain natives to authoritative `reciplexa_std` tables.
//!
//! Package synthetic RPX remains the Core import surface; hosts and tests should
//! prefer std APIs (`classify_char`, `break_opportunity`) for layout decisions.

use reciplexa_std::japanese::break_opportunity_chars;

/// Fixed (prev, next, expected kind) samples shared by package linebreak notes
/// and `reciplexa_std::japanese::break_opportunity`.
///
/// Kinds are `"allowed" | "prohibited" | "inseparable"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinebreakParitySample {
    pub prev: char,
    pub next: char,
    pub expected: &'static str,
    pub note: &'static str,
}

/// Package kinsoku-profile / sample-pair-rules oriented parity set.
pub fn linebreak_parity_samples() -> &'static [LinebreakParitySample] {
    &[
        LinebreakParitySample {
            prev: '「',
            next: 'あ',
            expected: "prohibited",
            note: "open + hiragana (line-end kinsoku)",
        },
        LinebreakParitySample {
            prev: 'あ',
            next: '」',
            expected: "prohibited",
            note: "hiragana + close (line-head kinsoku)",
        },
        LinebreakParitySample {
            prev: 'あ',
            next: '。',
            expected: "prohibited",
            note: "hiragana + full-stop",
        },
        LinebreakParitySample {
            prev: 'あ',
            next: '、',
            expected: "prohibited",
            note: "hiragana + comma",
        },
        LinebreakParitySample {
            prev: '」',
            next: 'あ',
            expected: "allowed",
            note: "close + hiragana",
        },
        LinebreakParitySample {
            prev: '漢',
            next: '字',
            expected: "allowed",
            note: "ideograph run",
        },
        LinebreakParitySample {
            prev: '…',
            next: '‥',
            expected: "inseparable",
            note: "cl-08 inseparable",
        },
        LinebreakParitySample {
            prev: 'A',
            next: 'B',
            expected: "inseparable",
            note: "western letter run",
        },
        LinebreakParitySample {
            prev: 'ア',
            next: 'ー',
            expected: "prohibited",
            note: "katakana + prolonged",
        },
        LinebreakParitySample {
            prev: '〒',
            next: '1',
            expected: "prohibited",
            note: "prefix + digit",
        },
        LinebreakParitySample {
            prev: '1',
            next: '％',
            expected: "prohibited",
            note: "digit + postfix",
        },
        LinebreakParitySample {
            prev: '。',
            next: '「',
            expected: "prohibited",
            note: "full-stop before open",
        },
        LinebreakParitySample {
            prev: ' ',
            next: 'あ',
            expected: "allowed",
            note: "space + hiragana",
        },
        LinebreakParitySample {
            prev: '漢',
            next: '7',
            expected: "allowed",
            note: "ideograph + digit",
        },
    ]
}

/// Assert std `break_opportunity` matches the fixed package-oriented sample set.
///
/// Used by `reciplexa-package` tests; safe for hosts that want a quick self-check
/// that domain linebreak notes have not drifted from Rust tables.
pub fn check_linebreak_std_parity() -> Result<(), String> {
    for sample in linebreak_parity_samples() {
        let got = break_opportunity_chars(sample.prev, sample.next);
        let got_s = got.as_str();
        if got_s != sample.expected {
            return Err(format!(
                "linebreak parity mismatch for {}→{} ({}): expected {}, got {}",
                sample.prev, sample.next, sample.note, sample.expected, got_s
            ));
        }
    }
    Ok(())
}
