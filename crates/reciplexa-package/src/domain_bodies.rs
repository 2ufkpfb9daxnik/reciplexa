//! Native bodies for std domain packages (synthesized RPX until eval binds constructors).

use crate::domain_native::{DomainNativeModule, DomainNativeRegistry};

/// Seed registry with std domain natives that are ready to replace `.rpx` bodies.
pub fn std_domain_natives() -> DomainNativeRegistry {
    let mut reg = DomainNativeRegistry::empty();
    reg.register(length_units_module());
    reg.register(color_srgb_module());
    reg.register(graphics_color_module());
    reg.register(graphics_page_module());
    reg.register(graphics_shapes_module());
    reg.register(math_atoms_module());
    reg.register(math_scripts_module());
    reg.register(math_frac_module());
    reg.register(math_sqrt_module());
    reg.register(math_delimiters_module());
    reg.register(math_matrix_module());
    reg.register(math_accents_module());
    reg.register(math_bigops_module());
    reg.register(math_cases_module());
    reg.register(math_align_module());
    reg.register(math_stack_module());
    reg.register(japanese_classes_module());
    reg.register(japanese_linebreak_module());
    reg.register(japanese_kihon_module());
    reg.register(japanese_markup_module());
    reg.register(document_page_module());
    reg
}

/// N1.1 — `length/units` constructors (parity with former `packages/length/src/units.rpx`).
pub fn length_units_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "length/units".into(),
        exports: vec![
            "mm".into(),
            "cm".into(),
            "pt".into(),
            "bp".into(),
            "inch".into(),
            "q".into(),
            "px".into(),
            "em".into(),
            "zero".into(),
            "to-mm".into(),
            "from-mm".into(),
            "add-mm".into(),
            "scale-length".into(),
        ],
        synthetic_source: length_units_source().into(),
    }
}

/// Rust-owned source for `length/units` (package API unchanged).
pub fn length_units_source() -> &'static str {
    r#"(// native: length/units — synthesized by reciplexa-package::domain_bodies)
(val mm (fn (value)
  (record (unit "mm") (value value))))
(val cm (fn (value)
  (record (unit "cm") (value value))))
(val pt (fn (value)
  (record (unit "pt") (value value))))
(val bp (fn (value)
  (record (unit "bp") (value value))))
(val inch (fn (value)
  (record (unit "inch") (value value))))
(val q (fn (value)
  (record (unit "q") (value value))))
(val px (fn (value)
  (record (unit "px") (value value))))
(val em (fn (value)
  (record (unit "em") (value value))))
(val zero (record (unit "mm") (value 0)))
(val to-mm (fn (len)
  (if (= (field len unit) "mm") (field len value)
    (if (= (field len unit) "cm") (* (field len value) 10)
      (if (= (field len unit) "pt") (* (field len value) 0.3527777778)
        (if (= (field len unit) "bp") (* (field len value) 0.3527777778)
          (if (= (field len unit) "inch") (* (field len value) 25.4)
            (if (= (field len unit) "q") (* (field len value) 0.25)
              (if (= (field len unit) "px") (* (field len value) 0.2645833333)
                (field len value))))))))))
(val from-mm (fn (value)
  (record (unit "mm") (value value))))
(val add-mm (fn (a b)
  (record (unit "mm") (value (+ (to-mm a) (to-mm b))))))
(val scale-length (fn (len factor)
  (record (unit (field len unit)) (value (* (field len value) factor)))))
"#
}

/// N1.4 — `color/srgb` (standalone color package).
pub fn color_srgb_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "color/srgb".into(),
        exports: vec![
            "srgb".into(),
            "rgb".into(),
            "rgba".into(),
            "from-byte".into(),
            "gray".into(),
            "black".into(),
            "white".into(),
            "red".into(),
            "green".into(),
            "blue".into(),
            "yellow".into(),
            "cyan".into(),
            "magenta".into(),
            "orange".into(),
            "gray50".into(),
            "transparent".into(),
            "with-alpha".into(),
        ],
        synthetic_source: color_srgb_source().into(),
    }
}

pub fn color_srgb_source() -> &'static str {
    r#"(// native: color/srgb — synthesized by reciplexa-package::domain_bodies)
(val srgb (fn (r g b)
  (record (tag "srgb") (r r) (g g) (b b))))
(val rgb (fn (r g b)
  (record (tag "rgb") (r r) (g g) (b b))))
(val rgba (fn (r g b a)
  (record (tag "rgba") (r r) (g g) (b b) (a a))))
(val from-byte (fn (r g b)
  (record (tag "rgb")
    (r (/ r 255)) (g (/ g 255)) (b (/ b 255)))))
(val gray (fn (level)
  (record (tag "rgb") (r level) (g level) (b level))))
(val black (record (tag "rgb") (r 0) (g 0) (b 0)))
(val white (record (tag "rgb") (r 1) (g 1) (b 1)))
(val red (record (tag "rgb") (r 1) (g 0) (b 0)))
(val green (record (tag "rgb") (r 0) (g 1) (b 0)))
(val blue (record (tag "rgb") (r 0) (g 0) (b 1)))
(val yellow (record (tag "rgb") (r 1) (g 1) (b 0)))
(val cyan (record (tag "rgb") (r 0) (g 1) (b 1)))
(val magenta (record (tag "rgb") (r 1) (g 0) (b 1)))
(val orange (record (tag "rgb") (r 1) (g 0.5) (b 0)))
(val gray50 (record (tag "rgb") (r 0.5) (g 0.5) (b 0.5)))
(val transparent (record (tag "rgba") (r 0) (g 0) (b 0) (a 0)))
(val with-alpha (fn (color a)
  (record (tag "rgba")
    (r (field color r)) (g (field color g)) (b (field color b)) (a a))))
"#
}

/// N1.4 — `graphics/color` (subset used by graphics demos; shared rgb tags).
pub fn graphics_color_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "graphics/color".into(),
        exports: vec![
            "rgb".into(),
            "rgba".into(),
            "black".into(),
            "white".into(),
            "red".into(),
            "green".into(),
            "blue".into(),
            "gray".into(),
            "transparent".into(),
        ],
        synthetic_source: graphics_color_source().into(),
    }
}

pub fn graphics_color_source() -> &'static str {
    r#"(// native: graphics/color — synthesized by reciplexa-package::domain_bodies)
(val rgb (fn (r g b)
  (record (tag "rgb") (r r) (g g) (b b))))
(val rgba (fn (r g b a)
  (record (tag "rgba") (r r) (g g) (b b) (a a))))
(val black (record (tag "rgb") (r 0) (g 0) (b 0)))
(val white (record (tag "rgb") (r 1) (g 1) (b 1)))
(val red (record (tag "rgb") (r 1) (g 0) (b 0)))
(val green (record (tag "rgb") (r 0) (g 1) (b 0)))
(val blue (record (tag "rgb") (r 0) (g 0) (b 1)))
(val gray (record (tag "rgb") (r 0.5) (g 0.5) (b 0.5)))
(val transparent (record (tag "rgba") (r 0) (g 0) (b 0) (a 0)))
"#
}

/// N2.2 — `graphics/page` paper sizes + page/pages constructors.
pub fn graphics_page_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "graphics/page".into(),
        exports: vec![
            "a4".into(),
            "letter".into(),
            "a5".into(),
            "a3".into(),
            "legal".into(),
            "square".into(),
            "page".into(),
            "pages".into(),
            "page-size".into(),
        ],
        synthetic_source: graphics_page_source().into(),
    }
}

pub fn graphics_page_source() -> &'static str {
    r#"(// native: graphics/page — synthesized by reciplexa-package::domain_bodies)
(val a4 (record (width 210) (height 297)))
(val letter (record (width 215.9) (height 279.4)))
(val a5 (record (width 148) (height 210)))
(val a3 (record (width 297) (height 420)))
(val legal (record (width 215.9) (height 355.6)))
(val square (record (width 210) (height 210)))
(val page (fn (size content)
  (record (tag "page") (size size) (content content))))
