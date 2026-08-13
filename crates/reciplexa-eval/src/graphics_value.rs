//! Slice D strangler: lower package `graphics/*` eval records to scene documents.
//!
//! Package constructors emit tagged `RuntimeValue::Record`s
//! (`tag: "page"|"circle"|"fill"|"rgb"|…`). Interim CST `(page)/(circle)` remains
//! the GUI path; this bridge is additive and does not touch keyword tables.

use reciplexa_scene::{
    Affine, Circle, Color, Document, Ellipse, Frame, Image, Line, Page, PaperSize, Polygon,
    Polyline, Rect, Ring, Shape, Text,
};

use crate::value::RuntimeValue;

/// Fail-fast error when a graphics value cannot become a scene node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsValueError {
    pub message: String,
}

impl GraphicsValueError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Lower a package-built graphics value to a [`Document`].
///
/// Accepts a single `page` record, a `pages` record (list of pages), or a cons-list of
/// `page` records (multipage package `main` trees). Also accepts N5.1 `doc-page` trees
/// via [`crate::document_value::document_from_doc_value`].
pub fn document_from_graphics_value(v: &RuntimeValue) -> Result<Document, GraphicsValueError> {
    if let Ok(fields) = record_fields(v, "document") {
        if tag_of(fields) == Some("doc-page") {
            return crate::document_value::document_from_doc_value(v);
        }
    }
    if is_cons_or_nil(v) {
        let pages: Vec<Page> = cons_items(v)?
            .into_iter()
            .map(page_from_graphics_value)
            .collect::<Result<_, _>>()?;
        if pages.is_empty() {
            return Err(GraphicsValueError::new("empty page list"));
        }
        return Ok(Document { pages });
    }
    if let Ok(fields) = record_fields(v, "document") {
        if tag_of(fields) == Some("pages") {
            let items = field(fields, "items")
                .ok_or_else(|| GraphicsValueError::new("pages missing items"))?;
            let pages: Vec<Page> = cons_items(items)?
                .into_iter()
                .map(page_from_graphics_value)
                .collect::<Result<_, _>>()?;
            if pages.is_empty() {
                return Err(GraphicsValueError::new("pages expects at least one page"));
            }
            return Ok(Document { pages });
        }
    }
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
    let shapes = shapes_from_list_or_single(content)?;
    Ok(Page { paper, shapes })
}

/// Lower a package shape / paint / transform record to a scene [`Shape`].
///
/// Tags aligned with interim lower + `packages/graphics` constructors:
/// `circle`/`rect`/`ellipse`/`line`/`polyline`/`polygon`/`ring`/`frame`/
/// `text`/`image`/`group`/`translate`/`rotate`/`scale`/`opacity`/
/// `fill`/`stroke`/`paint`.
pub fn shape_from_graphics_value(v: &RuntimeValue) -> Result<Shape, GraphicsValueError> {
    let fields = record_fields(v, "shape")?;
    let tag = tag_of(fields).ok_or_else(|| GraphicsValueError::new("shape record missing tag"))?;
    match tag {
        "circle" => shape_circle(fields, Color::BLACK),
        "rect" => shape_rect(fields, Color::BLACK),
        "ellipse" => shape_ellipse(fields, Color::BLACK),
        "line" => shape_line(fields, Color::BLACK, 0.5),
        "polyline" => shape_polyline(fields, Color::BLACK, 0.5),
        "polygon" => shape_polygon(fields, Color::BLACK),
        "ring" => shape_ring(fields, Color::BLACK),
        "frame" => shape_frame(fields, None),
        "text" => shape_text(fields, Color::BLACK),
        "image" => shape_image(fields),
        "group" => shape_group(fields),
        "translate" => shape_translate(fields),
        "rotate" => shape_rotate(fields),
        "scale" => shape_scale(fields),
        "opacity" => shape_opacity(fields),
        "fill" => {
            let shape = field(fields, "shape")
                .ok_or_else(|| GraphicsValueError::new("fill missing shape"))?;
            let color_v = field(fields, "color")
                .ok_or_else(|| GraphicsValueError::new("fill missing color"))?;
            let color = color_from_graphics_value(color_v)?;
            shape_with_fill(shape, color)
        }
        "stroke" => {
            let shape = field(fields, "shape")
                .ok_or_else(|| GraphicsValueError::new("stroke missing shape"))?;
            let width = number_field(fields, "width")?;
            let color_v = field(fields, "color")
                .ok_or_else(|| GraphicsValueError::new("stroke missing color"))?;
            let color = color_from_graphics_value(color_v)?;
            shape_with_stroke(shape, width, color)
        }
        "paint" => shape_paint(fields),
        other => Err(GraphicsValueError::new(format!(
            "unsupported graphics shape tag `{other}`"
        ))),
    }
}

