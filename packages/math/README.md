# packages/math

Editable math-tree package for Reciplexa (Core-evaluable records).

## Inspiration

Constructors follow the **SATySFi math model** (atom classes, scripts, fractions, radicals, delimiters, matrices, accents, big operators) and align naming with `crates/reciplexa-std/src/math.rs` where overlap exists:

| RPX | Notes |
|-----|-------|
| `ord`/`op`/`bin`/`rel`/`open`/`close`/`punct`/`fence` | `MathClass` strings |
| `symbol` / `row` | `Symbol` / `Row` |
| `fraction` | `Fraction` |
| `sqrt` / `radical-indexed` | `Radical` |
| `scripts` / `superscript` / `subscript` | `Scripts` |
| `delimiter` / `paren` / … | `Delimiter` |
| `matrix` / `bmatrix` / `pmatrix` / … | `MathAtom::Matrix` (+ `MathMatrixKind`) in std |
| `hat` / `bar` / `vec` / `check` / `breve` / … | `MathAtom::Accent` |
| `sum` / `prod` / `int` / `lim` | `MathAtom::BigOp` with limits |
| `align` / `aligned` | `MathAtom::Aligned` |
| `stack` / `atop` / `substack` | `MathAtom::Stack` |

This package builds **trees**, not laid-out glyphs. Std offers fontless `MathAtom::estimate_box()` / `estimate_box_with_style(EstimateStyle::{Text,Display})` heuristics (`SCRIPT_SCALE` vs tighter `SCRIPT_SCALE_TEXT` for scripts/limits) plus `phantom_box` / `smash_box` (vphantom-ish zero-width / smash zero height+depth on a `MathBox`), `accent_clearance_em` / `accent_clearance_em_named` (compact over / wide over / underline / underbar; kinds include `check`/`breve`/`acute`/`grave`/`ring`), `scripts_attachment_offsets`, `matrix_column_widths` / `aligned_column_x` / cases left-align, `cases_brace_total_height_em` / `CASES_ROW_HEIGHT_EM` (left brace stretch = `nrows × row_height` for `math-cases`), `bigop_limit_offsets`, `fraction_rule_metrics`, `radical_vinculum_index_offsets`, `stackrel_spacing_offsets` / `underbrace_spacing`, and TeX-ish `class_spacing_em` (thin/med/thick muskip stubs for Ord/Op/Bin/Rel/Punct/Fence pairs used by Row `estimate_box`) — scaffolding only. Host consume: `layout_math_atom_to_shapes` / eval `layout_math_to_shapes` places Scripts / BigOp / Matrix / Accent via those helpers, draws Fraction / Radical rules as scene `Line`s, and stretches Delimiter fence `Text` `size_mm` (see `lang/live-layout-plan.md` LL7–LL17). Language builtin `math-box` lowers a math tag record (or symbol string) via `math_value` → `estimate_box`; optional record field `style` (`"text"` / `"display"`) selects EstimateStyle (note: `"text"` string lit may arrive as a shape tag). Builtins `math-phantom` / `math-smash` wrap `phantom_box` / `smash_box` over the same inputs (or an already-estimated math-box metrics record). Optional `stretch-factor` on `math-delimiter` records and builtin `stretchy-delim` expose the stretchy fence heuristic. Line breaking, real OpenType MATH stretchy fences, real matrix alignment, and MATH tables remain future layout work (`lang/ja-math-deepen-plan.md`).

## Import

```text
(import math/atoms only ord bin rel row)
(import math/scripts only superscript scripts)
(import math/frac only fraction)
(import math/sqrt only sqrt radical-indexed)
(import math/delimiters only paren)
(import math/matrix only bmatrix matrix-row)
(import math/accents only hat vec)
(import math/bigops only sum int)
```

## Example

See `examples/pkg_math.rpx`, `examples/pkg_math_box.rpx` (`math-box` builtin), `examples/pkg_math_phantom.rpx` (`math-phantom` / `math-smash`), `examples/pkg_math_spacing.rpx` (Row class spacing), and `examples/pkg_live_math.rpx` (live-layout frac+scripts+delim).
