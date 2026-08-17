//! Direct Native v2 runtime for japanese record constructors (DN2-4).

use reciplexa_std::japanese::{
    break_pair_matrix_cell, classify_char, is_hangable, is_line_end_prohibited,
    is_line_head_prohibited, BreakOpportunity, CharClass,
};

use crate::domain_native::{
    DomainNativeOp, JapaneseClassesOp, JapaneseKihonOp, JapaneseLinebreakOp, JapaneseMarkupOp,
};
use crate::domain_native_failure::{take0, take1, take2, take3, take4, take5};
use crate::value::RuntimeValue;
use crate::EvalError;

pub fn call_japanese_constructor(
    op: DomainNativeOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DomainNativeOp::JapaneseClasses(sub) => call_japanese_classes(sub, args),
        DomainNativeOp::JapaneseLinebreak(sub) => call_japanese_linebreak(sub, args),
        DomainNativeOp::JapaneseKihon(sub) => call_japanese_kihon(sub, args),
        DomainNativeOp::JapaneseMarkup(sub) => call_japanese_markup(sub, args),
        other => Err(EvalError {
            message: format!("not a japanese constructor op: {other:?}"),
        }),
    }
}

const CLASSES: [(i128, &str, &str, &str); 30] = [
    (1, "cl-01", "opening-brackets", "始め括弧類"),
    (2, "cl-02", "closing-brackets", "終わり括弧類"),
    (3, "cl-03", "hyphens", "ハイフン類"),
    (4, "cl-04", "dividing-punctuation", "区切り約物"),
    (5, "cl-05", "middle-dots", "中点類"),
    (6, "cl-06", "full-stops", "句点類"),
    (7, "cl-07", "commas", "読点類"),
    (8, "cl-08", "inseparable", "分離禁止文字"),
    (9, "cl-09", "iteration-marks", "繰返し記号"),
    (10, "cl-10", "prolonged-sound-mark", "長音記号"),
    (11, "cl-11", "small-kana", "小書きの仮名"),
    (12, "cl-12", "prefixed-abbreviations", "前置省略記号"),
    (13, "cl-13", "postfixed-abbreviations", "後置省略記号"),
    (14, "cl-14", "spaces", "和字間隔等"),
    (15, "cl-15", "hiragana", "平仮名"),
    (16, "cl-16", "katakana", "片仮名"),
    (17, "cl-17", "math-symbols", "等号類"),
    (18, "cl-18", "grouped-numerals", "連数字"),
    (19, "cl-19", "ideographic", "漢字等"),
    (20, "cl-20", "numeric", "数字"),
    (21, "cl-21", "unit-symbols", "単位記号中の欧字等"),
    (22, "cl-22", "enclosed-alphanumerics", "囲み文字"),
    (23, "cl-23", "ornaments", "装飾文字"),
    (24, "cl-24", "simple-western", "単純な欧字"),
    (25, "cl-25", "complex-western", "複雑な欧字"),
    (26, "cl-26", "warichu-close", "割注終わり括弧類"),
    (27, "cl-27", "western-characters", "欧文用文字"),
    (28, "cl-28", "attached-western", "添付欧文"),
    (29, "cl-29", "warichu-open", "割注始め括弧類"),
    (30, "cl-30", "tate-chu-yoko", "縦中横"),
];

