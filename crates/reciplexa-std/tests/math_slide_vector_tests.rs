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
    let sum = MathAtom::big_op(id(5), "∑", Some(i), Some(n), Some(a));
    assert_eq!(sum.child_count(), 3);
    assert!(sum.linearize().contains('∑'));
    assert_eq!(
        MathAtom::paren(id(6), MathAtom::symbol(id(7), "x", MathClass::Ordinary)).linearize(),
        "(x)"
    );
    assert_eq!(
        MathAtom::superscript(
            id(8),
            MathAtom::symbol(id(9), "x", MathClass::Ordinary),
            MathAtom::symbol(id(10), "2", MathClass::Ordinary)
        )
        .linearize(),
        "x^2"
    );
    // Multi-character script bodies get TeX-style braces (Display uses linearize).
    let twelve = MathAtom::row(
        id(11),
        vec![
            MathAtom::symbol(id(12), "1", MathClass::Ordinary),
            MathAtom::symbol(id(13), "2", MathClass::Ordinary),
        ],
    );
    assert_eq!(
        MathAtom::superscript(
            id(14),
            MathAtom::symbol(id(15), "x", MathClass::Ordinary),
            twelve
        )
        .linearize(),
        "x^{12}"
    );
    assert_eq!(
        format!(
            "{}",
            MathAtom::subscript(
                id(16),
                MathAtom::symbol(id(17), "a", MathClass::Ordinary),
                MathAtom::row(
                    id(18),
                    vec![
                        MathAtom::symbol(id(19), "i", MathClass::Ordinary),
                        MathAtom::symbol(id(20), "j", MathClass::Ordinary),
                    ]
                )
            )
        ),
        "a_{ij}"
    );
}

#[test]
fn math_accent_clearance_above_and_below_base() {
    use reciplexa_std::math::{
        underbrace_spacing, MathAccentKind, ACCENT_CLEARANCE_EM, UNDERBRACE_CLEARANCE_EM,
    };
    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let base = a.estimate_box();
    let hat = MathAtom::accent(id(2), MathAccentKind::Hat, a.clone()).estimate_box();
    assert!(
        (hat.height - (base.height + ACCENT_CLEARANCE_EM)).abs() < 1e-9,
        "hat should add clearance above base"
    );
    assert!((hat.depth - base.depth).abs() < 1e-9);

    let under = MathAtom::accent(id(3), MathAccentKind::Underline, a.clone()).estimate_box();
    assert!((under.height - base.height).abs() < 1e-9);
    assert!(
        (under.depth - (base.depth + UNDERBRACE_CLEARANCE_EM)).abs() < 1e-9,
        "underline/underbrace should add underbrace-style clearance below base"
    );
    assert!((underbrace_spacing() - UNDERBRACE_CLEARANCE_EM).abs() < 1e-9);

    let wide = MathAtom::accent(id(4), MathAccentKind::WideHat, a).estimate_box();
    assert!(wide.height > hat.height);
}

#[test]
fn math_stackrel_spacing_offsets() {
    use reciplexa_std::math::{
        stackrel_spacing_offsets, MathBox, MathStackKind, STACKREL_GAP_EM,
    };

    let upper = MathBox::new(0.8, 0.5, 0.1);
    let lower = MathBox::new(1.0, 0.7, 0.2);
    let (uy, ly) = stackrel_spacing_offsets(upper, lower);
    assert!((uy - (STACKREL_GAP_EM * 0.5 + upper.depth)).abs() < 1e-9);
    assert!((ly - (-(STACKREL_GAP_EM * 0.5 + lower.height))).abs() < 1e-9);
    assert!(uy > 0.0 && ly < 0.0);

    let rel = MathAtom::symbol(id(1), "=", MathClass::Relation);
    let base = MathAtom::symbol(id(2), "x", MathClass::Ordinary);
    let st = MathAtom::stack(id(3), MathStackKind::Stackrel, vec![rel.clone(), base.clone()]);
    let sb = st.estimate_box();
    let (euy, ely) = stackrel_spacing_offsets(rel.estimate_box(), base.estimate_box());
    assert!((sb.height - (euy + rel.estimate_box().height)).abs() < 1e-9);
    assert!((sb.depth - ((-ely) + base.estimate_box().depth)).abs() < 1e-9);
    assert!(sb.total_height() > base.estimate_box().total_height());
}

