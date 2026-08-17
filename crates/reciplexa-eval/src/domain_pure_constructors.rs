//! Direct Native v2 runtime for pure record constructors (DN2-2).

use crate::domain_native::{
    ColorSrgbOp, DomainNativeOp, GraphicsColorOp, GraphicsPageOp, GraphicsShapesOp,
};
use crate::domain_native_failure::{take1, take2, take3, take4, take5, take6};
use crate::value::RuntimeValue;
use crate::EvalError;

pub fn call_pure_constructor(
    op: DomainNativeOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        DomainNativeOp::ColorSrgb(sub) => call_color_srgb(sub, args),
        DomainNativeOp::GraphicsColor(sub) => call_graphics_color(sub, args),
        DomainNativeOp::GraphicsPage(sub) => call_graphics_page(sub, args),
        DomainNativeOp::GraphicsShapes(sub) => call_graphics_shapes(sub, args),
        other => Err(EvalError {
            message: format!("not a pure constructor op: {other:?}"),
        }),
    }
}

fn call_color_srgb(op: ColorSrgbOp, args: &[RuntimeValue]) -> Result<RuntimeValue, EvalError> {
    match op {
        ColorSrgbOp::Srgb => {
            let [r, g, b] = take3(args, "`color/srgb srgb`")?;
            rgb_tag_record("srgb", r, g, b)
        }
        ColorSrgbOp::Rgb => {
            let [r, g, b] = take3(args, "`color/srgb rgb`")?;
            rgb_tag_record("rgb", r, g, b)
        }
        ColorSrgbOp::Rgba => {
            let [r, g, b, a] = take4(args, "`color/srgb rgba`")?;
            rgba_tag_record(r, g, b, a)
        }
        ColorSrgbOp::FromByte => {
            let [r, g, b] = take3(args, "`color/srgb from-byte`")?;
            Ok(record(vec![
                ("tag".into(), str_val("rgb")),
                ("r".into(), num_val(as_f64(r, "r")? / 255.0)),
                ("g".into(), num_val(as_f64(g, "g")? / 255.0)),
                ("b".into(), num_val(as_f64(b, "b")? / 255.0)),
            ]))
        }
        ColorSrgbOp::Gray => {
            let [level] = take1(args, "`color/srgb gray`")?;
            rgb_tag_record("rgb", level, level, level)
        }
        ColorSrgbOp::Black => constant_rgb(0, 0, 0),
        ColorSrgbOp::White => constant_rgb(1, 1, 1),
        ColorSrgbOp::Red => constant_rgb(1, 0, 0),
        ColorSrgbOp::Green => constant_rgb(0, 1, 0),
        ColorSrgbOp::Blue => constant_rgb(0, 0, 1),
        ColorSrgbOp::Yellow => constant_rgb(1, 1, 0),
        ColorSrgbOp::Cyan => constant_rgb(0, 1, 1),
        ColorSrgbOp::Magenta => constant_rgb(1, 0, 1),
        ColorSrgbOp::Orange => Ok(record(vec![
            ("tag".into(), str_val("rgb")),
            ("r".into(), int_val(1)),
            ("g".into(), num_val(0.5)),
            ("b".into(), int_val(0)),
        ])),
        ColorSrgbOp::Gray50 => Ok(record(vec![
            ("tag".into(), str_val("rgb")),
            ("r".into(), num_val(0.5)),
            ("g".into(), num_val(0.5)),
            ("b".into(), num_val(0.5)),
        ])),
        ColorSrgbOp::Transparent => Ok(record(vec![
            ("tag".into(), str_val("rgba")),
            ("r".into(), int_val(0)),
            ("g".into(), int_val(0)),
            ("b".into(), int_val(0)),
            ("a".into(), int_val(0)),
        ])),
        ColorSrgbOp::WithAlpha => {
            let [color, a] = take2(args, "`color/srgb with-alpha`")?;
            Ok(record(vec![
                ("tag".into(), str_val("rgba")),
                ("r".into(), field_value(color, "r")?),
                ("g".into(), field_value(color, "g")?),
                ("b".into(), field_value(color, "b")?),
                ("a".into(), a.clone()),
            ]))
        }
    }
}

fn call_graphics_color(
    op: GraphicsColorOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        GraphicsColorOp::Rgb => {
            let [r, g, b] = take3(args, "`graphics/color rgb`")?;
            rgb_tag_record("rgb", r, g, b)
        }
        GraphicsColorOp::Rgba => {
            let [r, g, b, a] = take4(args, "`graphics/color rgba`")?;
            rgba_tag_record(r, g, b, a)
        }
        GraphicsColorOp::Black => constant_rgb(0, 0, 0),
        GraphicsColorOp::White => constant_rgb(1, 1, 1),
        GraphicsColorOp::Red => constant_rgb(1, 0, 0),
        GraphicsColorOp::Green => constant_rgb(0, 1, 0),
        GraphicsColorOp::Blue => constant_rgb(0, 0, 1),
        GraphicsColorOp::Gray => Ok(record(vec![
            ("tag".into(), str_val("rgb")),
            ("r".into(), num_val(0.5)),
            ("g".into(), num_val(0.5)),
            ("b".into(), num_val(0.5)),
        ])),
        GraphicsColorOp::Transparent => Ok(record(vec![
            ("tag".into(), str_val("rgba")),
            ("r".into(), int_val(0)),
            ("g".into(), int_val(0)),
            ("b".into(), int_val(0)),
            ("a".into(), int_val(0)),
        ])),
    }
}

