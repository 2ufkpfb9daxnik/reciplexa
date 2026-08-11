# Coverage holdouts (core / bind / eval)

Target: ≥99% region coverage on Rust implementation (`tests/` / `main` / `bin` excluded). Floor for documenting residuals: all three ≥95%.

Measured with `cargo llvm-cov -p reciplexa-core -p reciplexa-eval -p reciplexa-bind --json --offline` (Env D: `CARGO_TARGET_DIR=d:\reciplexa\target`).

## Current (this pass)

| Crate | Regions | % | vs ≥95% |
|-------|--------:|--:|---------|
| `reciplexa-eval` | 2128/2223 | **95.73%** | met |
| `reciplexa-core` | 8718/9540 | **91.38%** | short (~360 regions) |
| `reciplexa-bind` | 2395/2650 | **90.38%** | short (~122 regions) |

Scoped three-crate total ≈ **91.87%**. Full workspace regenerate recorded in `coverage-status.md`.

## Justified / intentional residues

Prefer **eliminate** over documenting when reachable. Items below are either still under active residual matrices or true holdouts.

### Prefer eliminate (still chasing)

1. **`elaborate.rs` (~437)** — Dense Ok/Err source matrices + cfg(test) private helpers (`normalize_intersect`, `register_top_binding_names`, ambient/seq) landed; residual is mostly `?` Err-only region ends and rare syntax shapes (data param sections, positivity, type-form tails). Keep expanding matrices rather than pub-exposing parsers.
2. **`check.rs` (~242)** — Same `?` pattern on `infer_with_effects` arms (Handle/App/If/Match/Record*). Round7–8 CoreExpr matrices help; remaining are Err-only edges after successful happy paths.
3. **`eval.rs` Cont `other` (~95)** — Deep Cont returns of `Resumed`/`Forward`/`Performed` inside compound Frames (App args, Record fields, Seq, Cast). Round4–8 inject oneshot Conts; leftover arms need Cont results *during* deep-resume re-perform, not only top-level eval.
4. **`resolve.rs` (~81)** — Soft `continue` on malformed atoms / structured-comment skips / rare pattern shapes. Soft skips are intentional; convertible soft paths should become assertable Err when safe.

### Holdouts (justified)

1. **`load_module_tree` OS/IO** — `read_dir` / `read_to_string` / non-UTF-8 stem / locked-file races. Unit tests cover path-not-found, empty dir, self-import, non-`.rpx` skip; true OS failures stay holdouts (dead without injection seams).
2. **Export/binding invariant** — Soft `continue` **eliminated**: `binding_for_export` returns internal `ModuleError`; cfg(test) covers the miss. Do not reintroduce soft skips.
3. **`expr.rs` `variant_payload_irrefutable` region ends** — Helper is cfg(test)-exercised; llvm still reports a handful of region entries around the match / `#[cfg(test)]` boundary (instrumentation noise). Prefer leave rather than API churn.
4. **Cast/unify exotic fragments (~79 / ~55)** — Open-row / forall / gradual stubs and evidence algebra tails; many `Unknown` decide paths. Fill when TYP semantic subtyping deepens; until then partial.

## Non-goals this pass

- jlreq / math package depth (blocked until graphics static surface is solid — see `package-plan.md` Slice C graphics checkbox).
- Push / publish.
- Treating CLI / syntax / gui / view under-99 as blockers for graphics Slice C (those crates remain below 99% in the last full workspace table; separate waves).
