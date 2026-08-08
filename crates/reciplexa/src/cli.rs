//! Subcommands for syntax inspection (`roadmap.md` Phase 1 §3.4).

use std::fs;
use std::io::{self, Write};

use reciplexa_diagnostic::{push_syntax_parse_errors, render_diagnostic_line, DiagnosticCollector};
use reciplexa_identity::package::{ModuleId, PackageInstanceId};
use reciplexa_source::line_index::LineIndex;
use reciplexa_source::resource::{SourceResource, SourceResourceId};
use reciplexa_syntax::{parse_source, unparse};

pub fn cmd_parse(path: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    let resource = SourceResource::from_bytes(SourceResourceId::new(1), &bytes)
        .map_err(|e| format!("decode: {e}"))?;
    let parse = parse_source(resource.text());
    if parse.errors.is_empty() {
        println!(
            "ok: parsed {} top-level forms",
            parse.root.children().count()
        );
        Ok(())
    } else {
        emit_parse_diagnostics(resource.text(), &parse.errors)?;
        Err(format!("{} parse error(s)", parse.errors.len()))
    }
}

pub fn cmd_format(path: &str) -> Result<(), String> {
    let src = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let parse = parse_source(&src);
    if !parse.errors.is_empty() {
        emit_parse_diagnostics(&src, &parse.errors)?;
        return Err(format!("{} parse error(s)", parse.errors.len()));
    }
    let formatted = unparse(&parse.root);
    print!("{formatted}");
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
        emit_parse_diagnostics(&src, &parse.errors)?;
        return Err(format!("{} parse error(s)", parse.errors.len()));
    }
    Ok(())
}

fn emit_parse_diagnostics(
    src: &str,
    errors: &[reciplexa_syntax::ParseError],
) -> Result<(), String> {
    let mut collector = DiagnosticCollector::new();
    push_syntax_parse_errors(
        &mut collector,
        PackageInstanceId::new(1),
        ModuleId::new(1),
        SourceResourceId::new(1),
        errors,
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_example() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/black_circle.rpx");
        cmd_parse(path.to_str().unwrap()).expect("parse");
    }

    #[test]
    fn format_roundtrip_example() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/black_circle.rpx");
        cmd_format(path.to_str().unwrap()).expect("format");
    }

    #[test]
    fn inspect_syntax_lists_forms() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = manifest.join("examples/black_circle.rpx");
        cmd_inspect_syntax(path.to_str().unwrap()).expect("inspect");
    }
}
