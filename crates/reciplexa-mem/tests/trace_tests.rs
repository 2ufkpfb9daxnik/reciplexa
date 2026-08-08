use reciplexa_mem::reg::Reg;
use reciplexa_mem::trace::RcTrace;

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
