//! Font resource identity, face load, and glyph metrics.

use std::path::Path;
use std::sync::OnceLock;

use ttf_parser::{Face, GlyphId};

use crate::error::LayoutError;
use crate::fixture::fixture_font_bytes;

/// Stable font identity: human label plus a content digest of the face bytes.
///
/// The digest is the ABI/resource identity (spec §19 spirit). Native handles
/// and filesystem paths are not identities.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontId {
    pub label: String,
    pub digest: String,
}

impl FontId {
    pub fn as_key(&self) -> String {
        format!("{}#{}", self.label, self.digest)
    }
}

/// FNV-1a 64 plus length; stable, offline, not a cryptographic claim.
pub fn content_digest(bytes: &[u8]) -> String {
    let mut h = 0xcbf29ce484222325u64;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{:016x}-{:x}", h, bytes.len())
}

/// Owned font bytes plus identity. Parse [`Face`] per call (no self-ref).
#[derive(Debug, Clone)]
pub struct LoadedFont {
    pub id: FontId,
    bytes: Vec<u8>,
    /// TrueType Collection face index (0 for a single-face TTF).
    face_index: u32,
}

impl LoadedFont {
    pub fn from_bytes(bytes: Vec<u8>, label: impl Into<String>) -> Result<Self, LayoutError> {
        let n = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1).max(1);
        let mut last_err = None;
        for i in 0..n {
            match Face::parse(&bytes, i) {
                Ok(_) => {
                    let digest = content_digest(&bytes);
                    return Ok(Self {
                        id: FontId {
                            label: label.into(),
                            digest,
                        },
                        bytes,
                        face_index: i,
                    });
                }
                Err(e) => last_err = Some(e),
            }
        }
        Err(LayoutError::InvalidFont {
            detail: last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "no parseable face in font bytes".into()),
        })
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, LayoutError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| LayoutError::InvalidFont {
            detail: format!("read {}: {e}", path.display()),
        })?;
        let label = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("runtime-font")
            .to_string();
        Self::from_bytes(bytes, label)
    }

    /// Pinned redistributable fixture (tests and offline default).
    pub fn fixture() -> Self {
        static FONT: OnceLock<LoadedFont> = OnceLock::new();
        FONT.get_or_init(|| {
            Self::from_bytes(fixture_font_bytes().to_vec(), "ReciplexaFixture")
                .expect("pinned fixture TTF must parse")
        })
        .clone()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn face_index(&self) -> u32 {
        self.face_index
    }

    pub fn face(&self) -> Face<'_> {
        Face::parse(&self.bytes, self.face_index).expect("LoadedFont bytes were validated")
    }

    pub fn is_fixture(&self) -> bool {
        self.id.label == "ReciplexaFixture"
    }

    pub fn units_per_em(&self) -> u16 {
        self.face().units_per_em()
    }

    pub fn glyph_id(&self, c: char) -> Result<u16, LayoutError> {
        self.face()
            .glyph_index(c)
            .map(|g| g.0)
            .ok_or_else(|| LayoutError::MissingGlyph {
                font_id: self.id.as_key(),
                scalar: c,
            })
    }

    pub fn hor_advance_units(&self, gid: u16) -> Result<u16, LayoutError> {
        self.face()
            .glyph_hor_advance(GlyphId(gid))
            .ok_or_else(|| LayoutError::Engine {
                detail: format!("missing hmtx for gid {gid}"),
            })
    }

    /// Horizontal advance in em (design units / units-per-em).
    pub fn hor_advance_em(&self, c: char) -> Result<f64, LayoutError> {
        let gid = self.glyph_id(c)?;
        self.hor_advance_em_gid(gid)
    }

    pub fn hor_advance_em_gid(&self, gid: u16) -> Result<f64, LayoutError> {
        let adv = self.hor_advance_units(gid)?;
        let upem = f64::from(self.units_per_em().max(1));
        Ok(f64::from(adv) / upem)
    }

    /// Ink height (above baseline) and depth (below) in em from glyf bbox.
    pub fn glyph_ink_em(&self, gid: u16) -> Result<(f64, f64), LayoutError> {
        let bbox = self
            .face()
            .glyph_bounding_box(GlyphId(gid))
            .ok_or_else(|| LayoutError::Engine {
                detail: format!("missing glyf bbox for gid {gid}"),
            })?;
        let upem = f64::from(self.units_per_em().max(1));
        let height = f64::from(bbox.y_max.max(0)) / upem;
        let depth = f64::from((-bbox.y_min).max(0)) / upem;
        Ok((height, depth))
    }

    pub fn ver_advance_em_gid(&self, gid: u16) -> f64 {
        let upem = f64::from(self.units_per_em().max(1));
        self.face()
            .glyph_ver_advance(GlyphId(gid))
            .map(|v| f64::from(v) / upem)
            .filter(|v| *v > 0.0)
            .unwrap_or(1.0)
    }

    pub fn italic_correction_em(&self, c: char) -> Result<f64, LayoutError> {
        let gid = self.glyph_id(c)?;
        Ok(self.italic_correction_gid_em(gid))
    }

    /// MATH italic correction for a GID, or 0 when the table/glyph is absent.
    pub fn italic_correction_gid_em(&self, gid: u16) -> f64 {
        let face = self.face();
        let Some(math) = face.tables().math else {
            return 0.0;
        };
        let Some(info) = math.glyph_info else {
            return 0.0;
        };
        let Some(table) = info.italic_corrections else {
            return 0.0;
        };
        let Some(v) = table.get(GlyphId(gid)) else {
            return 0.0;
        };
        let upem = f64::from(self.units_per_em().max(1));
        f64::from(v.value) / upem
    }

    /// Height-dependent MATH corner kern in em, or 0 when absent.
    pub fn math_kern_em(&self, gid: u16, corner: MathKernCorner, height_em: f64) -> f64 {
        let face = self.face();
        let Some(math) = face.tables().math else {
            return 0.0;
        };
        let Some(info) = math.glyph_info else {
            return 0.0;
        };
        let Some(kerns) = info.kern_infos else {
            return 0.0;
        };
        let Some(ki) = kerns.get(GlyphId(gid)) else {
            return 0.0;
        };
        let table = match corner {
            MathKernCorner::TopRight => ki.top_right,
            MathKernCorner::TopLeft => ki.top_left,
            MathKernCorner::BottomRight => ki.bottom_right,
            MathKernCorner::BottomLeft => ki.bottom_left,
        };
        let Some(kern) = table else {
            return 0.0;
        };
        let upem = f64::from(self.units_per_em().max(1));
        let height_du = (height_em * upem).round() as i16;
        let n = kern.count();
        let mut idx = n;
        for i in 0..n {
            let Some(h) = kern.height(i) else {
                break;
            };
            if height_du < h.value {
                idx = i;
                break;
            }
        }
        let Some(v) = kern.kern(idx) else {
            return 0.0;
        };
        f64::from(v.value) / upem
    }
}

