# document package sources

Module bodies are Rust native (`reciplexa_package::domain_bodies` / `std_domain_natives`).

| Module | Status |
|--------|--------|
| `page` | **native** (N5.1) — flow constructors; Step 14 slice 2 adds `page-framed` / `margins` / `pagebreak` / `block-pagebreak` |

This package is the **document** surface (flow / section / heading / …). Scene paper pages remain `graphics/page`. Example `pkg_columns.rpx` demos multi-column stub placement via `measure_columns`. Interim CST keyword tables are production-isolated (`interim-surface`).
