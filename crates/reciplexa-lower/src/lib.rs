//! Minimal CST → scene lowering for pages, shapes, and Glisp-like transforms.
//!
//! Supported forms (Lisp mode):
//!
//! ```text
//! (page a4
//!   (circle <x> <y> <r>)
//!   (circle <x> <y> <r> red)
//!   (circle <x> <y> <r> (rgb 0.1 0.2 0.3))
//!   (translate <tx> <ty> <shape…>)
//!   (rotate <deg> <shape…>)
//!   (scale <s> <shape…>)
//!   (scale <sx> <sy> <shape…>)
//!   (text <x> <y> <size-mm> "…")
//!   (line <x1> <y1> <x2> <y2> [color [width-mm]]))
//! ```

#![forbid(unsafe_code)]

mod sync;

pub use sync::{
    collect_drag_targets, collect_drag_targets_page, nudge_drag_target, nudge_first_translate,
    DragTarget, SyncError,
};

use reciplexa_scene::{
    Affine, Circle, Color, Document, Ellipse, Frame, Line, Page, PaperSize, Polyline, Rect, Ring,
    Shape, Text,
};
use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// Lowering / validation error (fail-fast: no partial scene for rendering).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerError {
    pub message: String,
}

impl LowerError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parse source and lower it to a [`Document`].
pub fn lower_source(input: &str) -> Result<Document, LowerError> {
    let parse = parse_source(input);
    let root = parse
        .into_result()
        .map_err(|errs| LowerError::new(format!("parse error: {}", errs[0].message)))?;
    lower_syntax(&root)
}

/// Lower an already-parsed CST root (`SourceFile`).
pub fn lower_syntax(root: &SyntaxNode) -> Result<Document, LowerError> {
    if root.kind() != SyntaxKind::SourceFile {
        return Err(LowerError::new(format!(
            "expected SourceFile, got {:?}",
            root.kind()
        )));
    }

    let forms: Vec<SyntaxNode> = root.children().collect();
    if forms.is_empty() {
        return Err(LowerError::new("empty source: expected a (page …) form"));
    }

    let mut pages = Vec::with_capacity(forms.len());
    for form in forms {
        pages.push(lower_page(&form)?);
    }
    Ok(Document { pages })
}

fn lower_page(node: &SyntaxNode) -> Result<Page, LowerError> {
    let items = list_items(node, "page")?;
    let head = ident_at(&items, 0, "page")?;
    if head != "page" {
        return Err(LowerError::new(format!(
            "expected head `page`, found `{head}`"
        )));
    }
    if items.len() < 2 {
        return Err(LowerError::new(
            "`page` requires a paper size, e.g. (page a4 …)",
        ));
    }

    let paper = match atom_ident(&items[1])? {
        "a4" => PaperSize::a4(),
        other => {
            return Err(LowerError::new(format!(
                "unknown paper size `{other}` (only `a4` for now)"
            )))
        }
    };

    let mut shapes = Vec::new();
    for item in items.iter().skip(2) {
        shapes.push(lower_shape_child(item)?);
    }
    Ok(Page { paper, shapes })
}

fn lower_shape_child(child: &Child) -> Result<Shape, LowerError> {
    match child {
        Child::Node(n) => lower_shape(n),
        Child::Token(t) => Err(LowerError::new(format!(
            "expected a shape list, got token {:?}",
            t.kind()
        ))),
    }
}

fn lower_shape(node: &SyntaxNode) -> Result<Shape, LowerError> {
    let items = list_items(node, "shape")?;
    let head = ident_at(&items, 0, "shape")?;
    match head {
        "circle" => lower_circle(&items),
        "rect" => lower_rect(&items),
        "ellipse" => lower_ellipse(&items),
        "ring" => lower_ring(&items),
        "frame" => lower_frame(&items),
        "text" => lower_text(&items),
        "line" => lower_line(&items),
        "polyline" => lower_polyline(&items),
        "group" => lower_group(&items),
        "translate" => lower_translate(&items),
        "rotate" => lower_rotate(&items),
        "scale" => lower_scale(&items),
        other => Err(LowerError::new(format!("unknown shape `{other}`"))),
    }
}

