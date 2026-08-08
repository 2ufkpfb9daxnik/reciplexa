//! Phase 5 runtime integration smoke tests.

use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_eval::{eval_expr, UnitHost};
use reciplexa_ir::{lower_expr, verify_one_shot, Continuation, ContinuationId, LoweredOp};
use reciplexa_runtime::{RootScope, SpawnPolicy, TaskId, TestScheduler};
use std::collections::HashMap;

#[test]
fn effect_lowering_and_eval_pipeline() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("ok".into()))),
    };
    let lowered = lower_expr(&expr);
    assert!(!lowered.ops.is_empty());
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(format!("{v:?}"), "Unit");
}

#[test]
fn one_shot_verifier_blocks_double_resume() {
    let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Resume { cont: 1 }];
    let mut conts = vec![Continuation::new(ContinuationId(1))];
    assert!(verify_one_shot(&ops, &mut conts).is_err());
}

#[test]
fn structured_runtime_spawn_and_shutdown() {
    let mut root = RootScope::new();
    let (_scope, handle) = root.spawn(SpawnPolicy::FailFast);
    root.child_finished(handle.task_id);
    assert!(root.shutdown().is_ok());
}

#[test]
fn deterministic_scheduler_replays() {
    let mut sched = TestScheduler::new();
    sched.enqueue(TaskId(1));
    sched.enqueue(TaskId(2));
    let log = sched.run_to_completion();
    assert_eq!(log, vec![TaskId(1), TaskId(2)]);
}
