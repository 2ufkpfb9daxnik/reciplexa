//! Pinned redistributable fixture TTF (generated offline, no system fonts).

use std::sync::OnceLock;

/// Distinctive MATH constants (design units, UPEM 1000) for PT-4 tests.
pub const FIXTURE_SCRIPT_PERCENT_SCALE_DOWN: i16 = 70;
pub const FIXTURE_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN: i16 = 55;
pub const FIXTURE_AXIS_HEIGHT: i16 = 270;
pub const FIXTURE_FRACTION_RULE_THICKNESS: i16 = 80;
pub const FIXTURE_SUPERSCRIPT_SHIFT_UP: i16 = 420;
pub const FIXTURE_SUBSCRIPT_SHIFT_DOWN: i16 = 210;
pub const FIXTURE_RADICAL_RULE_THICKNESS: i16 = 70;
pub const FIXTURE_DISPLAY_OPERATOR_MIN_HEIGHT: u16 = 1400;
pub const FIXTURE_DELIMITED_SUB_FORMULA_MIN_HEIGHT: u16 = 1200;
pub const FIXTURE_ACCENT_BASE_HEIGHT: i16 = 480;
pub const FIXTURE_SPACE_AFTER_SCRIPT: i16 = 40;
pub const FIXTURE_UPPER_LIMIT_GAP_MIN: i16 = 50;
pub const FIXTURE_UPPER_LIMIT_BASELINE_RISE_MIN: i16 = 200;
pub const FIXTURE_LOWER_LIMIT_GAP_MIN: i16 = 50;
pub const FIXTURE_LOWER_LIMIT_BASELINE_DROP_MIN: i16 = 200;
pub const FIXTURE_STACK_GAP_MIN: i16 = 120;
pub const FIXTURE_STACK_DISPLAY_STYLE_GAP_MIN: i16 = 200;
pub const FIXTURE_OVERBAR_VERTICAL_GAP: i16 = 60;
pub const FIXTURE_OVERBAR_RULE_THICKNESS: i16 = 50;
pub const FIXTURE_OVERBAR_EXTRA_ASCENDER: i16 = 40;
pub const FIXTURE_UNDERBAR_VERTICAL_GAP: i16 = 60;
pub const FIXTURE_UNDERBAR_RULE_THICKNESS: i16 = 50;
pub const FIXTURE_UNDERBAR_EXTRA_DESCENDER: i16 = 40;
pub const FIXTURE_RADICAL_VERTICAL_GAP: i16 = 60;
pub const FIXTURE_RADICAL_DISPLAY_STYLE_VERTICAL_GAP: i16 = 100;
pub const FIXTURE_RADICAL_EXTRA_ASCENDER: i16 = 40;
pub const FIXTURE_RADICAL_KERN_BEFORE_DEGREE: i16 = 80;
pub const FIXTURE_RADICAL_KERN_AFTER_DEGREE: i16 = -80;
pub const FIXTURE_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT: i16 = 60;
pub const FIXTURE_FRACTION_NUMERATOR_SHIFT_UP: i16 = 400;
pub const FIXTURE_FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP: i16 = 500;
pub const FIXTURE_FRACTION_DENOMINATOR_SHIFT_DOWN: i16 = 400;
pub const FIXTURE_FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN: i16 = 500;
/// MATH Variants `advanceMeasurement` for the cmap `(` glyph (design units).
pub const FIXTURE_PAREN_VARIANT_ADVANCE: u16 = 1000;
/// MATH Variants `advanceMeasurement` for the construction-only tall paren.
pub const FIXTURE_TALL_PAREN_VARIANT_ADVANCE: u16 = 1800;
/// MATH assembly extender `fullAdvance` (design units).
pub const FIXTURE_EXTENDER_ADVANCE: u16 = 400;
/// MATH assembly top/bottom piece `fullAdvance` (design units).
pub const FIXTURE_ASSEMBLY_END_ADVANCE: u16 = 600;
/// MATH italic correction for cmap `f` (design units).
pub const FIXTURE_ITALIC_CORRECTION_F: i16 = 180;
/// MATH top-right kern for cmap `f` (design units; height-independent).
pub const FIXTURE_MATH_KERN_TOP_RIGHT: i16 = 120;
/// MATH bottom-right kern for cmap `f` (design units; height-independent).
pub const FIXTURE_MATH_KERN_BOTTOM_RIGHT: i16 = -40;

/// Extra CJK ideographs used by Step 7 examples (hiragana/katakana are full ranges).
const EXTRA_IDEOGRAPHS: &str =
    "一三上下二京今仕付以件作保先冬処分列初則印厚参句可右告四型報夏天字実小少展左式弧強折括捗描数文方日春書期本条来東格概様段気混添漢点照版現理目禁秋稿第箇細組続縦置草落行装補要視試詰詳語調足返進配閉開項頭題令和年重";