fn call_graphics_page(
    op: GraphicsPageOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        GraphicsPageOp::A4 => size_record(210, 297),
        GraphicsPageOp::Letter => size_record_f64(215.9, 279.4),
        GraphicsPageOp::A5 => size_record(148, 210),
        GraphicsPageOp::A3 => size_record(297, 420),
        GraphicsPageOp::Legal => size_record_f64(215.9, 355.6),
        GraphicsPageOp::Square => size_record(210, 210),
        GraphicsPageOp::Page => {
            let [size, content] = take2(args, "`graphics/page page`")?;
            Ok(record(vec![
                ("tag".into(), tag_value("page")),
                ("size".into(), size.clone()),
                ("content".into(), content.clone()),
            ]))
        }
        GraphicsPageOp::Pages => {
            let [items] = take1(args, "`graphics/page pages`")?;
            Ok(record(vec![
                ("tag".into(), tag_value("pages")),
                ("items".into(), items.clone()),
            ]))
        }
        GraphicsPageOp::PageSize => {
            let [width, height] = take2(args, "`graphics/page page-size`")?;
            Ok(record(vec![
                ("width".into(), width.clone()),
                ("height".into(), height.clone()),
            ]))
        }
    }
}

fn call_graphics_shapes(
    op: GraphicsShapesOp,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, EvalError> {
    match op {
        GraphicsShapesOp::Circle => {
            let [x, y, r] = take3(args, "`graphics/shapes circle`")?;
            tagged("circle", vec![("x", x), ("y", y), ("r", r)])
        }
        GraphicsShapesOp::Rect => {
            let [x, y, w, h] = take4(args, "`graphics/shapes rect`")?;
            tagged("rect", vec![("x", x), ("y", y), ("w", w), ("h", h)])
        }
        GraphicsShapesOp::Ellipse => {
            let [x, y, rx, ry] = take4(args, "`graphics/shapes ellipse`")?;
            tagged("ellipse", vec![("x", x), ("y", y), ("rx", rx), ("ry", ry)])
        }
        GraphicsShapesOp::Line => {
            let [x1, y1, x2, y2] = take4(args, "`graphics/shapes line`")?;
            tagged("line", vec![("x1", x1), ("y1", y1), ("x2", x2), ("y2", y2)])
        }
        GraphicsShapesOp::Path => {
            let [points, closed] = take2(args, "`graphics/shapes path`")?;
            tagged("path", vec![("points", points), ("closed", closed)])
        }
        GraphicsShapesOp::Polyline => {
            let [points] = take1(args, "`graphics/shapes polyline`")?;
            tagged("polyline", vec![("points", points)])
        }
        GraphicsShapesOp::Polygon => {
            let [points] = take1(args, "`graphics/shapes polygon`")?;
            tagged("polygon", vec![("points", points)])
        }
        GraphicsShapesOp::Ring => {
            let [x, y, r, width] = take4(args, "`graphics/shapes ring`")?;
            tagged("ring", vec![("x", x), ("y", y), ("r", r), ("width", width)])
        }
        GraphicsShapesOp::Frame => {
            let [x, y, w, h, width, color] = take6(args, "`graphics/shapes frame`")?;
            tagged(
                "frame",
                vec![
                    ("x", x),
                    ("y", y),
                    ("w", w),
                    ("h", h),
                    ("width", width),
                    ("color", color),
                ],
            )
        }
        GraphicsShapesOp::Group => {
            let [children] = take1(args, "`graphics/shapes group`")?;
            tagged("group", vec![("children", children)])
        }
        GraphicsShapesOp::Text => {
            let [x, y, size, content] = take4(args, "`graphics/shapes text`")?;
            tagged(
                "text",
                vec![("x", x), ("y", y), ("size", size), ("content", content)],
            )
        }
        GraphicsShapesOp::TextBox => {
            let [x, y, size, w, h, content] = take6(args, "`graphics/shapes text-box`")?;
            tagged(
                "text",
                vec![
                    ("x", x),
                    ("y", y),
                    ("size", size),
                    ("w", w),
                    ("h", h),
                    ("content", content),
                ],
            )
        }
        GraphicsShapesOp::Image => {
            let [path, x, y, w, h] = take5(args, "`graphics/shapes image`")?;
            tagged(
                "image",
                vec![("path", path), ("x", x), ("y", y), ("w", w), ("h", h)],
            )
        }
        GraphicsShapesOp::Translate => {
            let [dx, dy, child] = take3(args, "`graphics/shapes translate`")?;
            tagged("translate", vec![("dx", dx), ("dy", dy), ("child", child)])
        }
        GraphicsShapesOp::Rotate => {
            let [degrees, child] = take2(args, "`graphics/shapes rotate`")?;
            tagged("rotate", vec![("degrees", degrees), ("child", child)])
        }
        GraphicsShapesOp::Scale => {
            let [sx, sy, child] = take3(args, "`graphics/shapes scale`")?;
            tagged("scale", vec![("sx", sx), ("sy", sy), ("child", child)])
        }
        GraphicsShapesOp::Opacity => {
            let [alpha, child] = take2(args, "`graphics/shapes opacity`")?;
            tagged("opacity", vec![("alpha", alpha), ("child", child)])
        }
        GraphicsShapesOp::Fill => {
            let [shape, color] = take2(args, "`graphics/shapes fill`")?;
            tagged("fill", vec![("shape", shape), ("color", color)])
        }
        GraphicsShapesOp::Stroke => {
            let [shape, width, color] = take3(args, "`graphics/shapes stroke`")?;
            tagged(
                "stroke",
                vec![("shape", shape), ("width", width), ("color", color)],
            )
        }
        GraphicsShapesOp::Paint => {
            let [shape, fill_color, stroke_width, stroke_color] =
                take4(args, "`graphics/shapes paint`")?;
            Ok(record(vec![
                ("tag".into(), tag_value("paint")),
                ("shape".into(), shape.clone()),
                ("fill".into(), fill_color.clone()),
                ("stroke-width".into(), stroke_width.clone()),
                ("stroke-color".into(), stroke_color.clone()),
            ]))
        }
    }
}

fn rgb_tag_record(
    tag: &str,
    r: &RuntimeValue,
    g: &RuntimeValue,
    b: &RuntimeValue,
) -> Result<RuntimeValue, EvalError> {
    Ok(record(vec![
        ("tag".into(), str_val(tag)),
        ("r".into(), r.clone()),
        ("g".into(), g.clone()),
        ("b".into(), b.clone()),
    ]))
}

fn rgba_tag_record(
    r: &RuntimeValue,
    g: &RuntimeValue,
    b: &RuntimeValue,
    a: &RuntimeValue,
) -> Result<RuntimeValue, EvalError> {
    Ok(record(vec![
        ("tag".into(), str_val("rgba")),
        ("r".into(), r.clone()),
        ("g".into(), g.clone()),
        ("b".into(), b.clone()),
        ("a".into(), a.clone()),
    ]))
}

fn constant_rgb(r: i128, g: i128, b: i128) -> Result<RuntimeValue, EvalError> {
    Ok(record(vec![
        ("tag".into(), str_val("rgb")),
        ("r".into(), int_val(r)),
        ("g".into(), int_val(g)),
        ("b".into(), int_val(b)),
    ]))
}

fn size_record(width: i128, height: i128) -> Result<RuntimeValue, EvalError> {
    Ok(record(vec![
        ("width".into(), int_val(width)),
        ("height".into(), int_val(height)),
    ]))
}

fn size_record_f64(width: f64, height: f64) -> Result<RuntimeValue, EvalError> {
    Ok(record(vec![
        ("width".into(), num_val(width)),
        ("height".into(), num_val(height)),
    ]))
}

fn tagged(tag: &str, fields: Vec<(&str, &RuntimeValue)>) -> Result<RuntimeValue, EvalError> {
    let mut out = vec![("tag".into(), tag_value(tag))];
    for (k, v) in fields {
        out.push((k.into(), v.clone()));
    }
    Ok(record(out))
}

fn tag_value(tag: &str) -> RuntimeValue {
    match tag {
        "circle" | "rect" | "text" => RuntimeValue::ShapeTag(tag.into()),
        other => RuntimeValue::String(other.into()),
    }
}

fn record(fields: Vec<(String, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(fields)
}

fn str_val(s: &str) -> RuntimeValue {
    RuntimeValue::String(s.into())
}

fn int_val(n: i128) -> RuntimeValue {
    RuntimeValue::Int(n)
}

fn num_val(n: f64) -> RuntimeValue {
    RuntimeValue::Number(n)
}

fn as_f64(v: &RuntimeValue, ctx: &str) -> Result<f64, EvalError> {
    match v {
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Ok(*n),
        RuntimeValue::Int(n) => Ok(*n as f64),
        other => Err(EvalError {
            message: format!("{ctx}: expected number, got {other}"),
        }),
    }
}

fn field_value(rec: &RuntimeValue, key: &str) -> Result<RuntimeValue, EvalError> {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| EvalError {
                message: format!("record missing field `{key}`"),
            }),
        other => Err(EvalError {
            message: format!("expected record, got {other}"),
        }),
    }
}
