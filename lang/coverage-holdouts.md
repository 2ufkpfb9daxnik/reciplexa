# Coverage holdouts (core / bind / eval)

Target: ≥99% region coverage on Rust implementation (`tests/` / `main` / `bin` excluded). Floor for documenting residuals: all three ≥95%.

Measured with `cargo llvm-cov --workspace --json --offline` (Env D: `CARGO_TARGET_DIR=d:\reciplexa\target`). Prefer workspace table in `coverage-status.md` (scoped `-p` remasures can inflate bind via module.cfg(test)).

## Current (this pass)

| Crate | Regions | % | vs ≥95% |
|-------|--------:|--:|---------|
| `reciplexa-eval` | 2115/2198 | **96.22%** | met |
| `reciplexa-bind` | 2399/2524 | **95.05%** | met |
| `reciplexa-core` | 8742/9532 | **91.71%** | short (~694 regions to 99%) |

Filtered workspace overall **97.20%**.

## Justified / intentional residues

Prefer **eliminate** over documenting when reachable. Items below are either still under active residual matrices or true holdouts.

### Prefer eliminate (still chasing)

1. **`elaborate.rs` (~426)** — Round9 local/rec type-annotation matrices + `unreachable!` → Err landed; residual is mostly `?` Err-only region ends and rare syntax shapes (data param sections, positivity, type-form tails). Keep expanding matrices rather than pub-exposing parsers.
2. **`check.rs` (~229)** — Same `?` pattern on `infer_with_effects` arms (Handle/App/If/Match/Record*). Round7–9 CoreExpr matrices help; remaining are Err-only edges after successful happy paths.
3. **`eval.rs` Cont other (~83)** — Collapsed duplicate `Resumed`/`Forward` into `other` (region count down); round9 Cont bubble matrices hit compound forms. Leftover arms need Cont results *during* deep-resume re-perform / builtin edges.
4. **`resolve.rs` (~79)** — Soft `continue` on malformed atoms / structured-comment skips / rare pattern shapes. Soft skips are intentional; convertible soft paths should become assertable Err when safe.

### Holdouts (justified)

1. **`load_module_tree` OS/IO** — `read_dir` / `read_to_string` / non-UTF-8 stem / locked-file races. Unit tests cover path-not-found, empty dir, self-import, non-`.rpx` skip, nested entry imports; true OS failures stay holdouts (dead without injection seams).
2. **Export/binding invariant** — Soft `continue` **eliminated**: `binding_for_export` returns internal `ModuleError`; cfg(test) covers the miss. Do not reintroduce soft skips.
3. **`expr.rs` `variant_payload_irrefutable` region ends** — Helper is cfg(test)-exercised; llvm still reports a handful of region entries around the match / `#[cfg(test)]` boundary (instrumentation noise). Prefer leave rather than API churn.
4. **Cast/unify exotic fragments (~79 / ~55)** — Open-row / forall / gradual stubs and evidence algebra tails; many `Unknown` decide paths. Fill when TYP semantic subtyping deepens; until then partial.

## Non-goals this pass

- jlreq / math package depth (graphics static surface now includes text/image/transforms/opacity — Slice C graphics checkbox can proceed when strangler ready; still defer jlreq/math package depth per plan).
- Push / publish.
- Treating CLI / syntax / gui / view under-99 as blockers for graphics Slice C (those crates remain below 99% in the workspace table; separate waves).