pub fn fixture_font_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(build_fixture_ttf).as_slice()
}

fn build_fixture_ttf() -> Vec<u8> {
    let mut glyphs: Vec<GlyphSpec> = Vec::new();
    glyphs.push(GlyphSpec {
        ch: None,
        advance: 500,
        kind: GlyphKind::Notdef,
    });
    for cp in 0x20u32..=0x7Eu32 {
        let ch = char::from_u32(cp).expect("ascii");
        glyphs.push(GlyphSpec {
            ch: Some(ch),
            advance: ascii_advance(ch),
            kind: GlyphKind::Rect,
        });
    }
    for cp in 0x3040u32..=0x309Fu32 {
        if let Some(ch) = char::from_u32(cp) {
            glyphs.push(GlyphSpec {
                ch: Some(ch),
                advance: 980,
                kind: GlyphKind::Rect,
            });
        }
    }
    for cp in 0x30A0u32..=0x30FFu32 {
        if let Some(ch) = char::from_u32(cp) {
            glyphs.push(GlyphSpec {
                ch: Some(ch),
                advance: 980,
                kind: GlyphKind::Rect,
            });
        }
    }
    const PUNCT: &str = "。「」、…『』（）【】・！？•·§–—　〜（）";
    for ch in PUNCT.chars() {
        if glyphs.iter().any(|g| g.ch == Some(ch)) {
            continue;
        }
        glyphs.push(GlyphSpec {
            ch: Some(ch),
            advance: punct_advance(ch),
            kind: GlyphKind::Rect,
        });
    }
    const MATH_EXTRA: &str = "∑√→∞±×≤≥∫Σ";
    for ch in MATH_EXTRA.chars() {
        if glyphs.iter().any(|g| g.ch == Some(ch)) {
            continue;
        }
        glyphs.push(GlyphSpec {
            ch: Some(ch),
            advance: 900,
            kind: GlyphKind::Rect,
        });
    }
    for ch in EXTRA_IDEOGRAPHS.chars() {
        if glyphs.iter().any(|g| g.ch == Some(ch)) {
            continue;
        }
        glyphs.push(GlyphSpec {
            ch: Some(ch),
            advance: 1000,
            kind: GlyphKind::Rect,
        });
    }
    // Taller paren variant + extender for MATH stretchy (no cmap; construction-only).
    let paren_gid = glyphs
        .iter()
        .position(|g| g.ch == Some('('))
        .expect("ascii paren");
    let close_gid = glyphs
        .iter()
        .position(|g| g.ch == Some(')'))
        .expect("ascii close paren");
    let tall_paren_gid = glyphs.len() as u16;
    glyphs.push(GlyphSpec {
        ch: None,
        advance: 400,
        kind: GlyphKind::TallRect,
    });
    let extender_gid = glyphs.len() as u16;
    glyphs.push(GlyphSpec {
        ch: None,
        advance: 400,
        kind: GlyphKind::Rect,
    });

    build_sfnt(
        &glyphs,
        Some((
            paren_gid as u16,
            close_gid as u16,
            tall_paren_gid,
            extender_gid,
        )),
        &[],
    )
}

/// Minimal face for fallback tests: only U+263A WHITE SMILING FACE.
pub fn build_fallback_only_font_bytes() -> Vec<u8> {
    let glyphs = vec![
        GlyphSpec {
            ch: None,
            advance: 500,
            kind: GlyphKind::Notdef,
        },
        GlyphSpec {
            ch: Some('☺'),
            advance: 800,
            kind: GlyphKind::Rect,
        },
    ];
    build_sfnt(&glyphs, None, &[])
}

/// Fixture with `liga` GSUB: `f` + `i` → single ligature glyph (gid 3).
pub fn build_liga_fixture_font_bytes() -> Vec<u8> {
    let glyphs = vec![
        GlyphSpec {
            ch: None,
            advance: 500,
            kind: GlyphKind::Notdef,
        },
        GlyphSpec {
            ch: Some('f'),
            advance: 400,
            kind: GlyphKind::Rect,
        },
        GlyphSpec {
            ch: Some('i'),
            advance: 350,
            kind: GlyphKind::Rect,
        },
        GlyphSpec {
            ch: None,
            advance: 700,
            kind: GlyphKind::Rect,
        },
    ];
    let gsub = build_gsub_liga(1, 2, 3);
    build_sfnt(&glyphs, None, &[(b"GSUB", gsub)])
}

