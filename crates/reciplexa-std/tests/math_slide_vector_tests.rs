//! Integration coverage for math / slide / vector (moved out of `src` cfg(test)).

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::document::{Block, Heading};
use reciplexa_std::math::{MathAtom, MathClass};
use reciplexa_std::slide::{Master, Notes, Placeholder, PlaceholderKind, Slide, Theme, Transition};
use reciplexa_std::style::Stroke;
use reciplexa_std::vector::{
    BooleanOp, BooleanPathOp, Gradient, GradientStop, PathCommand, StrokeCap, StrokeJoin,
    VectorPath, VectorStroke,
};
use reciplexa_std::{Color, Length, Point, Size};

fn id(n: u64) -> StableNodeId {
    StableNodeId::new(n)
}

#[test]
fn math_class_and_tree() {
    assert_eq!(MathClass::Ordinary.as_str(), "ord");
    assert_eq!(MathClass::Operator.to_string(), "op");
    assert_eq!(MathClass::Binary.as_str(), "bin");
    assert_eq!(MathClass::Relation.as_str(), "rel");
    assert_eq!(MathClass::Open.as_str(), "open");
    assert_eq!(MathClass::Close.as_str(), "close");
    assert_eq!(MathClass::Punctuation.as_str(), "punct");
    assert_eq!(MathClass::Fence.as_str(), "fence");

    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let b = MathAtom::symbol(id(2), "b", MathClass::Ordinary);
    let two = MathAtom::symbol(id(3), "2", MathClass::Ordinary);
    let frac = MathAtom::fraction(id(4), a.clone(), b.clone());
    assert_eq!(frac.child_count(), 2);
    assert_eq!(frac.linearize(), "(a/b)");
    let sqrt = MathAtom::radical(id(5), frac.clone());
    assert_eq!(sqrt.child_count(), 1);
    let cbrt = MathAtom::radical_indexed(id(6), two.clone(), a.clone());
    assert_eq!(cbrt.child_count(), 2);
    assert!(cbrt.linearize().contains("root"));
    let scripts = MathAtom::scripts(id(7), a.clone(), Some(two.clone()), Some(b.clone()));
    assert_eq!(scripts.child_count(), 3);
    assert_eq!(scripts.to_string(), "a^2_b");
    let scripts_sup = MathAtom::scripts(id(8), a.clone(), Some(two.clone()), None);
    assert_eq!(scripts_sup.child_count(), 2);
    let scripts_sub = MathAtom::scripts(id(9), a.clone(), None, Some(b.clone()));
    assert_eq!(scripts_sub.child_count(), 2);
    let row = MathAtom::row(id(10), vec![a.clone(), b.clone()]);
    assert_eq!(row.child_count(), 2);
    assert_eq!(row.linearize(), "ab");
    let delim = MathAtom::delimiter(id(11), "(", ")", row);
    assert_eq!(delim.child_count(), 1);
    assert_eq!(delim.linearize(), "(ab)");
    assert_eq!(a.child_count(), 0);
    assert_eq!(a.id(), id(1));
    assert_eq!(frac.id(), id(4));
    assert_eq!(sqrt.id(), id(5));
    assert_eq!(cbrt.id(), id(6));
    assert_eq!(scripts.id(), id(7));
    assert_eq!(delim.id(), id(11));
}

#[test]
fn math_accent_and_big_op() {
    use reciplexa_std::math::MathAccentKind;
    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let n = MathAtom::symbol(id(2), "n", MathClass::Ordinary);
    let hat = MathAtom::accent(id(3), MathAccentKind::Hat, a.clone());
    assert_eq!(hat.child_count(), 1);
    assert_eq!(hat.linearize(), "hat{a}");
    assert_eq!(MathAccentKind::Vec.as_str(), "vec");
    let i = MathAtom::symbol(id(4), "i", MathClass::Ordinary);
    let sum = MathAtom::big_op(
        id(5),
        "∑",
        Some(i),
        Some(n),
        Some(a),
    );
    assert_eq!(sum.child_count(), 3);
    assert!(sum.linearize().contains('∑'));
    assert_eq!(MathAtom::paren(id(6), MathAtom::symbol(id(7), "x", MathClass::Ordinary)).linearize(), "(x)");
    assert_eq!(
        MathAtom::superscript(id(8), MathAtom::symbol(id(9), "x", MathClass::Ordinary), MathAtom::symbol(id(10), "2", MathClass::Ordinary))
            .linearize(),
        "x^2"
    );
}

