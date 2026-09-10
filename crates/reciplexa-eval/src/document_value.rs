//! N5.1b — lower package `document/page` (`doc-*`) eval records to scene documents.
//!
//! Flow-oriented constructors from `packages/document` become a simple text layout on
//! paper via shared std live-layout helpers (`break_line` / `place_lines` / indent /
//! columns). Interim CST `(page)/(circle)` keyword tables stay untouched.

use reciplexa_scene::{Color, Document, Frame, Line, Page, PaperSize, Rect, Shape, Text};
use reciplexa_std::japanese::{
    layout_column_paragraph_shapes, layout_wrapped_paragraph_shapes, ParagraphSceneLayout,
    DOC_TEXT_MAX_EM,
};
use reciplexa_text_layout::{
    host_product_font, layout_column_paragraph_product, layout_wrapped_paragraph_product_lines,
    positioned_lines_to_shapes, TypesetEngine,
};

use crate::graphics_value::GraphicsValueError;
use crate::value::RuntimeValue;

/// Soft-wrap + place pitch (mm): negative so baselines step down the page.
const DOC_LINE_PITCH_EXTRA_MM: f64 = 3.0;
const DOC_BASE_X_MM: f64 = 20.0;
const FLOW_BODY_SIZE_MM: f64 = 4.0;
const LIST_INDENT_MM: f64 = 8.0;
const NOTE_INDENT_MM: f64 = 8.0;
const NOTE_SIZE_MM: f64 = 3.5;
const CAPTION_SIZE_MM: f64 = 3.5;
const TABLE_CELL_SIZE_MM: f64 = 3.5;
const TABLE_PAD_X_MM: f64 = 2.0;
const TABLE_PAD_Y_MM: f64 = 2.0;
const TABLE_STROKE_MM: f64 = 0.35;
const FIGURE_HEIGHT_MM: f64 = 22.0;
const FIGURE_CAPTION_GAP_MM: f64 = 3.0;

/// Lower a `tag: "doc-page"` package value to a scene [`Document`] (live layout).
///
/// Uses std `break_line` + `place_lines_horizontal` + `indent_first_line` +
/// `measure_columns` heuristics — not production JLReq layout.
pub fn layout_doc_page_to_scene(v: &RuntimeValue) -> Result<Document, GraphicsValueError> {
    layout_doc_page_to_scene_with_engine(v, TypesetEngine::Stub)
}

/// Lower a `doc-page` with an explicit typeset engine.
///
/// [`TypesetEngine::Product`] uses the font-backed JLReq engine and the host
/// product face (fixture unless `RECIPLEXA_CJK_FONT` is set).
pub fn layout_doc_page_to_scene_with_engine(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Document, GraphicsValueError> {
    let pages = pages_from_doc_value_with_engine(v, engine)?;
    Ok(Document { pages })
}

/// Lower a `tag: "doc-page"` package value to a scene [`Document`].
///
/// Thin alias of [`layout_doc_page_to_scene`] for the graphics bridge.
pub fn document_from_doc_value(v: &RuntimeValue) -> Result<Document, GraphicsValueError> {
    layout_doc_page_to_scene(v)
}

/// Lower a `doc-page` record to a scene [`Page`] with a naive top-down text layout.
pub fn page_from_doc_value(v: &RuntimeValue) -> Result<Page, GraphicsValueError> {
    let pages = pages_from_doc_value_with_engine(v, TypesetEngine::Stub)?;
    pages
        .into_iter()
        .next()
        .ok_or_else(|| GraphicsValueError::new("doc-page produced no pages"))
}

/// Default 版面 matching historical `DOC_BASE_X_MM` / top inset (not content-width wrap).
const DEFAULT_HANMEN: DocHanmen = DocHanmen {
    top_mm: 25.0,
    right_mm: 20.0,
    bottom_mm: 25.0,
    left_mm: DOC_BASE_X_MM,
    explicit: false,
};

#[derive(Debug, Clone, Copy)]
struct DocHanmen {
    top_mm: f64,
    right_mm: f64,
    #[allow(dead_code)]
    bottom_mm: f64,
    left_mm: f64,
    /// When true, wrap to paper width minus left/right (preview and export share it).
    explicit: bool,
}

struct DocLayout {
    paper: PaperSize,
    hanmen: DocHanmen,
    engine: TypesetEngine,
    pages: Vec<Page>,
    shapes: Vec<Shape>,
    cursor_y: f64,
}

impl DocLayout {
    fn new(paper: PaperSize, hanmen: DocHanmen, engine: TypesetEngine) -> Self {
        let cursor_y = paper.height_mm - hanmen.top_mm;
        Self {
            paper,
            hanmen,
            engine,
            pages: Vec::new(),
            shapes: Vec::new(),
            cursor_y,
        }
    }

    fn start_y(&self) -> f64 {
        self.paper.height_mm - self.hanmen.top_mm
    }

    fn wrap_em(&self, size_mm: f64) -> f64 {
        self.wrap_em_at(size_mm, 0.0)
    }

    fn wrap_em_at(&self, size_mm: f64, extra_left_mm: f64) -> f64 {
        if self.hanmen.explicit {
            let w =
                self.paper.width_mm - self.hanmen.left_mm - extra_left_mm - self.hanmen.right_mm;
            (w / size_mm.max(0.01)).max(1.0)
        } else {
            let w = DOC_TEXT_MAX_EM * size_mm - extra_left_mm;
            (w / size_mm.max(0.01)).max(1.0)
        }
    }

    fn content_width_mm(&self) -> f64 {
        if self.hanmen.explicit {
            (self.paper.width_mm - self.hanmen.left_mm - self.hanmen.right_mm).max(1.0)
        } else {
            (DOC_TEXT_MAX_EM * FLOW_BODY_SIZE_MM).max(1.0)
        }
    }

    fn pagebreak(&mut self) {
        if !self.shapes.is_empty() {
            self.flush_page();
        }
        self.cursor_y = self.start_y();
    }

    fn flush_page(&mut self) {
        self.pages.push(Page {
            paper: self.paper,
            shapes: std::mem::take(&mut self.shapes),
        });
    }

    fn finish(mut self) -> Vec<Page> {
        if !self.shapes.is_empty() || self.pages.is_empty() {
            self.flush_page();
        }
        self.pages
    }
}

fn pages_from_doc_value_with_engine(
    v: &RuntimeValue,
    engine: TypesetEngine,
) -> Result<Vec<Page>, GraphicsValueError> {
    let fields = record_fields(v, "doc-page")?;
    expect_tag(fields, "doc-page")?;
    let paper_v =
        field(fields, "paper").ok_or_else(|| GraphicsValueError::new("doc-page missing paper"))?;
    let flow_v =
        field(fields, "flow").ok_or_else(|| GraphicsValueError::new("doc-page missing flow"))?;
    let paper = paper_from_size_value(paper_v)?;
    let hanmen = match field(fields, "margins") {
        Some(m) => hanmen_from_value(m)?,
        None => DEFAULT_HANMEN,
    };
    let mut layout = DocLayout::new(paper, hanmen, engine);
    collect_flow_shapes(flow_v, &mut layout)?;
    Ok(layout.finish())
}

fn hanmen_from_value(v: &RuntimeValue) -> Result<DocHanmen, GraphicsValueError> {
    let fields = record_fields(v, "doc-margins")?;
    expect_tag(fields, "doc-margins")?;
    let top_mm = field(fields, "top")
        .and_then(as_f64)
        .ok_or_else(|| GraphicsValueError::new("doc-margins missing top"))?;
    let right_mm = field(fields, "right")
        .and_then(as_f64)
        .ok_or_else(|| GraphicsValueError::new("doc-margins missing right"))?;
    let bottom_mm = field(fields, "bottom")
        .and_then(as_f64)
        .ok_or_else(|| GraphicsValueError::new("doc-margins missing bottom"))?;
    let left_mm = field(fields, "left")
        .and_then(as_f64)
        .ok_or_else(|| GraphicsValueError::new("doc-margins missing left"))?;
    if left_mm < 0.0 || right_mm < 0.0 || top_mm < 0.0 || bottom_mm < 0.0 {
        return Err(GraphicsValueError::new("doc-margins must be non-negative"));
    }
    Ok(DocHanmen {
        top_mm,
        right_mm,
        bottom_mm,
        left_mm,
        explicit: true,
    })
}

fn collect_flow_shapes(v: &RuntimeValue, layout: &mut DocLayout) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc-flow")?;
    expect_tag(fields, "doc-flow")?;
    let sections = field(fields, "sections")
        .ok_or_else(|| GraphicsValueError::new("doc-flow missing sections"))?;
    for section in cons_items(sections)? {
        collect_section_shapes(section, layout)?;
    }
    Ok(())
}

