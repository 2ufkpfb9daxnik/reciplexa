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
///
/// **Gaps (Wave 5 Y0 still open):** full JLReq appendix A membership for every
/// scalar; cl-18 grouped numerals / cl-28 attached western / cl-30 tate-chu-yoko
/// detection; most Enclosed CJK Letters (U+3200–32FF) beyond a sample; full
/// CJK Compatibility (U+3300–33FF) unit/square set; vertical presentation forms;
/// and normative pair-matrix class nuance beyond coarse buckets.
///
/// **Wave 20:** emoji / color-presentation scalars (pictographs, dingbats,
/// variation selectors, emoji tags) map to [`CharClass::Other`] (explicit
/// unknown) — not Ideographic via the high-BMP fallback.
pub fn classify_char(c: char) -> CharClass {
    // Specific punctuation / symbols first (mirrors package classify-sample +
    // common JLReq appendix A samples + Wave 5 UCS-ish expansions).
    match c {
        // cl-01 — opening brackets (fullwidth + halfwidth + CJK / general punct quotes).
        '「' | '『' | '（' | '〔' | '［' | '｛' | '〈' | '《' | '【' | '〖' | '〘' | '〚' | '｢'
        | '〝' | '‘' | '“' | '‚' | '‛' | '‟' | '‹' | '«' | '(' | '[' | '{' => {
            return CharClass::OpeningBrackets;
        }
        // cl-02 — closing brackets.
        '」' | '』' | '）' | '〕' | '］' | '｝' | '〉' | '》' | '】' | '〗' | '〙' | '〛' | '｣'
        | '〞' | '〟' | '’' | '”' | '„' | '›' | '»' | ')' | ']' | '}' => {
            return CharClass::ClosingBrackets;
        }
        // cl-29 / cl-26 — warichu brackets (subset).
        '｟' | '⸨' => return CharClass::WarichuOpen,
        '｠' | '⸩' => return CharClass::WarichuClose,
        // cl-03 — hyphens / wave dashes / wavy overlines / swung dash / general punct.
        '—' | '–' | '‐' | '‑' | '‒' | '―' | '−' | '-' | '－' | '﹣' | '〜' | '～' | '〰' | '゠'
        | '⁓' | '∿' | '﹏' | '﹋' | '﹌' | '⁃' => {
            return CharClass::Hyphens;
        }
        // cl-04 — dividing punctuation (incl. ideographic closing mark / fraction slash /
        // general punctuation double marks / fullwidth reverse solidus family).
        '！' | '？' | '‼' | '⁇' | '⁈' | '⁉' | '!' | '?' | '〆' | '／' | '＼' | '⁄' | '⧸' | '⧹'
        | '†' | '‡' | '‽' | '⁎' | '⁕' => {
            return CharClass::DividingPunctuation;
        }
        // cl-05 — middle dots / colon-like separators / bullet-ish general punct.
        '・' | '·' | '･' | '∶' | '︰' | ':' | '：' | ';' | '；' | '⁚' | '⁝' | '•' | '‣' | '․'
        | '⁏' | '﹕' | '﹔' => {
            return CharClass::MiddleDots;
        }
        // cl-06 / cl-07 — stops and commas (fullwidth + halfwidth CJK forms).
        '。' | '．' | '.' | '｡' => return CharClass::FullStops,
        '、' | '，' | ',' | '､' => return CharClass::Commas,
        // cl-08 — inseparable (ellipsis / ditto / vertical kana repeat).
        '…' | '‥' | '⋯' | '⋰' | '⋱' | '〳' | '〴' | '〵' | '〃' | '〱' | '〲' => {
            return CharClass::Inseparable;
        }
        // cl-09 / cl-10.
        '々' | 'ゝ' | 'ゞ' | 'ヽ' | 'ヾ' | '〻' => return CharClass::IterationMarks,
        'ー' | 'ｰ' | 'ㅡ' => return CharClass::ProlongedSoundMark,
        // cl-12 / cl-13 — prefixed / postfixed abbreviations (fullwidth currency/sign).
        '〒' | '￥' | '＄' | '￡' | '￦' | '＃' | '№' | '＠' | '#' | '$' | '¥' | '£' | '€'
        | '₩' | '@' | '〠' | '〶' => {
            return CharClass::PrefixedAbbreviations;
        }
        '％' | '%' | '‰' | '‱' | '℃' | '℉' | '°' | '′' | '″' | '‴' | '㌫' | '￠' =>
        {
            return CharClass::PostfixedAbbreviations;
        }
        // cl-17 — math / equality-like symbols (ASCII + fullwidth operators).
        '＝' | '=' | '≠' | '≦' | '≧' | '≤' | '≥' | '≈' | '+' | '＋' | '±' | '×' | '÷' | '＜'
        | '＞' | '<' | '>' | '＆' | '&' | '￤' | '￢' | '￩' | '￪' | '￫' | '￬' => {
            return CharClass::MathSymbols;
        }
        // cl-21 — unit symbols (squared/cubed metric forms + litre + CJK compatibility).
        '㎜' | '㎝' | '㎞' | '㎡' | '㎥' | '㎎' | '㎏' | '㏄' | 'ℓ' | 'Å' | '㍉' | '㍍' | '㌔'
        | '㌢' | '㍗' | '㌘' | '㌧' | '㌃' | '㌶' | '㍑' | '㍊' | '㌻' => {
            return CharClass::UnitSymbols;
        }
        // cl-22 — enclosed alphanumerics / circled forms (common subset + CJK circled).
        '①'..='⑳'
        | '⑴'..='⒇'
        | '⒈'..='⒛'
        | 'ⓐ'..='ⓩ'
        | 'Ⓐ'..='Ⓩ'
        | '❶'..='❿'
        | '➀'..='➉'
        | '⓵'..='⓾'
        | '㈠'..='㈩'
        | '㊀'..='㊉'
        | '㈱'
        | '㈲'
        | '㈹'
        | '㊤'
        | '㊥'
        | '㊦'
        | '㊧'
        | '㊨' => {
            return CharClass::EnclosedAlphanumerics;
        }
        // cl-23 — ornaments / reference marks / geta / part alternation / masu /
        // CJK symbols leftovers that map cleanly as decoration.
        '※' | '＊' | '*' | '☆' | '★' | '○' | '●' | '◎' | '◇' | '◆' | '□' | '■' | '△' | '▲'
        | '▽' | '▼' | '♠' | '♣' | '♥' | '♦' | '〓' | '〽' | '〇' | '￣' | '＾' | '｀' | '｜'
        | '|' | '^' | '`' | '〄' | '〼' | '✦' | '✧' | '〷' | '〹' | '〺' => {
            return CharClass::Ornaments;
        }
        // Additional corner / angle brackets (math-ish forms).
        '〈' | '⟪' | '⟬' | '⟮' | '⌈' | '⌊' | '⌜' | '⌞' => {
            return CharClass::OpeningBrackets;
        }
        '〉' | '⟫' | '⟭' | '⟯' | '⌉' | '⌋' | '⌝' | '⌟' => {
            return CharClass::ClosingBrackets;
        }
        _ => {}
    }

    // cl-14 — spaces: ASCII / Unicode space separators / ZWSP-adjacent / ideographic.
    if c.is_whitespace()
        || matches!(
            c,
            '\u{00A0}' // NBSP
                | '\u{1680}' // Ogham space
                | '\u{2000}'
                ..='\u{200A}' // en/em/thin/hair/… spaces
                | '\u{200B}' // ZWSP (treat as space for linebreak stubs)
                | '\u{200C}' // ZWNJ
                | '\u{200D}' // ZWJ
                | '\u{202F}' // narrow NBSP
                | '\u{205F}' // medium mathematical space
                | '\u{3000}' // ideographic space
                | '\u{FEFF}' // BOM / ZWNBSP
        )
    {
        return CharClass::Spaces;
    }

    // cl-11 — small kana (hiragana + katakana + halfwidth small + phonetic extensions).
    // Complete common small set; vertical presentation forms remain a gap.
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
            // Katakana Phonetic Extensions (U+31F0–31FF) — all small.
            | 'ㇰ'..='ㇿ'
    ) {
        return CharClass::SmallKana;
    }

    match c {
        // cl-15 / cl-16 — kana blocks (fullwidth + halfwidth katakana; phonetic
        // extensions handled as SmallKana above).
        '\u{3040}'..='\u{309F}' => CharClass::Hiragana,
        '\u{30A0}'..='\u{30FF}' | '\u{FF66}'..='\u{FF9D}' => CharClass::Katakana,
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
        '_' | '＿' | '\'' | '＇' | '¨' => CharClass::SimpleWestern,
        // More Halfwidth and Fullwidth Forms punctuation chunks (U+FF00–FF60)
        // not matched above → coarse western / ornament-ish.
        '\u{FF01}'..='\u{FF0F}'
        | '\u{FF1A}'..='\u{FF20}'
        | '\u{FF3B}'..='\u{FF40}'
        | '\u{FF5B}'..='\u{FF65}' => CharClass::DividingPunctuation,
        // Residual CJK Symbols and Punctuation (U+3000–303F) not matched above —
        // treat as ideographic punctuation / square letters.
        '\u{3001}'..='\u{303F}' => CharClass::Ideographic,
        // General Punctuation leftovers (U+2000–206F) not matched → ornaments.
        '\u{2010}'..='\u{2027}' | '\u{2030}'..='\u{205E}' => CharClass::Ornaments,
        // cl-25 — Latin-1 / Extended-A letters + combining marks (complex western).
        '\u{00C0}'..='\u{00D6}'
        | '\u{00D8}'..='\u{00F6}'
        | '\u{00F8}'..='\u{00FF}'
        | '\u{0100}'..='\u{024F}'
        | '\u{0300}'..='\u{036F}' => CharClass::ComplexWestern,
        // Wave 20 — emoji / color presentation: explicit Other (unknown), not
        // Ideographic. Listed ornaments above still win for ♠★○… samples.
        '\u{FE0E}' | '\u{FE0F}' // text / emoji variation selectors
        | '\u{2600}'..='\u{26FF}' // Misc Symbols (unmatched → Other)
        | '\u{2700}'..='\u{27BF}' // Dingbats
        | '\u{1F000}'..='\u{1FAFF}' // mahjong .. Symbols and Pictographs Ext-A
        | '\u{E0020}'..='\u{E007F}' => CharClass::Other, // emoji tag characters
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
/// Resolved via denser [`BREAK_PAIR_MATRIX`] — still **not** normative appendix C.
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
/// character samples and [`BREAK_PAIR_MATRIX`] (still a class-level stub, not appendix C).
pub fn is_line_head_prohibited(class: CharClass) -> bool {
    is_line_head_prohibited_id(class.id())
}

/// Classes that must not end a line (sample line-end prohibited).
///
/// Aligned with package `sample-line-end-prohibited` (opens + prefixed abbrevs)
/// and [`BREAK_PAIR_MATRIX`].
pub fn is_line_end_prohibited(class: CharClass) -> bool {
    is_line_end_prohibited_id(class.id())
}

/// Hangable punctuation classes (JLReq-style stub).
///
/// Matches package `hangable-class?` / `kinsoku-profile.hangable-classes`:
/// **cl-06** full stops and **cl-07** commas. Not a full hanging-punctuation /
/// justification model — hosts may query this; [`break_line`] optionally
/// allows these glyphs to stick slightly past `max_em_units`.
pub fn is_hangable(class: CharClass) -> bool {
    matches!(class, CharClass::FullStops | CharClass::Commas)
}

/// Default end-of-line hang budget (em) for hangable punctuation.
///
/// Policy stub only: JLReq hanging punctuation uses a richer measure-overhang
/// model. Hosts / future `break_line` refinements may consult [`hang_width_em`]
/// instead of treating hang as unbounded past the measure.
pub const HANG_WIDTH_EM: f64 = 0.5;

/// Abstract overhang width (em) allowed past the line measure for `class`.
///
/// Returns [`HANG_WIDTH_EM`] for [`is_hangable`] classes, else `0.0`.
/// Not full JLReq hanging / justification.
pub fn hang_width_em(class: CharClass) -> f64 {
    if is_hangable(class) {
        HANG_WIDTH_EM
    } else {
        0.0
    }
}

/// [`hang_width_em`] after [`classify_char`].
pub fn hang_width_em_char(c: char) -> f64 {
    hang_width_em(classify_char(c))
}

/// Dimension of the §C-inspired class×class break matrix (`Other`=0 .. `cl-30`=30).
pub const BREAK_PAIR_MATRIX_DIM: usize = 31;

/// §C-inspired class×class break pair matrix for classes we implement.
///
/// Index by [`CharClass::id`] (`0` = [`CharClass::Other`], `1..=30` = cl-01..cl-30).
/// Densifies the former ad-hoc `break_opportunity` rules into a table: inseparable
/// runs, line-head / line-end kinsoku, and digit-open / prefix-digit / stop-open
/// quirks aligned with package `sample-pair-rules` / `numeric-before-close`.
/// **Not** the full normative JLReq appendix C matrix.
pub static BREAK_PAIR_MATRIX: [[BreakOpportunity; BREAK_PAIR_MATRIX_DIM]; BREAK_PAIR_MATRIX_DIM] =
    build_break_pair_matrix();

const fn build_break_pair_matrix(
) -> [[BreakOpportunity; BREAK_PAIR_MATRIX_DIM]; BREAK_PAIR_MATRIX_DIM] {
    let mut m = [[BreakOpportunity::Allowed; BREAK_PAIR_MATRIX_DIM]; BREAK_PAIR_MATRIX_DIM];
    let mut prev = 0usize;
    while prev < BREAK_PAIR_MATRIX_DIM {
        let mut next = 0usize;
        while next < BREAK_PAIR_MATRIX_DIM {
            m[prev][next] = break_pair_cell_rules(prev as u8, next as u8);
            next += 1;
        }
        prev += 1;
    }
    m
}

/// Class-id rules used to fill [`BREAK_PAIR_MATRIX`] (same semantics as lookup).
const fn break_pair_cell_rules(prev_id: u8, next_id: u8) -> BreakOpportunity {
    // cl-08 inseparable with anything (including itself).
    if prev_id == 8 || next_id == 8 {
        return BreakOpportunity::Inseparable;
    }
    // Western letter runs (cl-24/25/27) stay together.
    if is_western_run_id(prev_id) && is_western_run_id(next_id) {
        return BreakOpportunity::Inseparable;
    }
    // Numeric / grouped-numeral runs.
    if prev_id == 20 && next_id == 20 {
        return BreakOpportunity::Inseparable;
    }
    if prev_id == 18 && next_id == 18 {
        return BreakOpportunity::Inseparable;
    }
    // Line-end / line-head kinsoku (class-level).
    if is_line_end_prohibited_id(prev_id) || is_line_head_prohibited_id(next_id) {
        return BreakOpportunity::Prohibited;
    }
    // Digit-open / digit-unit quirks (package numeric-before-close + unit).
    // Digit+close/postfix already covered by line-head on next; open/unit are not.
    if prev_id == 20 && (next_id == 1 || next_id == 21) {
        return BreakOpportunity::Prohibited;
    }
    // Full stop / comma before open (package sample-pair-rules).
    if (prev_id == 6 || prev_id == 7) && next_id == 1 {
        return BreakOpportunity::Prohibited;
    }
    BreakOpportunity::Allowed
}

const fn is_western_run_id(id: u8) -> bool {
    matches!(id, 24 | 25 | 27)
}

const fn is_line_head_prohibited_id(id: u8) -> bool {
    matches!(id, 2 | 3 | 4 | 5 | 6 | 7 | 9 | 10 | 11 | 13 | 26)
}

const fn is_line_end_prohibited_id(id: u8) -> bool {
    matches!(id, 1 | 12 | 28 | 29)
}

/// Look up [`BREAK_PAIR_MATRIX`] by class id (`0..=30`). Out-of-range → [`Allowed`].
pub fn break_pair_matrix_cell(prev_id: u8, next_id: u8) -> BreakOpportunity {
    let p = prev_id as usize;
    let n = next_id as usize;
    if p >= BREAK_PAIR_MATRIX_DIM || n >= BREAK_PAIR_MATRIX_DIM {
        return BreakOpportunity::Allowed;
    }
    BREAK_PAIR_MATRIX[p][n]
}

/// Decide break opportunity between adjacent classified characters.
///
/// Uses the denser §C-inspired [`BREAK_PAIR_MATRIX`] (not normative appendix C).
/// Prefer calling this from future layout; package `japanese/linebreak`
/// `break-between` is a synthetic RPX mirror for Core imports.
pub fn break_opportunity(prev: CharClass, next: CharClass) -> BreakOpportunity {
    break_pair_matrix_cell(prev.id(), next.id())
}

/// Classify two chars and return the break opportunity between them.
pub fn break_opportunity_chars(prev: char, next: char) -> BreakOpportunity {
    break_opportunity(classify_char(prev), classify_char(next))
}

/// Approximate advance width in em for linebreak budgeting.
///
/// Naive heuristic: ASCII ≈ 0.5 em, everything else (ideograph-like) ≈ 1 em.
pub fn char_em_width(c: char) -> f64 {
    if c.is_ascii() {
        0.5
    } else {
        1.0
    }
}

/// Extend `end` through a contiguous cl-08 run (may overrun the em budget).
///
/// Keeps ellipsis / ditto / vertical-kana-repeat sequences intact so soft-wrap
/// does not open a cut between two [`CharClass::Inseparable`] glyphs.
fn extend_end_through_cl08_run(chars: &[char], start: usize, mut end: usize) -> usize {
    if end <= start || end == 0 {
        return end;
    }
    while end < chars.len()
        && classify_char(chars[end]) == CharClass::Inseparable
        && classify_char(chars[end - 1]) == CharClass::Inseparable
    {
        end += 1;
    }
    end
}

/// Walk `cut` left when it would split a cl-08×cl-08 pair (after kinsoku / force).
fn snap_cut_off_cl08_run(chars: &[char], start: usize, mut cut: usize) -> usize {
    while cut > start + 1
        && cut < chars.len()
        && classify_char(chars[cut - 1]) == CharClass::Inseparable
        && classify_char(chars[cut]) == CharClass::Inseparable
    {
        cut -= 1;
    }
    cut
}

/// Choose soft-wrap cut index in `(start, end]` (starts the next line).
///
/// Prefers an [`BreakOpportunity::Allowed`] break **before** an ASCII space when
/// one exists (western word wrap). Otherwise the rightmost Allowed cut; if none,
/// forces `end` (may split a long latin run when no space is available).
fn choose_soft_wrap_cut(chars: &[char], start: usize, end: usize) -> usize {
    let mut cut = end;
    let mut found = false;
    let mut space_cut: Option<usize> = None;
    for cand in (start + 1..=end).rev() {
        if cand < chars.len() && break_opportunity_chars(chars[cand - 1], chars[cand]).may_break() {
            if !found {
                cut = cand;
                found = true;
            }
            if chars[cand].is_ascii_whitespace() {
                space_cut = Some(cand);
                break;
            }
        }
    }
    if let Some(s) = space_cut {
        s
    } else if found {
        cut
    } else {
        end
    }
}

/// Soft-wrap `text` into lines of at most `max_em_units` em (fontless).
///
/// Uses [`break_opportunity`] between adjacent characters and [`char_em_width`].
/// Applies class-level kinsoku so e.g. `。` does not start a line when an earlier
/// break exists. Not a full JLReq / CSS line breaker.
///
/// **Hangable stub:** if the next character would overflow but its class is
/// [`is_hangable`] (cl-06/07), it may still be appended past the measure. This
/// is not full JLReq hanging punctuation or justification.
///
/// **Inseparable run glue:** contiguous cl-08 sequences are kept together
/// (budget overrun stub) and cuts are snapped off cl-08×cl-08 pairs.
///
/// **Western soft-wrap:** prefers a break before ASCII space when available so
/// mid-latin cuts are avoided when a word boundary exists in the window.
///
/// `max_em_units <= 0` returns the whole string as one line (empty input → empty vec).
pub fn break_line(text: &str, max_em_units: f64) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    if !(max_em_units > 0.0) {
        return vec![text.to_string()];
    }

    let mut out = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        // Greedy fit: take as many chars as fit (at least one, even if over-wide).
        let mut end = start;
        let mut used = 0.0;
        while end < chars.len() {
            let w = char_em_width(chars[end]);
            if end > start && used + w > max_em_units {
                // Optional hang: cl-06/07 may stick past the em budget (stub).
                if is_hangable(classify_char(chars[end])) {
                    end += 1;
                }
                break;
            }
            used += w;
            end += 1;
        }
        // cl-08 glue: finish an inseparable run even if past measure.
        end = extend_end_through_cl08_run(&chars, start, end);

        if end >= chars.len() {
            out.push(chars[start..].iter().collect());
            break;
        }

        let mut cut = choose_soft_wrap_cut(&chars, start, end);

        // Line-end kinsoku: don't finish on opening brackets / prefixed abbrevs.
        while cut > start + 1 && is_line_end_prohibited(classify_char(chars[cut - 1])) {
            cut -= 1;
        }
        // Line-start kinsoku: don't leave 。 etc. at the head of the remainder.
        while cut > start + 1
            && cut < chars.len()
            && is_line_head_prohibited(classify_char(chars[cut]))
        {
            cut -= 1;
        }
        cut = snap_cut_off_cl08_run(&chars, start, cut);

        // Ensure progress even if the whole window is kinsoku-stuck.
        if cut <= start {
            cut = start + 1;
        }

        out.push(chars[start..cut].iter().collect());
        start = cut;
    }
    out
}

