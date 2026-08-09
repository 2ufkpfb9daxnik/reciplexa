//! Decode byte/operation budgets for untrusted codecs.

/// Exhaustion reason when a decode budget is exceeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetExhausted {
    pub limit: u64,
    pub attempted: u64,
    pub kind: BudgetKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetKind {
    Bytes,
    Operations,
    Allocations,
}

/// Hard caps that force decode to stop before resource exhaustion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeBudget {
    pub max_bytes: u64,
    pub max_operations: u64,
    pub max_allocations: u64,
    bytes_used: u64,
    operations_used: u64,
    allocations_used: u64,
}

impl Default for DecodeBudget {
    fn default() -> Self {
        Self::new(16 * 1024 * 1024, 1_000_000, 64 * 1024)
    }
}

impl DecodeBudget {
    pub fn new(max_bytes: u64, max_operations: u64, max_allocations: u64) -> Self {
        Self {
            max_bytes,
            max_operations,
            max_allocations,
            bytes_used: 0,
            operations_used: 0,
            allocations_used: 0,
        }
    }

    pub fn bytes_used(&self) -> u64 {
        self.bytes_used
    }

    pub fn operations_used(&self) -> u64 {
        self.operations_used
    }

    pub fn allocations_used(&self) -> u64 {
        self.allocations_used
    }

    pub fn charge_bytes(&mut self, n: u64) -> Result<(), BudgetExhausted> {
        let attempted = self.bytes_used.saturating_add(n);
        if attempted > self.max_bytes {
            return Err(BudgetExhausted {
                limit: self.max_bytes,
                attempted,
                kind: BudgetKind::Bytes,
            });
        }
        self.bytes_used = attempted;
        Ok(())
    }

    pub fn charge_operations(&mut self, n: u64) -> Result<(), BudgetExhausted> {
        let attempted = self.operations_used.saturating_add(n);
        if attempted > self.max_operations {
            return Err(BudgetExhausted {
                limit: self.max_operations,
                attempted,
                kind: BudgetKind::Operations,
            });
        }
        self.operations_used = attempted;
        Ok(())
    }

    pub fn charge_allocations(&mut self, n: u64) -> Result<(), BudgetExhausted> {
        let attempted = self.allocations_used.saturating_add(n);
        if attempted > self.max_allocations {
            return Err(BudgetExhausted {
                limit: self.max_allocations,
                attempted,
                kind: BudgetKind::Allocations,
            });
        }
        self.allocations_used = attempted;
        Ok(())
    }

    pub fn reset_usage(&mut self) {
        self.bytes_used = 0;
        self.operations_used = 0;
        self.allocations_used = 0;
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.max_bytes.saturating_sub(self.bytes_used)
    }
}
