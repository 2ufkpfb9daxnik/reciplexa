# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline`.

## Overall

- **Filtered (impl src):** 44918/48407 regions = **92.79%** (3489 missed)
- **Raw llvm-cov totals (includes tests/bins):** 45135/53715 = 84.03%
- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa-core` | 6627 | 8541 | 1914 | 77.59% |
| `reciplexa-bind` | 2099 | 2462 | 363 | 85.26% |
| `reciplexa-eval` | 1988 | 2230 | 242 | 89.15% |
| `reciplexa-macro` | 2730 | 3004 | 274 | 90.88% |
| `reciplexa-package` | 1609 | 1740 | 131 | 92.47% |
| `reciplexa` | 1198 | 1282 | 84 | 93.45% |
| `reciplexa-mem` | 2272 | 2429 | 157 | 93.54% |
| `reciplexa-syntax` | 2998 | 3190 | 192 | 93.98% |
| `reciplexa-native` | 256 | 266 | 10 | 96.24% |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
| `reciplexa-gui` | 1640 | 1687 | 47 | 97.21% |
| `reciplexa-view` | 1653 | 1694 | 41 | 97.58% |
| `reciplexa-identity` | 931 | 945 | 14 | 98.52% |
| `reciplexa-effect` | 403 | 406 | 3 | 99.26% |
| `reciplexa-scene` | 333 | 335 | 2 | 99.40% |
| `reciplexa-runtime` | 356 | 358 | 2 | 99.44% |
| `reciplexa-lower` | 5845 | 5851 | 6 | 99.90% |
| `reciplexa-svg` | 69 | 69 | 0 | 100.00% |
| `reciplexa-proof` | 119 | 119 | 0 | 100.00% |
| `reciplexa-ir` | 179 | 179 | 0 | 100.00% |
| `reciplexa-video` | 213 | 213 | 0 | 100.00% |
| `reciplexa-harden` | 268 | 268 | 0 | 100.00% |
| `reciplexa-source` | 291 | 291 | 0 | 100.00% |
| `reciplexa-outcome` | 310 | 310 | 0 | 100.00% |
| `reciplexa-raster` | 323 | 323 | 0 | 100.00% |
| `reciplexa-opt` | 343 | 343 | 0 | 100.00% |
| `reciplexa-codec` | 352 | 352 | 0 | 100.00% |
| `reciplexa-motion` | 411 | 411 | 0 | 100.00% |
| `reciplexa-pptx` | 488 | 488 | 0 | 100.00% |
| `reciplexa-visual-ir` | 551 | 551 | 0 | 100.00% |
| `reciplexa-gui-runtime` | 604 | 604 | 0 | 100.00% |
| `reciplexa-diagnostic` | 783 | 783 | 0 | 100.00% |
| `reciplexa-types` | 826 | 826 | 0 | 100.00% |
| `reciplexa-document` | 872 | 872 | 0 | 100.00% |
| `reciplexa-backend` | 1039 | 1039 | 0 | 100.00% |
| `reciplexa-pdf` | 1523 | 1523 | 0 | 100.00% |
| `reciplexa-std` | 2226 | 2226 | 0 | 100.00% |

## Top files by missed regions

| Missed | Covered/Count | % | File |
|-------:|--------------:|------:|------|
| 1914 | — | — | `reciplexa-core` (aggregate; esp. check/elaborate/cast/unify) |
| 363 | — | — | `reciplexa-bind` (esp. resolve/module) |
| 274 | 2730/3004 | 90.88% | `reciplexa-macro` (esp. `lang_macro.rs`) |
| 242 | — | — | `reciplexa-eval` |
| 192 | — | — | `reciplexa-syntax` |
| 157 | 2272/2429 | 93.54% | `reciplexa-mem` |
| 131 | 1609/1740 | 92.47% | `reciplexa-package` |
| 47 | 1640/1687 | 97.21% | `reciplexa-gui` (mostly `fonts.rs` OS install path) |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.
- **gui / package / macro / mem push (this update):**
  - `reciplexa-gui`: **36.08% → 97.21%** — moved `preview_paint` + `fonts` into the lib; unit-tested geometry hits, aspect lock, and egui paint paths without a live window.
  - `reciplexa-package`: **83.85% → 92.47%** — lockfile consistency errors, workspace/rpxm/rpi edges, load discover/resolve failures.
  - `reciplexa-macro`: **89.65% → 90.88%** — MAC-001 hygiene/expand failure corpus (duplicate/conflict/cycle/arity/reserved).
  - `reciplexa-mem`: **89.21% → 93.54%** — observably_equal numerics, lower Seq/LetRec/Lambda/App/If, Select error path.
- Remaining holes of note: GUI CJK font success path (machine-dependent), package `load.rs` IO/alias overlay branches, lang_macro budget/gensym/hygiene interior, mem Select-as-Bool (Bool lowers to String today).
