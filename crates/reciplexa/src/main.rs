//! Minimal CLI: `reciplexa <input.rpx> <output.{pdf|svg|pptx}>`
//!
//! Compilation stages live in [`reciplexa::pipeline`] so the GUI can share them.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use reciplexa::document_pipeline::{document_snapshot_from_source, provenance_hints_for_scene};
use reciplexa::pipeline::{document_for_export, PipelineError};
use reciplexa_effect::{seed_from_env, EffectError, EffectHandler, LcgRng, Value};
use reciplexa_identity::document::DocumentIdentity;
use reciplexa_pdf::write_document_with_host_fonts;
use reciplexa_pptx::write_document as write_pptx;
use reciplexa_svg::write_document_with_hints;

fn main() -> ExitCode {
    match reciplexa::run_on_host_stack("reciplexa", cli_main) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cli_main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(first) = args.next() else {
        print_usage();
        return ExitCode::from(2);
    };

    match first.as_str() {
        "parse" => {
            let mut paths: Vec<String> = args.collect();
            let json = paths.first().is_some_and(|p| p == "--json");
            if json {
                paths.remove(0);
            }
            match paths.first() {
                Some(path) => run_cli(reciplexa::cli::cmd_parse(path, json)),
                None => {
                    eprintln!("usage: reciplexa parse [--json] <input.rpx>");
                    ExitCode::from(2)
                }
            }
        }
        "format" => match args.next() {
            Some(path) => run_cli(reciplexa::cli::cmd_format(&path)),
            None => {
                eprintln!("usage: reciplexa format <input.rpx>");
                ExitCode::from(2)
            }
        },
        "inspect-syntax" => match args.next() {
            Some(path) => run_cli(reciplexa::cli::cmd_inspect_syntax(&path)),
            None => {
                eprintln!("usage: reciplexa inspect-syntax <input.rpx>");
                ExitCode::from(2)
            }
        },
        "inspect-document" => match args.next() {
            Some(path) => run_cli(reciplexa::cli::cmd_inspect_document(&path)),
            None => {
                eprintln!("usage: reciplexa inspect-document <input.rpx>");
                ExitCode::from(2)
            }
        },
        "eval" => match args.next() {
            Some(path) => run_cli(reciplexa::cli::cmd_eval(&path)),
            None => {
                eprintln!("usage: reciplexa eval <input.rpx>");
                ExitCode::from(2)
            }
        },
        input => {
            let Some(output) = args.next() else {
                eprintln!("usage: reciplexa <input.rpx> <output.pdf|svg|pptx>");
                return ExitCode::from(2);
            };
            match render(input, &output) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn run_cli(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  reciplexa <input.rpx> <output.pdf|svg|pptx>");
    eprintln!("  reciplexa parse <input.rpx>");
    eprintln!("  reciplexa format <input.rpx>");
    eprintln!("  reciplexa inspect-syntax <input.rpx>");
    eprintln!("  reciplexa inspect-document <input.rpx>");
    eprintln!("  reciplexa eval <input.rpx>");
    eprintln!();
    eprintln!("env: RECIPLEXA_PACKAGE_GRAPHICS=1 forces package graphics ingest;");
    eprintln!("     interim top-level (page …) is refused (use package import);");
    eprintln!("     RECIPLEXA_PACKAGE_ROOT=<packages-dir> overrides search path.");
}

struct CliHandler {
    rng: LcgRng,
}

impl Default for CliHandler {
    fn default() -> Self {
        // RECIPLEXA_SEED overrides the default seed of 1.
        Self {
            rng: LcgRng::new(seed_from_env()),
        }
    }
}

impl EffectHandler for CliHandler {
    fn on_log(&mut self, message: &str) -> Result<Value, EffectError> {
        // Residual `log` effect only — never dump expression results as a REPL would.
        println!("{message}");
        Ok(Value::Unit)
    }

    fn on_random(&mut self) -> Result<Value, EffectError> {
        Ok(Value::Number(self.rng.next_unit()))
    }

    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError> {
        eprintln!("{path}");
        Ok(Value::Unit)
    }
}

fn render(input: &str, output: &str) -> Result<(), String> {
    let src = fs::read_to_string(input).map_err(|e| format!("read {input}: {e}"))?;
    let (doc, _) = document_for_export(&mut CliHandler::default(), &src).map_err(fmt_pipeline)?;
    let path = PathBuf::from(output);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
        }
    }
    let mut file = fs::File::create(&path).map_err(|e| format!("create {output}: {e}"))?;
    let base = PathBuf::from(input)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf());
    match export_kind(&path) {
        ExportKind::Pdf => write_document_with_host_fonts(&doc, base.as_deref(), &mut file)
            .map_err(|e| format!("pdf: {e:?}"))?,
        ExportKind::Svg => {
            let hints = document_snapshot_from_source(&src, DocumentIdentity::new(1))
                .map(|snap| provenance_hints_for_scene(&doc, &snap))
                .unwrap_or_default();
            write_document_with_hints(&doc, &hints, &mut file).map_err(|e| format!("svg: {e}"))?
        }
        ExportKind::Pptx => write_pptx(&doc, &mut file).map_err(|e| format!("pptx: {e}"))?,
    }
    Ok(())
}

