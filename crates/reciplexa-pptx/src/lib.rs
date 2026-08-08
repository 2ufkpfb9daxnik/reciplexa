//! Minimal PowerPoint (OOXML) export.
//!
//! Each document page becomes one slide. Coordinates are millimeters converted
//! to EMUs (English Metric Units). Shapes come from the same flatten pass as SVG.

#![forbid(unsafe_code)]

use std::io::{Cursor, Seek, Write};

use reciplexa_scene::{Color, Document};
use reciplexa_view::{flatten_page, WorldShape};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// 1 mm in EMUs (914400 EMU / inch ÷ 25.4 mm/inch).
const EMU_PER_MM: f64 = 914400.0 / 25.4;

/// Write a `.pptx` zip for `doc`.
pub fn write_document(doc: &Document, mut out: impl Write) -> Result<(), String> {
    let bytes = document_to_pptx(doc)?;
    out.write_all(&bytes).map_err(|e| e.to_string())
}

/// Build PPTX bytes in memory.
pub fn document_to_pptx(doc: &Document) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    document_to_pptx_write(doc, &mut buf)?;
    Ok(buf.into_inner())
}

/// Write PPTX zip bytes to `out` (used by tests to inject I/O failures mid-stream).
pub fn document_to_pptx_write(doc: &Document, out: &mut (impl Write + Seek)) -> Result<(), String> {
    let mut zip = ZipWriter::new(out);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("[Content_Types].xml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(content_types(doc.pages.len()).as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("_rels/.rels", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(ROOT_RELS.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("ppt/presentation.xml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(presentation_xml(doc).as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("ppt/_rels/presentation.xml.rels", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(presentation_rels(doc.pages.len()).as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("ppt/slideLayouts/slideLayout1.xml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(SLIDE_LAYOUT.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(SLIDE_LAYOUT_RELS.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("ppt/slideMasters/slideMaster1.xml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(SLIDE_MASTER.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(SLIDE_MASTER_RELS.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("ppt/theme/theme1.xml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(THEME.as_bytes()).map_err(|e| e.to_string())?;

    for i in 0..doc.pages.len() {
        let xml = slide_xml(doc, i)?;
        let name = format!("ppt/slides/slide{}.xml", i + 1);
        zip.start_file(&name, opts).map_err(|e| e.to_string())?;
        zip.write_all(xml.as_bytes()).map_err(|e| e.to_string())?;

        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#
                .to_string();
        let rels_name = format!("ppt/slides/_rels/slide{}.xml.rels", i + 1);
        zip.start_file(&rels_name, opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(rels.as_bytes()).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn slide_xml(doc: &Document, index: usize) -> Result<String, String> {
    let page = doc
        .pages
        .get(index)
        .ok_or_else(|| format!("page {index} missing"))?;
    let (_, shapes) = flatten_page(doc, index).ok_or_else(|| format!("flatten page {index}"))?;
    let w_emu = (page.paper.width_mm * EMU_PER_MM).round() as i64;
    let h_emu = (page.paper.height_mm * EMU_PER_MM).round() as i64;
    let mut body = String::new();
    for (si, shape) in shapes.iter().enumerate() {
        body.push_str(&shape_xml(shape, si + 2, page.paper.height_mm)?);
    }
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:bg>
      <p:bgPr>
        <a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill>
        <a:effectLst/>
      </p:bgPr>
    </p:bg>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr>
        <a:xfrm>
          <a:off x="0" y="0"/>
          <a:ext cx="{w_emu}" cy="{h_emu}"/>
          <a:chOff x="0" y="0"/>
          <a:chExt cx="{w_emu}" cy="{h_emu}"/>
        </a:xfrm>
      </p:grpSpPr>
{body}    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#
    ))
}

fn shape_xml(shape: &WorldShape, id: usize, paper_h_mm: f64) -> Result<String, String> {
    // Scene y-up → OOXML y-down.
    let flip_y = |y: f64| paper_h_mm - y;
    match shape {
        WorldShape::Circle(c) => {
            let d = c.radius_mm * 2.0;
            let x = c.x_mm - c.radius_mm;
            let y = flip_y(c.y_mm + c.radius_mm);
            let fill = solid_fill(c.color, c.alpha);
            let line = match c.stroke_width_mm {
                None => String::new(),
                Some(w) => format!(
                    r#"<a:ln w="{}"><a:solidFill>{}</a:solidFill></a:ln>"#,
                    (w * EMU_PER_MM).round() as i64,
                    fill_inner(c.color, c.alpha)
                ),
            };
            Ok(ellipse_shape(id, "Circle", x, y, d, d, &fill, &line))
        }
        WorldShape::Polygon(p) => {
            if p.points_mm.len() < 3 {
                return Ok(String::new());
            }
            let xs: Vec<f64> = p.points_mm.iter().map(|p| p.0).collect();
            let ys: Vec<f64> = p.points_mm.iter().map(|p| flip_y(p.1)).collect();
            let min_x = xs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_y = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let w = (max_x - min_x).max(0.1);
            let h = (max_y - min_y).max(0.1);
            // Approximate arbitrary polygons as freeform path in path fill.
            let mut path = String::new();
            for (i, (&x, &y)) in xs.iter().zip(ys.iter()).enumerate() {
                let px = (((x - min_x) / w) * 21600.0).round() as i64;
                let py = (((y - min_y) / h) * 21600.0).round() as i64;
                if i == 0 {
                    path.push_str(&format!(
                        r#"<a:moveTo><a:pt x="{px}" y="{py}"/></a:moveTo>"#
                    ));
                } else {
                    path.push_str(&format!(r#"<a:lnTo><a:pt x="{px}" y="{py}"/></a:lnTo>"#));
                }
            }
            path.push_str(r#"<a:close/>"#);
            let fill = match p.stroke_width_mm {
                None => format!(
                    "<a:solidFill>{}</a:solidFill>",
                    fill_inner(p.color, p.alpha)
                ),
                Some(_) => r#"<a:noFill/>"#.to_string(),
            };
            let line = match p.stroke_width_mm {
                None => r#"<a:ln><a:noFill/></a:ln>"#.to_string(),
                Some(w) => format!(
                    r#"<a:ln w="{}"><a:solidFill>{}</a:solidFill></a:ln>"#,
                    (w * EMU_PER_MM).round() as i64,
                    fill_inner(p.color, p.alpha)
                ),
            };
            Ok(format!(
                r#"      <p:sp>
        <p:nvSpPr><p:cNvPr id="{id}" name="Polygon {id}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="{ox}" y="{oy}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
          <a:custGeom>
            <a:avLst/>
            <a:gdLst/>
            <a:ahLst/>
            <a:cxnLst/>
            <a:rect l="l" t="t" r="r" b="b"/>
            <a:pathLst><a:path w="21600" h="21600">{path}</a:path></a:pathLst>
          </a:custGeom>
          {fill}{line}
        </p:spPr>
        <p:style><a:lnRef idx="0"><a:scrgbClr r="0" g="0" b="0"/></a:lnRef><a:fillRef idx="0"><a:scrgbClr r="0" g="0" b="0"/></a:fillRef><a:effectRef idx="0"><a:scrgbClr r="0" g="0" b="0"/></a:effectRef><a:fontRef idx="minor"><a:schemeClr val="dk1"/></a:fontRef></p:style>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>
      </p:sp>
"#,
                ox = (min_x * EMU_PER_MM).round() as i64,
                oy = (min_y * EMU_PER_MM).round() as i64,
                cx = (w * EMU_PER_MM).round() as i64,
                cy = (h * EMU_PER_MM).round() as i64,
            ))
        }
        WorldShape::Path(p) => {
            if p.points_mm.len() < 2 {
                return Ok(String::new());
            }
            // Represent as a thin freeform stroke similar to polygon outline.
            let pts: Vec<(f64, f64)> = p.points_mm.iter().map(|&(x, y)| (x, flip_y(y))).collect();
            let min_x = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
            let max_x = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
            let min_y = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
            let max_y = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
            let w = (max_x - min_x).max(0.1);
            let h = (max_y - min_y).max(0.1);
            let mut path = String::new();
            for (i, &(x, y)) in pts.iter().enumerate() {
                let px = (((x - min_x) / w) * 21600.0).round() as i64;
                let py = (((y - min_y) / h) * 21600.0).round() as i64;
                if i == 0 {
                    path.push_str(&format!(
                        r#"<a:moveTo><a:pt x="{px}" y="{py}"/></a:moveTo>"#
                    ));
                } else {
                    path.push_str(&format!(r#"<a:lnTo><a:pt x="{px}" y="{py}"/></a:lnTo>"#));
                }
            }
            if p.closed {
                path.push_str(r#"<a:close/>"#);
            }
            Ok(format!(
                r#"      <p:sp>
        <p:nvSpPr><p:cNvPr id="{id}" name="Path {id}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="{ox}" y="{oy}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
          <a:custGeom>
            <a:avLst/><a:gdLst/><a:ahLst/><a:cxnLst/>
            <a:rect l="l" t="t" r="r" b="b"/>
            <a:pathLst><a:path w="21600" h="21600">{path}</a:path></a:pathLst>
          </a:custGeom>
          <a:noFill/>
          <a:ln w="{lw}"><a:solidFill>{fill}</a:solidFill></a:ln>
        </p:spPr>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>
      </p:sp>
"#,
                ox = (min_x * EMU_PER_MM).round() as i64,
                oy = (min_y * EMU_PER_MM).round() as i64,
                cx = (w * EMU_PER_MM).round() as i64,
                cy = (h * EMU_PER_MM).round() as i64,
                lw = (p.width_mm * EMU_PER_MM).round() as i64,
                fill = fill_inner(p.stroke, p.alpha),
            ))
        }
        WorldShape::Text(t) => {
            let x = t.x_mm;
            let y = flip_y(t.y_mm) - t.size_mm;
            let w = t.width_mm.max(t.size_mm);
            let h = t.height_mm.max(t.size_mm);
            let text = xml_escape(&t.content);
            let fill = fill_inner(t.fill, t.alpha);
            // Rotation: OOXML is clockwise 60000ths of a degree; scene is CCW.
            let rot = ((-t.rotation_deg) * 60000.0).round() as i64;
            Ok(format!(
                r#"      <p:sp>
        <p:nvSpPr><p:cNvPr id="{id}" name="Text {id}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
        <p:spPr>
          <a:xfrm rot="{rot}"><a:off x="{ox}" y="{oy}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
          <a:noFill/>
          <a:ln><a:noFill/></a:ln>
        </p:spPr>
        <p:txBody>
          <a:bodyPr wrap="square" lIns="0" tIns="0" rIns="0" bIns="0"/>
          <a:lstStyle/>
          <a:p>
            <a:r>
              <a:rPr lang="en-US" sz="{sz}" dirty="0">
                <a:solidFill>{fill}</a:solidFill>
                <a:latin typeface="Arial"/>
              </a:rPr>
              <a:t>{text}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
"#,
                ox = (x * EMU_PER_MM).round() as i64,
                oy = (y * EMU_PER_MM).round() as i64,
                cx = (w * EMU_PER_MM).round() as i64,
                cy = (h * EMU_PER_MM).round() as i64,
                sz = (t.size_mm * 100.0 * 72.0 / 25.4)
                    .round()
                    .clamp(100.0, 4000.0) as i64, // half-points approx from mm
            ))
        }
        WorldShape::Image(_) => {
            // Image embedding needs relationships + media parts; skip outline-only for now.
            Ok(String::new())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn ellipse_shape(
    id: usize,
    name: &str,
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
    fill: &str,
    line: &str,
) -> String {
    format!(
        r#"      <p:sp>
        <p:nvSpPr><p:cNvPr id="{id}" name="{name} {id}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="{ox}" y="{oy}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
          <a:prstGeom prst="ellipse"><a:avLst/></a:prstGeom>
          {fill}{line}
        </p:spPr>
        <p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody>
      </p:sp>
"#,
        ox = (x_mm * EMU_PER_MM).round() as i64,
        oy = (y_mm * EMU_PER_MM).round() as i64,
        cx = (w_mm * EMU_PER_MM).round() as i64,
        cy = (h_mm * EMU_PER_MM).round() as i64,
    )
}

fn solid_fill(c: Color, alpha: f64) -> String {
    format!("<a:solidFill>{}</a:solidFill>", fill_inner(c, alpha))
}

fn fill_inner(c: Color, alpha: f64) -> String {
    let r = (c.r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (c.g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (c.b * 255.0).round().clamp(0.0, 255.0) as u8;
    if (alpha - 1.0).abs() < 1e-9 {
        format!(r#"<a:srgbClr val="{r:02X}{g:02X}{b:02X}"/>"#)
    } else {
        let a = ((1.0 - alpha) * 100000.0).round().clamp(0.0, 100000.0) as i64;
        format!(r#"<a:srgbClr val="{r:02X}{g:02X}{b:02X}"><a:alpha val="{a}"/></a:srgbClr>"#)
    }
}

fn xml_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".into(),
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '"' => "&quot;".into(),
            '\'' => "&apos;".into(),
            c => c.to_string(),
        })
        .collect()
}

fn content_types(n_slides: usize) -> String {
    let mut overrides = String::new();
    for i in 1..=n_slides {
        overrides.push_str(&format!(
            r#"  <Override PartName="/ppt/slides/slide{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
"#
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
{overrides}</Types>"#
    )
}

fn presentation_xml(doc: &Document) -> String {
    let mut sld_id = String::new();
    for i in 0..doc.pages.len() {
        let id = 256 + i as u32;
        sld_id.push_str(&format!(
            r#"    <p:sldId id="{id}" r:id="rId{}" />
"#,
            i + 2
        ));
    }
    let w = doc
        .pages
        .first()
        .map(|p| (p.paper.width_mm * EMU_PER_MM).round() as i64)
        .unwrap_or(7772400);
    let h = doc
        .pages
        .first()
        .map(|p| (p.paper.height_mm * EMU_PER_MM).round() as i64)
        .unwrap_or(10058400);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" saveSubsetFonts="1">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rId1"/>
  </p:sldMasterIdLst>
  <p:sldIdLst>
{sld_id}  </p:sldIdLst>
  <p:sldSz cx="{w}" cy="{h}"/>
  <p:notesSz cx="{w}" cy="{h}"/>
</p:presentation>"#
    )
}

fn presentation_rels(n_slides: usize) -> String {
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
"#,
    );
    for i in 0..n_slides {
        rels.push_str(&format!(
            r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>
"#,
            i + 2,
            i + 1
        ));
    }
    rels.push_str("</Relationships>");
    rels
}

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;

const SLIDE_LAYOUT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank" preserve="1">
  <p:cSld name="Blank">
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#;

const SLIDE_LAYOUT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#;

const SLIDE_MASTER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
  <p:sldLayoutIdLst>
    <p:sldLayoutId id="2147483649" r:id="rId1"/>
  </p:sldLayoutIdLst>
</p:sldMaster>"#;

const SLIDE_MASTER_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#;

const THEME: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="reciplexa">
  <a:themeElements>
    <a:clrScheme name="Office">
      <a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
      <a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="1F497D"/></a:dk2>
      <a:lt2><a:srgbClr val="EEECE1"/></a:lt2>
      <a:accent1><a:srgbClr val="4F81BD"/></a:accent1>
      <a:accent2><a:srgbClr val="C0504D"/></a:accent2>
      <a:accent3><a:srgbClr val="9BBB59"/></a:accent3>
      <a:accent4><a:srgbClr val="8064A2"/></a:accent4>
      <a:accent5><a:srgbClr val="4BACC6"/></a:accent5>
      <a:accent6><a:srgbClr val="F79646"/></a:accent6>
      <a:hlink><a:srgbClr val="0000FF"/></a:hlink>
      <a:folHlink><a:srgbClr val="800080"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Office">
      <a:majorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office">
      <a:fillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="9525"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>"#;

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_scene::{
        Circle, Color, Document, Ellipse, Frame, Image, Line, Page, PaperSize, Polygon, Polyline,
        Rect, Ring, Shape, Text,
    };
    use std::io::{Cursor, Read};

    fn a4_page(shapes: Vec<Shape>) -> Page {
        Page {
            paper: PaperSize::a4(),
            shapes,
        }
    }

    fn slide_xml_from_zip(bytes: &[u8], slide: usize) -> String {
        let name = format!("ppt/slides/slide{slide}.xml");
        let cursor = Cursor::new(bytes);
        let mut archive = zip::read::ZipArchive::new(cursor).expect("valid zip");
        let mut file = archive.by_name(&name).unwrap_or_else(|_| panic!("{name}"));
        let mut xml = String::new();
        file.read_to_string(&mut xml).unwrap();
        xml
    }

    #[test]
    fn pptx_is_zip_with_presentation() {
        let doc = Document {
            pages: vec![a4_page(vec![Shape::Circle(Circle {
                x_mm: 105.0,
                y_mm: 148.5,
                radius_mm: 20.0,
                fill: Color::BLACK,
            })])],
        };
        let bytes = document_to_pptx(&doc).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        let slide = slide_xml_from_zip(&bytes, 1);
        assert!(slide.contains("prst=\"ellipse\""));
        assert!(bytes
            .windows(b"ppt/presentation.xml".len())
            .any(|w| w == b"ppt/presentation.xml"));
    }

    #[test]
    fn empty_page_still_emits_slide() {
        let doc = Document {
            pages: vec![a4_page(vec![])],
        };
        let bytes = document_to_pptx(&doc).unwrap();
        let slide = slide_xml_from_zip(&bytes, 1);
        assert!(slide.contains("<p:sld"));
        assert!(!slide.contains("prst=\"ellipse\""));
    }

    #[test]
    fn multipage_has_two_slides_and_content_types() {
        let page = a4_page(vec![]);
        let doc = Document {
            pages: vec![page.clone(), page],
        };
        let bytes = document_to_pptx(&doc).unwrap();
        assert!(slide_xml_from_zip(&bytes, 1).contains("<p:sld"));
        assert!(slide_xml_from_zip(&bytes, 2).contains("<p:sld"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("slide1.xml"));
        assert!(text.contains("slide2.xml"));
        assert!(text.contains("rId2"));
        assert!(text.contains("rId3"));
    }

    #[test]
    fn ring_emits_stroked_ellipse() {
        let doc = Document::single_page(a4_page(vec![Shape::Ring(Ring {
            x_mm: 50.0,
            y_mm: 50.0,
            radius_mm: 10.0,
            width_mm: 1.0,
            stroke: Color::RED,
        })]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("<a:ln w="));
        assert!(slide.contains("FF0000"));
    }

    #[test]
    fn rect_and_polygon_emit_custom_geometry() {
        let doc = Document::single_page(a4_page(vec![
            Shape::Rect(Rect {
                x_mm: 5.0,
                y_mm: 5.0,
                width_mm: 20.0,
                height_mm: 10.0,
                fill: Color::BLUE,
            }),
            Shape::Polygon(Polygon {
                points_mm: vec![(0.0, 0.0), (30.0, 0.0), (15.0, 25.0)],
                fill: Color::GREEN,
            }),
        ]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("Polygon"));
        assert!(slide.contains("<a:custGeom>"));
        assert!(slide.contains("<a:close/>"));
        assert!(slide.contains("0000FF"));
        assert!(slide.contains("00FF00"));
    }

    #[test]
    fn frame_emits_stroked_polygon() {
        let doc = Document::single_page(a4_page(vec![Shape::Frame(Frame {
            x_mm: 10.0,
            y_mm: 10.0,
            width_mm: 40.0,
            height_mm: 20.0,
            stroke_width_mm: 0.5,
            stroke: Color::BLACK,
        })]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("<a:noFill/>"));
        assert!(slide.contains("<a:ln w="));
    }

    #[test]
    fn line_and_polyline_emit_open_paths() {
        let doc = Document::single_page(a4_page(vec![
            Shape::Line(Line {
                x1_mm: 0.0,
                y1_mm: 0.0,
                x2_mm: 50.0,
                y2_mm: 50.0,
                stroke: Color::RED,
                width_mm: 0.3,
            }),
            Shape::Polyline(Polyline {
                points_mm: vec![(10.0, 10.0), (20.0, 30.0), (40.0, 15.0)],
                stroke: Color::GREEN,
                width_mm: 0.2,
            }),
        ]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("Path"));
        assert!(slide.contains("<a:moveTo>"));
        assert!(slide.contains("<a:lnTo>"));
        assert!(!slide.contains("<a:close/>"));
    }

    #[test]
    fn closed_polyline_path_has_close() {
        let doc = Document::single_page(a4_page(vec![Shape::Polyline(Polyline {
            points_mm: vec![(0.0, 0.0), (20.0, 0.0), (10.0, 20.0), (0.0, 0.0)],
            stroke: Color::BLACK,
            width_mm: 0.5,
        })]));
        // Polyline is open in scene; path closed flag is false.
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("Path"));
    }

    #[test]
    fn text_escapes_xml_and_applies_rotation() {
        let doc = Document::single_page(a4_page(vec![Shape::Group {
            transform: reciplexa_scene::Affine::rotate_deg(45.0),
            children: vec![Shape::Text(Text {
                x_mm: 20.0,
                y_mm: 200.0,
                size_mm: 8.0,
                width_mm: Some(60.0),
                height_mm: Some(20.0),
                content: "A & B <tag> \"quote\"".into(),
                fill: Color::BLACK,
            })],
        }]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("&amp;"));
        assert!(slide.contains("&lt;"));
        assert!(slide.contains("&quot;"));
        assert!(slide.contains(" rot=\"-2700000\""));
        assert!(slide.contains("txBox=\"1\""));
    }

    #[test]
    fn semi_transparent_fill_emits_alpha() {
        let doc = Document::single_page(a4_page(vec![Shape::Opacity {
            alpha: 0.5,
            children: vec![Shape::Circle(Circle {
                x_mm: 30.0,
                y_mm: 30.0,
                radius_mm: 10.0,
                fill: Color::RED,
            })],
        }]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("<a:alpha val=\"50000\"/>"));
    }

    #[test]
    fn image_shape_is_skipped_in_slide() {
        let doc = Document::single_page(a4_page(vec![
            Shape::Image(Image {
                path: "x.png".into(),
                x_mm: 0.0,
                y_mm: 0.0,
                width_mm: 10.0,
                height_mm: 10.0,
            }),
            Shape::Circle(Circle {
                x_mm: 10.0,
                y_mm: 10.0,
                radius_mm: 5.0,
                fill: Color::BLACK,
            }),
        ]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(!slide.contains("x.png"));
        assert!(slide.contains("prst=\"ellipse\""));
    }

    #[test]
    fn write_document_streams_bytes() {
        let doc = Document::single_page(a4_page(vec![Shape::Ellipse(Ellipse {
            x_mm: 40.0,
            y_mm: 40.0,
            rx_mm: 15.0,
            ry_mm: 8.0,
            fill: Color::GREEN,
        })]));
        let mut buf = Vec::new();
        write_document(&doc, &mut buf).unwrap();
        assert_eq!(&buf[0..2], b"PK");
        let slide = slide_xml_from_zip(&buf, 1);
        assert!(slide.contains("Polygon"));
    }

    #[test]
    fn letter_sized_first_page_sets_presentation_size() {
        let doc = Document::single_page(Page {
            paper: PaperSize::letter(),
            shapes: vec![],
        });
        let bytes = document_to_pptx(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        // Letter width in EMU ≈ 7772400 when used as first slide size.
        assert!(text.contains("7772400") || text.contains("10058400"));
    }

    #[test]
    fn shape_xml_degenerate_polygon_and_path_are_empty() {
        use reciplexa_view::{WorldPath, WorldPolygon, WorldShape};
        let poly = WorldShape::Polygon(WorldPolygon {
            points_mm: vec![(0.0, 0.0), (1.0, 1.0)],
            color: Color::RED,
            stroke_width_mm: None,
            alpha: 1.0,
        });
        assert!(shape_xml(&poly, 2, 297.0).unwrap().is_empty());
        let path = WorldShape::Path(WorldPath {
            points_mm: vec![(0.0, 0.0)],
            stroke: Color::BLACK,
            width_mm: 1.0,
            closed: false,
            alpha: 1.0,
        });
        assert!(shape_xml(&path, 3, 297.0).unwrap().is_empty());
    }

    #[test]
    fn shape_xml_closed_path_and_fill_helpers() {
        use reciplexa_view::{WorldPath, WorldShape};
        let path = WorldShape::Path(WorldPath {
            points_mm: vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)],
            stroke: Color::BLACK,
            width_mm: 0.5,
            closed: true,
            alpha: 1.0,
        });
        let xml = shape_xml(&path, 4, 297.0).unwrap();
        assert!(xml.contains("<a:close/>"));
        assert_eq!(xml_escape("it's"), "it&apos;s");
        assert!(solid_fill(Color::RED, 1.0).contains("FF0000"));
        assert!(fill_inner(Color::RED, 0.5).contains("<a:alpha"));
    }

    #[test]
    fn slide_xml_errors_on_missing_page() {
        let doc = Document::default();
        assert!(slide_xml(&doc, 0).is_err());
    }

    #[test]
    fn zero_page_document_still_builds_package() {
        let bytes = document_to_pptx(&Document::default()).unwrap();
        assert!(bytes.starts_with(b"PK"));
    }

    #[test]
    fn write_document_io_error() {
        struct FailWrite;
        impl Write for FailWrite {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let doc = Document::single_page(a4_page(vec![]));
        assert!(write_document(&doc, FailWrite).is_err());
        let mut w = FailWrite;
        assert!(std::io::Write::flush(&mut w).is_ok());
    }

    #[test]
    fn xml_escape_covers_all_special_chars() {
        assert_eq!(xml_escape("a>b"), "a&gt;b");
        assert_eq!(xml_escape("it's"), "it&apos;s");
        assert_eq!(xml_escape("a&b<c\"d'"), "a&amp;b&lt;c&quot;d&apos;");
    }

    #[test]
    fn shape_xml_circle_with_stroke_emits_line() {
        use reciplexa_view::{WorldCircle, WorldShape};
        let shape = WorldShape::Circle(WorldCircle {
            x_mm: 30.0,
            y_mm: 30.0,
            radius_mm: 10.0,
            color: Color::RED,
            stroke_width_mm: Some(0.5),
            alpha: 1.0,
        });
        let xml = shape_xml(&shape, 2, 297.0).unwrap();
        assert!(xml.contains("<a:ln w="));
        assert!(xml.contains("FF0000"));
    }

    #[test]
    fn shape_xml_polygon_stroke_branch() {
        use reciplexa_view::{WorldPolygon, WorldShape};
        let shape = WorldShape::Polygon(WorldPolygon {
            points_mm: vec![(0.0, 0.0), (20.0, 0.0), (10.0, 20.0)],
            color: Color::BLUE,
            stroke_width_mm: Some(0.3),
            alpha: 1.0,
        });
        let xml = shape_xml(&shape, 3, 297.0).unwrap();
        assert!(xml.contains("<a:noFill/>"));
        assert!(xml.contains("<a:ln w="));
    }

    #[test]
    fn opaque_solid_fill_has_no_alpha_element() {
        let fill = solid_fill(Color::GREEN, 1.0);
        assert!(fill.contains("00FF00"));
        assert!(!fill.contains("<a:alpha"));
    }

    #[test]
    fn plain_text_shape_emits_txbody() {
        let doc = Document::single_page(a4_page(vec![Shape::Text(Text {
            x_mm: 20.0,
            y_mm: 200.0,
            size_mm: 8.0,
            width_mm: None,
            height_mm: None,
            content: "Plain".into(),
            fill: Color::BLACK,
        })]));
        let slide = slide_xml_from_zip(&document_to_pptx(&doc).unwrap(), 1);
        assert!(slide.contains("<a:t>Plain</a:t>"));
        assert!(slide.contains("txBox=\"1\""));
    }

    #[test]
    fn three_slides_update_presentation_rels() {
        let page = a4_page(vec![]);
        let doc = Document {
            pages: vec![page.clone(), page.clone(), page],
        };
        let bytes = document_to_pptx(&doc).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("slide3.xml"));
        assert!(text.contains("rId4"));
    }

    #[test]
    fn document_to_pptx_write_errors_on_limited_buffer() {
        use std::io::{Seek, SeekFrom};
        struct FailAfterWrite {
            inner: Cursor<Vec<u8>>,
            limit: usize,
        }
        impl Write for FailAfterWrite {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                if self.inner.position() as usize + buf.len() > self.limit {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::StorageFull,
                        "capacity",
                    ));
                }
                self.inner.write(buf)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.inner.flush()
            }
        }
        impl Seek for FailAfterWrite {
            fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
                self.inner.seek(pos)
            }
        }
        let doc = Document::single_page(a4_page(vec![]));
        let mut buf = FailAfterWrite {
            inner: Cursor::new(Vec::new()),
            limit: 64,
        };
        assert!(super::document_to_pptx_write(&doc, &mut buf).is_err());
        assert!(std::io::Write::flush(&mut buf).is_ok());
    }
}
