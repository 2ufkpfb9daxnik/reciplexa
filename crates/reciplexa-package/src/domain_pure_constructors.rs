//! Direct Native v2 typed exports for pure constructors (DN2-2).

use std::collections::BTreeMap;

use reciplexa_core::ty::{CoreType, EffectRow};
use reciplexa_eval::domain_native::{
    ColorSrgbOp, DomainNativeOp, GraphicsColorOp, GraphicsPageOp, GraphicsShapesOp,
};

use crate::domain_native::{DomainNativeExport, DomainNativeModule};

fn dynamic() -> CoreType {
    CoreType::Dynamic(Box::new(CoreType::Any))
}

fn nullary_dynamic() -> CoreType {
    CoreType::Fun {
        args: vec![],
        ret: Box::new(dynamic()),
        effects: EffectRow::default(),
    }
}

fn fun_n(n: usize) -> CoreType {
    CoreType::Fun {
        args: vec![dynamic(); n],
        ret: Box::new(dynamic()),
        effects: EffectRow::default(),
    }
}

fn insert(
    exports: &mut BTreeMap<String, DomainNativeExport>,
    name: &str,
    ty: CoreType,
    op: DomainNativeOp,
) {
    exports.insert(name.into(), DomainNativeExport::new(name, ty, op));
}

