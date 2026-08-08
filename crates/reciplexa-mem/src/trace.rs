//! Debug resource trace for conservative RC (Step A).

use crate::reg::Reg;

/// Observed memory events during execution (diagnostic only).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RcTrace {
    pub allocs: Vec<Reg>,
    pub dups: Vec<(Reg, Reg)>,
    pub drops: Vec<Reg>,
    pub reuses: Vec<(Reg, Reg)>,
    pub cleanups: Vec<String>,
}

impl RcTrace {
    pub fn record_alloc(&mut self, reg: Reg) {
        self.allocs.push(reg);
    }

    pub fn record_dup(&mut self, dst: Reg, src: Reg) {
        self.dups.push((dst, src));
    }

    pub fn record_drop(&mut self, reg: Reg) {
        self.drops.push(reg);
    }

    pub fn record_reuse(&mut self, dst: Reg, src: Reg) {
        self.reuses.push((dst, src));
    }

    pub fn record_cleanup(&mut self, label: String) {
        self.cleanups.push(label);
    }

    pub fn drop_count(&self, reg: Reg) -> usize {
        self.drops.iter().filter(|r| **r == reg).count()
    }
}
