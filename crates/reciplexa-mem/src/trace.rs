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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_records_all_event_kinds() {
        let mut trace = RcTrace::default();
        trace.record_alloc(Reg(0));
        trace.record_dup(Reg(1), Reg(0));
        trace.record_drop(Reg(0));
        trace.record_reuse(Reg(2), Reg(1));
        trace.record_cleanup("scope".into());
        assert_eq!(trace.allocs, vec![Reg(0)]);
        assert_eq!(trace.dups, vec![(Reg(1), Reg(0))]);
        assert_eq!(trace.drops, vec![Reg(0)]);
        assert_eq!(trace.reuses, vec![(Reg(2), Reg(1))]);
        assert_eq!(trace.cleanups, vec!["scope"]);
        assert_eq!(trace.drop_count(Reg(0)), 1);
        assert_eq!(trace.drop_count(Reg(9)), 0);
    }
}