/// MATH `MathKernInfo` corner used for script / limit attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathKernCorner {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// Runtime Japanese face: `RECIPLEXA_CJK_FONT`, else a local system CJK file,
/// else the pinned fixture.
///
/// System fonts are not test identities. Unit tests should call [`LoadedFont::fixture`].
pub fn host_product_font() -> Result<LoadedFont, LayoutError> {
    if let Ok(path) = std::env::var("RECIPLEXA_CJK_FONT") {
        if !path.is_empty() {
            return LoadedFont::from_path(path);
        }
    }
    if let Some(path) = probe_system_cjk_font() {
        if let Ok(font) = LoadedFont::from_path(&path) {
            return Ok(font);
        }
    }
    Ok(LoadedFont::fixture())
}

/// MATH face: `RECIPLEXA_MATH_FONT` when set, else the fixture (has a MATH table).
///
/// Ordinary CJK faces typically lack MATH; do not silently reuse them.
pub fn host_math_font() -> Result<LoadedFont, LayoutError> {
    if let Ok(path) = std::env::var("RECIPLEXA_MATH_FONT") {
        if !path.is_empty() {
            return LoadedFont::from_path(path);
        }
    }
    Ok(LoadedFont::fixture())
}

fn probe_system_cjk_font() -> Option<std::path::PathBuf> {
    let windir = std::env::var_os("WINDIR").unwrap_or_else(|| r"C:\Windows".into());
    let fonts = std::path::PathBuf::from(windir).join("Fonts");
    for name in [
        "YuGothR.ttc",
        "BIZ-UDGothicR.ttc",
        "meiryo.ttc",
        "msgothic.ttc",
        "NotoSans-Regular.ttf",
        "NotoSansJP-VF.ttf",
    ] {
        let p = fonts.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