#[cfg(test)]
mod liga_fixture_tests {
    use super::*;

    #[test]
    fn liga_fixture_parses_gsub() {
        let bytes = build_liga_fixture_font_bytes();
        let face = ttf_parser::Face::parse(&bytes, 0).expect("parse liga fixture");
        assert!(
            face.tables().gsub.is_some(),
            "liga fixture must include a GSUB table"
        );
    }
}

fn build_gsub_liga(f_gid: u16, i_gid: u16, lig_gid: u16) -> Vec<u8> {
    let mut gsub = Vec::new();
    put_u32(&mut gsub, 0x0001_0000);
    put_u16(&mut gsub, 10);
    put_u16(&mut gsub, 30);
    put_u16(&mut gsub, 44);
    // ScriptList @10
    put_u16(&mut gsub, 1);
    gsub.extend_from_slice(b"latn");
    put_u16(&mut gsub, 8);
    // Script @18
    put_u16(&mut gsub, 4);
    put_u16(&mut gsub, 0);
    // LangSys @22
    put_u16(&mut gsub, 0);
    put_u16(&mut gsub, 0xFFFF);
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 0);
    // FeatureList @30
    put_u16(&mut gsub, 1);
    gsub.extend_from_slice(b"liga");
    put_u16(&mut gsub, 8);
    // Feature @38
    put_u16(&mut gsub, 0);
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 0);
    // LookupList @44
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 6);
    // Lookup @50
    put_u16(&mut gsub, 4);
    put_u16(&mut gsub, 0);
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 8);
    // LigatureSubst @58
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 8);
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 14);
    // Coverage @66
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, f_gid);
    // LigatureSet @72
    put_u16(&mut gsub, 1);
    put_u16(&mut gsub, 4);
    // Ligature @76
    put_u16(&mut gsub, lig_gid);
    put_u16(&mut gsub, 2);
    put_u16(&mut gsub, f_gid);
    put_u16(&mut gsub, i_gid);
    gsub
}

fn ascii_advance(ch: char) -> u16 {
    match ch {
        ' ' => 300,
        'A' => 600,
        'x' => 520,
        '+' => 780,
        '(' | ')' => 400,
        '=' => 780,
        '.' | ',' => 300,
        '0'..='9' => 500,
        'a'..='z' => 540,
        'B'..='Z' => 620,
        _ => 500,
    }
}

fn punct_advance(ch: char) -> u16 {
    match ch {
        '。' => 400,
        '、' => 450,
        '「' | '」' | '『' | '』' => 500,
        '…' => 1000,
        _ => 500,
    }
}

#[derive(Clone, Copy)]
enum GlyphKind {
    Notdef,
    Rect,
    TallRect,
}

struct GlyphSpec {
    ch: Option<char>,
    advance: u16,
    kind: GlyphKind,
}

