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
| `matrix` / `bmatrix` / `pmatrix` / … | Matrix stub trees (not in std yet) |
| `hat` / `bar` / `vec` / … | Accent stub trees |
| `sum` / `prod` / `int` / `lim` | Big-operator stub trees with limits |

This package builds **trees**, not laid-out glyphs. Line breaking, stretchy fences, matrix alignment, and font math tables remain future layout work.

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