(val pages (fn (items)
  (record (tag "pages") (items items))))
(val page-size (fn (width height)
  (record (width width) (height height))))
"#
}

/// N2.3–N2.7 — `graphics/shapes` constructors (full parity with former shapes.rpx).
pub fn graphics_shapes_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "graphics/shapes".into(),
        exports: vec![
            "circle".into(),
            "rect".into(),
            "ellipse".into(),
            "line".into(),
            "path".into(),
            "polyline".into(),
            "polygon".into(),
            "ring".into(),
            "frame".into(),
            "group".into(),
            "text".into(),
            "text-box".into(),
            "image".into(),
            "translate".into(),
            "rotate".into(),
            "scale".into(),
            "opacity".into(),
            "fill".into(),
            "stroke".into(),
            "paint".into(),
        ],
        synthetic_source: graphics_shapes_source().into(),
    }
}

pub fn graphics_shapes_source() -> &'static str {
    r#"(// native: graphics/shapes — synthesized by reciplexa-package::domain_bodies)
(val circle (fn (x y r)
  (record (tag "circle") (x x) (y y) (r r))))
(val rect (fn (x y w h)
  (record (tag "rect") (x x) (y y) (w w) (h h))))
(val ellipse (fn (x y rx ry)
  (record (tag "ellipse") (x x) (y y) (rx rx) (ry ry))))
(val line (fn (x1 y1 x2 y2)
  (record (tag "line") (x1 x1) (y1 y1) (x2 x2) (y2 y2))))
(val path (fn (points closed)
  (record (tag "path") (points points) (closed closed))))
(val polyline (fn (points)
  (record (tag "polyline") (points points))))
(val polygon (fn (points)
  (record (tag "polygon") (points points))))
(val ring (fn (x y r width)
  (record (tag "ring") (x x) (y y) (r r) (width width))))
(val frame (fn (x y w h width color)
  (record (tag "frame") (x x) (y y) (w w) (h h) (width width) (color color))))
(val group (fn (children)
  (record (tag "group") (children children))))
(val text (fn (x y size content)
  (record (tag "text") (x x) (y y) (size size) (content content))))
(val text-box (fn (x y size w h content)
  (record (tag "text") (x x) (y y) (size size) (w w) (h h) (content content))))
(val image (fn (path x y w h)
  (record (tag "image") (path path) (x x) (y y) (w w) (h h))))
(val translate (fn (dx dy child)
  (record (tag "translate") (dx dx) (dy dy) (child child))))
(val rotate (fn (degrees child)
  (record (tag "rotate") (degrees degrees) (child child))))
(val scale (fn (sx sy child)
  (record (tag "scale") (sx sx) (sy sy) (child child))))
(val opacity (fn (alpha child)
  (record (tag "opacity") (alpha alpha) (child child))))
(val fill (fn (shape color)
  (record (tag "fill") (shape shape) (color color))))
(val stroke (fn (shape width color)
  (record (tag "stroke") (shape shape) (width width) (color color))))
(val paint (fn (shape fill-color stroke-width stroke-color)
  (record (tag "paint") (shape shape) (fill fill-color)
    (stroke-width stroke-width) (stroke-color stroke-color))))
(// Present in body but omitted from shapes.rpi (MOD §9.2 export filter).)
(val shapes-internal-tag (fn (name)
  (record (tag name))))
"#
}

/// N3.1 — `math/atoms` atom class + symbol/row constructors.
pub fn math_atoms_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/atoms".into(),
        exports: vec![
            "class-ord".into(),
            "class-op".into(),
            "class-bin".into(),
            "class-rel".into(),
            "class-open".into(),
            "class-close".into(),
            "class-punct".into(),
            "class-fence".into(),
            "symbol".into(),
            "ord".into(),
            "op".into(),
            "bin".into(),
            "rel".into(),
            "open".into(),
            "close".into(),
            "punct".into(),
            "fence".into(),
            "row".into(),
            "operatorname".into(),
            "mathrm".into(),
            "mathbf".into(),
            "textop".into(),
            "fraction".into(),
            "superscript".into(),
            "subscript".into(),
        ],
        synthetic_source: math_atoms_source().into(),
    }
}

pub fn math_atoms_source() -> &'static str {
    r#"(// native: math/atoms — synthesized by reciplexa-package::domain_bodies)
(val class-ord "ord")
(val class-op "op")
(val class-bin "bin")
(val class-rel "rel")
(val class-open "open")
(val class-close "close")
(val class-punct "punct")
(val class-fence "fence")
(val symbol (fn (glyph class)
  (record (tag "math-symbol") (glyph glyph) (class class))))
(val ord (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "ord"))))
(val op (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "op"))))
(val bin (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "bin"))))
(val rel (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "rel"))))
(val open (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "open"))))
(val close (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "close"))))
(val punct (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "punct"))))
(val fence (fn (glyph)
  (record (tag "math-symbol") (glyph glyph) (class "fence"))))
(val row (fn (children)
  (record (tag "math-row") (children children))))
(val operatorname (fn (name)
  (record (tag "math-operatorname") (name name) (class "op") (variant "operatorname"))))
(val mathrm (fn (body)
  (record (tag "math-text") (body body) (variant "roman") (class "ord"))))
(val mathbf (fn (body)
  (record (tag "math-text") (body body) (variant "bold") (class "ord"))))
(val textop (fn (glyph)
  (record (tag "math-textop") (glyph glyph) (class "op"))))
(val fraction (fn (numerator denominator)
  (record (tag "math-fraction")
    (numerator numerator) (denominator denominator))))
(val superscript (fn (base exp)
  (record (tag "math-scripts")
    (base base)
    (superscript exp)
    (subscript (record (tag "math-absent"))))))
(val subscript (fn (base exp)
  (record (tag "math-scripts")
    (base base)
    (superscript (record (tag "math-absent")))
    (subscript exp))))
"#
}

/// N3.2 — `math/scripts` superscript / subscript / under / over.
pub fn math_scripts_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/scripts".into(),
        exports: vec![
            "absent".into(),
            "superscript".into(),
            "subscript".into(),
            "scripts".into(),
            "super".into(),
            "sub".into(),
            "overline".into(),
            "underline".into(),
            "overbrace".into(),
            "underbrace".into(),
            "overset".into(),
            "underset".into(),
            "over".into(),
            "under".into(),
        ],
        synthetic_source: math_scripts_source().into(),
    }
}

pub fn math_scripts_source() -> &'static str {
    r#"(// native: math/scripts — synthesized by reciplexa-package::domain_bodies)
(val absent (record (tag "math-absent")))
(val superscript (fn (base exp)
  (record (tag "math-scripts")
    (base base)
    (superscript exp)
    (subscript (record (tag "math-absent"))))))
(val subscript (fn (base exp)
  (record (tag "math-scripts")
    (base base)
    (superscript (record (tag "math-absent")))
    (subscript exp))))
(val scripts (fn (base sup sub)
  (record (tag "math-scripts")
    (base base) (superscript sup) (subscript sub))))
(val super (fn (base exp)
  (record (tag "math-scripts")
    (base base)
    (superscript exp)
    (subscript (record (tag "math-absent"))))))
(val sub (fn (base exp)
  (record (tag "math-scripts")
    (base base)
    (superscript (record (tag "math-absent")))
    (subscript exp))))
(val overline (fn (body)
  (record (tag "math-over") (kind "overline") (body body))))
(val underline (fn (body)
  (record (tag "math-under") (kind "underline") (body body))))
(val overbrace (fn (body)
  (record (tag "math-over") (kind "overbrace") (body body))))
(val underbrace (fn (body)
  (record (tag "math-under") (kind "underbrace") (body body))))
