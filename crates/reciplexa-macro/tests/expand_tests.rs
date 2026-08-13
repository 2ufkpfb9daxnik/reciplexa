use reciplexa_macro::*;
use reciplexa_syntax::parse_source;

#[test]
fn expands_color_byte_to_rgb() {
    let src = "(page a4 (circle 1 2 3 (color-byte 255 0 0)))";
    let out = expand_source(src).unwrap();
    assert!(out.contains("(rgb 1 0 0)"));
    assert!(!out.contains("color-byte"));
    // Surrounding form preserved.
    assert!(out.contains("(page a4 (circle 1 2 3 "));
}

#[test]
fn expands_hline_to_line() {
    let src = "(page a4 (hline 10 100 50 red 1.5))";
    let out = expand_source(src).unwrap();
    assert!(out.contains("(line 10 50 100 50 red 1.5)"));
    assert!(!out.contains("hline"));
}

#[test]
fn expands_vline_to_line() {
    let src = "(page a4 (vline 10 100 50 blue))";
    let out = expand_source(src).unwrap();
    assert!(out.contains("(line 50 10 50 100 blue)"));
    assert!(!out.contains("vline"));
}

#[test]
fn expands_rule_to_hline() {
    let src = "(page a4 (rule 200 (gray 0.3) 0.5))";
    let out = expand_source(src).unwrap();
    // gray expands first, then rule → hline → line
    assert!(out.contains("(line 20 "));
    assert!(out.contains(" 190 "));
    assert!(!out.contains("rule"));
    assert!(!out.contains("hline"));
}

#[test]
fn expands_square_to_rect() {
    let src = "(page a4 (square 10 20 30 red))";
    let out = expand_source(src).unwrap();
    assert_eq!(out, "(page a4 (rect 10 20 30 30 red))");
}

#[test]
fn expands_gray_to_rgb() {
    let src = "(page a4 (circle 1 2 3 (gray 0.25)))";
    let out = expand_source(src).unwrap();
    assert!(out.contains("(rgb 0.25 0.25 0.25)"));
    assert!(!out.contains("gray"));
}