fn collect_section_shapes(
    v: &RuntimeValue,
    layout: &mut DocLayout,
) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc-section")?;
    expect_tag(fields, "doc-section")?;
    if let Some(title) = field(fields, "title") {
        push_heading_or_paragraph(title, layout)?;
    }
    let blocks = field(fields, "blocks")
        .ok_or_else(|| GraphicsValueError::new("doc-section missing blocks"))?;
    for block in cons_items(blocks)? {
        collect_block_shapes(block, layout)?;
    }
    Ok(())
}

fn collect_block_shapes(
    v: &RuntimeValue,
    layout: &mut DocLayout,
) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc-block")?;
    expect_tag(fields, "doc-block")?;
    let kind = match field(fields, "kind") {
        Some(RuntimeValue::String(s)) => s.as_str(),
        _ => {
            return Err(GraphicsValueError::new("doc-block missing kind string"));
        }
    };
    match kind {
        "heading" => {
            let heading = field(fields, "heading")
                .ok_or_else(|| GraphicsValueError::new("doc-block heading missing heading"))?;
            push_heading_or_paragraph(heading, layout)
        }
        "paragraph" => {
            let paragraph = field(fields, "paragraph")
                .ok_or_else(|| GraphicsValueError::new("doc-block paragraph missing paragraph"))?;
            push_heading_or_paragraph(paragraph, layout)
        }
        "columns" => {
            let columns = field(fields, "columns")
                .ok_or_else(|| GraphicsValueError::new("doc-block columns missing columns"))?;
            push_heading_or_paragraph(columns, layout)
        }
        "pagebreak" => {
            layout.pagebreak();
            Ok(())
        }
        "spacer" => {
            let spacer = field(fields, "spacer")
                .ok_or_else(|| GraphicsValueError::new("doc-block spacer missing spacer"))?;
            let spacer_fields = record_fields(spacer, "doc-spacer")?;
            expect_tag(spacer_fields, "doc-spacer")?;
            let len = field(spacer_fields, "length")
                .and_then(as_f64)
                .unwrap_or(4.0);
            layout.cursor_y -= len;
            Ok(())
        }
        "list" => {
            let list = field(fields, "list")
                .ok_or_else(|| GraphicsValueError::new("doc-block list missing list"))?;
            push_list(list, layout)
        }
        "table" => {
            let table = field(fields, "table")
                .ok_or_else(|| GraphicsValueError::new("doc-block table missing table"))?;
            push_table(table, layout)
        }
        "figure" => {
            let figure = field(fields, "figure")
                .ok_or_else(|| GraphicsValueError::new("doc-block figure missing figure"))?;
            push_figure(figure, layout)
        }
        "note" => {
            let note = field(fields, "note")
                .ok_or_else(|| GraphicsValueError::new("doc-block note missing note"))?;
            push_note(note, layout)
        }
        other => Err(GraphicsValueError::new(format!(
            "unsupported doc-block kind `{other}`"
        ))),
    }
}

