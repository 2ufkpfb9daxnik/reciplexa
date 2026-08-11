# Workspace region coverage status

Baseline llvm-cov **region** coverage for the Rust workspace (language + hosts: gui/pdf/pptx and related).

Analysis excludes `tests/`, `main.rs`, and `bin/` paths (implementation/src focus).

Generated via `cargo llvm-cov --workspace --json --offline` → `.tmp/cov_workspace.json` / `_cov_table.py`.

## Overall

- **Filtered (impl src):** 47874/49294 regions = **97.12%** (1420 missed).
- **Raw (incl. tests/bins):** 48091/54602 = 88.08%.
- **Target:** 100% region coverage on Rust-native implementation code (at least ~99%).

## Packages by region % (worst first)

| Package | Covered | Count | Missed | % |
|---------|--------:|------:|-------:|------:|
| `reciplexa-core` | 8719 | 9532 | 813 | 91.47% |
| `reciplexa` | 1198 | 1282 | 84 | 93.45% |
| `reciplexa-syntax` | 3000 | 3190 | 190 | 94.04% |
| `reciplexa-bind` | 2395 | 2524 | 129 | 94.89% |
| `reciplexa-eval` | 2129 | 2223 | 94 | 95.77% |
| `reciplexa-test` | 190 | 197 | 7 | 96.45% |
| `reciplexa-view` | 1653 | 1694 | 41 | 97.58% |
| `reciplexa-gui` | 1670 | 1687 | 17 | 98.99% |
| `reciplexa-package` | 1772 | 1789 | 17 | 99.05% |
| `reciplexa-macro` | 2750 | 2767 | 17 | 99.39% |
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
| 813 | 8719/9532 | 91.47% | `reciplexa-core` (esp. elaborate ~437, check ~242) |
| 190 | 3000/3190 | 94.04% | `reciplexa-syntax` |
| 129 | 2395/2524 | 94.89% | `reciplexa-bind` |
| 94 | 2129/2223 | 95.77% | `reciplexa-eval` |
| 84 | 1198/1282 | 93.45% | `reciplexa` (cli) |
| 41 | 1653/1694 | 97.58% | `reciplexa-view` |
| 17 | 1670/1687 | 98.99% | `reciplexa-gui` |
| 17 | 1772/1789 | 99.05% | `reciplexa-package` |
| 17 | 2750/2767 | 99.39% | `reciplexa-macro` |

## Notes

- Do not treat this file as a substitute for live llvm-cov; re-run after filling holes.
- **core / eval / bind residual wave (this update):**
  - Full workspace remasure: filtered **95.16% → 97.12%**.
  - `reciplexa-core`: **90.68% → 91.47%** (workspace) — round8 CoreExpr/cast/unify + elaborate cfg(test) private helpers / denser data-param Err matrix. Residual: elaborate `?` ends + check Err-only arms (~813).
  - `reciplexa-bind`: **90.08% → 94.89%** (workspace; scoped-only remasure shows lower region count on module.cfg(test)) — soft export/binding continue **eliminated** (`binding_for_export` internal Err) + round9 IO/interface tests. Residual: resolve soft skips + load_module_tree OS edges (~129).
  - `reciplexa-eval`: **95.73% → 95.77%** — Cont other arm cascade round8. Residual ~94 Cont/deep-resume arms.
- See `lang/coverage-holdouts.md` for justified misses and prefer-eliminate residuals.
- **Graphics Slice C (static):** `packages/graphics` expanded (line/path/fill/stroke/paint, page sizes, rgba); GUI interim retained; example `examples/pkg_graphics_static.rpx`.
- Remaining under-99: core, cli, syntax, bind, eval, test, view, gui (gui tip 98.99%).
