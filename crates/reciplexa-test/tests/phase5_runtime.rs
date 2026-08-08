//! Phase 5 runtime integration conformance tests.

use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_eval::{eval_expr, UnitHost};
use reciplexa_ir::{
    eval_expr_to_ir, verify_one_shot, Continuation, ContinuationId, InterpretError, Interpreter,
    LoweredOp,
};
use reciplexa_runtime::{CancellationTokenSource, RootScope, SpawnPolicy, TaskId, TestScheduler};
use std::collections::HashMap;

#[test]
fn effect_lowering_and_eval_pipeline() {
    let expr = CoreExpr::Perform {
        op: "log".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::String("ok".into()))),
    };
    let lowered = eval_expr_to_ir(&expr);
    assert!(!lowered.0.ops.is_empty());
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

#[test]
fn lowered_ir_interpreter_runs_perform() {
    let mut interp = Interpreter::new();
    let ops = vec![
        LoweredOp::Perform {
            op: "log".to_string(),
        },
        LoweredOp::Return,
    ];
    assert!(interp.run(&ops).is_ok());
}

#[test]
fn interpreter_enforces_one_shot_resume() {
    let mut interp = Interpreter::new();
    interp.register_continuation(ContinuationId(7));
    let ops = vec![LoweredOp::Resume { cont: 7 }, LoweredOp::Resume { cont: 7 }];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::DoubleResume(_))
    ));
}

#[test]
fn cancellation_runs_cleanup_hooks() {
    let mut root = RootScope::new();
    root.on_cleanup("resource-1");
    let mut token = CancellationTokenSource::new(1);
    let (_scope, handle) = root.spawn(SpawnPolicy::FailFast);
    root.child_finished(handle.task_id);
    let _ = root.cancel_all(&mut token);
    assert!(root.cleanup_ran());
    assert_eq!(root.cleanup_hook_count(), 0);
}

#[test]
fn failure_raise_without_handler_errors() {
    let mut interp = Interpreter::new();
    let ops = vec![LoweredOp::Raise {
        tag: "fail".to_string(),
    }];
    assert!(interp.run(&ops).is_err());
}