fn lower_circle(items: &[Child]) -> Result<Shape, LowerError> {
    // (circle x y r) | (circle x y r color)
    if items.len() != 4 && items.len() != 5 {
        return Err(LowerError::new(
            "`circle` expects (circle x y r) or (circle x y r color)",
        ));
    }
    let x = number_at(items, 1, "circle x")?;
    let y = number_at(items, 2, "circle y")?;
    let r = number_at(items, 3, "circle radius")?;
    let fill = if items.len() == 5 {
        lower_color(&items[4])?
    } else {
        Color::BLACK
    };
    let circle = Circle {
        x_mm: x,
        y_mm: y,
        radius_mm: r,
        fill,
    };
    if !circle.is_drawable() {
        return Err(LowerError::new(format!(
            "circle is not drawable (radius={r})"
        )));
    }
    Ok(Shape::Circle(circle))
}

fn lower_rect(items: &[Child]) -> Result<Shape, LowerError> {
    // (rect x y w h) | (rect x y w h color)
    if items.len() != 5 && items.len() != 6 {
        return Err(LowerError::new(
            "`rect` expects (rect x y w h) or (rect x y w h color)",
        ));
    }
    let x = number_at(items, 1, "rect x")?;
    let y = number_at(items, 2, "rect y")?;
    let w = number_at(items, 3, "rect width")?;
    let h = number_at(items, 4, "rect height")?;
    let fill = if items.len() == 6 {
        lower_color(&items[5])?
    } else {
        Color::BLACK
    };
    let rect = Rect {
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
        fill,
    };
    if !rect.is_drawable() {
        return Err(LowerError::new(format!(
            "rect is not drawable (w={w}, h={h})"
        )));
    }
    Ok(Shape::Rect(rect))
}

fn lower_ellipse(items: &[Child]) -> Result<Shape, LowerError> {
    // (ellipse x y rx ry) | (ellipse x y rx ry color)
    if items.len() != 5 && items.len() != 6 {
        return Err(LowerError::new(
            "`ellipse` expects (ellipse x y rx ry) or with a trailing color",
        ));
    }
    let x = number_at(items, 1, "ellipse x")?;
    let y = number_at(items, 2, "ellipse y")?;
    let rx = number_at(items, 3, "ellipse rx")?;
    let ry = number_at(items, 4, "ellipse ry")?;
    let fill = if items.len() == 6 {
        lower_color(&items[5])?
    } else {
        Color::BLACK
    };
    let ellipse = Ellipse {
        x_mm: x,
        y_mm: y,
        rx_mm: rx,
        ry_mm: ry,
        fill,
    };
    if !ellipse.is_drawable() {
        return Err(LowerError::new(format!(
            "ellipse is not drawable (rx={rx}, ry={ry})"
        )));
    }
    Ok(Shape::Ellipse(ellipse))
}

fn lower_ring(items: &[Child]) -> Result<Shape, LowerError> {
    // (ring x y r width) | (ring x y r width color)
    if items.len() != 5 && items.len() != 6 {
        return Err(LowerError::new(
            "`ring` expects (ring x y r width-mm [color])",
        ));
    }
    let x = number_at(items, 1, "ring x")?;
    let y = number_at(items, 2, "ring y")?;
    let r = number_at(items, 3, "ring radius")?;
    let width = number_at(items, 4, "ring width")?;
    let stroke = if items.len() == 6 {
        lower_color(&items[5])?
    } else {
        Color::BLACK
    };
    let ring = Ring {
        x_mm: x,
        y_mm: y,
        radius_mm: r,
        width_mm: width,
        stroke,
    };
    if !ring.is_drawable() {
        return Err(LowerError::new("ring is not drawable"));
    }
    Ok(Shape::Ring(ring))
}

