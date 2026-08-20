//! Rebase GUI selection and source-editor spans after a document revision.

use reciplexa_lower::{collect_layers_authoring, LayerInfo};

/// Anchor layer selection across source revisions using stable layer metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerSelectionAnchor {
    pub kind: String,
    pub label: String,
    pub root_snippet: String,
}

pub fn capture_layer_anchors(source: &str, page_index: usize, selected: &[usize]) -> Vec<LayerSelectionAnchor> {
    let Ok(layers) = collect_layers_authoring(source, page_index) else {
        return Vec::new();
    };
    selected
        .iter()
        .filter_map(|&i| layers.get(i).map(|layer| layer_anchor(source, layer)))
        .collect()
}

fn layer_anchor(source: &str, layer: &LayerInfo) -> LayerSelectionAnchor {
    let snippet_end = layer.root_end.min(layer.root_start.saturating_add(96));
    LayerSelectionAnchor {
        kind: layer.kind.clone(),
        label: layer.label.clone(),
        root_snippet: source
            .get(layer.root_start..snippet_end)
            .unwrap_or("")
            .to_string(),
    }
}

pub fn resolve_layer_indices(
    source: &str,
    page_index: usize,
    anchors: &[LayerSelectionAnchor],
) -> Vec<usize> {
    if anchors.is_empty() {
        return Vec::new();
    }
    let Ok(layers) = collect_layers_authoring(source, page_index) else {
        return Vec::new();
    };
    anchors
        .iter()
        .filter_map(|anchor| best_layer_match(source, &layers, anchor))
        .collect()
}

fn best_layer_match(
    source: &str,
    layers: &[LayerInfo],
    anchor: &LayerSelectionAnchor,
) -> Option<usize> {
    let mut candidates: Vec<usize> = layers
        .iter()
        .enumerate()
        .filter(|(_, l)| l.kind == anchor.kind && l.label == anchor.label)
        .map(|(i, _)| i)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(candidates[0]);
    }
    if anchor.root_snippet.is_empty() {
        return Some(candidates[0]);
    }
    candidates.sort_by_key(|&i| snippet_score(source, &layers[i], &anchor.root_snippet));
    Some(candidates[0])
}

fn snippet_score(source: &str, layer: &LayerInfo, snippet: &str) -> usize {
    let end = layer.root_end.min(layer.root_start.saturating_add(snippet.len().max(96)));
    let actual = source.get(layer.root_start..end).unwrap_or("");
    usize::MAX - common_prefix_len(actual.as_bytes(), snippet.as_bytes())
}

fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

/// Map a byte offset from an old source revision into a new revision.
pub fn rebase_byte_offset(old: &str, new: &str, offset: usize) -> usize {
    let offset = offset.min(old.len());
    let old_bytes = old.as_bytes();
    let new_bytes = new.as_bytes();

    let mut prefix = 0usize;
    while prefix < offset
        && prefix < old.len()
        && prefix < new.len()
        && old_bytes[prefix] == new_bytes[prefix]
    {
        prefix += 1;
    }
    if offset <= prefix {
        return offset;
    }

    let mut old_suffix = old.len();
    let mut new_suffix = new.len();
    while old_suffix > prefix
        && new_suffix > prefix
        && old_bytes[old_suffix - 1] == new_bytes[new_suffix - 1]
    {
        old_suffix -= 1;
        new_suffix -= 1;
    }

    if offset >= old_suffix {
        let tail = old.len().saturating_sub(offset);
        return new.len().saturating_sub(tail);
    }

    let old_mid = old_suffix.saturating_sub(prefix);
    let new_mid = new_suffix.saturating_sub(prefix);
    let rel = offset.saturating_sub(prefix);
    if old_mid == 0 {
        return (prefix + rel).min(new.len());
    }
    let mapped = prefix + rel.saturating_mul(new_mid) / old_mid;
    mapped.min(new.len())
}

pub fn rebase_byte_range(old: &str, new: &str, start: usize, end: usize) -> (usize, usize) {
    let start = rebase_byte_offset(old, new, start);
    let end = rebase_byte_offset(old, new, end);
    if end < start {
        (start, start)
    } else {
        (start, end)
    }
}

pub fn byte_to_char_index(s: &str, byte: usize) -> usize {
    let mut b = byte.min(s.len());
    while b > 0 && !s.is_char_boundary(b) {
        b -= 1;
    }
    s[..b].chars().count()
}

pub fn char_to_byte_index(s: &str, char_index: usize) -> usize {
    for (count, (b, _)) in s.char_indices().enumerate() {
        if count == char_index {
            return b;
        }
    }
    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebase_offset_unchanged_in_common_prefix() {
        let old = "abcdef";
        let new = "abcXYZdef";
        assert_eq!(rebase_byte_offset(old, new, 2), 2);
    }

    #[test]
    fn rebase_offset_in_inserted_region() {
        let old = "abc def";
        let new = "abc LONG def";
        let mapped = rebase_byte_offset(old, new, 4);
        assert!(mapped >= 4 && mapped <= new.len());
    }

    #[test]
    fn layer_anchor_survives_text_edit() {
        const PKG: &str = include_str!("../../../examples/text_line.rpx");
        let layers = collect_layers_authoring(PKG, 0).expect("layers");
        assert!(!layers.is_empty());
        let anchors = capture_layer_anchors(PKG, 0, &[0]);
        let edited = PKG.replace("circle", "ellipse");
        let resolved = resolve_layer_indices(&edited, 0, &anchors);
        assert_eq!(resolved, vec![0]);
    }

    #[test]
    fn rebase_range_preserves_order() {
        let old = "hello world";
        let new = "hello brave world";
        let (s, e) = rebase_byte_range(old, new, 6, 11);
        assert!(s <= e);
        assert!(e <= new.len());
    }
}