fn build_sfnt(
    glyphs: &[GlyphSpec],
    math_paren: Option<(u16, u16, u16, u16)>,
    extra: &[(&[u8; 4], Vec<u8>)],
) -> Vec<u8> {
    let num_glyphs = u16::try_from(glyphs.len()).expect("glyph count");
    let mut cmap_pairs: Vec<(u16, u16)> = Vec::new();
    for (i, g) in glyphs.iter().enumerate() {
        if let Some(ch) = g.ch {
            let cp = ch as u32;
            if cp <= 0xFFFF {
                cmap_pairs.push((cp as u16, i as u16));
            }
        }
    }
    cmap_pairs.sort_by_key(|p| p.0);
    cmap_pairs.dedup_by_key(|p| p.0);

    let cmap = build_cmap(&cmap_pairs);
    let (glyf, loca, x_min, y_min, x_max, y_max) = build_glyf_loca(glyphs);
    let hmtx = build_hmtx(glyphs);
    let hhea = build_hhea(glyphs, x_max);
    let maxp = build_maxp(num_glyphs);
    let head = build_head(x_min, y_min, x_max, y_max);
    let name = build_name();
    let post = build_post();
    let mut tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"cmap", cmap),
        (b"glyf", glyf),
        (b"head", head),
        (b"hhea", hhea),
        (b"hmtx", hmtx),
        (b"loca", loca),
        (b"maxp", maxp),
        (b"name", name),
        (b"post", post),
    ];
    if let Some((paren_gid, close_gid, tall_paren_gid, extender_gid)) = math_paren {
        let f_gid = glyphs
            .iter()
            .position(|g| g.ch == Some('f'))
            .expect("ascii f") as u16;
        tables.push((
            b"MATH",
            build_math(paren_gid, close_gid, tall_paren_gid, extender_gid, f_gid),
        ));
    }
    for (tag, data) in extra {
        tables.push((tag, data.clone()));
    }
    tables.sort_by(|a, b| a.0.cmp(b.0));

    let num_tables = u16::try_from(tables.len()).expect("tables");
    let entry_selector = floor_log2(num_tables);
    let search_range = (1u16 << entry_selector) * 16;
    let range_shift = num_tables * 16 - search_range;

    let header_len = 12 + 16 * tables.len();
    let mut offsets = Vec::with_capacity(tables.len());
    let mut cursor = header_len;
    let mut padded: Vec<Vec<u8>> = Vec::new();
    for (_, data) in &tables {
        offsets.push(cursor as u32);
        let mut p = data.clone();
        while p.len() % 4 != 0 {
            p.push(0);
        }
        cursor += p.len();
        padded.push(p);
    }

    let mut file = Vec::new();
    put_u32(&mut file, 0x0001_0000);
    put_u16(&mut file, num_tables);
    put_u16(&mut file, search_range);
    put_u16(&mut file, entry_selector);
    put_u16(&mut file, range_shift);
    for i in 0..tables.len() {
        file.extend_from_slice(tables[i].0);
        put_u32(&mut file, table_checksum(&padded[i]));
        put_u32(&mut file, offsets[i]);
        put_u32(&mut file, tables[i].1.len() as u32);
    }
    for p in &padded {
        file.extend_from_slice(p);
    }

    // Patch head.checkSumAdjustment so the whole-file checksum is 0xB1B0AFBA.
    let head_tag = b"head";
    let mut head_offset = None;
    let mut rec = 12usize;
    for _ in 0..tables.len() {
        if &file[rec..rec + 4] == head_tag {
            head_offset =
                Some(u32::from_be_bytes(file[rec + 8..rec + 12].try_into().unwrap()) as usize);
            break;
        }
        rec += 16;
    }
    let head_offset = head_offset.expect("head");
    // checkSumAdjustment is at byte 8 of head.
    for b in file.iter_mut().skip(head_offset + 8).take(4) {
        *b = 0;
    }
    let sum = file_checksum(&file);
    let adjust = 0xB1B0AFBAu32.wrapping_sub(sum);
    file[head_offset + 8..head_offset + 12].copy_from_slice(&adjust.to_be_bytes());
    file
}

fn floor_log2(n: u16) -> u16 {
    15u16.saturating_sub(n.leading_zeros() as u16)
}

fn table_checksum(padded: &[u8]) -> u32 {
    let mut sum = 0u32;
    for chunk in padded.chunks(4) {
        let mut b = [0u8; 4];
        b[..chunk.len()].copy_from_slice(chunk);
        sum = sum.wrapping_add(u32::from_be_bytes(b));
    }
    sum
}

fn file_checksum(file: &[u8]) -> u32 {
    table_checksum(file)
}

fn build_cmap(pairs: &[(u16, u16)]) -> Vec<u8> {
    let mut segs: Vec<(u16, u16, i16)> = Vec::new();
    let mut i = 0;
    while i < pairs.len() {
        let start_c = pairs[i].0;
        let start_g = pairs[i].1;
        let mut j = i + 1;
        while j < pairs.len()
            && pairs[j].0 == pairs[j - 1].0 + 1
            && pairs[j].1 == pairs[j - 1].1 + 1
        {
            j += 1;
        }
        let end_c = pairs[j - 1].0;
        let delta = (start_g as i32 - start_c as i32) as i16;
        segs.push((end_c, start_c, delta));
        i = j;
    }
    segs.push((0xFFFF, 0xFFFF, 1));

    let seg_count = segs.len() as u16;
    let seg_count_x2 = seg_count * 2;
    let entry_selector = floor_log2(seg_count);
    let search_range = (1u16 << entry_selector) * 2;
    let range_shift = seg_count_x2 - search_range;
    let length = 16u16 + seg_count * 8;

    let mut body = Vec::new();
    put_u16(&mut body, 4);
    put_u16(&mut body, length);
    put_u16(&mut body, 0);
    put_u16(&mut body, seg_count_x2);
    put_u16(&mut body, search_range);
    put_u16(&mut body, entry_selector);
    put_u16(&mut body, range_shift);
    for s in &segs {
        put_u16(&mut body, s.0);
    }
    put_u16(&mut body, 0);
    for s in &segs {
        put_u16(&mut body, s.1);
    }
    for s in &segs {
        put_i16(&mut body, s.2);
    }
    for _ in &segs {
        put_u16(&mut body, 0);
    }

    let mut cmap = Vec::new();
    put_u16(&mut cmap, 0);
    put_u16(&mut cmap, 1);
    put_u16(&mut cmap, 3);
    put_u16(&mut cmap, 1);
    put_u32(&mut cmap, 12);
    cmap.extend_from_slice(&body);
    cmap
}