fn lower_frame(items: &[Child]) -> Result<Shape, LowerError> {
    // (frame x y w h width) | (frame x y w h width color)
    if items.len() != 6 && items.len() != 7 {
        return Err(LowerError::new(
            "`frame` expects (frame x y w h stroke-width-mm [color])",
        ));
    }
    let x = number_at(items, 1, "frame x")?;
    let y = number_at(items, 2, "frame y")?;
    let w = number_at(items, 3, "frame width")?;
    let h = number_at(items, 4, "frame height")?;
    let sw = number_at(items, 5, "frame stroke width")?;
    let stroke = if items.len() == 7 {
        lower_color(&items[6])?
    } else {
        Color::BLACK
    };
    let frame = Frame {
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
        stroke_width_mm: sw,
        stroke,
    };
    if !frame.is_drawable() {
        return Err(LowerError::new("frame is not drawable"));
    }
    Ok(Shape::Frame(frame))
}

fn lower_text(items: &[Child]) -> Result<Shape, LowerError> {
    // (text x y size "content") | (text x y size "content" color)
    if items.len() != 5 && items.len() != 6 {
        return Err(LowerError::new(
            "`text` expects (text x y size-mm \"…\") or with a trailing color",
        ));
    }
    let x = number_at(items, 1, "text x")?;
    let y = number_at(items, 2, "text y")?;
    let size = number_at(items, 3, "text size")?;
    let content = string_at(items, 4, "text content")?;
    let fill = if items.len() == 6 {
        lower_color(&items[5])?
    } else {
        Color::BLACK
    };
    let text = Text {
        x_mm: x,
        y_mm: y,
        size_mm: size,
        content,
        fill,
    };
    if !text.is_drawable() {
        return Err(LowerError::new("text is not drawable"));
    }
    Ok(Shape::Text(text))
}

fn lower_line(items: &[Child]) -> Result<Shape, LowerError> {
    // (line x1 y1 x2 y2) | (line x1 y1 x2 y2 color) | (line x1 y1 x2 y2 color width)
    if items.len() < 5 || items.len() > 7 {
        return Err(LowerError::new(
            "`line` expects (line x1 y1 x2 y2 [color [width-mm]])",
        ));
    }
    let x1 = number_at(items, 1, "line x1")?;
    let y1 = number_at(items, 2, "line y1")?;
    let x2 = number_at(items, 3, "line x2")?;
    let y2 = number_at(items, 4, "line y2")?;
    let mut stroke = Color::BLACK;
    let mut width = 0.5;
    if items.len() >= 6 {
        stroke = lower_color(&items[5])?;
    }
    if items.len() == 7 {
        width = number_at(items, 6, "line width")?;
    }
    let line = Line {
        x1_mm: x1,
        y1_mm: y1,
        x2_mm: x2,
        y2_mm: y2,
        stroke,
        width_mm: width,
    };
    if !line.is_drawable() {
        return Err(LowerError::new("line is not drawable"));
    }
    Ok(Shape::Line(line))
}

fn lower_polyline(items: &[Child]) -> Result<Shape, LowerError> {
    // (polyline x1 y1 x2 y2 … [color [width]])
    if items.len() < 5 {
        return Err(LowerError::new(
            "`polyline` expects at least two points (x y)×2",
        ));
    }
    let mut end = items.len();
    let mut stroke = Color::BLACK;
    let mut width = 0.5;
    // Optional trailing width number
    if end >= 2 && matches!(&items[end - 1], Child::Token(t) if t.kind() == SyntaxKind::Number) {
        // Could be last y, or width. Disambiguate: if preceding is color, it's width.
        if end >= 3 && is_color_child(&items[end - 2]) {
            width = number_at(items, end - 1, "polyline width")?;
            stroke = lower_color(&items[end - 2])?;
            end -= 2;
        }
    } else if end >= 2 && is_color_child(&items[end - 1]) {
        stroke = lower_color(&items[end - 1])?;
        end -= 1;
    }
    let coords = &items[1..end];
    if coords.len() < 4 || coords.len() % 2 != 0 {
        return Err(LowerError::new(
            "`polyline` needs an even number of coordinates (≥4)",
        ));
    }
    let mut points_mm = Vec::with_capacity(coords.len() / 2);
    for i in (0..coords.len()).step_by(2) {
        let x = number_at_slice(coords, i, "polyline x")?;
        let y = number_at_slice(coords, i + 1, "polyline y")?;
        points_mm.push((x, y));
    }
    let poly = Polyline {
        points_mm,
        stroke,
        width_mm: width,
    };
    if !poly.is_drawable() {
        return Err(LowerError::new("polyline is not drawable"));
    }
    Ok(Shape::Polyline(poly))
}