/// Naive horizontal justification: place glyphs so the line spans `target_em`.
///
/// Extra space (`target_em − Σ char_em_width`) is split evenly across gaps where
/// [`break_opportunity`] is [`BreakOpportunity::Allowed`]. If there are no such
/// gaps (or `target_em` ≤ natural width), glyphs pack left with natural advances.
/// Returns `(char, x_em)` left edges from 0. Not CSS/`text-justify` / JLReq.
pub fn justify_line(chars: &[char], target_em: f64) -> Vec<(char, f64)> {
    if chars.is_empty() {
        return Vec::new();
    }
    let widths: Vec<f64> = chars.iter().copied().map(char_em_width).collect();
    let natural: f64 = widths.iter().sum();
    let mut gap_extra = vec![0.0_f64; chars.len().saturating_sub(1)];
    if target_em > natural && chars.len() > 1 {
        let mut allowed_idx = Vec::new();
        for i in 0..chars.len() - 1 {
            if break_opportunity_chars(chars[i], chars[i + 1]).may_break() {
                allowed_idx.push(i);
            }
        }
        if !allowed_idx.is_empty() {
            let each = (target_em - natural) / allowed_idx.len() as f64;
            for &i in &allowed_idx {
                gap_extra[i] = each;
            }
        }
    }
    let mut out = Vec::with_capacity(chars.len());
    let mut x = 0.0_f64;
    for (i, &ch) in chars.iter().enumerate() {
        out.push((ch, x));
        x += widths[i];
        if i < gap_extra.len() {
            x += gap_extra[i];
        }
    }
    out
}

