# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline` → `.tmp/cov.json` / `_cov_table.py`.

Package remasure for this wave: `cargo llvm-cov --package reciplexa-core --json --offline` → `.tmp/core_cov.json`.

## Overall

- **Filtered (impl src):** 47891/49269 regions = **97.20%** (1378 missed) — last full workspace table; core package remasure below.

- **Raw (incl. tests/bins):** 48108/54577 = 88.15%.

- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa-core` | 10297 | 10782 | 485 | **95.50%** (prior package remasure; see note) |
| `reciplexa` | 1198 | 1282 | 84 | 93.45% |
| `reciplexa-syntax` | 3002 | 3190 | 188 | 94.11% |
| `reciplexa-bind` | 2399 | 2524 | 125 | 95.05% |
| `reciplexa-eval` | 2115 | 2198 | 83 | 96.22% (+ graphics_value bridge; remasure pending) |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
| `reciplexa-view` | 1653 | 1694 | 41 | 97.58% |
| `reciplexa-gui` | 1670 | 1687 | 17 | 98.99% |
| `reciplexa-package` | 1772 | 1789 | 17 | 99.05% |
| `reciplexa-runtime` | 356 | 358 | 2 | 99.44% |
| `reciplexa-macro` | 2752 | 2767 | 15 | 99.46% |
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
| 485 | 10297/10782 | 95.50% | `reciplexa-core` (esp. elaborate / check; cast/unify residual matrices landed) |
| 188 | 3002/3190 | 94.11% | `reciplexa-syntax` |
| 125 | 2399/2524 | 95.05% | `reciplexa-bind` |
| 84 | 1198/1282 | 93.45% | `reciplexa` (cli) |
| 83 | 2115/2198 | 96.22% | `reciplexa-eval` |
| 41 | 1653/1694 | 97.58% | `reciplexa-view` |
| 17 | 1670/1687 | 98.99% | `reciplexa-gui` |
| 17 | 1772/1789 | 99.05% | `reciplexa-package` |
| 15 | 2752/2767 | 99.46% | `reciplexa-macro` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.

- **core residual wave (this update):**
  - Prior package remasure: **95.50%** (485 missed). Cast Variant∩ / nested Compose + unify Any/lacks/open exclusive Err matrices landed.
  - Fresh `cargo llvm-cov -p reciplexa-core --json` this session warned **mismatched data** and inflated src region counts via large `#[cfg(test)]` helpers (summary appeared ~82% incl. polluted attribution). Prefer the prior 95.50% package baseline until a clean remasure without mismatch warnings.
  - Still short of ≥99% (~381 regions at 10782 count); next: elaborate `?` ends + check OpenRecord/get Err leaves.

- See `lang/coverage-holdouts.md` for justified misses and prefer-eliminate residuals.

- Remaining under-99: core, cli, syntax, bind, eval, test, view, gui (gui tip 98.99%).
