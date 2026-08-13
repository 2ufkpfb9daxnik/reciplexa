# GUI CST sync v2 (package AST rewrite)

**Status:** active (S6a–S6b done)
**Goal:** Make package-shaped authoring (`(import …)(val main (page …))`) writable in the GUI (nudge/size), then migrate goldens and **retire interim keyword tables**.

**Normative context:** N5 dual-path left interim `(page)/(circle)` for writable CST sync. This phase closes that holdout.

## Non-goals (keep soft-refuse)

- Markup authoring rewritten via synthetic expand (`markup_ja`) — still soft-refuse edits
- Non-literal geometry (`(+ x 1)`, let-bound coords)
- Full group/reorder CRUD on package AST (partial OK)
- `document/page` (`doc-*`) layout sync (graphics/page first)

## Units

| ID | Unit | Done when |
|----|------|-----------|
| S0 | Package page locator + paint wrappers (`fill`/`stroke`/`paint`/`list`) | **done** — `sync/package.rs` + unit tests |
| S1 | Circle nudge + layers for package (`pkg_black_circle`) | **done** — GUI nudge updates authoring numbers; keyword tables kept |
| S2 | Package size targets (circle radius) | **done** — scale radius on package circle |
| S3 | More shapes + translate wrap parity | **done** — rect/ellipse/text/line (+ wrap rules) |
| S4 | Multipage + align rules (markup stays refuse) | **done** — multipage package; markup soft-refuse |
| S5 | Migrate GUI golden to package; audit interim examples | **done** — `black_circle.rpx` is package-shaped; interim twin quarantined |
| S5b | Migrate remaining graphics examples; quarantine fixtures | **done** — product `examples/` package-shaped; keyword fixture at `crates/reciplexa-lower/tests/fixtures/interim_page.rpx` |
| S6a | Deprecate interim ingest (require-package gate) | **done** — `RECIPLEXA_REQUIRE_PACKAGE=1` rejects bare top-level `(page …)`; keyword tables still in types/lower/bind |
| S6b | Delete production keyword arms (or `#[cfg(test)]` quarantine) | **done** — keyword arms behind `interim-surface` / `cfg(test)`; production pipeline always refuses bare `(page …)`; fixture crates enable the feature in the Env D gate |

## Approach

- **Locate** with CST (`SyntaxNode` spans); **mutate** authoring **string** (same hybrid as interim sync).
- Paint wrappers are transparent for layer flatten (match eval bridge).
- Do not sync against `RuntimeValue` records.

## Gate

```
CARGO_TARGET_DIR=d:\reciplexa\target TEMP/TMP=d:\reciplexa\.tmp
cargo fmt --all
cargo clippy -p reciplexa-lower -p reciplexa-gui --all-targets --offline -- -D warnings
cargo test -p reciplexa-lower -p reciplexa-gui --offline
cargo check --offline -p reciplexa-gui
```

## S6 status

**S6a–S6b done.** Product `examples/` GUI/render surface is package-shaped. Production preview/typecheck/lower reject keyword `(page)/(circle)` tables (`interim-surface` feature / `cfg(test)` only). Pipeline always refuses bare top-level `(page …)`. Fixture lower: `lower_interim_source` / enable `interim-surface` on `reciplexa-lower` / `reciplexa-types` / `reciplexa-bind`.