fn shape_with_fill(v: &RuntimeValue, fill: Color) -> Result<Shape, GraphicsValueError> {
    let fields = record_fields(v, "filled shape")?;
    let tag = tag_of(fields).ok_or_else(|| GraphicsValueError::new("shape missing tag"))?;
    match tag {
        "circle" => shape_circle(fields, fill),
        "rect" => shape_rect(fields, fill),
        "ellipse" => shape_ellipse(fields, fill),
        "polygon" => shape_polygon(fields, fill),
        "text" => shape_text(fields, fill),
        other => Err(GraphicsValueError::new(format!(
            "fill applied to unsupported tag `{other}`"
        ))),
    }
}

fn shape_with_stroke(
    v: &RuntimeValue,
    width: f64,
    stroke: Color,
) -> Result<Shape, GraphicsValueError> {
    let fields = record_fields(v, "stroked shape")?;
    let tag = tag_of(fields).ok_or_else(|| GraphicsValueError::new("shape missing tag"))?;
    match tag {
        "circle" => {
            let x = number_field(fields, "x")?;
            let y = number_field(fields, "y")?;
            let r = number_field(fields, "r")?;
            let ring = Ring {
                x_mm: x,
                y_mm: y,
                radius_mm: r,
                width_mm: width,
                stroke,
            };
            if !ring.is_drawable() {
                return Err(GraphicsValueError::new("stroke circle is not drawable"));
            }
            Ok(Shape::Ring(ring))
        }
        "rect" => {
            let x = number_field(fields, "x")?;
            let y = number_field(fields, "y")?;
            let w = number_field(fields, "w")?;
            let h = number_field(fields, "h")?;
            let frame = Frame {
                x_mm: x,
                y_mm: y,
                width_mm: w,
                height_mm: h,
                stroke_width_mm: width,
                stroke,
            };
            if !frame.is_drawable() {
                return Err(GraphicsValueError::new("stroke rect is not drawable"));
            }
            Ok(Shape::Frame(frame))
        }
        "line" => shape_line(fields, stroke, width),
        "polyline" => shape_polyline(fields, stroke, width),
        other => Err(GraphicsValueError::new(format!(
            "stroke applied to unsupported tag `{other}`"
        ))),
    }
}

fn shape_paint(fields: &[(String, RuntimeValue)]) -> Result<Shape, GraphicsValueError> {
    let shape =
        field(fields, "shape").ok_or_else(|| GraphicsValueError::new("paint missing shape"))?;
    let fill_v =
        field(fields, "fill").ok_or_else(|| GraphicsValueError::new("paint missing fill"))?;
    let stroke_width = number_field(fields, "stroke-width")?;
    let stroke_v = field(fields, "stroke-color")
        .ok_or_else(|| GraphicsValueError::new("paint missing stroke-color"))?;
    let fill = color_from_graphics_value(fill_v)?;
    let stroke = color_from_graphics_value(stroke_v)?;
    let fill_is_transparent = color_alpha(fill_v).is_some_and(|a| a <= 0.0);

    let mut children = Vec::new();
    if !fill_is_transparent {
        if let Ok(s) = shape_with_fill(shape, fill) {
            children.push(s);
        }
    }
    if stroke_width > 0.0 {
        if let Ok(s) = shape_with_stroke(shape, stroke_width, stroke) {
            children.push(s);
        }
    }
    if children.is_empty() {
        // Fall back to stroke-only line-like paint, else filled shape.
        if stroke_width > 0.0 {
            shape_with_stroke(shape, stroke_width, stroke)
        } else {
            shape_with_fill(shape, fill)
        }
    } else if children.len() == 1 {
        Ok(children.pop().expect("len 1"))
    } else {
        Ok(Shape::Group {
            transform: Affine::identity(),
            children,
        })
    }
}

