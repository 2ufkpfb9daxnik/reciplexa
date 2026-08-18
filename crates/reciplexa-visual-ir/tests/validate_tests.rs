use reciplexa_scene::Color;
use reciplexa_visual_ir::render::*;
use reciplexa_visual_ir::validate::*;

fn base_page(nodes: Vec<RenderNode>) -> RenderDocument {
    RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: 297.0,
            nodes,
        }],
    }
}

fn rect_node(id: u64, width_mm: f64, height_mm: f64, alpha: f64) -> RenderNode {
    RenderNode::Rect {
        id: RenderNodeId::new(id),
        x_mm: 0.0,
        y_mm: 0.0,
        width_mm,
        height_mm,
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha,
    }
}

#[test]
fn rejects_empty_document() {
    let doc = RenderDocument { pages: vec![] };
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::EmptyDocument)
    ));
}

#[test]
fn rejects_zero_page_width() {
    let doc = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 0.0,
            height_mm: 297.0,
            nodes: vec![],
        }],
    };
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidPageSize { page: 0 })
    ));
}

#[test]
fn rejects_negative_page_height() {
    let doc = RenderDocument {
        pages: vec![RenderPage {
            width_mm: 210.0,
            height_mm: -1.0,
            nodes: vec![],
        }],
    };
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidPageSize { page: 0 })
    ));
}

#[test]
fn rejects_nan_page_size() {
    let doc = RenderDocument {
        pages: vec![RenderPage {
            width_mm: f64::NAN,
            height_mm: 297.0,
            nodes: vec![],
        }],
    };
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidPageSize { page: 0 })
    ));
}

#[test]
fn rejects_duplicate_node_ids() {
    let doc = base_page(vec![
        rect_node(1, 10.0, 10.0, 1.0),
        rect_node(1, 5.0, 5.0, 1.0),
    ]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::DuplicateNodeId(1))
    ));
}

#[test]
fn rejects_invalid_rect_dimensions() {
    let doc = base_page(vec![rect_node(1, -1.0, 1.0, 1.0)]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "rect dimensions must be positive"
    ));
}

