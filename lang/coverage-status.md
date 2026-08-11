# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline`.

## Overall

- **Filtered (impl src):** 41663/48020 regions = **86.76%** (6357 missed)
- **Raw llvm-cov totals (includes tests/bins):** 41880/53328 = 78.53%
- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa-gui` | 469 | 1300 | 831 | 36.08% |
| `reciplexa-eval` | 1348 | 2230 | 882 | 60.45% |
| `reciplexa-bind` | 1689 | 2462 | 773 | 68.60% |
| `reciplexa-core` | 5885 | 8541 | 2656 | 68.90% |
| `reciplexa-package` | 1459 | 1740 | 281 | 83.85% |
| `reciplexa-mem` | 2167 | 2429 | 262 | 89.21% |
| `reciplexa-macro` | 2693 | 3004 | 311 | 89.65% |
| `reciplexa` | 1198 | 1282 | 84 | 93.45% |
| `reciplexa-syntax` | 2998 | 3190 | 192 | 93.98% |
| `reciplexa-native` | 256 | 266 | 10 | 96.24% |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
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
| 1184 | 1945/3129 | 62.16% | `reciplexa-core/src/check.rs` |
| 759 | 44/803 | 5.48% | `reciplexa-gui/src/preview_paint.rs` |
| 722 | 2877/3599 | 79.94% | `reciplexa-core/src/elaborate.rs` |
| 722 | 1148/1870 | 61.39% | `reciplexa-eval/src/eval.rs` |
| 632 | 948/1580 | 60.00% | `reciplexa-bind/src/resolve.rs` |
| 533 | 473/1006 | 47.02% | `reciplexa-core/src/cast.rs` |
| 295 | 1155/1450 | 79.66% | `reciplexa-macro/src/lang_macro.rs` |
| 198 | 406/604 | 67.22% | `reciplexa-core/src/unify.rs` |
| 147 | 147/294 | 50.00% | `reciplexa-eval/src/value.rs` |
| 141 | 657/798 | 82.33% | `reciplexa-bind/src/module.rs` |
| 137 | 384/521 | 73.70% | `reciplexa-package/src/load.rs` |
| 123 | 392/515 | 76.12% | `reciplexa-mem/src/lower.rs` |
| 62 | 309/371 | 83.29% | `reciplexa-syntax/src/number_lit.rs` |
| 59 | 0/59 | 0.00% | `reciplexa-gui/src/fonts.rs` |
| 57 | 481/538 | 89.41% | `reciplexa/src/cli.rs` |
| 54 | 465/519 | 89.60% | `reciplexa-package/src/rpxm.rs` |
| 53 | 124/177 | 70.06% | `reciplexa-package/src/rpi.rs` |
| 52 | 508/560 | 90.71% | `reciplexa-mem/src/exec.rs` |
| 47 | 653/700 | 93.29% | `reciplexa-syntax/src/parse.rs` |
| 41 | 1653/1694 | 97.58% | `reciplexa-view/src/lib.rs` |
| 40 | 670/710 | 94.37% | `reciplexa-syntax/src/lexer.rs` |
| 36 | 146/182 | 80.22% | `reciplexa-mem/src/equiv.rs` |
| 26 | 111/137 | 81.02% | `reciplexa-package/src/lockfile.rs` |
| 25 | 282/307 | 91.86% | `reciplexa-syntax/src/string_lit.rs` |
| 18 | 253/271 | 93.36% | `reciplexa-mem/src/perceus.rs` |
| 17 | 77/94 | 81.91% | `reciplexa-core/src/ty.rs` |
| 16 | 973/989 | 98.38% | `reciplexa-macro/src/doc_layout.rs` |
| 16 | 208/224 | 92.86% | `reciplexa-mem/src/conservative.rs` |
| 15 | 128/143 | 89.51% | `reciplexa-mem/src/ir.rs` |
| 14 | 299/313 | 95.53% | `reciplexa/src/pipeline.rs` |
| 13 | 53/66 | 80.30% | `reciplexa-eval/src/control.rs` |
| 13 | 391/404 | 96.78% | `reciplexa-syntax/src/ident.rs` |
| 13 | 418/431 | 96.98% | `reciplexa/src/document_pipeline.rs` |
| 12 | 119/131 | 90.84% | `reciplexa-gui/src/canvas_sync.rs` |
| 10 | 117/127 | 92.13% | `reciplexa-native/src/registry.rs` |
| 10 | 65/75 | 86.67% | `reciplexa-package/src/workspace.rs` |
| 9 | 184/193 | 95.34% | `reciplexa-identity/src/package.rs` |
| 6 | 63/69 | 91.30% | `reciplexa-test/src/outcome.rs` |
| 5 | 13/18 | 72.22% | `reciplexa-identity/src/provenance.rs` |
| 5 | 281/286 | 98.25% | `reciplexa-syntax/src/markup.rs` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.
- Filling coverage gaps is tracked separately; this commit is baseline status only.