fn push_heading_or_paragraph(
    v: &RuntimeValue,
    layout: &mut DocLayout,
) -> Result<(), GraphicsValueError> {
    let fields = record_fields(v, "doc text node")?;
    let tag = tag_of(fields).unwrap_or("");
    match tag {
        "doc-heading" => {
            let level = field(fields, "level").and_then(as_f64).unwrap_or(1.0);
            let text = string_field(fields, "text")?;
            let size = match level as i64 {
                1 => 8.0,
                2 => 6.5,
                _ => 5.5,
            };
            push_soft_wrapped_text(&text, size, 0.0, layout)
        }
        "doc-paragraph" => {
            let text = string_field(fields, "text")?;
            let size = 4.0;
            let indent_em = field(fields, "indent-em").and_then(as_f64).unwrap_or(0.0);
            push_soft_wrapped_text(&text, size, indent_em, layout)
        }
        "doc-columns" => push_doc_columns(fields, layout),
        other => Err(GraphicsValueError::new(format!(
            "expected doc-heading, doc-paragraph, or doc-columns, got `{other}`"
        ))),
    }
}

/// Place paragraph texts into columns via std [`layout_column_paragraph_shapes`].
fn push_doc_columns(
    fields: &[(String, RuntimeValue)],
    layout: &mut DocLayout,
) -> Result<(), GraphicsValueError> {
    let count = field(fields, "count")
        .and_then(as_f64)
        .unwrap_or(2.0)
        .max(0.0) as u32;
    let gutter_em = field(fields, "gutter-em").and_then(as_f64).unwrap_or(1.0);
    let total_em = field(fields, "total-em")
        .and_then(as_f64)
        .unwrap_or(DOC_TEXT_MAX_EM);
    let paragraphs = field(fields, "paragraphs")
        .ok_or_else(|| GraphicsValueError::new("doc-columns missing paragraphs"))?;
    let items = cons_items(paragraphs)?;
    let mut texts = Vec::new();
    for item in items.iter().take(count as usize) {
        let text = match item {
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Record(pf) => {
                expect_tag(pf, "doc-paragraph")?;
                string_field(pf, "text")?
            }
            _ => {
                return Err(GraphicsValueError::new(
                    "doc-columns paragraphs expect string or doc-paragraph",
                ));
            }
        };
        texts.push(text);
    }
    let size_mm = 4.0;
    let pitch = -(size_mm + DOC_LINE_PITCH_EXTRA_MM);
    let scene = ParagraphSceneLayout {
        base_x_mm: layout.hanmen.left_mm,
        start_y_mm: layout.cursor_y,
        size_mm,
        pitch_mm: pitch,
        fill: Color::BLACK,
    };
    let total_em = if layout.hanmen.explicit {
        layout.wrap_em(size_mm)
    } else {
        total_em
    };
    let min_y = match layout.engine {
        TypesetEngine::Stub => {
            let (t, min_y) =
                layout_column_paragraph_shapes(&texts, total_em, count, gutter_em, &scene);
            layout.shapes.extend(t.into_iter().map(Shape::Text));
            min_y
        }
        TypesetEngine::Product => {
            let font = host_product_font().map_err(|e| GraphicsValueError::new(e.to_string()))?;
            let (s, min_y) =
                layout_column_paragraph_product(&font, &texts, total_em, count, gutter_em, &scene)
                    .map_err(|e| GraphicsValueError::new(e.to_string()))?;
            layout.shapes.extend(s);
            min_y
        }
    };
    // After a block of lines, advance past the last baseline by one pitch step.
    layout.cursor_y = if texts.is_empty() {
        layout.cursor_y
    } else {
        min_y + pitch
    };
    Ok(())
}

/// Soft-wrap via std [`layout_wrapped_paragraph_shapes`] (break_line + place_lines + indent).
fn push_soft_wrapped_text(
    text: &str,
    size_mm: f64,
    indent_em: f64,
    layout: &mut DocLayout,
) -> Result<(), GraphicsValueError> {
    push_flow_text(text, size_mm, indent_em, 0.0, layout)
}

fn push_flow_text(
    text: &str,
    size_mm: f64,
    indent_em: f64,
    extra_left_mm: f64,
    layout: &mut DocLayout,
) -> Result<(), GraphicsValueError> {
    let pitch = -(size_mm + DOC_LINE_PITCH_EXTRA_MM);
    let wrap_em = layout.wrap_em_at(size_mm, extra_left_mm);
    let scene = ParagraphSceneLayout {
        base_x_mm: layout.hanmen.left_mm + extra_left_mm,
        start_y_mm: layout.cursor_y,
        size_mm,
        pitch_mm: pitch,
        fill: Color::BLACK,
    };
    let n_lines = emit_flow_text(text, indent_em, wrap_em, &scene, layout)?;
    layout.cursor_y += pitch * n_lines as f64;
    Ok(())
}

fn emit_flow_text(
    text: &str,
    indent_em: f64,
    wrap_em: f64,
    scene: &ParagraphSceneLayout,
    layout: &mut DocLayout,
) -> Result<usize, GraphicsValueError> {
    // Product emits one GlyphRun per cluster; pitch must step by wrapped
    // lines, not glyph count, or the next block falls off the page.
    match layout.engine {
        TypesetEngine::Stub => {
            let t = layout_wrapped_paragraph_shapes(text, wrap_em, indent_em, scene);
            let n = t.len().max(1);
            layout.shapes.extend(t.into_iter().map(Shape::Text));
            Ok(n)
        }
        TypesetEngine::Product => {
            let font = host_product_font().map_err(|e| GraphicsValueError::new(e.to_string()))?;
            let lines =
                layout_wrapped_paragraph_product_lines(&font, text, wrap_em, indent_em, scene)
                    .map_err(|e| GraphicsValueError::new(e.to_string()))?;
            if lines.is_empty() {
                layout.shapes.push(Shape::Text(Text {
                    x_mm: scene.base_x_mm,
                    y_mm: scene.start_y_mm,
                    size_mm: scene.size_mm,
                    width_mm: None,
                    height_mm: None,
                    content: String::new(),
                    fill: scene.fill,
                }));
                Ok(1)
            } else {
                layout
                    .shapes
                    .extend(positioned_lines_to_shapes(&lines, scene.fill));
                Ok(lines.len())
            }
        }
    }
}

fn push_note(note: &RuntimeValue, layout: &mut DocLayout) -> Result<(), GraphicsValueError> {
    let text = match note {
        RuntimeValue::Record(fields) => {
            expect_tag(fields, "doc-note")?;
            string_field(fields, "text")?
        }
        _ => atom_text(note)?,
    };
    let line = format!("Note: {text}");
    push_flow_text(&line, NOTE_SIZE_MM, 0.0, NOTE_INDENT_MM, layout)
}