/// Fontless glyph extent stub `(width_em, height_em)` for vertical metrics.
///
/// Half-width western / numeric / ASCII spaces ≈ `(0.5, 1.0)`; square CJK /
/// fullwidth letters ≈ `(1.0, 1.0)`. Not OpenType `vmtx` / real glyph bounds.
fn glyph_extent_em(c: char) -> (f64, f64) {
    match classify_char(c) {
        CharClass::WesternCharacters
        | CharClass::SimpleWestern
        | CharClass::ComplexWestern
        | CharClass::Numeric
        | CharClass::GroupedNumerals
        | CharClass::AttachedWestern
        | CharClass::Spaces => (0.5, 1.0),
        CharClass::Hyphens | CharClass::DividingPunctuation | CharClass::MiddleDots
            if c.is_ascii() =>
        {
            (0.5, 1.0)
        }
        CharClass::Other if c.is_ascii() => (0.5, 1.0),
        _ => (1.0, 1.0),
    }
}

/// Approximate vertical advance (em) for `vertical-rl` line budgeting.
///
/// Uses [`needs_tate_rotation`] + a taller/wider swap: rotated glyphs advance by
/// their horizontal width (former width becomes the line measure); upright
/// glyphs advance by height. Square CJK stay ≈ 1 em; ASCII western ≈ 0.5 em
/// after rotation. Hangable punctuation still advances (no special vertical hang).
/// Not tate-chu-yoko / OpenType `vmtx`.
pub fn vertical_advance_em(c: char) -> f64 {
    let (width_em, height_em) = glyph_extent_em(c);
    if needs_tate_rotation(c) {
        width_em
    } else {
        height_em
    }
}

