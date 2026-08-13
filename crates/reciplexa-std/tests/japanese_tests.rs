//! Coverage for `reciplexa_std::japanese` (OPEN-TEXT-JA-001 deepen J0+).

use reciplexa_std::japanese::{
    break_opportunity, break_opportunity_chars, classify_char, is_line_end_prohibited,
    is_line_head_prohibited, BreakOpportunity, CharClass,
};

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

/// J5 — one sample per expanded class / range (still a subset, not full UCS).
#[test]
fn classify_char_expanded_class_samples() {
    // Punctuation expansions (kinsoku + JLReq appendix A common forms).
    assert_eq!(classify_char('【'), CharClass::OpeningBrackets);
    assert_eq!(classify_char('】'), CharClass::ClosingBrackets);
    assert_eq!(classify_char('〖'), CharClass::OpeningBrackets);
    assert_eq!(classify_char('〗'), CharClass::ClosingBrackets);
    assert_eq!(classify_char('｢'), CharClass::OpeningBrackets);
    assert_eq!(classify_char('｣'), CharClass::ClosingBrackets);
    assert_eq!(classify_char('―'), CharClass::Hyphens);
    assert_eq!(classify_char('－'), CharClass::Hyphens);
    assert_eq!(classify_char('‼'), CharClass::DividingPunctuation);
    assert_eq!(classify_char('：'), CharClass::MiddleDots);
    assert_eq!(classify_char('．'), CharClass::FullStops);
    assert_eq!(classify_char('，'), CharClass::Commas);
    assert_eq!(classify_char('⋯'), CharClass::Inseparable);
    assert_eq!(classify_char('〻'), CharClass::IterationMarks);
    assert_eq!(classify_char('€'), CharClass::PrefixedAbbreviations);
    assert_eq!(classify_char('‰'), CharClass::PostfixedAbbreviations);
    assert_eq!(classify_char('≤'), CharClass::MathSymbols);

    // Warichu / unit / enclosed / ornament.
    assert_eq!(classify_char('｟'), CharClass::WarichuOpen);
    assert_eq!(classify_char('｠'), CharClass::WarichuClose);
    assert_eq!(classify_char('㎜'), CharClass::UnitSymbols);
    assert_eq!(classify_char('①'), CharClass::EnclosedAlphanumerics);
    assert_eq!(classify_char('※'), CharClass::Ornaments);

    // Kana / digits / latin ranges.
    assert_eq!(classify_char('ｯ'), CharClass::SmallKana);
    assert_eq!(classify_char('ｱ'), CharClass::Katakana); // halfwidth
    assert_eq!(classify_char('７'), CharClass::Numeric); // fullwidth digit
    assert_eq!(classify_char('²'), CharClass::Numeric);
    assert_eq!(classify_char('Ａ'), CharClass::WesternCharacters); // fullwidth Latin
    assert_eq!(classify_char('_'), CharClass::SimpleWestern);
    assert_eq!(classify_char('é'), CharClass::ComplexWestern);
    assert_eq!(classify_char('\u{00A0}'), CharClass::Spaces); // NBSP
}

#[test]
fn break_opportunity_kinsoku_stub() {
    assert!(is_line_head_prohibited(CharClass::FullStops));
    assert!(is_line_end_prohibited(CharClass::OpeningBrackets));

    // Open + letter: cannot break after open (line-end prohibited).
    assert_eq!(
        break_opportunity(CharClass::OpeningBrackets, CharClass::Hiragana),
        BreakOpportunity::Prohibited
    );
    // Letter + full stop: cannot break before stop (line-head prohibited).
    assert_eq!(
        break_opportunity_chars('あ', '。'),
        BreakOpportunity::Prohibited
    );
    // Closing before letter: break after close is allowed (close is head-kinsoku only).
    assert_eq!(
        break_opportunity_chars('」', 'あ'),
        BreakOpportunity::Allowed
    );
    // Letter before close: cannot put close at line head.
    assert_eq!(
        break_opportunity_chars('あ', '」'),
        BreakOpportunity::Prohibited
    );
    // Inseparable ellipsis run.
    assert_eq!(
        break_opportunity(CharClass::Inseparable, CharClass::Inseparable),
        BreakOpportunity::Inseparable
    );
    // Western letter run stays together.
    assert_eq!(
        break_opportunity_chars('A', 'B'),
        BreakOpportunity::Inseparable
    );
    // Ideograph boundary may break.
    assert_eq!(
        break_opportunity(CharClass::Ideographic, CharClass::Ideographic),
        BreakOpportunity::Allowed
    );
    assert!(BreakOpportunity::Allowed.may_break());
    assert!(!BreakOpportunity::Prohibited.may_break());
    assert_eq!(BreakOpportunity::Inseparable.as_str(), "inseparable");
}

