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
/// through to coarse buckets (ideographic / western / other). Coverage is
/// intentionally broader than the package `classify-sample` glyph list, still
/// a subset of JLReq appendix A.
pub fn classify_char(c: char) -> CharClass {
    // Specific punctuation / symbols first (mirrors package classify-sample +
    // common JLReq appendix A samples).
    match c {
        // cl-01 — opening brackets (fullwidth + halfwidth + CJK quotation).
        '「' | '『' | '（' | '〔' | '［' | '｛' | '〈' | '《' | '【' | '〖' | '〘' | '〚' | '｢'
        | '〝' | '‘' | '“' | '(' | '[' | '{' => {
            return CharClass::OpeningBrackets;
        }
        // cl-02 — closing brackets.
        '」' | '』' | '）' | '〕' | '］' | '｝' | '〉' | '》' | '】' | '〗' | '〙' | '〛' | '｣'
        | '〞' | '〟' | '’' | '”' | ')' | ']' | '}' => {
            return CharClass::ClosingBrackets;
        }
        // cl-29 / cl-26 — warichu brackets (subset).
        '｟' | '⸨' => return CharClass::WarichuOpen,
        '｠' | '⸩' => return CharClass::WarichuClose,
        // cl-03 — hyphens / wave dashes / ideographic wavy dash.
        '—' | '–' | '‐' | '‑' | '‒' | '―' | '−' | '-' | '－' | '﹣' | '〜' | '～' | '〰' | '゠' =>
        {
            return CharClass::Hyphens;
        }
        // cl-04 — dividing punctuation (incl. ideographic closing mark).
        '！' | '？' | '‼' | '⁇' | '⁈' | '⁉' | '!' | '?' | '〆' | '／' | '＼' => {
            return CharClass::DividingPunctuation;
        }
        // cl-05 — middle dots / colon-like separators.
        '・' | '·' | '･' | '∶' | '︰' | ':' | '：' | ';' | '；' => {
            return CharClass::MiddleDots;
        }
        // cl-06 / cl-07 — stops and commas (fullwidth + halfwidth CJK forms).
        '。' | '．' | '.' | '｡' => return CharClass::FullStops,
        '、' | '，' | ',' | '､' => return CharClass::Commas,
        // cl-08 — inseparable (ellipsis / ditto / vertical iteration forms).
        '…' | '‥' | '⋯' | '〳' | '〴' | '〵' | '〃' => return CharClass::Inseparable,
        // cl-09 / cl-10.
        '々' | 'ゝ' | 'ゞ' | 'ヽ' | 'ヾ' | '〻' => return CharClass::IterationMarks,
        'ー' | 'ｰ' | 'ㅡ' => return CharClass::ProlongedSoundMark,
        // cl-12 / cl-13 — prefixed / postfixed abbreviations (fullwidth currency/sign).
        '〒' | '￥' | '＄' | '￡' | '＃' | '№' | '＠' | '#' | '$' | '¥' | '£' | '€' | '₩' | '@' =>
        {
            return CharClass::PrefixedAbbreviations;
        }
        '％' | '%' | '‰' | '℃' | '°' | '′' | '″' | '㌫' | '￠' => {
            return CharClass::PostfixedAbbreviations;
        }
        // cl-17 — math / equality-like symbols (ASCII + fullwidth operators).
        '＝' | '=' | '≠' | '≦' | '≧' | '≤' | '≥' | '≈' | '+' | '＋' | '±' | '×' | '÷' | '＜'
        | '＞' | '<' | '>' | '＆' | '&' => {
            return CharClass::MathSymbols;
        }
        // cl-21 — unit symbols (squared/cubed metric forms + litre).
        '㎜' | '㎝' | '㎞' | '㎡' | '㎥' | '㎎' | '㎏' | '㏄' | 'ℓ' | 'Å' => {
            return CharClass::UnitSymbols;
        }
        // cl-22 — enclosed alphanumerics / circled forms (common subset).
        '①'..='⑳' | '⑴'..='⒇' | '⒈'..='⒛' | 'ⓐ'..='ⓩ' | 'Ⓐ'..='Ⓩ' | '㈱' | '㈲' | '㈹' =>
        {
            return CharClass::EnclosedAlphanumerics;
        }
        // cl-23 — ornaments / reference marks / geta / part alternation.
        '※' | '＊' | '*' | '☆' | '★' | '○' | '●' | '◎' | '◇' | '◆' | '□' | '■' | '△' | '▲'
        | '▽' | '▼' | '♠' | '♣' | '♥' | '♦' | '〓' | '〽' | '〇' | '￣' | '＾' | '｀' | '｜'
        | '|' | '^' | '`' => {
            return CharClass::Ornaments;
        }
        _ => {}
    }

    if c.is_whitespace() || c == '\u{3000}' || c == '\u{00A0}' || c == '\u{2002}' || c == '\u{2003}'
    {
        return CharClass::Spaces;
    }

    // cl-11 — small kana (hiragana + katakana + halfwidth small katakana).
    if matches!(
        c,
        'ぁ' | 'ぃ'
            | 'ぅ'
            | 'ぇ'
            | 'ぉ'
            | 'っ'
            | 'ゃ'
            | 'ゅ'
            | 'ょ'
            | 'ゎ'
            | 'ゕ'
            | 'ゖ'
            | 'ァ'
            | 'ィ'
            | 'ゥ'
            | 'ェ'
            | 'ォ'
            | 'ッ'
            | 'ャ'
            | 'ュ'
            | 'ョ'
            | 'ヮ'
            | 'ヵ'
            | 'ヶ'
            | 'ｧ'
            | 'ｨ'
            | 'ｩ'
            | 'ｪ'
            | 'ｫ'
            | 'ｬ'
            | 'ｭ'
            | 'ｮ'
            | 'ｯ'
    ) {
        return CharClass::SmallKana;
    }

    match c {
        // cl-15 / cl-16 — kana blocks (fullwidth + halfwidth katakana).
        '\u{3040}'..='\u{309F}' => CharClass::Hiragana,
        '\u{30A0}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}' | '\u{FF66}'..='\u{FF9D}' => {
            CharClass::Katakana
        }
        // cl-20 — ASCII, fullwidth, and common superscript/subscript digits.
        '0'..='9'
        | '０'..='９'
        | '⁰'
        | '¹'
        | '²'
        | '³'
        | '⁴'
        | '⁵'
        | '⁶'
        | '⁷'
        | '⁸'
        | '⁹'
        | '₀'
        | '₁'
        | '₂'
        | '₃'
        | '₄'
        | '₅'
        | '₆'
        | '₇'
        | '₈'
        | '₉' => CharClass::Numeric,
        // cl-27 — ASCII / fullwidth Latin (package classify-sample maps "A" → 27).
        'A'..='Z' | 'a'..='z' | 'Ａ'..='Ｚ' | 'ａ'..='ｚ' => CharClass::WesternCharacters,
        // cl-24 — underscore / apostrophe-like simple western connectors (subset).
        '_' | '＿' | '\'' => CharClass::SimpleWestern,
        // Residual CJK Symbols and Punctuation (U+3000–303F) not matched above —
        // treat as ideographic punctuation / square letters.
        '\u{3001}'..='\u{303F}' => CharClass::Ideographic,
        // cl-25 — Latin-1 / Extended-A letters + combining marks (complex western).
        '\u{00C0}'..='\u{00D6}'
        | '\u{00D8}'..='\u{00F6}'
        | '\u{00F8}'..='\u{00FF}'
        | '\u{0100}'..='\u{024F}'
        | '\u{0300}'..='\u{036F}' => CharClass::ComplexWestern,
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
///
/// Aligned with package `kinsoku-profile` / `sample-line-head-prohibited`
/// character samples (still a class-level stub, not appendix C).
pub fn is_line_head_prohibited(class: CharClass) -> bool {
    matches!(
        class,
        CharClass::ClosingBrackets
            | CharClass::Hyphens
            | CharClass::DividingPunctuation
            | CharClass::MiddleDots
            | CharClass::FullStops
            | CharClass::Commas
            | CharClass::IterationMarks
            | CharClass::ProlongedSoundMark
            | CharClass::SmallKana
            | CharClass::PostfixedAbbreviations
            | CharClass::WarichuClose
    )
}

/// Classes that must not end a line (sample line-end prohibited).
///
/// Aligned with package `sample-line-end-prohibited` (opens + prefixed abbrevs).
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
    // Simple / complex / ASCII western letter runs stay together.
    if is_western_run_class(prev) && is_western_run_class(next) {
        return BreakOpportunity::Inseparable;
    }
    if prev == CharClass::Numeric && next == CharClass::Numeric {
        return BreakOpportunity::Inseparable;
    }
    if prev == CharClass::GroupedNumerals && next == CharClass::GroupedNumerals {
        return BreakOpportunity::Inseparable;
    }
    if is_line_end_prohibited(prev) || is_line_head_prohibited(next) {
        return BreakOpportunity::Prohibited;
    }
    // Digit before close/open/postfix (package numeric-before-close stub).
    if prev == CharClass::Numeric
        && matches!(
            next,
            CharClass::ClosingBrackets
                | CharClass::OpeningBrackets
                | CharClass::PostfixedAbbreviations
                | CharClass::UnitSymbols
        )
    {
        return BreakOpportunity::Prohibited;
    }
    // Prefixed abbrev + digit (package sample-pair-rules).
    if prev == CharClass::PrefixedAbbreviations && next == CharClass::Numeric {
        return BreakOpportunity::Prohibited;
    }
    // Full stop / comma before open — keep on same line (package samples).
    if matches!(prev, CharClass::FullStops | CharClass::Commas)
        && matches!(next, CharClass::OpeningBrackets)
    {
        return BreakOpportunity::Prohibited;
    }
    BreakOpportunity::Allowed
}

fn is_western_run_class(class: CharClass) -> bool {
    matches!(
        class,
        CharClass::SimpleWestern | CharClass::ComplexWestern | CharClass::WesternCharacters
    )
}

/// Classify two chars and return the break opportunity between them.
pub fn break_opportunity_chars(prev: char, next: char) -> BreakOpportunity {
    break_opportunity(classify_char(prev), classify_char(next))
}

/// Writing mode for kihon-hanmen / vertical stubs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WritingMode {
    /// CSS `horizontal-tb` / package `writing-mode-horizontal`.
    HorizontalTb,
    /// CSS `vertical-rl` / package `writing-mode-vertical`.
    VerticalRl,
}

