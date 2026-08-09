//! Phase 14 conformance: Reference Client standard packages (`rpx.std.*`).

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::Shape;
use reciplexa_std::motion::{
    keyframe_track, DurationMs, Easing, TemporalPlacement, TimeMs, Timeline, TimelineTrack,
};
use reciplexa_std::{
    Block, BooleanOp, BooleanPathOp, Canvas, Color, Column, Ellipse, Figure, Fill, Font, Gradient,
    GradientStop, Group, Heading, Image, Length, Line, List, MathAtom, MathClass, Opacity, Padding,
    Page, Path, PathCommand, Point, Range, Rect, Rectangle, Row, Size, Slide, Span, Stack, Stroke,
    Style, Text, TextBox, TextStyle, Theme, Transform, Transition, VectorPath, VisualNode,
};
use reciplexa_test::{run_conformance, ConformanceCase};

fn nid(n: u64) -> StableNodeId {
    StableNodeId::new(n)
}

#[test]
fn core_units_and_range() {
    let case = ConformanceCase::new("TEST-STD-001", "Phase 14", "core units");
    run_conformance(&case, || {
        let r = Rect::from_xywh(0.0, 0.0, 10.0, 20.0);
        assert!(r.is_drawable());
        assert!(r.contains_point(Point::mm(5.0, 10.0)));
        let range = Range::new(Length::mm(0.0), Length::mm(10.0));
        assert!(range.contains(&Length::mm(5.0)));
        assert_eq!(Color::named("black"), Some(Color::BLACK));
    });
}

#[test]
fn visual_scene_bridge_roundtrip_rect() {
    let case = ConformanceCase::new("TEST-STD-002", "Phase 14", "visual->scene");
    run_conformance(&case, || {
        let mut canvas = Canvas::a4(nid(1));
        canvas.push(VisualNode::Rectangle(Rectangle::new(
            nid(2),
            Rect::from_xywh(10.0, 20.0, 30.0, 40.0),
            Style::filled(Color::RED),
        )));
        canvas.push(VisualNode::Ellipse(Ellipse::new(
            nid(3),
            Point::mm(50.0, 50.0),
            Size::mm(10.0, 10.0),
            Style::filled(Color::BLUE),
        )));
        canvas.push(VisualNode::Line(Line::new(
            nid(4),
            Point::mm(0.0, 0.0),
            Point::mm(1.0, 1.0),
            Stroke::new(Color::BLACK, Length::mm(0.5)),
        )));
        let shapes = canvas.to_scene_shapes();
        assert_eq!(shapes.len(), 3);
        assert!(matches!(shapes[0], Shape::Rect(_)));
        assert!(matches!(shapes[1], Shape::Circle(_)));
        assert!(matches!(shapes[2], Shape::Line(_)));
        let back = reciplexa_std::rectangle_from_scene(
            nid(9),
            match &shapes[0] {
                Shape::Rect(r) => r,
                _ => panic!("rect"),
            },
        );
        assert_eq!(back.style.fill, Fill::Solid(Color::RED));
    });
}

#[test]
fn text_layout_document_compose() {
    let case = ConformanceCase::new("TEST-STD-003", "Phase 14", "text layout document");
    run_conformance(&case, || {
        let style = TextStyle::body();
        assert!(Font::default().is_named());
        let text = Text::new(nid(1), Point::mm(0.0, 0.0), "hello", style.clone());
        assert!(matches!(text.to_scene_shape(), Some(Shape::Text(_))));
        let box_ = TextBox::new(nid(2), Rect::from_xywh(0.0, 0.0, 80.0, 30.0), style.clone())
            .with_plain("box");
        assert!(box_.to_scene_shape().is_some());
        let mut col = Column::new(nid(3));
        col.stack_mut().padding = Padding::all(Length::mm(2.0));
        let _ = Span::new(nid(4), "s", style);
        let mut page = Page::a4(nid(10));
        let mut section =
            reciplexa_std::Section::new(nid(11)).with_heading(Heading::new(nid(12), 1, "Intro"));
        section.push(Block::Figure(
            Figure::new(nid(13), nid(2)).with_caption("fig"),
        ));
        page.flow.push_section(section);
        assert!(!page.flow.is_empty());
        let _ = List::ordered(nid(14));
        let _ = Stack::new(nid(15), reciplexa_std::Axis::Horizontal);
        let _ = Row::new(nid(16));
    });
}

#[test]
fn math_slide_vector_motion() {
    let case = ConformanceCase::new("TEST-STD-004", "Phase 14", "math slide vector motion");
    run_conformance(&case, || {
        let atom = MathAtom::fraction(
            nid(1),
            MathAtom::symbol(nid(2), "a", MathClass::Ordinary),
            MathAtom::symbol(nid(3), "b", MathClass::Ordinary),
        );
        assert_eq!(atom.linearize(), "(a/b)");
        let theme = Theme::light(nid(4), "Default");
        let mut slide = Slide::new(nid(5), nid(6), theme.id).with_transition(Transition::Fade);
        slide.push_block(Block::Heading(Heading::new(nid(7), 1, "Title")));
        assert_eq!(slide.transition, Transition::Fade);
        let path = VectorPath::new(nid(8))
            .move_to(Point::ORIGIN)
            .line_to(Point::mm(1.0, 0.0))
            .close();
        assert!(path.commands.iter().any(PathCommand::is_close));
        let g = Gradient::Linear {
            id: nid(9),
            start: Point::ORIGIN,
            end: Point::mm(1.0, 0.0),
            stops: vec![
                GradientStop::new(0.0, Color::BLACK).unwrap(),
                GradientStop::new(1.0, Color::WHITE).unwrap(),
            ],
        };
        assert!(g.is_usable());
        let _ = BooleanPathOp::new(nid(10), BooleanOp::Union, nid(8), nid(8));
        let mut tl = Timeline::new(nid(11), DurationMs(500));
        tl.push_track(TimelineTrack {
            id: 1,
            node: Some(nid(2)),
            name: "x".into(),
            placement: TemporalPlacement::span(TimeMs(0), TimeMs(500)),
            track: keyframe_track(vec![(TimeMs(0), 0.0), (TimeMs(500), 1.0)], Easing::Linear),
        });
        tl.play();
        tl.tick(100);
        assert!(tl.is_playing() || tl.inner.playhead.0 > 0);
        let _ = Group::new(nid(12));
        let _ = Image::new(nid(13), "x.png", Rect::from_xywh(0.0, 0.0, 1.0, 1.0));
        let _ = Path::open(
            nid(14),
            vec![Point::mm(0.0, 0.0), Point::mm(1.0, 1.0)],
            Stroke::new(Color::BLACK, Length::mm(1.0)),
        );
        let _ = Opacity::OPAQUE;
        let _ = Transform::IDENTITY;
        let _ = Style::filled(Color::BLACK);
    });
}
