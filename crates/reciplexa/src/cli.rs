//! Subcommands for syntax inspection (`roadmap.md` Phase 1 §3.4).

use std::fs;
use std::io::{self};

use reciplexa_diagnostic::{push_syntax_parse_errors, render_diagnostic_line, DiagnosticCollector};
use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::line_index::LineIndex;
use reciplexa_source::resource::{SourceResource, SourceResourceId};
use reciplexa_syntax::{parse_source, unparse};

pub fn cmd_parse(path: &str, json: bool) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    let resource = SourceResource::from_bytes(SourceResourceId::new(1), &bytes)
        .map_err(|e| format!("decode: {e}"))?;
    let parse = parse_source(resource.text());
    if parse.errors.is_empty() {
        if json {
            println!(
                "{{\"status\":\"ok\",\"forms\":{}}}",
                parse.root.children().count()
            );
        } else {
            println!(
                "ok: parsed {} top-level forms",
                parse.root.children().count()
            );
        }
        Ok(())
    } else {
        emit_parse_diagnostics(resource.text(), &parse.errors, json)?;
        Err(format!("{} parse error(s)", parse.errors.len()))
    }
}

pub fn cmd_format(path: &str) -> Result<(), String> {
    let src = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let parse = parse_source(&src);
    if !parse.errors.is_empty() {
        emit_parse_diagnostics(&src, &parse.errors, false)?;
        return Err(format!("{} parse error(s)", parse.errors.len()));
    }
    let formatted = unparse(&parse.root);
    print!("{formatted}");
    Ok(())
}

pub fn cmd_inspect_document(path: &str) -> Result<(), String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    match crate::document_pipeline::document_snapshot_from_source(
        &src,
        reciplexa_identity::document::DocumentIdentity::new(1),
    ) {
        Ok(snap) => {
            println!(
                "document {} rev {} nodes {}",
                snap.identity.get(),
                snap.revision.get(),
                snap.nodes.iter().count()
            );
            for node in snap.nodes.iter() {
                println!("  {:?} id={}", node.kind, node.id.get());
            }
            // HC2: optional host note when the snapshot carries Text shapes (e.g. JA wrap).
            let shapes = crate::document_pipeline::preview_shapes(&snap);
            if shapes
                .iter()
                .any(|s| matches!(s, reciplexa_scene::Shape::Text(_)))
            {
                let doc = reciplexa_scene::Document::single_page(reciplexa_scene::Page {
                    paper: reciplexa_scene::PaperSize::a4(),
                    shapes,
                });
                let metrics = reciplexa_package::preview_doc_text_metrics_from_document(&doc);
                println!("{}", metrics.diagnostic_note());
            }
            Ok(())
        }
        Err(doc_err) => {
            // Host layout: math-ish package mains get a fontless box estimate.
            if !source_looks_math_ish(&src) {
                return Err(doc_err);
            }
            match try_inspect_math_main(path) {
                Ok(note) => {
                    println!("{note}");
                    Ok(())
                }
                Err(math_err) => Err(format!("{doc_err}; math estimate: {math_err}")),
            }
        }
    }
}

fn source_looks_math_ish(src: &str) -> bool {
    src.contains("(import math/")
}

fn try_inspect_math_main(path: &str) -> Result<String, String> {
    use std::path::Path;

    use reciplexa_package::{estimate_package_math_main, LocalPackageIndex};

    let entry = Path::new(path);
    let search_roots = crate::pipeline::package_search_roots_near(Some(entry));
    let roots: Vec<&Path> = search_roots.iter().map(|p| p.as_path()).collect();
    let idx = LocalPackageIndex::discover(&roots).map_err(|e| e.to_string())?;
    let box_ = estimate_package_math_main(entry, &idx).map_err(|e| e.to_string())?;
    Ok(format!(
        "math box estimate: width={:.3}em height={:.3}em depth={:.3}em total={:.3}em",
        box_.width,
        box_.height,
        box_.depth,
        box_.total_height()
    ))
}

pub fn cmd_inspect_syntax(path: &str) -> Result<(), String> {
    let src = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let parse = parse_source(&src);
    println!("{:?}", parse.root.kind());
    for child in parse.root.children() {
        println!("  form {:?}", child.kind());
    }
    if !parse.errors.is_empty() {
        emit_parse_diagnostics(&src, &parse.errors, false)?;
        return Err(format!("{} parse error(s)", parse.errors.len()));
    }
    Ok(())
}

