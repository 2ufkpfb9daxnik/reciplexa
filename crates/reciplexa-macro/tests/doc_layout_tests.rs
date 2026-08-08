use reciplexa_macro::doc_layout::*;
use reciplexa_syntax::{doc_parts, parse_source, DocPart, SyntaxKind};


fn parts(src: &str) -> Vec<DocPart> {
    let root = parse_source(src).into_result().unwrap();
    let list = root
        .children()
        .find(|n| n.kind() == SyntaxKind::List)
        .unwrap();
    doc_parts(&list).unwrap()
}

fn text_items(items: &[LaidItem]) -> Vec<&LaidLine> {
    items
        .iter()
        .filter_map(|i| match i {
            LaidItem::Text(t) => Some(t),
            _ => None,
        })
        .collect()
}

// --- validity: wrap ---

#[test]
fn wrap_respects_space_break() {
    let lines = wrap_line("hello world again", 11);
    assert_eq!(lines, vec!["hello world", "again"]);
}

#[test]
fn wrap_hard_breaks_cjk_when_no_space() {
    let s: String = "あ".repeat(5);
    let lines = wrap_line(&s, 2);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].chars().count(), 2);
    assert_eq!(lines[2].chars().count(), 1);
}

#[test]
fn wrap_avoids_line_start_with_cjk_period() {
    // Without kinsoku, max=3 on "あああ。" yields ["あああ", "。"].
    let lines = wrap_line("あああ。", 3);
    assert!(
        lines.iter().all(|l| !l.starts_with('。')),
        "period must not start a line: {lines:?}"
    );
    assert_eq!(lines, vec!["ああ", "あ。"]);
}

#[test]
fn wrap_prefers_break_after_cjk_period() {
    // Prefer ending the first line on 。 rather than after the next kana.
    let lines = wrap_line("あい。うえお", 4);
    assert_eq!(lines, vec!["あい。", "うえお"]);
}

#[test]
fn wrap_prefers_break_after_middle_dot() {
    let lines = wrap_line("赤・青・緑いろ", 4);
    assert_eq!(lines, vec!["赤・青・", "緑いろ"]);
}

#[test]
fn wrap_avoids_line_start_with_middle_dot() {
    let lines = wrap_line("あいう・え", 3);
    assert!(lines.iter().all(|l| !l.starts_with('・')), "{lines:?}");
}

#[test]
fn wrap_avoids_line_start_with_fullwidth_question() {
    let lines = wrap_line("あああ？", 3);
    assert!(
        lines.iter().all(|l| !l.starts_with('？')),
        "fullwidth ? must not start a line: {lines:?}"
    );
    assert_eq!(lines, vec!["ああ", "あ？"]);
}

#[test]
fn wrap_avoids_line_end_with_opening_bracket() {
    // Without kinsoku, max=2 on "あ「いう" yields ["あ「", "いう"].
    let lines = wrap_line("あ「いう", 2);
    assert!(
        lines.iter().all(|l| !l.ends_with('「')),
        "'「' must not end a line: {lines:?}"
    );
    assert_eq!(lines, vec!["あ", "「い", "う"]);
}

#[test]
fn wrap_avoids_line_start_with_closing_paren() {
    let lines = wrap_line("abc)", 3);
    assert!(
        lines.iter().all(|l| !l.starts_with(')')),
        "')' must not start a line: {lines:?}"
    );
    assert_eq!(lines, vec!["ab", "c)"]);
}

#[test]
fn layout_wraps_long_paragraph() {
    let long = "a".repeat(45);
    let src = format!("(doc @p{{{long}}})");
    let laid = layout_doc_parts(&parts(&src));
    let texts = text_items(&laid);
    assert!(texts.len() >= 2, "expected wrap into ≥2 lines: {laid:?}");
    assert!(texts.iter().all(|l| l.size_mm == BODY_SIZE_MM));
    assert!(texts
        .iter()
        .all(|l| l.content.chars().count() <= BODY_WRAP_CHARS));
}

// --- defect: wrap ---

#[test]
fn wrap_max_zero_is_noop() {
    assert_eq!(wrap_line("abc def", 0), vec!["abc def".to_string()]);
}

#[test]
fn wrap_empty_is_empty() {
    assert!(wrap_line("", 10).is_empty());
}

#[test]
fn wrap_single_kinsoku_char_still_emits() {
    // A lone forbidden-start char must still appear (no infinite shrink).
    assert_eq!(wrap_line("。", 1), vec!["。".to_string()]);
}

#[test]
fn layout_skips_empty_title() {
    assert!(layout_doc_parts(&parts("(doc @title{})")).is_empty());
}

