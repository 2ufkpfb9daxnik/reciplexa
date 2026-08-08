use reciplexa_ir::{verify_one_shot, Continuation, ContinuationId, LoweredOp, VerifyError};

#[test]
fn detects_double_resume_in_program() {
    let ops = vec![LoweredOp::Resume { cont: 1 }, LoweredOp::Resume { cont: 1 }];
    let mut conts = vec![Continuation::new(ContinuationId(1))];
    assert!(matches!(
        verify_one_shot(&ops, &mut conts),
        Err(VerifyError::DoubleResume(_))
    ));
}

#[test]
fn accepts_single_resume() {
    let ops = vec![
        LoweredOp::Perform {
            op: "log".into(),
        },
        LoweredOp::Resume { cont: 1 },
        LoweredOp::Return,
    ];
    let mut conts = vec![Continuation::new(ContinuationId(1))];
    assert_eq!(verify_one_shot(&ops, &mut conts), Ok(()));
}

#[test]
fn unknown_continuation_is_invalid() {
    let ops = vec![LoweredOp::Resume { cont: 99 }];
    let mut conts = vec![Continuation::new(ContinuationId(1))];
    assert!(matches!(
        verify_one_shot(&ops, &mut conts),
        Err(VerifyError::InvalidProgram(_))
    ));
}

#[test]
fn empty_and_non_resume_ops_ok() {
    let ops = vec![LoweredOp::Return, LoweredOp::Raise { tag: "t".into() }];
    let mut conts: Vec<Continuation> = vec![];
    assert_eq!(verify_one_shot(&ops, &mut conts), Ok(()));
    assert_eq!(verify_one_shot(&[], &mut conts), Ok(()));
}

#[test]
fn verify_error_debug() {
    let _ = format!("{:?}", VerifyError::InvalidProgram("x"));
    let _ = format!("{:?}", VerifyError::DoubleResume(ContinuationId(1)));
}
