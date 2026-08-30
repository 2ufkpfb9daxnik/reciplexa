//! Hierarchical layer rows: authoring parent + GlyphRun children (Step 14 slice 1).

use reciplexa_view::WorldShape;

use super::LayerInfo;

fn world_text_content(shape: &WorldShape) -> Option<&str> {
    match shape {
        WorldShape::Text(t) => Some(t.content.as_str()),
        _ => None,
    }
}

fn glyph_child(parent: &LayerInfo, shape_index: usize, content: &str, depth: usize) -> LayerInfo {
    let short: String = content.chars().take(24).collect();
    let mut child = LayerInfo::new(
        "glyph",
        format!("glyph \"{short}\""),
        parent.byte_start,
        parent.byte_end,
        parent.root_start,
        parent.root_end,
    );
    child.depth = depth;
    child.authoring_index = parent.authoring_index;
    child.glyph_child = true;
    child.shape_index = Some(shape_index);
    child.text_content = Some(content.to_string());
    child
}

fn consume_matching_shapes(shapes: &[WorldShape], start: usize, expected: &str) -> Option<usize> {
    if expected.is_empty() {
        return None;
    }
    let mut acc = String::new();
    for (offset, shape) in shapes[start..].iter().enumerate() {
        let piece = world_text_content(shape)?;
        acc.push_str(piece);
        if acc == expected {
            return Some(offset + 1);
        }
        if !expected.starts_with(&acc) {
            return None;
        }
    }
    None
}

fn wants_glyph_children(layer: &LayerInfo) -> bool {
    if layer.kind.starts_with("math-") {
        return false;
    }
    layer.kind == "text"
        || layer.kind == "vert-sample"
        || layer.kind == "vert-ruby"
        || layer.kind.starts_with("doc-")
        || layer.text_content.is_some()
}

/// Map authoring layers onto flattened scene shapes, inserting GlyphRun children.
pub fn attach_glyph_children(layers: Vec<LayerInfo>, shapes: &[WorldShape]) -> Vec<LayerInfo> {
    if layers.is_empty() {
        return layers;
    }
    if shapes.is_empty() {
        let mut layers = layers;
        assign_authoring_indices(&mut layers);
        return layers;
    }
    if layers.len() == shapes.len() {
        return attach_aligned(layers, shapes);
    }
    attach_by_content(layers, shapes)
}

fn attach_aligned(layers: Vec<LayerInfo>, shapes: &[WorldShape]) -> Vec<LayerInfo> {
    let mut out = Vec::with_capacity(layers.len().saturating_mul(2));
    for (i, mut layer) in layers.into_iter().enumerate() {
        layer.authoring_index = i;
        layer.shape_index = Some(i);
        let textish = wants_glyph_children(&layer) && world_text_content(&shapes[i]).is_some();
        if textish {
            let content = world_text_content(&shapes[i]).unwrap_or("");
            let mut parent = layer.clone();
            parent.shape_index = None;
            let depth = parent.depth + 1;
            out.push(parent);
            out.push(glyph_child(&layer, i, content, depth));
        } else {
            out.push(layer);
        }
    }
    out
}

fn attach_by_content(layers: Vec<LayerInfo>, shapes: &[WorldShape]) -> Vec<LayerInfo> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    for (i, mut layer) in layers.into_iter().enumerate() {
        layer.authoring_index = i;
        let expected = layer.text_content.clone().filter(|s| !s.is_empty());
        if let Some(expected) = expected.filter(|_| wants_glyph_children(&layer)) {
            let mut matched =
                consume_matching_shapes(shapes, cursor, &expected).map(|n| (cursor, n));
            if matched.is_none() {
                for start in cursor..shapes.len() {
                    if let Some(n) = consume_matching_shapes(shapes, start, &expected) {
                        matched = Some((start, n));
                        break;
                    }
                }
            }
            if let Some((start, n)) = matched {
                let depth = layer.depth + 1;
                out.push(layer.clone());
                for k in 0..n {
                    let idx = start + k;
                    let content = world_text_content(&shapes[idx]).unwrap_or("");
                    out.push(glyph_child(&layer, idx, content, depth));
                }
                cursor = start + n;
                continue;
            }
        }
        out.push(layer);
    }
    out
}

pub fn assign_authoring_indices(layers: &mut [LayerInfo]) {
    for (i, layer) in layers.iter_mut().enumerate() {
        if !layer.glyph_child {
            layer.authoring_index = i;
        }
    }
}