fn call_japanese_classes(
    op: JapaneseClassesOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        JapaneseClassesOp::Class => {
            let [id, code, name, name_ja] = take4(args, "`japanese/classes class`")?;
            Ok(jlreq_class_record(id, code, name, name_ja))
        }
        JapaneseClassesOp::AllClassIds | JapaneseClassesOp::AllClasses => {
            take0(args, &format!("`japanese/classes {:?}`", op))?;
            Ok(all_class_ids_list())
        }
        JapaneseClassesOp::ClassName => {
            let [class_id] = take1(args, "`japanese/classes class-name`")?;
            Ok(str_val(class_name(class_id_from_value(class_id)?)))
        }
        JapaneseClassesOp::ClassNameJa => {
            let [class_id] = take1(args, "`japanese/classes class-name-ja`")?;
            Ok(str_val(class_name_ja(class_id_from_value(class_id)?)))
        }
        JapaneseClassesOp::Cl01 => cl_record(0, args),
        JapaneseClassesOp::Cl02 => cl_record(1, args),
        JapaneseClassesOp::Cl03 => cl_record(2, args),
        JapaneseClassesOp::Cl04 => cl_record(3, args),
        JapaneseClassesOp::Cl05 => cl_record(4, args),
        JapaneseClassesOp::Cl06 => cl_record(5, args),
        JapaneseClassesOp::Cl07 => cl_record(6, args),
        JapaneseClassesOp::Cl08 => cl_record(7, args),
        JapaneseClassesOp::Cl09 => cl_record(8, args),
        JapaneseClassesOp::Cl10 => cl_record(9, args),
        JapaneseClassesOp::Cl11 => cl_record(10, args),
        JapaneseClassesOp::Cl12 => cl_record(11, args),
        JapaneseClassesOp::Cl13 => cl_record(12, args),
        JapaneseClassesOp::Cl14 => cl_record(13, args),
        JapaneseClassesOp::Cl15 => cl_record(14, args),
        JapaneseClassesOp::Cl16 => cl_record(15, args),
        JapaneseClassesOp::Cl17 => cl_record(16, args),
        JapaneseClassesOp::Cl18 => cl_record(17, args),
        JapaneseClassesOp::Cl19 => cl_record(18, args),
        JapaneseClassesOp::Cl20 => cl_record(19, args),
        JapaneseClassesOp::Cl21 => cl_record(20, args),
        JapaneseClassesOp::Cl22 => cl_record(21, args),
        JapaneseClassesOp::Cl23 => cl_record(22, args),
        JapaneseClassesOp::Cl24 => cl_record(23, args),
        JapaneseClassesOp::Cl25 => cl_record(24, args),
        JapaneseClassesOp::Cl26 => cl_record(25, args),
        JapaneseClassesOp::Cl27 => cl_record(26, args),
        JapaneseClassesOp::Cl28 => cl_record(27, args),
        JapaneseClassesOp::Cl29 => cl_record(28, args),
        JapaneseClassesOp::Cl30 => cl_record(29, args),
        JapaneseClassesOp::AdvanceEm => {
            let [class_id] = take1(args, "`japanese/classes advance-em`")?;
            Ok(num_val(advance_em(class_id_from_value(class_id)?)))
        }
        JapaneseClassesOp::IsSquareLetter => {
            let [class_id] = take1(args, "`japanese/classes is-square-letter?`")?;
            Ok(bool_val(is_square_letter(class_id_from_value(class_id)?)))
        }
        JapaneseClassesOp::IsPunctuationClass => {
            let [class_id] = take1(args, "`japanese/classes is-punctuation-class?`")?;
            Ok(bool_val(is_punctuation_class(class_id_from_value(
                class_id,
            )?)))
        }
        JapaneseClassesOp::IsKanaClass => {
            let [class_id] = take1(args, "`japanese/classes is-kana-class?`")?;
            Ok(bool_val(is_kana_class(class_id_from_value(class_id)?)))
        }
        JapaneseClassesOp::IsWesternClass => {
            let [class_id] = take1(args, "`japanese/classes is-western-class?`")?;
            Ok(bool_val(is_western_class(class_id_from_value(class_id)?)))
        }
    }
}

fn cl_record(index: usize, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    take0(args, &format!("`japanese/classes {}`", CLASSES[index].1))?;
    let (id, code, name, name_ja) = CLASSES[index];
    Ok(jlreq_class_static(id, code, name, name_ja))
}

