//! Embed a subset CJK TrueType as a selectable CID/Type0 font (Identity-H + ToUnicode).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use subsetter::{subset, GlyphRemapper};
use ttf_parser::{Face, GlyphId};

use crate::PdfError;

/// Subset TTF + CID maps for `/Identity-H` text that is copyable in PDF readers.
pub struct CjkFontEmbed {
    pub subset_ttf: Vec<u8>,
    /// Unicode scalar → remapped CID (== subset GID).
    pub unicode_to_cid: BTreeMap<u32, u16>,
    /// CID → advance width in 1000-em PDF units.
    pub cid_widths: BTreeMap<u16, u16>,
    pub font_bbox: [i32; 4],
    pub ascent: i32,
    pub descent: i32,
    pub base_name: String,
}

impl CjkFontEmbed {
    pub fn build(chars: &BTreeSet<char>) -> Result<Self, PdfError> {
        if chars.is_empty() {
            return Err(PdfError::InvalidShape(
                "internal: empty CJK char set".into(),
            ));
        }
        let path = system_cjk_font_path().ok_or_else(|| {
            PdfError::InvalidShape(
                "non-ASCII text needs a CJK font (set RECIPLEXA_CJK_FONT or install Yu Gothic / Noto Sans JP)"
                    .into(),
            )
        })?;
        let data = std::fs::read(&path)
            .map_err(|e| PdfError::InvalidShape(format!("read font {}: {e}", path.display())))?;
        let face = Face::parse(&data, 0)
            .map_err(|e| PdfError::InvalidShape(format!("parse font {}: {e}", path.display())))?;
        let units = u32::from(face.units_per_em());
        if units == 0 {
            return Err(PdfError::InvalidShape("font units_per_em is zero".into()));
        }

        let mut remapper = GlyphRemapper::new();
        remapper.remap(0); // .notdef

        let mut unicode_to_old: BTreeMap<u32, u16> = BTreeMap::new();
        for ch in chars {
            if ch.is_control() {
                continue;
            }
            let Some(gid) = face.glyph_index(*ch) else {
                return Err(PdfError::InvalidShape(format!(
                    "font missing glyph for U+{:04X}",
                    *ch as u32
                )));
            };
            remapper.remap(gid.0);
            unicode_to_old.insert(*ch as u32, gid.0);
        }

        let subset_ttf = subset(&data, 0, &remapper)
            .map_err(|e| PdfError::InvalidShape(format!("subset font {}: {e}", path.display())))?;

        let mut unicode_to_cid = BTreeMap::new();
        let mut cid_widths = BTreeMap::new();
        for (uni, old_gid) in unicode_to_old {
            let cid = remapper.get(old_gid).ok_or_else(|| {
                PdfError::InvalidShape(format!("missing remapped CID for GID {old_gid}"))
            })?;
            unicode_to_cid.insert(uni, cid);
            let adv = face
                .glyph_hor_advance(GlyphId(old_gid))
                .map(u32::from)
                .unwrap_or(units / 2);
            let w = ((adv * 1000) / units) as u16;
            cid_widths.insert(cid, w);
        }
        if let Some(cid0) = remapper.get(0) {
            let adv = face
                .glyph_hor_advance(GlyphId(0))
                .map(u32::from)
                .unwrap_or(units / 2);
            cid_widths.insert(cid0, ((adv * 1000) / units) as u16);
        }

        let scale = 1000.0 / f64::from(units);
        let (bbox, ascent, descent) = metrics_1000(&face, scale);

        // Tag must be 6 uppercase A–Z for PDF subset fonts.
        let tag = subset_tag(&subset_ttf);
        let base_name = format!("{tag}+ReciplexaCJK");

        Ok(Self {
            subset_ttf,
            unicode_to_cid,
            cid_widths,
            font_bbox: bbox,
            ascent,
            descent,
            base_name,
        })
    }

    pub fn encode_hex(&self, content: &str) -> Result<String, PdfError> {
        let mut hex = String::with_capacity(content.len() * 4);
        for ch in content.chars() {
            if ch == '\n' || ch == '\r' {
                return Err(PdfError::InvalidShape(
                    "encode_hex expects a single line (no embedded newlines)".into(),
                ));
            }
            let cid = self
                .unicode_to_cid
                .get(&(ch as u32))
                .ok_or_else(|| PdfError::InvalidShape(format!("no CID for U+{:04X}", ch as u32)))?;
            hex.push_str(&format!("{cid:04X}"));
        }
        Ok(hex)
    }

    pub fn to_unicode_cmap(&self) -> String {
        let mut pairs: Vec<(u16, u32)> =
            self.unicode_to_cid.iter().map(|(u, c)| (*c, *u)).collect();
        pairs.sort_by_key(|(c, _)| *c);

        let mut out = String::from(
            "/CIDInit /ProcSet findresource begin\n\
             12 dict begin\n\
             begincmap\n\
             /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
             /CMapName /Adobe-Identity-UCS def\n\
             /CMapType 2 def\n\
             1 begincodespacerange\n\
             <0000> <FFFF>\n\
             endcodespacerange\n",
        );
        for chunk in pairs.chunks(100) {
            out.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for (cid, uni) in chunk {
                out.push_str(&format!("<{cid:04X}> {}\n", utf16_hex(*uni)));
            }
            out.push_str("endbfchar\n");
        }
        out.push_str(
            "endcmap\n\
             CMapName currentdict /CMap defineresource pop\n\
             end\n\
             end",
        );
        out
    }

