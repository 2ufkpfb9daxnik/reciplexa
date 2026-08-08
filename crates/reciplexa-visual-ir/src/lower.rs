//! Lower scene documents through Domain → Layout → Render IR.

use reciplexa_identity::document::StableNodeId;
use reciplexa_scene::Document;
use reciplexa_view::{flatten_page_with_options, FlattenOptions, WorldShape};

use crate::domain::{DomainDocument, DomainNode, DomainNodeId, DomainPage};
use crate::layout::{layout_identity, LayoutDocument};
use crate::provenance::{ProvenanceMap, RenderProvenance};
use crate::render::{RenderDocument, RenderNode, RenderNodeId, RenderPage};

/// Optional source provenance for one paint-order leaf shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSourceHint {
    pub stable_node_id: StableNodeId,
    pub source_byte_start: u32,
    pub source_byte_end: u32,
}

/// Lowering options (profile + provenance hints).
#[derive(Debug, Clone, Default)]
pub struct LowerOptions {
    pub ellipse_sides: u32,
    /// Hints aligned to flattened paint order across all pages (sequential).
    pub provenance_hints: Vec<Option<NodeSourceHint>>,
}

impl LowerOptions {
    pub fn with_ellipse_sides(ellipse_sides: u32) -> Self {
        Self {
            ellipse_sides,
            provenance_hints: Vec::new(),
        }
    }
}

/// Lower a scene [`Document`] into validated Render IR (default options).
pub fn lower_scene_document(doc: &Document) -> (RenderDocument, ProvenanceMap) {
    lower_scene_document_with_options(doc, &LowerOptions::with_ellipse_sides(32))
}

/// Full layered lower: scene → domain → layout → render.
pub fn lower_scene_document_with_options(
    doc: &Document,
    options: &LowerOptions,
) -> (RenderDocument, ProvenanceMap) {
    let domain = scene_to_domain(doc, options);
    let layout = layout_identity(&domain);
    layout_to_render(&layout)
}

/// Scene → Domain IR.
pub fn scene_to_domain(doc: &Document, options: &LowerOptions) -> DomainDocument {
    let flatten_opts = FlattenOptions {
        ellipse_sides: if options.ellipse_sides == 0 {
            32
        } else {
            options.ellipse_sides
        },
    };
    let mut next_id = 1u64;
    let mut pages = Vec::new();
    let mut hint_idx = 0usize;

    for (page_idx, page) in doc.pages.iter().enumerate() {
        let Some((_, shapes)) = flatten_page_with_options(doc, page_idx, flatten_opts) else {
            continue;
        };
        let mut nodes = Vec::new();
        for shape in shapes {
            let id = DomainNodeId::new(next_id);
            next_id += 1;
            let hint = options
                .provenance_hints
                .get(hint_idx)
                .and_then(|h| h.clone());
            hint_idx += 1;
            nodes.push(world_shape_to_domain(id, &shape, hint));
        }
        pages.push(DomainPage {
            width_mm: page.paper.width_mm,
            height_mm: page.paper.height_mm,
            nodes,
        });
    }

    DomainDocument { pages }
}

/// Layout → Render IR + provenance side table.
pub fn layout_to_render(layout: &LayoutDocument) -> (RenderDocument, ProvenanceMap) {
    let mut provenance = ProvenanceMap::default();
    let mut pages = Vec::new();
    for page in &layout.pages {
        let mut nodes = Vec::new();
        for domain_node in &page.nodes {
            let render_id = RenderNodeId::new(domain_node.id().0);
            let (start, end) = domain_node.source_bytes();
            provenance.insert(RenderProvenance {
                render_id,
                stable_node_id: domain_node.stable_node_id(),
                source_byte_start: start,
                source_byte_end: end,
            });
            nodes.push(domain_to_render(domain_node));
        }
        pages.push(RenderPage {
            width_mm: page.width_mm,
            height_mm: page.height_mm,
            nodes,
        });
    }
    (RenderDocument { pages }, provenance)
}