fn call_japanese_linebreak(
    op: JapaneseLinebreakOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        JapaneseLinebreakOp::SampleLineHeadProhibited => constant_str(
            "」、。．，）〕］｝〉》』】！？ーぁぃぅぇぉっゃゅょァィゥェォッャュョヽヾゝゞ々",
            args,
            "`japanese/linebreak sample-line-head-prohibited`",
        ),
        JapaneseLinebreakOp::SampleLineEndProhibited => constant_str(
            "「『（〔［｛〈《￥＄￡＃",
            args,
            "`japanese/linebreak sample-line-end-prohibited`",
        ),
        JapaneseLinebreakOp::SampleInseparable => {
            constant_str("—…‥〳〴〵", args, "`japanese/linebreak sample-inseparable`")
        }
        JapaneseLinebreakOp::Opportunity => {
            let [kind, before, after, note] = take4(args, "`japanese/linebreak opportunity`")?;
            Ok(break_opportunity_record(kind, before, after, note))
        }
        JapaneseLinebreakOp::LineHeadProhibitedClass => {
            let [class_id] = take1(args, "`japanese/linebreak line-head-prohibited-class?`")?;
            let id = class_id_from_value(class_id)?;
            Ok(bool_val(
                CharClass::from_id(id)
                    .map(is_line_head_prohibited)
                    .unwrap_or(false),
            ))
        }
        JapaneseLinebreakOp::LineEndProhibitedClass => {
            let [class_id] = take1(args, "`japanese/linebreak line-end-prohibited-class?`")?;
            let id = class_id_from_value(class_id)?;
            Ok(bool_val(
                CharClass::from_id(id)
                    .map(is_line_end_prohibited)
                    .unwrap_or(false),
            ))
        }
        JapaneseLinebreakOp::InseparablePair => {
            let [before, after] = take2(args, "`japanese/linebreak inseparable-pair?`")?;
            let before_id = class_id_from_value(before)?;
            let after_id = class_id_from_value(after)?;
            Ok(bool_val(
                CharClass::from_id(before_id).is_some()
                    && CharClass::from_id(after_id).is_some()
                    && break_pair_matrix_cell(before_id, after_id) == BreakOpportunity::Inseparable,
            ))
        }
        JapaneseLinebreakOp::PairRule => {
            let [before, after, kind, note] = take4(args, "`japanese/linebreak pair-rule`")?;
            Ok(pair_rule_record(before, after, kind, note))
        }
        JapaneseLinebreakOp::SamplePairRules => {
            take0(args, "`japanese/linebreak sample-pair-rules`")?;
            Ok(sample_pair_rules())
        }
        JapaneseLinebreakOp::BreakBetween => {
            let [before, after] = take2(args, "`japanese/linebreak break-between`")?;
            let before_id = class_id_from_value(before)?;
            let after_id = class_id_from_value(after)?;
            let kind = break_pair_matrix_cell(before_id, after_id);
            Ok(break_opportunity_record(
                &str_val(kind.as_str()),
                before,
                after,
                &str_val("std break_pair_matrix"),
            ))
        }
        JapaneseLinebreakOp::HangableClass => {
            let [class_id] = take1(args, "`japanese/linebreak hangable-class?`")?;
            let id = class_id_from_value(class_id)?;
            Ok(bool_val(
                CharClass::from_id(id).map(is_hangable).unwrap_or(false),
            ))
        }
        JapaneseLinebreakOp::NumericBeforeCloseProhibited => {
            let [before, after] = take2(
                args,
                "`japanese/linebreak numeric-before-close-prohibited?`",
            )?;
            let before_id = class_id_from_value(before)?;
            let after_id = class_id_from_value(after)?;
            Ok(bool_val(
                before_id == 20 && (after_id == 1 || after_id == 2),
            ))
        }
        JapaneseLinebreakOp::KinsokuProfile => {
            take0(args, "`japanese/linebreak kinsoku-profile`")?;
            Ok(kinsoku_profile())
        }
        JapaneseLinebreakOp::ClassifySample => {
            let [glyph] = take1(args, "`japanese/linebreak classify-sample`")?;
            match glyph {
                RuntimeValue::String(s) => {
                    let ch = s.chars().next().ok_or_else(|| EvalError {
                        message: "`japanese/linebreak classify-sample`: empty string".into(),
                    })?;
                    Ok(int_val(classify_char(ch).id() as i128))
                }
                other => Err(EvalError {
                    message: format!(
                        "`japanese/linebreak classify-sample`: expected string, got {other}"
                    ),
                }),
            }
        }
    }
}

