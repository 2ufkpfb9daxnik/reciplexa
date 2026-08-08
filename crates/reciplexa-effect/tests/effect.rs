use reciplexa_effect::*;

// --- validity ---

#[test]
fn parses_known_ops() {
    assert_eq!(EffectOp::parse("log"), Some(EffectOp::Log));
    assert_eq!(EffectOp::parse("write-path"), Some(EffectOp::WritePath));
}

#[test]
fn test_handler_collects_log_and_write() {
    let mut h = TestHandler::default();
    run_perform(
        &mut h,
        &Perform {
            op: EffectOp::Log,
            payload: "hi".into(),
        },
    )
    .unwrap();
    run_perform(
        &mut h,
        &Perform {
            op: EffectOp::WritePath,
            payload: "out.pdf".into(),
        },
    )
    .unwrap();
    assert_eq!(h.logs, vec!["hi"]);
    assert_eq!(h.writes, vec!["out.pdf"]);
}

#[test]
fn random_consumes_sequence() {
    let mut h = TestHandler::with_random_seq(vec![0.25, 0.5]);
    assert_eq!(
        run_perform(
            &mut h,
            &Perform {
                op: EffectOp::Random,
                payload: String::new(),
            }
        )
        .unwrap(),
        Value::Number(0.25)
    );
    assert_eq!(
        run_perform(
            &mut h,
            &Perform {
                op: EffectOp::Random,
                payload: String::new(),
            }
        )
        .unwrap(),
        Value::Number(0.5)
    );
}

#[test]
fn lcg_same_seed_same_sequence() {
    let mut a = LcgRng::new(42);
    let mut b = LcgRng::new(42);
    let seq_a: Vec<f64> = (0..8).map(|_| a.next_unit()).collect();
    let seq_b: Vec<f64> = (0..8).map(|_| b.next_unit()).collect();
    assert_eq!(seq_a, seq_b);
    for v in &seq_a {
        assert!(*v >= 0.0 && *v < 1.0, "out of range: {v}");
    }
}

#[test]
fn lcg_different_seeds_diverge() {
    let mut a = LcgRng::new(1);
    let mut b = LcgRng::new(2);
    let first_a = a.next_unit();
    let first_b = b.next_unit();
    assert_ne!(first_a, first_b);
}

#[test]
fn seed_from_env_var_defaults_and_parses() {
    assert_eq!(seed_from_env_var(None), 1);
    assert_eq!(seed_from_env_var(Some("")), 1);
    assert_eq!(seed_from_env_var(Some("  ")), 1);
    assert_eq!(seed_from_env_var(Some("42")), 42);
    assert_eq!(seed_from_env_var(Some(" 99 ")), 99);
}

#[test]
fn seed_from_env_var_rejects_garbage() {
    assert_eq!(seed_from_env_var(Some("nope")), 1);
    assert_eq!(seed_from_env_var(Some("-3")), 1);
}

// --- defect ---

#[test]
fn unknown_op_name_is_none() {
    assert_eq!(EffectOp::parse("draw"), None);
    assert_eq!(EffectOp::parse(""), None);
}

#[test]
fn collects_performs_from_src_blocks() {
    let src = r#"
(src
  (perform log "hello")
  (perform write-path "out.pdf"))
(page a4 (circle 1 2 3))
"#;
    let ps = collect_performs(src).unwrap();
    assert_eq!(ps.len(), 2);
    assert_eq!(ps[0].op, EffectOp::Log);
    assert_eq!(ps[0].payload, "hello");
    assert_eq!(ps[1].op, EffectOp::WritePath);
}

#[test]
fn run_src_returns_values_in_order() {
    let src = r#"(src (perform log "a") (perform random))"#;
    let mut h = TestHandler::with_random_seq(vec![0.42]);
    let vals = run_source_effects(&mut h, src).unwrap();
    assert_eq!(h.logs, vec!["a"]);
    assert_eq!(vals, vec![Value::Unit, Value::Number(0.42)]);
}

#[test]
fn handle_log_mutes_nested_log() {
    let src = r#"
(src
  (perform log "outer")
  (handle log
(perform log "silent")
(perform random))
  (perform log "after"))
"#;
    let mut h = TestHandler::with_random_seq(vec![0.1]);
    let vals = run_source_effects(&mut h, src).unwrap();
    assert_eq!(h.logs, vec!["outer", "after"]);
    assert!(!h.logs.iter().any(|l| l == "silent"));
    assert_eq!(vals.len(), 3);
    assert_eq!(vals[1], Value::Number(0.1));
}