/// Fontless vertical glyph orientation for `vertical-rl` (tategaki) stubs.
///
/// Not OpenType `vert` / `vrt2` or CSS `text-orientation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalGlyphOrientation {
    /// Keep the glyph upright (typical CJK square letters).
    Upright,
    /// Rotate a quarter-turn for tategaki (typical ASCII / Latin-1 western).
    Rotated,
}

impl VerticalGlyphOrientation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upright => "upright",
            Self::Rotated => "rotated",
        }
    }

    pub fn needs_rotation(self) -> bool {
        matches!(self, Self::Rotated)
    }
}

impl fmt::Display for VerticalGlyphOrientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classify vertical glyph orientation for `c` (fontless stub).
///
/// ASCII / Latin-1 western letters and digits → [`Rotated`]; CJK square letters
/// and fullwidth latin → [`Upright`]. Tate-chu-yoko / vertical presentation
/// forms remain future work.
pub fn vertical_glyph_orientation(c: char) -> VerticalGlyphOrientation {
    let rotated = match classify_char(c) {
        CharClass::ComplexWestern => true,
        CharClass::WesternCharacters
        | CharClass::SimpleWestern
        | CharClass::Numeric
        | CharClass::GroupedNumerals
        | CharClass::AttachedWestern => c.is_ascii(),
        CharClass::Spaces => c.is_ascii_whitespace(),
        CharClass::Hyphens | CharClass::DividingPunctuation | CharClass::MiddleDots
            if c.is_ascii() =>
        {
            true
        }
        CharClass::Other if c.is_ascii() => true,
        _ => false,
    };
    if rotated {
        VerticalGlyphOrientation::Rotated
    } else {
        VerticalGlyphOrientation::Upright
    }
}