#[test]
fn theme_master_slide() {
    assert!(Theme::light(id(1), "L").to_string().contains("L"));
    assert_eq!(Theme::dark(id(2), "D").background, Color::BLACK);
    assert_eq!(PlaceholderKind::Title.as_str(), "title");
    assert_eq!(PlaceholderKind::Body.to_string(), "body");
    assert_eq!(PlaceholderKind::Figure.as_str(), "figure");
    assert_eq!(PlaceholderKind::Footer.as_str(), "footer");
    assert_eq!(PlaceholderKind::SlideNumber.as_str(), "slide-number");
    let mut master = Master::widescreen(id(3), "Title");
    master.push_placeholder(Placeholder::new(
        id(4),
        PlaceholderKind::Title,
        Size::mm(200.0, 30.0),
    ));
    assert!(master.find(PlaceholderKind::Title).is_some());
    assert!(master.find(PlaceholderKind::Body).is_none());
    assert_eq!(Transition::default(), Transition::None);
    assert_eq!(Transition::Fade.as_str(), "fade");
    assert_eq!(Transition::Push.to_string(), "push");
    assert_eq!(Transition::Dissolve.as_str(), "dissolve");
    let notes = Notes::new(id(5), "say hi");
    assert!(!notes.is_empty());
    assert!(Notes::new(id(5), "").is_empty());
    let mut slide = Slide::new(id(6), id(3), id(1))
        .with_notes(notes)
        .with_transition(Transition::Fade);
    slide.push_block(Block::Heading(Heading::new(id(7), 1, "Hello")));
    assert_eq!(slide.blocks.len(), 1);
    assert_eq!(slide.transition, Transition::Fade);
}

#[test]
fn path_commands_and_stroke() {
    assert!(PathCommand::Close.is_close());
    assert!(!PathCommand::MoveTo(Point::ORIGIN).is_close());
    let path = VectorPath::new(id(1))
        .move_to(Point::mm(0.0, 0.0))
        .line_to(Point::mm(1.0, 0.0))
        .quad_to(Point::mm(1.0, 1.0), Point::mm(0.0, 1.0))
        .cubic_to(
            Point::mm(0.0, 0.5),
            Point::mm(-0.5, 0.5),
            Point::mm(0.0, 0.0),
        )
        .close();
    assert!(!path.is_empty());
    assert_eq!(path.endpoint_count(), 4);
    assert!(VectorPath::new(id(1)).is_empty());
    let stroke = VectorStroke::solid(Stroke::new(Color::BLACK, Length::mm(1.0)))
        .with_dash(vec![Length::mm(2.0), Length::mm(1.0)]);
    assert!(stroke.is_dashed());
    assert!(!VectorStroke::solid(Stroke::new(Color::BLACK, Length::mm(1.0))).is_dashed());
    let _ = StrokeJoin::Round;
    let _ = StrokeCap::Square;
    let _ = StrokeJoin::Bevel;
    let _ = StrokeCap::Round;
}

#[test]
fn gradient_and_boolean() {
    assert!(GradientStop::new(-0.1, Color::BLACK).is_none());
    assert!(GradientStop::new(1.1, Color::BLACK).is_none());
    assert!(GradientStop::new(0.5, Color::new(2.0, 0.0, 0.0)).is_none());
    let s0 = GradientStop::new(0.0, Color::BLACK).unwrap();
    let s1 = GradientStop::new(1.0, Color::WHITE).unwrap();
    let lin = Gradient::Linear {
        id: id(1),
        start: Point::ORIGIN,
        end: Point::mm(10.0, 0.0),
        stops: vec![s0, s1],
    };
    assert!(lin.is_usable());
    assert_eq!(lin.id(), id(1));
    let rad = Gradient::Radial {
        id: id(2),
        center: Point::ORIGIN,
        radius: Length::mm(5.0),
        stops: vec![s0],
    };
    assert!(!rad.is_usable());
    assert_eq!(rad.stop_count(), 1);
    assert_eq!(BooleanOp::Union.as_str(), "union");
    assert_eq!(BooleanOp::Intersection.to_string(), "intersection");
    assert_eq!(BooleanOp::Difference.as_str(), "difference");
    assert_eq!(BooleanOp::Xor.as_str(), "xor");
    let op = BooleanPathOp::new(id(3), BooleanOp::Xor, id(1), id(2));
    assert_eq!(op.op, BooleanOp::Xor);
}