#[test]
fn kihon_hanmen_size_helpers() {
    use reciplexa_std::japanese::{line_rate, KihonHanmen, WritingMode};

    let h = KihonHanmen::default_horizontal();
    assert_eq!(h.writing_mode, WritingMode::HorizontalTb);
    assert!(!h.writing_mode.is_vertical());
    assert_eq!(h.writing_mode.as_str(), "horizontal-tb");
    assert!((h.line_gap_em() - 0.5).abs() < 1e-9);
    assert!((h.hanmen_inline_em() - 40.0).abs() < 1e-9);
    assert!((h.hanmen_block_em() - 44.5).abs() < 1e-9);
    assert!((h.indent_em(1) - 1.0).abs() < 1e-9);
    assert!((h.heading_band_em(2) - 2.5).abs() < 1e-9);

    let v = KihonHanmen::default_vertical();
    assert!(v.writing_mode.is_vertical());
    assert_eq!(v.writing_mode.to_string(), "vertical-rl");
    assert!((v.hanmen_inline_em() - 35.0).abs() < 1e-9);
    assert!((v.hanmen_block_em() - 29.5).abs() < 1e-9);
    assert!((line_rate::DEFAULT - 1.5).abs() < 1e-9);
    assert!((line_rate::SOLID - 1.0).abs() < 1e-9);
}

#[test]
fn ruby_and_tate_chu_yoko_types() {
    use reciplexa_std::japanese::{Ruby, RubyKind, TateChuYoko, TategakiParagraph, WritingMode};

    let r = Ruby::simple("漢", "かん");
    assert_eq!(r.kind, RubyKind::Simple);
    assert_eq!(r.tag(), "ja-ruby");
    assert_eq!(r.to_string(), "漢(かん)");
    let j = Ruby::jukugo("東京", "とうきょう");
    assert_eq!(j.kind.as_str(), "jukugo");
    let t = TateChuYoko::new("12");
    assert_eq!(t.tag(), "ja-tate-chu-yoko");
    assert!(t.to_string().contains("12"));
    let p = TategakiParagraph::new("縦書き");
    assert_eq!(p.writing_mode, WritingMode::VerticalRl);
    assert_eq!(p.tag(), "ja-tategaki-paragraph");
}

/// Demo: classify + break over a short Japanese phrase (J4).
#[test]
fn classify_and_break_phrase_demo() {
    let phrase: Vec<char> = "「東京。」AB".chars().collect();
    let classes: Vec<_> = phrase.iter().copied().map(classify_char).collect();
    assert_eq!(classes[0], CharClass::OpeningBrackets);
    assert_eq!(classes[1], CharClass::Ideographic);
    assert_eq!(classes[3], CharClass::FullStops);
    assert_eq!(classes[4], CharClass::ClosingBrackets);
    assert_eq!(classes[5], CharClass::WesternCharacters);

    let mut breaks = Vec::new();
    for w in phrase.windows(2) {
        breaks.push(break_opportunity_chars(w[0], w[1]));
    }
    // 「東 — open forbids break after itself
    assert_eq!(breaks[0], BreakOpportunity::Prohibited);
    // 東京 — ideographs may break
    assert_eq!(breaks[1], BreakOpportunity::Allowed);
    // 京。 — full stop is line-head prohibited
    assert_eq!(breaks[2], BreakOpportunity::Prohibited);
    // 。」 — close is line-head prohibited
    assert_eq!(breaks[3], BreakOpportunity::Prohibited);
    // 」A — allowed after close
    assert_eq!(breaks[4], BreakOpportunity::Allowed);
    // AB — western run inseparable
    assert_eq!(breaks[5], BreakOpportunity::Inseparable);
}
