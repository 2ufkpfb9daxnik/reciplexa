# packages/math

Editable math-tree package for Reciplexa (Core-evaluable records).

## Inspiration

Constructors follow the **SATySFi math model** (atom classes, scripts, fractions, radicals, delimiters) and align naming with `crates/reciplexa-std/src/math.rs`:

| RPX | `MathClass` / `MathAtom` |
|-----|--------------------------|
| `ord`/`op`/`bin`/`rel`/`open`/`close`/`punct`/`fence` | `Ordinary`…`Fence` (`"ord"`…`"fence"`) |
| `symbol` / `row` | `Symbol` / `Row` |
| `fraction` | `Fraction` |
| `sqrt` / `radical-indexed` | `Radical` |
| `scripts` / `superscript` / `subscript` | `Scripts` |
| `delimiter` / `paren` / … | `Delimiter` |

This package builds **trees**, not laid-out glyphs. Line breaking, stretchy fences, and font math tables remain future layout work.

## Import

```text
(import math/atoms only ord bin rel row)
(import math/scripts only superscript scripts)
(import math/frac only fraction)
(import math/sqrt only sqrt radical-indexed)
(import math/delimiters only paren)
```

## Example

See `examples/pkg_math.rpx`.
