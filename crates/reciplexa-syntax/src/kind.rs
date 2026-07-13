//! Syntax kinds for the reciplexa lossless tree.
//!
//! We map 1:1 onto `rowan::SyntaxKind` via `u16` tags. Tokens and future
//! composite node kinds share one enum so the green tree stays uniform.
//! Conversion is match-based (no `unsafe`) to keep `forbid(unsafe_code)`.

use rowan::Language;

/// All token and (later) node kinds in the CST.
///
/// Token variants mirror the hand-written lexer's `TokenKind` surface.
/// Composite node kinds will be added when the parser lands; keeping them
/// in one enum avoids a second mapping layer between lexer and rowan.
///
/// `__Last` is a discriminant sentinel for density / bounds tests—not an
/// API stability marker—so we intentionally avoid `#[non_exhaustive]`.
#[allow(clippy::manual_non_exhaustive)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // --- punct ---
    LParen,
    RParen,
    At,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // --- data ---
    Ident,
    Number,
    String,
    TextChunk,

    // --- trivia (must remain in the tree for lossless editing) ---
    Whitespace,
    Newline,
    Comment,

    // --- error recovery leaf ---
    Error,

    // --- composite nodes (parser) ---
    SourceFile,
    /// `( … )` list form.
    List,
    /// `[ … ]` list form (data / args).
    BracketList,
    /// `{ … }` group (Scribble bodies; also allowed in Lisp for symmetry).
    BraceList,
    /// `@` escape: `@ident`, `@ident{…}`, or `@(...)`.
    AtExpr,
    /// Wrapper around a skipped/unexpected span when fail-fast collection runs.
    ErrorNode,

    // --- sentinel: must stay last for raw-tag bounds checks in tests ---
    #[doc(hidden)]
    __Last,
}

impl SyntaxKind {
    /// Convert a raw rowan tag back into a kind.
    ///
    /// Unknown tags become `Error` rather than panicking: a corrupt or
    /// forward-incompatible green tree should surface as a syntax error,
    /// not abort the host process (fail-fast for *programs*, not for the engine).
    pub fn from_raw(raw: rowan::SyntaxKind) -> Self {
        match raw.0 {
            x if x == Self::LParen as u16 => Self::LParen,
            x if x == Self::RParen as u16 => Self::RParen,
            x if x == Self::At as u16 => Self::At,
            x if x == Self::LBrace as u16 => Self::LBrace,
            x if x == Self::RBrace as u16 => Self::RBrace,
            x if x == Self::LBracket as u16 => Self::LBracket,
            x if x == Self::RBracket as u16 => Self::RBracket,
            x if x == Self::Ident as u16 => Self::Ident,
            x if x == Self::Number as u16 => Self::Number,
            x if x == Self::String as u16 => Self::String,
            x if x == Self::TextChunk as u16 => Self::TextChunk,
            x if x == Self::Whitespace as u16 => Self::Whitespace,
            x if x == Self::Newline as u16 => Self::Newline,
            x if x == Self::Comment as u16 => Self::Comment,
            x if x == Self::Error as u16 => Self::Error,
            x if x == Self::SourceFile as u16 => Self::SourceFile,
            x if x == Self::List as u16 => Self::List,
            x if x == Self::BracketList as u16 => Self::BracketList,
            x if x == Self::BraceList as u16 => Self::BraceList,
            x if x == Self::AtExpr as u16 => Self::AtExpr,
            x if x == Self::ErrorNode as u16 => Self::ErrorNode,
            _ => Self::Error,
        }
    }

    pub fn to_raw(self) -> rowan::SyntaxKind {
        rowan::SyntaxKind(self as u16)
    }

    pub const fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }

    pub const fn is_token(self) -> bool {
        !matches!(
            self,
            Self::SourceFile
                | Self::List
                | Self::BracketList
                | Self::BraceList
                | Self::AtExpr
                | Self::ErrorNode
                | Self::__Last
        )
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        kind.to_raw()
    }
}

/// Rowan language marker for reciplexa green/red trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyntaxLanguage {}

impl Language for SyntaxLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from_raw(raw)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.to_raw()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<SyntaxLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<SyntaxLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<SyntaxLanguage>;

#[cfg(test)]
mod tests {
    use super::*;
    use rowan::Language;

    // --- validity ---

    #[test]
    fn roundtrip_every_real_kind_through_language() {
        let kinds = [
            SyntaxKind::LParen,
            SyntaxKind::RParen,
            SyntaxKind::At,
            SyntaxKind::LBrace,
            SyntaxKind::RBrace,
            SyntaxKind::LBracket,
            SyntaxKind::RBracket,
            SyntaxKind::Ident,
            SyntaxKind::Number,
            SyntaxKind::String,
            SyntaxKind::TextChunk,
            SyntaxKind::Whitespace,
            SyntaxKind::Newline,
            SyntaxKind::Comment,
            SyntaxKind::Error,
            SyntaxKind::SourceFile,
            SyntaxKind::List,
            SyntaxKind::BracketList,
            SyntaxKind::BraceList,
            SyntaxKind::AtExpr,
            SyntaxKind::ErrorNode,
        ];
        for kind in kinds {
            let raw = SyntaxLanguage::kind_to_raw(kind);
            assert_eq!(SyntaxLanguage::kind_from_raw(raw), kind);
        }
    }

    #[test]
    fn composite_kinds_are_not_tokens() {
        assert!(!SyntaxKind::SourceFile.is_token());
        assert!(!SyntaxKind::List.is_token());
        assert!(!SyntaxKind::AtExpr.is_token());
        assert!(SyntaxKind::Ident.is_token());
        assert!(SyntaxKind::Whitespace.is_token());
    }

    #[test]
    fn trivia_predicate_matches_design_token_set() {
        assert!(SyntaxKind::Whitespace.is_trivia());
        assert!(SyntaxKind::Newline.is_trivia());
        assert!(SyntaxKind::Comment.is_trivia());
        assert!(!SyntaxKind::Ident.is_trivia());
        assert!(!SyntaxKind::LParen.is_trivia());
        assert!(!SyntaxKind::Error.is_trivia());
    }

    #[test]
    fn discriminants_are_dense_from_zero() {
        // Dense tags keep rowan maps small and make accidental gaps obvious.
        assert_eq!(SyntaxKind::LParen as u16, 0);
        assert_eq!(SyntaxKind::ErrorNode as u16 + 1, SyntaxKind::__Last as u16);
        assert!(SyntaxKind::SourceFile as u16 > SyntaxKind::Error as u16);
    }

    // --- defect ---

    #[test]
    fn unknown_raw_tag_becomes_error_not_panic() {
        let garbage = rowan::SyntaxKind(u16::MAX);
        assert_eq!(SyntaxKind::from_raw(garbage), SyntaxKind::Error);
    }

    #[test]
    fn hidden_last_sentinel_is_not_a_roundtrippable_token() {
        // __Last exists only as an upper bound; feeding its tag must not
        // invent a phantom token kind.
        let raw = rowan::SyntaxKind(SyntaxKind::__Last as u16);
        assert_eq!(SyntaxKind::from_raw(raw), SyntaxKind::Error);
    }
}
