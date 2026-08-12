//! Slice D strangler: lower package `graphics/*` eval records to scene documents.
//!
//! Package constructors emit tagged `RuntimeValue::Record`s
//! (`tag: "page"|"circle"|"fill"|"rgb"|…`). Interim CST `(page)/(circle)` remains
//! the GUI path; this bridge is additive and does not touch keyword tables.

use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};

use crate::value::RuntimeValue;

/// Fail-fast error when a graphics value cannot become a scene node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsValueError {
    pub message: String,
}

impl GraphicsValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Lower a package-built graphics value (typically a `page` record) to a [`Document`].
pub fn document_from_graphics_value(v: &RuntimeValue) -> Result<Document, GraphicsValueError> {
    let page = page_from_graphics_value(v)?;
    Ok(Document::single_page(page))
}

/// Lower a `tag: "page"` record to a scene [`Page`].
pub fn page_from_graphics_value(v: &RuntimeValue) -> Result<Page, GraphicsValueError> {
    let fields = record_fields(v, "page")?;
    expect_tag(fields, "page")?;
    let size = field(fields, "size").ok_or_else(|| GraphicsValueError::new("page missing size"))?;
    let content =
        field(fields, "content").ok_or_else(|| GraphicsValueError::new("page missing content"))?;
    let paper = paper_from_size_value(size)?;
    let shape = shape_from_graphics_value(content)?;
    Ok(Page {
        paper,
        shapes: vec![shape],
    })
}

/// Lower a package shape / paint / transform record to a scene [`Shape`].
///
/// v1 tags: `circle`, `fill` (with nested shape + color). Default fill is black.
pub fn shape_from_graphics_value(v: &RuntimeValue) -> Result<Shape, GraphicsValueError> {
    let fields = record_fields(v, "shape")?;
    let tag = tag_of(fields).ok_or_else(|| GraphicsValueError::new("shape record missing tag"))?;
    match tag {
        "circle" => shape_circle(fields, Color::BLACK),
        "fill" => {
            let shape = field(fields, "shape")
                .ok_or_else(|| GraphicsValueError::new("fill missing shape"))?;
            let color_v = field(fields, "color")
                .ok_or_else(|| GraphicsValueError::new("fill missing color"))?;
            let color = color_from_graphics_value(color_v)?;
            shape_with_fill(shape, color)
        }
        other => Err(GraphicsValueError::new(format!(
            "unsupported graphics shape tag `{other}` (v1: circle, fill)"
        ))),
    }
}

fn shape_with_fill(v: &RuntimeValue, fill: Color) -> Result<Shape, GraphicsValueError> {
    let fields = record_fields(v, "filled shape")?;
    let tag = tag_of(fields).ok_or_else(|| GraphicsValueError::new("shape missing tag"))?;
    match tag {
        "circle" => shape_circle(fields, fill),
        other => Err(GraphicsValueError::new(format!(
            "fill applied to unsupported tag `{other}` (v1: circle)"
        ))),
    }
}

fn shape_circle(
    fields: &[(String, RuntimeValue)],
    fill: Color,
) -> Result<Shape, GraphicsValueError> {
    let x = number_field(fields, "x")?;
    let y = number_field(fields, "y")?;
    let r = number_field(fields, "r")?;
    let circle = Circle {
        x_mm: x,
        y_mm: y,
        radius_mm: r,
        fill,
    };
    if !circle.is_drawable() {
        return Err(GraphicsValueError::new(format!(
            "circle is not drawable (radius={r})"
        )));
    }
    Ok(Shape::Circle(circle))
}

/// Color from package `rgb` / `rgba` records (alpha ignored for scene Color).
pub fn color_from_graphics_value(v: &RuntimeValue) -> Result<Color, GraphicsValueError> {
    let fields = record_fields(v, "color")?;
    let tag = tag_of(fields).unwrap_or("");
    match tag {
        "rgb" | "rgba" | "" => {
            let r = number_field(fields, "r")?;
            let g = number_field(fields, "g")?;
            let b = number_field(fields, "b")?;
            let c = Color::new(r, g, b);
            if !c.is_channel_valid() {
                return Err(GraphicsValueError::new("rgb channels must be in 0..=1"));
            }
            Ok(c)
        }
        other => Err(GraphicsValueError::new(format!(
            "unsupported color tag `{other}`"
        ))),
    }
}

fn paper_from_size_value(v: &RuntimeValue) -> Result<PaperSize, GraphicsValueError> {
    let fields = record_fields(v, "page size")?;
    let width_mm = number_field(fields, "width")?;
    let height_mm = number_field(fields, "height")?;
    let paper = PaperSize {
        width_mm,
        height_mm,
    };
    if !paper.is_positive() {
        return Err(GraphicsValueError::new("paper size must be positive"));
    }
    Ok(paper)
}