(val overset (fn (over body)
  (record (tag "math-over") (kind "overset") (label over) (body body))))
(val underset (fn (under body)
  (record (tag "math-under") (kind "underset") (label under) (body body))))
(val over (fn (over body)
  (record (tag "math-over") (kind "over") (label over) (body body))))
(val under (fn (under body)
  (record (tag "math-under") (kind "under") (label under) (body body))))
"#
}

/// N3.2 — `math/frac` fraction constructors.
pub fn math_frac_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/frac".into(),
        exports: vec!["fraction".into(), "over".into()],
        synthetic_source: math_frac_source().into(),
    }
}

pub fn math_frac_source() -> &'static str {
    r#"(// native: math/frac — synthesized by reciplexa-package::domain_bodies)
(val fraction (fn (numerator denominator)
  (record (tag "math-fraction")
    (numerator numerator) (denominator denominator))))
(val over (fn (numerator denominator)
  (record (tag "math-fraction")
    (numerator numerator) (denominator denominator))))
"#
}

/// N3.2 — `math/sqrt` radical constructors.
pub fn math_sqrt_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/sqrt".into(),
        exports: vec![
            "absent-index".into(),
            "sqrt".into(),
            "radical".into(),
            "radical-indexed".into(),
            "root".into(),
        ],
        synthetic_source: math_sqrt_source().into(),
    }
}

pub fn math_sqrt_source() -> &'static str {
    r#"(// native: math/sqrt — synthesized by reciplexa-package::domain_bodies)
(val absent-index (record (tag "math-absent")))
(val sqrt (fn (radicand)
  (record (tag "math-radical")
    (index (record (tag "math-absent")))
    (radicand radicand))))
(val radical (fn (radicand)
  (record (tag "math-radical")
    (index (record (tag "math-absent")))
    (radicand radicand))))
(val radical-indexed (fn (index radicand)
  (record (tag "math-radical")
    (index index) (radicand radicand))))
(val root (fn (index radicand)
  (record (tag "math-radical")
    (index index) (radicand radicand))))
"#
}

/// N3.3 — `math/delimiters` delimiter / fence wrappers.
pub fn math_delimiters_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/delimiters".into(),
        exports: vec![
            "delimiter".into(),
            "paren".into(),
            "brackets".into(),
            "braces".into(),
            "angles".into(),
            "abs".into(),
            "floor".into(),
            "ceil".into(),
        ],
        synthetic_source: math_delimiters_source().into(),
    }
}

pub fn math_delimiters_source() -> &'static str {
    r#"(// native: math/delimiters — synthesized by reciplexa-package::domain_bodies)
(val delimiter (fn (left right body)
  (record (tag "math-delimiter")
    (left left) (right right) (body body))))
(val paren (fn (body)
  (record (tag "math-delimiter") (left "(") (right ")") (body body))))
(val brackets (fn (body)
  (record (tag "math-delimiter") (left "[") (right "]") (body body))))
(val braces (fn (body)
  (record (tag "math-delimiter") (left "{") (right "}") (body body))))
(val angles (fn (body)
  (record (tag "math-delimiter") (left "⟨") (right "⟩") (body body))))
(val abs (fn (body)
  (record (tag "math-delimiter") (left "|") (right "|") (body body))))
(val floor (fn (body)
  (record (tag "math-delimiter") (left "⌊") (right "⌋") (body body))))
(val ceil (fn (body)
  (record (tag "math-delimiter") (left "⌈") (right "⌉") (body body))))
"#
}

/// N3.4 — `math/matrix` matrix constructors.
pub fn math_matrix_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/matrix".into(),
        exports: vec![
            "matrix".into(),
            "bmatrix".into(),
            "pmatrix".into(),
            "vmatrix".into(),
            "matrix-row".into(),
            "smallmatrix".into(),
            "matrix-env".into(),
            "array-env".into(),
            "matrix-delim".into(),
            "bmatrix-env".into(),
            "pmatrix-env".into(),
        ],
        synthetic_source: math_matrix_source().into(),
    }
}

pub fn math_matrix_source() -> &'static str {
    r#"(// native: math/matrix — synthesized by reciplexa-package::domain_bodies)
(val matrix (fn (rows)
  (record (tag "math-matrix") (kind "matrix") (rows rows))))
(val bmatrix (fn (rows)
  (record (tag "math-matrix") (kind "bmatrix") (rows rows)
    (left "[") (right "]"))))
(val pmatrix (fn (rows)
  (record (tag "math-matrix") (kind "pmatrix") (rows rows)
    (left "(") (right ")"))))
(val vmatrix (fn (rows)
  (record (tag "math-matrix") (kind "vmatrix") (rows rows)
    (left "|") (right "|"))))
(val matrix-row (fn (cells)
  (record (tag "math-matrix-row") (cells cells))))
(val smallmatrix (fn (rows)
  (record (tag "math-matrix") (kind "smallmatrix") (rows rows) (script-style true))))
(val matrix-env (fn (rows)
  (record (tag "math-matrix-env") (kind "matrix") (rows rows) (delimiters "none"))))
(val array-env (fn (column-align rows)
  (record (tag "math-matrix-env") (kind "array") (rows rows) (column-align column-align))))
(val matrix-delim (fn (left right rows)
  (record (tag "math-matrix") (kind "delimited") (left left) (right right) (rows rows))))
(val bmatrix-env (fn (rows)
  (record (tag "math-matrix") (kind "delimited-bmatrix") (left "[") (right "]") (rows rows))))
(val pmatrix-env (fn (rows)
  (record (tag "math-matrix") (kind "delimited-pmatrix") (left "(") (right ")") (rows rows))))
"#
}

/// N3.4 — `math/accents` accent wrappers.
pub fn math_accents_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/accents".into(),
        exports: vec![
            "accent".into(),
            "hat".into(),
            "bar".into(),
            "vec".into(),
            "tilde".into(),
            "dot".into(),
            "ddot".into(),
            "overline".into(),
            "underline".into(),
            "widehat".into(),
            "widetilde".into(),
        ],
        synthetic_source: math_accents_source().into(),
    }
}

pub fn math_accents_source() -> &'static str {
    r#"(// native: math/accents — synthesized by reciplexa-package::domain_bodies)
(val accent (fn (kind base)
  (record (tag "math-accent") (kind kind) (base base))))
(val hat (fn (base)
  (record (tag "math-accent") (kind "hat") (base base))))
(val bar (fn (base)
  (record (tag "math-accent") (kind "bar") (base base))))
(val vec (fn (base)
  (record (tag "math-accent") (kind "vec") (base base))))
(val tilde (fn (base)
  (record (tag "math-accent") (kind "tilde") (base base))))
(val dot (fn (base)
  (record (tag "math-accent") (kind "dot") (base base))))
(val ddot (fn (base)
  (record (tag "math-accent") (kind "ddot") (base base))))
(val overline (fn (base)
  (record (tag "math-accent") (kind "overline") (base base))))
(val underline (fn (base)
  (record (tag "math-accent") (kind "underline") (base base))))
(val widehat (fn (base)
  (record (tag "math-accent") (kind "widehat") (base base))))
(val widetilde (fn (base)
  (record (tag "math-accent") (kind "widetilde") (base base))))
"#
}

/// N3.4 — `math/bigops` large operators with optional limits.
pub fn math_bigops_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/bigops".into(),
        exports: vec![
            "bigop".into(),
            "sum".into(),
            "prod".into(),
            "integral".into(),
            "oint".into(),
            "lim".into(),
            "limsup".into(),
            "liminf".into(),
            "max".into(),
            "min".into(),
            "withlimits".into(),
            "nolimits".into(),
            "sum-nolimits".into(),
            "integral-nolimits".into(),
            "bigop-scripts".into(),
            "sum-scripts".into(),
            "prod-scripts".into(),
            "integral-scripts".into(),
            "oint-scripts".into(),
        ],
        synthetic_source: math_bigops_source().into(),
    }
}

