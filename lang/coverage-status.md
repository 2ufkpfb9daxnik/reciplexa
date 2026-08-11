# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline` (package rows for core/bind/eval remasured scoped).

## Overall

- **Filtered (impl src):** last full workspace was 45907/48243 regions = **95.16%** (2336 missed) — not remasured this pass.
- **Scoped remasure (core+bind+eval):** 13035/14268 = **91.36%** (instrumented src-only view for those crates).
- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa-core` | 8583 | 9465 | 882 | 90.68% |
| `reciplexa-bind` | 2324 | 2580 | 256 | 90.08% |
| `reciplexa` | 1198 | 1282 | 84 | 93.45% |
| `reciplexa-syntax` | 2998 | 3190 | 192 | 93.98% |
| `reciplexa-eval` | 2128 | 2223 | 95 | 95.73% |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
| `reciplexa-gui` | 1640 | 1687 | 47 | 97.21% |
| `reciplexa-view` | 1653 | 1694 | 41 | 97.58% |
| `reciplexa-package` | 1772 | 1789 | 17 | 99.05% |
| `reciplexa-macro` | 2745 | 2767 | 22 | 99.20% |
| `reciplexa-runtime` | 356 | 358 | 2 | 99.44% |
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
| 882 | 8583/9465 | 90.68% | `reciplexa-core` (esp. elaborate ~473, check ~256) |
| 256 | 2324/2580 | 90.08% | `reciplexa-bind` |
| 192 | 2998/3190 | 93.98% | `reciplexa-syntax` |
| 95 | 2128/2223 | 95.73% | `reciplexa-eval` |
| 84 | 1198/1282 | 93.45% | `reciplexa` (cli) |
| 47 | 1640/1687 | 97.21% | `reciplexa-gui` |
| 41 | 1653/1694 | 97.58% | `reciplexa-view` |
| 22 | 2745/2767 | 99.20% | `reciplexa-macro` |
| 17 | 1772/1789 | 99.05% | `reciplexa-package` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.
- **core / eval / bind push (this update):** remasured with `cargo llvm-cov -p reciplexa-core -p reciplexa-eval -p reciplexa-bind --json --offline`.
  - `reciplexa-core`: **84.40% → 90.68%** — round7 CoreExpr/check + cfg(test) private helpers (is_expansive, subst_*, occurs/enforce_lacks, cast numeric-promote), dense elaborate Err/Ok unit matrix. Residual: elaborate.rs (~473) dominate; many `?` Err-only region ends in check.
  - `reciplexa-eval`: **95.07% → 95.73%** — deep Cont re-perform + compound Forward; removed dead post-expand parse Err. Remaining ~95 mostly Cont other / deep_resume arms.
  - `reciplexa-bind`: **92.80% → 90.08%** (region count rose after link refactor to validated unit indices; covered regions also rose). Round7–8 resolve/module tests. Residual: soft `continue` on export/binding divergence, load_module_tree IO/UTF-8 edges, resolve pattern outliers.
- Prior package/macro/mem tip batch kept; overall workspace table mixed (core/bind/eval from scoped remasure; others last full workspace run).
- Remaining under-99: core/bind/eval, cli, syntax, test, gui, view.
