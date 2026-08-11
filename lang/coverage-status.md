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
| `reciplexa-core` | 6775 | 8541 | 1766 | 79.32% |
| `reciplexa-bind` | 2163 | 2462 | 299 | 87.86% |
| `reciplexa-macro` | 2730 | 3004 | 274 | 90.88% |
| `reciplexa-package` | 1609 | 1740 | 131 | 92.47% |
| `reciplexa-eval` | 2069 | 2230 | 161 | 92.78% |
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
| 1766 | 6775/8541 | 79.32% | `reciplexa-core` (esp. check/elaborate/cast/unify) |
| 299 | 2163/2462 | 87.86% | `reciplexa-bind` (esp. resolve/module) |
| 274 | 2730/3004 | 90.88% | `reciplexa-macro` (esp. `lang_macro.rs`) |
| 161 | 2069/2230 | 92.78% | `reciplexa-eval` |
| 192 | — | — | `reciplexa-syntax` |
| 157 | 2272/2429 | 93.54% | `reciplexa-mem` |
| 131 | 1609/1740 | 92.47% | `reciplexa-package` |
| 47 | 1640/1687 | 97.21% | `reciplexa-gui` (mostly `fonts.rs` OS install path) |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.
- **core / eval / bind push (this update):** remasured with `cargo llvm-cov -p reciplexa-core -p reciplexa-eval -p reciplexa-bind`.
  - `reciplexa-core`: **68.90% → 79.32%** — cast/unify/ty helpers, coerce/insert-casts, Handle/LetRec/Set typing, elaborator error surfaces (check.rs + elaborate.rs still dominate misses).
  - `reciplexa-eval`: **60.45% → 92.78%** — deep resume across expr forms, cast evidence kinds, MemoryFsHost/value surfaces, Resumed/Forward propagation.
  - `reciplexa-bind`: **68.60% → 87.86%** — language resolve forms (data/let/match/import), module interface/load error arms.
- Remaining holes of note: `reciplexa-core` check/elaborate (~1500 regions), bind resolve residual match/import arms, eval deep-resume `other`/short-circuit dead Err regions.