/// Whether `c` should be rotated in `vertical-rl` — alias of
/// [`VerticalGlyphOrientation::needs_rotation`] via [`vertical_glyph_orientation`].
pub fn needs_tate_rotation(c: char) -> bool {
    vertical_glyph_orientation(c).needs_rotation()
}

/// Soft-wrap for `vertical-rl` using [`vertical_advance_em`] + [`break_opportunity`].
///
/// Same kinsoku / hangable / cl-08 glue stubs as [`break_line`], but the measure
/// is vertical advance along the line. Optional scaffolding only — not JLReq
/// vertical layout.
pub fn break_line_vertical(text: &str, max_em_units: f64) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    if !(max_em_units > 0.0) {
        return vec![text.to_string()];
    }

    let mut out = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let mut end = start;
        let mut used = 0.0;
        while end < chars.len() {
            let w = vertical_advance_em(chars[end]);
            if end > start && used + w > max_em_units {
                if is_hangable(classify_char(chars[end])) {
                    end += 1;
                }
                break;
            }
            used += w;
            end += 1;
        }
        end = extend_end_through_cl08_run(&chars, start, end);

        if end >= chars.len() {
            out.push(chars[start..].iter().collect());
            break;
        }

        let mut cut = choose_soft_wrap_cut(&chars, start, end);

        while cut > start + 1 && is_line_end_prohibited(classify_char(chars[cut - 1])) {
            cut -= 1;
        }
        while cut > start + 1
            && cut < chars.len()
            && is_line_head_prohibited(classify_char(chars[cut]))
        {
            cut -= 1;
        }
        cut = snap_cut_off_cl08_run(&chars, start, cut);

        if cut <= start {
            cut = start + 1;
        }

        out.push(chars[start..cut].iter().collect());
        start = cut;
    }
    out
}

