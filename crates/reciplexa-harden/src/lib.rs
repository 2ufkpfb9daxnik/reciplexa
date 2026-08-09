//! Hardening — decode budgets, sandbox fences, fuzz hooks, audits (Phase 13).

#![forbid(unsafe_code)]

pub mod budget;

pub use budget::{BudgetExhausted, BudgetKind, DecodeBudget};
