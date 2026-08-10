//! SYN-001 reserved special-form names.
//!
//! Binding these as `val` / `fn` / `let` / … names is a static error
//! (SYN-001 §6 Core special forms + surface special forms / literals).

/// Returns true when `name` must not be used as a binder.
pub fn is_reserved_special_form(name: &str) -> bool {
    matches!(
        name,
        // SYN-001 §6 Core special forms
        "markup"
            | "fn"
            | "val"
            | "type"
            | "type-alias"
            | "local"
            | "rec"
            | "let"
            | "letrec"
            | "if"
            | "seq"
            | "var"
            | "set"
            | "handle"
            | "with"
            // Surface special forms & literals
            | "data"
            | "match"
            | "record"
            | "field"
            | "list"
            | "tuple"
            | "perform"
            | "macro"
            | "true"
            | "false"
            | "unit"
    )
}