fn color_alpha(v: &RuntimeValue) -> Option<f64> {
    let fields = record_fields(v, "color").ok()?;
    number_field(fields, "a").ok()
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

fn shape_rect(fields: &[(String, RuntimeValue)], fill: Color) -> Result<Shape, GraphicsValueError> {
    let x = number_field(fields, "x")?;
    let y = number_field(fields, "y")?;
    let w = number_field(fields, "w")?;
    let h = number_field(fields, "h")?;
    let rect = Rect {
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
        fill,
    };
    if !rect.is_drawable() {
        return Err(GraphicsValueError::new(format!(
            "rect is not drawable (w={w}, h={h})"
        )));
    }
    Ok(Shape::Rect(rect))
}

fn shape_ellipse(
    fields: &[(String, RuntimeValue)],
    fill: Color,
) -> Result<Shape, GraphicsValueError> {
    let x = number_field(fields, "x")?;
    let y = number_field(fields, "y")?;
    let rx = number_field(fields, "rx")?;
    let ry = number_field(fields, "ry")?;
    let ellipse = Ellipse {
        x_mm: x,
        y_mm: y,
        rx_mm: rx,
        ry_mm: ry,
        fill,
    };
    if !ellipse.is_drawable() {
        return Err(GraphicsValueError::new(format!(
            "ellipse is not drawable (rx={rx}, ry={ry})"
        )));
    }
    Ok(Shape::Ellipse(ellipse))
}

fn shape_line(
    fields: &[(String, RuntimeValue)],
    stroke: Color,
    width: f64,
) -> Result<Shape, GraphicsValueError> {
    let x1 = number_field(fields, "x1")?;
    let y1 = number_field(fields, "y1")?;
    let x2 = number_field(fields, "x2")?;
    let y2 = number_field(fields, "y2")?;
    let line = Line {
        x1_mm: x1,
        y1_mm: y1,
        x2_mm: x2,
        y2_mm: y2,
        stroke,
        width_mm: width,
    };
    if !line.is_drawable() {
        return Err(GraphicsValueError::new("line is not drawable"));
    }
    Ok(Shape::Line(line))
}

fn shape_polyline(
    fields: &[(String, RuntimeValue)],
    stroke: Color,
    width: f64,
) -> Result<Shape, GraphicsValueError> {
    let points = field(fields, "points")
        .ok_or_else(|| GraphicsValueError::new("polyline missing points"))?;
    let coords = number_list(points, "polyline points")?;
    if coords.len() < 4 || !coords.len().is_multiple_of(2) {
        return Err(GraphicsValueError::new(
            "polyline needs an even number of coordinates (≥4)",
        ));
    }
    let mut points_mm = Vec::with_capacity(coords.len() / 2);
    for i in (0..coords.len()).step_by(2) {
        points_mm.push((coords[i], coords[i + 1]));
    }
    let poly = Polyline {
        points_mm,
        stroke,
        width_mm: width,
    };
    if !poly.is_drawable() {
        return Err(GraphicsValueError::new("polyline is not drawable"));
    }
    Ok(Shape::Polyline(poly))
}

fn shape_polygon(
    fields: &[(String, RuntimeValue)],
    fill: Color,
) -> Result<Shape, GraphicsValueError> {
    let points =
        field(fields, "points").ok_or_else(|| GraphicsValueError::new("polygon missing points"))?;
    let coords = number_list(points, "polygon points")?;
    if coords.len() < 6 || !coords.len().is_multiple_of(2) {
        return Err(GraphicsValueError::new(
            "polygon needs an even number of coordinates (≥6)",
        ));
    }
    let mut points_mm = Vec::with_capacity(coords.len() / 2);
    for i in (0..coords.len()).step_by(2) {
        points_mm.push((coords[i], coords[i + 1]));
    }
    let poly = Polygon { points_mm, fill };
    if !poly.is_drawable() {
        return Err(GraphicsValueError::new("polygon is not drawable"));
    }
    Ok(Shape::Polygon(poly))
}

fn shape_ring(
    fields: &[(String, RuntimeValue)],
    default_stroke: Color,
) -> Result<Shape, GraphicsValueError> {
    let x = number_field(fields, "x")?;
    let y = number_field(fields, "y")?;
    let r = number_field(fields, "r")?;
    let width = number_field(fields, "width")?;
    let stroke = optional_color_field(fields, "color")?.unwrap_or(default_stroke);
    let ring = Ring {
        x_mm: x,
        y_mm: y,
        radius_mm: r,
        width_mm: width,
        stroke,
    };
    if !ring.is_drawable() {
        return Err(GraphicsValueError::new("ring is not drawable"));
    }
    Ok(Shape::Ring(ring))
}

fn shape_frame(
    fields: &[(String, RuntimeValue)],
    stroke_override: Option<Color>,
) -> Result<Shape, GraphicsValueError> {
    let x = number_field(fields, "x")?;
    let y = number_field(fields, "y")?;
    let w = number_field(fields, "w")?;
    let h = number_field(fields, "h")?;
    let width = number_field(fields, "width")?;
    let stroke = match stroke_override {
        Some(c) => c,
        None => optional_color_field(fields, "color")?.unwrap_or(Color::BLACK),
    };
    let frame = Frame {
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
        stroke_width_mm: width,
        stroke,
    };
    if !frame.is_drawable() {
        return Err(GraphicsValueError::new("frame is not drawable"));
    }
    Ok(Shape::Frame(frame))
}

fn shape_text(fields: &[(String, RuntimeValue)], fill: Color) -> Result<Shape, GraphicsValueError> {
    let x = number_field(fields, "x")?;
    let y = number_field(fields, "y")?;
    let size = number_field(fields, "size")?;
    let content = string_field(fields, "content")?;
    let width_mm = optional_number_field(fields, "w")?;
    let height_mm = optional_number_field(fields, "h")?;
    let text = Text {
        x_mm: x,
        y_mm: y,
        size_mm: size,
        width_mm,
        height_mm,
        content,
        fill,
    };
    if !text.is_drawable() {
        return Err(GraphicsValueError::new("text is not drawable"));
    }
    Ok(Shape::Text(text))
}

fn shape_image(fields: &[(String, RuntimeValue)]) -> Result<Shape, GraphicsValueError> {
    let path = string_field(fields, "path")?;
    let x = number_field(fields, "x")?;
    let y = number_field(fields, "y")?;
    let w = number_field(fields, "w")?;
    let h = number_field(fields, "h")?;
    let image = Image {
        path,
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
    };
    if !image.is_drawable() {
        return Err(GraphicsValueError::new("image is not drawable"));
    }
    Ok(Shape::Image(image))
}

fn shape_group(fields: &[(String, RuntimeValue)]) -> Result<Shape, GraphicsValueError> {
    let children_v = field(fields, "children")
        .ok_or_else(|| GraphicsValueError::new("group missing children"))?;
    let children = shapes_from_list_or_single(children_v)?;
    if children.is_empty() {
        return Err(GraphicsValueError::new("group expects at least one shape"));
    }
    Ok(Shape::Group {
        transform: Affine::identity(),
        children,
    })
}

fn shape_translate(fields: &[(String, RuntimeValue)]) -> Result<Shape, GraphicsValueError> {
    let dx = number_field(fields, "dx")?;
    let dy = number_field(fields, "dy")?;
    let child =
        field(fields, "child").ok_or_else(|| GraphicsValueError::new("translate missing child"))?;
    Ok(Shape::Group {
        transform: Affine::translate(dx, dy),
        children: shapes_from_list_or_single(child)?,
    })
}

fn shape_rotate(fields: &[(String, RuntimeValue)]) -> Result<Shape, GraphicsValueError> {
    let degrees = number_field(fields, "degrees")?;
    let child =
        field(fields, "child").ok_or_else(|| GraphicsValueError::new("rotate missing child"))?;
    Ok(Shape::Group {
        transform: Affine::rotate_deg(degrees),
        children: shapes_from_list_or_single(child)?,
    })
}

fn shape_scale(fields: &[(String, RuntimeValue)]) -> Result<Shape, GraphicsValueError> {
    let sx = number_field(fields, "sx")?;
    let sy = number_field(fields, "sy")?;
    let child =
        field(fields, "child").ok_or_else(|| GraphicsValueError::new("scale missing child"))?;
    Ok(Shape::Group {
        transform: Affine::scale(sx, sy),
        children: shapes_from_list_or_single(child)?,
    })
}

fn shape_opacity(fields: &[(String, RuntimeValue)]) -> Result<Shape, GraphicsValueError> {
    let alpha = number_field(fields, "alpha")?;
    if !(0.0..=1.0).contains(&alpha) {
        return Err(GraphicsValueError::new(
            "opacity alpha must be a finite number in 0..=1",
        ));
    }
    let child =
        field(fields, "child").ok_or_else(|| GraphicsValueError::new("opacity missing child"))?;
    Ok(Shape::Opacity {
        alpha,
        children: shapes_from_list_or_single(child)?,
    })
}

/// Color from package `rgb` / `rgba` / `srgb` records (alpha ignored for scene Color).
pub fn color_from_graphics_value(v: &RuntimeValue) -> Result<Color, GraphicsValueError> {
    let fields = record_fields(v, "color")?;
    let tag = tag_of(fields).unwrap_or("");
    match tag {
        "rgb" | "rgba" | "srgb" | "" => {
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

fn shapes_from_list_or_single(v: &RuntimeValue) -> Result<Vec<Shape>, GraphicsValueError> {
    if is_cons_or_nil(v) {
        let mut out = Vec::new();
        for item in cons_items(v)? {
            out.push(shape_from_graphics_value(item)?);
        }
        Ok(out)
    } else {
        Ok(vec![shape_from_graphics_value(v)?])
    }
}

fn is_cons_or_nil(v: &RuntimeValue) -> bool {
    matches!(
        v,
        RuntimeValue::Variant { tag, .. } if tag == "cons" || tag == "nil"
    )
}

fn cons_items(v: &RuntimeValue) -> Result<Vec<&RuntimeValue>, GraphicsValueError> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match cur {
            RuntimeValue::Variant { tag, .. } if tag == "nil" => break,
            RuntimeValue::Variant { tag, payload } if tag == "cons" => {
                let Some(payload) = payload.as_ref() else {
                    return Err(GraphicsValueError::new("cons variant missing payload"));
                };
                let fields = record_fields(payload, "cons")?;
                let head = field(fields, "head")
                    .ok_or_else(|| GraphicsValueError::new("cons missing head"))?;
                let tail = field(fields, "tail")
                    .ok_or_else(|| GraphicsValueError::new("cons missing tail"))?;
                out.push(head);
                cur = tail;
            }
            _ => {
                return Err(GraphicsValueError::new(
                    "expected cons/nil list of shapes or numbers",
                ));
            }
        }
    }
    Ok(out)
}

fn number_list(v: &RuntimeValue, ctx: &str) -> Result<Vec<f64>, GraphicsValueError> {
    if !is_cons_or_nil(v) {
        return Err(GraphicsValueError::new(format!(
            "{ctx}: expected number list"
        )));
    }
    let mut out = Vec::new();
    for item in cons_items(v)? {
        out.push(as_f64(item).ok_or_else(|| {
            GraphicsValueError::new(format!("{ctx}: list element is not numeric"))
        })?);
    }
    Ok(out)
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

fn optional_number_field(
    fields: &[(String, RuntimeValue)],
    name: &str,
) -> Result<Option<f64>, GraphicsValueError> {
    match field(fields, name) {
        None => Ok(None),
        Some(v) => Ok(Some(as_f64(v).ok_or_else(|| {
            GraphicsValueError::new(format!("field `{name}` is not numeric"))
        })?)),
    }
}

fn optional_color_field(
    fields: &[(String, RuntimeValue)],
    name: &str,
) -> Result<Option<Color>, GraphicsValueError> {
    match field(fields, name) {
        None => Ok(None),
        Some(v) => Ok(Some(color_from_graphics_value(v)?)),
    }
}

fn string_field(
    fields: &[(String, RuntimeValue)],
    name: &str,
) -> Result<String, GraphicsValueError> {
    let v = field(fields, name)
        .ok_or_else(|| GraphicsValueError::new(format!("missing string field `{name}`")))?;
    match v {
        RuntimeValue::String(s) => Ok(s.clone()),
        _ => Err(GraphicsValueError::new(format!(
            "field `{name}` is not a string"
        ))),
    }
}

fn as_f64(v: &RuntimeValue) -> Option<f64> {
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Some(*n),
        RuntimeValue::Int(n) => Some(*n as f64),
        _ => None,
    }
}
