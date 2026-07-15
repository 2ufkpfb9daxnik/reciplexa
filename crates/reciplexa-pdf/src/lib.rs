//! Minimal PDF emitter for [`reciplexa_scene::Document`].
//!
//! Hand-rolled PDF-1.4: shapes, Helvetica ASCII text, system-font glyph outlines
//! for non-ASCII (CJK), and embedded PNG/JPEG images. Named paper sizes are
//! temporary sugar; numeric `(page w h …)` is the core. JLReq typesetting stays
//! a later package.

#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use reciplexa_scene::{Affine, Color, Document, Page, Shape};

/// Errors while building a PDF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfError {
    EmptyDocument,
    InvalidPage(String),
    InvalidShape(String),
    Write(String),
}

struct EmbeddedImage {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
}

/// Decoded RGB8 raster for PDF embed and GUI preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterRgb {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

/// Load a PNG or JPEG from disk into RGB8 (paths as given; caller resolves relatives).
pub fn load_raster_file(path: &Path) -> Result<RasterRgb, String> {
    let img = load_raster_rgb(path)?;
    Ok(RasterRgb {
        width: img.width,
        height: img.height,
        rgb: img.rgb,
    })
}

struct ImageStore {
    by_key: HashMap<String, usize>,
    images: Vec<EmbeddedImage>,
    base: Option<PathBuf>,
}

impl ImageStore {
    fn new(base: Option<&Path>) -> Self {
        Self {
            by_key: HashMap::new(),
            images: Vec::new(),
            base: base.map(Path::to_path_buf),
        }
    }

    fn get_or_load(&mut self, path: &str) -> Result<usize, PdfError> {
        if let Some(&id) = self.by_key.get(path) {
            return Ok(id);
        }
        let resolved = resolve_image_path(path, self.base.as_deref());
        let img = load_raster_rgb(&resolved).map_err(|e| {
            PdfError::InvalidShape(format!("image `{path}` ({}): {e}", resolved.display()))
        })?;
        let id = self.images.len();
        self.by_key.insert(path.to_string(), id);
        self.images.push(img);
        Ok(id)
    }
}

struct PageEmit {
    ops: String,
    /// Opacity percents `0..=100` referenced as `/GSk gs`.
    opacities: BTreeSet<u8>,
    /// Embedded image ids referenced as `/Imk Do`.
    images: BTreeSet<usize>,
}

/// Render a scene document to PDF bytes (image paths relative to the process CWD).
pub fn document_to_pdf(doc: &Document) -> Result<Vec<u8>, PdfError> {
    document_to_pdf_with_base(doc, None)
}

/// Render a scene document; resolve `(image "…")` paths relative to `base` when set
/// (typically the directory containing the `.rpx` source).
pub fn document_to_pdf_with_base(doc: &Document, base: Option<&Path>) -> Result<Vec<u8>, PdfError> {
    if doc.pages.is_empty() {
        return Err(PdfError::EmptyDocument);
    }

    let mut store = ImageStore::new(base);
    let mut page_contents = Vec::with_capacity(doc.pages.len());
    let mut page_sizes = Vec::with_capacity(doc.pages.len());

    for (i, page) in doc.pages.iter().enumerate() {
        if !page.paper.is_positive() {
            return Err(PdfError::InvalidPage(format!(
                "page {i}: non-positive paper size"
            )));
        }
        let w_pt = mm_to_pt(page.paper.width_mm);
        let h_pt = mm_to_pt(page.paper.height_mm);
        page_sizes.push((w_pt, h_pt));
        page_contents.push(render_page_content(page, i, &mut store)?);
    }

    Ok(assemble_pdf(&page_sizes, &page_contents, &store.images))
}

pub fn write_document(doc: &Document, w: impl std::io::Write) -> Result<(), PdfError> {
    write_document_with_base(doc, None, w)
}

pub fn write_document_with_base(
    doc: &Document,
    base: Option<&Path>,
    mut w: impl std::io::Write,
) -> Result<(), PdfError> {
    let bytes = document_to_pdf_with_base(doc, base)?;
    w.write_all(&bytes)
        .map_err(|e| PdfError::Write(e.to_string()))?;
    Ok(())
}

fn resolve_image_path(path: &str, base: Option<&Path>) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    match base {
        Some(b) => b.join(p),
        None => p.to_path_buf(),
    }
}

