# Language kernel completion plan (excl. PKG-001)

Goal: complete LEX→SYN→MAC→RES→TYP/ROW/EFF→Core→EVAL/BND (+ MOD/KER/RSC/EDT/TST language parts) per `lang/specification.md`. Packages (graphics/math/Japanese) deferred.

## Spec alignment progress (2026-08-11)

### Fixed this round
| Area | Spec | Status |
|---|---|---|
| DAT match `_` / `bind` / nested payload patterns | DAT §14–16 | **done** (`CorePattern`; elaborate + eval + check) |
| EFF ambient `(log …)` / `(random)` | EFF-001 DD-EFF-013 | **done** (→ `Perform`; Core `handle` still primary) |
| `local` / `rec` declaration groups | SYN §13.4–13.5 | **done** (local→lets; rec→`LetRec`) |
| MOD `import` path / `as` / flat `only` | MOD §6 | **done** (parse + link skeleton; qualified refs deferred) |
| KER ops `/ > <= >= !=` | SYN §5 / KER | **done** (`primitive_env` + Ident lex) |
| String literals: no backslash escapes | SYN §8.1 | **done** (lexer + elaborate/effect/lower read path) |

### Remaining gaps (pre-PKG / language)
| Gap | Notes |
|---|---|
| EFF Surface `with` / first-class handler values | Spec DD-EFF-012; Core still op+fn `handle` |
| EFF multi-shot / shallow handlers | intentional deferral |
| `"""` multi-quote string delimiters | SYN §8.2–8.3 not implemented |
| GUI/macro writers that invent `\n`/`\"` escapes | write path should use `"""` for specials |
| MOD qualified refs (`color/black`) after `as` | alias stored; no path resolution yet |
| MOD `only x as y` item rename | parse deferred |
| Top-level `rec` groups spanning later `val` | expression `rec` only |
| `local` `var` / nested `rec` inside local | val-only decls for now |
| DAT literal patterns / tuple patterns | §15.5 / §17 |
| DAT multi-payload / parameterized ADTs | DAT-001 deferral |
| `/` not in normal idents (path-only) | `/` still `ident_continue` so `graphics/color` is one Ident |
| ROW unrestricted / multi-tail | ROW-001 deferral |
| Full MOD functors / signatures | MOD-001 deferral |

## Status: **COMPLETE for pre-PKG** (with gaps above tracked)

Language kernel is complete enough for pre-PKG. Intentional deferrals listed below.

## Ordered work
1. CoreExpr::Var + env; if; n-ary fn/app — **done**
2. Surface elaborator: val/fn/let/seq → Core — **done** (+ perform/handle; ambient apps; local/rec)
3. RES lexical resolve over binders — **done** (`resolve_language_source` + `BindingMap`)
4. MAC-001 hygienic macros (language); quarantine graphics macros — **done** (`expand_language`)
5. TYP-001 on Core (+ dynamic stub) — **done** (`typecheck_language_source`, `CoreType::Dynamic`)
6. ROW-001 fragment — **done** (closed + `OpenRecord` row vars; `Lacks` enforced in unify)
7. DAT interim: data/match — **done** (surface `data`/`match` → Core; `_`/`bind`/nested; **static exhaustiveness**)
8. EFF-001 deep handlers — **done** (deep one-shot resume; ambient apps; **EffectRow on Fun** via `infer_with_effects`)
9. BND letrec/var/set — **done** (`LetRec`, `LocalVar`/`Set` + cell alive escape check; `local`/`rec` sugar)
10. MOD-001 outer module + import — **done** (`elaborate_units` + **`load_module_tree`** + import `as`/path/`only`)
11. KER/RSC typed host ops — **done** (`+ - * / < > <= >= = !=` builtins; `MemoryFsHost` for `read-file`/`write-file`)
12. EDT thread BindingId; retire prototype "language" tests — **done** (`BindingMap` use-sites; `document_surface` cfg module; `lang_kernel_suite` language gate)
13. GUI text-move sync tests (expanded vs authoring) — **done** (+ `japanese_page` nudge regression)

## Gate
```
CARGO_TARGET_DIR=d:\reciplexa\target TEMP/TMP=d:\reciplexa\.tmp
cargo fmt --all
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --offline -p reciplexa-gui
```
**Last gate: green** (fmt / clippy -D warnings / test --workspace / check gui).

## Intentional deferrals (post pre-PKG / PKG+)
| ID | Deferred |
|---|---|
| **PKG-001** | graphics/math/Japanese packages |
| **EFF-001** | Multi-shot / shallow choice handlers; first-class `with` handler values |
| **MOD-001** | Full ML functors / signatures; qualified import refs |
| **KER-001** | Full typed Rust/foreign ABI (beyond BuiltinOp + EffectHost) |
| **DAT-001** | Multi-payload / parameterized ADTs; literal/tuple patterns |
| **BND-001** | Full escape analysis; typed `var` store |
| **RSC-001** | Richer resource catalog / path safety |
| **EDT-001** | SyntaxNodeId provenance through elaborate→eval (BindingId use-site map done) |
| **ROW-001** | Full unrestricted row tallying / multi-tail polymorphism |
| **SYN-001** | `"""` multi-quote string delimiters |
