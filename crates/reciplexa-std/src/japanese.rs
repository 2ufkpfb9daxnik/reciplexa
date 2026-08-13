//! `rpx.std.japanese` — JLReq-oriented character class / linebreak / kihon stubs.
//!
//! Honest scope: useful Rust tables and APIs that package natives and future
//! layout can call. This is **not** a full JLReq UCS membership matrix or a
//! typesetting engine (`OPEN-TEXT-JA-001`). See `lang/ja-math-deepen-plan.md`.
//!
//! Package synthetic RPX (`japanese/*` in `reciplexa-package::domain_bodies`)
//! remains the Core-evaluable import surface; prefer these Rust APIs for hosts.

use std::fmt;

/// JLReq character class ids `cl-01` .. `cl-30` (plus `Other` for unknown).
///
/// Names align with `packages/japanese/interface/classes.rpi` / domain natives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CharClass {
    /// cl-01 始め括弧類
    OpeningBrackets = 1,
    /// cl-02 終わり括弧類
    ClosingBrackets = 2,
    /// cl-03 ハイフン類
    Hyphens = 3,
    /// cl-04 区切り約物
    DividingPunctuation = 4,
    /// cl-05 中点類
    MiddleDots = 5,
    /// cl-06 句点類
    FullStops = 6,
    /// cl-07 読点類
    Commas = 7,
    /// cl-08 分離禁止文字
    Inseparable = 8,
    /// cl-09 繰返し記号
    IterationMarks = 9,
    /// cl-10 長音記号
    ProlongedSoundMark = 10,
    /// cl-11 小書きの仮名
    SmallKana = 11,
    /// cl-12 前置省略記号
    PrefixedAbbreviations = 12,
    /// cl-13 後置省略記号
    PostfixedAbbreviations = 13,
    /// cl-14 和字間隔等
    Spaces = 14,
    /// cl-15 平仮名
    Hiragana = 15,
    /// cl-16 片仮名
    Katakana = 16,
    /// cl-17 等号類
    MathSymbols = 17,
    /// cl-18 連数字
    GroupedNumerals = 18,
    /// cl-19 漢字等
    Ideographic = 19,
    /// cl-20 数字
    Numeric = 20,
    /// cl-21 単位記号中の欧字等
    UnitSymbols = 21,
    /// cl-22 囲み文字
    EnclosedAlphanumerics = 22,
    /// cl-23 装飾文字
    Ornaments = 23,
    /// cl-24 単純な欧字
    SimpleWestern = 24,
    /// cl-25 複雑な欧字
    ComplexWestern = 25,
    /// cl-26 割注終わり括弧類
    WarichuClose = 26,
    /// cl-27 欧文用文字
    WesternCharacters = 27,
    /// cl-28 添付欧文
    AttachedWestern = 28,
    /// cl-29 割注始め括弧類
    WarichuOpen = 29,
    /// cl-30 縦中横
    TateChuYoko = 30,
    /// Outside the stub classifier subset.
    Other = 0,
}

impl CharClass {
    /// Numeric id matching package `cl-NN` records (`1`..=`30`, or `0` for Other).
    pub fn id(self) -> u8 {
        self as u8
    }

