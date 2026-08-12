# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline` → `.tmp/cov.json` / `_cov_table.py`.

## Overall

- **Filtered (impl src):** 51111/52255 regions = **97.81%** (1144 missed)

- **Raw (incl. tests/bins):** 51330/57568 = 89.16%.

- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa-eval` | 3300 | 3575 | 275 | **92.31%** (graphics_value bridge residual) |
| `reciplexa-bind` | 2399 | 2524 | 125 | 95.05% |
| `reciplexa` | 1280 | 1343 | 63 | 95.31% |
| `reciplexa-core` | 10455 | 10936 | 481 | 95.60% |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
| `reciplexa-syntax` | 3099 | 3190 | 91 | 97.15% |
| `reciplexa-gui` | 1692 | 1735 | 43 | 97.52% |
| `reciplexa-package` | 1802 | 1831 | 29 | 98.42% |
| `reciplexa-runtime` | 356 | 358 | 2 | 99.44% |
| `reciplexa-macro` | 2753 | 2767 | 14 | 99.49% |
| `reciplexa-view` | 1743 | 1748 | 5 | **99.71%** |
| `reciplexa-mem` | 2443 | 2446 | 3 | 99.88% |
| `reciplexa-lower` | 5845 | 5851 | 6 | 99.90% |
| `reciplexa-backend` | 1039 | 1039 | 0 | 100.00% |
| `reciplexa-codec` | 352 | 352 | 0 | 100.00% |
| `reciplexa-diagnostic` | 783 | 783 | 0 | 100.00% |
| `reciplexa-document` | 872 | 872 | 0 | 100.00% |
| `reciplexa-effect` | 406 | 406 | 0 | 100.00% |
| `reciplexa-gui-runtime` | 604 | 604 | 0 | 100.00% |
| `reciplexa-harden` | 268 | 268 | 0 | 100.00% |
| `reciplexa-identity` | 945 | 945 | 0 | 100.00% |
| `reciplexa-ir` | 179 | 179 | 0 | 100.00% |
| `reciplexa-motion` | 411 | 411 | 0 | 100.00% |
| `reciplexa-native` | 266 | 266 | 0 | 100.00% |
| `reciplexa-opt` | 343 | 343 | 0 | 100.00% |
| `reciplexa-outcome` | 310 | 310 | 0 | 100.00% |
| `reciplexa-pdf` | 1523 | 1523 | 0 | 100.00% |
| `reciplexa-pptx` | 488 | 488 | 0 | 100.00% |
| `reciplexa-proof` | 119 | 119 | 0 | 100.00% |
| `reciplexa-raster` | 323 | 323 | 0 | 100.00% |
| `reciplexa-scene` | 347 | 347 | 0 | 100.00% |
| `reciplexa-source` | 291 | 291 | 0 | 100.00% |
| `reciplexa-std` | 2226 | 2226 | 0 | 100.00% |
| `reciplexa-svg` | 69 | 69 | 0 | 100.00% |
| `reciplexa-types` | 826 | 826 | 0 | 100.00% |
| `reciplexa-video` | 213 | 213 | 0 | 100.00% |
| `reciplexa-visual-ir` | 551 | 551 | 0 | 100.00% |

## Top files by missed regions

| Missed | Covered/Count | % | File |
|-------:|--------------:|------:|------|
| 481 | 10455/10936 | 95.60% | `reciplexa-core` (elaborate / check / unify / cast) |
| 275 | 3300/3575 | 92.31% | `reciplexa-eval` (esp. `graphics_value.rs` + Cont edges) |
| 125 | 2399/2524 | 95.05% | `reciplexa-bind` |
| 91 | 3099/3190 | 97.15% | `reciplexa-syntax` |
| 63 | 1280/1343 | 95.31% | `reciplexa` (cli) |
| 43 | 1692/1735 | 97.52% | `reciplexa-gui` |
| 7 | 190/197 | 96.45% | `reciplexa-test` (derive noise) |
| 5 | 1743/1748 | 99.71% | `reciplexa-view` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.

- **This wave:** view tip (empty bounds + expect-helper arms), syntax literal/lexer matrices, cli CliEvalHost, gui fonts/canvas soft-align, core round12 OpenRecord/pattern/cast, eval graphics_value bridge suites. Filtered overall **97.20% → 97.81%**.

- **eval note:** Slice D `graphics_value.rs` landed with a large new region count; bridge suites recovered package % from a mid-wave ~82% dip to **92.31%**. Still the worst package; continue paint/error leaves before promoting holdouts.

- **gui note:** fonts success path is sensitive to `WINDIR` mutation races in unit tests; tip remasures can under-count relative to a clean workspace run.

- See `lang/coverage-holdouts.md` for justified misses and prefer-eliminate residuals.

- Remaining under-99: eval, bind, cli, core, test, syntax, gui (view tipped to 99.71%).