impl WritingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HorizontalTb => "horizontal-tb",
            Self::VerticalRl => "vertical-rl",
        }
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Self::VerticalRl)
    }
}

impl fmt::Display for WritingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kihon-hanmen geometry stub (character frame × lines × line-rate).
///
/// Em sizes are abstract; document lower does not yet place these.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KihonHanmen {
    pub char_size_em: f64,
    pub line_length: u32,
    pub line_count: u32,
    pub line_rate: f64,
    pub writing_mode: WritingMode,
}

impl KihonHanmen {
    pub fn new(
        char_size_em: f64,
        line_length: u32,
        line_count: u32,
        line_rate: f64,
        writing_mode: WritingMode,
    ) -> Self {
        Self {
            char_size_em,
            line_length,
            line_count,
            line_rate,
            writing_mode,
        }
    }

    pub fn default_horizontal() -> Self {
        Self::new(1.0, 40, 30, 1.5, WritingMode::HorizontalTb)
    }

    pub fn default_vertical() -> Self {
        Self::new(1.0, 35, 20, 1.5, WritingMode::VerticalRl)
    }

    /// Gap between lines in em (line_rate − 1) × char size.
    pub fn line_gap_em(self) -> f64 {
        self.char_size_em * (self.line_rate - 1.0)
    }