fn build_glyf_loca(glyphs: &[GlyphSpec]) -> (Vec<u8>, Vec<u8>, i16, i16, i16, i16) {
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    let mut g_xmin = i16::MAX;
    let mut g_ymin = i16::MAX;
    let mut g_xmax = i16::MIN;
    let mut g_ymax = i16::MIN;
    for g in glyphs {
        put_u32(&mut loca, glyf.len() as u32);
        let h = match g.kind {
            GlyphKind::TallRect => 1800i16,
            _ => 700i16,
        };
        let w = i16::try_from(g.advance.max(2)).unwrap_or(2);
        let x0 = 40i16;
        let x1 = (w - 40).max(x0 + 1);
        let y0 = 0i16;
        let y1 = h;
        g_xmin = g_xmin.min(x0);
        g_ymin = g_ymin.min(y0);
        g_xmax = g_xmax.max(x1);
        g_ymax = g_ymax.max(y1);
        put_i16(&mut glyf, 1);
        put_i16(&mut glyf, x0);
        put_i16(&mut glyf, y0);
        put_i16(&mut glyf, x1);
        put_i16(&mut glyf, y1);
        put_u16(&mut glyf, 3);
        put_u16(&mut glyf, 0);
        glyf.extend_from_slice(&[0x01, 0x01, 0x01, 0x01]);
        put_i16(&mut glyf, x0);
        put_i16(&mut glyf, x1 - x0);
        put_i16(&mut glyf, 0);
        put_i16(&mut glyf, x0 - x1);
        put_i16(&mut glyf, y0);
        put_i16(&mut glyf, 0);
        put_i16(&mut glyf, y1 - y0);
        put_i16(&mut glyf, 0);
        while glyf.len() % 4 != 0 {
            glyf.push(0);
        }
    }
    put_u32(&mut loca, glyf.len() as u32);
    (glyf, loca, g_xmin, g_ymin, g_xmax, g_ymax)
}

fn build_hmtx(glyphs: &[GlyphSpec]) -> Vec<u8> {
    let mut b = Vec::new();
    for g in glyphs {
        put_u16(&mut b, g.advance);
        put_i16(&mut b, 40);
    }
    b
}

fn build_hhea(glyphs: &[GlyphSpec], x_max: i16) -> Vec<u8> {
    let adv_max = glyphs.iter().map(|g| g.advance).max().unwrap_or(0);
    let mut b = Vec::new();
    put_u32(&mut b, 0x0001_0000);
    put_i16(&mut b, 800);
    put_i16(&mut b, -200);
    put_i16(&mut b, 0);
    put_u16(&mut b, adv_max);
    put_i16(&mut b, 40);
    put_i16(&mut b, 0);
    put_i16(&mut b, x_max);
    put_i16(&mut b, 1);
    put_i16(&mut b, 0);
    put_i16(&mut b, 0);
    for _ in 0..4 {
        put_i16(&mut b, 0);
    }
    put_i16(&mut b, 0);
    put_u16(&mut b, glyphs.len() as u16);
    b
}

fn build_maxp(num_glyphs: u16) -> Vec<u8> {
    let mut b = Vec::new();
    put_u32(&mut b, 0x0001_0000);
    put_u16(&mut b, num_glyphs);
    put_u16(&mut b, 4);
    put_u16(&mut b, 1);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 2);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    b
}

fn build_head(x_min: i16, y_min: i16, x_max: i16, y_max: i16) -> Vec<u8> {
    let mut b = Vec::new();
    put_u16(&mut b, 1);
    put_u16(&mut b, 0);
    put_u32(&mut b, 0x0001_0000);
    put_u32(&mut b, 0); // checkSumAdjustment patched later
    put_u32(&mut b, 0x5F0F_3CF5);
    put_u16(&mut b, 0);
    put_u16(&mut b, 1000);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_i16(&mut b, x_min);
    put_i16(&mut b, y_min);
    put_i16(&mut b, x_max);
    put_i16(&mut b, y_max);
    put_u16(&mut b, 0);
    put_u16(&mut b, 8);
    put_i16(&mut b, 2);
    put_i16(&mut b, 1); // long loca
    put_i16(&mut b, 0);
    b
}