/// First-line paragraph indent stub (JA 字下げ).
///
/// Returns `(x_em, line)` pairs: the first line is offset by `indent_em` (clamped
/// at ≥ 0); subsequent lines stay at `0.0`. Empty input → empty output. Not
/// hanging indent / kihon placement / full JLReq paragraph composition.
pub fn indent_first_line(lines: &[String], indent_em: f64) -> Vec<(f64, String)> {
    let indent = if indent_em > 0.0 { indent_em } else { 0.0 };
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let x = if i == 0 { indent } else { 0.0 };
            (x, line.clone())
        })
        .collect()
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

    /// Baseline-to-baseline pitch in em (`char_size_em × line_rate`).
    ///
    /// Pass to [`place_lines_horizontal`] / [`place_lines_vertical`] for kihon
    /// line spacing. Fontless stub — not full JLReq hanmen composition.
    pub fn line_pitch_em(self) -> f64 {
        self.char_size_em * self.line_rate
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

/// Annotation glyph scale vs base (fontless ruby stub; ~half-size furigana).
pub const RUBY_ANNOTATION_SCALE: f64 = 0.5;

/// Extra block height (em) reserved above the base for the ruby band.
pub const RUBY_HEIGHT_BUMP_EM: f64 = 0.5;

/// Fontless ruby metrics (abstract em). Not JLReq placement / jukugo distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RubyBox {
    pub base_width: f64,
    pub annotation_width: f64,
    /// Combined advance: `max(base_width, annotation_width)`.
    pub advance_width: f64,
    /// Base line box (~1 em) plus optional ruby band bump.
    pub height: f64,
}

impl RubyBox {
    pub const fn new(
        base_width: f64,
        annotation_width: f64,
        advance_width: f64,
        height: f64,
    ) -> Self {
        Self {
            base_width,
            annotation_width,
            advance_width,
            height,
        }
    }
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

    /// Estimate advance / height without fonts (layout scaffolding only).
    pub fn estimate_box(&self) -> RubyBox {
        ruby_estimate_box(self)
    }
}

/// Estimate ruby box metrics from [`Ruby`] (same as [`Ruby::estimate_box`]).
pub fn ruby_estimate_box(ruby: &Ruby) -> RubyBox {
    let base_width: f64 = ruby.base.chars().map(char_em_width).sum();
    let annotation_width: f64 =
        ruby.annotation.chars().map(char_em_width).sum::<f64>() * RUBY_ANNOTATION_SCALE;
    let advance_width = base_width.max(annotation_width);
    // Base character band (~1 em) + optional ruby-overhang bump.
    // `kind` is reserved for future jukugo distribution heuristics.
    let height = match ruby.kind {
        RubyKind::Simple | RubyKind::Jukugo => 1.0 + RUBY_HEIGHT_BUMP_EM,
    };
    RubyBox::new(base_width, annotation_width, advance_width, height)
}

