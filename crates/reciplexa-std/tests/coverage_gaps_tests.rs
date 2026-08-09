//! Extra boundary/partition cases to push reciplexa-std region coverage ≥99%.

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::Shape;
use reciplexa_std::motion::{keyframe_track, DurationMs, Easing, TimeMs, Timeline};
use reciplexa_std::*;

fn id(n: u64) -> StableNodeId {
    StableNodeId::new(n)
}

#[test]
fn core_rect_new_and_point_size_ctors() {
    let r = Rect::new(Point::new(Length::mm(1.0), Length::mm(2.0)), Size::new(Length::mm(3.0), Length::mm(4.0)));
    assert_eq!(r.width().as_mm(), 3.0);
    assert_eq!(r.height().as_mm(), 4.0);
    assert!(!Point::mm(f64::NAN, 0.0).is_finite());
    assert_eq!(Angle::ZERO.as_degrees(), 0.0);
    assert_eq!(Length::ZERO.to_string(), "0mm");
}

#[test]
fn style_display_with_and_without_stroke() {
    let plain = Style::filled(Color::BLACK);
    assert!(format!("{plain}").contains("Style"));
    let stroked = plain.with_stroke(Stroke::new(Color::RED, Length::mm(1.0)));
    assert!(format!("{stroked}").contains("stroke:"));
    assert!(!Style {
        fill: Fill::None,
        stroke: Some(Stroke::new(Color::BLACK, Length::ZERO)),
        opacity: Opacity::OPAQUE,
    }
    .is_drawable());
}

#[test]
fn visual_scene_partitions_and_bridges() {
    assert!(Transform::IDENTITY.is_identity());
    assert_eq!(Transform::default(), Transform::IDENTITY);
    let t = Transform::translate(Length::mm(10.0), Length::mm(20.0));
    assert_eq!(t.to_affine().e, 10.0);
    assert!(t.to_string().contains("T("));

    let mut c = Canvas::a4(id(1));
    c.push(VisualNode::Rectangle(Rectangle::new(
        id(2),
        Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
        Style::filled(Color::BLACK),
    )));
    assert_eq!(c.to_scene_shapes().len(), 1);

    let frame = Rect::from_xywh(1.0, 2.0, 3.0, 4.0);
    assert!(matches!(
        Rectangle::new(id(1), frame, Style::filled(Color::RED)).to_scene_shape(),
        Some(Shape::Rect(_))
    ));
    assert!(matches!(
        Rectangle::new(
            id(1),
            frame,
            Style::stroked(Stroke::new(Color::BLACK, Length::mm(0.5)))
        )
        .to_scene_shape(),
        Some(Shape::Frame(_))
    ));
    assert!(matches!(
        Rectangle::new(
            id(1),
            frame,
            Style::filled(Color::BLUE).with_stroke(Stroke::new(Color::BLACK, Length::mm(1.0)))
        )
        .to_scene_shape(),
        Some(Shape::Rect(_))
    ));
    // Solid fill + undrawable stroke hits rectangle match fallback
    assert!(Rectangle::new(
        id(1),
        frame,
        Style {
            fill: Fill::Solid(Color::RED),
            stroke: Some(Stroke::new(Color::BLACK, Length::ZERO)),
            opacity: Opacity::OPAQUE,
        },
    )
    .to_scene_shape()
    .is_none());

    let style = Style::filled(Color::BLUE);
    assert!(matches!(
        Ellipse::new(id(1), Point::ORIGIN, Size::mm(5.0, 5.0), style).to_scene_shape(),
        Some(Shape::Circle(_))
    ));
    assert!(matches!(
        Ellipse::new(id(1), Point::ORIGIN, Size::mm(5.0, 3.0), style).to_scene_shape(),
        Some(Shape::Ellipse(_))
    ));
    assert!(matches!(
        Ellipse::new(
            id(1),
            Point::ORIGIN,
            Size::mm(5.0, 5.0),
            Style::stroked(Stroke::new(Color::BLACK, Length::mm(1.0)))
        )
        .to_scene_shape(),
        Some(Shape::Ring(_))
    ));

    let stroke = Stroke::new(Color::BLACK, Length::mm(1.0));
    assert!(matches!(
        Line::new(id(1), Point::mm(0.0, 0.0), Point::mm(1.0, 1.0), stroke).to_scene_shape(),
        Some(Shape::Line(_))
    ));
    assert!(Line::new(id(1), Point::ORIGIN, Point::ORIGIN, stroke)
        .to_scene_shape()
        .is_none());
    assert!(matches!(
        Path::open(
            id(2),
            vec![Point::mm(0.0, 0.0), Point::mm(1.0, 0.0)],
            stroke
        )
        .to_scene_shape(),
        Some(Shape::Polyline(_))
    ));
    assert!(matches!(
        Path::closed(
            id(3),
            vec![Point::mm(0.0, 0.0), Point::mm(1.0, 0.0), Point::mm(0.0, 1.0)],
            Color::RED
        )
        .to_scene_shape(),
        Some(Shape::Polygon(_))
    ));
    // Invalid solid fill on closed path
    let mut bad_fill = Path::closed(
        id(3),
        vec![Point::mm(0.0, 0.0), Point::mm(1.0, 0.0), Point::mm(0.0, 1.0)],
        Color::RED,
    );
    bad_fill.style.fill = Fill::Solid(Color::new(2.0, 0.0, 0.0));
    assert!(bad_fill.to_scene_shape().is_none());

    assert!(matches!(
        Image::new(id(4), "a.png", Rect::from_xywh(0.0, 0.0, 10.0, 10.0)).to_scene_shape(),
        Some(Shape::Image(_))
    ));
    let mut g = Group::new(id(5));
    g.children.push(VisualNode::Line(Line::new(
        id(1),
        Point::mm(0.0, 0.0),
        Point::mm(1.0, 1.0),
        stroke,
    )));
    assert!(matches!(g.to_scene_shape(), Some(Shape::Group { .. })));
    g.opacity = Opacity::new(0.5);
    assert!(matches!(g.to_scene_shape(), Some(Shape::Opacity { .. })));
    assert_eq!(scene_text_box(0.0, 0.0, 12.0, "hi", Color::BLACK).content, "hi");
    let back = rectangle_from_scene(
        id(9),
        &reciplexa_scene::Rect {
            x_mm: 0.0,
            y_mm: 0.0,
            width_mm: 5.0,
            height_mm: 5.0,
            fill: Color::GREEN.to_scene(),
        },
    );
    assert_eq!(back.style.fill, Fill::Solid(Color::GREEN));
}