fn load_raster_rgb(path: &Path) -> Result<EmbeddedImage, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() >= 3 && bytes[0] == 0xff && bytes[1] == 0xd8 && bytes[2] == 0xff {
        return load_jpeg_rgb(&bytes);
    }
    if bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
        return load_png_rgb_bytes(&bytes);
    }
    // Fall back by extension when magic is ambiguous.
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => load_jpeg_rgb(&bytes),
        Some("png") => load_png_rgb_bytes(&bytes),
        _ => {
            Err("unsupported image format (need PNG or JPEG; sniff magic or use .png/.jpg)".into())
        }
    }
}

fn load_jpeg_rgb(bytes: &[u8]) -> Result<EmbeddedImage, String> {
    let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(bytes));
    let pixels = decoder.decode().map_err(|e| e.to_string())?;
    let info = decoder
        .info()
        .ok_or_else(|| "JPEG missing header info".to_string())?;
    let width = u32::from(info.width);
    let height = u32::from(info.height);
    let rgb = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => pixels,
        jpeg_decoder::PixelFormat::L8 => {
            let mut out = Vec::with_capacity(pixels.len() * 3);
            for g in pixels {
                out.push(g);
                out.push(g);
                out.push(g);
            }
            out
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            // Approximate CMYK → RGB for embedded photos.
            let mut out = Vec::with_capacity((pixels.len() / 4) * 3);
            for px in pixels.chunks_exact(4) {
                let (c, m, y, k) = (
                    f32::from(px[0]) / 255.0,
                    f32::from(px[1]) / 255.0,
                    f32::from(px[2]) / 255.0,
                    f32::from(px[3]) / 255.0,
                );
                out.push(((1.0 - c) * (1.0 - k) * 255.0).round() as u8);
                out.push(((1.0 - m) * (1.0 - k) * 255.0).round() as u8);
                out.push(((1.0 - y) * (1.0 - k) * 255.0).round() as u8);
            }
            out
        }
        other => return Err(format!("unsupported JPEG pixel format {other:?}")),
    };
    if rgb.len() != (width as usize) * (height as usize) * 3 {
        return Err(format!(
            "JPEG size mismatch: got {} bytes for {width}x{height} RGB",
            rgb.len()
        ));
    }
    Ok(EmbeddedImage { width, height, rgb })
}

fn load_png_rgb_bytes(bytes: &[u8]) -> Result<EmbeddedImage, String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let width = info.width;
    let height = info.height;
    let rgb = match info.color_type {
        png::ColorType::Rgb => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgba => {
            let src = &buf[..info.buffer_size()];
            let mut out = Vec::with_capacity((width * height * 3) as usize);
            for px in src.chunks_exact(4) {
                out.push(px[0]);
                out.push(px[1]);
                out.push(px[2]);
            }
            out
        }
        png::ColorType::Grayscale => {
            let src = &buf[..info.buffer_size()];
            let mut out = Vec::with_capacity((width * height * 3) as usize);
            for &g in src {
                out.push(g);
                out.push(g);
                out.push(g);
            }
            out
        }
        png::ColorType::GrayscaleAlpha => {
            let src = &buf[..info.buffer_size()];
            let mut out = Vec::with_capacity((width * height * 3) as usize);
            for px in src.chunks_exact(2) {
                out.push(px[0]);
                out.push(px[0]);
                out.push(px[0]);
            }
            out
        }
        other => {
            return Err(format!(
                "unsupported PNG color type {other:?} (need RGB/RGBA/Gray)"
            ))
        }
    };
    if rgb.len() != (width as usize) * (height as usize) * 3 {
        return Err(format!(
            "PNG size mismatch: got {} bytes for {width}x{height} RGB",
            rgb.len()
        ));
    }
    Ok(EmbeddedImage { width, height, rgb })
}

fn render_page_content(
    page: &Page,
    index: usize,
    store: &mut ImageStore,
) -> Result<PageEmit, PdfError> {
    let mut ops = String::new();
    let mut opacities = BTreeSet::new();
    let mut images = BTreeSet::new();
    for (si, shape) in page.shapes.iter().enumerate() {
        let emit = render_shape(shape, &format!("page {index} shape {si}"), 1.0, store)?;
        ops.push_str(&emit.ops);
        opacities.extend(emit.opacities);
        images.extend(emit.images);
    }
    Ok(PageEmit {
        ops,
        opacities,
        images,
    })
}