pub fn math_bigops_source() -> &'static str {
    r#"(// native: math/bigops — synthesized by reciplexa-package::domain_bodies)
(val bigop (fn (glyph lower upper body)
  (record (tag "math-bigop")
    (glyph glyph) (lower lower) (upper upper) (body body))))
(val sum (fn (lower upper body)
  (record (tag "math-bigop")
    (glyph "∑") (lower lower) (upper upper) (body body))))
(val prod (fn (lower upper body)
  (record (tag "math-bigop")
    (glyph "∏") (lower lower) (upper upper) (body body))))
(val integral (fn (lower upper body)
  (record (tag "math-bigop")
    (glyph "∫") (lower lower) (upper upper) (body body))))
(val oint (fn (lower upper body)
  (record (tag "math-bigop")
    (glyph "∮") (lower lower) (upper upper) (body body))))
(val lim (fn (lower body)
  (record (tag "math-bigop")
    (glyph "lim")
    (lower lower)
    (upper (record (tag "math-absent")))
    (body body))))
(val limsup (fn (lower body)
  (record (tag "math-bigop")
    (glyph "limsup")
    (lower lower)
    (upper (record (tag "math-absent")))
    (body body))))
(val liminf (fn (lower body)
  (record (tag "math-bigop")
    (glyph "liminf")
    (lower lower)
    (upper (record (tag "math-absent")))
    (body body))))
(val max (fn (lower body)
  (record (tag "math-bigop")
    (glyph "max")
    (lower lower)
    (upper (record (tag "math-absent")))
    (body body))))
(val min (fn (lower body)
  (record (tag "math-bigop")
    (glyph "min")
    (lower lower)
    (upper (record (tag "math-absent")))
    (body body))))
(val withlimits (fn (op lower upper body)
  (record (tag "math-bigop")
    (glyph op) (lower lower) (upper upper) (body body) (limits "display"))))
(val nolimits (fn (op body)
  (record (tag "math-bigop")
    (glyph op)
    (lower (record (tag "math-absent")))
    (upper (record (tag "math-absent")))
    (body body)
    (limits "script"))))
(val sum-nolimits (fn (body)
  (record (tag "math-bigop")
    (glyph "∑")
    (lower (record (tag "math-absent")))
    (upper (record (tag "math-absent")))
    (body body))))
(val integral-nolimits (fn (body)
  (record (tag "math-bigop")
    (glyph "∫")
    (lower (record (tag "math-absent")))
    (upper (record (tag "math-absent")))
    (body body))))
(val bigop-scripts (fn (glyph sup sub body)
  (record (tag "math-bigop-scripts")
    (glyph glyph) (superscript sup) (subscript sub) (body body))))
(val sum-scripts (fn (sup sub body)
  (record (tag "math-bigop-scripts")
    (glyph "∑") (superscript sup) (subscript sub) (body body))))
(val prod-scripts (fn (sup sub body)
  (record (tag "math-bigop-scripts")
    (glyph "∏") (superscript sup) (subscript sub) (body body))))
(val integral-scripts (fn (sup sub body)
  (record (tag "math-bigop-scripts")
    (glyph "∫") (superscript sup) (subscript sub) (body body))))
(val oint-scripts (fn (sup sub body)
  (record (tag "math-bigop-scripts")
    (glyph "∮") (superscript sup) (subscript sub) (body body))))
"#
}

/// N3.3 — `math/cases` piecewise / cases constructs.
pub fn math_cases_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/cases".into(),
        exports: vec![
            "case-arm".into(),
            "cases".into(),
            "cases-lr".into(),
            "cases-delim".into(),
            "left-cases".into(),
            "right-cases".into(),
            "piecewise".into(),
            "otherwise".into(),
        ],
        synthetic_source: math_cases_source().into(),
    }
}

pub fn math_cases_source() -> &'static str {
    r#"(// native: math/cases — synthesized by reciplexa-package::domain_bodies)
(val case-arm (fn (body guard)
  (record (tag "math-case-arm") (body body) (guard guard))))
(val cases (fn (arms)
  (record (tag "math-cases") (arms arms) (left "{") (right ""))))
(val cases-lr (fn (left right arms)
  (record (tag "math-cases") (arms arms) (left left) (right right))))
(val cases-delim (fn (left right arms)
  (record (tag "math-cases") (arms arms) (left left) (right right) (delimited true))))
(val left-cases (fn (arms)
  (record (tag "math-cases") (arms arms) (left "{") (right "") (delimited true))))
(val right-cases (fn (arms)
  (record (tag "math-cases") (arms arms) (left "") (right "}") (delimited true))))
(val piecewise (fn (arms)
  (record (tag "math-cases")
    (arms arms) (left "{") (right "") (kind "piecewise"))))
(val otherwise (fn (body)
  (record (tag "math-case-arm")
    (body body)
    (guard (record (tag "math-symbol") (glyph "otherwise") (class "ord"))))))
"#
}

/// N3.5 — `math/align` alignment rows.
pub fn math_align_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/align".into(),
        exports: vec![
            "align-row".into(),
            "aligned".into(),
            "align".into(),
            "align-left".into(),
            "align-center".into(),
            "align-right".into(),
            "align-at".into(),
            "align-eq".into(),
        ],
        synthetic_source: math_align_source().into(),
    }
}

pub fn math_align_source() -> &'static str {
    r#"(// native: math/align — synthesized by reciplexa-package::domain_bodies)
(val align-row (fn (cells)
  (record (tag "math-align-row") (cells cells))))
(val aligned (fn (rows)
  (record (tag "math-aligned") (rows rows))))
(val align (fn (position body)
  (record (tag "math-align") (position position) (body body))))
(val align-left (fn (body)
  (record (tag "math-align") (position "left") (body body))))
(val align-center (fn (body)
  (record (tag "math-align") (position "center") (body body))))
(val align-right (fn (body)
  (record (tag "math-align") (position "right") (body body))))
(val align-at (fn (marker body)
  (record (tag "math-align-at") (marker marker) (body body))))
(val align-eq (fn (rows)
  (record (tag "math-align-eq") (rows rows) (relation "="))))
"#
}

/// N3.5 — `math/stack` vertical stacks and relation symbols.
pub fn math_stack_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "math/stack".into(),
        exports: vec![
            "stack".into(),
            "stackrel".into(),
            "overset-rel".into(),
            "underset-rel".into(),
            "atop".into(),
            "substack".into(),
        ],
        synthetic_source: math_stack_source().into(),
    }
}

pub fn math_stack_source() -> &'static str {
    r#"(// native: math/stack — synthesized by reciplexa-package::domain_bodies)
(val stack (fn (children)
  (record (tag "math-stack") (children children))))
(val stackrel (fn (relation base)
  (record (tag "math-stackrel") (relation relation) (base base))))
(val overset-rel (fn (relation base)
  (record (tag "math-stackrel") (relation relation) (base base) (kind "overset"))))
(val underset-rel (fn (relation base)
  (record (tag "math-stackrel") (relation relation) (base base) (kind "underset"))))
(val atop (fn (top bottom)
  (record (tag "math-stack") (children (list top bottom)) (kind "atop"))))
(val substack (fn (rows)
  (record (tag "math-substack") (rows rows))))
"#
}