/// Attach typed DN2 exports for `color/srgb`.
pub fn populate_color_srgb_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::ColorSrgb(sub);
    insert(&mut exports, "srgb", fun_n(3), op(ColorSrgbOp::Srgb));
    insert(&mut exports, "rgb", fun_n(3), op(ColorSrgbOp::Rgb));
    insert(&mut exports, "rgba", fun_n(4), op(ColorSrgbOp::Rgba));
    insert(
        &mut exports,
        "from-byte",
        fun_n(3),
        op(ColorSrgbOp::FromByte),
    );
    insert(&mut exports, "gray", fun_n(1), op(ColorSrgbOp::Gray));
    insert(
        &mut exports,
        "black",
        nullary_dynamic(),
        op(ColorSrgbOp::Black),
    );
    insert(
        &mut exports,
        "white",
        nullary_dynamic(),
        op(ColorSrgbOp::White),
    );
    insert(&mut exports, "red", nullary_dynamic(), op(ColorSrgbOp::Red));
    insert(
        &mut exports,
        "green",
        nullary_dynamic(),
        op(ColorSrgbOp::Green),
    );
    insert(
        &mut exports,
        "blue",
        nullary_dynamic(),
        op(ColorSrgbOp::Blue),
    );
    insert(
        &mut exports,
        "yellow",
        nullary_dynamic(),
        op(ColorSrgbOp::Yellow),
    );
    insert(
        &mut exports,
        "cyan",
        nullary_dynamic(),
        op(ColorSrgbOp::Cyan),
    );
    insert(
        &mut exports,
        "magenta",
        nullary_dynamic(),
        op(ColorSrgbOp::Magenta),
    );
    insert(
        &mut exports,
        "orange",
        nullary_dynamic(),
        op(ColorSrgbOp::Orange),
    );
    insert(
        &mut exports,
        "gray50",
        nullary_dynamic(),
        op(ColorSrgbOp::Gray50),
    );
    insert(
        &mut exports,
        "transparent",
        nullary_dynamic(),
        op(ColorSrgbOp::Transparent),
    );
    insert(
        &mut exports,
        "with-alpha",
        fun_n(2),
        op(ColorSrgbOp::WithAlpha),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `graphics/color`.
pub fn populate_graphics_color_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::GraphicsColor(sub);
    insert(&mut exports, "rgb", fun_n(3), op(GraphicsColorOp::Rgb));
    insert(&mut exports, "rgba", fun_n(4), op(GraphicsColorOp::Rgba));
    insert(
        &mut exports,
        "black",
        nullary_dynamic(),
        op(GraphicsColorOp::Black),
    );
    insert(
        &mut exports,
        "white",
        nullary_dynamic(),
        op(GraphicsColorOp::White),
    );
    insert(
        &mut exports,
        "red",
        nullary_dynamic(),
        op(GraphicsColorOp::Red),
    );
    insert(
        &mut exports,
        "green",
        nullary_dynamic(),
        op(GraphicsColorOp::Green),
    );
    insert(
        &mut exports,
        "blue",
        nullary_dynamic(),
        op(GraphicsColorOp::Blue),
    );
    insert(
        &mut exports,
        "gray",
        nullary_dynamic(),
        op(GraphicsColorOp::Gray),
    );
    insert(
        &mut exports,
        "transparent",
        nullary_dynamic(),
        op(GraphicsColorOp::Transparent),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `graphics/page`.
pub fn populate_graphics_page_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::GraphicsPage(sub);
    insert(
        &mut exports,
        "a4",
        nullary_dynamic(),
        op(GraphicsPageOp::A4),
    );
    insert(
        &mut exports,
        "letter",
        nullary_dynamic(),
        op(GraphicsPageOp::Letter),
    );
    insert(
        &mut exports,
        "a5",
        nullary_dynamic(),
        op(GraphicsPageOp::A5),
    );
    insert(
        &mut exports,
        "a3",
        nullary_dynamic(),
        op(GraphicsPageOp::A3),
    );
    insert(
        &mut exports,
        "legal",
        nullary_dynamic(),
        op(GraphicsPageOp::Legal),
    );
    insert(
        &mut exports,
        "square",
        nullary_dynamic(),
        op(GraphicsPageOp::Square),
    );
    insert(&mut exports, "page", fun_n(2), op(GraphicsPageOp::Page));
    insert(&mut exports, "pages", fun_n(1), op(GraphicsPageOp::Pages));
    insert(
        &mut exports,
        "page-size",
        fun_n(2),
        op(GraphicsPageOp::PageSize),
    );
    module.typed_exports = exports;
}

/// Attach typed DN2 exports for `graphics/shapes`.
pub fn populate_graphics_shapes_typed_exports(module: &mut DomainNativeModule) {
    let mut exports = BTreeMap::new();
    let op = |sub| DomainNativeOp::GraphicsShapes(sub);
    insert(
        &mut exports,
        "circle",
        fun_n(3),
        op(GraphicsShapesOp::Circle),
    );
    insert(&mut exports, "rect", fun_n(4), op(GraphicsShapesOp::Rect));
    insert(
        &mut exports,
        "ellipse",
        fun_n(4),
        op(GraphicsShapesOp::Ellipse),
    );
    insert(&mut exports, "line", fun_n(4), op(GraphicsShapesOp::Line));
    insert(&mut exports, "path", fun_n(2), op(GraphicsShapesOp::Path));
    insert(
        &mut exports,
        "polyline",
        fun_n(1),
        op(GraphicsShapesOp::Polyline),
    );
    insert(
        &mut exports,
        "polygon",
        fun_n(1),
        op(GraphicsShapesOp::Polygon),
    );
    insert(&mut exports, "ring", fun_n(4), op(GraphicsShapesOp::Ring));
    insert(&mut exports, "frame", fun_n(6), op(GraphicsShapesOp::Frame));
    insert(&mut exports, "group", fun_n(1), op(GraphicsShapesOp::Group));
    insert(&mut exports, "text", fun_n(4), op(GraphicsShapesOp::Text));
    insert(
        &mut exports,
        "text-box",
        fun_n(6),
        op(GraphicsShapesOp::TextBox),
    );
    insert(&mut exports, "image", fun_n(5), op(GraphicsShapesOp::Image));
    insert(
        &mut exports,
        "translate",
        fun_n(3),
        op(GraphicsShapesOp::Translate),
    );
    insert(
        &mut exports,
        "rotate",
        fun_n(2),
        op(GraphicsShapesOp::Rotate),
    );
    insert(&mut exports, "scale", fun_n(3), op(GraphicsShapesOp::Scale));
    insert(
        &mut exports,
        "opacity",
        fun_n(2),
        op(GraphicsShapesOp::Opacity),
    );
    insert(&mut exports, "fill", fun_n(2), op(GraphicsShapesOp::Fill));
    insert(
        &mut exports,
        "stroke",
        fun_n(3),
        op(GraphicsShapesOp::Stroke),
    );
    insert(&mut exports, "paint", fun_n(4), op(GraphicsShapesOp::Paint));
    module.typed_exports = exports;
}