fn render_shape(
    shape: &Shape,
    ctx: &str,
    parent_alpha: f64,
    store: &mut ImageStore,
) -> Result<PageEmit, PdfError> {
    match shape {
        Shape::Circle(c) => {
            if !c.is_drawable() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: circle not drawable"
                )));
            }
            Ok(ops_only(circle_path_ops(
                c.x_mm,
                c.y_mm,
                c.radius_mm,
                c.fill,
            )))
        }
        Shape::Rect(r) => {
            if !r.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: rect not drawable")));
            }
            Ok(ops_only(rect_path_ops(
                r.x_mm,
                r.y_mm,
                r.width_mm,
                r.height_mm,
                r.fill,
            )))
        }
        Shape::Ellipse(e) => {
            if !e.is_drawable() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: ellipse not drawable"
                )));
            }
            Ok(ops_only(ellipse_path_ops(
                e.x_mm, e.y_mm, e.rx_mm, e.ry_mm, e.fill,
            )))
        }
        Shape::Ring(r) => {
            if !r.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: ring not drawable")));
            }
            Ok(ops_only(ring_path_ops(
                r.x_mm,
                r.y_mm,
                r.radius_mm,
                r.width_mm,
                r.stroke,
            )))
        }
        Shape::Frame(f) => {
            if !f.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: frame not drawable")));
            }
            Ok(ops_only(frame_path_ops(
                f.x_mm,
                f.y_mm,
                f.width_mm,
                f.height_mm,
                f.stroke_width_mm,
                f.stroke,
            )))
        }
        Shape::Text(t) => {
            if !t.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: text not drawable")));
            }
            Ok(ops_only(text_ops(
                t.x_mm, t.y_mm, t.size_mm, &t.content, t.fill,
            )?))
        }
        Shape::Line(l) => {
            if !l.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: line not drawable")));
            }
            Ok(ops_only(line_ops(
                l.x1_mm, l.y1_mm, l.x2_mm, l.y2_mm, l.stroke, l.width_mm,
            )))
        }
        Shape::Polyline(p) => {
            if !p.is_drawable() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: polyline not drawable"
                )));
            }
            Ok(ops_only(polyline_ops(&p.points_mm, p.stroke, p.width_mm)))
        }
        Shape::Polygon(p) => {
            if !p.is_drawable() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: polygon not drawable"
                )));
            }
            Ok(ops_only(polygon_fill_ops(&p.points_mm, p.fill)))
        }
        Shape::Image(img) => {
            if !img.is_drawable() {
                return Err(PdfError::InvalidShape(format!("{ctx}: image not drawable")));
            }
            let id = store.get_or_load(&img.path)?;
            let mut images = BTreeSet::new();
            images.insert(id);
            Ok(PageEmit {
                ops: image_xobject_ops(img.x_mm, img.y_mm, img.width_mm, img.height_mm, id),
                opacities: BTreeSet::new(),
                images,
            })
        }
        Shape::Opacity { alpha, children } => {
            if !(0.0..=1.0).contains(alpha) || !alpha.is_finite() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: opacity must be finite in 0..=1"
                )));
            }
            let combined = parent_alpha * (*alpha);
            let pct = (combined * 100.0).round().clamp(0.0, 100.0) as u8;
            let mut ops = format!("q\n/GS{pct} gs\n");
            let mut opacities = BTreeSet::from([pct]);
            let mut images = BTreeSet::new();
            for (i, child) in children.iter().enumerate() {
                let emit = render_shape(child, &format!("{ctx}/{i}"), combined, store)?;
                ops.push_str(&emit.ops);
                opacities.extend(emit.opacities);
                images.extend(emit.images);
            }
            ops.push_str("Q\n");
            Ok(PageEmit {
                ops,
                opacities,
                images,
            })
        }
        Shape::Group {
            transform,
            children,
        } => {
            if !transform.is_finite() {
                return Err(PdfError::InvalidShape(format!(
                    "{ctx}: non-finite transform"
                )));
            }
            let mut ops = String::from("q\n");
            ops.push_str(&affine_cm_ops(*transform));
            let mut opacities = BTreeSet::new();
            let mut images = BTreeSet::new();
            for (i, child) in children.iter().enumerate() {
                let emit = render_shape(child, &format!("{ctx}/{i}"), parent_alpha, store)?;
                ops.push_str(&emit.ops);
                opacities.extend(emit.opacities);
                images.extend(emit.images);
            }
            ops.push_str("Q\n");
            Ok(PageEmit {
                ops,
                opacities,
                images,
            })
        }
    }
}

fn ops_only(ops: String) -> PageEmit {
    PageEmit {
        ops,
        opacities: BTreeSet::new(),
        images: BTreeSet::new(),
    }
}