/// Host for `reciplexa eval`: residual effects only (not a REPL).
///
/// Ambient `log` writes the message to stdout. Pure programs produce no stdout.
#[derive(Default)]
struct CliEvalHost {
    fs: reciplexa_eval::MemoryFsHost,
}

impl reciplexa_eval::EffectHost for CliEvalHost {
    fn perform(
        &mut self,
        op: &str,
        arg: reciplexa_eval::RuntimeValue,
    ) -> reciplexa_eval::EvalResult {
        match op {
            "log" => {
                match &arg {
                    reciplexa_eval::RuntimeValue::String(s) => println!("{s}"),
                    other => println!("{other}"),
                }
                Ok(reciplexa_eval::RuntimeValue::Unit)
            }
            other => self.fs.perform(other, arg),
        }
    }
}

/// Elaborate + evaluate a language-kernel `.rpx` source (no page/graphics).
///
/// Does **not** print the final value (this is not a REPL). Terminal output comes
/// only from residual effects such as ambient `(log …)`.
pub fn cmd_eval(path: &str) -> Result<(), String> {
    let src = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let mut host = CliEvalHost::default();
    let _value = reciplexa_eval::eval_source_with_host(&src, &mut host).map_err(|e| e.message)?;
    Ok(())
}

fn emit_parse_diagnostics(
    src: &str,
    errors: &[reciplexa_syntax::ParseError],
    json: bool,
) -> Result<(), String> {
    if json {
        emit_parse_diagnostics_into(src, errors, true, &mut io::stdout())
    } else {
        emit_parse_diagnostics_into(src, errors, false, &mut io::stderr())
    }
}