fn call_japanese_kihon(
    op: JapaneseKihonOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        JapaneseKihonOp::WritingModeHorizontal => constant_str(
            "horizontal-tb",
            args,
            "`japanese/kihon writing-mode-horizontal`",
        ),
        JapaneseKihonOp::WritingModeVertical => constant_str(
            "vertical-rl",
            args,
            "`japanese/kihon writing-mode-vertical`",
        ),
        JapaneseKihonOp::LineRateSolid => {
            constant_num(1.0, args, "`japanese/kihon line-rate-solid`")
        }
        JapaneseKihonOp::LineRateCompact => {
            constant_num(1.2, args, "`japanese/kihon line-rate-compact`")
        }
        JapaneseKihonOp::LineRateDefault => {
            constant_num(1.5, args, "`japanese/kihon line-rate-default`")
        }
        JapaneseKihonOp::LineRateRelaxed => {
            constant_num(1.7, args, "`japanese/kihon line-rate-relaxed`")
        }
        JapaneseKihonOp::LineRateLoose => {
            constant_num(2.0, args, "`japanese/kihon line-rate-loose`")
        }
        JapaneseKihonOp::SolidSetting => {
            constant_bool(true, args, "`japanese/kihon solid-setting`")
        }
        JapaneseKihonOp::CharacterFrame => {
            let [size_em] = take1(args, "`japanese/kihon character-frame`")?;
            Ok(record(vec![
                ("tag".into(), str_val("jlreq-character-frame")),
                ("size-em".into(), size_em.clone()),
                ("shape".into(), str_val("square")),
            ]))
        }
        JapaneseKihonOp::LineMetrics => {
            let [char_size_em, line_rate] = take2(args, "`japanese/kihon line-metrics`")?;
            let cs = as_f64(char_size_em, "char-size-em")?;
            let lr = as_f64(line_rate, "line-rate")?;
            Ok(record(vec![
                ("tag".into(), str_val("jlreq-line-metrics")),
                ("char-size-em".into(), char_size_em.clone()),
                ("line-rate".into(), line_rate.clone()),
                ("line-pitch-em".into(), num_val(cs * lr)),
                ("line-gap-em".into(), num_val(cs * (lr - 1.0))),
            ]))
        }
        JapaneseKihonOp::KihonHanmen => {
            let [char_size_em, line_length, line_count, line_rate, mode] =
                take5(args, "`japanese/kihon kihon-hanmen`")?;
            let cs = as_f64(char_size_em, "char-size-em")?;
            let ll = as_f64(line_length, "line-length")?;
            let lc = as_f64(line_count, "line-count")?;
            let lr = as_f64(line_rate, "line-rate")?;
            let line_gap = cs * (lr - 1.0);
            Ok(record(vec![
                ("tag".into(), str_val("kihon-hanmen")),
                ("char-size-em".into(), char_size_em.clone()),
                ("line-length".into(), line_length.clone()),
                ("line-count".into(), line_count.clone()),
                ("line-rate".into(), line_rate.clone()),
                ("line-gap-em".into(), num_val(line_gap)),
                ("writing-mode".into(), mode.clone()),
                ("solid-setting".into(), bool_val(true)),
                ("hanmen-inline-em".into(), num_val(cs * ll)),
                (
                    "hanmen-block-em".into(),
                    num_val(cs * lc + line_gap * (lc - 1.0)),
                ),
            ]))
        }
        JapaneseKihonOp::DefaultHorizontalKihon => {
            take0(args, "`japanese/kihon default-horizontal-kihon`")?;
            Ok(default_horizontal_kihon())
        }
        JapaneseKihonOp::DefaultVerticalKihon => {
            take0(args, "`japanese/kihon default-vertical-kihon`")?;
            Ok(default_vertical_kihon())
        }
        JapaneseKihonOp::HeadingBandEm => {
            let [kihon, lines] = take2(args, "`japanese/kihon heading-band-em`")?;
            let cs = as_f64(&field_value(kihon, "char-size-em")?, "char-size-em")?;
            let gap = as_f64(&field_value(kihon, "line-gap-em")?, "line-gap-em")?;
            let n = as_f64(lines, "lines")?;
            Ok(num_val(cs * n + gap * (n - 1.0)))
        }
        JapaneseKihonOp::IndentEm => {
            let [kihon, chars] = take2(args, "`japanese/kihon indent-em`")?;
            let cs = as_f64(&field_value(kihon, "char-size-em")?, "char-size-em")?;
            let n = as_f64(chars, "chars")?;
            Ok(num_val(cs * n))
        }
        JapaneseKihonOp::TrimSize => {
            let [width_mm, height_mm] = take2(args, "`japanese/kihon trim-size`")?;
            Ok(record(vec![
                ("tag".into(), str_val("jlreq-trim-size")),
                ("width-mm".into(), width_mm.clone()),
                ("height-mm".into(), height_mm.clone()),
            ]))
        }
        JapaneseKihonOp::A5Trim => trim_record(148, 210, args, "a5-trim"),
        JapaneseKihonOp::B5JisTrim => trim_record(182, 257, args, "b5-jis-trim"),
        JapaneseKihonOp::A4Trim => trim_record(210, 297, args, "a4-trim"),
        JapaneseKihonOp::Margins => {
            let [top, right, bottom, left] = take4(args, "`japanese/kihon margins`")?;
            Ok(record(vec![
                ("tag".into(), str_val("jlreq-margins")),
                ("top-mm".into(), top.clone()),
                ("right-mm".into(), right.clone()),
                ("bottom-mm".into(), bottom.clone()),
                ("left-mm".into(), left.clone()),
            ]))
        }
        JapaneseKihonOp::PlaceHanmen => {
            let [trim, margins_rec, hanmen] = take3(args, "`japanese/kihon place-hanmen`")?;
            Ok(record(vec![
                ("tag".into(), str_val("jlreq-hanmen-placement")),
                ("trim".into(), trim.clone()),
                ("margins".into(), margins_rec.clone()),
                ("hanmen".into(), hanmen.clone()),
                (
                    "note".into(),
                    str_val("geometry stub — lower does not yet consume kihon"),
                ),
            ]))
        }
        JapaneseKihonOp::ColumnCountOne => {
            constant_int(1, args, "`japanese/kihon column-count-one`")
        }
        JapaneseKihonOp::ColumnCountTwo => {
            constant_int(2, args, "`japanese/kihon column-count-two`")
        }
        JapaneseKihonOp::MultiColumn => {
            let [hanmen, columns, gutter_em] = take3(args, "`japanese/kihon multi-column`")?;
            Ok(record(vec![
                ("tag".into(), str_val("jlreq-multi-column")),
                ("hanmen".into(), hanmen.clone()),
                ("columns".into(), columns.clone()),
                ("gutter-em".into(), gutter_em.clone()),
                ("completeness".into(), str_val("stub")),
            ]))
        }
        JapaneseKihonOp::VerticalFlow => {
            let [body] = take1(args, "`japanese/kihon vertical-flow`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-vertical-flow")),
                ("body".into(), body.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
                ("note".into(), str_val("tategaki block stub")),
            ]))
        }
        JapaneseKihonOp::TateDigits => {
            let [digits] = take1(args, "`japanese/kihon tate-digits`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-tate-digits")),
                ("digits".into(), digits.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
            ]))
        }
        JapaneseKihonOp::VerticalStack => {
            let [items] = take1(args, "`japanese/kihon vertical-stack`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-vertical-stack")),
                ("items".into(), items.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
            ]))
        }
        JapaneseKihonOp::TateChuYokoSpan => {
            let [body] = take1(args, "`japanese/kihon tate-chu-yoko-span`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-tate-chu-yoko")),
                ("body".into(), body.clone()),
                ("kind".into(), str_val("inline-span")),
            ]))
        }
        JapaneseKihonOp::VerticalText => {
            let [x, y, size, content] = take4(args, "`japanese/kihon vertical-text`")?;
            Ok(record(vec![
                ("tag".into(), str_val("text")),
                ("x".into(), x.clone()),
                ("y".into(), y.clone()),
                ("size".into(), size.clone()),
                ("content".into(), content.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
                ("placement".into(), str_val("tategaki-stub")),
            ]))
        }
        JapaneseKihonOp::PlaceVerticalText => {
            let [x, y, char_size_em, content] =
                take4(args, "`japanese/kihon place-vertical-text`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-vertical-text-placement")),
                ("x".into(), x.clone()),
                ("y".into(), y.clone()),
                ("char-size-em".into(), char_size_em.clone()),
                ("content".into(), content.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
                (
                    "graphics-text".into(),
                    record(vec![
                        ("tag".into(), str_val("text")),
                        ("x".into(), x.clone()),
                        ("y".into(), y.clone()),
                        ("size".into(), char_size_em.clone()),
                        ("content".into(), content.clone()),
                        ("writing-mode".into(), str_val("vertical-rl")),
                    ]),
                ),
            ]))
        }
        JapaneseKihonOp::VerticalTextStack => {
            let [origin_x, origin_y, char_size_em, items] =
                take4(args, "`japanese/kihon vertical-text-stack`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-vertical-text-stack")),
                ("origin-x".into(), origin_x.clone()),
                ("origin-y".into(), origin_y.clone()),
                ("char-size-em".into(), char_size_em.clone()),
                ("items".into(), items.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
                (
                    "note".into(),
                    str_val("stack of vertical-text records; lower via graphics bridge"),
                ),
            ]))
        }
    }
}

