//! Host engine selector. Product is the production path; stub is explicit.

/// Layout engine used by hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypesetEngine {
    /// Fontless `reciplexa-std` heuristics (reference / differential).
    Stub,
    /// Font-backed JLReq / MATH engines in this crate.
    Product,
}

/// Production default is [`TypesetEngine::Product`].
///
/// Set `RECIPLEXA_TYPESET_ENGINE=stub` to force the reference heuristics.
/// Empty / unset / `product` → product.
pub fn host_typeset_engine() -> TypesetEngine {
    match std::env::var("RECIPLEXA_TYPESET_ENGINE") {
        Ok(s) if s.eq_ignore_ascii_case("stub") => TypesetEngine::Stub,
        _ => TypesetEngine::Product,
    }
}

impl TypesetEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stub => "stub",
            Self::Product => "product",
        }
    }
}
