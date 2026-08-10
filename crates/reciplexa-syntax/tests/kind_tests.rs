//! Integration tests moved from src/kind.rs for region coverage.

use reciplexa_syntax::kind::*;

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
        SyntaxKind::StructuredComment,
        SyntaxKind::Arrow,
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
    assert!(!SyntaxKind::StructuredComment.is_token());
    assert!(SyntaxKind::Ident.is_token());
    assert!(SyntaxKind::Arrow.is_token());
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
    assert_eq!(SyntaxKind::Arrow as u16 + 1, SyntaxKind::__Last as u16);
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
