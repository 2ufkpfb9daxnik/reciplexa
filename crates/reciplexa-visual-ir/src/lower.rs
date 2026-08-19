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

    for (page_idx, _) in doc.pages.iter().enumerate() {
        // `page_idx` comes from `enumerate` over `doc.pages`, so flatten cannot miss.
        let (page, shapes) = flatten_page_with_options(doc, page_idx, flatten_opts)
            .expect("page index from enumerate is in range");
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
            glyph_ids: t.glyph_ids.clone(),
            font_digest: t.font_digest.clone(),
            glyph_advances_mm: t.glyph_advances_mm.clone(),
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
            glyph_ids,
            font_digest,
            glyph_advances_mm,
            ..
        } => {
            if let (Some(gids), Some(digest)) = (glyph_ids.as_ref(), font_digest.as_ref()) {
                if !gids.is_empty() {
                    let n = gids.len();
                    let advances = glyph_advances_mm
                        .clone()
                        .filter(|a| a.len() == n)
                        .unwrap_or_else(|| vec![*width_mm / n as f64; n]);
                    let mut cluster_starts = Vec::with_capacity(n);
                    let mut cluster_ends = Vec::with_capacity(n);
                    let mut byte = 0u32;
                    for (i, ch) in content.chars().enumerate() {
                        if i >= n {
                            break;
                        }
                        let end = byte + ch.len_utf8() as u32;
                        cluster_starts.push(byte);
                        cluster_ends.push(end);
                        byte = end;
                    }
                    while cluster_starts.len() < n {
                        cluster_starts.push(byte);
                        cluster_ends.push(content.len() as u32);
                    }
                    RenderNode::GlyphRun {
                        id: RenderNodeId::new(id.0),
                        font_digest: digest.clone(),
                        font_label: String::new(),
                        x_mm: *x_mm,
                        y_mm: *y_mm,
                        size_mm: *size_mm,
                        content: content.clone(),
                        fill: *fill,
                        alpha: *alpha,
                        gids: gids.clone(),
                        advances_mm: advances,
                        cluster_starts,
                        cluster_ends,
                    }
                } else {
                    RenderNode::Text {
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
                    }
                }
            } else {
                RenderNode::Text {
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
                }
            }
        }
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

/// Lower a product [`reciplexa_text_layout::GlyphRun`] to Render IR (IR-001).
pub fn render_from_glyph_run(
    id: RenderNodeId,
    run: &reciplexa_text_layout::GlyphRun,
    fill: reciplexa_scene::Color,
    alpha: f64,
) -> RenderNode {
    RenderNode::GlyphRun {
        id,
        font_digest: run.font.digest.clone(),
        font_label: run.font.label.clone(),
        x_mm: run.glyphs.first().map(|g| g.x_mm).unwrap_or(0.0),
        y_mm: run.glyphs.first().map(|g| g.y_mm).unwrap_or(0.0),
        size_mm: run.size_mm,
        content: run.content.clone(),
        fill,
        alpha,
        gids: run.glyphs.iter().map(|g| g.gid).collect(),
        advances_mm: run.glyphs.iter().map(|g| g.advance_mm).collect(),
        cluster_starts: run.glyphs.iter().map(|g| g.cluster_start as u32).collect(),
        cluster_ends: run.glyphs.iter().map(|g| g.cluster_end as u32).collect(),
    }
}
