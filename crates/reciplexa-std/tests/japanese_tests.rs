//! Coverage for `reciplexa_std::japanese` (OPEN-TEXT-JA-001 deepen J0+).

use reciplexa_std::japanese::{
    break_line, break_opportunity, break_opportunity_chars, char_em_width, classify_char,
    is_line_end_prohibited, is_line_head_prohibited, BreakOpportunity, CharClass,
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

/// Ideographic punctuation (CJK Symbols block) + more fullwidth forms.
#[test]
fn classify_char_ideographic_punct_and_fullwidth() {
    // CJK quotation / ditto / closing mark / wavy dash.
    assert_eq!(classify_char('〝'), CharClass::OpeningBrackets);
    assert_eq!(classify_char('〞'), CharClass::ClosingBrackets);
    assert_eq!(classify_char('〟'), CharClass::ClosingBrackets);
    assert_eq!(classify_char('〃'), CharClass::Inseparable);
    assert_eq!(classify_char('〆'), CharClass::DividingPunctuation);
    assert_eq!(classify_char('〰'), CharClass::Hyphens);
    assert_eq!(classify_char('゠'), CharClass::Hyphens);
    assert_eq!(classify_char('〓'), CharClass::Ornaments);
    assert_eq!(classify_char('〽'), CharClass::Ornaments);
    assert_eq!(classify_char('〇'), CharClass::Ornaments);

    // Halfwidth CJK punctuation (FF61–FF65).
    assert_eq!(classify_char('｡'), CharClass::FullStops);
    assert_eq!(classify_char('､'), CharClass::Commas);

    // Fullwidth operators / signs (FF01–FF5E / FFE0+).
    assert_eq!(classify_char('＋'), CharClass::MathSymbols);
    assert_eq!(classify_char('＜'), CharClass::MathSymbols);
    assert_eq!(classify_char('＞'), CharClass::MathSymbols);
    assert_eq!(classify_char('＆'), CharClass::MathSymbols);
    assert_eq!(classify_char('＠'), CharClass::PrefixedAbbreviations);
    assert_eq!(classify_char('／'), CharClass::DividingPunctuation);
    assert_eq!(classify_char('＼'), CharClass::DividingPunctuation);
    assert_eq!(classify_char('＿'), CharClass::SimpleWestern);
    assert_eq!(classify_char('｜'), CharClass::Ornaments);
    assert_eq!(classify_char('￣'), CharClass::Ornaments);

    // Residual CJK Symbols and Punctuation → ideographic bucket.
    assert_eq!(classify_char('\u{3030}'), CharClass::Hyphens); // 〰 already above
    assert_eq!(classify_char('\u{303E}'), CharClass::Ideographic); // 〾
}

/// More CJK symbols, Unicode spaces, and wave-dash family glyphs.
#[test]
fn classify_char_cjk_symbols_spaces_wave_dash() {
    // Wave dash / wavy line family → hyphens (cl-03).
    assert_eq!(classify_char('〜'), CharClass::Hyphens); // WAVE DASH
    assert_eq!(classify_char('～'), CharClass::Hyphens); // FULLWIDTH TILDE
    assert_eq!(classify_char('〰'), CharClass::Hyphens); // WAVY DASH
    assert_eq!(classify_char('⁓'), CharClass::Hyphens); // SWUNG DASH
    assert_eq!(classify_char('∿'), CharClass::Hyphens); // SINE WAVE
    assert_eq!(classify_char('﹏'), CharClass::Hyphens); // WAVY LOW LINE
    assert_eq!(classify_char('﹋'), CharClass::Hyphens); // WAVY OVERLINE

    // Extra spaces (cl-14).
    assert_eq!(classify_char('\u{2000}'), CharClass::Spaces); // EN QUAD
    assert_eq!(classify_char('\u{2009}'), CharClass::Spaces); // THIN SPACE
    assert_eq!(classify_char('\u{200B}'), CharClass::Spaces); // ZWSP
    assert_eq!(classify_char('\u{202F}'), CharClass::Spaces); // NARROW NBSP
    assert_eq!(classify_char('\u{205F}'), CharClass::Spaces); // MMSP
    assert_eq!(classify_char('\u{3000}'), CharClass::Spaces); // IDEOGRAPHIC SPACE
    assert_eq!(classify_char('\u{FEFF}'), CharClass::Spaces); // ZWNBSP

    // More CJK / math brackets and symbols.
    assert_eq!(classify_char('⌈'), CharClass::OpeningBrackets);
    assert_eq!(classify_char('⌋'), CharClass::ClosingBrackets);
    assert_eq!(classify_char('⁄'), CharClass::DividingPunctuation);
    assert_eq!(classify_char('〱'), CharClass::Inseparable);
    assert_eq!(classify_char('〄'), CharClass::Ornaments);
    assert_eq!(classify_char('〼'), CharClass::Ornaments);
    assert_eq!(classify_char('㍉'), CharClass::UnitSymbols);
    assert_eq!(classify_char('❶'), CharClass::EnclosedAlphanumerics);
    assert_eq!(classify_char('㊤'), CharClass::EnclosedAlphanumerics);
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

/// J6 — package kinsoku-profile sample strings ↔ break matrix.
#[test]
fn break_opportunity_kinsoku_profile_samples() {
    // From japanese/linebreak `sample-line-head-prohibited`.
    const LINE_HEAD: &str =
        "」、。．，）〕］｝〉》』】！？ーぁぃぅぇぉっゃゅょァィゥェォッャュョヽヾゝゞ々";
    for ch in LINE_HEAD.chars() {
        let class = classify_char(ch);
        assert!(
            is_line_head_prohibited(class),
            "expected line-head prohibited for {ch:?} ({class})"
        );
        assert_eq!(
            break_opportunity_chars('あ', ch),
            BreakOpportunity::Prohibited,
            "letter + {ch:?} should be prohibited"
        );
    }

    // From japanese/linebreak `sample-line-end-prohibited`.
    const LINE_END: &str = "「『（〔［｛〈《￥＄￡＃";
    for ch in LINE_END.chars() {
        let class = classify_char(ch);
        assert!(
            is_line_end_prohibited(class),
            "expected line-end prohibited for {ch:?} ({class})"
        );
        assert_eq!(
            break_opportunity_chars(ch, 'あ'),
            BreakOpportunity::Prohibited,
            "{ch:?} + letter should be prohibited"
        );
    }

    // Inseparable sample glyphs (ellipsis / vertical forms; em-dash is cl-03).
    for ch in "…‥〳〴〵".chars() {
        assert_eq!(
            break_opportunity_chars(ch, 'あ'),
            BreakOpportunity::Inseparable
        );
    }
    assert_eq!(classify_char('—'), CharClass::Hyphens);
    assert_eq!(
        break_opportunity_chars('あ', '—'),
        BreakOpportunity::Prohibited
    );

    // Extra prohibited pairs from package sample-pair-rules.
    assert_eq!(
        break_opportunity_chars('〒', '1'),
        BreakOpportunity::Prohibited
    );
    assert_eq!(
        break_opportunity_chars('1', '％'),
        BreakOpportunity::Prohibited
    );
    assert_eq!(
        break_opportunity_chars('。', '「'),
        BreakOpportunity::Prohibited
    );
    assert_eq!(
        break_opportunity_chars('、', '「'),
        BreakOpportunity::Prohibited
    );
    assert_eq!(
        break_opportunity_chars('ア', 'ー'),
        BreakOpportunity::Prohibited
    );
    assert_eq!(
        break_opportunity_chars('｟', '漢'),
        BreakOpportunity::Prohibited
    );
    assert_eq!(
        break_opportunity_chars('漢', '｠'),
        BreakOpportunity::Prohibited
    );
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

/// Tip coverage: Display / from_id / Other / grouped-numeral break / ASCII other.
#[test]
fn tip_char_class_display_and_edge_classify() {
    for id in 1u8..=30 {
        let c = CharClass::from_id(id).expect("id");
        assert_eq!(c.id(), id);
        assert!(!c.code().is_empty());
        assert!(!c.name().is_empty());
        assert_eq!(c.to_string(), c.code());
    }
    assert_eq!(CharClass::Other.code(), "cl-other");
    assert_eq!(CharClass::Other.to_string(), "cl-other");
    assert!(!CharClass::Other.is_punctuation());
    assert!(!CharClass::Other.is_kana());
    assert!(!CharClass::Other.is_western());
    assert!(!CharClass::Other.is_square_letter());

    assert_eq!(
        break_opportunity(CharClass::GroupedNumerals, CharClass::GroupedNumerals),
        BreakOpportunity::Inseparable
    );
    assert_eq!(BreakOpportunity::Allowed.to_string(), "allowed");
    assert_eq!(BreakOpportunity::Prohibited.to_string(), "prohibited");

    // ASCII control / residual → Other; high BMP fallback → Ideographic.
    assert_eq!(classify_char('\u{0001}'), CharClass::Other);
    assert_eq!(classify_char('♠'), CharClass::Ornaments);
    assert_eq!(classify_char('ß'), CharClass::ComplexWestern);
}

#[test]
fn char_em_width_ascii_half_ideograph_full() {
    assert_eq!(char_em_width('A'), 0.5);
    assert_eq!(char_em_width('9'), 0.5);
    assert_eq!(char_em_width(' '), 0.5);
    assert_eq!(char_em_width('あ'), 1.0);
    assert_eq!(char_em_width('漢'), 1.0);
    assert_eq!(char_em_width('。'), 1.0);
}

#[test]
fn break_line_kinsoku_no_period_at_line_start() {
    // Without kinsoku, max=3em on "あああ。" would yield ["あああ", "。"].
    let lines = break_line("あああ。", 3.0);
    assert!(
        lines.iter().all(|l| !l.starts_with('。')),
        "period must not start a line: {lines:?}"
    );
    assert_eq!(lines, vec!["ああ".to_string(), "あ。".to_string()]);
}

#[test]
fn break_line_avoids_open_at_line_end() {
    let lines = break_line("あ「いう", 2.0);
    assert!(
        lines.iter().all(|l| !l.ends_with('「')),
        "'「' must not end a line: {lines:?}"
    );
    assert_eq!(
        lines,
        vec!["あ".to_string(), "「い".to_string(), "う".to_string()]
    );
}

#[test]
fn break_line_edges_and_ascii_half_width() {
    assert!(break_line("", 4.0).is_empty());
    assert_eq!(break_line("東京", 0.0), vec!["東京".to_string()]);
    // Lone head-kinsoku char still emits.
    assert_eq!(break_line("。", 1.0), vec!["。".to_string()]);
    // ASCII letters are half-em → four fit in 2.0 em.
    assert_eq!(
        break_line("ABCD", 2.0),
        vec!["ABCD".to_string()]
    );
    assert_eq!(
        break_line("ABCDEF", 2.0),
        vec!["ABCD".to_string(), "EF".to_string()]
    );
}