fn is_color_child(child: &Child) -> bool {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Color::named(t.text()).is_some(),
        Child::Node(n) => list_items(n, "color")
            .ok()
            .and_then(|items| ident_at(&items, 0, "color").ok().map(|h| h == "rgb"))
            .unwrap_or(false),
        _ => false,
    }
}

fn number_at_slice(items: &[Child], index: usize, ctx: &str) -> Result<f64, LowerError> {
    number_at(items, index, ctx)
}

fn string_at(items: &[Child], index: usize, ctx: &str) -> Result<String, LowerError> {
    let Some(child) = items.get(index) else {
        return Err(LowerError::new(format!("{ctx}: missing string")));
    };
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::String => {
            let raw = t.text();
            if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
                Ok(unescape_string(&raw[1..raw.len() - 1]))
            } else {
                Err(LowerError::new(format!("{ctx}: malformed string token")))
            }
        }
        Child::Token(t) => Err(LowerError::new(format!(
            "{ctx}: expected String, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "{ctx}: expected String, got node {:?}",
            n.kind()
        ))),
    }
}

fn unescape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn lower_color(child: &Child) -> Result<Color, LowerError> {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Color::named(t.text())
            .ok_or_else(|| LowerError::new(format!("unknown color `{}`", t.text()))),
        Child::Node(n) => {
            let items = list_items(n, "color")?;
            let head = ident_at(&items, 0, "color")?;
            if head != "rgb" {
                return Err(LowerError::new(format!(
                    "unknown color form `{head}` (expected rgb)"
                )));
            }
            if items.len() != 4 {
                return Err(LowerError::new("`rgb` expects three channels"));
            }
            let r = number_at(&items, 1, "rgb r")?;
            let g = number_at(&items, 2, "rgb g")?;
            let b = number_at(&items, 3, "rgb b")?;
            let c = Color::new(r, g, b);
            if !c.is_channel_valid() {
                return Err(LowerError::new("rgb channels must be in 0..=1"));
            }
            Ok(c)
        }
        Child::Token(t) => Err(LowerError::new(format!(
            "expected color ident or (rgb …), got {:?}",
            t.kind()
        ))),
    }
}

fn lower_translate(items: &[Child]) -> Result<Shape, LowerError> {
    // (translate tx ty shape…)
    if items.len() < 4 {
        return Err(LowerError::new(
            "`translate` expects tx ty and at least one shape",
        ));
    }
    let tx = number_at(items, 1, "translate x")?;
    let ty = number_at(items, 2, "translate y")?;
    let children = lower_shape_tail(&items[3..])?;
    Ok(Shape::Group {
        transform: Affine::translate(tx, ty),
        children,
    })
}

fn lower_group(items: &[Child]) -> Result<Shape, LowerError> {
    // (group shape…) — identity transform, for layering / annotations.
    if items.len() < 2 {
        return Err(LowerError::new("`group` expects at least one shape"));
    }
    Ok(Shape::Group {
        transform: Affine::identity(),
        children: lower_shape_tail(&items[1..])?,
    })
}