fn circle_path_ops(x_mm: f64, y_mm: f64, r_mm: f64, fill: Color) -> String {
    // Bézier circle approximation with κ = 4*(√2-1)/3.
    let k = 0.5522847498;
    let x = mm_to_pt(x_mm);
    let y = mm_to_pt(y_mm);
    let r = mm_to_pt(r_mm);
    let kr = r * k;
    format!(
        "{r:.4} {g:.4} {b:.4} rg\n\
         {x:.4} {y0:.4} m\n\
         {x1:.4} {y0:.4} {x2:.4} {y1:.4} {x2:.4} {y:.4} c\n\
         {x2:.4} {y3:.4} {x1:.4} {y4:.4} {x:.4} {y4:.4} c\n\
         {x5:.4} {y4:.4} {x6:.4} {y3:.4} {x6:.4} {y:.4} c\n\
         {x6:.4} {y1:.4} {x5:.4} {y0:.4} {x:.4} {y0:.4} c\n\
         f\n",
        r = fill.r,
        g = fill.g,
        b = fill.b,
        y0 = y - r,
        x1 = x + kr,
        x2 = x + r,
        y1 = y - kr,
        y3 = y + kr,
        y4 = y + r,
        x5 = x - kr,
        x6 = x - r,
    )
}

fn rect_path_ops(x_mm: f64, y_mm: f64, w_mm: f64, h_mm: f64, fill: Color) -> String {
    format!(
        "{r:.4} {g:.4} {b:.4} rg\n{x:.4} {y:.4} {w:.4} {h:.4} re\nf\n",
        r = fill.r,
        g = fill.g,
        b = fill.b,
        x = mm_to_pt(x_mm),
        y = mm_to_pt(y_mm),
        w = mm_to_pt(w_mm),
        h = mm_to_pt(h_mm),
    )
}

fn ellipse_path_ops(x_mm: f64, y_mm: f64, rx_mm: f64, ry_mm: f64, fill: Color) -> String {
    let k = 0.5522847498;
    let x = mm_to_pt(x_mm);
    let y = mm_to_pt(y_mm);
    let rx = mm_to_pt(rx_mm);
    let ry = mm_to_pt(ry_mm);
    let kx = rx * k;
    let ky = ry * k;
    format!(
        "{r:.4} {g:.4} {b:.4} rg\n\
         {x:.4} {y0:.4} m\n\
         {x1:.4} {y0:.4} {x2:.4} {y1:.4} {x2:.4} {y:.4} c\n\
         {x2:.4} {y3:.4} {x1:.4} {y4:.4} {x:.4} {y4:.4} c\n\
         {x5:.4} {y4:.4} {x6:.4} {y3:.4} {x6:.4} {y:.4} c\n\
         {x6:.4} {y1:.4} {x5:.4} {y0:.4} {x:.4} {y0:.4} c\n\
         f\n",
        r = fill.r,
        g = fill.g,
        b = fill.b,
        y0 = y - ry,
        x1 = x + kx,
        x2 = x + rx,
        y1 = y - ky,
        y3 = y + ky,
        y4 = y + ry,
        x5 = x - kx,
        x6 = x - rx,
    )
}

fn ring_path_ops(x_mm: f64, y_mm: f64, r_mm: f64, width_mm: f64, stroke: Color) -> String {
    let k = 0.5522847498;
    let x = mm_to_pt(x_mm);
    let y = mm_to_pt(y_mm);
    let r = mm_to_pt(r_mm);
    let kr = r * k;
    format!(
        "q\n{r:.4} {g:.4} {b:.4} RG\n{w:.4} w\n\
         {x:.4} {y0:.4} m\n\
         {x1:.4} {y0:.4} {x2:.4} {y1:.4} {x2:.4} {y:.4} c\n\
         {x2:.4} {y3:.4} {x1:.4} {y4:.4} {x:.4} {y4:.4} c\n\
         {x5:.4} {y4:.4} {x6:.4} {y3:.4} {x6:.4} {y:.4} c\n\
         {x6:.4} {y1:.4} {x5:.4} {y0:.4} {x:.4} {y0:.4} c\ns\nQ\n",
        r = stroke.r,
        g = stroke.g,
        b = stroke.b,
        w = mm_to_pt(width_mm),
        y0 = y - r,
        x1 = x + kr,
        x2 = x + r,
        y1 = y - kr,
        y3 = y + kr,
        y4 = y + r,
        x5 = x - kr,
        x6 = x - r,
    )
}