/// N4.1 — `japanese/classes` JLReq character class tables (cl-01..cl-30).
pub fn japanese_classes_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "japanese/classes".into(),
        exports: vec![
            "class".into(),
            "all-class-ids".into(),
            "all-classes".into(),
            "class-name".into(),
            "class-name-ja".into(),
            "cl-01".into(),
            "cl-02".into(),
            "cl-03".into(),
            "cl-04".into(),
            "cl-05".into(),
            "cl-06".into(),
            "cl-07".into(),
            "cl-08".into(),
            "cl-09".into(),
            "cl-10".into(),
            "cl-11".into(),
            "cl-12".into(),
            "cl-13".into(),
            "cl-14".into(),
            "cl-15".into(),
            "cl-16".into(),
            "cl-17".into(),
            "cl-18".into(),
            "cl-19".into(),
            "cl-20".into(),
            "cl-21".into(),
            "cl-22".into(),
            "cl-23".into(),
            "cl-24".into(),
            "cl-25".into(),
            "cl-26".into(),
            "cl-27".into(),
            "cl-28".into(),
            "cl-29".into(),
            "cl-30".into(),
            "advance-em".into(),
            "is-square-letter?".into(),
            "is-punctuation-class?".into(),
            "is-kana-class?".into(),
            "is-western-class?".into(),
        ],
        synthetic_source: japanese_classes_source().into(),
    }
}

pub fn japanese_classes_source() -> &'static str {
    r#"(// native: japanese/classes — synthesized by reciplexa-package::domain_bodies)
(val class (fn (id code name name-ja)
  (record (tag "jlreq-class") (id id) (code code) (name name) (name-ja name-ja))))
(val all-class-ids
  (list 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30))
(val all-classes all-class-ids)
(val class-name (fn (class-id)
  (match class-id
    (1 -> "opening-brackets")
    (2 -> "closing-brackets")
    (3 -> "hyphens")
    (4 -> "dividing-punctuation")
    (5 -> "middle-dots")
    (6 -> "full-stops")
    (7 -> "commas")
    (8 -> "inseparable")
    (9 -> "iteration-marks")
    (10 -> "prolonged-sound-mark")
    (11 -> "small-kana")
    (12 -> "prefixed-abbreviations")
    (13 -> "postfixed-abbreviations")
    (14 -> "spaces")
    (15 -> "hiragana")
    (16 -> "katakana")
    (17 -> "math-symbols")
    (18 -> "grouped-numerals")
    (19 -> "ideographic")
    (20 -> "numeric")
    (21 -> "unit-symbols")
    (22 -> "enclosed-alphanumerics")
    (23 -> "ornaments")
    (24 -> "simple-western")
    (25 -> "complex-western")
    (26 -> "warichu-close")
    (27 -> "western-characters")
    (28 -> "attached-western")
    (29 -> "warichu-open")
    (30 -> "tate-chu-yoko")
    (_ -> "other"))))
(val class-name-ja (fn (class-id)
  (match class-id
    (1 -> "始め括弧類")
    (2 -> "終わり括弧類")
    (3 -> "ハイフン類")
    (4 -> "区切り約物")
    (5 -> "中点類")
    (6 -> "句点類")
    (7 -> "読点類")
    (8 -> "分離禁止文字")
    (9 -> "繰返し記号")
    (10 -> "長音記号")
    (11 -> "小書きの仮名")
    (12 -> "前置省略記号")
    (13 -> "後置省略記号")
    (14 -> "和字間隔等")
    (15 -> "平仮名")
    (16 -> "片仮名")
    (17 -> "等号類")
    (18 -> "連数字")
    (19 -> "漢字等")
    (20 -> "数字")
    (21 -> "単位記号中の欧字等")
    (22 -> "囲み文字")
    (23 -> "装飾文字")
    (24 -> "単純な欧字")
    (25 -> "複雑な欧字")
    (26 -> "割注終わり括弧類")
    (27 -> "欧文用文字")
    (28 -> "添付欧文")
    (29 -> "割注始め括弧類")
    (30 -> "縦中横")
    (_ -> "その他"))))
(val cl-01
  (record (tag "jlreq-class") (id 1) (code "cl-01") (name "opening-brackets") (name-ja "始め括弧類")))
(val cl-02
  (record (tag "jlreq-class") (id 2) (code "cl-02") (name "closing-brackets") (name-ja "終わり括弧類")))
(val cl-03
  (record (tag "jlreq-class") (id 3) (code "cl-03") (name "hyphens") (name-ja "ハイフン類")))
(val cl-04
  (record (tag "jlreq-class") (id 4) (code "cl-04") (name "dividing-punctuation") (name-ja "区切り約物")))
(val cl-05
  (record (tag "jlreq-class") (id 5) (code "cl-05") (name "middle-dots") (name-ja "中点類")))
(val cl-06
  (record (tag "jlreq-class") (id 6) (code "cl-06") (name "full-stops") (name-ja "句点類")))
(val cl-07
  (record (tag "jlreq-class") (id 7) (code "cl-07") (name "commas") (name-ja "読点類")))
(val cl-08
  (record (tag "jlreq-class") (id 8) (code "cl-08") (name "inseparable") (name-ja "分離禁止文字")))
(val cl-09
  (record (tag "jlreq-class") (id 9) (code "cl-09") (name "iteration-marks") (name-ja "繰返し記号")))
(val cl-10
  (record (tag "jlreq-class") (id 10) (code "cl-10") (name "prolonged-sound-mark") (name-ja "長音記号")))
(val cl-11
  (record (tag "jlreq-class") (id 11) (code "cl-11") (name "small-kana") (name-ja "小書きの仮名")))
(val cl-12
  (record (tag "jlreq-class") (id 12) (code "cl-12") (name "prefixed-abbreviations") (name-ja "前置省略記号")))
(val cl-13
  (record (tag "jlreq-class") (id 13) (code "cl-13") (name "postfixed-abbreviations") (name-ja "後置省略記号")))
(val cl-14
  (record (tag "jlreq-class") (id 14) (code "cl-14") (name "spaces") (name-ja "和字間隔等")))
(val cl-15
  (record (tag "jlreq-class") (id 15) (code "cl-15") (name "hiragana") (name-ja "平仮名")))
(val cl-16
  (record (tag "jlreq-class") (id 16) (code "cl-16") (name "katakana") (name-ja "片仮名")))
(val cl-17
  (record (tag "jlreq-class") (id 17) (code "cl-17") (name "math-symbols") (name-ja "等号類")))
(val cl-18
  (record (tag "jlreq-class") (id 18) (code "cl-18") (name "grouped-numerals") (name-ja "連数字")))
(val cl-19
  (record (tag "jlreq-class") (id 19) (code "cl-19") (name "ideographic") (name-ja "漢字等")))
(val cl-20
  (record (tag "jlreq-class") (id 20) (code "cl-20") (name "numeric") (name-ja "数字")))
(val cl-21
  (record (tag "jlreq-class") (id 21) (code "cl-21") (name "unit-symbols") (name-ja "単位記号中の欧字等")))
(val cl-22
  (record (tag "jlreq-class") (id 22) (code "cl-22") (name "enclosed-alphanumerics") (name-ja "囲み文字")))
(val cl-23
  (record (tag "jlreq-class") (id 23) (code "cl-23") (name "ornaments") (name-ja "装飾文字")))
(val cl-24
  (record (tag "jlreq-class") (id 24) (code "cl-24") (name "simple-western") (name-ja "単純な欧字")))
(val cl-25
  (record (tag "jlreq-class") (id 25) (code "cl-25") (name "complex-western") (name-ja "複雑な欧字")))
(val cl-26
  (record (tag "jlreq-class") (id 26) (code "cl-26") (name "warichu-close") (name-ja "割注終わり括弧類")))
(val cl-27
  (record (tag "jlreq-class") (id 27) (code "cl-27") (name "western-characters") (name-ja "欧文用文字")))
(val cl-28
  (record (tag "jlreq-class") (id 28) (code "cl-28") (name "attached-western") (name-ja "添付欧文")))
(val cl-29
  (record (tag "jlreq-class") (id 29) (code "cl-29") (name "warichu-open") (name-ja "割注始め括弧類")))
(val cl-30
  (record (tag "jlreq-class") (id 30) (code "cl-30") (name "tate-chu-yoko") (name-ja "縦中横")))
