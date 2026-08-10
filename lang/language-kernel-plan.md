# Language kernel completion plan (excl. PKG-001)

Goal: complete LEX→SYN→MAC→RES→TYP/ROW/EFF→Core→EVAL/BND (+ MOD/KER/RSC/EDT/TST language parts) per `lang/specification.md`. Packages (graphics/math/Japanese) deferred.

## Status: **COMPLETE for pre-PKG**

Language kernel is complete enough for pre-PKG. Intentional deferrals listed below.

## Ordered work
1. CoreExpr::Var + env; if; n-ary fn/app — **done**
2. Surface elaborator: val/fn/let/seq → Core — **done** (+ perform/handle)
3. RES lexical resolve over binders — **done** (`resolve_language_source` + `BindingMap`)
4. MAC-001 hygienic macros (language); quarantine graphics macros — **done** (`expand_language`)
5. TYP-001 on Core (+ dynamic stub) — **done** (`typecheck_language_source`, `CoreType::Dynamic`)
6. ROW-001 fragment — **done** (closed + `OpenRecord` row vars; `Lacks` enforced in unify)
7. DAT interim: data/match — **done** (surface `data`/`match` → Core; **static exhaustiveness**)
8. EFF-001 deep handlers — **done** (deep one-shot resume; **EffectRow on Fun** via `infer_with_effects`)
9. BND letrec/var/set — **done** (`LetRec`, `LocalVar`/`Set` + cell alive escape check)
10. MOD-001 outer module + import — **done** (`elaborate_units` + **`load_module_tree`** sibling `.rpx`)
11. KER/RSC typed host ops — **done** (`+ - * < =` builtins; `MemoryFsHost` for `read-file`/`write-file`)
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
| **EFF-001** | Multi-shot / shallow choice handlers |
| **MOD-001** | Full ML functors / signatures |
| **KER-001** | Full typed Rust/foreign ABI (beyond BuiltinOp + EffectHost) |
| **DAT-001** | Multi-payload / parameterized ADTs |
| **BND-001** | Full escape analysis; typed `var` store |
| **RSC-001** | Richer resource catalog / path safety |
| **EDT-001** | SyntaxNodeId provenance through elaborate→eval (BindingId use-site map done) |
| **ROW-001** | Full unrestricted row tallying / multi-tail polymorphism |
