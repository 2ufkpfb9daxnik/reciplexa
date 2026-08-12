# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline` → `.tmp/cov.json` / `_cov_table.py`.

## Overall

- **Filtered (impl src):** 51136/52207 regions = **97.95%** (1071 missed)

- **Raw (incl. tests/bins):** 51358/57525 = 89.28%.

- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa` | 1399 | 1500 | 101 | 93.27% |
| `reciplexa-bind` | 2399 | 2524 | 125 | 95.05% |
| `reciplexa-core` | 10455 | 10936 | 481 | 95.60% |
| `reciplexa-eval` | 3185 | 3343 | 158 | **95.27%** (graphics_value bridge recovered) |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
| `reciplexa-syntax` | 3099 | 3190 | 91 | 97.15% |
| `reciplexa-gui` | 1692 | 1735 | 43 | 97.52% |
| `reciplexa-package` | 1823 | 1858 | 35 | 98.12% |
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
| 158 | 3185/3343 | 95.27% | `reciplexa-eval` (graphics_value bridge + eval Cont edges) |
| 125 | 2399/2524 | 95.05% | `reciplexa-bind` |
| 101 | 1399/1500 | 93.27% | `reciplexa` (cli) |
| 91 | 3099/3190 | 97.15% | `reciplexa-syntax` |
| 43 | 1692/1735 | 97.52% | `reciplexa-gui` |
| 7 | 190/197 | 96.45% | `reciplexa-test` (derive noise) |
| 5 | 1743/1748 | 99.71% | `reciplexa-view` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.

- **This wave:** multipage `graphics_value` bridge (cons/`pages` records, multi-shape page content), external bridge suites (round2–4 + expanded integration matrix), eval round11–12 residual leaves, inline `graphics_value` unit tests removed (integration suites attribute to src). Filtered overall **97.81% → 97.95%**; **eval 92.31% → 95.27%** (≥95% floor restored).

- **eval note:** `graphics_value.rs` bridge leaves largely covered; residual ~75 regions are paint/? gap ends and `eval.rs` Cont other (~84). Holdouts in `lang/coverage-holdouts.md`.

- **core / bind / cli / syntax / gui:** still under 99%; next chase targets after eval floor.

- See `lang/coverage-holdouts.md` for justified misses and prefer-eliminate residuals.

- Remaining under-99: cli, bind, core, eval, test, syntax, gui (view tipped to 99.71%).