(val advance-em (fn (class-id)
  (if (= class-id 1) 0.5
    (if (= class-id 2) 0.5
      (if (= class-id 5) 0.5
        (if (= class-id 6) 0.5
          (if (= class-id 7) 0.5
            (if (= class-id 14) 0.5
              (if (= class-id 26) 0.25 1)))))))))
(val is-square-letter? (fn (class-id)
  (if (= class-id 15) true
    (if (= class-id 16) true
      (if (= class-id 19) true
        (if (= class-id 20) true false))))))
(val is-punctuation-class? (fn (class-id)
  (if (= class-id 1) true
    (if (= class-id 2) true
      (if (= class-id 3) true
        (if (= class-id 4) true
          (if (= class-id 5) true
            (if (= class-id 6) true
              (if (= class-id 7) true false)))))))))
(val is-kana-class? (fn (class-id)
  (if (= class-id 11) true
    (if (= class-id 15) true
      (if (= class-id 16) true false)))))
(val is-western-class? (fn (class-id)
  (if (= class-id 21) true
    (if (= class-id 24) true
      (if (= class-id 25) true
        (if (= class-id 27) true
          (if (= class-id 28) true false)))))))
"#
}

/// N4.2 — `japanese/linebreak` kinsoku / break-opportunity stubs.
pub fn japanese_linebreak_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "japanese/linebreak".into(),
        exports: vec![
            "sample-line-head-prohibited".into(),
            "sample-line-end-prohibited".into(),
            "sample-inseparable".into(),
            "opportunity".into(),
            "line-head-prohibited-class?".into(),
            "line-end-prohibited-class?".into(),
            "inseparable-pair?".into(),
            "pair-rule".into(),
            "sample-pair-rules".into(),
            "break-between".into(),
            "hangable-class?".into(),
            "numeric-before-close-prohibited?".into(),
            "kinsoku-profile".into(),
            "classify-sample".into(),
        ],
        synthetic_source: japanese_linebreak_source().into(),
    }
}

pub fn japanese_linebreak_source() -> &'static str {
    // NOTE: Synthetic RPX keeps the package import API for Core eval.
    // Authoritative pair / classify tables for future layout live in
    // `reciplexa_std::japanese` (`classify_char`, `break_opportunity`).
    r#"(// native: japanese/linebreak — synthesized by reciplexa-package::domain_bodies)
(// authoritative Rust: reciplexa_std::japanese::{classify_char, break_opportunity})
(val sample-line-head-prohibited
  "」、。．，）〕］｝〉》』】！？ーぁぃぅぇぉっゃゅょァィゥェォッャュョヽヾゝゞ々")
(val sample-line-end-prohibited
  "「『（〔［｛〈《￥＄￡＃")
(val sample-inseparable
  "—…‥〳〴〵")
(val opportunity (fn (kind before after note)
  (record (tag "jlreq-break-opportunity")
    (kind kind) (before before) (after after) (note note))))
(val line-head-prohibited-class? (fn (class-id)
  (if (= class-id 2) true
    (if (= class-id 6) true
      (if (= class-id 7) true
        (if (= class-id 9) true
          (if (= class-id 10) true
            (if (= class-id 11) true
              (if (= class-id 29) true false)))))))))
(val line-end-prohibited-class? (fn (class-id)
  (if (= class-id 1) true
    (if (= class-id 12) true
      (if (= class-id 28) true false)))))
(val inseparable-pair? (fn (before after)
  (if (= before 8) true
    (if (= after 8) true
      (if (= before 24)
        (if (= after 24) true false)
        false)))))
(val pair-rule (fn (before after kind note)
  (record (tag "jlreq-pair-rule")
    (before before) (after after) (kind kind) (note note))))
(val sample-pair-rules
  (list
    (record (tag "jlreq-pair-rule") (before 1) (after 15) (kind "prohibited") (note "open + letter"))
    (record (tag "jlreq-pair-rule") (before 15) (after 2) (kind "prohibited") (note "letter + close"))
    (record (tag "jlreq-pair-rule") (before 15) (after 6) (kind "prohibited") (note "letter + full-stop"))
    (record (tag "jlreq-pair-rule") (before 15) (after 7) (kind "prohibited") (note "letter + comma"))
    (record (tag "jlreq-pair-rule") (before 8) (after 8) (kind "inseparable") (note "cl-08"))
    (record (tag "jlreq-pair-rule") (before 19) (after 19) (kind "allowed") (note "ideograph run"))
    (record (tag "jlreq-pair-rule") (before 27) (after 27) (kind "allowed") (note "western run"))
    (record (tag "jlreq-pair-rule") (before 15) (after 27) (kind "allowed") (note "JP/western boundary"))
    (record (tag "jlreq-pair-rule") (before 19) (after 20) (kind "allowed") (note "ideograph + digit"))
    (record (tag "jlreq-pair-rule") (before 20) (after 13) (kind "prohibited") (note "digit + postfix"))
    (record (tag "jlreq-pair-rule") (before 20) (after 1) (kind "prohibited") (note "digit + open"))
    (record (tag "jlreq-pair-rule") (before 20) (after 2) (kind "prohibited") (note "digit + close"))
    (record (tag "jlreq-pair-rule") (before 20) (after 21) (kind "prohibited") (note "digit + unit"))
    (record (tag "jlreq-pair-rule") (before 12) (after 20) (kind "prohibited") (note "prefix + digit"))
    (record (tag "jlreq-pair-rule") (before 24) (after 27) (kind "inseparable") (note "simple×western run"))
    (record (tag "jlreq-pair-rule") (before 8) (after 19) (kind "inseparable") (note "cl-08 × ideograph"))
    (record (tag "jlreq-pair-rule") (before 30) (after 30) (kind "allowed") (note "tate-chu-yoko corner"))
    (record (tag "jlreq-pair-rule") (before 14) (after 15) (kind "allowed") (note "space + hiragana"))
    (record (tag "jlreq-pair-rule") (before 16) (after 10) (kind "prohibited") (note "katakana + prolonged"))
    (record (tag "jlreq-pair-rule") (before 24) (after 24) (kind "inseparable") (note "simple western run"))
    (record (tag "jlreq-pair-rule") (before 6) (after 1) (kind "prohibited") (note "full-stop before open"))
    (record (tag "jlreq-pair-rule") (before 7) (after 1) (kind "prohibited") (note "comma before open"))
    (record (tag "jlreq-pair-rule") (before 2) (after 15) (kind "prohibited") (note "close before hiragana"))
    (record (tag "jlreq-pair-rule") (before 2) (after 19) (kind "prohibited") (note "close before ideograph"))
    (record (tag "jlreq-pair-rule") (before 19) (after 2) (kind "prohibited") (note "ideograph before close"))
    (record (tag "jlreq-pair-rule") (before 27) (after 27) (kind "allowed") (note "latin run"))
    (record (tag "jlreq-pair-rule") (before 30) (after 19) (kind "allowed") (note "tate-chu-yoko + ideograph"))))
(val hangable-class? (fn (class-id)
  (if (= class-id 6) true
    (if (= class-id 7) true false))))
(val numeric-before-close-prohibited? (fn (before after)
  (if (= before 20)
    (if (= after 2) true
      (if (= after 1) true false))
    false)))
(val break-between (fn (before after)
  (if (= before 8)
    (record (tag "jlreq-break-opportunity") (kind "inseparable") (before before) (after after) (note "cl-08"))
    (if (= after 8)
      (record (tag "jlreq-break-opportunity") (kind "inseparable") (before before) (after after) (note "cl-08"))
      (if (= before 1)
        (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "line-end kinsoku"))
        (if (= after 2)
          (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "line-head kinsoku"))
          (if (= after 6)
            (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "line-head kinsoku"))
            (if (= after 7)
              (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "line-head kinsoku"))
              (if (= after 9)
                (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "iteration mark head"))
                (if (= after 10)
                  (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "prolonged-sound head"))
                  (if (= after 11)
                    (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "small-kana head"))
                    (if (= before 12)
                      (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "prefixed abbrev end"))
                      (if (= before 20)
                        (if (= after 2)
                          (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "digit before close/open"))
                          (if (= after 1)
                            (record (tag "jlreq-break-opportunity") (kind "prohibited") (before before) (after after) (note "digit before close/open"))
                            (record (tag "jlreq-break-opportunity") (kind "allowed") (before before) (after after) (note "default allow"))))
                        (record (tag "jlreq-break-opportunity") (kind "allowed") (before before) (after after) (note "default allow")))))))))))))))