/// Side offset (em) of vertical-ruby annotation from the base glyph axis.
///
/// In `vertical-rl`, furigana sits toward the line-start side; this stub uses a
/// fixed positive offset (host may mirror for rl). Not JLReq ruby placement.
pub const VERTICAL_RUBY_SIDE_EM: f64 = 0.55;

/// Fontless vertical-ruby metrics — annotation beside the base in vertical text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerticalRubyBox {
    /// Sum of [`vertical_advance_em`] over the base.
    pub base_advance: f64,
    /// Scaled annotation advance along the vertical measure.
    pub annotation_advance: f64,
    /// Combined vertical advance: `max(base_advance, annotation_advance)`.
    pub advance: f64,
    /// Inline (perpendicular) extent: ~1 em base + side ruby band.
    pub inline_em: f64,
    /// Annotation axis offset from the base (positive = annotation side).
    pub annotation_side_x: f64,
}

impl VerticalRubyBox {
    pub const fn new(
        base_advance: f64,
        annotation_advance: f64,
        advance: f64,
        inline_em: f64,
        annotation_side_x: f64,
    ) -> Self {
        Self {
            base_advance,
            annotation_advance,
            advance,
            inline_em,
            annotation_side_x,
        }
    }
}

/// Estimate vertical-ruby metrics (annotation to the side — estimate only).
pub fn vertical_ruby_estimate_box(ruby: &Ruby) -> VerticalRubyBox {
    let base_advance: f64 = ruby.base.chars().map(vertical_advance_em).sum();
    let annotation_advance: f64 = ruby
        .annotation
        .chars()
        .map(vertical_advance_em)
        .sum::<f64>()
        * RUBY_ANNOTATION_SCALE;
    let advance = base_advance.max(annotation_advance);
    let inline_em = 1.0 + VERTICAL_RUBY_SIDE_EM;
    VerticalRubyBox::new(
        base_advance,
        annotation_advance,
        advance,
        inline_em,
        VERTICAL_RUBY_SIDE_EM,
    )
}

impl Ruby {
    /// Vertical-writing ruby estimate (side annotation); see [`vertical_ruby_estimate_box`].
    pub fn estimate_vertical_box(&self) -> VerticalRubyBox {
        vertical_ruby_estimate_box(self)
    }
}

/// Bou (傍点) mark size (em) — sesame/emphasis dots beside glyphs (fontless stub).
pub const BOU_MARK_SIZE_EM: f64 = 0.25;

/// Side offset (em) of bou marks from the glyph axis in vertical text.
pub const BOU_SIDE_OFFSET_EM: f64 = 0.55;

/// Fontless bou / emphasis placement estimate — package `ja-emphasis` companion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BouBox {
    /// Vertical (or inline) advance of the marked body.
    pub advance: f64,
    /// Inline pad reserved for marks.
    pub side_em: f64,
    /// Mark glyph size (em).
    pub mark_size: f64,
}

impl BouBox {
    pub const fn new(advance: f64, side_em: f64, mark_size: f64) -> Self {
        Self {
            advance,
            side_em,
            mark_size,
        }
    }
}

/// Estimate bou (傍点) band metrics for an emphasis body string.
pub fn bou_estimate_box(body: &str) -> BouBox {
    let advance: f64 = body.chars().map(vertical_advance_em).sum();
    BouBox::new(
        advance,
        BOU_SIDE_OFFSET_EM + BOU_MARK_SIZE_EM * 0.5,
        BOU_MARK_SIZE_EM,
    )
}

/// Per-glyph bou mark centers: `(advance_along, side_x)` from the start of `body`.
///
/// Marks sit at mid-glyph along the advance axis and at [`BOU_SIDE_OFFSET_EM`]
/// to the side. Heuristic only — not JLReq / CSS `text-emphasis` positioning.
pub fn bou_mark_offsets(body: &str) -> Vec<(f64, f64)> {
    let mut along = 0.0_f64;
    let mut out = Vec::new();
    for c in body.chars() {
        let adv = vertical_advance_em(c);
        out.push((along + adv * 0.5, BOU_SIDE_OFFSET_EM));
        along += adv;
    }
    out
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

    /// Horizontal run width of digits/latin set upright in vertical text (fontless).
    pub fn estimate_box(&self) -> TateChuYokoBox {
        tate_chu_yoko_estimate_box(self)
    }
}

/// Fontless tate-chu-yoko metrics: inline advance of a horizontal span in vertical context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TateChuYokoBox {
    /// Sum of [`char_em_width`] over the body (ASCII half-em, else full em).
    pub advance_width: f64,
    /// Vertical band occupied by the run (~1 em of the surrounding vertical measure).
    pub block_em: f64,
}

impl TateChuYokoBox {
    pub const fn new(advance_width: f64, block_em: f64) -> Self {
        Self {
            advance_width,
            block_em,
        }
    }
}