#[test]
fn math_fraction_rule_thickness_and_clearance() {
    use reciplexa_std::math::{
        fraction_rule_metrics, FRAC_DEN_CLEARANCE_EM, FRAC_NUM_CLEARANCE_EM, FRAC_RULE_THICKNESS_EM,
    };
    let (rule, num_clr, den_clr) = fraction_rule_metrics();
    assert!((rule - FRAC_RULE_THICKNESS_EM).abs() < 1e-9);
    assert!((num_clr - FRAC_NUM_CLEARANCE_EM).abs() < 1e-9);
    assert!((den_clr - FRAC_DEN_CLEARANCE_EM).abs() < 1e-9);

    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let b = MathAtom::symbol(id(2), "b", MathClass::Ordinary);
    let num = a.estimate_box();
    let den = b.estimate_box();
    let fb = MathAtom::fraction(id(3), a, b).estimate_box();
    assert!(
        (fb.height - (num.total_height() + FRAC_NUM_CLEARANCE_EM + FRAC_RULE_THICKNESS_EM)).abs()
            < 1e-9
    );
    assert!((fb.depth - (den.total_height() + FRAC_DEN_CLEARANCE_EM)).abs() < 1e-9);
}

#[test]
fn math_radical_vinculum_index_offsets() {
    use reciplexa_std::math::{
        radical_vinculum_index_offsets, MathBox, RADICAL_VINCULUM_CLEARANCE_EM,
        RADICAL_VINCULUM_THICKNESS_EM, SCRIPT_SCALE,
    };

    let body = MathBox::new(1.0, 0.7, 0.2);
    let (vy, ix, iy) = radical_vinculum_index_offsets(body, None);
    assert!((vy - (body.height + RADICAL_VINCULUM_CLEARANCE_EM)).abs() < 1e-9);
    assert_eq!((ix, iy), (0.0, 0.0));

    let idx = MathBox::new(0.5, 0.4, 0.1);
    let (vy2, ix2, iy2) = radical_vinculum_index_offsets(body, Some(idx));
    assert!((vy2 - vy).abs() < 1e-9);
    assert!(ix2 < 0.0, "index left of surd: {ix2}");
    assert!(iy2 > 0.0, "index raised: {iy2}");

    let plain = MathAtom::radical(id(10), MathAtom::symbol(id(11), "x", MathClass::Ordinary));
    let indexed = MathAtom::radical_indexed(
        id(12),
        MathAtom::symbol(id(13), "3", MathClass::Ordinary),
        MathAtom::symbol(id(14), "x", MathClass::Ordinary),
    );
    let pb = plain.estimate_box();
    let ib = indexed.estimate_box();
    assert!(pb.height >= 0.7 + RADICAL_VINCULUM_CLEARANCE_EM + RADICAL_VINCULUM_THICKNESS_EM - 1e-9);
    assert!(ib.width > pb.width);
    let _ = SCRIPT_SCALE;
}

#[test]
fn math_estimate_box_relative_sizes() {
    use reciplexa_std::math::MathBox;
    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let b = MathAtom::symbol(id(2), "b", MathClass::Ordinary);
    let sym = a.estimate_box();
    assert!(sym.width >= 0.5);
    assert!(sym.total_height() > 0.0);

    let frac = MathAtom::fraction(id(3), a.clone(), b.clone());
    let fb = frac.estimate_box();
    assert!(fb.total_height() > sym.total_height());
    assert!(fb.width >= sym.width);

    let scripts = MathAtom::scripts(
        id(4),
        a.clone(),
        Some(MathAtom::symbol(id(5), "2", MathClass::Ordinary)),
        None,
    );
    assert!(scripts.estimate_box().width > a.estimate_box().width);

    let empty = MathBox::new(0.0, 0.0, 0.0);
    assert_eq!(empty.total_height(), 0.0);
}

#[test]
fn estimate_style_text_shrinks_scripts_more_than_display() {
    use reciplexa_std::math::{EstimateStyle, SCRIPT_SCALE, SCRIPT_SCALE_TEXT};

    assert!((EstimateStyle::Display.script_scale() - SCRIPT_SCALE).abs() < 1e-9);
    assert!((EstimateStyle::Text.script_scale() - SCRIPT_SCALE_TEXT).abs() < 1e-9);
    assert!(SCRIPT_SCALE_TEXT < SCRIPT_SCALE);
    assert_eq!(EstimateStyle::parse("text"), Some(EstimateStyle::Text));
    assert_eq!(EstimateStyle::parse("display"), Some(EstimateStyle::Display));
    assert_eq!(EstimateStyle::parse("weird"), None);
    assert_eq!(EstimateStyle::Text.as_str(), "text");
    assert_eq!(EstimateStyle::Display.to_string(), "display");

    let base = MathAtom::symbol(id(1), "x", MathClass::Ordinary);
    let sup = MathAtom::symbol(id(2), "2", MathClass::Ordinary);
    let scripts = MathAtom::scripts(id(3), base, Some(sup), None);
    let display = scripts.estimate_box();
    let text = scripts.estimate_box_with_style(EstimateStyle::Text);
    assert_eq!(display, scripts.estimate_box_with_style(EstimateStyle::Display));
    assert!(text.width < display.width);
    assert!(text.height < display.height);
}

