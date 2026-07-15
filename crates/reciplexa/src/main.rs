//! Minimal CLI: `reciplexa <input.rpx> <output.pdf>`
//!
//! Compilation stages live in [`reciplexa::pipeline`] so the GUI can share them.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use reciplexa::pipeline::{document_for_export, PipelineError};
use reciplexa_effect::{seed_from_env, EffectError, EffectHandler, LcgRng, Value};
use reciplexa_pdf::write_document_with_base;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(input) = args.next() else {
        eprintln!("usage: reciplexa <input.rpx> <output.pdf>");
        return ExitCode::from(2);
    };
    let Some(output) = args.next() else {
        eprintln!("usage: reciplexa <input.rpx> <output.pdf>");
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
    write_document_with_base(&doc, base.as_deref(), file).map_err(|e| format!("pdf: {e:?}"))?;
    Ok(())
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
    fn renders_text_line_and_macro_examples() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo = manifest_dir.join("../..");
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
            "long_doc.rpx",
        ] {
            let input = repo.join("examples").join(name);
            let output = repo.join("target").join(format!("test-{name}.pdf"));
            render(input.to_str().unwrap(), output.to_str().unwrap())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
}
