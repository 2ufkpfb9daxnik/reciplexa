use reciplexa_ir::{
    ContinuationId, HandlerFrame, HandlerKind, InterpretError, Interpreter, LoweredOp,
};

#[test]
fn default_interpreter_matches_new() {
    let a = Interpreter::default();
    let b = Interpreter::new();
    assert_eq!(a.continuations.len(), b.continuations.len());
    assert_eq!(a.handlers.len(), b.handlers.len());
}

#[test]
fn empty_program_returns_unit() {
    let mut interp = Interpreter::new();
    assert!(interp.run(&[]).is_ok());
}

#[test]
fn perform_without_return_falls_through() {
    let mut interp = Interpreter::new();
    let ops = vec![LoweredOp::Perform {
        op: "log".to_string(),
    }];
    assert!(interp.run(&ops).is_ok());
}

#[test]
fn perform_and_return() {
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
fn resume_once_succeeds() {
    let mut interp = Interpreter::new();
    interp.register_continuation(ContinuationId(1));
    let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Return];
    assert!(interp.run(&ops).is_ok());
}

#[test]
fn double_resume_fails() {
    let mut interp = Interpreter::new();
    interp.register_continuation(ContinuationId(1));
    let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Resume { cont: 1 }];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::DoubleResume(_))
    ));
}

#[test]
fn unknown_continuation_fails() {
    let mut interp = Interpreter::new();
    let ops = vec![LoweredOp::Resume { cont: 99 }];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::UnknownContinuation(_))
    ));
}

#[test]
fn perform_unknown_op_is_handler_mismatch() {
    let mut interp = Interpreter::new();
    let ops = vec![
        LoweredOp::Perform {
            op: "nope".to_string(),
        },
        LoweredOp::Return,
    ];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::HandlerMismatch)
    ));
}

#[test]
fn raise_with_matching_handler_jumps() {
    let mut interp = Interpreter::new();
    interp.push_handler(HandlerFrame {
        kind: HandlerKind::Op("log".to_string()),
        handler_pc: 2,
    });
    let ops = vec![
        LoweredOp::Raise {
            tag: "log".to_string(),
        },
        LoweredOp::Return,
        LoweredOp::Return,
    ];
    assert!(interp.run(&ops).is_ok());
}

#[test]
fn raise_without_handler_fails() {
    let mut interp = Interpreter::new();
    let ops = vec![LoweredOp::Raise {
        tag: "log".to_string(),
    }];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::UnhandledRaise(_))
    ));
}

#[test]
fn failure_handler_always_unhandled() {
    let mut interp = Interpreter::new();
    interp.push_handler(HandlerFrame {
        kind: HandlerKind::Failure,
        handler_pc: 1,
    });
    let ops = vec![LoweredOp::Raise {
        tag: "log".to_string(),
    }];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::UnhandledRaise(_))
    ));
}

#[test]
fn raise_with_non_matching_handler_fails() {
    let mut interp = Interpreter::new();
    interp.push_handler(HandlerFrame {
        kind: HandlerKind::Op("write-path".to_string()),
        handler_pc: 1,
    });
    let ops = vec![LoweredOp::Raise {
        tag: "log".to_string(),
    }];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::UnhandledRaise(_))
    ));
}

#[test]
fn raise_with_return_handler_falls_through() {
    let mut interp = Interpreter::new();
    interp.push_handler(HandlerFrame {
        kind: HandlerKind::Return,
        handler_pc: 1,
    });
    let ops = vec![LoweredOp::Raise {
        tag: "log".to_string(),
    }];
    assert!(matches!(
        interp.run(&ops),
        Err(InterpretError::UnhandledRaise(_))
    ));
}

#[test]
fn interpret_error_debug_partitions() {
    let _ = format!("{:?}", InterpretError::HandlerMismatch);
    let _ = format!("{:?}", InterpretError::UnhandledRaise("t".into()));
    let _ = format!("{:?}", InterpretError::UnknownContinuation(ContinuationId(1)));
    let _ = format!("{:?}", InterpretError::DoubleResume(ContinuationId(1)));
}