fn call_japanese_markup(
    op: JapaneseMarkupOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        JapaneseMarkupOp::Heading => {
            let [title] = take1(args, "`japanese/markup heading`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-heading")),
                ("title".into(), title.clone()),
            ]))
        }
        JapaneseMarkupOp::Section => {
            let [title] = take1(args, "`japanese/markup section`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-section")),
                ("title".into(), title.clone()),
            ]))
        }
        JapaneseMarkupOp::Paragraph => {
            let [body] = take1(args, "`japanese/markup paragraph`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-paragraph")),
                ("body".into(), body.clone()),
                ("indent-em".into(), int_val(1)),
            ]))
        }
        JapaneseMarkupOp::Note => {
            let [body] = take1(args, "`japanese/markup note`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-note")),
                ("body".into(), body.clone()),
            ]))
        }
        JapaneseMarkupOp::Emphasis => {
            let [body] = take1(args, "`japanese/markup emphasis`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-emphasis")),
                ("body".into(), body.clone()),
            ]))
        }
        JapaneseMarkupOp::Warn => {
            let [body] = take1(args, "`japanese/markup warn`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-warn")),
                ("body".into(), body.clone()),
            ]))
        }
        JapaneseMarkupOp::Todo => {
            let [body] = take1(args, "`japanese/markup todo`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-todo")),
                ("body".into(), body.clone()),
            ]))
        }
        JapaneseMarkupOp::Ruby => {
            let [base, annotation] = take2(args, "`japanese/markup ruby`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-ruby")),
                ("base".into(), base.clone()),
                ("annotation".into(), annotation.clone()),
                ("kind".into(), str_val("simple")),
            ]))
        }
        JapaneseMarkupOp::JukugoRuby => {
            let [base, annotation] = take2(args, "`japanese/markup jukugo-ruby`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-ruby")),
                ("base".into(), base.clone()),
                ("annotation".into(), annotation.clone()),
                ("kind".into(), str_val("jukugo")),
            ]))
        }
        JapaneseMarkupOp::TateChuYoko => {
            let [body] = take1(args, "`japanese/markup tate-chu-yoko`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-tate-chu-yoko")),
                ("body".into(), body.clone()),
            ]))
        }
        JapaneseMarkupOp::TategakiParagraph => {
            let [body] = take1(args, "`japanese/markup tategaki-paragraph`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-tategaki-paragraph")),
                ("body".into(), body.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
            ]))
        }
        JapaneseMarkupOp::TategakiText => {
            let [x, y, size, content] = take4(args, "`japanese/markup tategaki-text`")?;
            Ok(record(vec![
                ("tag".into(), str_val("text")),
                ("x".into(), x.clone()),
                ("y".into(), y.clone()),
                ("size".into(), size.clone()),
                ("content".into(), content.clone()),
                ("writing-mode".into(), str_val("vertical-rl")),
                ("source".into(), str_val("ja-markup-tategaki")),
            ]))
        }
        JapaneseMarkupOp::MarkupBridge => {
            let [title, markup_source] = take2(args, "`japanese/markup markup-bridge`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-markup-bridge")),
                ("title".into(), title.clone()),
                ("markup-source".into(), markup_source.clone()),
                (
                    "note".into(),
                    str_val("package mirror of SYN @-markup examples/markup_ja.rpx"),
                ),
            ]))
        }
        JapaneseMarkupOp::DocWithMarkup => {
            let [title, markup_source, children] =
                take3(args, "`japanese/markup doc-with-markup`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-doc")),
                ("title".into(), title.clone()),
                ("markup-source".into(), markup_source.clone()),
                ("children".into(), children.clone()),
                ("profile".into(), str_val("jlreq-oriented-stub")),
            ]))
        }
        JapaneseMarkupOp::Ul => {
            let [items] = take1(args, "`japanese/markup ul`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-ul")),
                ("items".into(), items.clone()),
            ]))
        }
        JapaneseMarkupOp::Doc => {
            let [title, children] = take2(args, "`japanese/markup doc`")?;
            Ok(record(vec![
                ("tag".into(), str_val("ja-doc")),
                ("title".into(), title.clone()),
                ("children".into(), children.clone()),
                ("profile".into(), str_val("jlreq-oriented-stub")),
            ]))
        }
    }
}