fn record_fields<'a>(
    v: &'a RuntimeValue,
    ctx: &str,
) -> Result<&'a [(String, RuntimeValue)], GraphicsValueError> {
    match v {
        RuntimeValue::Record(fields) => Ok(fields.as_slice()),
        _ => Err(GraphicsValueError::new(format!(
            "{ctx}: expected record, got non-record"
        ))),
    }
}

fn field<'a>(fields: &'a [(String, RuntimeValue)], name: &str) -> Option<&'a RuntimeValue> {
    fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

fn tag_of(fields: &[(String, RuntimeValue)]) -> Option<&str> {
    match field(fields, "tag")? {
        RuntimeValue::String(s) => Some(s.as_str()),
        // Kernel surface lower maps "circle"/"rect"/"text" string lits → ShapeTag.
        RuntimeValue::ShapeTag(s) => Some(s.as_str()),
        _ => None,
    }
}

fn expect_tag(fields: &[(String, RuntimeValue)], want: &str) -> Result<(), GraphicsValueError> {
    match tag_of(fields) {
        Some(t) if t == want => Ok(()),
        Some(t) => Err(GraphicsValueError::new(format!(
            "expected tag `{want}`, got `{t}`"
        ))),
        None => Err(GraphicsValueError::new(format!(
            "expected tag `{want}`, missing tag"
        ))),
    }
}

fn number_field(fields: &[(String, RuntimeValue)], name: &str) -> Result<f64, GraphicsValueError> {
    let v = field(fields, name)
        .ok_or_else(|| GraphicsValueError::new(format!("missing numeric field `{name}`")))?;
    as_f64(v).ok_or_else(|| GraphicsValueError::new(format!("field `{name}` is not numeric")))
}

fn as_f64(v: &RuntimeValue) -> Option<f64> {
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Some(*n),
        RuntimeValue::Int(n) => Some(*n as f64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::Shape;

    fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
        RuntimeValue::Record(
            fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn num(n: f64) -> RuntimeValue {
        RuntimeValue::F64(n)
    }

    fn circle_rec(x: f64, y: f64, r: f64) -> RuntimeValue {
        rec(vec![
            ("tag", RuntimeValue::String("circle".into())),
            ("x", num(x)),
            ("y", num(y)),
            ("r", num(r)),
        ])
    }

    fn rgb_rec(r: f64, g: f64, b: f64) -> RuntimeValue {
        rec(vec![
            ("tag", RuntimeValue::String("rgb".into())),
            ("r", num(r)),
            ("g", num(g)),
            ("b", num(b)),
        ])
    }

    fn a4_size() -> RuntimeValue {
        rec(vec![("width", num(210.0)), ("height", num(297.0))])
    }

    #[test]
    fn page_fill_circle_matches_scene_geometry() {
        let page = rec(vec![
            ("tag", RuntimeValue::String("page".into())),
            ("size", a4_size()),
            (
                "content",
                rec(vec![
                    ("tag", RuntimeValue::String("fill".into())),
                    ("shape", circle_rec(105.0, 148.5, 40.0)),
                    ("color", rgb_rec(0.0, 0.0, 0.0)),
                ]),
            ),
        ]);
        let doc = document_from_graphics_value(&page).expect("bridge");
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.pages[0].paper, PaperSize::a4());
        match &doc.pages[0].shapes[0] {
            Shape::Circle(c) => {
                assert_eq!(c.x_mm, 105.0);
                assert_eq!(c.y_mm, 148.5);
                assert_eq!(c.radius_mm, 40.0);
                assert_eq!(c.fill, Color::BLACK);
            }
            other => panic!("expected circle, got {other:?}"),
        }
    }

    #[test]
    fn bare_circle_defaults_to_black() {
        let shape = shape_from_graphics_value(&circle_rec(1.0, 2.0, 3.0)).unwrap();
        match shape {
            Shape::Circle(c) => {
                assert_eq!(c.fill, Color::BLACK);
                assert_eq!(c.radius_mm, 3.0);
            }
            other => panic!("expected circle, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_shape_tag() {
        let v = rec(vec![("tag", RuntimeValue::String("hexagon".into()))]);
        assert!(shape_from_graphics_value(&v).is_err());
    }

    #[test]
    fn rejects_non_drawable_circle() {
        assert!(shape_from_graphics_value(&circle_rec(0.0, 0.0, 0.0)).is_err());
    }
}