/// Estimate tate-chu-yoko box (same as [`TateChuYoko::estimate_box`]).
pub fn tate_chu_yoko_estimate_box(span: &TateChuYoko) -> TateChuYokoBox {
    let advance_width: f64 = span.body.chars().map(char_em_width).sum();
    TateChuYokoBox::new(advance_width, 1.0)
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

/// Place lines for horizontal-tb writing: block axis is **y** (downward).
///
/// Returns `(line, y)` with `y = start_y + i × pitch_em`. Use
/// [`KihonHanmen::line_pitch_em`] for kihon baseline pitch. Fontless stub —
/// not full JLReq / CSS line-height composition.
pub fn place_lines_horizontal<S: AsRef<str>>(
    lines: &[S],
    start_y: f64,
    pitch_em: f64,
) -> Vec<(String, f64)> {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| (line.as_ref().to_string(), start_y + i as f64 * pitch_em))
        .collect()
}

/// Place lines for vertical-rl writing: block axis is **x** (column advance).
///
/// Returns `(line, x)` with `x = start_x + i × pitch_em`. Positive pitch steps
/// rightward; hosts using vertical-rl may negate for column progression.
/// Fontless stub — not OpenType `vert` / full tategaki composition.
pub fn place_lines_vertical<S: AsRef<str>>(
    lines: &[S],
    start_x: f64,
    pitch_em: f64,
) -> Vec<(String, f64)> {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| (line.as_ref().to_string(), start_x + i as f64 * pitch_em))
        .collect()
}

/// Place each line as a scene [`reciplexa_scene::Text`] shape (host/layout scaffolding).
///
/// Uses [`place_lines_horizontal`] for y placement from `(x_mm, y_mm)` with step
/// `leading_mm` (treat as pitch in the same unit as coordinates). Not glyph
/// shaping — content strings only.
pub fn lines_to_text_shapes<S: AsRef<str>>(
    lines: &[S],
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    leading_mm: f64,
    fill: reciplexa_scene::Color,
) -> Vec<reciplexa_scene::Text> {
    place_lines_horizontal(lines, y_mm, leading_mm)
        .into_iter()
        .map(|(content, y)| reciplexa_scene::Text {
            x_mm,
            y_mm: y,
            size_mm,
            width_mm: None,
            height_mm: None,
            content,
            fill,
        })
        .collect()
}

/// Place each line as a scene [`reciplexa_scene::Text`] for vertical-rl columns.
///
/// Uses [`place_lines_vertical`] for **x** placement from `(x_mm, y_mm)` with
/// column pitch `leading_mm` (same unit as coordinates). Shared `y_mm` per
/// column; positive pitch steps rightward (hosts may negate for vertical-rl).
/// Fontless stub — not OpenType `vert` / full tategaki composition.
pub fn lines_to_vertical_text_shapes<S: AsRef<str>>(
    lines: &[S],
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    leading_mm: f64,
    fill: reciplexa_scene::Color,
) -> Vec<reciplexa_scene::Text> {
    place_lines_vertical(lines, x_mm, leading_mm)
        .into_iter()
        .map(|(content, x)| reciplexa_scene::Text {
            x_mm: x,
            y_mm,
            size_mm,
            width_mm: None,
            height_mm: None,
            content,
            fill,
        })
        .collect()
}

/// Soft-wrap with [`break_line`], then [`lines_to_text_shapes`].
pub fn break_line_to_text_shapes(
    text: &str,
    max_em_units: f64,
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    leading_mm: f64,
    fill: reciplexa_scene::Color,
) -> Vec<reciplexa_scene::Text> {
    let lines = break_line(text, max_em_units);
    lines_to_text_shapes(&lines, x_mm, y_mm, size_mm, leading_mm, fill)
}

/// Soft-wrap a scene [`Text`]'s `content` via [`break_line`].
///
/// Reuses `text.x_mm` / `y_mm` / `size_mm` / `fill`; subsequent lines step by
/// `leading_mm`. `max_em_units` is the fontless break budget (same as
/// [`break_line`]) — `width_mm` is not interpreted as an em budget.
/// Host scaffolding only; not GUI text_box production layout.
pub fn wrap_text_shape_content(
    text: &reciplexa_scene::Text,
    max_em_units: f64,
    leading_mm: f64,
) -> Vec<reciplexa_scene::Text> {
    break_line_to_text_shapes(
        &text.content,
        max_em_units,
        text.x_mm,
        text.y_mm,
        text.size_mm,
        leading_mm,
        text.fill,
    )
}

/// Justify one line with [`justify_line`], then place each glyph as a scene [`Text`].
///
/// `em_mm` converts abstract em x-offsets to millimeters (`x_mm + x_em * em_mm`).
/// One Text per glyph — host scaffolding only, not CSS/`text-justify` / JLReq.
pub fn justify_line_to_text_shapes(
    text: &str,
    target_em: f64,
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    em_mm: f64,
    fill: reciplexa_scene::Color,
) -> Vec<reciplexa_scene::Text> {
    let chars: Vec<char> = text.chars().collect();
    let placed = justify_line(&chars, target_em);
    placed
        .into_iter()
        .map(|(ch, x_em)| reciplexa_scene::Text {
            x_mm: x_mm + x_em * em_mm,
            y_mm,
            size_mm,
            width_mm: None,
            height_mm: None,
            content: ch.to_string(),
            fill,
        })
        .collect()
}
