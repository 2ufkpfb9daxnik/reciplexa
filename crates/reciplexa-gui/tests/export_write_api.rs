//! Regression: export writers must accept `&mut File` (GUI/CLI call sites).
//!
//! Coverage-driven API changes previously broke `cargo run -p reciplexa-gui`
//! while package unit tests still passed.

use reciplexa_pdf::write_document_with_base;
use reciplexa_pptx::write_document as write_pptx;
use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};
use reciplexa_svg::write_document as write_svg;
use std::fs::File;
use std::io::Read;

fn sample_doc() -> Document {
    Document::single_page(Page {
        paper: PaperSize::a4(),
        shapes: vec![Shape::Circle(Circle {
            x_mm: 105.0,
            y_mm: 148.5,
            radius_mm: 40.0,
            fill: Color::BLACK,
        })],
    })
}

#[test]
fn export_writers_accept_mut_file() {
    let dir =
        std::env::temp_dir().join(format!("reciplexa_gui_export_smoke_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let doc = sample_doc();

    for (name, write) in [
        (
            "out.pdf",
            Box::new(|d: &Document, f: &mut File| {
                write_document_with_base(d, None, f).map_err(|e| format!("{e:?}"))
            }) as Box<dyn Fn(&Document, &mut File) -> Result<(), String>>,
        ),
        (
            "out.svg",
            Box::new(|d: &Document, f: &mut File| write_svg(d, f).map_err(|e| e.to_string())),
        ),
        (
            "out.pptx",
            Box::new(|d: &Document, f: &mut File| write_pptx(d, f)),
        ),
    ] {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        write(&doc, &mut file).unwrap();
        drop(file);
        let mut bytes = Vec::new();
        File::open(&path).unwrap().read_to_end(&mut bytes).unwrap();
        assert!(!bytes.is_empty(), "{name} should be non-empty");
    }

    let _ = std::fs::remove_dir_all(&dir);
}