    /// Package-style code, e.g. `"cl-01"`.
    pub fn code(self) -> &'static str {
        match self {
            Self::OpeningBrackets => "cl-01",
            Self::ClosingBrackets => "cl-02",
            Self::Hyphens => "cl-03",
            Self::DividingPunctuation => "cl-04",
            Self::MiddleDots => "cl-05",
            Self::FullStops => "cl-06",
            Self::Commas => "cl-07",
            Self::Inseparable => "cl-08",
            Self::IterationMarks => "cl-09",
            Self::ProlongedSoundMark => "cl-10",
            Self::SmallKana => "cl-11",
            Self::PrefixedAbbreviations => "cl-12",
            Self::PostfixedAbbreviations => "cl-13",
            Self::Spaces => "cl-14",
            Self::Hiragana => "cl-15",
            Self::Katakana => "cl-16",
            Self::MathSymbols => "cl-17",
            Self::GroupedNumerals => "cl-18",
            Self::Ideographic => "cl-19",
            Self::Numeric => "cl-20",
            Self::UnitSymbols => "cl-21",
            Self::EnclosedAlphanumerics => "cl-22",
            Self::Ornaments => "cl-23",
            Self::SimpleWestern => "cl-24",
            Self::ComplexWestern => "cl-25",
            Self::WarichuClose => "cl-26",
            Self::WesternCharacters => "cl-27",
            Self::AttachedWestern => "cl-28",
            Self::WarichuOpen => "cl-29",
            Self::TateChuYoko => "cl-30",
            Self::Other => "cl-other",
        }
    }

    /// English name used by `japanese/classes` `class-name`.
    pub fn name(self) -> &'static str {
        match self {
            Self::OpeningBrackets => "opening-brackets",
            Self::ClosingBrackets => "closing-brackets",
            Self::Hyphens => "hyphens",
            Self::DividingPunctuation => "dividing-punctuation",
            Self::MiddleDots => "middle-dots",
            Self::FullStops => "full-stops",
            Self::Commas => "commas",
            Self::Inseparable => "inseparable",
            Self::IterationMarks => "iteration-marks",
            Self::ProlongedSoundMark => "prolonged-sound-mark",
            Self::SmallKana => "small-kana",
            Self::PrefixedAbbreviations => "prefixed-abbreviations",
            Self::PostfixedAbbreviations => "postfixed-abbreviations",
            Self::Spaces => "spaces",
            Self::Hiragana => "hiragana",
            Self::Katakana => "katakana",
            Self::MathSymbols => "math-symbols",
            Self::GroupedNumerals => "grouped-numerals",
            Self::Ideographic => "ideographic",
            Self::Numeric => "numeric",
            Self::UnitSymbols => "unit-symbols",
            Self::EnclosedAlphanumerics => "enclosed-alphanumerics",
            Self::Ornaments => "ornaments",
            Self::SimpleWestern => "simple-western",
            Self::ComplexWestern => "complex-western",
            Self::WarichuClose => "warichu-close",
            Self::WesternCharacters => "western-characters",
            Self::AttachedWestern => "attached-western",
            Self::WarichuOpen => "warichu-open",
            Self::TateChuYoko => "tate-chu-yoko",
            Self::Other => "other",
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            1 => Self::OpeningBrackets,
            2 => Self::ClosingBrackets,
            3 => Self::Hyphens,
            4 => Self::DividingPunctuation,
            5 => Self::MiddleDots,
            6 => Self::FullStops,
            7 => Self::Commas,
            8 => Self::Inseparable,
            9 => Self::IterationMarks,
            10 => Self::ProlongedSoundMark,
            11 => Self::SmallKana,
            12 => Self::PrefixedAbbreviations,
            13 => Self::PostfixedAbbreviations,
            14 => Self::Spaces,
            15 => Self::Hiragana,
            16 => Self::Katakana,
            17 => Self::MathSymbols,
            18 => Self::GroupedNumerals,
            19 => Self::Ideographic,
            20 => Self::Numeric,
            21 => Self::UnitSymbols,
            22 => Self::EnclosedAlphanumerics,
            23 => Self::Ornaments,
            24 => Self::SimpleWestern,
            25 => Self::ComplexWestern,
            26 => Self::WarichuClose,
            27 => Self::WesternCharacters,
            28 => Self::AttachedWestern,
            29 => Self::WarichuOpen,
            30 => Self::TateChuYoko,
            _ => return None,
        })
    }

    pub fn is_punctuation(self) -> bool {
        matches!(
            self,
            Self::OpeningBrackets
                | Self::ClosingBrackets
                | Self::Hyphens
                | Self::DividingPunctuation
                | Self::MiddleDots
                | Self::FullStops
                | Self::Commas
        )
    }

    pub fn is_kana(self) -> bool {
        matches!(self, Self::SmallKana | Self::Hiragana | Self::Katakana)
    }

    pub fn is_western(self) -> bool {
        matches!(
            self,
            Self::UnitSymbols
                | Self::SimpleWestern
                | Self::ComplexWestern
                | Self::WesternCharacters
                | Self::AttachedWestern
        )
    }

    pub fn is_square_letter(self) -> bool {
        matches!(
            self,
            Self::Hiragana | Self::Katakana | Self::Ideographic | Self::Numeric
        )
    }
}

