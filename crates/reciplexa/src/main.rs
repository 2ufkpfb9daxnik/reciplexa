//! Minimal CLI: `reciplexa <input.rpx> <output.{pdf|svg|pptx}>`
//!
//! Compilation stages live in [`reciplexa::pipeline`] so the GUI can share them.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use reciplexa::pipeline::{document_for_export, PipelineError};
use reciplexa_effect::{seed_from_env, EffectError, EffectHandler, LcgRng, Value};
use reciplexa_pdf::write_document_with_base;
use reciplexa_pptx::write_document as write_pptx;
use reciplexa_svg::write_document as write_svg;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(input) = args.next() else {
        eprintln!("usage: reciplexa <input.rpx> <output.pdf|svg|pptx>");
        return ExitCode::from(2);
    };
    let Some(output) = args.next() else {
        eprintln!("usage: reciplexa <input.rpx> <output.pdf|svg|pptx>");
        return ExitCode::from(2);
    };

    match render(&input, &output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
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
        eprintln!("[perform log] {message}");
        Ok(Value::Unit)
    }

    fn on_random(&mut self) -> Result<Value, EffectError> {
        Ok(Value::Number(self.rng.next_unit()))
    }

    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError> {
        eprintln!("[perform write-path] {path}");
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
    let file = fs::File::create(&path).map_err(|e| format!("create {output}: {e}"))?;
    let base = PathBuf::from(input)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf());
    match export_kind(&path) {
        ExportKind::Pdf => write_document_with_base(&doc, base.as_deref(), file)
            .map_err(|e| format!("pdf: {e:?}"))?,
        ExportKind::Svg => write_svg(&doc, file).map_err(|e| format!("svg: {e}"))?,
        ExportKind::Pptx => write_pptx(&doc, file).map_err(|e| format!("pptx: {e}"))?,
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
        let pptx = fs::read(&pptx_out).unwrap();
        assert_eq!(&pptx[0..2], b"PK");
    }

    #[test]
    fn renders_text_line_and_macro_examples() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo = manifest_dir.join("../..");
        let has_cjk = reciplexa_pdf::cjk_font_path().is_some();
        for name in [
            "text_and_line.rpx",
            "color_byte.rpx",
            "ellipse.rpx",
            "outlines.rpx",
            "polyline.rpx",
            "two_pages.rpx",
            "image_placeholder.rpx",
            "with_src_log.rpx",
            "polygon.rpx",
            "japanese_report_stub.rpx",
            "japanese_doc.rpx",
            "letter_opacity.rpx",
            "hello_doc.rpx",
            "hline_macro.rpx",
            "gray_macro.rpx",
            "with_handle_log.rpx",
            "with_handle_write_path.rpx",
            "with_code.rpx",
            "multiline_doc.rpx",
            "doc_title_p.rpx",
            "doc_link_caption.rpx",
            "doc_with_figure.rpx",
            "long_doc.rpx",
        ] {
            if example_needs_cjk_font(name) && !has_cjk {
                eprintln!(
                    "skip {name}: no CJK font (set RECIPLEXA_CJK_FONT or install NotoSansJP)"
                );
                continue;
            }
            let input = repo.join("examples").join(name);
            let output = repo.join("target").join(format!("test-{name}.pdf"));
            render(input.to_str().unwrap(), output.to_str().unwrap())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    fn example_needs_cjk_font(name: &str) -> bool {
        matches!(
            name,
            "japanese_report_stub.rpx"
                | "japanese_doc.rpx"
                | "multiline_doc.rpx"
                | "long_doc.rpx"
                | "doc_title_p.rpx"
                | "hello_doc.rpx"
        )
    }
}
