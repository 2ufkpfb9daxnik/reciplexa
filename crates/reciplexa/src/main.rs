//! Minimal CLI: `reciplexa <input.rpx> <output.pdf>`

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use reciplexa_effect::{run_source_effects, EffectError, EffectHandler, Value};
use reciplexa_lower::lower_source;
use reciplexa_macro::expand_source;
use reciplexa_pdf::write_document_with_base;
use reciplexa_types::typecheck_source;

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

struct CliHandler;

impl EffectHandler for CliHandler {
    fn on_log(&mut self, message: &str) -> Result<Value, EffectError> {
        eprintln!("[perform log] {message}");
        Ok(Value::Unit)
    }

    fn on_random(&mut self) -> Result<Value, EffectError> {
        // Deterministic stub until a real RNG effect is wired.
        Ok(Value::Number(0.0))
    }

    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError> {
        eprintln!("[perform write-path] {path}");
        Ok(Value::Unit)
    }
}

fn render(input: &str, output: &str) -> Result<(), String> {
    let src = fs::read_to_string(input).map_err(|e| format!("read {input}: {e}"))?;
    let expanded = expand_source(&src).map_err(|e| format!("macro: {}", e.message))?;
    typecheck_source(&expanded)
        .map_err(|e| format!("type: {} @{}..{}", e.message, e.start, e.end))?;
    run_source_effects(&mut CliHandler, &expanded).map_err(|e| format!("effect: {}", e.message))?;
    let doc = lower_source(&expanded).map_err(|e| e.message)?;
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
            "letter_opacity.rpx",
            "hello_doc.rpx",
            "hline_macro.rpx",
            "gray_macro.rpx",
            "with_handle_log.rpx",
            "multiline_doc.rpx",
        ] {
            let input = repo.join("examples").join(name);
            let output = repo.join("target").join(format!("test-{name}.pdf"));
            render(input.to_str().unwrap(), output.to_str().unwrap())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }
}
