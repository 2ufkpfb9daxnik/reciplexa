# Language kernel completion plan (excl. PKG-001)

Goal: complete LEX→SYN→MAC→RES→TYP/ROW/EFF→Core→EVAL/BND (+ MOD/KER/RSC/EDT/TST language parts) per `lang/specification.md`. Packages (graphics/math/Japanese) deferred.

## Ordered work
1. CoreExpr::Var + env; if; n-ary fn/app — **done**
2. Surface elaborator: val/fn/let/seq → Core — **done** (+ perform/handle)
3. RES lexical resolve over binders — **done** (`resolve_language_source`)
4. MAC-001 hygienic macros (language); quarantine graphics macros — **done** (`expand_language`)
5. TYP-001 on Core (+ dynamic stub) — **done v0** (`typecheck_language_source`, `CoreType::Dynamic`)
6. ROW-001 fragment — **done v0** (closed-record unify + `CoreType::Lacks` stub; no open row vars)
7. DAT interim: data/match — **partial** (Core Variant/Match + eval; no surface `data`/`match` elaborate)
8. EFF-001 deep handlers — **shallow/one-shot v0** (`CoreExpr::Handle`; resume aborts handler, no deep continue)
9. BND letrec/var/set — **not started**
10. MOD-001 outer module + import — **done v0** (`elaborate_units` in-memory import skeleton)
11. KER/RSC typed host ops — **not started** (host `EffectHost` / document effects only)
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

## Remaining gaps (KER / RSC / EDT / DAT)
| ID | Gap |
|---|---|
| **DAT-001** | Surface `data` / `match` elaborate; richer ADTs; pattern exhaustiveness |
| **KER-001** | Typed Rust/foreign primitive ABI boundary (not just stringy `EffectHost`) |
| **RSC-001** | Resource / I/O / host op catalog wired through Core types + effects |
| **EDT-001** | Thread `BindingId` / `SyntaxNodeId` provenance through resolve→elaborate→eval |
| **EFF-001** | Deep handlers (continue body after resume); effect rows in infer |
| **ROW-001** | Open records / row variables; enforce `Lacks` |
| **BND-001** | `letrec` / `var` / `set!` on Core |
| **MOD-001** | Signatures, functors, filesystem sibling load, resolve across units |