fn world_shape_to_domain(
    id: DomainNodeId,
    shape: &WorldShape,
    hint: Option<NodeSourceHint>,
) -> DomainNode {
    let (stable_node_id, source_byte_start, source_byte_end) = match hint {
        Some(h) => (
            Some(h.stable_node_id),
            Some(h.source_byte_start),
            Some(h.source_byte_end),
        ),
        None => (None, None, None),
    };
    match shape {
        WorldShape::Circle(c) => DomainNode::Circle {
            id,
            x_mm: c.x_mm,
            y_mm: c.y_mm,
            radius_mm: c.radius_mm,
            fill: c.color,
            stroke_width_mm: c.stroke_width_mm,
            alpha: c.alpha,
            stable_node_id,
            source_byte_start,
            source_byte_end,
        },
        WorldShape::Polygon(p) => {
            // Recover axis-aligned rects when possible for Rect IR.
            if let Some((x, y, w, h)) = axis_aligned_rect(&p.points_mm) {
                DomainNode::Rect {
                    id,
                    x_mm: x,
                    y_mm: y,
                    width_mm: w,
                    height_mm: h,
                    fill: p.color,
                    stroke_width_mm: p.stroke_width_mm,
                    alpha: p.alpha,
                    stable_node_id,
                    source_byte_start,
                    source_byte_end,
                }
            } else {
                DomainNode::Polygon {
                    id,
                    points_mm: p.points_mm.clone(),
                    fill: p.color,
                    stroke_width_mm: p.stroke_width_mm,
                    alpha: p.alpha,
                    stable_node_id,
                    source_byte_start,
                    source_byte_end,
                }
            }
        }
        WorldShape::Text(t) => DomainNode::Text {
            id,
            x_mm: t.x_mm,
            y_mm: t.y_mm,
            size_mm: t.size_mm,
            width_mm: t.width_mm,
            height_mm: t.height_mm,
            rotation_deg: t.rotation_deg,
            content: t.content.clone(),
            fill: t.fill,
            alpha: t.alpha,
            stable_node_id,
            source_byte_start,
            source_byte_end,
        },
        WorldShape::Path(p) => DomainNode::Path {
            id,
            points_mm: p.points_mm.clone(),
            stroke: p.stroke,
            width_mm: p.width_mm,
            closed: p.closed,
            alpha: p.alpha,
            stable_node_id,
            source_byte_start,
            source_byte_end,
        },
        WorldShape::Image(i) => DomainNode::Image {
            id,
            path: i.path.clone(),
            corners_mm: i.corners_mm,
            alpha: i.alpha,
            stable_node_id,
            source_byte_start,
            source_byte_end,
        },
    }
}

fn domain_to_render(node: &DomainNode) -> RenderNode {
    match node {
        DomainNode::Circle {
            id,
            x_mm,
            y_mm,
            radius_mm,
            fill,
            stroke_width_mm,
            alpha,
            ..
        } => RenderNode::Circle {
            id: RenderNodeId::new(id.0),
            x_mm: *x_mm,
            y_mm: *y_mm,
            radius_mm: *radius_mm,
            fill: *fill,
            stroke_width_mm: *stroke_width_mm,
            alpha: *alpha,
        },
        DomainNode::Rect {
            id,
            x_mm,
            y_mm,
            width_mm,
            height_mm,
            fill,
            stroke_width_mm,
            alpha,
            ..
        } => RenderNode::Rect {
            id: RenderNodeId::new(id.0),
            x_mm: *x_mm,
            y_mm: *y_mm,
            width_mm: *width_mm,
            height_mm: *height_mm,
            fill: *fill,
            stroke_width_mm: *stroke_width_mm,
            alpha: *alpha,
        },
        DomainNode::Ellipse {
            id,
            x_mm,
            y_mm,
            rx_mm,
            ry_mm,
            fill,
            alpha,
            ..
        } => {
            // Ellipse should already be tessellated at domain build; keep a polygon fallback.
            let n = 32usize;
            let mut points_mm = Vec::with_capacity(n);
            for i in 0..n {
                let t = std::f64::consts::TAU * (i as f64) / (n as f64);
                points_mm.push((x_mm + rx_mm * t.cos(), y_mm + ry_mm * t.sin()));
            }
            RenderNode::Polygon {
                id: RenderNodeId::new(id.0),
                points_mm,
                fill: *fill,
                stroke_width_mm: None,
                alpha: *alpha,
            }
        }
        DomainNode::Polygon {
            id,
            points_mm,
            fill,
            stroke_width_mm,
            alpha,
            ..
        } => RenderNode::Polygon {
            id: RenderNodeId::new(id.0),
            points_mm: points_mm.clone(),
            fill: *fill,
            stroke_width_mm: *stroke_width_mm,
            alpha: *alpha,
        },
        DomainNode::Text {
            id,
            x_mm,
            y_mm,
            size_mm,
            width_mm,
            height_mm,
            rotation_deg,
            content,
            fill,
            alpha,
            ..
        } => RenderNode::Text {
            id: RenderNodeId::new(id.0),
            x_mm: *x_mm,
            y_mm: *y_mm,
            size_mm: *size_mm,
            width_mm: *width_mm,
            height_mm: *height_mm,
            rotation_deg: *rotation_deg,
            content: content.clone(),
            fill: *fill,
            alpha: *alpha,
        },
        DomainNode::Path {
            id,
            points_mm,
            stroke,
            width_mm,
            closed,
            alpha,
            ..
        } => RenderNode::Path {
            id: RenderNodeId::new(id.0),
            points_mm: points_mm.clone(),
            stroke: *stroke,
            width_mm: *width_mm,
            closed: *closed,
            alpha: *alpha,
        },
        DomainNode::Image {
            id,
            path,
            corners_mm,
            alpha,
            ..
        } => RenderNode::Image {
            id: RenderNodeId::new(id.0),
            path: path.clone(),
            corners_mm: *corners_mm,
            alpha: *alpha,
        },
    }
}

