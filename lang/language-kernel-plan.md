# Language kernel completion plan (excl. PKG-001)

Goal: complete LEX→SYN→MAC→RES→TYP/ROW/EFF→Core→EVAL/BND (+ MOD/KER/RSC/EDT/TST language parts) per `lang/specification.md`. Packages (graphics/math/Japanese) deferred.

## Spec alignment progress (2026-08-11)

### Fixed this round
| Area | Spec | Status |
|---|---|---|
| MAC-001 `$params ...+` / template `...`; reject legacy no-`->` | MAC-001 | **done** (`lang_macro`; reserved via SYN table) |
| SYN §7 numeric: radix / `_` / scientific / reject leading zeros | SYN §7 | **done** (`number_lit` + lexer; unit suffix stays Number+Ident) |
| SYN §3 identifiers: NFC, kebab, no `_` in binders, trailing `?!` | SYN §3–5 | **done** (`ident.rs` + lexer/resolve/elaborate) |
| `record-update` / `record-extend` | SYN §15.5 | **done** (CoreExpr + elaborate/check/eval) |
| KER ops `/ > <= >= !=` in typecheck + resolve keywords | SYN §5 / KER | **done** (`check.rs` env + `is_language_keyword`) |
| Surface `(type name Ty)` / `(dynamic)` / `(union …)` stub | SYN §16 | **done** (`DataEnv.type_aliases`, `CoreType::Union`) |
| DAT match `_` / `bind` / nested payload patterns | DAT §14–16 | **done** (`CorePattern`; elaborate + eval + check) |
| EFF ambient `(log …)` / `(random)` | EFF-001 DD-EFF-013 | **done** (→ `Perform`; Core `handle` still primary) |
| `local` / `rec` declaration groups | SYN §13.4–13.5 | **done** (local→lets; rec→`LetRec`) |
| MOD `import` path / `as` / flat `only` | MOD §6 | **done** (parse + link skeleton; qualified refs deferred) |
| String literals: no backslash escapes | SYN §8.1 | **done** (lexer + elaborate/effect/lower read path) |
| EFF Surface `with` / first-class `handler` | DD-EFF-011/012 | **done** (`HandlerValue` + `With`; interim op+fn shape) |
| `"""` multi-quote string delimiters | SYN §8.2–8.3 | **done** (lexer + `decode_string_literal` / encode) |
| DAT literal / tuple / multi-payload patterns | DAT §14–17 | **done** (`Lit` / `Tuple`; arity≥2 ctors) |
| MOD qualified refs + `only x as y` | MOD §6.2–6.6 | **done** (`alias/export`, `ImportItem.rename`) |
| Top-level `rec`; `local` `var` / nested `rec` | SYN §13.4–13.5 | **done** |
| Writers inventing `\n`/`\"` escapes | SYN §8.1 | **done** (`encode_string_literal` in GUI/macro/document/lower) |
| DAT record patterns (§18) | DAT §18 | **done** (`CorePattern::Record`; partial required fields; elaborate/eval/check/resolve) |
| `/` path-only vs division | SYN §4–5 | **done** (`/` not ident_continue; `coalesce_slash_paths`; `//` kept) |
| Mem IR `record-update` / `record-extend` | MEM | **done** (Construct tags + exec merge; lower no longer returns base unchanged) |
| DAT parameterized ADTs (light) | DAT-001 §1.2 | **done** (parse `((a type)…)` + `DataEnv.type_params`; match/eval work; **type instantiation / ctor typing still Dynamic**) |

### Remaining gaps (pre-PKG / language)
| Gap | Notes |
|---|---|
| EFF multi-shot / shallow handlers | intentional deferral |
| EFF handler `return` clauses / full `Handler<L,A,B,H>` typing | interim op+fn handlers only |
| ROW unrestricted / multi-tail | ROW-001 deferral |
| Full MOD functors / signatures | MOD-001 deferral |
| DAT ADT type instantiation | params stored; no `option number` type apps / polymorphic ctor check yet |
| Unit suffixes as typed constructors | interim Number+Ident (`40mm`); NumberWithUnit / package apply deferred with PKG |
| `CoreType::Union` / local type-alias registration | union unifies like Dynamic; local `(type …)` parse-checked only |
| Record pattern optional fields | §18.5 — use `option` match on `(field …)` (not direct record decomp) |

## Status: **COMPLETE for pre-PKG** (with gaps above tracked)

Language kernel is complete enough for pre-PKG. Intentional deferrals listed below.

## Ordered work
1. CoreExpr::Var + env; if; n-ary fn/app — **done**
2. Surface elaborator: val/fn/let/seq → Core — **done** (+ perform/handle/handler/with; ambient apps; local/rec; type aliases; record-update/extend)
3. RES lexical resolve over binders — **done** (`resolve_language_source` + `BindingMap` + NFC)
4. MAC-001 hygienic macros (language); quarantine graphics macros — **done** (`expand_language`; `...+` / reject legacy)
5. TYP-001 on Core (+ dynamic stub) — **done** (`typecheck_language_source`, `CoreType::Dynamic` / `Union`)
6. ROW-001 fragment — **done** (closed + `OpenRecord` row vars; `Lacks` enforced in unify)
7. DAT interim: data/match — **done** (surface `data`/`match` → Core; `_`/`bind`/nested/literal/tuple/multi-payload/record; params skeleton; **static exhaustiveness**)
8. EFF-001 deep handlers — **done** (deep one-shot resume; ambient apps; first-class `handler`/`with`; **EffectRow on Fun** via `infer_with_effects`)
9. BND letrec/var/set — **done** (`LetRec`, `LocalVar`/`Set` + cell alive escape check; top-level `rec`; `local` val/var/rec)
10. MOD-001 outer module + import — **done** (`elaborate_units` + **`load_module_tree`** + import `as`/path/`only`/rename + qualified refs; path `/` coalesced)
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
| **EFF-001** | Multi-shot / shallow choice handlers; full return-clause handler typing |
| **MOD-001** | Full ML functors / signatures |
| **KER-001** | Full typed Rust/foreign ABI (beyond BuiltinOp + EffectHost) |
| **DAT-001** | Full polymorphic ADT typing / type apps (params parse+env done; record patterns done) |
| **BND-001** | Full escape analysis; typed `var` store |
| **RSC-001** | Richer resource catalog / path safety |
| **EDT-001** | SyntaxNodeId provenance through elaborate→eval (BindingId use-site map done) |
| **ROW-001** | Full unrestricted row tallying / multi-tail polymorphism |