fn push_list(list: &RuntimeValue, layout: &mut DocLayout) -> Result<(), GraphicsValueError> {
    let fields = record_fields(list, "doc-list")?;
    expect_tag(fields, "doc-list")?;
    let ordered = matches!(field(fields, "ordered"), Some(RuntimeValue::Bool(true)));
    let items =
        field(fields, "items").ok_or_else(|| GraphicsValueError::new("doc-list missing items"))?;
    for (i, item) in cons_items(items)?.into_iter().enumerate() {
        let marker = if ordered {
            format!("{}.", i + 1)
        } else {
            "-".to_string()
        };
        let texts = list_item_texts(item)?;
        let marker_y = layout.cursor_y;
        push_flow_text(&marker, FLOW_BODY_SIZE_MM, 0.0, 0.0, layout)?;
        let after_marker = layout.cursor_y;
        layout.cursor_y = marker_y;
        if texts.is_empty() {
            layout.cursor_y = after_marker;
            continue;
        }
        for t in texts {
            push_flow_text(&t, FLOW_BODY_SIZE_MM, 0.0, LIST_INDENT_MM, layout)?;
        }
        layout.cursor_y = layout.cursor_y.min(after_marker);
    }
    Ok(())
}

fn list_item_texts(item: &RuntimeValue) -> Result<Vec<String>, GraphicsValueError> {
    match item {
        RuntimeValue::String(s) => Ok(vec![s.clone()]),
        RuntimeValue::Record(fields) => match tag_of(fields).unwrap_or("") {
            "doc-list-item" => {
                let paragraphs = field(fields, "paragraphs")
                    .ok_or_else(|| GraphicsValueError::new("doc-list-item missing paragraphs"))?;
                texts_from_flow_value(paragraphs)
            }
            "doc-paragraph" => Ok(vec![string_field(fields, "text")?]),
            _ => Ok(vec![atom_text(item)?]),
        },
        _ => texts_from_flow_value(item),
    }
}

fn texts_from_flow_value(v: &RuntimeValue) -> Result<Vec<String>, GraphicsValueError> {
    match cons_items(v) {
        Ok(items) => items.into_iter().map(atom_text).collect(),
        Err(_) => Ok(vec![atom_text(v)?]),
    }
}

fn push_table(table: &RuntimeValue, layout: &mut DocLayout) -> Result<(), GraphicsValueError> {
    let fields = record_fields(table, "doc-table")?;
    expect_tag(fields, "doc-table")?;
    let columns_v = field(fields, "columns")
        .ok_or_else(|| GraphicsValueError::new("doc-table missing columns"))?;
    let rows_v =
        field(fields, "rows").ok_or_else(|| GraphicsValueError::new("doc-table missing rows"))?;
    let mut rows: Vec<Vec<String>> = Vec::new();
    let ncols = if let Some(n) = as_f64(columns_v) {
        (n.max(1.0) as usize).max(1)
    } else {
        let header = cons_items(columns_v)?
            .into_iter()
            .map(atom_text)
            .collect::<Result<Vec<_>, _>>()?;
        let n = header.len().max(1);
        if !header.is_empty() {
            rows.push(header);
        }
        n
    };
    for row in cons_items(rows_v)? {
        let cells = match cons_items(row) {
            Ok(items) => items
                .into_iter()
                .map(atom_text)
                .collect::<Result<Vec<_>, _>>()?,
            Err(_) => vec![atom_text(row)?],
        };
        rows.push(cells);
    }
    if rows.is_empty() {
        return Ok(());
    }
    let size_mm = TABLE_CELL_SIZE_MM;
    let line_advance = size_mm + DOC_LINE_PITCH_EXTRA_MM;
    let pitch = -line_advance;
    let width = layout.content_width_mm();
    let col_w = width / ncols as f64;
    let wrap_em = ((col_w - 2.0 * TABLE_PAD_X_MM) / size_mm.max(0.01)).max(1.0);
    let x0 = layout.hanmen.left_mm;
    let table_top = layout.cursor_y;
    let mut row_y_top = table_top;
    let mut row_bottoms = Vec::new();
    for row in rows {
        let first_baseline = row_y_top - TABLE_PAD_Y_MM - size_mm;
        let mut max_lines = 1usize;
        for (ci, cell) in row.iter().take(ncols).enumerate() {
            let scene = ParagraphSceneLayout {
                base_x_mm: x0 + col_w * ci as f64 + TABLE_PAD_X_MM,
                start_y_mm: first_baseline,
                size_mm,
                pitch_mm: pitch,
                fill: Color::BLACK,
            };
            let n = emit_flow_text(cell, 0.0, wrap_em, &scene, layout)?;
            max_lines = max_lines.max(n);
        }
        let row_h = TABLE_PAD_Y_MM * 2.0 + line_advance * max_lines as f64;
        let bottom = row_y_top - row_h;
        row_bottoms.push(bottom);
        row_y_top = bottom;
    }
    let table_bottom = *row_bottoms.last().expect("non-empty rows");
    layout.shapes.push(Shape::Frame(Frame {
        x_mm: x0,
        y_mm: table_bottom,
        width_mm: width,
        height_mm: table_top - table_bottom,
        stroke_width_mm: TABLE_STROKE_MM,
        stroke: Color::BLACK,
    }));
    for ci in 1..ncols {
        let x = x0 + col_w * ci as f64;
        push_rule(x, table_bottom, x, table_top, layout);
    }
    for bottom in row_bottoms.iter().take(row_bottoms.len().saturating_sub(1)) {
        push_rule(x0, *bottom, x0 + width, *bottom, layout);
    }
    layout.cursor_y = table_bottom - 2.0;
    Ok(())
}

fn push_rule(x1: f64, y1: f64, x2: f64, y2: f64, layout: &mut DocLayout) {
    layout.shapes.push(Shape::Line(Line {
        x1_mm: x1,
        y1_mm: y1,
        x2_mm: x2,
        y2_mm: y2,
        stroke: Color::BLACK,
        width_mm: TABLE_STROKE_MM,
    }));
}