fn frame_path_ops(
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
    stroke_width_mm: f64,
    stroke: Color,
) -> String {
    format!(
        "q\n{r:.4} {g:.4} {b:.4} RG\n{sw:.4} w\n{x:.4} {y:.4} {w:.4} {h:.4} re\nS\nQ\n",
        r = stroke.r,
        g = stroke.g,
        b = stroke.b,
        sw = mm_to_pt(stroke_width_mm),
        x = mm_to_pt(x_mm),
        y = mm_to_pt(y_mm),
        w = mm_to_pt(w_mm),
        h = mm_to_pt(h_mm),
    )
}

fn line_ops(x1: f64, y1: f64, x2: f64, y2: f64, stroke: Color, width_mm: f64) -> String {
    format!(
        "q\n{r:.4} {g:.4} {b:.4} RG\n{w:.4} w\n{x1:.4} {y1:.4} m\n{x2:.4} {y2:.4} l\nS\nQ\n",
        r = stroke.r,
        g = stroke.g,
        b = stroke.b,
        w = mm_to_pt(width_mm),
        x1 = mm_to_pt(x1),
        y1 = mm_to_pt(y1),
        x2 = mm_to_pt(x2),
        y2 = mm_to_pt(y2),
    )
}

fn polyline_ops(points: &[(f64, f64)], stroke: Color, width_mm: f64) -> String {
    let mut ops = format!(
        "q\n{r:.4} {g:.4} {b:.4} RG\n{w:.4} w\n",
        r = stroke.r,
        g = stroke.g,
        b = stroke.b,
        w = mm_to_pt(width_mm),
    );
    for (i, &(x, y)) in points.iter().enumerate() {
        if i == 0 {
            ops.push_str(&format!("{:.4} {:.4} m\n", mm_to_pt(x), mm_to_pt(y)));
        } else {
            ops.push_str(&format!("{:.4} {:.4} l\n", mm_to_pt(x), mm_to_pt(y)));
        }
    }
    ops.push_str("S\nQ\n");
    ops
}

fn polygon_fill_ops(points: &[(f64, f64)], fill: Color) -> String {
    let mut ops = format!(
        "{r:.4} {g:.4} {b:.4} rg\n",
        r = fill.r,
        g = fill.g,
        b = fill.b,
    );
    for (i, &(x, y)) in points.iter().enumerate() {
        if i == 0 {
            ops.push_str(&format!("{:.4} {:.4} m\n", mm_to_pt(x), mm_to_pt(y)));
        } else {
            ops.push_str(&format!("{:.4} {:.4} l\n", mm_to_pt(x), mm_to_pt(y)));
        }
    }
    ops.push_str("f\n");
    ops
}

fn image_xobject_ops(x_mm: f64, y_mm: f64, w_mm: f64, h_mm: f64, id: usize) -> String {
    format!(
        "q\n{w:.4} 0 0 {h:.4} {x:.4} {y:.4} cm\n/Im{id} Do\nQ\n",
        w = mm_to_pt(w_mm),
        h = mm_to_pt(h_mm),
        x = mm_to_pt(x_mm),
        y = mm_to_pt(y_mm),
    )
}

fn text_ops(
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    content: &str,
    fill: Color,
) -> Result<String, PdfError> {
    if content.is_empty() {
        return Ok(String::new());
    }
    if content.is_ascii() {
        let escaped = pdf_escape_ascii(content)?;
        let size_pt = mm_to_pt(size_mm);
        return Ok(format!(
            "BT\n/F1 {size:.4} Tf\n{r:.4} {g:.4} {b:.4} rg\n{x:.4} {y:.4} Td\n({escaped}) Tj\nET\n",
            size = size_pt,
            r = fill.r,
            g = fill.g,
            b = fill.b,
            x = mm_to_pt(x_mm),
            y = mm_to_pt(y_mm),
        ));
    }
    // Non-ASCII: draw glyph outlines from a system CJK font (no CID embed yet).
    outline_text_ops(x_mm, y_mm, size_mm, content, fill)
}

fn pdf_escape_ascii(s: &str) -> Result<String, PdfError> {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                return Err(PdfError::InvalidShape(format!(
                    "unsupported control char U+{:04X} in text",
                    c as u32
                )));
            }
            c => out.push(c),
        }
    }
    Ok(out)
}

