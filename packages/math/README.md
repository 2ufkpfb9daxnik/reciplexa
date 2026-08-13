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
| `hat` / `bar` / `vec` / … | `MathAtom::Accent` |
| `sum` / `prod` / `int` / `lim` | `MathAtom::BigOp` with limits |
| `align` / `aligned` | `MathAtom::Aligned` |
| `stack` / `atop` / `substack` | `MathAtom::Stack` |

This package builds **trees**, not laid-out glyphs. Std offers fontless `MathAtom::estimate_box()` heuristics for scaffolding. Line breaking, stretchy fences, real matrix alignment, and OpenType MATH tables remain future layout work (`lang/ja-math-deepen-plan.md`).

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

See `examples/pkg_math.rpx`.