fn jlreq_class_record(
    id: &RuntimeValue,
    code: &RuntimeValue,
    name: &RuntimeValue,
    name_ja: &RuntimeValue,
) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("jlreq-class")),
        ("id".into(), id.clone()),
        ("code".into(), code.clone()),
        ("name".into(), name.clone()),
        ("name-ja".into(), name_ja.clone()),
    ])
}

fn jlreq_class_static(id: i128, code: &str, name: &str, name_ja: &str) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("jlreq-class")),
        ("id".into(), int_val(id)),
        ("code".into(), str_val(code)),
        ("name".into(), str_val(name)),
        ("name-ja".into(), str_val(name_ja)),
    ])
}

fn all_class_ids_list() -> RuntimeValue {
    cons_list_from((1..=30).map(int_val))
}

fn class_name(class_id: u8) -> &'static str {
    match class_id {
        1 => "opening-brackets",
        2 => "closing-brackets",
        3 => "hyphens",
        4 => "dividing-punctuation",
        5 => "middle-dots",
        6 => "full-stops",
        7 => "commas",
        8 => "inseparable",
        9 => "iteration-marks",
        10 => "prolonged-sound-mark",
        11 => "small-kana",
        12 => "prefixed-abbreviations",
        13 => "postfixed-abbreviations",
        14 => "spaces",
        15 => "hiragana",
        16 => "katakana",
        17 => "math-symbols",
        18 => "grouped-numerals",
        19 => "ideographic",
        20 => "numeric",
        21 => "unit-symbols",
        22 => "enclosed-alphanumerics",
        23 => "ornaments",
        24 => "simple-western",
        25 => "complex-western",
        26 => "warichu-close",
        27 => "western-characters",
        28 => "attached-western",
        29 => "warichu-open",
        30 => "tate-chu-yoko",
        _ => "other",
    }
}