fn system_cjk_font_path() -> Option<PathBuf> {
    let windir = std::env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into());
    let fonts = PathBuf::from(windir).join("Fonts");
    for name in [
        "NotoSansJP-VF.ttf",
        "NotoSansJP-VariableFont_wght.ttf",
        "NotoSans-Regular.ttf",
    ] {
        let p = fonts.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn outline_text_ops(
    x_mm: f64,
    y_mm: f64,
    size_mm: f64,
    content: &str,
    fill: Color,
) -> Result<String, PdfError> {
    let path = system_cjk_font_path().ok_or_else(|| {
        PdfError::InvalidShape(
            "non-ASCII text needs a system CJK font (expected NotoSansJP under %WINDIR%\\Fonts)"
                .into(),
        )
    })?;
    let data = std::fs::read(&path)
        .map_err(|e| PdfError::InvalidShape(format!("read font {}: {e}", path.display())))?;
    let face = ttf_parser::Face::parse(&data, 0)
        .map_err(|e| PdfError::InvalidShape(format!("parse font {}: {e}", path.display())))?;
    let units = f64::from(face.units_per_em());
    if units <= 0.0 {
        return Err(PdfError::InvalidShape("font units_per_em is zero".into()));
    }
    let size_pt = mm_to_pt(size_mm);
    let scale = size_pt / units;
    let mut x = mm_to_pt(x_mm);
    let y = mm_to_pt(y_mm);
    let mut ops = format!(
        "{r:.4} {g:.4} {b:.4} rg\n",
        r = fill.r,
        g = fill.g,
        b = fill.b
    );
    for ch in content.chars() {
        if ch == '\n' || ch == '\r' {
            continue;
        }
        let Some(gid) = face.glyph_index(ch) else {
            return Err(PdfError::InvalidShape(format!(
                "font missing glyph for U+{:04X}",
                ch as u32
            )));
        };
        let mut builder = PdfOutline {
            scale,
            origin_x: x,
            origin_y: y,
            last_x: 0.0,
            last_y: 0.0,
            ops: String::new(),
        };
        if face.outline_glyph(gid, &mut builder).is_none() {
            // Space / mark with no outline — still advance.
        } else {
            ops.push_str(&builder.ops);
            ops.push_str("f\n");
        }
        let adv = face
            .glyph_hor_advance(gid)
            .map(f64::from)
            .unwrap_or(units * 0.5);
        x += adv * scale;
    }
    Ok(ops)
}

struct PdfOutline {
    scale: f64,
    origin_x: f64,
    origin_y: f64,
    last_x: f32,
    last_y: f32,
    ops: String,
}

impl PdfOutline {
    fn map(&self, px: f32, py: f32) -> (f64, f64) {
        (
            self.origin_x + f64::from(px) * self.scale,
            self.origin_y + f64::from(py) * self.scale,
        )
    }
}

impl ttf_parser::OutlineBuilder for PdfOutline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.last_x = x;
        self.last_y = y;
        let (x, y) = self.map(x, y);
        self.ops.push_str(&format!("{x:.4} {y:.4} m\n"));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.last_x = x;
        self.last_y = y;
        let (x, y) = self.map(x, y);
        self.ops.push_str(&format!("{x:.4} {y:.4} l\n"));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let lx = self.last_x;
        let ly = self.last_y;
        let cx1 = lx + (2.0 / 3.0) * (x1 - lx);
        let cy1 = ly + (2.0 / 3.0) * (y1 - ly);
        let cx2 = x + (2.0 / 3.0) * (x1 - x);
        let cy2 = y + (2.0 / 3.0) * (y1 - y);
        self.last_x = x;
        self.last_y = y;
        let (x1, y1) = self.map(cx1, cy1);
        let (x2, y2) = self.map(cx2, cy2);
        let (x, y) = self.map(x, y);
        self.ops.push_str(&format!(
            "{x1:.4} {y1:.4} {x2:.4} {y2:.4} {x:.4} {y:.4} c\n"
        ));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.last_x = x;
        self.last_y = y;
        let (x1, y1) = self.map(x1, y1);
        let (x2, y2) = self.map(x2, y2);
        let (x, y) = self.map(x, y);
        self.ops.push_str(&format!(
            "{x1:.4} {y1:.4} {x2:.4} {y2:.4} {x:.4} {y:.4} c\n"
        ));
    }

    fn close(&mut self) {
        self.ops.push_str("h\n");
    }
}

fn affine_cm_ops(t: Affine) -> String {
    let s = 72.0 / 25.4;
    format!(
        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} cm\n",
        t.a,
        t.b,
        t.c,
        t.d,
        t.e * s,
        t.f * s
    )
}

fn mm_to_pt(mm: f64) -> f64 {
    mm * 72.0 / 25.4
}

