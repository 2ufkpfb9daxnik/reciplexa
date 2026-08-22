//! Typed text layout protocol (spec Part IV §17–18 spirit).

use crate::position::{Direction, WritingMode};

/// ISO 15924 script tag for shaping runs (stable within a run).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    Common,
    Latin,
    Han,
    Hiragana,
    Katakana,
    Arabic,
    Hebrew,
}

impl Script {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Common => "Zyyy",
            Self::Latin => "Latn",
            Self::Han => "Hani",
            Self::Hiragana => "Hira",
            Self::Katakana => "Kana",
            Self::Arabic => "Arab",
            Self::Hebrew => "Hebr",
        }
    }
}

/// Attributes held constant within one shaping run (spec §17 shaping run).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapingAttributes {
    pub script: Script,
    pub language: String,
    pub direction: Direction,
    pub writing_mode: WritingMode,
}

impl Default for ShapingAttributes {
    fn default() -> Self {
        Self {
            script: Script::Common,
            language: "und".into(),
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
        }
    }
}

impl ShapingAttributes {
    pub fn horizontal_ltr(language: impl Into<String>) -> Self {
        Self {
            script: Script::Common,
            language: language.into(),
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
        }
    }

    pub fn horizontal_rtl(language: impl Into<String>) -> Self {
        Self {
            script: Script::Arabic,
            language: language.into(),
            direction: Direction::Rtl,
            writing_mode: WritingMode::HorizontalTb,
        }
    }
}

/// Infer a coarse script from the first strong character in `text`.
pub fn infer_script(text: &str) -> Script {
    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            return Script::Latin;
        }
        if ('\u{3040}'..='\u{309F}').contains(&ch) {
            return Script::Hiragana;
        }
        if ('\u{30A0}'..='\u{30FF}').contains(&ch) {
            return Script::Katakana;
        }
        if ('\u{4E00}'..='\u{9FFF}').contains(&ch) {
            return Script::Han;
        }
        if ('\u{0590}'..='\u{05FF}').contains(&ch) {
            return Script::Hebrew;
        }
        if ('\u{0600}'..='\u{06FF}').contains(&ch) || ('\u{0750}'..='\u{077F}').contains(&ch) {
            return Script::Arabic;
        }
    }
    Script::Common
}
