# document package sources

Module bodies are Rust native (`reciplexa_package::domain_bodies` / `std_domain_natives`).

| Module | Status |
|--------|--------|
| `page` | **native** (N5.1) — mirrors `reciplexa_std::document` constructors as tagged records; Wave 15 `paragraph-indented`; Wave 24 `columns` / `block-columns` → `doc-columns` |

This package is the **document** surface (flow / section / heading / …). Scene paper pages remain `graphics/page`. Example `pkg_columns.rpx` demos multi-column stub placement via `measure_columns`. Interim CST keyword tables are production-isolated (`interim-surface`).