fn lower_rotate(items: &[Child]) -> Result<Shape, LowerError> {
    // (rotate deg shape…)
    if items.len() < 3 {
        return Err(LowerError::new(
            "`rotate` expects degrees and at least one shape",
        ));
    }
    let deg = number_at(items, 1, "rotate degrees")?;
    let children = lower_shape_tail(&items[2..])?;
    Ok(Shape::Group {
        transform: Affine::rotate_deg(deg),
        children,
    })
}

fn lower_scale(items: &[Child]) -> Result<Shape, LowerError> {
    // (scale s shape…) | (scale sx sy shape…)
    if items.len() < 3 {
        return Err(LowerError::new(
            "`scale` expects factor(s) and at least one shape",
        ));
    }
    let first = number_at(items, 1, "scale")?;
    let (transform, rest) = if items.len() >= 4
        && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
    {
        let sy = number_at(items, 2, "scale y")?;
        (Affine::scale(first, sy), &items[3..])
    } else {
        (Affine::scale_uniform(first), &items[2..])
    };
    if rest.is_empty() {
        return Err(LowerError::new("`scale` needs at least one shape body"));
    }
    Ok(Shape::Group {
        transform,
        children: lower_shape_tail(rest)?,
    })
}

fn lower_shape_tail(items: &[Child]) -> Result<Vec<Shape>, LowerError> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(lower_shape_child(item)?);
    }
    Ok(out)
}

#[derive(Debug)]
enum Child {
    Node(SyntaxNode),
    Token(SyntaxToken),
}

fn list_items(node: &SyntaxNode, ctx: &str) -> Result<Vec<Child>, LowerError> {
    if node.kind() != SyntaxKind::List {
        return Err(LowerError::new(format!(
            "{ctx}: expected a (…) list, got {:?}",
            node.kind()
        )));
    }
    let mut items = Vec::new();
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia()
                    || matches!(t.kind(), SyntaxKind::LParen | SyntaxKind::RParen)
                {
                    continue;
                }
                items.push(Child::Token(t));
            }
            SyntaxElement::Node(n) => items.push(Child::Node(n)),
        }
    }
    Ok(items)
}

fn ident_at<'a>(items: &'a [Child], index: usize, ctx: &str) -> Result<&'a str, LowerError> {
    let Some(child) = items.get(index) else {
        return Err(LowerError::new(format!("{ctx}: missing element {index}")));
    };
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Ok(t.text()),
        Child::Token(t) => Err(LowerError::new(format!(
            "{ctx}: expected Ident, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "{ctx}: expected Ident, got node {:?}",
            n.kind()
        ))),
    }
}

fn atom_ident(child: &Child) -> Result<&str, LowerError> {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Ok(t.text()),
        Child::Token(t) => Err(LowerError::new(format!(
            "expected paper Ident, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "expected paper Ident, got node {:?}",
            n.kind()
        ))),
    }
}