(val kinsoku-profile
  (record (tag "jlreq-kinsoku-profile")
    (line-head-prohibited-classes (list 2 6 7 9 10 11 29))
    (line-end-prohibited-classes (list 1 12 28))
    (hangable-classes (list 6 7))
    (sample-line-head-prohibited "」、。．，）〕］｝〉》』】！？ーぁぃぅぇぉっゃゅょァィゥェォッャュョヽヾゝゞ々")
    (sample-line-end-prohibited "「『（〔［｛〈《￥＄￡＃")
    (sample-inseparable "—…‥〳〴〵")
    (completeness "subset-stub")))
(// classify-sample kept for package API; prefer future intrinsic → reciplexa_std::japanese::classify_char)
(val classify-sample (fn (glyph)
  (if (= glyph "「") 1
    (if (= glyph "」") 2
      (if (= glyph "—") 3
        (if (= glyph "・") 5
          (if (= glyph "。") 6
            (if (= glyph "、") 7
              (if (= glyph "…") 8
                (if (= glyph "々") 9
                  (if (= glyph "ー") 10
                    (if (= glyph "ぁ") 11
                      (if (= glyph "〒") 12
                        (if (= glyph "あ") 15
                          (if (= glyph "ア") 16
                            (if (= glyph "＝") 17
                              (if (= glyph "A") 27
                                (if (= glyph "12") 30 19))))))))))))))))))
"#
}

/// N4.3 — `japanese/kihon` kihon-hanmen / line-rate / vertical stubs.
pub fn japanese_kihon_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "japanese/kihon".into(),
        exports: vec![
            "writing-mode-horizontal".into(),
            "writing-mode-vertical".into(),
            "line-rate-solid".into(),
            "line-rate-compact".into(),
            "line-rate-default".into(),
            "line-rate-relaxed".into(),
            "line-rate-loose".into(),
            "solid-setting".into(),
            "character-frame".into(),
            "line-metrics".into(),
            "kihon-hanmen".into(),
            "default-horizontal-kihon".into(),
            "default-vertical-kihon".into(),
            "heading-band-em".into(),
            "indent-em".into(),
            "trim-size".into(),
            "a5-trim".into(),
            "b5-jis-trim".into(),
            "a4-trim".into(),
            "margins".into(),
            "place-hanmen".into(),
            "column-count-one".into(),
            "column-count-two".into(),
            "multi-column".into(),
            "vertical-flow".into(),
            "tate-digits".into(),
            "vertical-stack".into(),
            "tate-chu-yoko-span".into(),
            "vertical-text".into(),
            "place-vertical-text".into(),
            "vertical-text-stack".into(),
        ],
        synthetic_source: japanese_kihon_source().into(),
    }
}

pub fn japanese_kihon_source() -> &'static str {
    r#"(// native: japanese/kihon — synthesized by reciplexa-package::domain_bodies)
(val writing-mode-horizontal "horizontal-tb")
(val writing-mode-vertical "vertical-rl")
(val line-rate-solid 1.0)
(val line-rate-compact 1.2)
(val line-rate-default 1.5)
(val line-rate-relaxed 1.7)
(val line-rate-loose 2.0)
(val solid-setting true)
(val character-frame (fn (size-em)
  (record (tag "jlreq-character-frame") (size-em size-em) (shape "square"))))
(val line-metrics (fn (char-size-em line-rate)
  (record (tag "jlreq-line-metrics")
    (char-size-em char-size-em)
    (line-rate line-rate)
    (line-pitch-em (* char-size-em line-rate))
    (line-gap-em (* char-size-em (- line-rate 1.0))))))
(val kihon-hanmen (fn (char-size-em line-length line-count line-rate mode)
  (record (tag "kihon-hanmen")
    (char-size-em char-size-em)
    (line-length line-length)
    (line-count line-count)
    (line-rate line-rate)
    (line-gap-em (* char-size-em (- line-rate 1.0)))
    (writing-mode mode)
    (solid-setting true)
    (hanmen-inline-em (* char-size-em line-length))
    (hanmen-block-em
      (+ (* char-size-em line-count)
         (* (* char-size-em (- line-rate 1.0)) (- line-count 1)))))))
(val default-horizontal-kihon
  (record (tag "kihon-hanmen")
    (char-size-em 1.0)
    (line-length 40)
    (line-count 30)
    (line-rate 1.5)
    (line-gap-em 0.5)
    (writing-mode "horizontal-tb")
    (solid-setting true)
    (hanmen-inline-em 40.0)
    (hanmen-block-em 44.5)))
(val default-vertical-kihon
  (record (tag "kihon-hanmen")
    (char-size-em 1.0)
    (line-length 35)
    (line-count 20)
    (line-rate 1.5)
    (line-gap-em 0.5)
    (writing-mode "vertical-rl")
    (solid-setting true)
    (hanmen-inline-em 35.0)
    (hanmen-block-em 29.5)))
(val heading-band-em (fn (kihon lines)
  (+ (* (field kihon char-size-em) lines)
     (* (field kihon line-gap-em) (- lines 1)))))
(val indent-em (fn (kihon chars)
  (* (field kihon char-size-em) chars)))
(val trim-size (fn (width-mm height-mm)
  (record (tag "jlreq-trim-size") (width-mm width-mm) (height-mm height-mm))))
(val a5-trim (record (tag "jlreq-trim-size") (width-mm 148) (height-mm 210)))
(val b5-jis-trim (record (tag "jlreq-trim-size") (width-mm 182) (height-mm 257)))
(val a4-trim (record (tag "jlreq-trim-size") (width-mm 210) (height-mm 297)))
(val margins (fn (top right bottom left)
  (record (tag "jlreq-margins")
    (top-mm top) (right-mm right) (bottom-mm bottom) (left-mm left))))
(val place-hanmen (fn (trim margins-rec hanmen)
  (record (tag "jlreq-hanmen-placement")
    (trim trim)
    (margins margins-rec)
    (hanmen hanmen)
    (note "geometry stub — lower does not yet consume kihon"))))
(val column-count-one 1)
(val column-count-two 2)
(val multi-column (fn (hanmen columns gutter-em)
  (record (tag "jlreq-multi-column")
    (hanmen hanmen)
    (columns columns)
    (gutter-em gutter-em)
    (completeness "stub"))))
(val vertical-flow (fn (body)
  (record (tag "ja-vertical-flow")
    (body body)
    (writing-mode "vertical-rl")
    (note "tategaki block stub"))))
(val tate-digits (fn (digits)
  (record (tag "ja-tate-digits") (digits digits) (writing-mode "vertical-rl"))))
(val vertical-stack (fn (items)
  (record (tag "ja-vertical-stack") (items items) (writing-mode "vertical-rl"))))
(val tate-chu-yoko-span (fn (body)
  (record (tag "ja-tate-chu-yoko") (body body) (kind "inline-span"))))
(val vertical-text (fn (x y size content)
  (record (tag "text")
    (x x) (y y) (size size) (content content)
    (writing-mode "vertical-rl")
    (placement "tategaki-stub"))))
(val place-vertical-text (fn (x y char-size-em content)
  (record (tag "ja-vertical-text-placement")
    (x x) (y y) (char-size-em char-size-em) (content content)
    (writing-mode "vertical-rl")
    (graphics-text
      (record (tag "text")
        (x x) (y y) (size char-size-em) (content content)
        (writing-mode "vertical-rl"))))))
