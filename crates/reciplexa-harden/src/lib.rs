//! Hardening — decode budgets, sandbox fences, fuzz hooks, audits (Phase 13).

#![forbid(unsafe_code)]

pub mod budget;
pub mod quota;
pub mod sandbox;

pub use budget::{BudgetExhausted, BudgetKind, DecodeBudget};
pub use quota::{QuotaExceeded, ResourceKind, ResourceQuota};
pub use sandbox::{SandboxDecision, SandboxFence, SandboxPolicy, SandboxRequirement};
