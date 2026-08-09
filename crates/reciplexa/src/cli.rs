//! Subcommands for syntax inspection (`roadmap.md` Phase 1 §3.4).

use std::fs;
use std::io::{self, Write};

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
    let snap = crate::document_pipeline::document_snapshot_from_source(
        &src,
        reciplexa_identity::document::DocumentIdentity::new(1),
    )?;
    println!(
        "document {} rev {} nodes {}",
        snap.identity.get(),
        snap.revision.get(),
        snap.nodes.iter().count()
    );
    for node in snap.nodes.iter() {
        println!("  {:?} id={}", node.kind, node.id.get());
    }
    Ok(())
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

fn emit_parse_diagnostics(
    src: &str,
    errors: &[reciplexa_syntax::ParseError],
    json: bool,
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
        let mut out = io::stdout();
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
        let mut out = io::stderr();
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
}
