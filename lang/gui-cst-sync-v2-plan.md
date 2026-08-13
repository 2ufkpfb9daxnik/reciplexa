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
| S5 | Migrate GUI golden to package; audit interim examples | `black_circle` → package twin as golden |
| S6 | Retire interim keyword tables | delete types/lower/bind surface arms; gate green |

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

## Tracking

Update this file + `lang/coverage-holdouts.md` / `implemented-features.md` as units land.
