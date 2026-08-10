//! DOCUMENT SURFACE typecheck tests (page / markup / shape prelude).
//!
//! These are **not** language-kernel semantics. Language TYP lives in
//! `reciplexa_core::typecheck_language_source` and `reciplexa-test` lang_kernel_suite.
//! Prototype package graphics remain here until PKG-001.
//!
//! Cases are nested under [`document_surface`] so this suite cannot be mistaken
//! for the language-kernel gate (`lang_kernel_suite`).

#[cfg(test)]
mod document_surface {
    use reciplexa_syntax::parse_source;
    use reciplexa_types::*;

    include!("support/document_surface_cases.rs");
}