fn build_name() -> Vec<u8> {
    let names: &[(u16, &str)] = &[
        (1, "ReciplexaFixture"),
        (2, "Regular"),
        (3, "ReciplexaFixture"),
        (4, "ReciplexaFixture Regular"),
        (6, "ReciplexaFixture-Regular"),
    ];
    let mut storage = Vec::new();
    let mut recs = Vec::new();
    for &(id, s) in names {
        let off = storage.len() as u16;
        let utf16: Vec<u8> = s.encode_utf16().flat_map(|c| c.to_be_bytes()).collect();
        recs.push((id, off, utf16.len() as u16));
        storage.extend_from_slice(&utf16);
    }
    let mut b = Vec::new();
    put_u16(&mut b, 0);
    put_u16(&mut b, recs.len() as u16);
    put_u16(&mut b, 6 + 12 * recs.len() as u16);
    for &(id, off, len) in &recs {
        put_u16(&mut b, 3);
        put_u16(&mut b, 1);
        put_u16(&mut b, 0x0409);
        put_u16(&mut b, id);
        put_u16(&mut b, len);
        put_u16(&mut b, off);
    }
    b.extend_from_slice(&storage);
    b
}

fn build_post() -> Vec<u8> {
    let mut b = Vec::new();
    put_u32(&mut b, 0x0003_0000);
    put_u32(&mut b, 0);
    put_i16(&mut b, 0);
    put_i16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    put_u32(&mut b, 0);
    b
}

fn build_math(
    paren_gid: u16,
    close_gid: u16,
    tall_paren_gid: u16,
    extender_gid: u16,
    f_gid: u16,
) -> Vec<u8> {
    // Header 10 bytes; constants at 10 (214 bytes); glyph info; then variants.
    let constants = build_math_constants();
    assert_eq!(constants.len(), 214);
    let glyph_info = build_math_glyph_info(f_gid);
    let variants = build_math_variants(paren_gid, close_gid, tall_paren_gid, extender_gid);
    let constants_off = 10u16;
    let glyph_info_off = constants_off + constants.len() as u16;
    let variants_off = glyph_info_off + glyph_info.len() as u16;
    let mut b = Vec::new();
    put_u16(&mut b, 1);
    put_u16(&mut b, 0);
    put_u16(&mut b, constants_off);
    put_u16(&mut b, glyph_info_off);
    put_u16(&mut b, variants_off);
    b.extend_from_slice(&constants);
    b.extend_from_slice(&glyph_info);
    b.extend_from_slice(&variants);
    b
}

fn build_math_glyph_info(f_gid: u16) -> Vec<u8> {
    // ItalicsCorrectionInfo (MathValues): 14 bytes. KernInfo: 30 bytes.
    // GlyphInfo header: 8 bytes. Total 52.
    let mut italics = Vec::new();
    put_u16(&mut italics, 8);
    put_u16(&mut italics, 1);
    italics.extend_from_slice(&math_value(FIXTURE_ITALIC_CORRECTION_F));
    put_u16(&mut italics, 1);
    put_u16(&mut italics, 1);
    put_u16(&mut italics, f_gid);
    debug_assert_eq!(italics.len(), 14);

    let mut kern = Vec::new();
    put_u16(&mut kern, 24);
    put_u16(&mut kern, 1);
    put_u16(&mut kern, 12);
    put_u16(&mut kern, 0);
    put_u16(&mut kern, 18);
    put_u16(&mut kern, 0);
    put_u16(&mut kern, 0);
    kern.extend_from_slice(&math_value(FIXTURE_MATH_KERN_TOP_RIGHT));
    put_u16(&mut kern, 0);
    kern.extend_from_slice(&math_value(FIXTURE_MATH_KERN_BOTTOM_RIGHT));
    put_u16(&mut kern, 1);
    put_u16(&mut kern, 1);
    put_u16(&mut kern, f_gid);
    debug_assert_eq!(kern.len(), 30);

    let mut b = Vec::new();
    put_u16(&mut b, 8);
    put_u16(&mut b, 0);
    put_u16(&mut b, 0);
    put_u16(&mut b, 8 + italics.len() as u16);
    b.extend_from_slice(&italics);
    b.extend_from_slice(&kern);
    debug_assert_eq!(b.len(), 52);
    b
}

fn math_value(value: i16) -> [u8; 4] {
    let mut a = [0u8; 4];
    a[..2].copy_from_slice(&value.to_be_bytes());
    a
}