fn class_name_ja(class_id: u8) -> &'static str {
    match class_id {
        1 => "始め括弧類",
        2 => "終わり括弧類",
        3 => "ハイフン類",
        4 => "区切り約物",
        5 => "中点類",
        6 => "句点類",
        7 => "読点類",
        8 => "分離禁止文字",
        9 => "繰返し記号",
        10 => "長音記号",
        11 => "小書きの仮名",
        12 => "前置省略記号",
        13 => "後置省略記号",
        14 => "和字間隔等",
        15 => "平仮名",
        16 => "片仮名",
        17 => "等号類",
        18 => "連数字",
        19 => "漢字等",
        20 => "数字",
        21 => "単位記号中の欧字等",
        22 => "囲み文字",
        23 => "装飾文字",
        24 => "単純な欧字",
        25 => "複雑な欧字",
        26 => "割注終わり括弧類",
        27 => "欧文用文字",
        28 => "添付欧文",
        29 => "割注始め括弧類",
        30 => "縦中横",
        _ => "その他",
    }
}

fn advance_em(class_id: u8) -> f64 {
    match class_id {
        1 | 2 | 5 | 6 | 7 | 14 => 0.5,
        26 => 0.25,
        _ => 1.0,
    }
}

fn is_square_letter(class_id: u8) -> bool {
    matches!(class_id, 15 | 16 | 19 | 20)
}

fn is_punctuation_class(class_id: u8) -> bool {
    matches!(class_id, 1..=7)
}

fn is_kana_class(class_id: u8) -> bool {
    matches!(class_id, 11 | 15 | 16)
}

fn is_western_class(class_id: u8) -> bool {
    matches!(class_id, 21 | 24 | 25 | 27 | 28)
}

fn break_opportunity_record(
    kind: &RuntimeValue,
    before: &RuntimeValue,
    after: &RuntimeValue,
    note: &RuntimeValue,
) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("jlreq-break-opportunity")),
        ("kind".into(), kind.clone()),
        ("before".into(), before.clone()),
        ("after".into(), after.clone()),
        ("note".into(), note.clone()),
    ])
}

fn pair_rule_record(
    before: &RuntimeValue,
    after: &RuntimeValue,
    kind: &RuntimeValue,
    note: &RuntimeValue,
) -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("jlreq-pair-rule")),
        ("before".into(), before.clone()),
        ("after".into(), after.clone()),
        ("kind".into(), kind.clone()),
        ("note".into(), note.clone()),
    ])
}

const SAMPLE_PAIR_RULES: [(i128, i128, &str, &str); 27] = [
    (1, 15, "prohibited", "open + letter"),
    (15, 2, "prohibited", "letter + close"),
    (15, 6, "prohibited", "letter + full-stop"),
    (15, 7, "prohibited", "letter + comma"),
    (8, 8, "inseparable", "cl-08"),
    (19, 19, "allowed", "ideograph run"),
    (27, 27, "allowed", "western run"),
    (15, 27, "allowed", "JP/western boundary"),
    (19, 20, "allowed", "ideograph + digit"),
    (20, 13, "prohibited", "digit + postfix"),
    (20, 1, "prohibited", "digit + open"),
    (20, 2, "prohibited", "digit + close"),
    (20, 21, "prohibited", "digit + unit"),
    (12, 20, "prohibited", "prefix + digit"),
    (24, 27, "inseparable", "simple×western run"),
    (8, 19, "inseparable", "cl-08 × ideograph"),
    (30, 30, "allowed", "tate-chu-yoko corner"),
    (14, 15, "allowed", "space + hiragana"),
    (16, 10, "prohibited", "katakana + prolonged"),
    (24, 24, "inseparable", "simple western run"),
    (6, 1, "prohibited", "full-stop before open"),
    (7, 1, "prohibited", "comma before open"),
    (2, 15, "prohibited", "close before hiragana"),
    (2, 19, "prohibited", "close before ideograph"),
    (19, 2, "prohibited", "ideograph before close"),
    (27, 27, "allowed", "latin run"),
    (30, 19, "allowed", "tate-chu-yoko + ideograph"),
];

fn sample_pair_rules() -> RuntimeValue {
    cons_list_from(SAMPLE_PAIR_RULES.iter().map(|(before, after, kind, note)| {
        record(vec![
            ("tag".into(), str_val("jlreq-pair-rule")),
            ("before".into(), int_val(*before)),
            ("after".into(), int_val(*after)),
            ("kind".into(), str_val(kind)),
            ("note".into(), str_val(note)),
        ])
    }))
}

fn kinsoku_profile() -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("jlreq-kinsoku-profile")),
        (
            "line-head-prohibited-classes".into(),
            cons_list_from([2, 6, 7, 9, 10, 11, 29].map(int_val)),
        ),
        (
            "line-end-prohibited-classes".into(),
            cons_list_from([1, 12, 28].map(int_val)),
        ),
        (
            "hangable-classes".into(),
            cons_list_from([6, 7].map(int_val)),
        ),
        (
            "sample-line-head-prohibited".into(),
            str_val(
                "」、。．，）〕］｝〉》』】！？ーぁぃぅぇぉっゃゅょァィゥェォッャュョヽヾゝゞ々",
            ),
        ),
        (
            "sample-line-end-prohibited".into(),
            str_val("「『（〔［｛〈《￥＄￡＃"),
        ),
        ("sample-inseparable".into(), str_val("—…‥〳〴〵")),
        ("completeness".into(), str_val("subset-stub")),
    ])
}