#[test]
fn visual_all_node_ids_and_error_arms() {
    let frame = Rect::from_xywh(0.0, 0.0, 10.0, 10.0);
    let rect = VisualNode::Rectangle(Rectangle::new(id(1), frame, Style::filled(Color::RED)));
    let ell = VisualNode::Ellipse(Ellipse::new(
        id(2),
        Point::mm(0.0, 0.0),
        Size::mm(4.0, 2.0),
        Style::filled(Color::BLUE),
    ));
    let line = VisualNode::Line(Line::new(
        id(3),
        Point::mm(0.0, 0.0),
        Point::mm(1.0, 0.0),
        Stroke::new(Color::BLACK, Length::mm(1.0)),
    ));
    let path = VisualNode::Path(Path::open(
        id(4),
        vec![Point::mm(0.0, 0.0), Point::mm(1.0, 1.0)],
        Stroke::new(Color::BLACK, Length::mm(1.0)),
    ));
    let img = VisualNode::Image(Image::new(id(5), "p.png", frame));
    let group = VisualNode::Group(Group::new(id(6)));
    for (n, expect) in [
        (&rect, id(1)),
        (&ell, id(2)),
        (&line, id(3)),
        (&path, id(4)),
        (&img, id(5)),
        (&group, id(6)),
    ] {
        assert_eq!(n.id(), expect);
        let _ = n.to_scene_shape();
    }

    // Rectangle catch-all: no fill, undrawable stroke
    assert!(Rectangle::new(
        id(7),
        frame,
        Style {
            fill: Fill::None,
            stroke: Some(Stroke::new(Color::BLACK, Length::ZERO)),
            opacity: Opacity::OPAQUE,
        },
    )
    .to_scene_shape()
    .is_none());

    // Ellipse catch-all: stroked non-circle
    assert!(Ellipse::new(
        id(8),
        Point::ORIGIN,
        Size::mm(4.0, 2.0),
        Style::stroked(Stroke::new(Color::BLACK, Length::mm(1.0))),
    )
    .to_scene_shape()
    .is_none());

    // Ellipse not drawable style
    assert!(Ellipse::new(
        id(8),
        Point::ORIGIN,
        Size::mm(4.0, 4.0),
        Style {
            fill: Fill::None,
            stroke: None,
            opacity: Opacity::OPAQUE,
        },
    )
    .to_scene_shape()
    .is_none());

    // Line undrawable stroke
    assert!(Line::new(
        id(9),
        Point::mm(0.0, 0.0),
        Point::mm(1.0, 1.0),
        Stroke::new(Color::BLACK, Length::ZERO),
    )
    .to_scene_shape()
    .is_none());

    // Closed path with invisible fill
    let mut closed = Path::closed(id(10), vec![Point::mm(0.0, 0.0), Point::mm(1.0, 0.0), Point::mm(0.0, 1.0)], Color::RED);
    closed.style.fill = Fill::None;
    assert!(closed.to_scene_shape().is_none());

    // Open path missing stroke / bad stroke
    let mut open = Path::open(id(11), vec![Point::mm(0.0, 0.0), Point::mm(1.0, 0.0)], Stroke::new(Color::BLACK, Length::mm(1.0)));
    open.style.stroke = None;
    assert!(open.to_scene_shape().is_none());
    open.style.stroke = Some(Stroke::new(Color::BLACK, Length::ZERO));
    assert!(open.to_scene_shape().is_none());

    // Image undrawable frame
    assert!(Image::new(id(12), "x.png", Rect::from_xywh(0.0, 0.0, 0.0, 1.0))
        .to_scene_shape()
        .is_none());

    // Group invalid opacity
    let mut g = Group::new(id(13));
    g.opacity = Opacity::new(2.0);
    assert!(g.to_scene_shape().is_none());

    assert!(matches!(
        Transform::translate(Length::mm(1.0), Length::mm(2.0)).to_affine().e,
        1.0
    ));
}