#[test]
fn rejects_rect_zero_height() {
    let doc = base_page(vec![rect_node(1, 1.0, 0.0, 1.0)]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_rect_alpha_below_zero() {
    let doc = base_page(vec![rect_node(1, 1.0, 1.0, -0.1)]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "alpha out of range"
    ));
}

#[test]
fn rejects_rect_alpha_above_one() {
    let doc = base_page(vec![rect_node(1, 1.0, 1.0, 1.1)]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_rect_nan_alpha() {
    let doc = base_page(vec![rect_node(1, 1.0, 1.0, f64::NAN)]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_circle_zero_radius() {
    let doc = base_page(vec![RenderNode::Circle {
        id: RenderNodeId::new(1),
        x_mm: 0.0,
        y_mm: 0.0,
        radius_mm: 0.0,
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
    }]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "circle radius must be positive"
    ));
}

#[test]
fn rejects_circle_negative_radius() {
    let doc = base_page(vec![RenderNode::Circle {
        id: RenderNodeId::new(1),
        x_mm: 0.0,
        y_mm: 0.0,
        radius_mm: -2.0,
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
    }]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_circle_alpha_out_of_range() {
    let doc = base_page(vec![RenderNode::Circle {
        id: RenderNodeId::new(1),
        x_mm: 0.0,
        y_mm: 0.0,
        radius_mm: 5.0,
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 2.0,
    }]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_polygon_too_few_points() {
    let doc = base_page(vec![RenderNode::Polygon {
        id: RenderNodeId::new(1),
        points_mm: vec![(0.0, 0.0), (1.0, 0.0)],
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
    }]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "polygon needs at least 3 points"
    ));
}

#[test]
fn rejects_polygon_non_finite_coordinate() {
    let doc = base_page(vec![RenderNode::Polygon {
        id: RenderNodeId::new(1),
        points_mm: vec![(0.0, 0.0), (1.0, f64::NAN), (0.0, 1.0)],
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: 1.0,
    }]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "polygon coordinate not finite"
    ));
}

#[test]
fn rejects_polygon_alpha_out_of_range() {
    let doc = base_page(vec![RenderNode::Polygon {
        id: RenderNodeId::new(1),
        points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
        fill: Color::BLACK,
        stroke_width_mm: None,
        alpha: -0.5,
    }]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_text_zero_size() {
    let doc = base_page(vec![RenderNode::Text {
        id: RenderNodeId::new(1),
        x_mm: 0.0,
        y_mm: 0.0,
        size_mm: 0.0,
        width_mm: 10.0,
        height_mm: 10.0,
        rotation_deg: 0.0,
        content: "x".into(),
        fill: Color::BLACK,
        alpha: 1.0,
    }]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "text size must be positive"
    ));
}

#[test]
fn rejects_text_alpha_out_of_range() {
    let doc = base_page(vec![RenderNode::Text {
        id: RenderNodeId::new(1),
        x_mm: 0.0,
        y_mm: 0.0,
        size_mm: 12.0,
        width_mm: 10.0,
        height_mm: 10.0,
        rotation_deg: 0.0,
        content: "x".into(),
        fill: Color::BLACK,
        alpha: f64::INFINITY,
    }]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_path_too_few_points() {
    let doc = base_page(vec![RenderNode::Path {
        id: RenderNodeId::new(1),
        points_mm: vec![(0.0, 0.0)],
        stroke: Color::BLACK,
        width_mm: 1.0,
        closed: false,
        alpha: 1.0,
    }]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "path needs at least 2 points"
    ));
}

#[test]
fn rejects_path_zero_width() {
    let doc = base_page(vec![RenderNode::Path {
        id: RenderNodeId::new(1),
        points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
        stroke: Color::BLACK,
        width_mm: 0.0,
        closed: false,
        alpha: 1.0,
    }]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "path width must be positive"
    ));
}

#[test]
fn rejects_path_alpha_out_of_range() {
    let doc = base_page(vec![RenderNode::Path {
        id: RenderNodeId::new(1),
        points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
        stroke: Color::BLACK,
        width_mm: 1.0,
        closed: false,
        alpha: 1.5,
    }]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn rejects_image_non_finite_corner() {
    let doc = base_page(vec![RenderNode::Image {
        id: RenderNodeId::new(1),
        path: "a.png".into(),
        corners_mm: [(0.0, 0.0), (f64::INFINITY, 0.0), (1.0, 1.0), (0.0, 1.0)],
        alpha: 1.0,
    }]);
    assert!(matches!(
        validate_render_document(&doc),
        Err(RenderValidationError::InvalidNode { reason, .. })
        if reason == "image corner not finite"
    ));
}

#[test]
fn rejects_image_alpha_out_of_range() {
    let doc = base_page(vec![RenderNode::Image {
        id: RenderNodeId::new(1),
        path: "a.png".into(),
        corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
        alpha: -1.0,
    }]);
    assert!(validate_render_document(&doc).is_err());
}

#[test]
fn accepts_valid_minimal_all_kinds() {
    let doc = base_page(vec![
        rect_node(1, 1.0, 1.0, 0.0),
        RenderNode::Circle {
            id: RenderNodeId::new(2),
            x_mm: 1.0,
            y_mm: 1.0,
            radius_mm: 0.1,
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 1.0,
        },
        RenderNode::Polygon {
            id: RenderNodeId::new(3),
            points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
            fill: Color::BLACK,
            stroke_width_mm: None,
            alpha: 0.5,
        },
        RenderNode::Text {
            id: RenderNodeId::new(4),
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 1.0,
            width_mm: 10.0,
            height_mm: 10.0,
            rotation_deg: 0.0,
            content: "ok".into(),
            fill: Color::BLACK,
            alpha: 1.0,
        },
        RenderNode::Path {
            id: RenderNodeId::new(5),
            points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
            stroke: Color::BLACK,
            width_mm: 0.1,
            closed: false,
            alpha: 1.0,
        },
        RenderNode::Image {
            id: RenderNodeId::new(6),
            path: "img.png".into(),
            corners_mm: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            alpha: 1.0,
        },
        RenderNode::GlyphRun {
            id: RenderNodeId::new(7),
            font_digest: "abc".into(),
            font_label: "ReciplexaFixture".into(),
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 4.0,
            content: "A".into(),
            fill: Color::BLACK,
            alpha: 1.0,
            gids: vec![1],
            advances_mm: vec![2.4],
            cluster_starts: vec![0],
            cluster_ends: vec![1],
        },
    ]);
    assert!(validate_render_document(&doc).is_ok());
}

#[test]
fn accepts_alpha_boundary_values() {
    let doc = base_page(vec![
        rect_node(1, 1.0, 1.0, 0.0),
        rect_node(2, 1.0, 1.0, 1.0),
    ]);
    assert!(validate_render_document(&doc).is_ok());
}