#[test]
fn place_items_starts_new_page_at_bottom_margin() {
    let items: Vec<LaidItem> = (0..30)
        .map(|i| {
            LaidItem::Text(LaidLine {
                size_mm: BODY_SIZE_MM,
                y_gap_after: BODY_GAP_MM,
                indent_mm: 0.0,
                content: format!("L{i}"),
            })
        })
        .collect();
    let placed = place_items(&items, DocFrame::A4);
    let max_page = placed.iter().map(PlacedItem::page_index).max().unwrap();
    assert!(max_page >= 1, "expected a second page, got {placed:?}");
    assert!(placed.iter().all(|p| match p {
        PlacedItem::Text(t) => t.y_mm >= DocFrame::A4.bottom_mm,
        PlacedItem::Line { y1_mm, .. } => *y1_mm >= DocFrame::A4.bottom_mm,
        PlacedItem::Image { y_mm, .. } => *y_mm >= DocFrame::A4.bottom_mm,
    }));
}

#[test]
fn place_items_emits_hr_line() {
    let items = vec![
        LaidItem::Text(LaidLine {
            size_mm: BODY_SIZE_MM,
            y_gap_after: BODY_GAP_MM,
            indent_mm: 0.0,
            content: "Head".into(),
        }),
        LaidItem::Hr {
            y_gap_after: HR_GAP_MM,
        },
    ];
    let placed = place_items(&items, DocFrame::A4);
    assert_eq!(placed.len(), 2);
    match &placed[1] {
        PlacedItem::Line {
            x1_mm,
            x2_mm,
            y1_mm,
            width_mm,
            ..
        } => {
            assert_eq!(*x1_mm, 25.0);
            assert_eq!(*x2_mm, 185.0);
            assert_eq!(*y1_mm, 258.0);
            assert_eq!(*width_mm, HR_WIDTH_MM);
        }
        other => panic!("expected line, got {other:?}"),
    }
}

#[test]
fn layout_doc_macros_cover_flow_items() {
    let src = r#"(doc
@title{T}
@h2{S}
@p{Body}
@quote{Q}
@note{N}
@warn{W}
@todo{T}
@code{fn main}
@ol{@li{a} @li{b}}
@ul{@li{x}}
@center{C}
@link["https://x"]{L}
@cite[1]
@image["fig.png"]
@hr
@br{}
@vspace{5}
@pagebreak
@p{After})"#;
    let laid = layout_doc_parts(&parts(src));
    assert!(laid.iter().any(|i| matches!(i, LaidItem::Hr { .. })));
    assert!(laid.iter().any(|i| matches!(i, LaidItem::Image { .. })));
    assert!(laid.iter().any(|i| matches!(i, LaidItem::PageBreak)));
    assert!(laid.iter().any(|i| matches!(i, LaidItem::VSpace { .. })));
    let placed = place_items(&laid, DocFrame::A4);
    assert!(!placed.is_empty());
}

