# Language kernel completion plan (excl. PKG-001)

Goal: complete LEX→SYN→MAC→RES→TYP/ROW/EFF→Core→EVAL/BND (+ MOD/KER/RSC/EDT/TST language parts) per `lang/specification.md`. Packages (graphics/math/Japanese) deferred.

## Ordered work
1. CoreExpr::Var + env; if; n-ary fn/app — **done**
2. Surface elaborator: val/fn/let/seq → Core — **done** (+ perform/handle)
3. RES lexical resolve over binders — **done** (`resolve_language_source`)
4. MAC-001 hygienic macros (language); quarantine graphics macros — **done** (`expand_language`)
5. TYP-001 on Core (+ dynamic stub) — **done v0** (`typecheck_language_source`, `CoreType::Dynamic`)
6. ROW-001 fragment — **done v0** (closed-record unify + `CoreType::Lacks` stub; no open row vars)
7. DAT interim: data/match — **done v0** (surface `data`/`match` → Core Variant/Match; tag + optional single binder)
8. EFF-001 deep handlers — **done v0** (deep one-shot resume; body continues after `resume`)
9. BND letrec/var/set — **done v0** (`LetRec`, `LocalVar`/`Set` + cell alive escape check)
10. MOD-001 outer module + import — **done v0** (`elaborate_units` in-memory import skeleton)
11. KER/RSC typed host ops — **done v0** (`+ - * < =` builtins; `MemoryFsHost` for `read-file`/`write-file`)
12. EDT thread BindingId/SyntaxNodeId; retire prototype "language" tests — **partial** (document surface docs + `lang_kernel_suite`; BindingId not threaded)
13. GUI text-move sync tests (expanded vs authoring) — **prior work**; not revisited here

## Gate
```
CARGO_TARGET_DIR=d:\reciplexa\target TEMP/TMP=d:\reciplexa\.tmp
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p reciplexa-gui
```
**Last gate: green** (fmt / clippy -D warnings / test --workspace / check gui).

## Remaining gaps
| ID | Gap |
|---|---|
| **DAT-001** | Richer ADTs (multi-payload, params); pattern exhaustiveness |
| **KER-001** | Full typed Rust/foreign ABI (beyond BuiltinOp + EffectHost) |
| **RSC-001** | Effect rows in Fun types; richer resource catalog / path safety |
| **EDT-001** | Thread `BindingId` / `SyntaxNodeId` provenance through resolve→elaborate→eval |
| **EFF-001** | Multi-shot / shallow choice; effect rows in infer beyond stubs |
| **ROW-001** | Open records / row variables; enforce `Lacks` |
| **BND-001** | Full escape analysis; typed `var` store |
| **MOD-001** | Signatures, functors, filesystem sibling load, resolve across units |
| **PKG-001** | Deferred (graphics/math/Japanese packages) |