/// Assemble PDF-1.4 with a shared Helvetica font and optional image XObjects.
fn assemble_pdf(
    page_sizes: &[(f64, f64)],
    contents: &[PageEmit],
    images: &[EmbeddedImage],
) -> Vec<u8> {
    assert_eq!(page_sizes.len(), contents.len());
    let n = page_sizes.len();
    let img_n = images.len();
    // 1 Catalog, 2 Pages, 3 Font, 4..3+img_n Images, then Pages, then contents
    let font_obj = 3;
    let image_obj0 = 4;
    let page_obj0 = 4 + img_n;
    let content_obj0 = page_obj0 + n;

    let mut objects: Vec<Vec<u8>> = Vec::new();
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());

    let kids: String = (0..n)
        .map(|i| format!("{} 0 R", page_obj0 + i))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!("<< /Type /Pages /Kids [{kids}] /Count {n} >>").into_bytes());

    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );

    for img in images {
        objects.push(image_xobject_bytes(img));
    }

    for (i, (w, h)) in page_sizes.iter().enumerate() {
        let content_id = content_obj0 + i;
        let gs = ext_gstate_dict(&contents[i].opacities);
        let xo = xobject_dict(&contents[i].images, image_obj0);
        let page_id_body = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w:.4} {h:.4}] \
             /Contents {content_id} 0 R \
             /Resources << /Font << /F1 {font_obj} 0 R >> {gs}{xo}>> >>"
        );
        objects.push(page_id_body.into_bytes());
    }

    for content in contents {
        let stream = content.ops.as_bytes();
        let mut obj = format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes();
        obj.extend_from_slice(stream);
        if !content.ops.ends_with('\n') {
            obj.push(b'\n');
        }
        obj.extend_from_slice(b"endstream");
        objects.push(obj);
    }

    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len() + 1);
    offsets.push(0);

    for (i, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        let id = i + 1;
        out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        out.extend_from_slice(obj);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_pos = out.len();
    let total_objs = objects.len() + 1;
    out.extend_from_slice(format!("xref\n0 {total_objs}\n").as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {total_objs} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n")
            .as_bytes(),
    );
    out
}

fn image_xobject_bytes(img: &EmbeddedImage) -> Vec<u8> {
    let mut obj = format!(
        "<< /Type /XObject /Subtype /Image /Width {} /Height {} \
         /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>\nstream\n",
        img.width,
        img.height,
        img.rgb.len()
    )
    .into_bytes();
    obj.extend_from_slice(&img.rgb);
    obj.extend_from_slice(b"\nendstream");
    obj
}

fn ext_gstate_dict(opacities: &BTreeSet<u8>) -> String {
    if opacities.is_empty() {
        return String::new();
    }
    let mut body = String::from("/ExtGState << ");
    for pct in opacities {
        let a = f64::from(*pct) / 100.0;
        body.push_str(&format!(
            "/GS{pct} << /Type /ExtGState /ca {a:.4} /CA {a:.4} >> "
        ));
    }
    body.push_str(">> ");
    body
}