fn emit_parse_diagnostics_into(
    src: &str,
    errors: &[reciplexa_syntax::ParseError],
    json: bool,
    out: &mut dyn io::Write,
) -> Result<(), String> {
    let mut collector = DiagnosticCollector::new();
    push_syntax_parse_errors(
        &mut collector,
        PackageInstanceId::new(1),
        ModuleId::new(1),
        SourceResourceId::new(1),
        errors,
    );
    if json {
        writeln!(out, "{{\"status\":\"error\",\"diagnostics\":[").map_err(|e| e.to_string())?;
        for (i, d) in collector.diagnostics().iter().enumerate() {
            if i > 0 {
                write!(out, ",").map_err(|e| e.to_string())?;
            }
            write!(out, "{}", diagnostic_to_json(d)).map_err(|e| e.to_string())?;
        }
        writeln!(out, "]}}").map_err(|e| e.to_string())?;
        return Ok(());
    }
    let index = LineIndex::new(src);
    for line in collector
        .diagnostics()
        .iter()
        .map(|d| render_diagnostic_line(&index, d))
    {
        writeln!(out, "{line}").map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn diagnostic_to_json(d: &reciplexa_diagnostic::Diagnostic) -> String {
    let severity = format!("{}", d.severity);
    let code = format!("{}", d.code);
    let message = d.message.fallback_summary();
    let (start, end) = match &d.primary_origin {
        Some(reciplexa_diagnostic::DiagnosticOrigin::Source(o)) => {
            (o.text_range.start().get(), o.text_range.end().get())
        }
        _ => (0, 0),
    };
    format!(
        "{{\"id\":{},\"severity\":\"{severity}\",\"code\":\"{code}\",\"message\":{msg},\"start\":{start},\"end\":{end}}}",
        d.id.get(),
        msg = json_string(&message),
    )
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_example() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/black_circle.rpx");
        cmd_parse(path.to_str().unwrap(), false).expect("parse");
    }

    #[test]
    fn format_roundtrip_example() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/black_circle.rpx");
        cmd_format(path.to_str().unwrap()).expect("format");
    }

    #[test]
    fn inspect_document_lists_nodes() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/black_circle.rpx");
        cmd_inspect_document(path.to_str().unwrap()).expect("inspect-document");
    }

    #[test]
    fn inspect_syntax_lists_forms() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/black_circle.rpx");
        cmd_inspect_syntax(path.to_str().unwrap()).expect("inspect");
    }

    #[test]
    fn parse_json_mode_ok() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("ok.rpx");
        std::fs::write(&path, "(page a4 (circle 1 2 3))").unwrap();
        cmd_parse(path.to_str().unwrap(), true).expect("json parse");
    }

    #[test]
    fn parse_invalid_emits_error() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad.rpx");
        std::fs::write(&path, "(page a4").unwrap();
        let err = cmd_parse(path.to_str().unwrap(), false).unwrap_err();
        assert!(err.contains("parse error"));
    }

    #[test]
    fn format_invalid_returns_error() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad_fmt.rpx");
        std::fs::write(&path, "(unclosed").unwrap();
        assert!(cmd_format(path.to_str().unwrap()).is_err());
    }

    #[test]
    fn json_string_escapes() {
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_string("line\n"), "\"line\\n\"");
    }

    #[test]
    fn read_missing_file_errors() {
        let err = cmd_parse("/nonexistent/rpx_file.rpx", false).unwrap_err();
        assert!(err.contains("read"));
    }

    #[test]
    fn parse_invalid_json_mode_emits_json_diagnostics() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad_json.rpx");
        std::fs::write(&path, "(page a4").unwrap();
        let err = cmd_parse(path.to_str().unwrap(), true).unwrap_err();
        assert!(err.contains("parse error"));
    }

    #[test]
    fn inspect_syntax_reports_errors_for_broken_file() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad_inspect.rpx");
        std::fs::write(&path, "(unclosed").unwrap();
        let err = cmd_inspect_syntax(path.to_str().unwrap()).unwrap_err();
        assert!(err.contains("parse error"));
    }

    #[test]
    fn format_roundtrip_writes_to_stdout() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("fmt_ok.rpx");
        std::fs::write(&path, "(page a4 (circle 1 2 3))\n").unwrap();
        cmd_format(path.to_str().unwrap()).expect("format ok");
    }

    #[test]
    fn json_string_escapes_control_chars() {
        assert_eq!(json_string("a\rb"), "\"a\\rb\"");
        assert_eq!(json_string("a\tb"), "\"a\\tb\"");
        assert_eq!(json_string("\u{0001}"), "\"\\u0001\"");
    }

    #[test]
    fn parse_invalid_utf8_errors() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad_utf8.rpx");
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        let err = cmd_parse(path.to_str().unwrap(), false).unwrap_err();
        assert!(err.contains("decode") || err.contains("read"));
    }

    #[test]
    fn inspect_document_missing_file_errors() {
        let err = cmd_inspect_document("/nonexistent/missing.rpx").unwrap_err();
        assert!(err.contains("read"));
    }

    #[test]
    fn eval_pure_fn_example() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/pure_fn.rpx");
        // Pure programs succeed with no stdout requirement (not a REPL).
        cmd_eval(path.to_str().unwrap()).expect("eval");
    }

    #[test]
    fn eval_missing_file_errors() {
        let err = cmd_eval("/nonexistent/missing.rpx").unwrap_err();
        assert!(err.contains("read"));
    }

    #[test]
    fn eval_source_with_host_log_is_only_output_channel() {
        use reciplexa_eval::{eval_source_with_host, EffectHost, EvalResult, RuntimeValue};

        #[derive(Default)]
        struct CaptureHost {
            logs: Vec<String>,
        }

        impl EffectHost for CaptureHost {
            fn perform(&mut self, op: &str, arg: RuntimeValue) -> EvalResult {
                match op {
                    "log" => {
                        let msg = match arg {
                            RuntimeValue::String(s) => s,
                            other => other.to_string(),
                        };
                        self.logs.push(msg);
                        Ok(RuntimeValue::Unit)
                    }
                    "random" => Ok(RuntimeValue::F64(0.5)),
                    other => Err(reciplexa_eval::EvalError {
                        message: format!("unknown op `{other}`"),
                    }),
                }
            }
        }

        let mut host = CaptureHost::default();
        let v =
            eval_source_with_host(r#"(val main (seq (log "hello") 42))"#, &mut host).expect("eval");
        assert_eq!(v, RuntimeValue::Int(42));
        assert_eq!(host.logs, vec!["hello".to_string()]);
        // Tip non-string log / random / unknown-op arms.
        let _ = host.perform("log", RuntimeValue::Int(7));
        let _ = host.perform("random", RuntimeValue::Unit);
        let _ = host.perform("nope", RuntimeValue::Unit);
    }

    #[test]
    fn cli_eval_host_log_string_and_non_string() {
        let mut host = CliEvalHost::default();
        use reciplexa_eval::{EffectHost, RuntimeValue};
        assert_eq!(
            host.perform("log", RuntimeValue::String("hi".into()))
                .unwrap(),
            RuntimeValue::Unit
        );
        assert_eq!(
            host.perform("log", RuntimeValue::Int(7)).unwrap(),
            RuntimeValue::Unit
        );
    }

    #[test]
    fn cli_eval_host_delegates_unknown_ops_to_fs() {
        let mut host = CliEvalHost::default();
        use reciplexa_eval::{EffectHost, RuntimeValue};
        // MemoryFsHost rejects unknown ops; cover the delegation arm.
        let err = host
            .perform("not-a-real-op", RuntimeValue::Unit)
            .unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn cmd_eval_runs_log_program() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("log_prog.rpx");
        std::fs::write(&path, r#"(val main (seq (log "from-cli") 1))"#).unwrap();
        cmd_eval(path.to_str().unwrap()).expect("eval log");
    }

    #[test]
    fn cmd_eval_rejects_bad_program() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("bad_eval.rpx");
        std::fs::write(&path, "(val main").unwrap();
        assert!(cmd_eval(path.to_str().unwrap()).is_err());
    }

    #[test]
    fn json_string_escapes_backslash() {
        assert_eq!(json_string("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn format_and_inspect_syntax_missing_file_errors() {
        let err = cmd_format("/nonexistent/rpx_format.rpx").unwrap_err();
        assert!(err.contains("read"));
        let err = cmd_inspect_syntax("/nonexistent/rpx_inspect.rpx").unwrap_err();
        assert!(err.contains("read"));
    }

    #[test]
    fn parse_json_mode_multiple_diagnostics() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("multi_err.rpx");
        // Two independently broken top-level forms → ≥2 diagnostics (i > 0 comma path).
        std::fs::write(&path, "(page a4\n(circle\n").unwrap();
        let err = cmd_parse(path.to_str().unwrap(), true).unwrap_err();
        assert!(err.contains("parse error"));
    }

    #[test]
    fn inspect_document_rejects_non_document_source() {
        let dir = std::env::temp_dir().join("rpx_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("pure.rpx");
        std::fs::write(&path, "(val main 1)").unwrap();
        let err = cmd_inspect_document(path.to_str().unwrap()).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn inspect_document_math_ish_prints_box_estimate() {
        std::thread::Builder::new()
            .name("inspect-math".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let entry = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../examples/pkg_math.rpx");
                // Capture stdout is awkward in unit tests; assert Ok + note via helper.
                let src = std::fs::read_to_string(&entry).expect("pkg_math");
                assert!(super::source_looks_math_ish(&src));
                let note = super::try_inspect_math_main(entry.to_str().unwrap())
                    .expect("math box estimate");
                assert!(
                    note.contains("math box estimate:") && note.contains("width="),
                    "unexpected note: {note}"
                );
                cmd_inspect_document(entry.to_str().unwrap()).expect("inspect math main");
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn diagnostic_to_json_unknown_origin_defaults_range() {
        use reciplexa_diagnostic::{
            Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticId, DiagnosticLifecycleStage,
            DiagnosticMessage, DiagnosticOrigin, DiagnosticSeverity,
        };
        let d = Diagnostic::new(
            DiagnosticId::new(1),
            DiagnosticCode::new("compiler", "syntax", "SYN-0001"),
            DiagnosticSeverity::Error,
            DiagnosticCategory::Syntax,
            DiagnosticLifecycleStage::Parse,
            DiagnosticMessage::new("x"),
        )
        .with_primary_origin(DiagnosticOrigin::Unknown);
        let json = diagnostic_to_json(&d);
        assert!(json.contains("\"start\":0"));
        assert!(json.contains("\"end\":0"));
    }

    #[test]
    fn emit_parse_diagnostics_write_err_arms() {
        use reciplexa_syntax::parse_source;
        use std::io;

        struct Boom;
        impl io::Write for Boom {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("boom"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let parse = parse_source("(page a4\n(circle\n");
        assert!(!parse.errors.is_empty());
        assert!(emit_parse_diagnostics_into("(page a4", &parse.errors, true, &mut Boom).is_err());
        assert!(emit_parse_diagnostics_into("(page a4", &parse.errors, false, &mut Boom).is_err());

        // Multi-diag JSON path: first write succeeds, later write fails (comma / body).
        struct FailAfter {
            left: usize,
        }
        impl io::Write for FailAfter {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                if self.left == 0 {
                    return Err(io::Error::other("boom-later"));
                }
                self.left -= 1;
                Ok(buf.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut w = FailAfter { left: 1 };
        let _ = emit_parse_diagnostics_into("(x", &parse.errors, true, &mut w);
        let mut w2 = FailAfter { left: 2 };
        let _ = emit_parse_diagnostics_into("(x", &parse.errors, true, &mut w2);
        let mut boom = Boom;
        let _ = std::io::Write::flush(&mut boom);
        let _ = std::io::Write::flush(&mut w2);
    }
}