#[test]
fn class_spacing_em_texish_ord_op_and_row_width() {
    use reciplexa_std::math::{
        class_spacing_em, MED_MUSKIP_EM, THICK_MUSKIP_EM, THIN_MUSKIP_EM,
    };

    assert!((class_spacing_em(MathClass::Ordinary, MathClass::Operator) - THIN_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Operator, MathClass::Ordinary) - THIN_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Operator, MathClass::Operator) - THIN_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Ordinary, MathClass::Binary) - MED_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Operator, MathClass::Binary) - MED_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Ordinary, MathClass::Relation) - THICK_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Binary, MathClass::Relation) - THICK_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Ordinary, MathClass::Punctuation) - THIN_MUSKIP_EM).abs() < 1e-9);
    assert_eq!(class_spacing_em(MathClass::Ordinary, MathClass::Ordinary), 0.0);
    assert_eq!(class_spacing_em(MathClass::Open, MathClass::Ordinary), 0.0);
    assert_eq!(class_spacing_em(MathClass::Close, MathClass::Fence), 0.0);

    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let plus = MathAtom::symbol(id(2), "+", MathClass::Binary);
    let b = MathAtom::symbol(id(3), "b", MathClass::Ordinary);
    let row = MathAtom::row(id(4), vec![a.clone(), plus.clone(), b.clone()]);
    let expected = a.estimate_box().width
        + plus.estimate_box().width
        + b.estimate_box().width
        + MED_MUSKIP_EM
        + MED_MUSKIP_EM;
    assert!((row.estimate_box().width - expected).abs() < 1e-9);
}