#[test]
fn text_font_display_and_undrawable_style() {
    let font = Font::new("A").with_fallback("B").with_fallback("C");
    assert_eq!(format!("{font}"), "A,B,C");
    let bad = TextStyle {
        font: Font::new(""),
        size: Length::mm(4.0),
        color: Color::BLACK,
        bold: false,
        italic: false,
    };
    assert!(!bad.is_drawable());
    assert!(Text::new(id(1), Point::ORIGIN, "x", bad.clone())
        .to_scene_shape()
        .is_none());
    assert!(TextBox::new(id(2), Rect::from_xywh(0.0, 0.0, 10.0, 10.0), bad)
        .with_plain("x")
        .to_scene_shape()
        .is_none());
    let para = Paragraph {
        id: id(3),
        spans: vec![Span::new(id(4), "", TextStyle::body())],
    };
    assert!(para.is_empty());
}

#[test]
fn math_linearize_all_script_shapes_and_ids() {
    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let b = MathAtom::symbol(id(2), "b", MathClass::Ordinary);
    let two = MathAtom::symbol(id(3), "2", MathClass::Ordinary);
    let row = MathAtom::row(id(10), vec![a.clone(), b.clone()]);
    assert_eq!(row.id(), id(10));
    let sqrt = MathAtom::radical(id(5), a.clone());
    assert_eq!(sqrt.linearize(), "sqrt{a}");
    let only_base = MathAtom::scripts(id(20), a.clone(), None, None);
    assert_eq!(only_base.linearize(), "a");
    assert_eq!(only_base.child_count(), 1);
    let sup = MathAtom::scripts(id(21), a.clone(), Some(two.clone()), None);
    assert_eq!(sup.linearize(), "a^2");
    let sub = MathAtom::scripts(id(22), a.clone(), None, Some(b.clone()));
    assert_eq!(sub.linearize(), "a_b");
    let _ = format!("{}", MathClass::Fence);
}

#[test]
fn slide_transition_none_and_vector_radial_id() {
    assert_eq!(Transition::None.as_str(), "none");
    assert_eq!(format!("{}", Transition::None), "none");
    let rad = Gradient::Radial {
        id: id(2),
        center: Point::ORIGIN,
        radius: Length::mm(1.0),
        stops: vec![],
    };
    assert_eq!(rad.id(), id(2));
    assert!(!rad.is_usable());
    let _ = StrokeJoin::Miter;
    let _ = StrokeCap::Butt;
    let path = VectorPath::new(id(1)).move_to(Point::ORIGIN);
    assert_eq!(path.endpoint_count(), 1);
    assert!(!PathCommand::LineTo(Point::ORIGIN).is_close());
}

#[test]
fn layout_align_stretch_and_document_blocks() {
    assert_eq!(Align::Stretch.to_string(), "stretch");
    let stack = Stack::new(id(1), Axis::Horizontal)
        .with_gap(Length::mm(1.0))
        .with_align(Align::End);
    assert_eq!(stack.align, Align::End);
    assert_eq!(Block::Spacer(Length::mm(1.0)).id(), None);
    let h = Heading::new(id(2), 2, "x");
    assert!(format!("{h}").contains("h2:"));
}

#[test]
fn motion_tick_while_paused_and_keyframe_helper() {
    let mut tl = Timeline::new(id(1), DurationMs(100));
    tl.tick(50);
    assert_eq!(tl.inner.playhead, TimeMs::ZERO);
    let track = keyframe_track(vec![(TimeMs(0), 0.0)], Easing::EaseInQuad);
    assert!(matches!(track, MotionTrack::Keyframes(_)));
}