#[test]
fn identity_when_no_macros() {
    let src = "(page a4 (circle 1 2 3 red))";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn parse_error_surfaces() {
    assert!(expand_source("(page").is_err());
}

#[test]
fn roundtrip_unparse_still_works_on_expanded() {
    use reciplexa_syntax::unparse;
    let out = expand_source("(circle 0 0 1 (color-byte 0 128 255))").unwrap();
    let root = parse_source(&out).into_result().unwrap();
    assert_eq!(unparse(&root), out);
}

#[test]
fn expands_plain_doc_to_page_text() {
    let out = expand_source("(markup Hello.)").unwrap();
    assert!(out.contains("(import graphics"));
    assert!(out.contains("(text 25 270 8 \"Hello.\")"));
    assert!(!out.contains("(markup "));
}

#[test]
fn expands_doc_with_em_markers() {
    let out = expand_source("(markup Hello @em{世界}.)").unwrap();
    assert!(out.contains("Hello *世界*."));
    assert!(out.contains("(import graphics"));
    assert!(out.contains("(text "));
}

#[test]
fn empty_doc_becomes_empty_page() {
    let out = expand_source("(markup)").unwrap();
    assert_eq!(
        out,
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn expands_multiline_doc_to_stacked_text() {
    let out = expand_source("(markup\nFirst\nSecond\n)").unwrap();
    assert!(out.contains("(text 25 270 8 \"First\")"));
    assert!(out.contains("(text 25 258 8 \"Second\")"));
}

// --- @title / @p package meaning (M8) — validity ---

#[test]
fn expands_title_at_larger_size() {
    let out = expand_source("(markup @title{Hello})").unwrap();
    assert!(
        out.contains("(text 25 270 14 \"Hello\")"),
        "title should use size 14: {out}"
    );
    assert!(!out.contains("(markup "));
}

#[test]
fn expands_h1_as_title_alias() {
    let out = expand_source("(markup @h1{Hello})").unwrap();
    assert!(
        out.contains("(text 25 270 14 \"Hello\")"),
        "h1 should match title size: {out}"
    );
}

#[test]
fn expands_title_then_paragraph_with_gap() {
    let out = expand_source("(markup @title{Report}\n@p{Body text})").unwrap();
    assert!(out.contains("(text 25 270 14 \"Report\")"), "{out}");
    // title gap 18 → next baseline at 270 - 18 = 252
    assert!(
        out.contains("(text 25 252 8 \"Body text\")"),
        "body after title gap: {out}"
    );
}

#[test]
fn expands_em_with_asterisk_markers() {
    let out = expand_source("(markup Hello @em{世界}.)").unwrap();
    assert!(out.contains("(text 25 270 8 \"Hello *世界*.\")"), "{out}");
}

#[test]
fn expands_italic_as_em_alias() {
    let out = expand_source("(markup Hello @italic{世界}.)").unwrap();
    assert!(out.contains("(text 25 270 8 \"Hello *世界*.\")"), "{out}");
}

#[test]
fn expands_strong_with_double_asterisks() {
    let out = expand_source("(markup Go @strong{fast}.)").unwrap();
    assert!(out.contains("(text 25 270 8 \"Go **fast**.\")"), "{out}");
}

#[test]
fn expands_warn_prefix_and_indent() {
    let out = expand_source("(markup @warn{Hot surface.})").unwrap();
    assert!(
        out.contains("(text 35 270 7 \"Warning: Hot surface.\")"),
        "{out}"
    );
}

#[test]
fn empty_em_strong_warn_skip() {
    assert_eq!(
        expand_source("(markup @em{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
    assert_eq!(
        expand_source("(markup @strong{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
    assert_eq!(
        expand_source("(markup @warn{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn expands_tt_with_backticks() {
    let out = expand_source("(markup Use @tt{cargo test}.)").unwrap();
    assert!(
        out.contains("(text 25 270 8 \"Use `cargo test`.\")"),
        "{out}"
    );
}

#[test]
fn expands_center_indents_short_line() {
    let out = expand_source("(markup @center{Hi})").unwrap();
    // ~2 chars → large indent off left 25; must not stay at 25
    assert!(
        !out.contains("(text 25 270 8 \"Hi\")"),
        "should be centered: {out}"
    );
    assert!(out.contains("\"Hi\")"), "{out}");
    assert!(out.contains("(text "), "{out}");
}

#[test]
fn empty_tt_center_skip() {
    assert_eq!(
        expand_source("(markup @tt{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
    assert_eq!(
        expand_source("(markup @center{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn expands_caption_indented_smaller() {
    let out = expand_source("(markup @caption{Fig. 1 A circle})").unwrap();
    assert!(out.contains("(text 33 270 6 \"Fig. 1 A circle\")"), "{out}");
}

#[test]
fn expands_image_emits_image_shape() {
    let out = expand_source(r#"(markup @image["figures/demo.png"])"#).unwrap();
    // Default 80×50; top at 270 → bottom y = 220
    assert!(
        out.contains(r#"(image "figures/demo.png" 25 220 80 50)"#),
        "{out}"
    );
}

#[test]
fn expands_figure_as_image_alias() {
    let out = expand_source(r#"(markup @figure["figures/demo.png"]{Cap})"#).unwrap();
    assert!(
        out.contains(r#"(image "figures/demo.png" 25 220 80 50)"#),
        "{out}"
    );
    assert!(out.contains("\"Cap\")"), "{out}");
}

#[test]
fn expands_image_custom_size_in_brackets() {
    let out = expand_source(r#"(markup @image["figures/demo.png" 100 40])"#).unwrap();
    assert!(
        out.contains(r#"(image "figures/demo.png" 25 230 100 40)"#),
        "{out}"
    );
}

#[test]
fn expands_image_with_caption_brace() {
    let out = expand_source(r#"(markup @image["figures/demo.png"]{Demo shot})"#).unwrap();
    assert!(
        out.contains(r#"(image "figures/demo.png" 25 220 80 50)"#),
        "{out}"
    );
    // After image height 50 + pad 8 → caption at 212
    assert!(
        out.contains("(text 33 212 6 \"Demo shot\")"),
        "caption under image: {out}"
    );
}

#[test]
fn empty_image_path_skips() {
    assert_eq!(
        expand_source("(markup @image[])").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
    assert_eq!(
        expand_source(r#"(markup @image[""])"#).unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn bad_image_size_falls_back_to_default() {
    let out = expand_source(r#"(markup @image["figures/demo.png" -1 40])"#).unwrap();
    assert!(
        out.contains(r#"(image "figures/demo.png" 25 220 80 50)"#),
        "non-positive size should use defaults: {out}"
    );
}

#[test]
fn expands_br_adds_gap_between_paragraphs() {
    let out = expand_source("(markup @p{Above}\n@br{}\n@p{Below})").unwrap();
    assert!(out.contains("(text 25 270 8 \"Above\")"), "{out}");
    // body gap 12 after Above, then +12 br → Below at 246
    assert!(
        out.contains("(text 25 246 8 \"Below\")"),
        "br should insert a blank gap: {out}"
    );
}

#[test]
fn expands_link_appends_url_in_parens() {
    let out = expand_source(r#"(markup See @link["https://example.com"]{docs}.)"#).unwrap();
    assert!(out.contains("See docs (https://example.com)."), "{out}");
}

#[test]
fn expands_cite_as_bracketed_key() {
    let out = expand_source(r#"(markup Cite @cite[42].)"#).unwrap();
    assert!(out.contains("Cite [42]."), "{out}");
}

#[test]
fn empty_caption_skips() {
    assert_eq!(
        expand_source("(markup @caption{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn empty_link_and_cite_are_skipped() {
    assert_eq!(
        expand_source("(markup @link[]{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
    assert_eq!(
        expand_source("(markup @cite[])").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

// --- defect ---

#[test]
fn empty_title_brace_skips_empty_text() {
    let out = expand_source("(markup @title{})").unwrap();
    assert_eq!(
        out, "(import graphics/page only a4 page)\n(val main (page a4 (list)))",
        "empty title must not emit empty text: {out}"
    );
}

#[test]
fn bare_title_without_brace_does_not_panic() {
    let out = expand_source("(markup @title)").unwrap();
    assert!(
        out.starts_with("(import graphics/page only a4 page)\n(val main (page a4 (list)))"),
        "{out}"
    );
    assert!(!out.contains("(text "), "bare @title has no content: {out}");
}

#[test]
fn unknown_at_form_identity_flattens() {
    let out = expand_source("(markup see @foo{bar} end)").unwrap();
    assert!(out.contains("(text 25 270 8 \"see bar end\")"), "{out}");
}

#[test]
fn expands_wrapped_long_paragraph() {
    let long = "word ".repeat(12);
    let long = long.trim();
    assert!(long.len() > 40);
    let src = format!("(markup @p{{{long}}})");
    let out = expand_source(&src).unwrap();
    let text_count = out.matches("(text ").count();
    assert!(
        text_count >= 2,
        "long paragraph should wrap to multiple text shapes: {out}"
    );
}

// --- @h2 package meaning ---

#[test]
fn expands_h2_between_title_and_body() {
    let out = expand_source("(markup @title{T}\n@h2{Section}\n@p{Body})").unwrap();
    assert!(out.contains("(text 25 270 14 \"T\")"), "{out}");
    // title gap 18 → 252; h2 size 11
    assert!(
        out.contains("(text 25 252 11 \"Section\")"),
        "h2 after title: {out}"
    );
    // h2 gap 14 → 238
    assert!(
        out.contains("(text 25 238 8 \"Body\")"),
        "body after h2: {out}"
    );
}

#[test]
fn expands_section_as_h2_alias() {
    let out = expand_source("(markup @section{Intro})").unwrap();
    assert!(out.contains("(text 25 270 11 \"Intro\")"), "{out}");
}

#[test]
fn expands_h3_between_h2_sizes() {
    let out = expand_source("(markup @h3{Detail})").unwrap();
    assert!(out.contains("(text 25 270 9 \"Detail\")"), "{out}");
}

#[test]
fn expands_subsubsection_as_h3_alias() {
    let out = expand_source("(markup @subsubsection{Detail})").unwrap();
    assert!(out.contains("(text 25 270 9 \"Detail\")"), "{out}");
}

#[test]
fn empty_h2_skips_empty_text() {
    let out = expand_source("(markup @h2{})").unwrap();
    assert_eq!(
        out,
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

// --- @li list items ---

#[test]
fn expands_li_with_bullet_prefix() {
    let out = expand_source("(markup @li{First}\n@li{Second})").unwrap();
    assert!(
        out.contains("(text 25 270 8 \"• First\")"),
        "first li: {out}"
    );
    assert!(
        out.contains("(text 25 258 8 \"• Second\")"),
        "second li: {out}"
    );
}

#[test]
fn expands_item_as_li_alias() {
    let out = expand_source("(markup @item{Alpha}\n@item{Beta})").unwrap();
    assert!(
        out.contains("(text 25 270 8 \"• Alpha\")"),
        "first item: {out}"
    );
    assert!(
        out.contains("(text 25 258 8 \"• Beta\")"),
        "second item: {out}"
    );
}

#[test]
fn empty_li_skips_empty_text() {
    let out = expand_source("(markup @li{})").unwrap();
    assert_eq!(
        out,
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn empty_item_skips() {
    assert_eq!(
        expand_source("(markup @item{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn long_doc_spills_onto_second_page() {
    // Body gap 12mm from y=270 down; bottom margin 25 → room for many lines.
    let mut body = String::from("(markup");
    for i in 0..30 {
        body.push_str(&format!("\n@p{{line{i}}}"));
    }
    body.push(')');
    let out = expand_source(&body).unwrap();
    let page_count = out.matches("(page a4").count();
    assert!(
        page_count >= 2,
        "expected multipage layout, got {page_count} page(s): {out}"
    );
}

// --- @quote ---

#[test]
fn expands_quote_indented_smaller() {
    let out = expand_source("(markup @quote{Cited line})").unwrap();
    // Indented left (35) and size 7.
    assert!(out.contains("(text 35 270 7 \"Cited line\")"), "{out}");
}

#[test]
fn expands_note_prefix_and_indent() {
    let out = expand_source("(markup @note{Watch the margins.})").unwrap();
    assert!(
        out.contains("(text 35 270 7 \"Note: Watch the margins.\")"),
        "{out}"
    );
}

#[test]
fn empty_note_skips() {
    assert_eq!(
        expand_source("(markup @note{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn empty_quote_skips() {
    assert_eq!(
        expand_source("(markup @quote{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

// --- numbered list: @ol{ item; item } uses `;` separators inside one brace ---

#[test]
fn expands_ol_numbers_items() {
    let out = expand_source("(markup @ol{Alpha; Beta; Gamma})").unwrap();
    assert!(out.contains("(text 25 270 8 \"1. Alpha\")"), "{out}");
    assert!(out.contains("(text 25 258 8 \"2. Beta\")"), "{out}");
    assert!(out.contains("(text 25 246 8 \"3. Gamma\")"), "{out}");
}

#[test]
fn expands_ul_bullets_semicolon_items() {
    let out = expand_source("(markup @ul{Alpha; Beta; Gamma})").unwrap();
    assert!(out.contains("(text 25 270 8 \"• Alpha\")"), "{out}");
    assert!(out.contains("(text 25 258 8 \"• Beta\")"), "{out}");
    assert!(out.contains("(text 25 246 8 \"• Gamma\")"), "{out}");
}

#[test]
fn expands_ul_runs_inline_marks_inside_items() {
    let out = expand_source("(markup @ul{plain; @em{hi}})").unwrap();
    assert!(out.contains("• plain"), "{out}");
    assert!(
        out.contains("• *hi*"),
        "inline @em inside @ul should mark: {out}"
    );
}

#[test]
fn expands_blockquote_as_quote_alias() {
    let out = expand_source("(markup @blockquote{Cited})").unwrap();
    assert!(out.contains("(text 35 270 7 \"Cited\")"), "{out}");
}

#[test]
fn expands_todo_prefix_callout() {
    let out = expand_source("(markup @todo{Ship it.})").unwrap();
    assert!(out.contains("(text 35 270 7 \"TODO: Ship it.\")"), "{out}");
}

#[test]
fn empty_ol_skips() {
    assert_eq!(
        expand_source("(markup @ol{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn empty_ul_skips() {
    assert_eq!(
        expand_source("(markup @ul{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn ol_trims_blank_segments() {
    let out = expand_source("(markup @ol{A;; B})").unwrap();
    assert!(out.contains("1. A"), "{out}");
    assert!(out.contains("2. B"), "{out}");
    assert!(!out.contains("3."), "{out}");
}

#[test]
fn ul_trims_blank_segments() {
    let out = expand_source("(markup @ul{A;; B})").unwrap();
    assert!(out.contains("• A"), "{out}");
    assert!(out.contains("• B"), "{out}");
}

// --- @vspace / @hr ---

#[test]
fn expands_vspace_shifts_following_text() {
    let out = expand_source("(markup @p{Above}\n@vspace{20}\n@p{Below})").unwrap();
    assert!(out.contains("(text 25 270 8 \"Above\")"), "{out}");
    // body gap 12 after Above, then +20 vspace → Below at 270 - 12 - 20 = 238
    assert!(
        out.contains("(text 25 238 8 \"Below\")"),
        "vspace should push following line: {out}"
    );
}

#[test]
fn expands_hr_emits_line_shape() {
    let out = expand_source("(markup @p{Head}\n@hr{}\n@p{Tail})").unwrap();
    assert!(out.contains("(text 25 270 8 \"Head\")"), "{out}");
    // After Head (gap 12): y=258 for the rule; A4 content width 25..185
    assert!(
        out.contains("(stroke (line 25 258 185 258) 0.4 black)"),
        "hr should emit a stroked line: {out}"
    );
    assert!(out.contains("(text 25 246 8 \"Tail\")"), "{out}");
}

#[test]
fn bad_vspace_payload_is_skipped() {
    let out = expand_source("(markup @p{A}\n@vspace{nope}\n@p{B})").unwrap();
    // Invalid vspace ignored → normal body gap 12 only.
    assert!(out.contains("(text 25 270 8 \"A\")"), "{out}");
    assert!(out.contains("(text 25 258 8 \"B\")"), "{out}");
}

// --- @pagebreak ---

#[test]
fn expands_pagebreak_starts_second_page() {
    let out = expand_source(
        "(markup @p{One}
@pagebreak{}
@p{Two})",
    )
    .unwrap();
    assert_eq!(out.matches("(page a4").count(), 2, "{out}");
    assert!(out.contains("(text 25 270 8 \"One\")"), "{out}");
    assert!(
        out.contains("(text 25 270 8 \"Two\")"),
        "Two should start at top of page 2: {out}"
    );
    assert!(
        out.contains("(val main (list"),
        "multipage package list: {out}"
    );
}

#[test]
fn pagebreak_alone_yields_empty_pages_skipped() {
    // Only a break with no drawable items → still one empty page is ok,
    // or empty doc page; we accept a single empty page.
    let out = expand_source("(markup @pagebreak{})").unwrap();
    assert!(
        out.contains("(import graphics/page only a4 page)\n(val main (page a4 (list)))"),
        "{out}"
    );
}

// --- @code ---

#[test]
fn expands_code_indented_smaller() {
    let out = expand_source("(markup @code{let x = 1})").unwrap();
    // left 25 + indent 8 = 33; size 6.5
    assert!(out.contains("(text 33 270 6.5 \"let x = 1\")"), "{out}");
}

#[test]
fn expands_code_preserves_internal_spaces() {
    let out = expand_source("(markup @code{a  b})").unwrap();
    assert!(
        out.contains("(text 33 270 6.5 \"a  b\")"),
        "code must keep double spaces: {out}"
    );
}

#[test]
fn expands_pre_as_code_alias() {
    let out = expand_source("(markup @pre{let x = 1})").unwrap();
    assert!(out.contains("(text 33 270 6.5 \"let x = 1\")"), "{out}");
}

#[test]
fn empty_code_skips() {
    assert_eq!(
        expand_source("(markup @code{})").unwrap(),
        "(import graphics/page only a4 page)\n(val main (page a4 (list)))"
    );
}

#[test]
fn format_frac_strips_trailing_zeros() {
    assert_eq!(format_frac(1.5), "1.5");
    assert_eq!(format_frac(2.0), "2");
}

#[test]
fn escape_lisp_string_uses_multi_quote_for_specials() {
    let src = "a\nb\tc\\d\"e";
    let lit = escape_lisp_string(src);
    assert!(lit.starts_with("\"\"\""));
    assert_eq!(reciplexa_syntax::decode_string_literal(&lit).unwrap(), src);
}

#[test]
fn color_byte_out_of_range_not_expanded() {
    let src = "(page a4 (circle 0 0 1 (color-byte 300 0 0)))";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn gray_out_of_range_not_expanded() {
    let src = "(page a4 (circle 0 0 1 (gray 2)))";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn short_axis_line_forms_not_expanded() {
    let src = "(page a4 (hline 1 2))";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn square_too_short_not_expanded() {
    let src = "(page a4 (square 1 2))";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn expand_error_new() {
    let err = ExpandError::new("x");
    assert_eq!(err.message, "x");
}

#[test]
fn macro_expansion_convergence_guard() {
    // color-byte always rewrites when in range; bounded loop should still finish.
    let out = expand_source("(color-byte 1 2 3)").unwrap();
    assert!(out.contains("(rgb"));
}

#[test]
fn format_frac_trims_fractional_zeros() {
    assert_eq!(format_frac(1.10), "1.1");
    assert_eq!(format_frac(0.125), "0.125");
}

#[test]
fn color_byte_wrong_arity_not_expanded() {
    let src = "(page a4 (circle 0 0 1 (color-byte 1 2)))";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn rule_minimal_y_only_expands() {
    let out = expand_source("(page a4 (rule 200))").unwrap();
    assert!(out.contains("(line 20 200 190 200)"), "{out}");
}

#[test]
fn vline_with_extra_color_expands() {
    let out = expand_source("(page a4 (vline 10 90 50 green 2))").unwrap();
    assert!(out.contains("(line 50 10 50 90 green 2)"), "{out}");
}

#[test]
fn square_with_fill_preserves_tail() {
    let out = expand_source("(page a4 (square 1 2 3 red))").unwrap();
    assert_eq!(out, "(page a4 (rect 1 2 3 3 red))");
}

#[test]
fn atom_text_node_child_in_square() {
    let out = expand_source("(page a4 (square 1 2 (rgb 1 0 0)))").unwrap();
    assert!(out.contains("(rect 1 2 (rgb 1 0 0) (rgb 1 0 0))"), "{out}");
}

#[test]
fn find_skip_empty_lists_and_non_token_heads() {
    // Empty / nested-head lists exercise `items.first()` continue arms across find_*.
    let src = "{brace} () ((nested)) (page a4 (circle 1 2 3 red))";
    let out = expand_source(src).unwrap();
    assert!(out.contains("(circle 1 2 3 red)"), "{out}");
    assert!(!out.contains("color-byte"));
}

#[test]
fn gray_wrong_arity_not_expanded() {
    let src = "(page a4 (circle 0 0 1 (gray 0.5 0.5)))";
    assert_eq!(expand_source(src).unwrap(), src);
    let bare = "(page a4 (gray))";
    assert_eq!(expand_source(bare).unwrap(), bare);
}

#[test]
fn rule_too_short_not_expanded() {
    let src = "(page a4 (rule))";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn color_byte_non_number_not_expanded() {
    let src = "(page a4 (circle 0 0 1 (color-byte foo 0 0)))";
    assert_eq!(expand_source(src).unwrap(), src);
    let nested = "(page a4 (gray (rgb 1 0 0)))";
    assert_eq!(expand_source(nested).unwrap(), nested);
    // Fail on later channels after the first number parses.
    assert_eq!(
        expand_source("(page a4 (color-byte 1 foo 0))").unwrap(),
        "(page a4 (color-byte 1 foo 0))"
    );
    assert_eq!(
        expand_source("(page a4 (color-byte 1 2 foo))").unwrap(),
        "(page a4 (color-byte 1 2 foo))"
    );
}

#[test]
fn markup_parts_error_skips_rewrite() {
    // Parses, but markup_parts rejects `@()` → find_markup yields no rewrite.
    let src = "(markup @())";
    assert_eq!(expand_source(src).unwrap(), src);
}

#[test]
fn markup_skip_non_markup_and_empty_list_siblings() {
    let out = expand_source("() ((x)) (markup Hi)").unwrap();
    assert!(out.contains("(text 25 270 8 \"Hi\")"), "{out}");
}

#[test]
fn format_frac_strips_fixed_precision_zeros() {
    assert_eq!(format_frac(1.5), "1.5");
    assert_eq!(format_frac(2.0), "2");
    assert_eq!(format_frac(0.1), "0.1");
}