fn push_figure(figure: &RuntimeValue, layout: &mut DocLayout) -> Result<(), GraphicsValueError> {
    let fields = record_fields(figure, "doc-figure")?;
    expect_tag(fields, "doc-figure")?;
    let visual = field(fields, "visual");
    let caption = field(fields, "caption");
    let width = layout.content_width_mm();
    let mut height = FIGURE_HEIGHT_MM;
    let mut label = String::new();
    if let Some(v) = visual {
        if let Some(n) = as_f64(v) {
            height = n.max(8.0);
        } else if let Ok(s) = atom_text(v) {
            label = s;
        }
    }
    let x = layout.hanmen.left_mm;
    let top = layout.cursor_y;
    let y = top - height;
    layout.shapes.push(Shape::Rect(Rect {
        x_mm: x,
        y_mm: y,
        width_mm: width,
        height_mm: height,
        fill: Color::new(0.88, 0.88, 0.88),
    }));
    layout.shapes.push(Shape::Frame(Frame {
        x_mm: x,
        y_mm: y,
        width_mm: width,
        height_mm: height,
        stroke_width_mm: 0.4,
        stroke: Color::BLACK,
    }));
    if !label.is_empty() {
        let scene = ParagraphSceneLayout {
            base_x_mm: x + 3.0,
            start_y_mm: y + height * 0.5,
            size_mm: FLOW_BODY_SIZE_MM,
            pitch_mm: -(FLOW_BODY_SIZE_MM + DOC_LINE_PITCH_EXTRA_MM),
            fill: Color::BLACK,
        };
        let wrap_em = layout.wrap_em_at(FLOW_BODY_SIZE_MM, 3.0);
        emit_flow_text(&label, 0.0, wrap_em, &scene, layout)?;
    }
    // Caption ink sits above its baseline; leave a full em plus gap under the box.
    layout.cursor_y = y - CAPTION_SIZE_MM - FIGURE_CAPTION_GAP_MM;
    if let Some(c) = caption {
        let s = atom_text(c)?;
        if !s.is_empty() {
            let line = format!("Caption: {s}");
            push_flow_text(&line, CAPTION_SIZE_MM, 0.0, 0.0, layout)?;
        }
    }
    Ok(())
}

fn atom_text(v: &RuntimeValue) -> Result<String, GraphicsValueError> {
    match v {
        RuntimeValue::String(s) => Ok(s.clone()),
        RuntimeValue::Int(n) => Ok(n.to_string()),
        RuntimeValue::Number(n) | RuntimeValue::F64(n) => Ok(n.to_string()),
        RuntimeValue::Record(fields) => string_field(fields, "text"),
        _ => Err(GraphicsValueError::new("expected string or text record")),
    }
}

fn paper_from_size_value(v: &RuntimeValue) -> Result<PaperSize, GraphicsValueError> {
    let fields = record_fields(v, "paper size")?;
    let width_mm = number_field(fields, "width")?;
    let height_mm = number_field(fields, "height")?;
    let paper = PaperSize {
        width_mm,
        height_mm,
    };
    if !paper.is_positive() {
        return Err(GraphicsValueError::new("paper size must be positive"));
    }
    Ok(paper)
}

fn record_fields<'a>(
    v: &'a RuntimeValue,
    ctx: &str,
) -> Result<&'a [(String, RuntimeValue)], GraphicsValueError> {
    match v {
        RuntimeValue::Record(fields) => Ok(fields.as_slice()),
        _ => Err(GraphicsValueError::new(format!(
            "{ctx}: expected record, got non-record"
        ))),
    }
}

fn field<'a>(fields: &'a [(String, RuntimeValue)], name: &str) -> Option<&'a RuntimeValue> {
    fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

fn tag_of(fields: &[(String, RuntimeValue)]) -> Option<&str> {
    match field(fields, "tag")? {
        RuntimeValue::String(s) => Some(s.as_str()),
        RuntimeValue::ShapeTag(s) => Some(s.as_str()),
        _ => None,
    }
}

fn expect_tag(fields: &[(String, RuntimeValue)], want: &str) -> Result<(), GraphicsValueError> {
    match tag_of(fields) {
        Some(t) if t == want => Ok(()),
        Some(t) => Err(GraphicsValueError::new(format!(
            "expected tag `{want}`, got `{t}`"
        ))),
        None => Err(GraphicsValueError::new(format!(
            "expected tag `{want}`, missing tag"
        ))),
    }
}

fn string_field(
    fields: &[(String, RuntimeValue)],
    name: &str,
) -> Result<String, GraphicsValueError> {
    match field(fields, name) {
        Some(RuntimeValue::String(s)) => Ok(s.clone()),
        _ => Err(GraphicsValueError::new(format!(
            "missing string field `{name}`"
        ))),
    }
}

fn number_field(fields: &[(String, RuntimeValue)], name: &str) -> Result<f64, GraphicsValueError> {
    field(fields, name)
        .and_then(as_f64)
        .ok_or_else(|| GraphicsValueError::new(format!("missing numeric field `{name}`")))
}

fn as_f64(v: &RuntimeValue) -> Option<f64> {
    match v {
        RuntimeValue::Int(n) => Some(*n as f64),
        RuntimeValue::F64(n) => Some(*n),
        RuntimeValue::Number(n) => Some(*n),
        _ => None,
    }
}