    /// Inline measure of the hanmen in em.
    pub fn hanmen_inline_em(self) -> f64 {
        self.char_size_em * f64::from(self.line_length)
    }

    /// Block measure: lines + gaps between them.
    pub fn hanmen_block_em(self) -> f64 {
        let lines = f64::from(self.line_count);
        let gaps = f64::from(self.line_count.saturating_sub(1));
        self.char_size_em * lines + self.line_gap_em() * gaps
    }

    /// Heading band height for `lines` of kihon characters.
    pub fn heading_band_em(self, lines: u32) -> f64 {
        let n = f64::from(lines);
        let gaps = f64::from(lines.saturating_sub(1));
        self.char_size_em * n + self.line_gap_em() * gaps
    }

    pub fn indent_em(self, chars: u32) -> f64 {
        self.char_size_em * f64::from(chars)
    }
}

/// Common line-rate presets from `japanese/kihon`.
pub mod line_rate {
    pub const SOLID: f64 = 1.0;
    pub const COMPACT: f64 = 1.2;
    pub const DEFAULT: f64 = 1.5;
    pub const RELAXED: f64 = 1.7;
    pub const LOOSE: f64 = 2.0;
}

/// Ruby annotation kind — aligns with `japanese/markup` `ruby` / `jukugo-ruby`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RubyKind {
    Simple,
    Jukugo,
}

impl RubyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Jukugo => "jukugo",
        }
    }
}

impl fmt::Display for RubyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Ruby (振り仮名) data — package tag `ja-ruby`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ruby {
    pub base: String,
    pub annotation: String,
    pub kind: RubyKind,
}

impl Ruby {
    pub fn simple(base: impl Into<String>, annotation: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            annotation: annotation.into(),
            kind: RubyKind::Simple,
        }
    }

    pub fn jukugo(base: impl Into<String>, annotation: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            annotation: annotation.into(),
            kind: RubyKind::Jukugo,
        }
    }

    pub fn tag(&self) -> &'static str {
        "ja-ruby"
    }
}

impl fmt::Display for Ruby {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.base, self.annotation)
    }
}

/// Tate-chu-yoko (縦中横) inline span — package tag `ja-tate-chu-yoko`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TateChuYoko {
    pub body: String,
}

impl TateChuYoko {
    pub fn new(body: impl Into<String>) -> Self {
        Self { body: body.into() }
    }

    pub fn tag(&self) -> &'static str {
        "ja-tate-chu-yoko"
    }
}

impl fmt::Display for TateChuYoko {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tate-chu-yoko({})", self.body)
    }
}

/// Tategaki paragraph stub — package tag `ja-tategaki-paragraph`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TategakiParagraph {
    pub body: String,
    pub writing_mode: WritingMode,
}

impl TategakiParagraph {
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            writing_mode: WritingMode::VerticalRl,
        }
    }

    pub fn tag(&self) -> &'static str {
        "ja-tategaki-paragraph"
    }
}
