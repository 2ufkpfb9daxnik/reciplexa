# Coverage holdouts (core / bind / eval)

Target: ≥99% region coverage on Rust implementation (`tests/` / `main` / `bin` excluded). Floor for documenting residuals: all three ≥95%.

Measured with `cargo llvm-cov --workspace --json --offline` (Env D: `CARGO_TARGET_DIR=d:\reciplexa\target`). Prefer workspace table in `coverage-status.md` (scoped `-p` remasures can inflate bind via module.cfg(test)).

**This wave remasure:** filtered 7-crate file-summary **96.97%** (32719/33740). N6 q BEST-ENTRY on the then-smaller set was **99.01%**. `graphics_value` **99.11%**. Core crate **98.39%**.

## Current (this pass)

| Crate | Regions | % | vs ≥95% |
|-------|--------:|--:|---------|
| `reciplexa-eval` | 7009/7304 | **95.96%** | met; `graphics_value` ≥99%; `math_value` 89.97% still the floor |
| `reciplexa-bind` | 2473/2547 | **97.09%** | met |
| `reciplexa-core` | 13118/13332 | **98.39%** | met (≥95% and ≥98%); short of per-file ≥99% |

## Justified / intentional residues

Prefer **eliminate** over documenting when reachable. Items below are either still under active residual matrices or true holdouts.

### Prefer eliminate (optional soft continue)

1. **`elaborate.rs`** — **97.62%** file-summary; residual pattern/rec token / numeric-singleton arms.
2. **`check.rs`** — **98.57%**; open-record/match/`insert_casts` residuals (cfg(test) tips grow denom).
3. **`cast.rs`** — **98.98%**; exotic gradual / normalize_intersect dup skip instrumentation.
4. **`eval.rs`** — **97.84%**; Cont other residuals during deep-resume.
5. **`math_value.rs`** — **89.97%**; remaining `?` / cons / metric-record leaves after round-2 matrix.
6. **`module.rs` / `load.rs`** — **98.27%** / **97.91%**; exclusive-lock / entry-map_err.
7. **`document_pipeline.rs`** — file-summary diluted on Env D; prefer or-live when hashes exist.

### Holdouts (justified / permanent)

1. **`load_module_tree` OS/IO** — `read_dir` entry iterator Err / locked-file races beyond exclusive-lock coverage.
2. **Export/binding invariant** — call-site `?` arms in `elaborate_units` stay unreachable while export-set ⊆ binding-table.
3. **`expr.rs` `variant_payload_irrefutable` region ends** — cfg(test) boundary instrumentation noise.
4. **Cast/unify exotic fragments** — Open-row / forall / gradual stubs; fill when TYP deepens. (`unify.rs` ≥99%.)
5. **Elaborate near-dead** — numeric-singleton `_` (lexer never yields non-Int/F64 Number lit). Quarantined `?` / number-string pattern `None` **deleted** in N6q.
6. **Deep package-module Let nesting** — Windows debug stack; package tests bump thread stack.
7. **`parse.rs` `bump_as` None** — tipped; residual empty-current instrumentation may remain.
8. **`graphics_bridge` I/O Err** — **cleared** (live-hash **100%**). Do **not** copy into `module.rs` / `cli.rs`.
9. **`reciplexa-test` assert Display write! Err** — near-unreachable Formatter noise (`outcome` **100%**).
10. **`document_pipeline` apply Err / parse-after-resolve** — **deleted** in N6q (resolve already rejects parse errors; SetLayout on checked layout node cannot UnknownNode).
11. **`pipeline.rs` Ident-via-`children()`** — **deleted** in N6p; or-live **≥99%** restored in N6q (thr 98.2%).

## N5 document-surface holdouts (not coverage gaps)

1. **Interim keyword tables (`page`/`circle`/…)** — **S6b done:** production arms gated by `interim-surface` / `cfg(test)`; pipeline always refuses bare `(page …)`. Fixture: `crates/reciplexa-lower/tests/fixtures/interim_page.rpx`. Track in [`implemented-features.md`](implemented-features.md).
2. **Package-shaped GUI edits** — S0–S6b done (package nudge/size/multipage + golden + example migration + keyword-arm quarantine); markup soft-refuse.

## Non-goals this pass

- Push / publish / commit (unless requested).
- Full JLReq UCS maps / math glyph layout.
