//! Direct Native v2 typed exports for japanese constructors (DN2-4).

use std::collections::BTreeMap;

use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_eval::domain_native::{
    DomainNativeOp, JapaneseClassesOp, JapaneseKihonOp, JapaneseLinebreakOp, JapaneseMarkupOp,
};

use crate::domain_native::{DomainNativeExport, DomainNativeModule};

fn dynamic() -> CoreType {
    CoreType::Dynamic(Box::new(CoreType::Any))
}

fn nullary_dynamic() -> CoreType {
    CoreType::Fun {
        args: vec![],
        ret: Box::new(dynamic()),
        effects: EffectRow::default(),
    }
}

fn fun_n(n: usize) -> CoreType {
    CoreType::Fun {
        args: vec![dynamic(); n],
        ret: Box::new(dynamic()),
        effects: EffectRow::default(),
    }
}

fn insert(
    exports: &mut BTreeMap<String, DomainNativeExport>,
    name: &str,
    ty: CoreType,
    op: DomainNativeOp,
) {
    exports.insert(name.into(), DomainNativeExport::new(name, ty, op));
}

/// Attach typed DN2 exports for `japanese/classes`.
pub fn populate_japanese_classes_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::JapaneseClasses(sub);
    insert(
        &mut exports,
        "class",
        fun_n(4),
        op(JapaneseClassesOp::Class),
    );
    insert(
        &mut exports,
        "all-class-ids",
        nullary_dynamic(),
        op(JapaneseClassesOp::AllClassIds),
    );
    insert(
        &mut exports,
        "all-classes",
        nullary_dynamic(),
        op(JapaneseClassesOp::AllClasses),
    );
    insert(
        &mut exports,
        "class-name",
        fun_n(1),
        op(JapaneseClassesOp::ClassName),
    );
    insert(
        &mut exports,
        "class-name-ja",
        fun_n(1),
        op(JapaneseClassesOp::ClassNameJa),
    );
    insert(
        &mut exports,
        "cl-01",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl01),
    );
    insert(
        &mut exports,
        "cl-02",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl02),
    );
    insert(
        &mut exports,
        "cl-03",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl03),
    );
    insert(
        &mut exports,
        "cl-04",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl04),
    );
    insert(
        &mut exports,
        "cl-05",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl05),
    );
    insert(
        &mut exports,
        "cl-06",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl06),
    );
    insert(
        &mut exports,
        "cl-07",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl07),
    );
    insert(
        &mut exports,
        "cl-08",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl08),
    );
    insert(
        &mut exports,
        "cl-09",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl09),
    );
    insert(
        &mut exports,
        "cl-10",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl10),
    );
    insert(
        &mut exports,
        "cl-11",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl11),
    );
    insert(
        &mut exports,
        "cl-12",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl12),
    );
    insert(
        &mut exports,
        "cl-13",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl13),
    );
    insert(
        &mut exports,
        "cl-14",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl14),
    );
    insert(
        &mut exports,
        "cl-15",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl15),
    );
    insert(
        &mut exports,
        "cl-16",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl16),
    );
    insert(
        &mut exports,
        "cl-17",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl17),
    );
    insert(
        &mut exports,
        "cl-18",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl18),
    );
    insert(
        &mut exports,
        "cl-19",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl19),
    );
    insert(
        &mut exports,
        "cl-20",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl20),
    );
    insert(
        &mut exports,
        "cl-21",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl21),
    );
    insert(
        &mut exports,
        "cl-22",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl22),
    );
    insert(
        &mut exports,
        "cl-23",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl23),
    );
    insert(
        &mut exports,
        "cl-24",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl24),
    );
    insert(
        &mut exports,
        "cl-25",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl25),
    );
    insert(
        &mut exports,
        "cl-26",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl26),
    );
    insert(
        &mut exports,
        "cl-27",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl27),
    );
    insert(
        &mut exports,
        "cl-28",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl28),
    );
    insert(
        &mut exports,
        "cl-29",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl29),
    );
    insert(
        &mut exports,
        "cl-30",
        nullary_dynamic(),
        op(JapaneseClassesOp::Cl30),
    );
    insert(
        &mut exports,
        "advance-em",
        fun_n(1),
        op(JapaneseClassesOp::AdvanceEm),
    );
    insert(
        &mut exports,
        "is-square-letter?",
        fun_n(1),
        op(JapaneseClassesOp::IsSquareLetter),
    );
    insert(
        &mut exports,
        "is-punctuation-class?",
        fun_n(1),
        op(JapaneseClassesOp::IsPunctuationClass),
    );
    insert(
        &mut exports,
        "is-kana-class?",
        fun_n(1),
        op(JapaneseClassesOp::IsKanaClass),
    );
    insert(
        &mut exports,
        "is-western-class?",
        fun_n(1),
        op(JapaneseClassesOp::IsWesternClass),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `japanese/linebreak`.
pub fn populate_japanese_linebreak_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::JapaneseLinebreak(sub);
    insert(
        &mut exports,
        "sample-line-head-prohibited",
        nullary_dynamic(),
        op(JapaneseLinebreakOp::SampleLineHeadProhibited),
    );
    insert(
        &mut exports,
        "sample-line-end-prohibited",
        nullary_dynamic(),
        op(JapaneseLinebreakOp::SampleLineEndProhibited),
    );
    insert(
        &mut exports,
        "sample-inseparable",
        nullary_dynamic(),
        op(JapaneseLinebreakOp::SampleInseparable),
    );
    insert(
        &mut exports,
        "opportunity",
        fun_n(4),
        op(JapaneseLinebreakOp::Opportunity),
    );
    insert(
        &mut exports,
        "line-head-prohibited-class?",
        fun_n(1),
        op(JapaneseLinebreakOp::LineHeadProhibitedClass),
    );
    insert(
        &mut exports,
        "line-end-prohibited-class?",
        fun_n(1),
        op(JapaneseLinebreakOp::LineEndProhibitedClass),
    );
    insert(
        &mut exports,
        "inseparable-pair?",
        fun_n(2),
        op(JapaneseLinebreakOp::InseparablePair),
    );
    insert(
        &mut exports,
        "pair-rule",
        fun_n(4),
        op(JapaneseLinebreakOp::PairRule),
    );
    insert(
        &mut exports,
        "sample-pair-rules",
        nullary_dynamic(),
        op(JapaneseLinebreakOp::SamplePairRules),
    );
    insert(
        &mut exports,
        "break-between",
        fun_n(2),
        op(JapaneseLinebreakOp::BreakBetween),
    );
    insert(
        &mut exports,
        "hangable-class?",
        fun_n(1),
        op(JapaneseLinebreakOp::HangableClass),
    );
    insert(
        &mut exports,
        "numeric-before-close-prohibited?",
        fun_n(2),
        op(JapaneseLinebreakOp::NumericBeforeCloseProhibited),
    );
    insert(
        &mut exports,
        "kinsoku-profile",
        nullary_dynamic(),
        op(JapaneseLinebreakOp::KinsokuProfile),
    );
    insert(
        &mut exports,
        "classify-sample",
        fun_n(1),
        op(JapaneseLinebreakOp::ClassifySample),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `japanese/kihon`.
pub fn populate_japanese_kihon_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::JapaneseKihon(sub);
    insert(
        &mut exports,
        "writing-mode-horizontal",
        nullary_dynamic(),
        op(JapaneseKihonOp::WritingModeHorizontal),
    );
    insert(
        &mut exports,
        "writing-mode-vertical",
        nullary_dynamic(),
        op(JapaneseKihonOp::WritingModeVertical),
    );
    insert(
        &mut exports,
        "line-rate-solid",
        nullary_dynamic(),
        op(JapaneseKihonOp::LineRateSolid),
    );
    insert(
        &mut exports,
        "line-rate-compact",
        nullary_dynamic(),
        op(JapaneseKihonOp::LineRateCompact),
    );
    insert(
        &mut exports,
        "line-rate-default",
        nullary_dynamic(),
        op(JapaneseKihonOp::LineRateDefault),
    );
    insert(
        &mut exports,
        "line-rate-relaxed",
        nullary_dynamic(),
        op(JapaneseKihonOp::LineRateRelaxed),
    );
    insert(
        &mut exports,
        "line-rate-loose",
        nullary_dynamic(),
        op(JapaneseKihonOp::LineRateLoose),
    );
    insert(
        &mut exports,
        "solid-setting",
        nullary_dynamic(),
        op(JapaneseKihonOp::SolidSetting),
    );
    insert(
        &mut exports,
        "character-frame",
        fun_n(1),
        op(JapaneseKihonOp::CharacterFrame),
    );
    insert(
        &mut exports,
        "line-metrics",
        fun_n(2),
        op(JapaneseKihonOp::LineMetrics),
    );
    insert(
        &mut exports,
        "kihon-hanmen",
        fun_n(5),
        op(JapaneseKihonOp::KihonHanmen),
    );
    insert(
        &mut exports,
        "default-horizontal-kihon",
        nullary_dynamic(),
        op(JapaneseKihonOp::DefaultHorizontalKihon),
    );
    insert(
        &mut exports,
        "default-vertical-kihon",
        nullary_dynamic(),
        op(JapaneseKihonOp::DefaultVerticalKihon),
    );
    insert(
        &mut exports,
        "heading-band-em",
        fun_n(2),
        op(JapaneseKihonOp::HeadingBandEm),
    );
    insert(
        &mut exports,
        "indent-em",
        fun_n(2),
        op(JapaneseKihonOp::IndentEm),
    );
    insert(
        &mut exports,
        "trim-size",
        fun_n(2),
        op(JapaneseKihonOp::TrimSize),
    );
    insert(
        &mut exports,
        "a5-trim",
        nullary_dynamic(),
        op(JapaneseKihonOp::A5Trim),
    );
    insert(
        &mut exports,
        "b5-jis-trim",
        nullary_dynamic(),
        op(JapaneseKihonOp::B5JisTrim),
    );
    insert(
        &mut exports,
        "a4-trim",
        nullary_dynamic(),
        op(JapaneseKihonOp::A4Trim),
    );
    insert(
        &mut exports,
        "margins",
        fun_n(4),
        op(JapaneseKihonOp::Margins),
    );
    insert(
        &mut exports,
        "place-hanmen",
        fun_n(3),
        op(JapaneseKihonOp::PlaceHanmen),
    );
    insert(
        &mut exports,
        "column-count-one",
        nullary_dynamic(),
        op(JapaneseKihonOp::ColumnCountOne),
    );
    insert(
        &mut exports,
        "column-count-two",
        nullary_dynamic(),
        op(JapaneseKihonOp::ColumnCountTwo),
    );
    insert(
        &mut exports,
        "multi-column",
        fun_n(3),
        op(JapaneseKihonOp::MultiColumn),
    );
    insert(
        &mut exports,
        "vertical-flow",
        fun_n(1),
        op(JapaneseKihonOp::VerticalFlow),
    );
    insert(
        &mut exports,
        "tate-digits",
        fun_n(1),
        op(JapaneseKihonOp::TateDigits),
    );
    insert(
        &mut exports,
        "vertical-stack",
        fun_n(1),
        op(JapaneseKihonOp::VerticalStack),
    );
    insert(
        &mut exports,
        "tate-chu-yoko-span",
        fun_n(1),
        op(JapaneseKihonOp::TateChuYokoSpan),
    );
    insert(
        &mut exports,
        "vertical-text",
        fun_n(4),
        op(JapaneseKihonOp::VerticalText),
    );
    insert(
        &mut exports,
        "place-vertical-text",
        fun_n(4),
        op(JapaneseKihonOp::PlaceVerticalText),
    );
    insert(
        &mut exports,
        "vertical-text-stack",
        fun_n(4),
        op(JapaneseKihonOp::VerticalTextStack),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `japanese/markup`.
pub fn populate_japanese_markup_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::JapaneseMarkup(sub);
    insert(
        &mut exports,
        "heading",
        fun_n(1),
        op(JapaneseMarkupOp::Heading),
    );
    insert(
        &mut exports,
        "section",
        fun_n(1),
        op(JapaneseMarkupOp::Section),
    );
    insert(
        &mut exports,
        "paragraph",
        fun_n(1),
        op(JapaneseMarkupOp::Paragraph),
    );
    insert(&mut exports, "note", fun_n(1), op(JapaneseMarkupOp::Note));
    insert(
        &mut exports,
        "emphasis",
        fun_n(1),
        op(JapaneseMarkupOp::Emphasis),
    );
    insert(&mut exports, "warn", fun_n(1), op(JapaneseMarkupOp::Warn));
    insert(&mut exports, "todo", fun_n(1), op(JapaneseMarkupOp::Todo));
    insert(&mut exports, "ruby", fun_n(2), op(JapaneseMarkupOp::Ruby));
    insert(
        &mut exports,
        "jukugo-ruby",
        fun_n(2),
        op(JapaneseMarkupOp::JukugoRuby),
    );
    insert(
        &mut exports,
        "tate-chu-yoko",
        fun_n(1),
        op(JapaneseMarkupOp::TateChuYoko),
    );
    insert(
        &mut exports,
        "tategaki-paragraph",
        fun_n(1),
        op(JapaneseMarkupOp::TategakiParagraph),
    );
    insert(
        &mut exports,
        "tategaki-text",
        fun_n(4),
        op(JapaneseMarkupOp::TategakiText),
    );
    insert(
        &mut exports,
        "markup-bridge",
        fun_n(2),
        op(JapaneseMarkupOp::MarkupBridge),
    );
    insert(
        &mut exports,
        "doc-with-markup",
        fun_n(3),
        op(JapaneseMarkupOp::DocWithMarkup),
    );
    insert(&mut exports, "ul", fun_n(1), op(JapaneseMarkupOp::Ul));
    insert(&mut exports, "doc", fun_n(2), op(JapaneseMarkupOp::Doc));
    module.typed_exports = exports;
}