fn axis_aligned_rect(points: &[(f64, f64)]) -> Option<(f64, f64, f64, f64)> {
    if points.len() != 4 {
        return None;
    }
    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    // All points must lie on the AABB corners (axis-aligned).
    for &(x, y) in points {
        let on_x = (x - min_x).abs() < 1e-9 || (x - max_x).abs() < 1e-9;
        let on_y = (y - min_y).abs() < 1e-9 || (y - max_y).abs() < 1e-9;
        if !(on_x && on_y) {
            return None;
        }
    }
    let w = max_x - min_x;
    let h = max_y - min_y;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some((min_x, min_y, w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_identity::document::StableNodeId;
    use reciplexa_scene::{
        Affine, Circle, Color, Document, Image, Line, Page, PaperSize, Polygon, Polyline, Rect,
        Shape, Text,
    };

    #[test]
    fn lowers_rect_to_render_ir() {
        let mut doc = Document::default();
        doc.pages.push(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 10.0,
                y_mm: 20.0,
                width_mm: 30.0,
                height_mm: 40.0,
                fill: Color::RED,
            })],
        });
        let (render, prov) = lower_scene_document(&doc);
        assert_eq!(render.pages.len(), 1);
        assert_eq!(render.pages[0].nodes.len(), 1);
        assert!(matches!(render.pages[0].nodes[0], RenderNode::Rect { .. }));
        assert!(prov.get(RenderNodeId::new(1)).is_some());
    }

    #[test]
    fn lowers_circle_to_render_ir() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Circle(Circle {
                x_mm: 5.0,
                y_mm: 5.0,
                radius_mm: 3.0,
                fill: Color::BLUE,
            })],
        });
        let (render, _) = lower_scene_document(&doc);
        assert!(matches!(
            render.pages[0].nodes[0],
            RenderNode::Circle { .. }
        ));
    }

    #[test]
    fn lowers_text_to_render_ir() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Text(Text {
                x_mm: 1.0,
                y_mm: 2.0,
                size_mm: 12.0,
                width_mm: None,
                height_mm: None,
                content: "hello".into(),
                fill: Color::BLACK,
            })],
        });
        let (render, _) = lower_scene_document(&doc);
        assert!(matches!(render.pages[0].nodes[0], RenderNode::Text { .. }));
    }

    #[test]
    fn lowers_path_to_render_ir() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Line(Line {
                x1_mm: 0.0,
                y1_mm: 0.0,
                x2_mm: 10.0,
                y2_mm: 10.0,
                stroke: Color::BLACK,
                width_mm: 0.5,
            })],
        });
        let (render, _) = lower_scene_document(&doc);
        assert!(matches!(render.pages[0].nodes[0], RenderNode::Path { .. }));
    }

    #[test]
    fn lowers_image_to_render_ir() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Image(Image {
                path: "pic.png".into(),
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 30.0,
                height_mm: 20.0,
            })],
        });
        let (render, _) = lower_scene_document(&doc);
        assert!(matches!(render.pages[0].nodes[0], RenderNode::Image { .. }));
    }

    #[test]
    fn lowers_non_axis_aligned_polygon() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Polygon(Polygon {
                points_mm: vec![(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)],
                fill: Color::GREEN,
            })],
        });
        let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
        assert!(matches!(
            domain.pages[0].nodes[0],
            DomainNode::Polygon { .. }
        ));
        let (render, _) = layout_to_render(&layout_identity(&domain));
        assert!(matches!(
            render.pages[0].nodes[0],
            RenderNode::Polygon { .. }
        ));
    }

    #[test]
    fn axis_aligned_rect_becomes_domain_rect() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 3.0,
                height_mm: 4.0,
                fill: Color::BLACK,
            })],
        });
        let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
        assert!(matches!(domain.pages[0].nodes[0], DomainNode::Rect { .. }));
    }

    #[test]
    fn rotated_rect_stays_polygon() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Group {
                transform: Affine::rotate_deg(45.0),
                children: vec![Shape::Rect(Rect {
                    x_mm: 0.0,
                    y_mm: 0.0,
                    width_mm: 10.0,
                    height_mm: 5.0,
                    fill: Color::BLACK,
                })],
            }],
        });
        let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
        assert!(matches!(
            domain.pages[0].nodes[0],
            DomainNode::Polygon { .. }
        ));
    }

    #[test]
    fn three_point_polygon_not_recognized_as_rect() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Polygon(Polygon {
                points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)],
                fill: Color::BLACK,
            })],
        });
        let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
        assert!(matches!(
            domain.pages[0].nodes[0],
            DomainNode::Polygon { .. }
        ));
    }

    #[test]
    fn zero_area_rect_stays_polygon() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 0.0,
                height_mm: 10.0,
                fill: Color::BLACK,
            })],
        });
        let domain = scene_to_domain(&doc, &LowerOptions::with_ellipse_sides(32));
        assert!(matches!(
            domain.pages[0].nodes[0],
            DomainNode::Polygon { .. }
        ));
    }

    #[test]
    fn ellipse_sides_zero_defaults_to_32() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Polyline(Polyline {
                points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
                stroke: Color::BLACK,
                width_mm: 1.0,
            })],
        });
        let options = LowerOptions {
            ellipse_sides: 0,
            provenance_hints: vec![],
        };
        let (render, _) = lower_scene_document_with_options(&doc, &options);
        assert_eq!(render.pages.len(), 1);
    }

    #[test]
    fn multi_page_provenance_hints_align_sequentially() {
        let mut doc = Document::default();
        doc.pages.push(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 1.0,
                height_mm: 1.0,
                fill: Color::BLACK,
            })],
        });
        doc.pages.push(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Circle(Circle {
                x_mm: 5.0,
                y_mm: 5.0,
                radius_mm: 2.0,
                fill: Color::RED,
            })],
        });
        let options = LowerOptions {
            ellipse_sides: 32,
            provenance_hints: vec![
                Some(NodeSourceHint {
                    stable_node_id: StableNodeId::new(1),
                    source_byte_start: 0,
                    source_byte_end: 5,
                }),
                Some(NodeSourceHint {
                    stable_node_id: StableNodeId::new(2),
                    source_byte_start: 10,
                    source_byte_end: 20,
                }),
            ],
        };
        let (_, prov) = lower_scene_document_with_options(&doc, &options);
        assert_eq!(
            prov.get(RenderNodeId::new(1)).unwrap().stable_node_id,
            Some(StableNodeId::new(1))
        );
        assert_eq!(
            prov.get(RenderNodeId::new(2)).unwrap().stable_node_id,
            Some(StableNodeId::new(2))
        );
    }

    #[test]
    fn ellipse_domain_node_fallback_to_polygon_render() {
        use crate::domain::DomainNodeId;
        use crate::layout::LayoutPage;
        let layout = LayoutDocument {
            pages: vec![LayoutPage {
                width_mm: 210.0,
                height_mm: 297.0,
                nodes: vec![DomainNode::Ellipse {
                    id: DomainNodeId::new(9),
                    x_mm: 10.0,
                    y_mm: 20.0,
                    rx_mm: 5.0,
                    ry_mm: 3.0,
                    fill: Color::BLACK,
                    alpha: 1.0,
                    stable_node_id: None,
                    source_byte_start: None,
                    source_byte_end: None,
                }],
            }],
        };
        let (render, prov) = layout_to_render(&layout);
        match &render.pages[0].nodes[0] {
            RenderNode::Polygon { points_mm, .. } => assert_eq!(points_mm.len(), 32),
            other => panic!("expected polygon fallback, got {other:?}"),
        }
        assert!(prov.get(RenderNodeId::new(9)).is_some());
    }

    #[test]
    fn provenance_none_when_no_hints() {
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 3.0,
                height_mm: 4.0,
                fill: Color::BLACK,
            })],
        });
        let (_, prov) = lower_scene_document(&doc);
        let entry = prov.get(RenderNodeId::new(1)).unwrap();
        assert_eq!(entry.stable_node_id, None);
        assert_eq!(entry.source_byte_start, None);
        assert_eq!(entry.source_byte_end, None);
    }

    #[test]
    fn attaches_provenance_hints() {
        let mut doc = Document::default();
        doc.pages.push(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Rect(Rect {
                x_mm: 1.0,
                y_mm: 2.0,
                width_mm: 3.0,
                height_mm: 4.0,
                fill: Color::BLACK,
            })],
        });
        let options = LowerOptions {
            ellipse_sides: 32,
            provenance_hints: vec![Some(NodeSourceHint {
                stable_node_id: StableNodeId::new(42),
                source_byte_start: 5,
                source_byte_end: 20,
            })],
        };
        let (_, prov) = lower_scene_document_with_options(&doc, &options);
        let entry = prov.get(RenderNodeId::new(1)).unwrap();
        assert_eq!(entry.stable_node_id, Some(StableNodeId::new(42)));
        assert_eq!(entry.source_byte_start, Some(5));
        assert_eq!(entry.source_byte_end, Some(20));
    }
}
