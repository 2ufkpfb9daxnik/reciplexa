//! JLReq Profile v1 classification and pair-rule tables.
//!
//! The v1 identity is this module plus [`JLREQ_PROFILE_V1_TABLES`]. Full UCS /
//! normative appendix C remain OPEN; v1 reuses the std Wave class buckets as
//! the frozen Profile v1 dataset rather than silently tracking later std
//! heuristic edits without a version bump.

use reciplexa_std::japanese::{
    break_opportunity_chars, classify_char, hang_width_em, is_hangable, is_line_end_prohibited,
    is_line_head_prohibited, is_trimmable_line_end, is_trimmable_line_head, trimming_width_em,
    BreakOpportunity, CharClass, BREAK_PAIR_MATRIX, BREAK_PAIR_MATRIX_DIM,
};

/// Versioned table identity consumed by the JLReq engine (not the stub API).
pub const JLREQ_PROFILE_V1_TABLES: &str = "jlreq-profile-v1.0";

pub fn class_of(c: char) -> CharClass {
    classify_char(c)
}

pub fn pair_break(prev: char, next: char) -> BreakOpportunity {
    break_opportunity_chars(prev, next)
}

pub fn hangable(class: CharClass) -> bool {
    is_hangable(class)
}

pub fn hang_em(class: CharClass) -> f64 {
    hang_width_em(class)
}

pub fn line_end_prohibited(class: CharClass) -> bool {
    is_line_end_prohibited(class)
}

pub fn line_head_prohibited(class: CharClass) -> bool {
    is_line_head_prohibited(class)
}

pub fn trimmable_line_end(class: CharClass) -> bool {
    is_trimmable_line_end(class)
}

pub fn trimmable_line_head(class: CharClass) -> bool {
    is_trimmable_line_head(class)
}

pub fn trim_em(class: CharClass) -> f64 {
    trimming_width_em(class)
}

/// Pin the v1 pair-matrix shape so a silent std resize cannot pass unnoticed.
pub fn pair_matrix_dim() -> usize {
    BREAK_PAIR_MATRIX_DIM
}

pub fn pair_matrix_fingerprint() -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for row in BREAK_PAIR_MATRIX.iter() {
        for cell in row {
            let tag = match cell {
                BreakOpportunity::Allowed => 1u64,
                BreakOpportunity::Prohibited => 2,
                BreakOpportunity::Inseparable => 3,
            };
            h ^= tag;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    h
}
