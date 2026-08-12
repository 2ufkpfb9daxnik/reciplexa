# Coverage holdouts (core / bind / eval)

Target: ≥99% region coverage on Rust implementation (`tests/` / `main` / `bin` excluded). Floor for documenting residuals: all three ≥95%.

Measured with `cargo llvm-cov --workspace --json --offline` (Env D: `CARGO_TARGET_DIR=d:\reciplexa\target`). Prefer workspace table in `coverage-status.md` (scoped `-p` remasures can inflate bind via module.cfg(test)).

**This wave core package remasure:** `cargo llvm-cov --package reciplexa-core --summary-only --offline`.

## Current (this pass)

| Crate | Regions | % | vs ≥95% |
|-------|--------:|--:|---------|
| `reciplexa-eval` | 2115/2198 | **96.22%** | met |
| `reciplexa-bind` | 2399/2524 | **95.05%** | met |
| `reciplexa-core` | 10297/10782 | **95.50%** | met (≥95%); short of ≥99% (~381 regions) |

Filtered workspace overall **97.20%** (last full table; core row superseded by package remasure above).

### Core file split (package)

| File | Cover | Missed |
|------|------:|-------:|
| `elaborate.rs` | 93.10% | ~274 |
| `check.rs` | 97.13% | ~123 |
| `unify.rs` | 94.70% | ~54 |
| `cast.rs` | 97.40% | ~33 |
| `expr.rs` | 99.08% | 1 |

## Justified / intentional residues

Prefer **eliminate** over documenting when reachable. Items below are either still under active residual matrices or true holdouts.

### Prefer eliminate (still chasing)

1. **`elaborate.rs` (~274)** — Round10–11 exhaustive Err matrices + binder/numeric probes landed; remaining are largely `?` Err-only region ends after syntax-kind guards where the lexer does not emit a failing `Number`/`String` spelling (or only via exotic CST shapes). Thin cfg(test) token walks help some; keep expanding before holdout promotion.
2. **`check.rs` (~123)** — Helper matrices hit variant keep/strip, occurrence, bind_pattern, letrec, numeric ambiguous/`Dynamic`. Leftover `?` / soft edges on Handle/With/Match happy-path region ends and cast insert tails.
3. **`unify.rs` (~54) / `cast.rs` (~33)** — Open-row / lacks / evidence / decidable-fragment tails; many partially exercised. Prefer more `occurs`/`plan_structural_check` leaves over new public API.
4. **`eval.rs` Cont other (~83)** — Leftover Cont results during deep-resume re-perform / builtin edges (workspace table).
5. **`resolve.rs` (~79)** — Soft `continue` on malformed atoms / structured-comment skips / rare pattern shapes.

### Holdouts (justified)

1. **`load_module_tree` OS/IO** — `read_dir` / `read_to_string` / non-UTF-8 stem / locked-file races. Unit tests cover path-not-found, empty dir, self-import, non-`.rpx` skip, nested entry imports; true OS failures stay holdouts (dead without injection seams).
2. **Export/binding invariant** — Soft `continue` **eliminated**: `binding_for_export` returns internal `ModuleError`; cfg(test) covers the miss. Do not reintroduce soft skips.
3. **`expr.rs` `variant_payload_irrefutable` region ends** — Helper is cfg(test)-exercised; llvm still reports a handful of region entries around the match / `#[cfg(test)]` boundary (instrumentation noise). Prefer leave rather than API churn.
4. **Cast/unify exotic fragments** — Open-row / forall / gradual stubs and evidence algebra tails; many `Unknown` decide paths. Fill when TYP semantic subtyping deepens; until then partial.
5. **Elaborate `?` after kind guard (candidate)** — When a token is already `SyntaxKind::Number`/`String`/`Ident` and the subsequent `parse_*`/`decode_*` Err arm is unreachable for any spelling the current lexer emits, document the arm rather than inventing green-tree fixtures. Do not promote wholesale until a failing Number/String corpus is exhausted.

## Non-goals this pass

- jlreq / math package depth (graphics static surface now includes text/image/transforms/opacity — Slice C graphics checkbox can proceed when strangler ready; still defer jlreq/math package depth per plan).
- Push / publish.
- Treating CLI / syntax / gui / view under-99 as blockers for graphics Slice C (those crates remain below 99% in the workspace table; separate waves).
