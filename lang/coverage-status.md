# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline`.

## Overall

- **Filtered (impl src):** 45907/48243 regions = **95.16%** (2336 missed)
- **Raw llvm-cov totals (includes tests/bins):** 46124/53551 = 86.13%
- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa-core` | 6964 | 8541 | 1577 | 81.54% |
| `reciplexa-bind` | 2254 | 2457 | 203 | 91.74% |
| `reciplexa` | 1198 | 1282 | 84 | 93.45% |
| `reciplexa-eval` | 2095 | 2230 | 135 | 93.95% |
| `reciplexa-syntax` | 2998 | 3190 | 192 | 93.98% |
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
| 1577 | 6964/8541 | 81.54% | `reciplexa-core` (esp. check/elaborate) |
| 203 | 2254/2457 | 91.74% | `reciplexa-bind` |
| 192 | 2998/3190 | 93.98% | `reciplexa-syntax` |
| 135 | 2095/2230 | 93.95% | `reciplexa-eval` |
| 84 | 1198/1282 | 93.45% | `reciplexa` (cli) |
| 47 | 1640/1687 | 97.21% | `reciplexa-gui` |
| 41 | 1653/1694 | 97.58% | `reciplexa-view` |
| 22 | 2745/2767 | 99.20% | `reciplexa-macro` |
| 17 | 1772/1789 | 99.05% | `reciplexa-package` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.
- **package / macro / mem + tip crates (this update):** remasured with `cargo llvm-cov --workspace --json --offline`.
  - `reciplexa-package`: **92.47% → 99.05%** — load/rpi/rpxm path-dep, ACL/share Io, cycles, interface stubs.
  - `reciplexa-macro`: **90.88% → 99.20%** — let/fn/local hygiene, budget/nested expand, Embed layout; unit tests moved out of `lang_macro.rs`.
  - `reciplexa-mem`: **93.54% → 99.88%** — `MemLiteral::Bool` + Select plumbing, Handle/With/Forward lower, exec/equiv leftovers.
  - Tipped to **100%:** effect, identity, native, scene (cheap Display/Err/Terminal arms).
- Remaining under-99 (outside this batch focus): core/bind/eval (parallel), cli, syntax, test, gui, view.