fn number_at(items: &[Child], index: usize, ctx: &str) -> Result<f64, LowerError> {
    let Some(child) = items.get(index) else {
        return Err(LowerError::new(format!("{ctx}: missing number")));
    };
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Number => t
            .text()
            .parse()
            .map_err(|_| LowerError::new(format!("{ctx}: cannot parse number `{}`", t.text()))),
        Child::Token(t) => Err(LowerError::new(format!(
            "{ctx}: expected Number, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "{ctx}: expected Number, got node {:?}",
            n.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_pdf::document_to_pdf;
    use reciplexa_scene::Shape;

    const BLACK_CIRCLE: &str = r#"
; A4 black circle (M3)
(page a4
  (circle 105 148.5 40))
"#;

    const TRANSFORMED: &str = r#"
(page a4
  (translate 105 148.5
    (rotate 30
      (scale 1.5
        (circle 0 0 20 red)))))
"#;

    // --- validity ---

    #[test]
    fn lowers_a4_black_circle() {
        let doc = lower_source(BLACK_CIRCLE).expect("lower");
        assert_eq!(doc.pages.len(), 1);
        let page = &doc.pages[0];
        assert_eq!(page.paper, PaperSize::a4());
        match &page.shapes[0] {
            Shape::Circle(c) => {
                assert_eq!(c.x_mm, 105.0);
                assert_eq!(c.y_mm, 148.5);
                assert_eq!(c.radius_mm, 40.0);
                assert_eq!(c.fill, Color::BLACK);
            }
            _ => panic!("expected circle"),
        }
    }

    #[test]
    fn lowers_nested_transforms_and_named_color() {
        let doc = lower_source(TRANSFORMED).unwrap();
        let outer = &doc.pages[0].shapes[0];
        match outer {
            Shape::Group {
                transform,
                children,
            } => {
                assert_eq!(*transform, Affine::translate(105.0, 148.5));
                match &children[0] {
                    Shape::Group { children, .. } => match &children[0] {
                        Shape::Group { children, .. } => match &children[0] {
                            Shape::Circle(c) => {
                                assert_eq!(c.fill, Color::RED);
                                assert_eq!(c.radius_mm, 20.0);
                            }
                            _ => panic!("expected circle"),
                        },
                        _ => panic!("expected scale group"),
                    },
                    _ => panic!("expected rotate group"),
                }
            }
            _ => panic!("expected translate group"),
        }
    }

    #[test]
    fn lowers_rgb_and_nonuniform_scale() {
        let src = "(page a4 (scale 2 3 (circle 0 0 5 (rgb 0.2 0.4 0.6))))";
        let doc = lower_source(src).unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Group {
                transform,
                children,
            } => {
                assert_eq!(*transform, Affine::scale(2.0, 3.0));
                match &children[0] {
                    Shape::Circle(c) => assert_eq!(c.fill, Color::new(0.2, 0.4, 0.6)),
                    _ => panic!("expected circle"),
                }
            }
            _ => panic!("expected group"),
        }
    }

    #[test]
    fn end_to_end_transformed_source_to_pdf() {
        let pdf = document_to_pdf(&lower_source(TRANSFORMED).unwrap()).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains(" cm\n"));
    }

    #[test]
    fn fixture_file_round_trips_to_pdf() {
        let src = include_str!("../../../examples/black_circle.rpx");
        let pdf = document_to_pdf(&lower_source(src).unwrap()).unwrap();
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
    }

    #[test]
    fn trivia_does_not_affect_lowering() {
        let src = "(page   a4\n\n  (circle 1 2 3))";
        let doc = lower_source(src).unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Circle(c) => {
                assert_eq!(c.x_mm, 1.0);
                assert_eq!(c.y_mm, 2.0);
                assert_eq!(c.radius_mm, 3.0);
            }
            _ => panic!("expected circle"),
        }
    }

    #[test]
    fn lowers_rect_with_color() {
        let doc = lower_source("(page a4 (rect 10 20 30 40 blue))").unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Rect(r) => {
                assert_eq!(r.width_mm, 30.0);
                assert_eq!(r.height_mm, 40.0);
                assert_eq!(r.fill, Color::BLUE);
            }
            _ => panic!("expected rect"),
        }
    }

    #[test]
    fn bad_rect_arity_fails() {
        assert!(lower_source("(page a4 (rect 1 2 3))").is_err());
    }

    #[test]
    fn lowers_text_and_line() {
        let src = r#"(page a4
  (text 20 250 5 "Hello" blue)
  (line 20 200 100 200 red 1))"#;
        let doc = lower_source(src).unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Text(t) => {
                assert_eq!(t.content, "Hello");
                assert_eq!(t.fill, Color::BLUE);
            }
            _ => panic!("expected text"),
        }
        match &doc.pages[0].shapes[1] {
            Shape::Line(l) => {
                assert_eq!(l.x2_mm, 100.0);
                assert_eq!(l.stroke, Color::RED);
                assert_eq!(l.width_mm, 1.0);
            }
            _ => panic!("expected line"),
        }
        let pdf = document_to_pdf(&doc).unwrap();
        assert!(String::from_utf8_lossy(&pdf).contains("(Hello) Tj"));
    }

    #[test]
    fn lowers_ellipse() {
        let doc = lower_source("(page a4 (ellipse 105 148.5 60 30 green))").unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Ellipse(e) => {
                assert_eq!(e.rx_mm, 60.0);
                assert_eq!(e.ry_mm, 30.0);
                assert_eq!(e.fill, Color::GREEN);
            }
            _ => panic!("expected ellipse"),
        }
    }

    #[test]
    fn lowers_ring_and_frame() {
        let src = "(page a4 (ring 1 2 3 0.5 red) (frame 0 0 10 20 1 blue))";
        let doc = lower_source(src).unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Ring(r) => {
                assert_eq!(r.radius_mm, 3.0);
                assert_eq!(r.stroke, Color::RED);
            }
            _ => panic!("expected ring"),
        }
        match &doc.pages[0].shapes[1] {
            Shape::Frame(f) => {
                assert_eq!(f.width_mm, 10.0);
                assert_eq!(f.stroke, Color::BLUE);
            }
            _ => panic!("expected frame"),
        }
        assert!(document_to_pdf(&doc).is_ok());
    }

    #[test]
    fn lowers_polyline() {
        let doc = lower_source("(page a4 (polyline 0 0 10 10 20 0 blue 1.5))").unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Polyline(p) => {
                assert_eq!(p.points_mm.len(), 3);
                assert_eq!(p.stroke, Color::BLUE);
                assert_eq!(p.width_mm, 1.5);
            }
            _ => panic!("expected polyline"),
        }
    }

    #[test]
    fn lowers_group_and_multipage() {
        let src = include_str!("../../../examples/two_pages.rpx");
        let doc = lower_source(src).unwrap();
        assert_eq!(doc.pages.len(), 2);
        match &doc.pages[1].shapes[1] {
            Shape::Group {
                transform,
                children,
            } => {
                assert_eq!(*transform, Affine::identity());
                assert_eq!(children.len(), 2);
            }
            _ => panic!("expected group"),
        }
        let pdf = document_to_pdf(&doc).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/Count 2"));
    }

    // --- defect ---

    #[test]
    fn empty_source_fails() {
        assert!(lower_source("").is_err());
        assert!(lower_source("   \n").is_err());
    }

    #[test]
    fn parse_error_fails_fast() {
        let err = lower_source("(page a4").unwrap_err();
        assert!(err.message.contains("parse error"));
    }

    #[test]
    fn unknown_shape_fails() {
        let err = lower_source("(page a4 (square 1 2 3))").unwrap_err();
        assert!(err.message.contains("unknown shape"));
    }

    #[test]
    fn wrong_circle_arity_fails() {
        assert!(lower_source("(page a4 (circle 1 2))").is_err());
        assert!(lower_source("(page a4 (circle 1 2 3 4 5))").is_err());
    }

    #[test]
    fn zero_radius_fails() {
        let err = lower_source("(page a4 (circle 1 2 0))").unwrap_err();
        assert!(err.message.contains("not drawable"));
    }

    #[test]
    fn unknown_paper_fails() {
        let err = lower_source("(page letter (circle 1 2 3))").unwrap_err();
        assert!(err.message.contains("unknown paper"));
    }

    #[test]
    fn non_page_head_fails() {
        let err = lower_source("(sheet a4)").unwrap_err();
        assert!(err.message.contains("expected head `page`"));
    }

    #[test]
    fn unknown_color_and_bad_rgb_fail() {
        assert!(lower_source("(page a4 (circle 0 0 1 puce))").is_err());
        assert!(lower_source("(page a4 (circle 0 0 1 (rgb 2 0 0)))").is_err());
        assert!(lower_source("(page a4 (circle 0 0 1 (rgb 0 0)))").is_err());
    }

    #[test]
    fn transform_without_body_fails() {
        assert!(lower_source("(page a4 (translate 1 2))").is_err());
        assert!(lower_source("(page a4 (rotate 90))").is_err());
        assert!(lower_source("(page a4 (scale 2))").is_err());
    }
}