#[test]
fn handle_write_path_mutes_nested_writes() {
    let src = r#"
(src
  (perform write-path "outer.pdf")
  (handle write-path
(perform write-path "silent.pdf")
(perform log "still logs"))
  (perform write-path "after.pdf"))
"#;
    let mut h = TestHandler::default();
    run_source_effects(&mut h, src).unwrap();
    assert_eq!(h.writes, vec!["outer.pdf", "after.pdf"]);
    assert!(!h.writes.iter().any(|w| w == "silent.pdf"));
    assert_eq!(h.logs, vec!["still logs"]);
}

#[test]
fn nested_handles_mute_independently() {
    let src = r#"
(src
  (handle log
(handle write-path
  (perform log "silent-log")
  (perform write-path "silent.pdf")
  (perform random)))
  (perform log "after")
  (perform write-path "after.pdf"))
"#;
    let mut h = TestHandler::with_random_seq(vec![0.5]);
    let vals = run_source_effects(&mut h, src).unwrap();
    assert_eq!(h.logs, vec!["after"]);
    assert_eq!(h.writes, vec!["after.pdf"]);
    assert_eq!(vals[0], Value::Number(0.5));
}

#[test]
fn handle_random_is_rejected() {
    let err = run_source_effects(
        &mut TestHandler::default(),
        "(src (handle random (perform log \"x\")))",
    )
    .unwrap_err();
    assert!(
        err.message.contains("handle") && err.message.contains("random"),
        "{}",
        err.message
    );
}

#[test]
fn unsupported_src_form_errors() {
    let err = run_source_effects(&mut TestHandler::default(), "(src (define x 1))").unwrap_err();
    assert!(err.message.contains("unsupported"));
}

#[test]
fn effect_op_display_roundtrip() {
    assert_eq!(EffectOp::Log.to_string(), "log");
    assert_eq!(EffectOp::Random.to_string(), "random");
    assert_eq!(EffectOp::WritePath.to_string(), "write-path");
}

#[test]
fn collect_performs_parse_error() {
    let err = collect_performs("(src (perform log").unwrap_err();
    assert!(err.message.contains("parse error"));
}

