# GUI CST sync v2 (package AST rewrite)

**Status:** active  
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
| S5 | Migrate GUI golden to package; audit interim examples | **done** — `black_circle.rpx` is package-shaped; interim twin `interim_black_circle.rpx` |
| S6 | Retire interim keyword tables | **blocked** — interim examples remain (`interim_black_circle`, `text_line`, `two_pages`, `letter_opacity`, `shapes`, `transforms`, `paths`, `image`, …); do not delete tables yet |

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

**Blocked** until remaining interim examples migrate off bare `(page)/(circle)` keyword surfaces. Critical GUI golden (`black_circle.rpx`) is package-shaped as of S5; keyword tables stay for `interim_black_circle.rpx`, `text_line.rpx`, `two_pages.rpx`, `letter_opacity.rpx`, and other interim CST examples.

Do **not** delete types/lower/bind keyword arms while those examples remain.