fn xobject_dict(ids: &BTreeSet<usize>, image_obj0: usize) -> String {
    if ids.is_empty() {
        return String::new();
    }
    let mut body = String::from("/XObject << ");
    for id in ids {
        body.push_str(&format!("/Im{id} {} 0 R ", image_obj0 + id));
    }
    body.push_str(">> ");
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{
        Circle, Color, Document, Ellipse, Image, Line, Page, PaperSize, Rect, Shape, Text,
    };
    use std::io::Write;

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

    fn write_temp_png(path: &Path, w: u32, h: u32, rgb: &[u8]) {
        let file = std::fs::File::create(path).unwrap();
        let mut enc = png::Encoder::new(file, w, h);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(rgb).unwrap();
    }

    #[test]
    fn black_circle_pdf_has_header_eof_and_a4_mediabox() {
        let bytes = document_to_pdf(&sample_doc()).expect("pdf");
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("595."));
        assert!(text.contains("841."));
        assert!(text.contains("/Helvetica"));
    }

    #[test]
    fn text_and_line_emit_operators() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![
                Shape::Text(Text {
                    x_mm: 20.0,
                    y_mm: 250.0,
                    size_mm: 5.0,
                    content: "Hello".into(),
                    fill: Color::BLACK,
                }),
                Shape::Line(Line {
                    x1_mm: 20.0,
                    y1_mm: 200.0,
                    x2_mm: 100.0,
                    y2_mm: 200.0,
                    stroke: Color::RED,
                    width_mm: 0.5,
                }),
            ],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("BT"));
        assert!(text.contains("Tj"));
        assert!(text.contains(" m\n"));
        assert!(text.contains(" l\n"));
    }

    #[test]
    fn non_ascii_text_uses_outline_glyphs_when_font_present() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Text(Text {
                x_mm: 10.0,
                y_mm: 10.0,
                size_mm: 5.0,
                content: "日本語".into(),
                fill: Color::BLACK,
            })],
        });
        if system_cjk_font_path().is_none() {
            let err = document_to_pdf(&doc).unwrap_err();
            assert!(matches!(err, PdfError::InvalidShape(_)));
            return;
        }
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        // Outline path fill, not Helvetica Tj.
        assert!(!text.contains("(日本語)"));
        assert!(text.contains(" m\n") || text.contains(" c\n"));
        assert!(text.contains("\nf\n") || text.contains(" f\n"));
    }

    #[test]
    fn empty_document_errors() {
        assert!(matches!(
            document_to_pdf(&Document::default()),
            Err(PdfError::EmptyDocument)
        ));
    }

    #[test]
    fn rect_pdf_contains_re_operator() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 10.0,
                y_mm: 20.0,
                width_mm: 30.0,
                height_mm: 40.0,
                fill: Color::BLUE,
            })],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains(" re\n"));
    }

    #[test]
    fn ellipse_pdf_contains_curve_ops() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Ellipse(Ellipse {
                x_mm: 50.0,
                y_mm: 50.0,
                rx_mm: 20.0,
                ry_mm: 10.0,
                fill: Color::GREEN,
            })],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains(" c\n"));
        assert!(text.contains("\nf\n") || text.ends_with("f\n"));
    }

    #[test]
    fn letter_page_mediabox_and_opacity_extgstate() {
        let doc = Document::single_page(Page {
            paper: PaperSize::letter(),
            shapes: vec![Shape::Opacity {
                alpha: 0.5,
                children: vec![Shape::Circle(Circle {
                    x_mm: 100.0,
                    y_mm: 140.0,
                    radius_mm: 30.0,
                    fill: Color::RED,
                })],
            }],
        });
        let bytes = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        // 215.9mm × 279.4mm → ~612 × 792 pt
        assert!(text.contains("612."));
        assert!(text.contains("792."));
        assert!(text.contains("/ExtGState"));
        assert!(text.contains("/GS50"));
        assert!(text.contains("/ca 0.5000"));
        assert!(text.contains("/GS50 gs"));
    }

    #[test]
    fn embeds_png_as_image_xobject() {
        let dir = std::env::temp_dir().join("reciplexa-pdf-png-test");
        let _ = std::fs::create_dir_all(&dir);
        let png_path = dir.join("dot.png");
        write_temp_png(
            &png_path,
            2,
            2,
            &[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0],
        );

        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Image(Image {
                path: "dot.png".into(),
                x_mm: 10.0,
                y_mm: 20.0,
                width_mm: 40.0,
                height_mm: 30.0,
            })],
        });
        let bytes = document_to_pdf_with_base(&doc, Some(&dir)).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("/Subtype /Image"));
        assert!(text.contains("/Width 2"));
        assert!(text.contains("/Height 2"));
        assert!(text.contains("/Im0 Do"));
        assert!(text.contains("/XObject"));
    }

    #[test]
    fn missing_png_fails_fast() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Image(Image {
                path: "no-such-file.png".into(),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 10.0,
            })],
        });
        let err = document_to_pdf(&doc).unwrap_err();
        match err {
            PdfError::InvalidShape(msg) => assert!(msg.contains("image")),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn fixture_demo_png_embeds() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let examples = repo.join("examples");
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Image(Image {
                path: "figures/demo.png".into(),
                x_mm: 40.0,
                y_mm: 80.0,
                width_mm: 130.0,
                height_mm: 100.0,
            })],
        });
        let bytes = document_to_pdf_with_base(&doc, Some(&examples)).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("/Subtype /Image"));
        let _ = std::io::sink().write(&bytes);
    }

    #[test]
    fn fixture_demo_jpeg_embeds() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let examples = repo.join("examples");
        let doc = Document::single_page(Page {
            paper: PaperSize {
                width_mm: 210.0,
                height_mm: 297.0,
            },
            shapes: vec![Shape::Image(Image {
                path: "figures/demo.jpg".into(),
                x_mm: 20.0,
                y_mm: 20.0,
                width_mm: 60.0,
                height_mm: 60.0,
            })],
        });
        let bytes = document_to_pdf_with_base(&doc, Some(&examples)).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("/Subtype /Image"));
    }
}