/// Authoring layer (parent) for a flattened scene shape.
pub fn parent_layer_index(layers: &[LayerInfo], shape_i: usize) -> Option<usize> {
    let child = layers.iter().position(|l| l.shape_index == Some(shape_i))?;
    if !layers[child].glyph_child {
        return Some(child);
    }
    let ai = layers[child].authoring_index;
    layers
        .iter()
        .position(|l| !l.glyph_child && l.authoring_index == ai)
        .or(Some(child))
}

/// Scene shapes owned by the selected authoring nodes (parent + glyph children).
pub fn shape_indices_for_selection(layers: &[LayerInfo], selected: &[usize]) -> Vec<usize> {
    let mut out = Vec::new();
    for &sel in selected {
        let Some(layer) = layers.get(sel) else {
            continue;
        };
        let ai = layer.authoring_index;
        for l in layers {
            if l.authoring_index != ai {
                continue;
            }
            if let Some(s) = l.shape_index {
                if !out.contains(&s) {
                    out.push(s);
                }
            }
        }
    }
    out
}

/// CST / authoring-only indices for mutations (`nudge_layer_*`, props, …).
pub fn authoring_indices_for_selection(layers: &[LayerInfo], selected: &[usize]) -> Vec<usize> {
    let mut v: Vec<usize> = selected
        .iter()
        .filter_map(|&i| layers.get(i).map(|l| l.authoring_index))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

pub fn layers_cover_shapes(layers: &[LayerInfo], n_shapes: usize) -> bool {
    if n_shapes == 0 {
        return true;
    }
    let mut seen = vec![false; n_shapes];
    for l in layers {
        if let Some(i) = l.shape_index {
            if i < n_shapes {
                seen[i] = true;
            }
        }
    }
    seen.iter().all(|&b| b)
}

/// Preview-row index of an authoring parent (for pane highlight).
pub fn preview_index_for_authoring(layers: &[LayerInfo], authoring_index: usize) -> Option<usize> {
    layers
        .iter()
        .position(|l| !l.glyph_child && l.authoring_index == authoring_index)
}

/// Scene shapes owned by authoring (CST) selection indices.
pub fn shape_indices_for_authoring(layers: &[LayerInfo], authoring: &[usize]) -> Vec<usize> {
    let preview: Vec<usize> = authoring
        .iter()
        .filter_map(|&ai| preview_index_for_authoring(layers, ai))
        .collect();
    shape_indices_for_selection(layers, &preview)
}

pub fn first_shape_for_authoring(layers: &[LayerInfo], authoring_index: usize) -> Option<usize> {
    layers
        .iter()
        .find(|l| l.authoring_index == authoring_index && l.shape_index.is_some())
        .and_then(|l| l.shape_index)
}

#[cfg(test)]
mod tests {
    use reciplexa_view::{WorldShape, WorldText};

    use super::*;

    fn text_shape(s: &str) -> WorldShape {
        WorldShape::Text(WorldText {
            x_mm: 0.0,
            y_mm: 0.0,
            size_mm: 8.0,
            width_mm: 8.0,
            height_mm: 8.0,
            rotation_deg: 0.0,
            content: s.into(),
            fill: reciplexa_scene::Color::BLACK,
            alpha: 1.0,
            glyph_ids: None,
            font_digest: None,
            glyph_advances_mm: None,
        })
    }

    #[test]
    fn aligned_text_gains_glyph_child() {
        let parent = LayerInfo::new("text", "text \"ab\"", 0, 1, 0, 1).with_text_content("ab");
        let other = LayerInfo::new("circle", "circle", 2, 3, 2, 3);
        let layers = attach_glyph_children(vec![parent, other], &[text_shape("ab")]);
        // 1:1 requires matching lens; this test uses content path.
        assert!(layers.iter().any(|l| l.kind == "glyph"));
        let parent = layers.iter().find(|l| l.kind == "text").unwrap();
        assert!(!parent.glyph_child);
        let child = layers.iter().find(|l| l.kind == "glyph").unwrap();
        assert_eq!(child.authoring_index, parent.authoring_index);
        assert_eq!(child.depth, parent.depth + 1);
    }

    #[test]
    fn vertical_sample_consumes_per_glyph_runs() {
        let sample = LayerInfo::new("vert-sample", "sample \"縦書き\"", 0, 1, 0, 1)
            .with_text_content("縦書き");
        let shapes = vec![text_shape("縦"), text_shape("書"), text_shape("き")];
        let layers = attach_glyph_children(vec![sample], &shapes);
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].kind, "vert-sample");
        assert!(layers[1].glyph_child && layers[1].label.contains("縦"));
        assert_eq!(parent_layer_index(&layers, 2), Some(0));
        assert_eq!(shape_indices_for_selection(&layers, &[0]), vec![0, 1, 2]);
        assert!(layers_cover_shapes(&layers, 3));
    }
}