fn cons_items(v: &RuntimeValue) -> Result<Vec<&RuntimeValue>, GraphicsValueError> {
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match cur {
            RuntimeValue::Variant { tag, .. } if tag == "nil" => break,
            RuntimeValue::Variant { tag, payload } if tag == "cons" => {
                let Some(payload) = payload.as_ref() else {
                    return Err(GraphicsValueError::new("cons variant missing payload"));
                };
                let fields = record_fields(payload, "cons")?;
                let head = field(fields, "head")
                    .ok_or_else(|| GraphicsValueError::new("cons missing head"))?;
                let tail = field(fields, "tail")
                    .ok_or_else(|| GraphicsValueError::new("cons missing tail"))?;
                out.push(head);
                cur = tail;
            }
            _ => {
                return Err(GraphicsValueError::new(
                    "expected cons/nil list of document nodes",
                ));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::Shape;

    fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
        RuntimeValue::Record(
            fields
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn cons(items: Vec<RuntimeValue>) -> RuntimeValue {
        let mut cur = RuntimeValue::Variant {
            tag: "nil".into(),
            payload: None,
        };
        for item in items.into_iter().rev() {
            cur = RuntimeValue::Variant {
                tag: "cons".into(),
                payload: Some(Box::new(rec(vec![("head", item), ("tail", cur)]))),
            };
        }
        cur
    }

    #[test]
    fn lowers_doc_page_heading_and_paragraph() {
        let heading = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(1)),
            ("text", RuntimeValue::String("Title".into())),
        ]);
        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String("Body".into())),
        ]);
        let spacer = rec(vec![
            ("tag", RuntimeValue::String("doc-spacer".into())),
            ("length", RuntimeValue::Int(4)),
        ]);
        let blocks = cons(vec![
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("heading".into())),
                ("heading", heading.clone()),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("paragraph".into())),
                ("paragraph", paragraph),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("spacer".into())),
                ("spacer", spacer),
            ]),
        ]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("title", heading),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = layout_doc_page_to_scene(&page).expect("doc lower");
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.pages[0].paper.width_mm, 210.0);
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.as_str()),
                _ => None,
            })
            .collect();
        assert!(texts.contains(&"Title"));
        assert!(texts.contains(&"Body"));
    }

    #[test]
    fn doc_paragraph_indent_em_offsets_first_text_x() {
        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String("字下げ本文".into())),
            ("indent-em", RuntimeValue::Number(1.0)),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("paragraph".into())),
            ("paragraph", paragraph),
        ])]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            (
                "title",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-heading".into())),
                    ("level", RuntimeValue::Int(2)),
                    ("text", RuntimeValue::String("H".into())),
                ]),
            ),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("doc lower");
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        let body = texts
            .iter()
            .find(|t| t.content == "字下げ本文")
            .expect("indented paragraph text");
        // BASE_X 20 + 1em * size 4.0 = 24.0
        assert!((body.x_mm - 24.0).abs() < 1e-9);
        let heading = texts.iter().find(|t| t.content == "H").expect("heading");
        assert!((heading.x_mm - 20.0).abs() < 1e-9);
    }

    #[test]
    fn doc_paragraph_long_text_uses_break_line() {
        // Longer than DOC_TEXT_MAX_EM (40): must emit multiple Text shapes via break_line.
        let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
        assert!(long.chars().count() > 40);
        let expected = reciplexa_std::japanese::break_line(long, 40.0);
        assert!(expected.len() > 1);

        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String(long.into())),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("paragraph".into())),
            ("paragraph", paragraph),
        ])]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("doc lower");
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, expected);
    }

    #[test]
    fn doc_heading_long_text_uses_break_line() {
        let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
        assert!(long.chars().count() > 40);
        let expected = reciplexa_std::japanese::break_line(long, 40.0);
        assert!(expected.len() > 1);

        let heading = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(1)),
            ("text", RuntimeValue::String(long.into())),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("heading".into())),
            ("heading", heading),
        ])]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("doc lower");
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t.content.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, expected);
    }

    #[test]
    fn rejects_non_doc_page() {
        let v = rec(vec![("tag", RuntimeValue::String("page".into()))]);
        assert!(document_from_doc_value(&v).is_err());
    }

    #[test]
    fn n5_3_tips_block_stubs_levels_and_error_arms() {
        // heading levels 2 / default + missing kind / unknown kind / bad text node
        let h2 = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(2)),
            ("text", RuntimeValue::String("H2".into())),
        ]);
        let h3 = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(9)),
            ("text", RuntimeValue::String("Hn".into())),
        ]);
        let blocks = cons(vec![
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("heading".into())),
                ("heading", h2),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("heading".into())),
                ("heading", h3),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("spacer".into())),
                (
                    "spacer",
                    rec(vec![
                        ("tag", RuntimeValue::String("doc-spacer".into())),
                        // missing length → default 4.0
                    ]),
                ),
            ]),
        ]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::F64(100.0)),
                    ("height", RuntimeValue::F64(200.0)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("levels");
        assert!(doc.pages[0]
            .shapes
            .iter()
            .any(|s| matches!(s, Shape::Text(t) if t.content == "H2")));

        assert!(document_from_doc_value(&rec(vec![(
            "tag",
            RuntimeValue::String("doc-page".into())
        ),]))
        .is_err());
        let bad_block = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::Int(1)),
        ]);
        let bad_section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", cons(vec![bad_block])),
        ]);
        let bad_flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![bad_section])),
        ]);
        let bad_page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", bad_flow),
        ]);
        assert!(document_from_doc_value(&bad_page).is_err());

        let unk = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("mystery".into())),
        ]);
        let unk_section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", cons(vec![unk])),
        ]);
        let unk_flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![unk_section])),
        ]);
        let unk_page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", unk_flow),
        ]);
        assert!(document_from_doc_value(&unk_page)
            .unwrap_err()
            .message
            .contains("unsupported"));

        // non-positive paper
        let zero_paper = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(0)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            (
                "flow",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-flow".into())),
                    ("sections", cons(vec![])),
                ]),
            ),
        ]);
        assert!(document_from_doc_value(&zero_paper).is_err());
    }

    #[test]
    fn doc_columns_places_paragraphs_with_measure_columns() {
        let columns = rec(vec![
            ("tag", RuntimeValue::String("doc-columns".into())),
            ("count", RuntimeValue::Int(2)),
            ("gutter-em", RuntimeValue::Number(1.0)),
            ("total-em", RuntimeValue::Number(21.0)),
            (
                "paragraphs",
                cons(vec![
                    RuntimeValue::String("左カラム".into()),
                    rec(vec![
                        ("tag", RuntimeValue::String("doc-paragraph".into())),
                        ("text", RuntimeValue::String("右カラム".into())),
                    ]),
                ]),
            ),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("columns".into())),
            ("columns", columns),
        ])]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            (
                "title",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-heading".into())),
                    ("level", RuntimeValue::Int(2)),
                    ("text", RuntimeValue::String("Cols".into())),
                ]),
            ),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc = document_from_doc_value(&page).expect("doc-columns lower");
        let texts: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(texts.iter().any(|t| t.content == "左カラム"));
        assert!(texts.iter().any(|t| t.content == "右カラム"));
        let left = texts.iter().find(|t| t.content == "左カラム").unwrap();
        let right = texts.iter().find(|t| t.content == "右カラム").unwrap();
        // col_w = (21-1)/2 = 10; xs = [0, 11]; size_mm = 4 → Δx = 44mm
        assert!((right.x_mm - left.x_mm - 11.0 * 4.0).abs() < 1e-6);
        assert!((left.y_mm - right.y_mm).abs() < 1e-9);
    }

    #[test]
    fn product_engine_wraps_with_font_metrics_not_stub() {
        let long = "ああああああああああああああああああああああああああああああああああああああああああああ";
        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String(long.into())),
        ]);
        let blocks = cons(vec![rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("paragraph".into())),
            ("paragraph", paragraph),
        ])]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            (
                "title",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-heading".into())),
                    ("level", RuntimeValue::Int(2)),
                    ("text", RuntimeValue::String("H".into())),
                ]),
            ),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let stub = layout_doc_page_to_scene(&page).expect("stub");
        let product =
            layout_doc_page_to_scene_with_engine(&page, TypesetEngine::Product).expect("product");
        let stub_n = stub.pages[0]
            .shapes
            .iter()
            .filter(|s| matches!(s, Shape::Text(_)))
            .count();
        let product_n = product.pages[0]
            .shapes
            .iter()
            .filter(|s| s.is_text_like())
            .count();
        assert!(product_n > 1);
        // Fixture あ is 0.98em vs stub 1.0em, so wrap count can differ.
        let _ = stub_n;
    }

    #[test]
    fn product_doc_advances_cursor_by_line_not_glyph_count() {
        // Ten Latin glyphs on one line. Pitch is size+3; the next block must
        // sit one step below, not ten glyph-counts down the page.
        let heading = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(1)),
            ("text", RuntimeValue::String("ABCDEFGHIJ".into())),
        ]);
        let paragraph = rec(vec![
            ("tag", RuntimeValue::String("doc-paragraph".into())),
            ("text", RuntimeValue::String("X".into())),
        ]);
        let blocks = cons(vec![
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("heading".into())),
                ("heading", heading),
            ]),
            rec(vec![
                ("tag", RuntimeValue::String("doc-block".into())),
                ("kind", RuntimeValue::String("paragraph".into())),
                ("paragraph", paragraph),
            ]),
        ]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("blocks", blocks),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let page = rec(vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ]);
        let doc =
            layout_doc_page_to_scene_with_engine(&page, TypesetEngine::Product).expect("product");
        let y_of = |ch: &str| {
            doc.pages[0]
                .shapes
                .iter()
                .find_map(|s| match s {
                    Shape::GlyphRun(g) if g.content == ch => Some(g.y_mm),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing {ch}"))
        };
        let heading_y = y_of("A");
        let body_y = y_of("X");
        let pitch = -(8.0 + DOC_LINE_PITCH_EXTRA_MM);
        assert!(
            (body_y - (heading_y + pitch)).abs() < 1e-6,
            "body must be one line-pitch below the heading, got heading={heading_y} body={body_y}"
        );
        assert!(
            body_y > 250.0,
            "one-line 10-glyph heading must not shove the next block down the page, body y={body_y}"
        );
    }

    fn flow_page(blocks: Vec<RuntimeValue>, margins: Option<RuntimeValue>) -> RuntimeValue {
        let heading = rec(vec![
            ("tag", RuntimeValue::String("doc-heading".into())),
            ("level", RuntimeValue::Int(1)),
            ("text", RuntimeValue::String("One".into())),
        ]);
        let section = rec(vec![
            ("tag", RuntimeValue::String("doc-section".into())),
            ("title", heading),
            ("blocks", cons(blocks)),
        ]);
        let flow = rec(vec![
            ("tag", RuntimeValue::String("doc-flow".into())),
            ("sections", cons(vec![section])),
        ]);
        let mut fields = vec![
            ("tag", RuntimeValue::String("doc-page".into())),
            (
                "paper",
                rec(vec![
                    ("width", RuntimeValue::Int(210)),
                    ("height", RuntimeValue::Int(297)),
                ]),
            ),
            ("flow", flow),
        ];
        if let Some(m) = margins {
            fields.push(("margins", m));
        }
        rec(fields)
    }

    fn para_block(text: &str) -> RuntimeValue {
        rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("paragraph".into())),
            (
                "paragraph",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-paragraph".into())),
                    ("text", RuntimeValue::String(text.into())),
                ]),
            ),
        ])
    }

    #[test]
    fn explicit_pagebreak_sends_next_block_to_new_page() {
        let break_block = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("pagebreak".into())),
        ]);
        let page = flow_page(
            vec![para_block("keep"), break_block, para_block("after")],
            None,
        );
        let doc = layout_doc_page_to_scene(&page).expect("layout");
        assert_eq!(doc.pages.len(), 2);
        let p0: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(Shape::text_content)
            .collect();
        let p1: Vec<_> = doc.pages[1]
            .shapes
            .iter()
            .filter_map(Shape::text_content)
            .collect();
        assert!(p0.contains(&"One"));
        assert!(p0.contains(&"keep"));
        assert!(!p0.contains(&"after"));
        assert!(p1.contains(&"after"));
        assert_eq!(doc.pages[1].paper.width_mm, 210.0);
    }

    #[test]
    fn framed_hanmen_offsets_preview_and_export_the_same() {
        let margins = rec(vec![
            ("tag", RuntimeValue::String("doc-margins".into())),
            ("top", RuntimeValue::Number(40.0)),
            ("right", RuntimeValue::Number(15.0)),
            ("bottom", RuntimeValue::Number(30.0)),
            ("left", RuntimeValue::Number(50.0)),
        ]);
        let page = flow_page(vec![para_block("body")], Some(margins));
        let stub = layout_doc_page_to_scene(&page).expect("stub");
        let product =
            layout_doc_page_to_scene_with_engine(&page, TypesetEngine::Product).expect("product");
        assert_eq!(stub.pages.len(), 1);
        assert_eq!(product.pages.len(), stub.pages.len());
        let stub_x = stub.pages[0].shapes[0].text_x_mm().expect("x");
        let prod_x = product.pages[0].shapes[0].text_x_mm().expect("x");
        assert!((stub_x - 50.0).abs() < 1e-6, "stub x={stub_x}");
        assert!((prod_x - 50.0).abs() < 1e-6, "product x={prod_x}");
        let stub_y = stub.pages[0].shapes[0].text_y_mm().expect("y");
        let prod_y = product.pages[0].shapes[0].text_y_mm().expect("y");
        let top_baseline = 297.0 - 40.0;
        assert!(
            (stub_y - top_baseline).abs() < 1e-6,
            "first baseline must sit at height-top, y={stub_y}"
        );
        assert!((stub_y - prod_y).abs() < 1e-6);
    }

    fn page_text(doc: &Document) -> String {
        doc.pages
            .iter()
            .flat_map(|p| p.shapes.iter().filter_map(Shape::text_content))
            .collect()
    }

    fn list_block(ordered: bool, items: Vec<&str>) -> RuntimeValue {
        rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("list".into())),
            (
                "list",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-list".into())),
                    ("ordered", RuntimeValue::Bool(ordered)),
                    (
                        "items",
                        cons(
                            items
                                .into_iter()
                                .map(|s| RuntimeValue::String(s.into()))
                                .collect(),
                        ),
                    ),
                ]),
            ),
        ])
    }

    #[test]
    fn flow_list_note_figure_table_emit_on_page() {
        let note = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("note".into())),
            (
                "note",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-note".into())),
                    ("text", RuntimeValue::String("Keep the hanmen".into())),
                ]),
            ),
        ]);
        let figure = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("figure".into())),
            (
                "figure",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-figure".into())),
                    ("visual", RuntimeValue::String("Diagram".into())),
                    ("caption", RuntimeValue::String("A box".into())),
                ]),
            ),
        ]);
        let table = rec(vec![
            ("tag", RuntimeValue::String("doc-block".into())),
            ("kind", RuntimeValue::String("table".into())),
            (
                "table",
                rec(vec![
                    ("tag", RuntimeValue::String("doc-table".into())),
                    (
                        "columns",
                        cons(vec![
                            RuntimeValue::String("Name".into()),
                            RuntimeValue::String("Qty".into()),
                        ]),
                    ),
                    (
                        "rows",
                        cons(vec![cons(vec![
                            RuntimeValue::String("A".into()),
                            RuntimeValue::String("1".into()),
                        ])]),
                    ),
                ]),
            ),
        ]);
        let margins = rec(vec![
            ("tag", RuntimeValue::String("doc-margins".into())),
            ("top", RuntimeValue::Number(28.0)),
            ("right", RuntimeValue::Number(20.0)),
            ("bottom", RuntimeValue::Number(28.0)),
            ("left", RuntimeValue::Number(24.0)),
        ]);
        let page = flow_page(
            vec![
                list_block(false, vec!["First item", "Second item"]),
                list_block(true, vec!["Step one"]),
                note,
                figure,
                table,
            ],
            Some(margins),
        );
        let stub = layout_doc_page_to_scene(&page).expect("stub");
        let product =
            layout_doc_page_to_scene_with_engine(&page, TypesetEngine::Product).expect("product");
        for doc in [&stub, &product] {
            let t = page_text(doc);
            assert!(t.contains("First item"), "{t}");
            assert!(t.contains("Second item"), "{t}");
            assert!(t.contains("Step one"), "{t}");
            assert!(t.contains("Note:"), "{t}");
            assert!(t.contains("Keep the hanmen"), "{t}");
            assert!(t.contains("Diagram"), "{t}");
            assert!(t.contains("Caption:"), "{t}");
            assert!(t.contains("Name"), "{t}");
            assert!(t.contains("Qty"), "{t}");
            assert!(t.contains('A'), "{t}");
            assert!(t.contains('1'), "{t}");
            assert!(
                doc.pages[0]
                    .shapes
                    .iter()
                    .any(|s| matches!(s, Shape::Frame(_) | Shape::Rect(_))),
                "figure must emit a box"
            );
            assert!(
                doc.pages[0]
                    .shapes
                    .iter()
                    .any(|s| matches!(s, Shape::Line(_))),
                "table must emit grid rules"
            );
            let rect = doc.pages[0]
                .shapes
                .iter()
                .find_map(|s| match s {
                    Shape::Rect(r) => Some(*r),
                    _ => None,
                })
                .expect("figure fill");
            let cap_y = doc.pages[0]
                .shapes
                .iter()
                .filter_map(|s| {
                    let y = s.text_y_mm()?;
                    (y < rect.y_mm - 1e-6).then_some(y)
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .expect("text below figure box");
            assert!(
                cap_y + CAPTION_SIZE_MM <= rect.y_mm + 1e-6,
                "caption y={cap_y} size={CAPTION_SIZE_MM} must sit below figure box bottom {}",
                rect.y_mm
            );
            let left = 24.0;
            for shape in &doc.pages[0].shapes {
                if let Some(x) = shape.text_x_mm() {
                    assert!(x + 1e-6 >= left, "x={x}");
                }
                if let Shape::Rect(r) = shape {
                    assert!(r.x_mm + 1e-6 >= left, "rect x={}", r.x_mm);
                }
                if let Shape::Frame(f) = shape {
                    assert!(f.x_mm + 1e-6 >= left, "frame x={}", f.x_mm);
                }
            }
        }
        let stub_x = stub.pages[0]
            .shapes
            .iter()
            .find_map(Shape::text_x_mm)
            .expect("stub x");
        let prod_x = product.pages[0]
            .shapes
            .iter()
            .find_map(Shape::text_x_mm)
            .expect("prod x");
        assert!((stub_x - prod_x).abs() < 1e-9);
    }
}
