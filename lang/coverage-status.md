# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline` → `.tmp/cov.json` / `_cov_table.py`.

## Overall

- **Filtered (impl src):** 51621/52642 regions = **98.06%** (1021 missed)

- **Raw (incl. tests/bins):** 51843/57960 = 89.45%.

- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa` | 1547 | 1647 | 100 | 93.93% |
| `reciplexa-bind` | 2399 | 2524 | 125 | 95.05% |
| `reciplexa-eval` | 3185 | 3343 | 158 | 95.27% |
| `reciplexa-core` | 10760 | 11220 | 460 | **95.90%** |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
| `reciplexa-syntax` | 3099 | 3190 | 91 | 97.15% |
| `reciplexa-package` | 1827 | 1862 | 35 | 98.12% |
| `reciplexa-gui` | 1720 | 1735 | 15 | **99.14%** |
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
| 460 | 10760/11220 | 95.90% | `reciplexa-core` (elaborate / check / unify / cast) |
| 158 | 3185/3343 | 95.27% | `reciplexa-eval` (graphics_value bridge + eval Cont edges) |
| 125 | 2399/2524 | 95.05% | `reciplexa-bind` |
| 100 | 1547/1647 | 93.93% | `reciplexa` (cli) |
| 91 | 3099/3190 | 97.15% | `reciplexa-syntax` |
| 15 | 1720/1735 | 99.14% | `reciplexa-gui` |
| 7 | 190/197 | 96.45% | `reciplexa-test` (derive noise) |
| 5 | 1743/1748 | 99.71% | `reciplexa-view` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.

- **This wave:** core round13 (elaborate record-field/pattern Err matrix, check infer/coerce cfg(test) helpers, cast compose leaves). Filtered overall **97.95% → 98.06%**; **core 95.60% → 95.90%**.

- **eval note:** `graphics_value.rs` bridge leaves largely covered; residual ~75 regions are paint/? gap ends and `eval.rs` Cont other (~84). Holdouts in `lang/coverage-holdouts.md`.

- **core / bind / cli / syntax / gui:** still under 99%; next chase targets after eval floor.

- See `lang/coverage-holdouts.md` for justified misses and prefer-eliminate residuals.

- Remaining under-99: cli, bind, core, eval, test, syntax, gui (view tipped to 99.71%).