    pub fn widths_array(&self) -> String {
        let mut out = String::from("[ ");
        for (cid, w) in &self.cid_widths {
            out.push_str(&format!("{cid} [{w}] "));
        }
        out.push(']');
        out
    }
}

fn utf16_hex(uni: u32) -> String {
    if uni <= 0xFFFF {
        format!("<{uni:04X}>")
    } else {
        let u = uni - 0x10000;
        let hi = 0xD800 + (u >> 10);
        let lo = 0xDC00 + (u & 0x3FF);
        format!("<{hi:04X}{lo:04X}>")
    }
}

fn metrics_1000(face: &Face<'_>, scale: f64) -> ([i32; 4], i32, i32) {
    let bbox = face.global_bounding_box();
    let font_bbox = [
        (f64::from(bbox.x_min) * scale).round() as i32,
        (f64::from(bbox.y_min) * scale).round() as i32,
        (f64::from(bbox.x_max) * scale).round() as i32,
        (f64::from(bbox.y_max) * scale).round() as i32,
    ];
    let ascent = (f64::from(face.ascender()) * scale).round() as i32;
    let descent = (f64::from(face.descender()) * scale).round() as i32;
    (font_bbox, ascent, descent)
}

fn subset_tag(bytes: &[u8]) -> String {
    // Deterministic 6-letter tag from content (PDF subset naming convention).
    let mut h: u32 = 2166136261;
    for b in bytes.iter().take(4096) {
        h ^= u32::from(*b);
        h = h.wrapping_mul(16777619);
    }
    let mut s = String::with_capacity(6);
    for i in 0..6 {
        let v = ((h >> (i * 5)) & 31) as u8;
        s.push(char::from(b'A' + v % 26));
    }
    s
}

pub fn system_cjk_font_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("RECIPLEXA_CJK_FONT") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    let windir = std::env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into());
    let fonts = PathBuf::from(windir).join("Fonts");
    // Prefer static faces for reliable subsetting; VF last.
    for name in [
        "YuGothR.ttc",
        "BIZ-UDGothicR.ttc",
        "meiryo.ttc",
        "msgothic.ttc",
        "NotoSans-Regular.ttf",
        "NotoSansJP-VF.ttf",
        "NotoSansJP-VariableFont_wght.ttf",
    ] {
        let p = fonts.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Public alias for CLI / tests.
pub fn cjk_font_path() -> Option<PathBuf> {
    system_cjk_font_path()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn subset_tag_is_six_uppercase_letters() {
        let tag = subset_tag(b"hello");
        assert_eq!(tag.len(), 6);
        assert!(tag.chars().all(|c| c.is_ascii_uppercase()));
        assert_eq!(tag, subset_tag(b"hello"));
        assert_ne!(tag, subset_tag(b"world"));
    }

    #[test]
    fn utf16_hex_bmp_and_supplementary() {
        assert_eq!(utf16_hex(0x0041), "<0041>");
        assert_eq!(utf16_hex(0x1F600), "<D83DDE00>");
    }

    #[test]
    fn build_rejects_empty_char_set() {
        let err = match CjkFontEmbed::build(&BTreeSet::new()) {
            Err(e) => e,
            Ok(_) => panic!("expected error"),
        };
        assert!(matches!(err, PdfError::InvalidShape(_)));
    }

    #[test]
    fn encode_hex_rejects_embedded_newlines() {
        if system_cjk_font_path().is_none() {
            return;
        }
        let mut chars = BTreeSet::new();
        chars.insert('日');
        let embed = CjkFontEmbed::build(&chars).unwrap();
        let err = embed.encode_hex("a\nb").unwrap_err();
        assert!(matches!(err, PdfError::InvalidShape(_)));
    }

    #[test]
    fn cjk_roundtrip_widths_and_cmap_when_font_available() {
        if system_cjk_font_path().is_none() {
            return;
        }
        let mut chars = BTreeSet::new();
        chars.insert('あ');
        chars.insert('語');
        let embed = CjkFontEmbed::build(&chars).unwrap();
        let hex = embed.encode_hex("あ").unwrap();
        assert!(!hex.is_empty());
        let cmap = embed.to_unicode_cmap();
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("endbfchar"));
        let widths = embed.widths_array();
        assert!(widths.starts_with("[ "));
        assert!(widths.ends_with(']'));
        assert!(!embed.base_name.is_empty());
        assert!(embed.font_bbox[2] >= embed.font_bbox[0]);
    }

    #[test]
    fn cjk_font_path_alias_matches_system() {
        assert_eq!(cjk_font_path(), system_cjk_font_path());
    }

    #[test]
    fn encode_hex_unknown_char_errors_when_font_available() {
        if system_cjk_font_path().is_none() {
            return;
        }
        let mut chars = BTreeSet::new();
        chars.insert('あ');
        let embed = CjkFontEmbed::build(&chars).unwrap();
        let err = match embed.encode_hex("あX") {
            Err(e) => e,
            Ok(_) => panic!("expected missing CID error"),
        };
        assert!(matches!(err, PdfError::InvalidShape(_)));
    }
}
