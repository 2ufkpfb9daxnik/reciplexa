//! Package ↔ std japanese linebreak parity (J7).

use reciplexa_package::{
    check_linebreak_std_parity, linebreak_parity_samples, japanese_linebreak_source,
};

#[test]
fn linebreak_std_parity_samples_pass() {
    assert!(
        linebreak_parity_samples().len() >= 10,
        "parity set should cover core kinsoku pairs"
    );
    check_linebreak_std_parity().expect("domain linebreak notes vs std break_opportunity");
}

#[test]
fn linebreak_source_mentions_std_authority_and_classify_note() {
    let src = japanese_linebreak_source();
    assert!(src.contains("reciplexa_std::japanese"));
    assert!(src.contains("break_opportunity"));
    assert!(src.contains("classify-sample"));
    assert!(src.contains("future intrinsic") || src.contains("classify_char"));
}