fn build_math_constants() -> Vec<u8> {
    let mut c = vec![0u8; 214];
    c[0..2].copy_from_slice(&FIXTURE_SCRIPT_PERCENT_SCALE_DOWN.to_be_bytes());
    c[2..4].copy_from_slice(&FIXTURE_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN.to_be_bytes());
    c[4..6].copy_from_slice(&FIXTURE_DELIMITED_SUB_FORMULA_MIN_HEIGHT.to_be_bytes());
    c[6..8].copy_from_slice(&FIXTURE_DISPLAY_OPERATOR_MIN_HEIGHT.to_be_bytes());
    let records: [(usize, i16); 50] = [
        (8, 100),
        (12, FIXTURE_AXIS_HEIGHT),
        (16, FIXTURE_ACCENT_BASE_HEIGHT),
        (20, 680),
        (24, FIXTURE_SUBSCRIPT_SHIFT_DOWN),
        (28, 380),
        (32, 80),
        (36, FIXTURE_SUPERSCRIPT_SHIFT_UP),
        (40, 280),
        (44, 80),
        (48, 250),
        (52, 150),
        (56, 240),
        (60, FIXTURE_SPACE_AFTER_SCRIPT),
        (64, FIXTURE_UPPER_LIMIT_GAP_MIN),
        (68, FIXTURE_UPPER_LIMIT_BASELINE_RISE_MIN),
        (72, FIXTURE_LOWER_LIMIT_GAP_MIN),
        (76, FIXTURE_LOWER_LIMIT_BASELINE_DROP_MIN),
        (80, 300),
        (84, 400),
        (88, 200),
        (92, 300),
        (96, FIXTURE_STACK_GAP_MIN),
        (100, FIXTURE_STACK_DISPLAY_STYLE_GAP_MIN),
        (104, 300),
        (108, 200),
        (112, 80),
        (116, 80),
        (120, FIXTURE_FRACTION_NUMERATOR_SHIFT_UP),
        (124, FIXTURE_FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP),
        (128, FIXTURE_FRACTION_DENOMINATOR_SHIFT_DOWN),
        (132, FIXTURE_FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN),
        (136, 40),
        (140, 80),
        (144, FIXTURE_FRACTION_RULE_THICKNESS),
        (148, 40),
        (152, 80),
        (156, 80),
        (160, 80),
        (164, FIXTURE_OVERBAR_VERTICAL_GAP),
        (168, FIXTURE_OVERBAR_RULE_THICKNESS),
        (172, FIXTURE_OVERBAR_EXTRA_ASCENDER),
        (176, FIXTURE_UNDERBAR_VERTICAL_GAP),
        (180, FIXTURE_UNDERBAR_RULE_THICKNESS),
        (184, FIXTURE_UNDERBAR_EXTRA_DESCENDER),
        (188, FIXTURE_RADICAL_VERTICAL_GAP),
        (192, FIXTURE_RADICAL_DISPLAY_STYLE_VERTICAL_GAP),
        (196, FIXTURE_RADICAL_RULE_THICKNESS),
        (200, FIXTURE_RADICAL_EXTRA_ASCENDER),
        (204, FIXTURE_RADICAL_KERN_BEFORE_DEGREE),
    ];
    for (off, v) in records {
        c[off..off + 4].copy_from_slice(&math_value(v));
    }
    c[208..212].copy_from_slice(&math_value(FIXTURE_RADICAL_KERN_AFTER_DEGREE));
    c[212..214].copy_from_slice(&FIXTURE_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT.to_be_bytes());
    c
}

/// MATH Variants Coverage Format 1 for two glyphs: format + count + glyphIds.
const MATH_VARIANTS_COVERAGE_LEN: u16 = 8;
/// After minConnectorOverlap, two coverage offsets, two counts, two vert offsets.
const MATH_VARIANTS_COVERAGE_OFFSET: u16 = 14;
const MATH_VARIANTS_CONSTRUCTION_OFFSET: u16 =
    MATH_VARIANTS_COVERAGE_OFFSET + MATH_VARIANTS_COVERAGE_LEN;
const MATH_GLYPH_CONSTRUCTION_LEN: u16 = 48;

fn build_math_variants(
    paren_gid: u16,
    close_gid: u16,
    tall_paren_gid: u16,
    extender_gid: u16,
) -> Vec<u8> {
    // Offsets are relative to the start of the MATH Variants table.
    let mut gids = [paren_gid, close_gid];
    gids.sort_unstable();
    let mut v = Vec::new();
    put_u16(&mut v, 50);
    put_u16(&mut v, MATH_VARIANTS_COVERAGE_OFFSET);
    put_u16(&mut v, 0);
    put_u16(&mut v, 2);
    put_u16(&mut v, 0);
    put_u16(&mut v, MATH_VARIANTS_CONSTRUCTION_OFFSET);
    put_u16(
        &mut v,
        MATH_VARIANTS_CONSTRUCTION_OFFSET + MATH_GLYPH_CONSTRUCTION_LEN,
    );
    put_u16(&mut v, 1);
    put_u16(&mut v, 2);
    put_u16(&mut v, gids[0]);
    put_u16(&mut v, gids[1]);
    debug_assert_eq!(v.len(), MATH_VARIANTS_CONSTRUCTION_OFFSET as usize);
    push_paren_construction(&mut v, gids[0], tall_paren_gid, extender_gid);
    push_paren_construction(&mut v, gids[1], tall_paren_gid, extender_gid);
    debug_assert_eq!(
        v.len(),
        (MATH_VARIANTS_CONSTRUCTION_OFFSET + 2 * MATH_GLYPH_CONSTRUCTION_LEN) as usize
    );
    v
}

