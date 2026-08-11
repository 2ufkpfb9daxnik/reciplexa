# 第II部 — gap / partial 優先度インデックス

各エージェントが埋め作業の優先度を付けるための在庫表。出典は `part2-conformance.md` の見出しステータスを、L4 見出しの `XXX-001` 機能ブロックに帰属させたもの。（下位見出しの多くは親機能 ID の下にぶら下がる。）

## 機能ブロック別 残件

| 機能 | gap | partial | 合計 |
|---|---:|---:|---:|
| `LEX` | 0 | 2 | 2 |
| `SYN` | 0 | 5 | 5 |
| `RES` | 0 | 2 | 2 |
| `DAT` | 0 | 2 | 2 |
| `EVAL` | 0 | 4 | 4 |
| `BND` | 0 | 2 | 2 |
| `MAC` | 0 | 3 | 3 |
| `TYP` | 0 | 22 | 22 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 0 | 2 | 2 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 0 | 2 | 2 |
| `PKG` | 0 | 8 | 8 |
| `KER` | 0 | 2 | 2 |
| `EDT` | 0 | 0 | 0 |
| `IR` | 0 | 1 | 1 |
| `ERR` | 0 | 3 | 3 |
| `MEM` | 0 | 1 | 1 |
| `TST` | 0 | 2 | 2 |
| **合計** | **0** | **67** | **67** |

（全体カウンタは `part2-conformance-stats.json` 参照。機能別内訳は L4 `XXX-001` ブロック帰属。）

### 合計の多い順（優先度の目安）
- `TYP`: gap 0 + partial 22 = **22**
- `PKG`: gap 0 + partial 8 = **8**
- `SYN`: gap 0 + partial 5 = **5**
- `EVAL`: gap 0 + partial 4 = **4**
- `MAC`: gap 0 + partial 3 = **3**
- `ERR`: gap 0 + partial 3 = **3**
- `LEX`: gap 0 + partial 2 = **2**
- `RES`: gap 0 + partial 2 = **2**
- `DAT`: gap 0 + partial 2 = **2**
- `BND`: gap 0 + partial 2 = **2**
- `ROW`: gap 0 + partial 2 = **2**
- `EFF`: gap 0 + partial 2 = **2**
- `RSC`: gap 0 + partial 2 = **2**
- `MOD`: gap 0 + partial 2 = **2**
- `KER`: gap 0 + partial 2 = **2**
- `TST`: gap 0 + partial 2 = **2**
- `IR`: gap 0 + partial 1 = **1**
- `MEM`: gap 0 + partial 1 = **1**



















---

## Top 30 `partial`（出現順・件数優先の大きい機能から）
1. **L2029** `TYP` — 13.6 `TYP-001` Gradual set-theoretic types
   - Bounded Dynamic/casts/evidence/EffectRow/ROW fragment; full set-theoretic solver deferred at TYP-ALG
2. **L2033** `TYP` — 概要・状態
   - Dynamic/unify/EffectRow/cast fragment; full set-theoretic algebra deferred
3. **L2037** `TYP` — 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure
   - Bounded Dynamic(S)+three-way use+CastEvidence+try/check-cast; foreign/guarantee remain deferred/meta
4. **L2089** `TYP` — `DD-TYP-DYN-009`: dynamic上限のwidening
   - Widen evidence for dynamic S→dynamic U when S<:U; deep provenance TBD
5. **L2113** `TYP` — `DD-TYP-DYN-013`: implicit cast failure
   - implicit Cast failure→EvalError; structured dynamic-type-error taxonomy still thin
6. **L2169** `TYP` — `DD-TYP-DYN-017`: evidence compositionと最適化
   - compose_evidence/simplify_evidence Identity absorption; widen-chain opt TBD
7. **L2173** `TYP` — `DD-TYP-DYN-018`: cast provenance
   - CastProvenance struct separate from evidence; full boundary IDs TBD
8. **L2185** `TYP` — `DD-TYP-DYN-021`: fixed-arity function cast
   - FunctionGuard fixed-arity plan; call-time arg/ret guard wrap interim
9. **L2189** `TYP` — `DD-TYP-DYN-022`: function引数の反変cast
   - FunctionGuard.arg_casts contravariant planning; runtime call wrap interim
10. **L2193** `TYP` — `DD-TYP-DYN-023`: function結果の共変cast
   - FunctionGuard.ret_cast covariant planning; runtime call wrap interim
11. **L2249** `TYP` — `DD-NAME-001`: 組込み型名と識別子の小文字規約
   - lowercase builtins via env; full namespace-collision warnings still thin
12. **L2253** `TYP` — `DD-NAME-002`: namespace間の同綴り衝突
   - lowercase builtins via env; full namespace-collision warnings still thin
13. **L2265** `TYP` — 適合試験
   - lang_kernel_suite TYP + cast_tests; full named TYP corpus still open
14. **L2485** `TYP` — `DD-TYP-EFF-005`: handlerとrunnerによるeffect縮小
   - handle removes op from residual; full handler return-clause typing interim
15. **L2525** `TYP` — `DD-TYP-FN-002`: fixed-arity function subtyping
   - Fun subtyping limited (arg contra / ret cov incomplete vs full DD)
16. **L2581** `TYP` — Row-polymorphic record
   - row-polymorphic records via OpenRecord; multi-tail deferred at ROW-001
17. **L2677** `TYP` — `DD-TYP-BIDI-004`: synthesis可能な式
   - literals/vars/app/record/ctors synthesize; empty-collection synth gaps remain
18. **L2681** `TYP` — `DD-TYP-BIDI-005`: checkingを優先する式
   - annotated bindings check; empty collections/intersection check gaps remain
19. **L2729** `TYP` — `DD-TYP-FRAG-003`: C — 注釈を要求し得るfragment
   - AnnotationRequired class reserved; incompleteness paths still light
20. **L2793** `TYP` — 適合試験
   - lang_kernel_suite TYP + cast_tests; full named TYP-ALG corpus open
21. **L2825** `TYP` — Function subtyping
   - basic Fun subtyping only; full DD suite incomplete
22. **L2865** `TYP` — Bidirectional checking
   - Fun/Record annotation checking present; full BIDI corpus incomplete
23. **L3657** `PKG` — 13.10 `PKG-001` パッケージmanifest・依存解決・ワークスペース・リソース
   - Slice A–C: local packages + path-dep lock + math/japanese/graphics stubs + workspace.rpxm parse; registry deferred
24. **L3701** `PKG` — 2. パッケージmanifest
   - parse_rpxm + PackageManifest; remaining schema depth vs full PKG-001
25. **L3713** `PKG` — 2.3 制限付きRPX形式
   - restricted sexp tokenize in rpxm.rs; not every static schema leaf
26. **L3717** `PKG` — 2.4 静的schema
   - restricted sexp + known-field gate; residual static schema leaves open
27. **L3725** `PKG` — 3. Manifestの基本構文
   - minimal/explicit/parensed manifest forms parsed; residual schema depth open
28. **L3745** `PKG` — 4. パッケージ名
   - package name parsed as identity; full naming-rule matrix still light
29. **L3761** `PKG` — 5. パッケージversion
   - version field parsed; full semver algebra deferred
30. **L3825** `PKG` — 7.3 内部モジュール
   - non-public cross-package import rejected; intra-package graph incomplete


---

全文の残 gap 一覧: [`part2-remaining-gaps.md`](part2-remaining-gaps.md)（0 件）。