(val vertical-text-stack (fn (origin-x origin-y char-size-em items)
  (record (tag "ja-vertical-text-stack")
    (origin-x origin-x) (origin-y origin-y)
    (char-size-em char-size-em) (items items)
    (writing-mode "vertical-rl")
    (note "stack of vertical-text records; lower via graphics bridge"))))
"#
}

/// N4.4 — `japanese/markup` document-oriented JP markup helpers.
pub fn japanese_markup_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "japanese/markup".into(),
        exports: vec![
            "heading".into(),
            "section".into(),
            "paragraph".into(),
            "note".into(),
            "emphasis".into(),
            "warn".into(),
            "todo".into(),
            "ruby".into(),
            "jukugo-ruby".into(),
            "tate-chu-yoko".into(),
            "tategaki-paragraph".into(),
            "tategaki-text".into(),
            "markup-bridge".into(),
            "doc-with-markup".into(),
            "ul".into(),
            "doc".into(),
        ],
        synthetic_source: japanese_markup_source().into(),
    }
}

pub fn japanese_markup_source() -> &'static str {
    r#"(// native: japanese/markup — synthesized by reciplexa-package::domain_bodies)
(val heading (fn (title)
  (record (tag "ja-heading") (title title))))
(val section (fn (title)
  (record (tag "ja-section") (title title))))
(val paragraph (fn (body)
  (record (tag "ja-paragraph") (body body) (indent-em 1))))
(val note (fn (body)
  (record (tag "ja-note") (body body))))
(val emphasis (fn (body)
  (record (tag "ja-emphasis") (body body))))
(val warn (fn (body)
  (record (tag "ja-warn") (body body))))
(val todo (fn (body)
  (record (tag "ja-todo") (body body))))
(val ruby (fn (base annotation)
  (record (tag "ja-ruby") (base base) (annotation annotation) (kind "simple"))))
(val jukugo-ruby (fn (base annotation)
  (record (tag "ja-ruby") (base base) (annotation annotation) (kind "jukugo"))))
(val tate-chu-yoko (fn (body)
  (record (tag "ja-tate-chu-yoko") (body body))))
(val tategaki-paragraph (fn (body)
  (record (tag "ja-tategaki-paragraph") (body body) (writing-mode "vertical-rl"))))
(val tategaki-text (fn (x y size content)
  (record (tag "text")
    (x x) (y y) (size size) (content content)
    (writing-mode "vertical-rl")
    (source "ja-markup-tategaki"))))
(val markup-bridge (fn (title markup-source)
  (record (tag "ja-markup-bridge")
    (title title)
    (markup-source markup-source)
    (note "package mirror of SYN @-markup examples/markup_ja.rpx"))))
(val doc-with-markup (fn (title markup-source children)
  (record (tag "ja-doc")
    (title title)
    (markup-source markup-source)
    (children children)
    (profile "jlreq-oriented-stub"))))
(val ul (fn (items)
  (record (tag "ja-ul") (items items))))
(val doc (fn (title children)
  (record (tag "ja-doc") (title title) (children children)
    (profile "jlreq-oriented-stub"))))
"#
}

/// N5.1 — `document/page` flow-oriented constructors (mirror `reciplexa_std::document`).
///
/// Tags use the `doc-*` prefix so they stay distinct from `graphics/page` scene tags.
/// Interim CST `(page)/(circle)` keyword tables are intentionally untouched.
pub fn document_page_module() -> DomainNativeModule {
    DomainNativeModule {
        module_path: "document/page".into(),
        exports: vec![
            "a4".into(),
            "letter".into(),
            "a5".into(),
            "a3".into(),
            "legal".into(),
            "page".into(),
            "flow".into(),
            "section".into(),
            "heading".into(),
            "paragraph".into(),
            "unordered-list".into(),
            "ordered-list".into(),
            "list-item".into(),
            "table".into(),
            "figure".into(),
            "spacer".into(),
            "block-heading".into(),
            "block-paragraph".into(),
            "block-list".into(),
            "block-table".into(),
            "block-figure".into(),
            "block-spacer".into(),
        ],
        synthetic_source: document_page_source().into(),
    }
}

pub fn document_page_source() -> &'static str {
    r#"(// native: document/page — synthesized by reciplexa-package::domain_bodies)
(val a4 (record (width 210) (height 297)))
(val letter (record (width 215.9) (height 279.4)))
(val a5 (record (width 148) (height 210)))
(val a3 (record (width 297) (height 420)))
(val legal (record (width 215.9) (height 355.6)))
(val page (fn (paper flow)
  (record (tag "doc-page") (paper paper) (flow flow))))
(val flow (fn (sections)
  (record (tag "doc-flow") (sections sections))))
(val section (fn (title blocks)
  (record (tag "doc-section") (title title) (blocks blocks))))
(val heading (fn (level text)
  (record (tag "doc-heading") (level level) (text text))))
(val paragraph (fn (text)
  (record (tag "doc-paragraph") (text text))))
(val unordered-list (fn (items)
  (record (tag "doc-list") (ordered false) (items items))))
(val ordered-list (fn (items)
  (record (tag "doc-list") (ordered true) (items items))))
(val list-item (fn (paragraphs)
  (record (tag "doc-list-item") (paragraphs paragraphs))))
(val table (fn (columns rows)
  (record (tag "doc-table") (columns columns) (rows rows))))
(val figure (fn (visual caption)
  (record (tag "doc-figure") (visual visual) (caption caption))))
(val spacer (fn (length)
  (record (tag "doc-spacer") (length length))))
(val block-heading (fn (heading)
  (record (tag "doc-block") (kind "heading") (heading heading))))
(val block-paragraph (fn (paragraph)
  (record (tag "doc-block") (kind "paragraph") (paragraph paragraph))))
(val block-list (fn (list)
  (record (tag "doc-block") (kind "list") (list list))))
(val block-table (fn (table)
  (record (tag "doc-block") (kind "table") (table table))))
(val block-figure (fn (figure)
  (record (tag "doc-block") (kind "figure") (figure figure))))
(val block-spacer (fn (spacer)
  (record (tag "doc-block") (kind "spacer") (spacer spacer))))
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_domain_natives_seed_length_color_page_shapes() {
        let reg = std_domain_natives();
        assert!(reg.contains("length/units"));
        assert!(reg.contains("color/srgb"));
        assert!(reg.contains("graphics/color"));
        assert!(reg.contains("graphics/page"));
        assert!(reg.contains("graphics/shapes"));
        assert!(reg.contains("math/atoms"));
        assert!(reg.contains("math/scripts"));
        assert!(reg.contains("math/frac"));
        assert!(reg.contains("math/sqrt"));
        assert!(reg.contains("math/delimiters"));
        assert!(reg.contains("math/matrix"));
        assert!(reg.contains("math/accents"));
        assert!(reg.contains("math/bigops"));
        assert!(reg.contains("math/cases"));
        assert!(reg.contains("math/align"));
        assert!(reg.contains("math/stack"));
        assert!(reg.contains("japanese/classes"));
        assert!(reg.contains("japanese/linebreak"));
        assert!(reg.contains("japanese/kihon"));
        assert!(reg.contains("japanese/markup"));
        assert!(reg.contains("document/page"));
        assert!(graphics_shapes_source().contains("(val circle "));
        assert!(graphics_shapes_source().contains("shapes-internal-tag"));
        assert!(math_atoms_source().contains("native: math/atoms"));
        assert!(math_stack_source().contains("native: math/stack"));
        assert!(japanese_classes_source().contains("native: japanese/classes"));
        assert!(japanese_markup_source().contains("native: japanese/markup"));
        assert!(document_page_source().contains("native: document/page"));
        assert!(document_page_source().contains("(tag \"doc-page\")"));
    }
}