fn push_paren_construction(v: &mut Vec<u8>, base_gid: u16, tall_gid: u16, extender_gid: u16) {
    // GlyphAssemblyOffset is relative to this construction table.
    put_u16(v, 12);
    put_u16(v, 2);
    put_u16(v, base_gid);
    put_u16(v, FIXTURE_PAREN_VARIANT_ADVANCE);
    put_u16(v, tall_gid);
    put_u16(v, FIXTURE_TALL_PAREN_VARIANT_ADVANCE);
    v.extend_from_slice(&math_value(0));
    put_u16(v, 3);
    push_glyph_part(v, base_gid, 0, 80, FIXTURE_ASSEMBLY_END_ADVANCE, 0);
    push_glyph_part(
        v,
        extender_gid,
        80,
        80,
        FIXTURE_EXTENDER_ADVANCE,
        1, // extender
    );
    push_glyph_part(v, tall_gid, 80, 0, FIXTURE_ASSEMBLY_END_ADVANCE, 0);
}

fn push_glyph_part(
    v: &mut Vec<u8>,
    glyph_id: u16,
    start_conn: u16,
    end_conn: u16,
    full_advance: u16,
    flags: u16,
) {
    put_u16(v, glyph_id);
    put_u16(v, start_conn);
    put_u16(v, end_conn);
    put_u16(v, full_advance);
    put_u16(v, flags);
}

fn put_u16(b: &mut Vec<u8>, v: u16) {
    b.extend_from_slice(&v.to_be_bytes());
}

fn put_i16(b: &mut Vec<u8>, v: i16) {
    b.extend_from_slice(&v.to_be_bytes());
}

fn put_u32(b: &mut Vec<u8>, v: u32) {
    b.extend_from_slice(&v.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use ttf_parser::Face;

    #[test]
    fn math_vertical_construction_exposes_tall_paren() {
        let face = Face::parse(fixture_font_bytes(), 0).expect("fixture face");
        let paren = face.glyph_index('(').expect("paren cmap");
        let variants = face
            .tables()
            .math
            .expect("MATH")
            .variants
            .expect("MATH variants");
        let cons = variants
            .vertical_constructions
            .get(paren)
            .expect("vertical construction for '('");
        assert_eq!(cons.variants.len(), 2);
        let base = cons.variants.get(0).expect("base variant");
        let tall = cons.variants.get(1).expect("tall variant");
        assert_eq!(base.variant_glyph, paren);
        assert_eq!(base.advance_measurement, FIXTURE_PAREN_VARIANT_ADVANCE);
        assert_ne!(tall.variant_glyph, paren);
        assert_eq!(tall.advance_measurement, FIXTURE_TALL_PAREN_VARIANT_ADVANCE);
        assert!(face.glyph_hor_advance(tall.variant_glyph).is_some());
        let assembly = cons.assembly.expect("glyph assembly");
        assert_eq!(assembly.parts.len(), 3);
        assert!(assembly
            .parts
            .get(1)
            .expect("extender")
            .part_flags
            .extender());
    }

    #[test]
    fn math_glyph_info_exposes_f_italic_and_kern() {
        let face = Face::parse(fixture_font_bytes(), 0).expect("fixture face");
        let f = face.glyph_index('f').expect("f cmap");
        let info = face
            .tables()
            .math
            .expect("MATH")
            .glyph_info
            .expect("glyph info");
        let ic = info
            .italic_corrections
            .expect("italic")
            .get(f)
            .expect("f italic")
            .value;
        assert_eq!(ic, FIXTURE_ITALIC_CORRECTION_F);
        let ki = info.kern_infos.expect("kern infos").get(f).expect("f kern");
        assert_eq!(
            ki.top_right.expect("top right").kern(0).expect("tr").value,
            FIXTURE_MATH_KERN_TOP_RIGHT
        );
        assert_eq!(
            ki.bottom_right
                .expect("bottom right")
                .kern(0)
                .expect("br")
                .value,
            FIXTURE_MATH_KERN_BOTTOM_RIGHT
        );
        assert!(ki.top_left.is_none());
        assert!(ki.bottom_left.is_none());
    }
}
