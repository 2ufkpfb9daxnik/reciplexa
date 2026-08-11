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

#[test]
fn interpreter_return_without_value() {
    let mut interp = Interpreter::new();
    let ops = vec![LoweredOp::Return];
    assert!(interp.run(&ops).is_ok());
}

#[test]
fn one_shot_verifier_accepts_single_resume() {
    let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Return];
    let mut conts = vec![Continuation::new(ContinuationId(1))];
    assert!(verify_one_shot(&ops, &mut conts).is_ok());
}

#[test]
fn scheduler_empty_run() {
    let mut sched = TestScheduler::new();
    assert!(sched.run_to_completion().is_empty());
}

#[test]
fn spawn_fail_fast_without_finish_errors_on_shutdown() {
    let mut root = RootScope::new();
    let (_scope, _handle) = root.spawn(SpawnPolicy::FailFast);
    assert!(root.shutdown().is_err());
}

#[test]
fn cancellation_token_cancel_flag() {
    let mut token = CancellationTokenSource::new(1);
    assert!(!token.is_cancelled());
    token.cancel();
    assert!(token.is_cancelled());
    assert_eq!(token.token().id(), 1);
}

#[test]
fn root_scope_shutdown_with_cleanup() {
    let mut root = RootScope::new();
    let (_scope, _handle) = root.spawn(SpawnPolicy::FailFast);
    let report = root.shutdown_with_cleanup();
    assert!(root.cleanup_ran());
    assert_eq!(
        report.cleanup_status,
        reciplexa_outcome::CleanupStatus::Completed
    );
}

#[test]
fn lowered_ir_resume_then_return() {
    let mut interp = Interpreter::new();
    interp.register_continuation(ContinuationId(3));
    let ops = vec![LoweredOp::Resume { cont: 3 }, LoweredOp::Return];
    assert!(interp.run(&ops).is_ok());
}

#[test]
fn eval_number_literal_pipeline() {
    let expr = CoreExpr::Lit(CoreLiteral::F64(7.5));
    let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    assert_eq!(format!("{v:?}"), "F64(7.5)");
}

#[test]
fn interpreter_unknown_resume_errors() {
    let mut interp = Interpreter::new();
    let ops = vec![LoweredOp::Resume { cont: 99 }];
    assert!(interp.run(&ops).is_err());
}

#[test]
fn root_scope_double_cancel_is_idempotent() {
    let mut root = RootScope::new();
    let mut token = CancellationTokenSource::new(2);
    let (_scope, handle) = root.spawn(SpawnPolicy::FailFast);
    root.child_finished(handle.task_id);
    let _ = root.cancel_all(&mut token);
    let _ = root.cancel_all(&mut token);
    assert!(token.is_cancelled());
}