#[test]
fn math_scripts_attachment_offsets_heuristic() {
    use reciplexa_std::math::{scripts_attachment_offsets, MathBox};

    let base = MathBox::new(1.0, 0.7, 0.2);
    let sub = MathBox::new(0.5, 0.4, 0.1);
    let sup = MathBox::new(0.5, 0.4, 0.1);
    let (sub_x, sub_y, sup_x, sup_y) =
        scripts_attachment_offsets(base, Some(sub), Some(sup));
    assert!((sub_x - 1.0).abs() < 1e-9);
    assert!((sup_x - 1.0).abs() < 1e-9);
    assert!(sub_y < 0.0, "subscript below baseline: {sub_y}");
    assert!(sup_y > 0.0, "superscript above baseline: {sup_y}");
    let none = scripts_attachment_offsets(base, None, None);
    assert_eq!(none, (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn math_bigop_limit_offsets_heuristic() {
    use reciplexa_std::math::{bigop_limit_offsets, MathBox, SCRIPT_SCALE};

    let op = MathBox::new(1.2, 0.9, 0.3);
    let lower = MathBox::new(0.8, 0.4, 0.1);
    let upper = MathBox::new(0.6, 0.4, 0.1);
    let (lx, ly, ux, uy) = bigop_limit_offsets(op, Some(lower), Some(upper));
    assert!((lx - (op.width - lower.width * SCRIPT_SCALE) * 0.5).abs() < 1e-9);
    assert!((ux - (op.width - upper.width * SCRIPT_SCALE) * 0.5).abs() < 1e-9);
    assert!(ly < 0.0, "lower limit below op: {ly}");
    assert!(uy > 0.0, "upper limit above op: {uy}");
    assert_eq!(bigop_limit_offsets(op, None, None), (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn math_matrix_column_widths_and_cases_left_align() {
    use reciplexa_std::math::{
        aligned_column_x, cases_column_align, matrix_cell_x_in_column, matrix_column_widths,
        MatrixColumnAlign, ALIGNED_COLUMN_GUTTER_EM,
    };

    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let bb = MathAtom::symbol(id(2), "bb", MathClass::Ordinary);
    let c = MathAtom::symbol(id(3), "c", MathClass::Ordinary);
    let d = MathAtom::symbol(id(4), "dddd", MathClass::Ordinary);
    let rows = vec![vec![a.clone(), bb.clone()], vec![c.clone(), d.clone()]];
    let widths = matrix_column_widths(&rows);
    assert_eq!(widths.len(), 2);
    assert!((widths[0] - a.estimate_box().width.max(c.estimate_box().width)).abs() < 1e-9);
    assert!((widths[1] - bb.estimate_box().width.max(d.estimate_box().width)).abs() < 1e-9);

    assert_eq!(cases_column_align(0), MatrixColumnAlign::Left);
    assert_eq!(cases_column_align(1), MatrixColumnAlign::Left);
    assert_eq!(
        matrix_cell_x_in_column(1.0, 3.0, MatrixColumnAlign::Left),
        0.0
    );
    assert!((matrix_cell_x_in_column(1.0, 3.0, MatrixColumnAlign::Center) - 1.0).abs() < 1e-9);
    assert!((matrix_cell_x_in_column(1.0, 3.0, MatrixColumnAlign::Right) - 2.0).abs() < 1e-9);

    assert!((aligned_column_x(&rows, 0)).abs() < 1e-9);
    assert!((aligned_column_x(&rows, 1) - (widths[0] + ALIGNED_COLUMN_GUTTER_EM)).abs() < 1e-9);
    let past = aligned_column_x(&rows, 99);
    assert!(
        (past - (widths[0] + ALIGNED_COLUMN_GUTTER_EM + widths[1] + ALIGNED_COLUMN_GUTTER_EM))
            .abs()
            < 1e-9
    );
    assert_eq!(aligned_column_x(&[], 0), 0.0);

    let m = MathAtom::matrix(
        id(5),
        reciplexa_std::math::MathMatrixKind::Plain,
        rows.clone(),
    );
    let aligned = MathAtom::aligned(id(6), rows);
    let expected_inner: f64 =
        widths.iter().sum::<f64>() + ALIGNED_COLUMN_GUTTER_EM * widths.len() as f64;
    assert!((m.estimate_box().width - expected_inner.max(0.5)).abs() < 1e-9);
    assert!((aligned.estimate_box().width - expected_inner.max(0.5)).abs() < 1e-9);
}

#[test]
fn math_delimiter_stretchy_grows_with_body() {
    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let b = MathAtom::symbol(id(2), "b", MathClass::Ordinary);
    let tall = MathAtom::fraction(id(3), a.clone(), b.clone());
    let flat = MathAtom::paren(id(4), a.clone());
    let around_frac = MathAtom::paren(id(5), tall.clone());

    let flat_box = flat.estimate_box();
    let tall_box = around_frac.estimate_box();
    assert!(
        tall_box.total_height() > flat_box.total_height(),
        "stretchy delimiter should grow with body: flat={flat_box:?} tall={tall_box:?}"
    );

    let stretched = MathAtom::delimiter_with_stretch(id(6), "(", ")", tall, 2.0);
    let defaulted = MathAtom::paren(id(7), MathAtom::fraction(id(8), a.clone(), b));
    assert!(
        stretched.estimate_box().total_height() > defaulted.estimate_box().total_height(),
        "stretch_factor > 1 should enlarge fences"
    );
}

#[test]
fn math_matrix_align_stack_nodes() {
    use reciplexa_std::math::{MathMatrixKind, MathStackKind};
    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let b = MathAtom::symbol(id(2), "b", MathClass::Ordinary);
    let c = MathAtom::symbol(id(3), "c", MathClass::Ordinary);
    let d = MathAtom::symbol(id(4), "d", MathClass::Ordinary);
    let m = MathAtom::matrix(
        id(5),
        MathMatrixKind::BMatrix,
        vec![vec![a.clone(), b.clone()], vec![c.clone(), d.clone()]],
    );
    assert_eq!(m.child_count(), 4);
    assert!(m.linearize().starts_with('['));
    assert!(m.estimate_box().width > a.estimate_box().width);

    let al = MathAtom::aligned(id(6), vec![vec![a.clone(), b.clone()]]);
    assert!(al.linearize().contains("align"));
    assert_eq!(MathMatrixKind::PMatrix.as_str(), "pmatrix");

    let st = MathAtom::atop(id(7), a.clone(), b.clone());
    assert_eq!(st.child_count(), 2);
    assert!(st.linearize().contains("atop"));
    assert_eq!(MathStackKind::Substack.to_string(), "substack");

    let delim = MathAtom::matrix_delimited(id(8), "{", "}", vec![vec![a]]);
    assert!(delim.linearize().starts_with('{'));
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
