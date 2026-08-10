//! Lexical scope and name resolution over syntax trees.
//!
//! Phase 1 binding layer — does not mutate the CST.

#![forbid(unsafe_code)]

pub mod module;
pub mod package;
pub mod resolve;
pub mod scope;

pub use module::{
    elaborate_units, ElaboratedUnit, ImportDecl, ModuleError, ModuleSkeleton, ModuleUnit,
};
pub use package::{resolve_package, PackageResolveResult};
pub use resolve::{
    resolve_language_source, resolve_source, BindingEnv, BindingMap, ResolveError, ResolveResult,
};
pub use scope::{ScopeStack, ScopeTree};