impl fmt::Display for CharClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Classify a single Unicode scalar with a **useful subset** of JLReq classes.
///
/// Not a full UCS membership table: uncommon punctuation and edge cases fall
/// through to coarse buckets (ideographic / western / other).
pub fn classify_char(c: char) -> CharClass {
    // Specific punctuation / symbols first (mirrors package classify-sample).
    match c {
        '「' | '『' | '（' | '〔' | '［' | '｛' | '〈' | '《' | '(' | '[' | '{' => {
            return CharClass::OpeningBrackets;
        }
        '」' | '』' | '）' | '〕' | '］' | '｝' | '〉' | '》' | ')' | ']' | '}' => {
            return CharClass::ClosingBrackets;
        }
        '—' | '–' | '‐' | '-' | '〜' | '～' => return CharClass::Hyphens,
        '！' | '？' | '!' | '?' => return CharClass::DividingPunctuation,
        '・' | '·' | '･' => return CharClass::MiddleDots,
        '。' | '．' | '.' => return CharClass::FullStops,
        '、' | '，' | ',' => return CharClass::Commas,
        '…' | '‥' | '〳' | '〴' | '〵' => return CharClass::Inseparable,
        '々' | 'ゝ' | 'ゞ' | 'ヽ' | 'ヾ' => return CharClass::IterationMarks,
        'ー' | 'ｰ' => return CharClass::ProlongedSoundMark,
        '〒' | '￥' | '＄' | '￡' | '＃' | '#' | '$' | '¥' | '£' => {
            return CharClass::PrefixedAbbreviations;
        }
        '％' | '%' | '‰' | '℃' | '°' => return CharClass::PostfixedAbbreviations,
        '＝' | '=' | '≠' | '≦' | '≧' | '+' | '±' | '×' | '÷' => {
            return CharClass::MathSymbols;
        }
        _ => {}
    }

    if c.is_whitespace() || c == '\u{3000}' {
        return CharClass::Spaces;
    }

    // Small kana (hiragana + katakana).
    if matches!(
        c,
        'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ' | 'っ' | 'ゃ' | 'ゅ' | 'ょ' | 'ゎ' | 'ゕ' | 'ゖ'
            | 'ァ' | 'ィ' | 'ゥ' | 'ェ' | 'ォ' | 'ッ' | 'ャ' | 'ュ' | 'ョ' | 'ヮ' | 'ヵ' | 'ヶ'
    ) {
        return CharClass::SmallKana;
    }

    match c {
        '\u{3040}'..='\u{309F}' => CharClass::Hiragana,
        '\u{30A0}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}' => CharClass::Katakana,
        '0'..='9' | '０'..='９' => CharClass::Numeric,
        'A'..='Z' | 'a'..='z' | 'Ａ'..='Ｚ' | 'ａ'..='ｚ' => CharClass::WesternCharacters,
        // CJK Unified Ideographs + common extensions / compatibility.
        '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{20000}'..='\u{2A6DF}' => CharClass::Ideographic,
        _ if (c as u32) >= 0x80 => CharClass::Ideographic,
        _ => CharClass::Other,
    }
}

/// Line-break opportunity between two character classes (JLReq-inspired stub).
///
/// Not the normative appendix C matrix — only common kinsoku / inseparable cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BreakOpportunity {
    Allowed,
    Prohibited,
    Inseparable,
}

impl BreakOpportunity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Prohibited => "prohibited",
            Self::Inseparable => "inseparable",
        }
    }

    pub fn may_break(self) -> bool {
        matches!(self, Self::Allowed)
    }
}

impl fmt::Display for BreakOpportunity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classes that must not start a line (sample kinsoku / line-head prohibited).
pub fn is_line_head_prohibited(class: CharClass) -> bool {
    matches!(
        class,
        CharClass::ClosingBrackets
            | CharClass::FullStops
            | CharClass::Commas
            | CharClass::IterationMarks
            | CharClass::ProlongedSoundMark
            | CharClass::SmallKana
            | CharClass::WarichuClose
            | CharClass::DividingPunctuation
            | CharClass::MiddleDots
            | CharClass::Hyphens
    )
}

/// Classes that must not end a line (sample line-end prohibited).
pub fn is_line_end_prohibited(class: CharClass) -> bool {
    matches!(
        class,
        CharClass::OpeningBrackets
            | CharClass::PrefixedAbbreviations
            | CharClass::AttachedWestern
            | CharClass::WarichuOpen
    )
}

/// Decide break opportunity between adjacent classified characters.
///
/// Prefer calling this from future layout; package `japanese/linebreak`
/// `break-between` is a synthetic RPX mirror for Core imports.
pub fn break_opportunity(prev: CharClass, next: CharClass) -> BreakOpportunity {
    if prev == CharClass::Inseparable || next == CharClass::Inseparable {
        return BreakOpportunity::Inseparable;
    }
    // Simple western / ASCII letter runs stay together (cl-24 / cl-27 style).
    if (prev == CharClass::SimpleWestern || prev == CharClass::WesternCharacters)
        && (next == CharClass::SimpleWestern || next == CharClass::WesternCharacters)
    {
        return BreakOpportunity::Inseparable;
    }
    if prev == CharClass::Numeric && next == CharClass::Numeric {
        return BreakOpportunity::Inseparable;
    }
    if is_line_end_prohibited(prev) || is_line_head_prohibited(next) {
        return BreakOpportunity::Prohibited;
    }
    // Digit before close/open (package numeric-before-close stub).
    if prev == CharClass::Numeric
        && matches!(
            next,
            CharClass::ClosingBrackets | CharClass::OpeningBrackets | CharClass::PostfixedAbbreviations
        )
    {
        return BreakOpportunity::Prohibited;
    }
    BreakOpportunity::Allowed
}

/// Classify two chars and return the break opportunity between them.
pub fn break_opportunity_chars(prev: char, next: char) -> BreakOpportunity {
    break_opportunity(classify_char(prev), classify_char(next))
}
