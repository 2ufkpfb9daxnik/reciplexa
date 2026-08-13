//! Coverage for `reciplexa_std::japanese` (OPEN-TEXT-JA-001 deepen J0+).

use reciplexa_std::japanese::{classify_char, CharClass};

#[test]
fn char_class_ids_and_codes() {
    assert_eq!(CharClass::OpeningBrackets.id(), 1);
    assert_eq!(CharClass::OpeningBrackets.code(), "cl-01");
    assert_eq!(CharClass::OpeningBrackets.name(), "opening-brackets");
    assert_eq!(CharClass::OpeningBrackets.to_string(), "cl-01");
    assert_eq!(CharClass::from_id(19), Some(CharClass::Ideographic));
    assert_eq!(CharClass::from_id(0), None);
    assert!(CharClass::FullStops.is_punctuation());
    assert!(CharClass::Hiragana.is_kana());
    assert!(CharClass::WesternCharacters.is_western());
    assert!(CharClass::Ideographic.is_square_letter());
}

#[test]
fn classify_char_useful_subset() {
    // Mirrors packages/japanese linebreak classify-sample glyphs where single-char.
    assert_eq!(classify_char('「'), CharClass::OpeningBrackets);
    assert_eq!(classify_char('」'), CharClass::ClosingBrackets);
    assert_eq!(classify_char('—'), CharClass::Hyphens);
    assert_eq!(classify_char('・'), CharClass::MiddleDots);
    assert_eq!(classify_char('。'), CharClass::FullStops);
    assert_eq!(classify_char('、'), CharClass::Commas);
    assert_eq!(classify_char('…'), CharClass::Inseparable);
    assert_eq!(classify_char('々'), CharClass::IterationMarks);
    assert_eq!(classify_char('ー'), CharClass::ProlongedSoundMark);
    assert_eq!(classify_char('ぁ'), CharClass::SmallKana);
    assert_eq!(classify_char('〒'), CharClass::PrefixedAbbreviations);
    assert_eq!(classify_char('あ'), CharClass::Hiragana);
    assert_eq!(classify_char('ア'), CharClass::Katakana);
    assert_eq!(classify_char('＝'), CharClass::MathSymbols);
    assert_eq!(classify_char('漢'), CharClass::Ideographic);
    assert_eq!(classify_char('A'), CharClass::WesternCharacters);
    assert_eq!(classify_char('7'), CharClass::Numeric);
    assert_eq!(classify_char(' '), CharClass::Spaces);
}