enum ExportKind {
    Pdf,
    Svg,
    Pptx,
}

fn export_kind(path: &Path) -> ExportKind {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("pdf")
        .to_ascii_lowercase()
        .as_str()
    {
        "svg" => ExportKind::Svg,
        "pptx" => ExportKind::Pptx,
        _ => ExportKind::Pdf,
    }
}

fn fmt_pipeline(e: PipelineError) -> String {
    e.display()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn renders_example_fixture_to_temp_pdf() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo = manifest_dir.join("../..");
        let input = repo.join("examples/black_circle.rpx");
        let output = repo.join("target/test-black-circle.pdf");
        render(input.to_str().unwrap(), output.to_str().unwrap()).expect("render");
        let bytes = fs::read(&output).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn renders_example_fixture_to_svg_and_pptx() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo = manifest_dir.join("../..");
        let input = repo.join("examples/black_circle.rpx");
        let svg_out = repo.join("target/test-black-circle.svg");
        let pptx_out = repo.join("target/test-black-circle.pptx");
        render(input.to_str().unwrap(), svg_out.to_str().unwrap()).expect("svg");
        render(input.to_str().unwrap(), pptx_out.to_str().unwrap()).expect("pptx");
        let svg = fs::read_to_string(&svg_out).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<circle"));
        assert!(
            svg.contains("id=\"rpx-"),
            "planned artifact element ids required"
        );
        let pptx = fs::read(&pptx_out).unwrap();
        assert_eq!(&pptx[0..2], b"PK");
    }

    #[test]
    fn renders_all_repo_examples() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo = manifest_dir.join("../..");
        let has_cjk = reciplexa_pdf::cjk_font_path().is_some();
        for name in [
            "black_circle.rpx",
            "text_line.rpx",
            "letter_opacity.rpx",
            "two_pages.rpx",
            "japanese_page.rpx",
            "macros.rpx",
            "effects.rpx",
            "markup_doc.rpx",
            "markup_ja.rpx",
            "decls_stub.rpx",
        ] {
            if example_needs_cjk_font(name) && !has_cjk {
                eprintln!(
                    "skip {name}: no CJK font (set RECIPLEXA_CJK_FONT or install NotoSansJP)"
                );
                continue;
            }
            let input = repo.join("examples").join(name);
            let src = fs::read_to_string(&input).unwrap_or_default();
            if src.contains("language-only") {
                eprintln!("skip {name}: language-only / package-import example");
                continue;
            }
            let output = repo.join("target").join(format!("test-{name}.pdf"));
            render(input.to_str().unwrap(), output.to_str().unwrap())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    fn example_needs_cjk_font(name: &str) -> bool {
        matches!(name, "japanese_page.rpx" | "markup_ja.rpx")
    }
}
