# Coverage holdouts (core / bind / eval)

Target: ≥99% region coverage on Rust implementation (`tests/` / `main` / `bin` excluded). Floor for documenting residuals: all three ≥95%.

Measured with `cargo llvm-cov --workspace --json --offline` (Env D: `CARGO_TARGET_DIR=d:\reciplexa\target`). Prefer workspace table in `coverage-status.md` (scoped `-p` remasures can inflate bind via module.cfg(test)).

**This wave remasure:** filtered overall **99.01%** (7-crate BEST-ENTRY 25614/25870). **≥99% crossed** (N6 q).

## Current (this pass)

| Crate | Regions | % | vs ≥95% |
|-------|--------:|--:|---------|
| `reciplexa-eval` | (see file split) | ≥95% | met |
| `reciplexa-bind` | (see file split) | ≥95% | met |
| `reciplexa-core` | (see file split) | ≥95% | met; file residuals short of per-file ≥99% |

## Justified / intentional residues

Prefer **eliminate** over documenting when reachable. Items below are either still under active residual matrices or true holdouts.

### Prefer eliminate (optional soft continue)

1. **`elaborate.rs`** — live-hash **98.91%**; residual pattern/rec token / numeric-singleton arms.
2. **`check.rs`** — **98.54%**; open-record/match/`insert_casts` residuals (cfg(test) tips grow denom).
3. **`cast.rs`** — **98.98%**; exotic gradual / normalize_intersect dup skip instrumentation.
4. **`eval.rs`** — **98.51%**; file-level region noise after Cont tips (or-live line spans clean).
5. **`module.rs` / `load.rs`** — **98.27%** / **97.90%**; exclusive-lock / entry-map_err.
6. **`document_pipeline.rs`** — **97.99%**; residual instrumentation after dead-arm deletes.

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

1. **Interim keyword tables (`page`/`circle`/…)** — **retained on purpose** for writable GUI CST sync (`black_circle.rpx`). Package twin + markup package emit + read-only scene layers complete the dual path. Deleting tables is **GUI CST sync v2** ([`gui-cst-sync-v2-plan.md`](gui-cst-sync-v2-plan.md)), out of N5 delete scope.
2. **Package-shaped GUI edits** — S0 locator landed (`sync/package.rs`); canvas nudge still soft-refuses until S1 wires `collect_layers_package` / `nudge_layer_package`.

## Non-goals this pass

- Push / publish / commit (unless requested).
- Full JLReq UCS maps / math glyph layout.