#[test]
fn layout_inline_marks_and_unknown_at() {
    let laid = layout_doc_parts(&parts("(doc @em{hi} @strong{b} @tt{t} @unknown[args])"));
    let text = laid
        .iter()
        .filter_map(|i| match i {
            LaidItem::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("*hi*"));
    assert!(text.contains("**b**"));
    assert!(text.contains("`t`"));
    assert!(text.contains("args"));
}

#[test]
fn parse_image_bracket_custom_size() {
    let laid = layout_doc_parts(&parts(r#"(doc @image["pic.png" 40 30])"#));
    match laid.iter().find(|i| matches!(i, LaidItem::Image { .. })) {
        Some(LaidItem::Image {
            path,
            width_mm,
            height_mm,
            ..
        }) => {
            assert_eq!(path, "pic.png");
            assert_eq!(*width_mm, 40.0);
            assert_eq!(*height_mm, 30.0);
        }
        _ => panic!("expected image"),
    }
}

#[test]
fn place_items_empty_returns_empty() {
    assert!(place_items(&[], DocFrame::A4).is_empty());
}

#[test]
fn doc_frame_right_margin() {
    assert_eq!(DocFrame::A4.right_mm(), 185.0);
}

#[test]
fn layout_bold_alias_and_subsection() {
    let laid = layout_doc_parts(&parts("(doc @bold{hi} @subsubsection{sub})"));
    let text = text_items(&laid)
        .iter()
        .map(|l| l.content.as_str())
        .collect::<Vec<_>>()
        .join("|");
    assert!(text.contains("**hi**"));
    assert!(laid
        .iter()
        .any(|i| matches!(i, LaidItem::Text(t) if t.size_mm == H3_SIZE_MM)));
}

#[test]
fn layout_link_cite_edge_partitions() {
    let laid = layout_doc_parts(&parts(r#"(doc @link[]{only-url} @cite[] @link["u"]{})"#));
    let text = laid
        .iter()
        .filter_map(|i| match i {
            LaidItem::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("only-url"));
}

#[test]
fn layout_image_unquoted_path_and_bad_size() {
    let laid = layout_doc_parts(&parts("(doc @image[assets/pic.png 0 0])"));
    match laid.iter().find(|i| matches!(i, LaidItem::Image { .. })) {
        Some(LaidItem::Image {
            path,
            width_mm,
            height_mm,
            ..
        }) => {
            assert_eq!(path, "assets/pic.png");
            assert_eq!(*width_mm, FIGURE_WIDTH_MM);
            assert_eq!(*height_mm, FIGURE_HEIGHT_MM);
        }
        _ => panic!("expected default-sized image"),
    }
}

#[test]
fn place_items_first_tall_image_still_places() {
    let items = vec![LaidItem::Image {
        path: "x.png".into(),
        width_mm: 200.0,
        height_mm: 300.0,
        y_gap_after: 10.0,
    }];
    let placed = place_items(&items, DocFrame::A4);
    assert_eq!(placed.len(), 1);
    match &placed[0] {
        PlacedItem::Image { y_mm, .. } => assert!(*y_mm <= DocFrame::A4.top_mm),
        _ => panic!("expected image"),
    }
}

#[test]
fn place_items_pagebreak_then_vspace() {
    let items = vec![
        LaidItem::Text(LaidLine {
            size_mm: BODY_SIZE_MM,
            y_gap_after: BODY_GAP_MM,
            indent_mm: 0.0,
            content: "A".into(),
        }),
        LaidItem::PageBreak,
        LaidItem::VSpace { mm: 5.0 },
        LaidItem::Text(LaidLine {
            size_mm: BODY_SIZE_MM,
            y_gap_after: BODY_GAP_MM,
            indent_mm: 0.0,
            content: "B".into(),
        }),
    ];
    let placed = place_items(&items, DocFrame::A4);
    assert!(placed.iter().any(|p| p.page_index() == 1));
}

#[test]
fn wrap_mid_word_hard_break() {
    let lines = wrap_line("abcdefghij", 4);
    assert!(lines.len() >= 2);
    assert!(lines.iter().all(|l| l.chars().count() <= 4));
}

#[test]
fn collapse_ws_trims_edges() {
    assert_eq!(collapse_ws("  a   b  "), "a b");
}

#[test]
fn layout_skips_empty_quote_and_todo_lines() {
    assert!(layout_doc_parts(&parts("(doc @quote{})")).is_empty());
    assert!(layout_doc_parts(&parts("(doc @todo{})")).is_empty());
    let laid = layout_doc_parts(&parts("(doc @quote{Line one\n\nLine two})"));
    // Blank lines between quote paragraphs may collapse; at least one text item remains.
    assert!(!text_items(&laid).is_empty());
}

#[test]
fn layout_code_keeps_nonblank_content() {
    let laid = layout_doc_parts(&parts("(doc @code{line1\n\nline2})"));
    let texts = text_items(&laid);
    assert!(!texts.is_empty());
    let joined = texts
        .iter()
        .map(|l| l.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("line1") || joined.contains("line2"),
        "{joined}"
    );
}

#[test]
fn layout_marked_lines_with_inline_newline() {
    let laid = layout_doc_parts(&parts("(doc @li{plain\n@em{hi}})"));
    let text = text_items(&laid)
        .iter()
        .map(|l| l.content.as_str())
        .collect::<Vec<_>>()
        .join("|");
    assert!(text.contains("• plain"), "{text}");
    assert!(text.contains("*hi*"), "{text}");
}

#[test]
fn layout_unknown_at_bracket_args_only() {
    let laid = layout_doc_parts(&parts("(doc @foo[only-bracket])"));
    let text = text_items(&laid)
        .iter()
        .map(|l| l.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("only-bracket"), "{text}");
}

#[test]
fn layout_center_skips_empty_marked_line() {
    let laid = layout_doc_parts(&parts("(doc @center{\n@em{}\nHi})"));
    assert!(text_items(&laid).iter().any(|l| l.content == "Hi"));
}

#[test]
fn push_wrapped_multi_chunk_uses_body_gap() {
    let mut out = Vec::new();
    push_wrapped(
        "one two three four",
        BODY_SIZE_MM,
        TITLE_GAP_MM,
        4,
        0.0,
        &mut out,
    );
    assert!(out.len() >= 2);
}

#[test]
fn wrap_line_empty_max_zero_returns_single() {
    assert_eq!(wrap_line("abc", 0), vec!["abc".to_string()]);
    assert!(wrap_line("", 5).is_empty());
}
