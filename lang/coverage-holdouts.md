# Coverage holdouts (core / bind / eval)

Target: ≥99% region coverage on Rust implementation (`tests/` / `main` / `bin` excluded). Floor for documenting residuals: all three ≥95%.

Measured with `cargo llvm-cov --workspace --json --offline` (Env D: `CARGO_TARGET_DIR=d:\reciplexa\target`). Prefer workspace table in `coverage-status.md` (scoped `-p` remasures can inflate bind via module.cfg(test)).

**This wave remasure:** filtered overall **98.06%** (51621/52642).

## Current (this pass)

| Crate | Regions | % | vs ≥95% |
|-------|--------:|--:|---------|
| `reciplexa-eval` | 3185/3343 | **95.27%** | met |
| `reciplexa-bind` | 2399/2524 | **95.05%** | met |
| `reciplexa-core` | 10760/11220 | **95.90%** | met (≥95%); short of ≥99% (~460 missed) |

### Core file split (workspace)

| File | Missed (approx) |
|------|----------------:|
| `elaborate.rs` | ~260 |
| `check.rs` | ~110 |
| `unify.rs` | ~54 |
| `cast.rs` | ~35 |
| `expr.rs` | 1 |

### Eval split

| File | Missed (approx) |
|------|----------------:|
| `graphics_value.rs` | ~190–220 (falling with bridge suites) |
| `eval.rs` Cont / deep-resume | ~84 |

## Justified / intentional residues

Prefer **eliminate** over documenting when reachable. Items below are either still under active residual matrices or true holdouts.

### Prefer eliminate (still chasing)

1. **`elaborate.rs` (~260)** — Round10–13 Err matrices landed; remaining are largely `?` Err-only region ends after syntax-kind guards.
2. **`check.rs` (~110)** — Round13 infer/coerce/insert cfg(test) matrix; leftover `?` region ends on effect/cast tails.
3. **`unify.rs` / `cast.rs`** — Open-row / lacks / Any-left residuals; remasure before promoting leftovers.
4. **`graphics_value.rs`** — Slice D strangler bridge; exhaustive tag suites landed this wave; continue paint/error/cons-list edge leaves until ≥95% package floor restored.
5. **`eval.rs` Cont other (~84)** — Leftover Cont results during deep-resume re-perform / builtin edges.
6. **`resolve.rs` (~79)** — Soft `continue` on malformed atoms / structured-comment skips / rare pattern shapes.

### Holdouts (justified)

1. **`load_module_tree` OS/IO** — `read_dir` / `read_to_string` / non-UTF-8 stem / locked-file races. Unit tests cover path-not-found, empty dir, self-import, non-`.rpx` skip, nested entry imports; true OS failures stay holdouts (dead without injection seams).
2. **Export/binding invariant** — Soft `continue` **eliminated**: `binding_for_export` returns internal `ModuleError`; cfg(test) covers the miss. Do not reintroduce soft skips.
3. **`expr.rs` `variant_payload_irrefutable` region ends** — Helper is cfg(test)-exercised; llvm still reports a handful of region entries around the match / `#[cfg(test)]` boundary (instrumentation noise). Prefer leave rather than API churn.
4. **Cast/unify exotic fragments** — Open-row / forall / gradual stubs and evidence algebra tails; many `Unknown` decide paths. Fill when TYP semantic subtyping deepens; until then partial.
5. **Elaborate `?` after kind guard (candidate)** — When a token is already `SyntaxKind::Number`/`String`/`Ident` and the subsequent `parse_*`/`decode_*` Err arm is unreachable for any spelling the current lexer emits, document the arm rather than inventing green-tree fixtures. Do not promote wholesale until a failing Number/String corpus is exhausted.
6. **Deep package-module Let nesting** — Elaborating many linked math modules can overflow default Windows debug stacks; package tests bump thread stack. Prefer eventual iterative nesting over documenting forever.
7. **`reciplexa-test` derive noise (7)** — `Debug`/`Clone`/`PartialEq` region entries with empty line attribution; forced format/clone exercises help little. Accept until harness API churn.

## Non-goals this pass

- Push / publish.
- Treating CLI / syntax / gui under-99 as blockers for graphics Slice D bridge growth (but eval package floor ≥95% is back on the chase list).
- Full JLReq UCS maps / math glyph layout.