#[test]
fn perform_unknown_op_errors() {
    let err =
        run_source_effects(&mut TestHandler::default(), r#"(src (perform draw "x"))"#).unwrap_err();
    assert!(err.message.contains("unknown effect op"));
}

#[test]
fn perform_missing_payload_errors() {
    let err = run_source_effects(&mut TestHandler::default(), "(src (perform log))").unwrap_err();
    assert!(err.message.contains("string payload"));
}

#[test]
fn perform_malformed_string_errors() {
    let err =
        run_source_effects(&mut TestHandler::default(), r#"(src (perform log bad))"#).unwrap_err();
    assert!(err.message.contains("payload") || err.message.contains("string"));
}

#[test]
fn run_src_forms_rejects_non_src() {
    use reciplexa_syntax::parse_source;
    let root = parse_source("(page a4)").into_result().unwrap();
    let page = root.children().next().unwrap();
    let err = run_src_forms(&mut TestHandler::default(), &page).unwrap_err();
    assert!(err.message.contains("src"));
}

#[test]
fn handle_unknown_op_errors() {
    let err = run_source_effects(
        &mut TestHandler::default(),
        "(src (handle bogus (perform log \"x\")))",
    )
    .unwrap_err();
    assert!(err.message.contains("unknown effect op"));
}

#[test]
fn handle_missing_op_name_errors() {
    let err = run_source_effects(&mut TestHandler::default(), "(src (handle))").unwrap_err();
    assert!(err.message.contains("effect op name"));
}

#[test]
fn unescape_string_all_escapes() {
    let src = r#"(src (perform log "a\nb\tc\\d\"e"))"#;
    let ps = collect_performs(src).unwrap();
    assert_eq!(ps[0].payload, "a\nb\tc\\d\"e");
}

#[test]
fn random_defaults_to_zero_when_sequence_exhausted() {
    let mut h = TestHandler::default();
    assert_eq!(
        run_perform(
            &mut h,
            &Perform {
                op: EffectOp::Random,
                payload: String::new(),
            }
        )
        .unwrap(),
        Value::Number(0.0)
    );
}

#[test]
fn lcg_seed_zero_uses_fallback_state() {
    let mut rng = LcgRng::new(0);
    assert!(rng.next_unit() >= 0.0);
}

#[test]
fn collect_performs_nested_in_handle() {
    let src = r#"(src (handle log (perform log "nested")))"#;
    let ps = collect_performs(src).unwrap();
    assert_eq!(ps.len(), 1);
    assert_eq!(ps[0].payload, "nested");
}

#[test]
fn effect_error_and_value_equality() {
    let e = EffectError::new("boom");
    assert_eq!(e.message, "boom");
    assert_eq!(Value::Unit, Value::Unit);
    assert_ne!(Value::Number(1.0), Value::Number(2.0));
}

#[test]
fn effect_op_display_and_parse_partitions() {
    assert_eq!(EffectOp::Log.to_string(), "log");
    assert_eq!(EffectOp::Random.to_string(), "random");
    assert_eq!(EffectOp::WritePath.to_string(), "write-path");
    assert_eq!(EffectOp::parse("random"), Some(EffectOp::Random));
    assert_eq!(EffectOp::parse("nope"), None);
    assert_eq!(EffectOp::parse(""), None);
}

#[test]
fn value_debug_and_string_variant() {
    let s = Value::String("x".into());
    assert!(format!("{s:?}").contains("x"));
    assert_eq!(s, Value::String("x".into()));
}

#[test]
fn run_source_effects_empty_src_and_unknown_op() {
    let mut h = TestHandler::default();
    assert!(run_source_effects(&mut h, "(src)").is_ok());
    let err = run_source_effects(&mut h, "(src (perform nope))").unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn collect_performs_empty_and_write_path() {
    assert!(collect_performs("(src)").unwrap().is_empty());
    let ps = collect_performs(r#"(src (perform write-path "a.pdf"))"#).unwrap();
    assert_eq!(ps.len(), 1);
    assert_eq!(ps[0].op, EffectOp::WritePath);
}

#[test]
fn seed_from_env_reads_process_env() {
    let _ = seed_from_env();
}

#[test]
fn run_source_skips_non_src_and_mutes_forward_write() {
    let mut h = TestHandler::default();
    // Non-src forms are skipped by run_source_effects.
    run_source_effects(&mut h, r#"(page a4)(src (perform log "x"))"#).unwrap();
    assert_eq!(h.logs, vec!["x"]);
    // Mute log but forward write-path through MuteOp::on_write_path.
    h.logs.clear();
    run_source_effects(
        &mut h,
        r#"(src (handle log (perform write-path "via.pdf") (perform log "silent")))"#,
    )
    .unwrap();
    assert_eq!(h.writes, vec!["via.pdf"]);
    assert!(h.logs.is_empty());
}

#[test]
fn handle_body_before_op_errors() {
    // List body appears before the op ident → dedicated diagnostic.
    let err = run_source_effects(
        &mut TestHandler::default(),
        r#"(src (handle (perform log "x") log))"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("op before the body"),
        "{}",
        err.message
    );
}

#[test]
fn unescape_cr_unknown_and_trailing_backslash() {
    let ps = collect_performs(r#"(src (perform log "a\rb\q c\ "))"#).unwrap();
    // `\r` → CR, `\q` → q, trailing `\` kept when escape is incomplete inside quotes —
    // the source uses `\ ` (backslash + space) as unknown escape → space.
    assert!(ps[0].payload.contains('\r'));
    assert!(ps[0].payload.contains('q'));
    let ps = collect_performs(r#"(src (perform log "trail\\"))"#).unwrap();
    assert!(ps[0].payload.ends_with('\\'));
    // Direct unit coverage for trailing lone backslash in unescape_string.
    assert_eq!(unescape_string_for_test("x\\"), "x\\");
    assert_eq!(unescape_string_for_test("a\\r"), "a\r");
    assert_eq!(unescape_string_for_test("a\\z"), "az");
}

#[test]
fn collect_and_run_skip_non_list_and_unknown_heads() {
    let ps = collect_performs(r#"(src (perform log "a") (noop) (handle log (perform log "b")))"#)
        .unwrap();
    assert_eq!(ps.len(), 2);
    // Headless / numeric-headed lists are ignored by list_head_ident.
    assert!(collect_performs("(src (1 2) ())").unwrap().is_empty());
}

#[test]
fn perform_op_must_be_ident_at_runtime() {
    let err =
        run_source_effects(&mut TestHandler::default(), r#"(src (perform 1 "x"))"#).unwrap_err();
    assert!(
        err.message.contains("op identifier") || err.message.contains("perform"),
        "{}",
        err.message
    );
}