fn default_horizontal_kihon() -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("kihon-hanmen")),
        ("char-size-em".into(), num_val(1.0)),
        ("line-length".into(), int_val(40)),
        ("line-count".into(), int_val(30)),
        ("line-rate".into(), num_val(1.5)),
        ("line-gap-em".into(), num_val(0.5)),
        ("writing-mode".into(), str_val("horizontal-tb")),
        ("solid-setting".into(), bool_val(true)),
        ("hanmen-inline-em".into(), num_val(40.0)),
        ("hanmen-block-em".into(), num_val(44.5)),
    ])
}

fn default_vertical_kihon() -> RuntimeValue {
    record(vec![
        ("tag".into(), str_val("kihon-hanmen")),
        ("char-size-em".into(), num_val(1.0)),
        ("line-length".into(), int_val(35)),
        ("line-count".into(), int_val(20)),
        ("line-rate".into(), num_val(1.5)),
        ("line-gap-em".into(), num_val(0.5)),
        ("writing-mode".into(), str_val("vertical-rl")),
        ("solid-setting".into(), bool_val(true)),
        ("hanmen-inline-em".into(), num_val(35.0)),
        ("hanmen-block-em".into(), num_val(29.5)),
    ])
}

fn trim_record(
    width: i128,
    height: i128,
    args: &[RuntimeValue],
    ctx: &str,
) -> Result<RuntimeValue, EvalError> {
    take0(args, &format!("`japanese/kihon {ctx}`"))?;
    Ok(record(vec![
        ("tag".into(), str_val("jlreq-trim-size")),
        ("width-mm".into(), int_val(width)),
        ("height-mm".into(), int_val(height)),
    ]))
}

fn constant_str(value: &str, args: &[RuntimeValue], ctx: &str) -> Result<RuntimeValue, EvalError> {
    take0(args, ctx)?;
    Ok(str_val(value))
}

fn constant_num(value: f64, args: &[RuntimeValue], ctx: &str) -> Result<RuntimeValue, EvalError> {
    take0(args, ctx)?;
    Ok(num_val(value))
}

fn constant_int(value: i128, args: &[RuntimeValue], ctx: &str) -> Result<RuntimeValue, EvalError> {
    take0(args, ctx)?;
    Ok(int_val(value))
}

fn constant_bool(value: bool, args: &[RuntimeValue], ctx: &str) -> Result<RuntimeValue, EvalError> {
    take0(args, ctx)?;
    Ok(bool_val(value))
}

fn class_id_from_value(v: &RuntimeValue) -> Result<u8, EvalError> {
    match v {
        RuntimeValue::Int(n) if (0..=255).contains(n) => Ok(*n as u8),
        RuntimeValue::Number(n) | RuntimeValue::F64(n)
            if n.fract() == 0.0 && (0.0..=255.0).contains(n) =>
        {
            Ok(*n as u8)
        }
        other => Err(EvalError {
            message: format!("expected class id (0..255 int), got {other}"),
        }),
    }
}

fn as_f64(v: &RuntimeValue, ctx: &str) -> Result<f64, EvalError> {
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Ok(*n),
        RuntimeValue::Int(n) => Ok(*n as f64),
        other => Err(EvalError {
            message: format!("{ctx}: expected number, got {other}"),
        }),
    }
}

fn field_value(rec: &RuntimeValue, key: &str) -> Result<RuntimeValue, EvalError> {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| EvalError {
                message: format!("record missing field `{key}`"),
            }),
        other => Err(EvalError {
            message: format!("expected record, got {other}"),
        }),
    }
}

fn cons_list_from(items: impl IntoIterator<Item = RuntimeValue>) -> RuntimeValue {
    let items: Vec<_> = items.into_iter().collect();
    let mut acc = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for item in items.into_iter().rev() {
        acc = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(RuntimeValue::Record(vec![
                ("head".into(), item),
                ("tail".into(), acc),
            ]))),
        };
    }
    acc
}

fn record(fields: Vec<(String, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(fields)
}

fn str_val(s: &str) -> RuntimeValue {
    RuntimeValue::String(s.into())
}

fn int_val(n: i128) -> RuntimeValue {
    RuntimeValue::Int(n)
}

fn num_val(n: f64) -> RuntimeValue {
    RuntimeValue::Number(n)
}

fn bool_val(b: bool) -> RuntimeValue {
    RuntimeValue::Bool(b)
}
