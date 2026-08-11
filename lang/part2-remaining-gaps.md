# Part II — remaining `gap` / `partial` inventory

Generated from `part2-conformance.md` + `part2-conformance-stats.json`.
Grouped by parent L4 `XXX-001` feature block (file order). `LINE` = line in `part2-conformance.md`.

## Current stats

| metric | count |
|---|---:|
| `total` | 1589 |
| `unchecked` | 0 |
| `ok` | 517 |
| `partial` | 416 |
| `gap` | 0 |
| `deferred` | 518 |
| `meta` | 138 |

Attributed checklist gaps: **0** (matches `stats.gap`).

## Gaps / partials per feature

| feature | gap | partial | gap+partial |
|---|---:|---:|---:|
| `LEX` | 0 | 3 | 3 |
| `SYN` | 0 | 22 | 22 |
| `RES` | 0 | 2 | 2 |
| `DAT` | 0 | 28 | 28 |
| `EVAL` | 0 | 4 | 4 |
| `BND` | 0 | 19 | 19 |
| `MAC` | 0 | 9 | 9 |
| `TYP` | 0 | 90 | 90 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 0 | 8 | 8 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 0 | 16 | 16 |
| `PKG` | 0 | 37 | 37 |
| `KER` | 0 | 2 | 2 |
| `EDT` | 0 | 19 | 19 |
| `IR` | 0 | 6 | 6 |
| `ERR` | 0 | 20 | 20 |
| `MEM` | 0 | 100 | 100 |
| `TST` | 0 | 27 | 27 |
| **total** | **0** | **416** | **416** |

## Remaining gaps (compact)

_None — `gap` count is 0._

## Partial inventory (by feature, largest first)

### `MEM` — 100 partial

- **L5582** — 13.14 `MEM-001` Perceusメモリ管理・スコープ付きリソース・継続・メモリ予算
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path; eval uses Rc not Perceus by default
- **L5618** — 1. メモリとResourceの分離
  - notes: policy: values vs resources; RSC/bracket separation incomplete
- **L5622** — 1.1 通常値
  - notes: policy: values vs resources; RSC/bracket separation incomplete
- **L5626** — 1.2 外部Resource
  - notes: policy: values vs resources; RSC/bracket separation incomplete
- **L5630** — 1.3 基本原則
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5634** — 2. Perceusによる自動メモリ管理
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5638** — 2.1 利用者から見える意味
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5642** — 2.2 回収時点
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5646** — 2.3 物理identity
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5650** — 3. Compilation pipeline
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5654** — 3.1 適用順序
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5658** — 3.2 Surface所有権注釈
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5662** — 3.3 Trusted boundary
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5666** — 4. 所有権Core IR
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5670** — 4.1 必須のCore要素
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5674** — 4.2 評価順序
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5678** — 5. dup
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5682** — 5.1 意味
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5686** — 5.2 挿入条件
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5690** — 5.3 利用者からの不可視性
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5694** — 6. drop
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5698** — 6.1 意味
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5702** — 6.2 最終使用位置
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5706** — 6.3 制御フロー
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5710** — 7. 分岐とJoin point
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5714** — 7.1 排他的分岐
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5718** — 7.2 Join時の整合
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5722** — 8. Reuse
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5726** — 8.1 位置付け
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5730** — 8.2 一意性
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5734** — 8.3 Reuse不成立
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5738** — 8.4 非保証
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5742** — 8.5 Reuse禁止値
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5746** — 9. Closure環境
  - notes: MakeClosure in mem lower; not default eval memory path
- **L5750** — 9.1 表現
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5754** — 9.2 生成
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5758** — 9.3 解放
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5766** — 9.5 Scoped値のcapture
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5770** — 10. var
  - notes: LocalVar escape check in eval; full Perceus-var integration incomplete
- **L5774** — 10.1 既存意味論
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5778** — 10.2 物理表現
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5802** — 12. 再帰型と循環値
  - notes: spec forbids heap cycles in v1; no cycle detector beyond policy
- **L5806** — 12.1 再帰data型
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5810** — 12.2 循環する実行時値
  - notes: spec forbids heap cycles in v1; no cycle detector beyond policy
- **L5814** — 12.3 論理的なID参照
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5818** — 13. Continuation
  - notes: Resume/DiscardCont/Raise in mem IR+exec; not full continuation model
- **L5822** — 13.1 表現
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5826** — 13.2 Capture
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5830** — 13.3 Resume
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5834** — 13.4 Discard
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5838** — 13.5 One-shot
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5842** — 13.6 Escape
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5850** — 14.1 明示的なunwind
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5854** — 14.2 Dropの欠落禁止
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5858** — 14.3 Handler節
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5866** — 15.1 隠れたscope
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5874** — 15.3 Escape禁止
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5882** — 15.5 独立した結果値
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5890** — 16.1 With-style API
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5894** — 16.2 低水準API
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5898** — 16.3 Release責任
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5902** — 16.4 自発的な無効化
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5938** — 19.1 Ownership metadata
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5942** — 19.2 Borrowed契約違反
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5946** — 19.3 Owned移送
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5954** — 20.1 v1の範囲
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5958** — 20.2 共有値
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5970** — 21.1 構造共有
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5974** — 21.2 解放
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5978** — 21.3 一意性
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5982** — 21.4 観測不能
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5986** — 22. メモリ割当とEffect
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5994** — 22.2 Perceus操作
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L5998** — 22.3 理由
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6006** — 23.1 適用単位
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6010** — 23.2 Job増分方式
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6014** — 23.3 Commit時の移管
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6022** — 24.1 型付きFailure
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6026** — 24.2 処理
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6030** — 24.3 予約領域
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6038** — 25.1 事前検査
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6042** — 25.2 分類
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6050** — 26.1 課金対象
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6054** — 26.2 超過時
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6062** — 27.1 有効なSnapshot handle
  - notes: DocumentSnapshot holds while handle live; retention-limit refuse 依存待ち(第III部|第V部)
- **L6066** — 27.2 保持policy
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6070** — 27.3 履歴破棄
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6082** — 28.2 分類
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6086** — 28.3 Cleanup
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6094** — 29.1 Wraparound禁止
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6098** — 29.2 実装
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6102** — 29.3 分類
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6106** — 29.4 その他の内部不変条件
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6126** — 31. メモリ観測API
  - notes: RC/dup/drop not exposed to RPX (§31.1); budget/peak observation API 依存待ち(第III部)|OPEN-MEM-PROF-001
- **L6130** — 31.1 非公開情報
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6134** — 31.2 許可される情報
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6138** — 31.3 安定性
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6142** — 32. GUI状態
  - notes: GUI uses explicit DocumentSnapshot (no implicit cell); full model OPEN-GUI-STATE-001
- **L6150** — 32.2 推奨モデル
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **L6154** — 33. 適合試験
  - notes: phase6_mem / mem tests exist; full MEM-001 suite incomplete

### `TYP` — 90 partial

- **L2022** — 13.6 `TYP-001` Gradual set-theoretic types
  - notes: Dynamic+unify+EffectRow+ROW fragment; set-theoretic/casts gap
- **L2026** — 概要・状態
  - notes: Dynamic/unify/EffectRow fragment; full set-theoretic deferred
- **L2030** — 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure
  - notes: Bounded Dynamic(S); three-way use; CastEvidence; try/check-cast; foreign/guarantee deferred/meta
- **L2038** — `DD-TYP-DYN-001`: static型とgradual型の分離
  - notes: static vs gradual: CoreType::Dynamic(bound) + Any top; no free static↔gradual mix
- **L2042** — `DD-TYP-DYN-002`: `dynamic S`の意味
  - notes: `dynamic S` elaborates to Dynamic(bound); dynamic never≃never; nested dynamic collapses
- **L2046** — `DD-TYP-DYN-003`: static top型`any`
  - notes: CoreType::Any + S<:any unify; any does not accept implicit cast
- **L2050** — `DD-TYP-DYN-004`: `never`およびdynamicの正規形
  - notes: Never + dynamic never normalize; CoreType::dynamic_bound
- **L2054** — `DD-TYP-DYN-005`: dynamic値をstatic型として使用する三段階判定
  - notes: judge_dynamic_use three-way; coerce_to_static / insert_implicit_casts
- **L2058** — 1. 上限全体が要求型へ含まれる場合
  - notes: FullyIncluded when S<:T → Identity (no runtime check)
- **L2062** — 2. 上限と要求型が互いに素である場合
  - notes: Disjoint when intersect(S,T)≃never → static reject in coerce/plan
- **L2066** — 3. 一部だけ重なる場合
  - notes: PartialOverlap → Cast evidence + success intersect(S,T)
- **L2070** — `DD-TYP-DYN-006`: cast成功後の型
  - notes: cast_success_type = intersect(S,T); Cast/TryCast/CheckCast return intersect
- **L2074** — `DD-TYP-DYN-007`: occurrence typingとの関係
  - notes: occurrence refine on Dynamic: then intersect / else dynamic(diff)
- **L2078** — `DD-TYP-DYN-008`: static値からdynamic値への導入
  - notes: Widen when T<:S; plan_cast rejects T</:S for to-dynamic
- **L2082** — `DD-TYP-DYN-009`: dynamic上限のwidening
  - notes: Widen evidence for dynamic S→dynamic U when S<:U; deep provenance TBD
- **L2102** — `DD-TYP-DYN-012`: runtime-checkableな型
  - notes: is_runtime_checkable for primitives/record/variant/fun; opaque/capability reject
- **L2106** — `DD-TYP-DYN-013`: implicit cast failure
  - notes: implicit Cast failure → EvalError dynamic cast failed; structured dynamic-type-error TBD
- **L2110** — `DD-TYP-DYN-014`: 明示的safe cast
  - notes: try-cast/check-cast → Option/Result of intersect(S,T)
- **L2114** — `DD-TYP-DYN-015`: cast evidence
  - notes: CastEvidence algebra + plan_cast_evidence + CastProvenance stub
- **L2118** — `Identity`
  - notes: CastEvidence::Identity
- **L2122** — `Widen`
  - notes: CastEvidence::Widen
- **L2126** — `TagCheck`
  - notes: CastEvidence::TagCheck
- **L2130** — `UnionCheck`
  - notes: CastEvidence::UnionCheck
- **L2134** — `IntersectionCheck`
  - notes: CastEvidence::IntersectionCheck
- **L2138** — `RecordCheck`
  - notes: CastEvidence::RecordCheck + field-type runtime checks
- **L2142** — `VariantCheck`
  - notes: CastEvidence::VariantCheck + payload runtime checks
- **L2146** — `FunctionGuard`
  - notes: CastEvidence::FunctionGuard {arity,arg_casts,ret_cast}
- **L2150** — `NominalCheck`
  - notes: CastEvidence::NominalCheck
- **L2154** — `Compose`
  - notes: CastEvidence::Compose + simplify_evidence
- **L2158** — `DD-TYP-DYN-016`: cast evidenceの純粋性
  - notes: evidence eval pure (inspect/wrap only); DD-TYP-DYN-016
- **L2162** — `DD-TYP-DYN-017`: evidence compositionと最適化
  - notes: compose_evidence / simplify_evidence Identity absorption; widen chain TBD
- **L2166** — `DD-TYP-DYN-018`: cast provenance
  - notes: CastProvenance struct separate from evidence; full boundary IDs TBD
- **L2170** — `DD-TYP-DYN-019`: recordおよびvariant cast
  - notes: RecordCheck/VariantCheck plan + deepened runtime field/payload checks
- **L2178** — `DD-TYP-DYN-021`: fixed-arity function cast
  - notes: FunctionGuard fixed-arity plan; call-time arg/ret guards stub
- **L2182** — `DD-TYP-DYN-022`: function引数の反変cast
  - notes: FunctionGuard.arg_casts contravariant planning; runtime call wrap TBD
- **L2186** — `DD-TYP-DYN-023`: function結果の共変cast
  - notes: FunctionGuard.ret_cast covariant planning; runtime call wrap TBD
- **L2190** — `DD-TYP-DYN-024`: function arityの制限
  - notes: arity equality required in FunctionGuard; varargs deferred
- **L2194** — `DD-TYP-DYN-025`: effectful function cast
  - notes: effect_subrow Es⊑Et in is_subtype/fun intersect; gradual effect cast out of v1
- **L2242** — `DD-NAME-001`: 組込み型名と識別子の小文字規約
  - notes: lowercase builtins via env; no full namespace-collision warnings
- **L2246** — `DD-NAME-002`: namespace間の同綴り衝突
  - notes: lowercase builtins via env; no full namespace-collision warnings
- **L2250** — dynamic typingのCore構文
  - notes: CoreExpr::Cast/TryCast/CheckCast in core
- **L2254** — dynamic typingの終端状態
  - notes: Cast/TryCast/CheckCast + NumericPromote terminal paths in eval
- **L2258** — 適合試験
  - notes: lang_kernel_suite TYP-001 + type aliases; not full TYP suite
- **L2262** — static injection
  - notes: Dynamic present; not full cast suite
- **L2266** — invalid static injection
  - notes: Dynamic present; not full cast suite
- **L2270** — dynamic widening
  - notes: Dynamic present; not full cast suite
- **L2274** — safe static use
  - notes: FullyIncluded / Identity when S<:T
- **L2278** — partial overlap
  - notes: PartialOverlap + cast evidence
- **L2282** — disjoint use
  - notes: Disjoint → static reject in coerce/plan
- **L2286** — cast precision
  - notes: cast_success_type = intersect(S,T)
- **L2290** — `any`と`dynamic any`
  - notes: Dynamic present; not full cast suite
- **L2298** — implicit cast failure
  - notes: implicit cast failure → EvalError dynamic cast failed
- **L2302** — explicit safe cast
  - notes: try-cast/check-cast Option/Result paths
- **L2306** — fixed-arity function cast
  - notes: FunctionGuard evidence for fixed-arity fun casts
- **L2310** — function result cast
  - notes: covariant ret_cast in FunctionGuard
- **L2314** — effect-compatible function cast
  - notes: effect_subrow compatible function cast planning
- **L2318** — effect-incompatible function cast
  - notes: effect-incompatible fun rejected via effect_subrow
- **L2370** — 13.6.2 `TYP-ALG-001` Algorithmic型検査、semantic subtypingの判定範囲、型推論
  - notes: decide_subtype three-valued + is_subtype approx; no full semantic solver
- **L2394** — `DD-TYP-ALG-003`: 診断分類
  - notes: TypeDiagClass + classify_decide; AnnotationRequired/ResourceLimit paths thin
- **L2402** — `annotation-required`
  - notes: TypeDiagClass::AnnotationRequired present; emitter path not fully wired
- **L2410** — `checker-resource-limit`
  - notes: TypeDiagClass::CheckerResourceLimit reserved; no step-budget emitter yet
- **L2414** — `unsupported-language-feature`
  - notes: TypeDiagClass::UnsupportedLanguageFeature reserved; limited use sites
- **L2426** — `DD-TYP-ALG-006`: 決定的なsolver budget
  - notes: checker terminates on fragment; explicit solver step budget absent
- **L2474** — `DD-TYP-EFF-004`: effect-row polymorphism
  - notes: EffectRow present; full effect-row polymorphism / quantify deferred
- **L2478** — `DD-TYP-EFF-005`: handlerとrunnerによるeffect縮小
  - notes: handle removes op from residual; full handler typing interim
- **L2498** — `DD-TYP-BOOL-002`: Surface negationの制限
  - notes: surface Not parses; unrestricted negation / full solve still stub
- **L2510** — Function型
  - notes: fixed-arity Fun in ty.rs; intersection types / coherence deferred
- **L2518** — `DD-TYP-FN-002`: fixed-arity function subtyping
  - notes: Fun subtyping limited (arg contra / ret cov incomplete vs full DD)
- **L2574** — Row-polymorphic record
  - notes: row-polymorphic records via OpenRecord; multi-tail deferred
- **L2646** — Bidirectional type checking
  - notes: check.rs mostly infer; limited check-via-unify mode
- **L2650** — `DD-TYP-BIDI-001`: bidirectional typing
  - notes: infer-primary bidirectional fragment (check.rs)
- **L2658** — Checking
  - notes: checking via unify against expected; not full check mode
- **L2662** — `DD-TYP-BIDI-002`: 型注釈付きbinding
  - notes: check.rs infer-primary; some check via unify
- **L2666** — `DD-TYP-BIDI-003`: 無注釈binding
  - notes: check.rs infer-primary; some check via unify
- **L2670** — `DD-TYP-BIDI-004`: synthesis可能な式
  - notes: check.rs infer-primary; some check via unify
- **L2674** — `DD-TYP-BIDI-005`: checkingを優先する式
  - notes: check.rs infer-primary; some check via unify
- **L2678** — `DD-TYP-BIDI-006`: `if`と`match`
  - notes: check.rs infer-primary; some check via unify
- **L2682** — `DD-TYP-BIDI-007`: subtyping、promotion、dynamic cast
  - notes: check.rs infer-primary; some check via unify
- **L2690** — 明示的多相型
  - notes: CoreType::Forall parses; prenex instantiation deferred
- **L2694** — `DD-TYP-POLY-001`: 明示的`forall`
  - notes: forall elaborates to CoreType::Forall; stub unify / instantiate
- **L2706** — `DD-TYP-POLY-004`: rank-1／prenex制限
  - notes: surface forall only; rank-1 prenex instantiation not enforced
- **L2722** — `DD-TYP-FRAG-003`: C — 注釈を要求し得るfragment
  - notes: AnnotationRequired class reserved for incompleteness paths
- **L2762** — Annotation required
  - notes: AnnotationRequired class reserved; thin emitter
- **L2770** — Resource limit
  - notes: CheckerResourceLimit class reserved
- **L2786** — 適合試験
  - notes: lang_kernel_suite TYP + cast tests; not full TYP-ALG corpus
- **L2794** — 診断分類
  - notes: TypeDiagClass taxonomy present; full diagnostic emitter incomplete
- **L2806** — EffectRow polymorphism
  - notes: EffectRow polymorphism suite thin vs DD-TYP-EFF-004
- **L2818** — Function subtyping
  - notes: basic Fun subtyping only; full DD suite incomplete
- **L2858** — Bidirectional checking
  - notes: basic Fun/Record checking; full BIDI suite incomplete
- **L2862** — Explicit `forall`
  - notes: explicit forall surface; instantiation suite thin

### `PKG` — 37 partial

- **L3650** — 13.10 `PKG-001` パッケージmanifest・依存解決・ワークスペース・リソース
  - notes: Slice A+B: local packages + path-dep aliases + rpx.lock path sources + math/japanese stubs + .rpi stubs; registry/workspace still deferred
- **L3694** — 2. パッケージmanifest
  - notes: parse_rpxm + PackageManifest JSON; schema incomplete vs PKG-001
- **L3698** — 2.1 ファイル名
  - notes: package.rpxm name convention in rpxm.rs; not full PKG path rules
- **L3702** — 2.2 Package root
  - notes: package.rpxm directory is package root via LocalPackageIndex::discover
- **L3706** — 2.3 制限付きRPX形式
  - notes: restricted sexp via tokenize in rpxm.rs; not full static schema
- **L3710** — 2.4 静的schema
  - notes: restricted sexp via tokenize in rpxm.rs; not full static schema
- **L3714** — 2.5 未知field
  - notes: format-version enforced; unknown fields still skipped (strict unknown-field reject deferred)
- **L3718** — 3. Manifestの基本構文
  - notes: restricted sexp via tokenize in rpxm.rs; not full static schema
- **L3722** — 3.1 最小形
  - notes: restricted sexp via tokenize in rpxm.rs; not full static schema
- **L3726** — 3.2 明示形
  - notes: restricted sexp via tokenize in rpxm.rs; not full static schema
- **L3730** — 3.3 Fieldの括弧
  - notes: flat fields + parenthesized public-modules/entry-points/dependencies parsed; unknown fields still ignored
- **L3734** — 3.4 Format version
  - notes: format-version 1 accepted; other versions rejected; unknown fields still gap
- **L3738** — 4. パッケージ名
  - notes: name/version/entry/dep/target fields parsed; naming rules incomplete
- **L3742** — 4.1 基本規則
  - notes: name/version/entry/dep/target fields parsed; naming rules incomplete
- **L3746** — 4.2 用途
  - notes: package name used as identity key in LocalPackageIndex / import first segment
- **L3754** — 5. パッケージversion
  - notes: name/version/entry/dep/target fields parsed; naming rules incomplete
- **L3758** — 5.1 基本形式
  - notes: name/version/entry/dep/target fields parsed; naming rules incomplete
- **L3778** — 6. Source rootとinterface root
  - notes: source-root parsed (default src); module files loaded via PackageManifest::module_source_path
- **L3818** — 7.3 内部モジュール
  - notes: non-public cross-package import rejected; intra-package graph incomplete
- **L3834** — 8. 実行エントリ
  - notes: entry-points parsed; legacy entry still supported; library packages may omit entry
- **L3842** — 8.2 Entry module
  - notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred
- **L3854** — 8.5 一module一entry
  - notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred
- **L3878** — 9.4 外部dependency
  - notes: scripts may import local std packages without consumer manifest; non-std deps still need manifest (Slice B)
- **L3886** — 10. 依存宣言
  - notes: DD-001 (dependencies (alias package name version path)) parsed; Phase 10 (dep …) kept
- **L3910** — 11. Version constraint
  - notes: DependencySpec path + version_req string; no full constraint solver
- **L3914** — 11.1 完全一致
  - notes: DependencySpec path + version_req string; no full constraint solver
- **L3934** — 12. Dependency source
  - notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred
- **L3998** — 15. 依存解決
  - notes: resolve_packages deterministic sort + lockfile stub
- **L4002** — 15.1 Manifestの役割
  - notes: resolve_packages deterministic sort + lockfile stub
- **L4022** — 15.6 決定性
  - notes: resolve_packages deterministic sort + lockfile stub
- **L4046** — 17. Lockfile
  - notes: resolve_packages deterministic sort + lockfile stub
- **L4050** — 17.1 ファイル名
  - notes: resolve_packages deterministic sort + lockfile stub
- **L4078** — 18. Lockfileの内容
  - notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred
- **L4090** — 18.3 Local path package
  - notes: local packages on disk under packages/; lockfile path source still stub
- **L4130** — 20.2 Manifest
  - notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred
- **L4134** — 20.3 基本構文
  - notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred
- **L4162** — 21.1 共通lockfile
  - notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred

### `DAT` — 28 partial

- **L538** — 13.2.3 `DAT-001` 代数的データ型・constructor・pattern・match
  - notes: kernel data/match done; poly typing / ctor types deferred (plan)
- **L578** — 1.2 Parameterを持つdata型
  - notes: elaborate.rs parses ((a type)…); no type-app instantiation
- **L590** — 2. `data`宣言が生成するbinding
  - notes: ctors registered; parent/sealed/ctor-types incomplete vs §2
- **L594** — 2.1 親type constructor
  - notes: DataEnv ctors; parent type / sealed set incomplete vs spec
- **L598** — 2.2 Value constructor
  - notes: DataEnv ctors; parent type / sealed set incomplete vs spec
- **L606** — 2.4 Sealed constructor集合
  - notes: DataEnv ctors; parent type / sealed set incomplete vs spec
- **L614** — 3. 名前とnamespace
  - notes: value-ns ctors via DataEnv; no ctor-type ns / full RES identity
- **L618** — 3.1 型namespaceと値namespace
  - notes: value-ns ctors via DataEnv; distinct type-ns ctor types absent
- **L622** — 3.2 型名とconstructor名の同名禁止
  - notes: name clash checks limited; no full dual-namespace enforcement
- **L626** — 3.3 Constructor名の重複
  - notes: value-ns ctors via DataEnv; no ctor-type ns / full RES identity
- **L630** — 3.4 Constructor identity
  - notes: value-ns ctors via DataEnv; no ctor-type ns / full RES identity
- **L710** — 8. Variance
  - notes: per-param cov/contra/invar/phantom inferred; variance subtyping lattice incomplete
- **L730** — 8.5 Phantom parameter
  - notes: phantom recorded in DataEnv.type_variances; unused-param warning channel deferred
- **L750** — 10. 相互再帰data型
  - notes: runtime rec data ok; positivity/kind mixing checks absent
- **L754** — 10.1 `rec` group
  - notes: runtime rec data ok; positivity/kind mixing checks absent
- **L758** — 10.2 Group内の可視性
  - notes: runtime rec data ok; positivity/kind mixing checks absent
- **L762** — 10.3 宣言kindの混在禁止
  - notes: runtime rec data ok; positivity/kind mixing checks absent
- **L906** — 17.3 Binding型
  - notes: pattern binders elaborate; static payload typing mostly Dynamic (DAT §17.3)
- **L938** — 18.6 Unknown row field
  - notes: unknown open-row fields bind as Dynamic in check.rs (DAT §18.6)
- **L942** — 19. Pattern typing
  - notes: check.rs binds pattern vars mostly as Dynamic
- **L946** — 19.1 三つの結果
  - notes: check.rs binds pattern vars mostly as Dynamic
- **L950** — 19.2 Constructor pattern
  - notes: DAT interim: elaborate.rs / expr.rs / eval.rs / check.rs
- **L962** — 19.5 Disjoint pattern
  - notes: check.rs binds pattern vars mostly as Dynamic
- **L978** — 20.3 結果型
  - notes: arm result types unify; finer GADT/refine still Dynamic-heavy (DAT §20.3)
- **L990** — 21. 網羅性・公開・適合試験
  - notes: lang_kernel_suite ADT-01..06/09/10; ADT-07/08 poly generalization still open
- **L1010** — 21.5 Sealed data
  - notes: ctors registered sealed in DataEnv; no abstract export yet
- **L1050** — 21.15 適合試験 ADT-07
  - notes: ADT-07 forall e. result<int,e> generalization not yet in checker
- **L1054** — 21.16 不適合試験 ADT-08
  - notes: ADT-08 var ungeneralized error-type diagnostic not implemented

### `TST` — 27 partial

- **L6198** — 13.15 `TST-001` Tests and conformance
  - notes: lang_kernel_suite maps TEST-STA-007/008, TEST-DYN-003, TEST-INT-002 + casts/bytes/any/variance/failure/forward/macro/.rpi
- **L6214** — テスト原則
  - notes: conformance.rs + lang_kernel_suite TEST-* IDs; full artifact trace incomplete
- **L6228** — 14.1 全体EBNF（未完成）
  - notes: skeleton EBNF; real grammar in reciplexa-syntax
- **L6236** — 14.3 予約語
  - notes: is_reserved_special_form / keywords; full policy 未決定
- **L6240** — 14.4 糖衣とCore
  - notes: elaborate covers many sugars; table incomplete
- **L6250** — 15.1 Kind
  - notes: kinds implicit in CoreType/rows; Module/Signature kinds absent
- **L6254** — 15.2 共通判断
  - notes: infer/check judgments in check.rs; module sig judgment absent
- **L6258** — 15.3 基本規則
  - notes: T-VAR/if/record etc. partially in checker
- **L6262** — 15.4 一般化
  - notes: let generalization light; value restriction incomplete
- **L6266** — 15.5 Subtypingと制約解決
  - notes: unify + subtype stubs; full constraint solver deferred
- **L6280** — 16.1 構成
  - notes: eval configurations in reciplexa-eval
- **L6288** — 16.3 効果伝播
  - notes: deep one-shot handlers; multi-shot deferred
- **L6292** — 16.4 SourceEdit
  - notes: syntax edit + source_sync; not full SourceEdit calculus
- **L6320** — 実装アーキテクチャ
  - notes: pipeline exists as vertical slice; not final
- **L6324** — 20.1 最終目標パイプライン
  - notes: bytes→CST→elaborate→check→eval→lower present; phase gaps
- **L6328** — 20.2 必要データ構造
  - notes: many structures exist; ModuleEnv/Typed Core incomplete
- **L6332** — 20.3 現行crateとの対応
  - notes: crate map outdated in places (core/eval/bind now carry more); still useful
- **L6342** — 21.1 型検査器要件
  - notes: typecheck_language_source; ModuleEnv/imported sigs incomplete
- **L6352** — 22.1 構文
  - notes: lexer/parser/edit tests present
- **L6356** — 22.2 静的意味
  - notes: check/unify + lang_kernel_suite STA-007/any/variance/casts coverage
- **L6360** — 22.3 動的意味
  - notes: eval + lang_kernel_suite TEST-DYN-003 failure/forward; multi-shot deferred
- **L6364** — 22.4 統合
  - notes: GUI/source_sync + TEST-INT-002 .rpi boundary; not full matrix
- **L6372** — 22.6 横断適合試験
  - notes: lang_kernel_suite cross-wires STA/DYN/INT + LANG-* IDs; not full cross suite
- **L6386** — 段階2: 仕様が明確になった
  - notes: spec largely written; grammar/Core still holes
- **L6390** — 段階3: 参照実装が動く
  - notes: reference pipeline runs; not full IR validators
- **L6394** — 段階4: 適合試験を通過
  - notes: some conformance tests; not versioned full suite
- **L6410** — 段階8: 実装と形式仕様の対応を確認
  - notes: crate↔spec mapping informal; no versioned correspondence report

### `SYN` — 22 partial

- **L102** — 13.2 `SYN-001` Code mode・字句・Surface構文・markup reader
  - notes: 字句〜markup+型表面/bytes核; intersect/糖衣等にギャップ
- **L110** — 0. 結論
  - notes: 結論の多くは実装; 一部PKG/型は未完
- **L114** — 1. Source fileと文字コード
  - notes: UTF-8+shebang trivia; BOM専用CST nodeなし
- **L122** — 1.2 BOM
  - notes: BOM検出・本体strip; CST専用nodeなし
- **L166** — 3. 識別子
  - notes: NFC/kebab/?!/_/不可視拒否ok; 厳密XIDは近似
- **L170** — 3.1 Unicode識別子
  - notes: is_alphabetic近似; 厳密XIDではない
- **L286** — 13. Source fileの宣言グループ
  - notes: val/rec/localあり; 自由式・注釈対応は弱い
- **L290** — 13.1 Top-level
  - notes: top-level宣言対応; 自由式を完全拒否せず
- **L294** — 13.2 型注釈
  - notes: (type name Ty)登録; 対val必須は未
- **L354** — 16. 型構文
  - notes: fn/app/forall/row/effects/union/intersect/not/diff; sugar gaps remain
- **L382** — 16.7 型alias
  - notes: type-alias登録; 再帰alias検査弱い
- **L386** — 17. `markup` reader
  - notes: reader/command核ok; sugar/型は部分
- **L394** — 17.2 Coreとpackageの分担
  - notes: CST readerあり; package分担は暫定
- **L410** — 17.6 任意のcode埋込み
  - notes: @(…)埋め込みCST可; walkはIdent名前提
- **L414** — 17.7 糖衣展開
  - notes: document/macro prototype; 完全sugar未
- **L430** — 17.11 Markupの型
  - notes: markup値の静的型は暫定/stub
- **L458** — 18.6 Markupの`[]`
  - notes: virtual `]` on EOF for @-expr brackets; multi-expr ErrorNode deferred
- **L462** — 18.7 Markup body
  - notes: markup body回復は汎用
- **L466** — 18.8 不正な`@`
  - notes: 不正@はError token/ node
- **L482** — 19. 適合例
  - notes: examples/testsで一部適合; 全列挙未
- **L486** — 20. 不適合例
  - notes: 拒否例の多くは検出; markup複数式等に差
- **L514** — Markup引数内の複数式
  - notes: markup複数式の拒否は部分的

### `ERR` — 20 partial

- **L5046** — 13.13 `ERR-001` 通常の失敗・Failure effect・後始末・Defect・最上位実行境界
  - notes: raise/handle/or-raise/as-result + Never; bracket/cleanup/defect deferred Part III
- **L5090** — 2. resultおよび専用結果型
  - notes: result-like data via DAT; dedicated ERR result API incomplete
- **L5094** — 2.1 用途
  - notes: result-like data via DAT; dedicated ERR result API incomplete
- **L5098** — 2.2 専用結果型
  - notes: custom result types via data; not a dedicated ERR API
- **L5102** — 2.3 複数errorの収集
  - notes: multi-error collect not a language primitive (user data only)
- **L5154** — 5.4 Handlerの結果型
  - notes: handler result typing via shared handle infer (interim)
- **L5178** — 7. 一つのFailure型への統合
  - notes: ERR §7 single-failure-type-per-boundary is policy, not enforced
- **L5210** — 9. resultとFailureの選択指針
  - notes: DAT option/result path; ERR choice-policy not auto-enforced
- **L5214** — 9.1 resultを推奨する場合
  - notes: DAT option/result path; ERR choice-policy not auto-enforced
- **L5334** — 18. Cancellation
  - notes: CancellationToken/Report in outcome+runtime; not full ERR cancel model
- **L5450** — 22.9 Foreign adapter
  - notes: classify_foreign_adapter + native quarantine→DefectReport; Terminal path stub; full fault boundary 依存待ち(第III部)
- **L5454** — 22.10 Resource exhaustion
  - notes: classify_resource_exhaustion + resource-error FailureCode; job budget/Terminal OOM path 依存待ち(第III部)
- **L5470** — 23.3 権限・機密性
  - notes: permission/secrecy fields present; host redaction 依存待ち(第III部)
- **L5478** — 25. Entry pointと実行環境
  - notes: raise/handle/FailureReport host path; RequiredEffects⊆Provided entry matrix 依存待ち(第III部)
- **L5490** — 26. 未処理Failure
  - notes: unhandled failure → FailureReport-shaped EvalError; Application sink/retry 依存待ち(第III部)
- **L5494** — 26.1 原則
  - notes: unhandled Failure surfaces at eval host; Application-boundary policy 依存待ち(第III部)
- **L5502** — 26.3 Runtime default表示
  - notes: FailureReport identity/code/message default display; full Error→Diagnostic explain API 依存待ち(第III部)
- **L5530** — 28. Diagnostic
  - notes: reciplexa-diagnostic exists; ERR primary/suppressed model incomplete
- **L5538** — 28.2 PrimaryとSuppressed
  - notes: FailureDiagnosticBundle primary→suppressed order; cleanup wiring 依存待ち(第III部)
- **L5546** — 29. 適合試験
  - notes: ERR-01/02/03 in handle_tests; full ERR-0N suite incomplete

### `BND` — 19 partial

- **L1210** — 13.4 `BND-001` `val`、`var`、`let`、`letrec`、`fn`
  - notes: let/letrec/var/set done; gen/escape/typed-store deferred (plan BND-001)
- **L1226** — `DD-BND-003`: `(type ...)`による型注釈
  - notes: surface (type …) aliases in DataEnv; checking incomplete vs DD
- **L1230** — `DD-BND-004`: 型注釈のscopeと対応関係
  - notes: surface (type …) aliases in DataEnv; checking incomplete vs DD
- **L1234** — `DD-BND-005`: 型注釈は検査される
  - notes: surface (type …) aliases in DataEnv; checking incomplete vs DD
- **L1242** — `DD-BND-007`: `letrec`内の型注釈
  - notes: LetRec elaborates; annotations / poly inference limited
- **L1254** — `DD-BND-010`: 再帰関数の型推論
  - notes: check.rs LetRec stub then unify; not full recursive inference
- **L1266** — `DD-BND-013`: `var`の型注釈
  - notes: var elaborates; typed store deferred (plan BND-001)
- **L1270** — `DD-BND-014`: `var`の格納型は固定
  - notes: runtime cell untyped; check binds init ty only
- **L1286** — `DD-BND-018`: local state identity
  - notes: Cell identity via Rc; limited vs spec
- **L1290** — `DD-BND-019`: local state escapeの禁止
  - notes: eval Cell.alive rejects use after scope exit; not full escape analysis
- **L1354** — top-level型注釈
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1358** — 関数型注釈
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1362** — 注釈不一致
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1366** — 対応bindingのない型注釈
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1378** — 一部だけ注釈
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1386** — 明示的多相型
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1390** — 注釈によるvalue restriction回避の拒否
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1406** — 明示union格納型
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite
- **L1418** — effectful関数値の一般化
  - notes: lang_kernel_suite covers letrec/var basics; not full BND suite

### `EDT` — 19 partial

- **L4346** — 13.11 `EDT-001` 編集スナップショット・トランザクション・競合・由来情報
  - notes: DocumentSnapshot/Transaction/BindingMap use-sites + MacroSourceMap; full GUI reconciliation 依存待ち(第V部)
- **L4374** — 1. 編集モデルの基本原則
  - notes: snapshot + StableNodeId + BindingId use-sites; full EDT value model 依存待ち(第V部)
- **L4378** — 1.1 不変値と継続的identity
  - notes: DocumentIdentity + StableNodeId; full EDT value model 依存待ち(第V部)
- **L4390** — 2. 文書の所有構造
  - notes: NodeStore ownership tree
- **L4578** — 9.3 SetProperty
  - notes: SetLayout/SetText; no generic SetProperty
- **L4610** — 11. 適用前条件
  - notes: basic UnknownNode/InvalidParent checks
- **L4666** — 14. 適用手順
  - notes: simplified apply path without full 9-step protocol
- **L4686** — 5. 現在snapshotから作業状態を作成
  - notes: mutates snapshot in place with rollback
- **L4706** — 15. 適用結果
  - notes: TransactionOutcome Applied/AppliedNoChange/Rejected
- **L4762** — 17. Provenance
  - notes: SourceProvenance + NodeProvenance; not full EDT provenance taxonomy
- **L4774** — 17.3 構造
  - notes: flat SourceProvenance struct
- **L4866** — 21. 公開API階層
  - notes: Rust document API only; not RPX-exported EDT API
- **L4870** — 21.1 純粋な第一級値
  - notes: DocumentEdit values; not RPX first-class
- **L4874** — 21.2 状態付きhandle
  - notes: working DocumentSnapshot handle in GUI path
- **L4890** — 22.1 高水準API
  - notes: source_sync GUI edits
- **L4894** — 22.2 低水準API
  - notes: DocumentTransaction ops
- **L4902** — 23. Pure処理とEffectful処理
  - notes: doc txs pure-ish; host I/O separate (RSC)
- **L4910** — 23.2 Effectful処理
  - notes: source rewrite side effects via GUI sync
- **L4918** — 24. 競合・不正・実行障害の分離
  - notes: TransactionError vs Outcome; no conflict type

### `MOD` — 16 partial

- **L3010** — 13.9 `MOD-001` モジュール・シグネチャ・Functor・分割コンパイル
  - notes: outer unit + import/link + .rpi export filter/path; signatures/functors deferred
- **L3022** — DD-001.2 中心的な決定
  - notes: import as/only/rename+qualified + .rpi export boundary; functors/signatures deferred
- **L3118** — 4.5 Top-level effect
  - notes: unit body elaborated as Core; no module-level effect gate
- **L3138** — 5.3 内部表現
  - notes: qualified name is string binder `alias/export`, not ModuleId path IR
- **L3178** — 7. Importと正式identity
  - notes: alias is local prefix only; no formal ModuleId identity layer
- **L3182** — 7.1 Aliasの効果
  - notes: alias does not change export identity (string copy of binding)
- **L3186** — 7.2 選択的import
  - notes: selective import binds locals; no sealed identity table
- **L3198** — 8. .rpiインターフェース
  - notes: .rpi stub parse + export name boundary; full signature checking deferred
- **L3202** — 8.1 役割
  - notes: .rpi lists public vals/types; enforced at package link
- **L3234** — 9.2 .rpiありのモジュール
  - notes: .rpi filters public exports at link; types/abstract not checked yet
- **L3494** — 20. 正式identity
  - notes: some opaque IDs exist in reciplexa-identity
- **L3514** — 20.5 TypeId
  - notes: TypeId opaque id exists; not wired through MOD metadata
- **L3518** — 20.6 ConstructorId
  - notes: ConstructorId opaque id exists; not MOD metadata
- **L3534** — 20.10 Rename
  - notes: import rename only; no formal Rename on identity
- **L3578** — 21.9 適合試験 MOD-01
  - notes: qualified path import covered by module_tests; not named MOD-01 suite
- **L3582** — 21.10 適合試験 MOD-02
  - notes: as+only+rename covered by elaborate_units_* tests; not MOD-02 harness

### `MAC` — 9 partial

- **L1450** — 13.5 `MAC-001` 最小式マクロ・展開・衛生性
  - notes: 式マクロ核ok; hygiene/scope/診断に差
- **L1626** — 7.3 Templateの構文妥当性
  - notes: def時var検査; 完全構文妥当性は弱い
- **L1654** — 9. マクロ名と呼出し
  - notes: 呼出し構文ok; 衝突検査弱
- **L1798** — 14.3 概念的なidentity
  - notes: MacroSourceMap SyntaxNodeId + BindingMap use-sites; Core BindingId string dual still open
- **L1862** — 18. エラー診断
  - notes: 主要診断あり; span/provenance弱
- **L1882** — 18.5 不正な展開結果
  - notes: 不正結果は後段エラーに依存
- **L1886** — 18.6 型エラー
  - notes: 型エラーのマクロ帰属なし
- **L1898** — 19.2 診断例
  - notes: expand provenance recorded; typed diagnostic attachment still light
- **L1926** — 21. 適合試験
  - notes: unit/integrationで一部; 全MAC試験未

### `EFF` — 8 partial

- **L2914** — 13.8 `EFF-001` Algebraic effects and handlers
  - notes: deep one-shot + ambient + with/handler; multi-shot/return deferred
- **L2918** — 状態
  - notes: deep one-shot + ambient + with/handler; multi-shot/return deferred
- **L2930** — `DD-EFF-003`: resumptionの型とscope
  - notes: one-shot resume value; resume typing interim Dynamic
- **L2958** — `DD-EFF-010`: handler valueのrank-1多相性
  - notes: HandlerValue is Dynamic in check.rs
- **L2970** — `DD-EFF-013`: ambient effectとnamed/scoped effect instance
  - notes: ambient ok (elaborate Perform); named/scoped instances deferred
- **L2982** — `DD-EFF-014`: EffectRowの意味
  - notes: thin EffectRow; handler typing interim (check.rs)
- **L2986** — `DD-EFF-015`: ambient effect rowと制約生成
  - notes: thin EffectRow; handler typing interim (check.rs)
- **L2990** — `DD-EFF-016`: handlerの型付け骨格
  - notes: thin EffectRow; handler typing interim (check.rs)

### `IR` — 6 partial

- **L5010** — 13.12 `IR-001` Layered visual/motion/render IR
  - notes: backends+motion+view exist; full layered IR schema still provisional
- **L5018** — SurfaceとArtifact
  - notes: scene/document surfaces exist; Artifact algebra incomplete
- **L5022** — RenderIR node algebra
  - notes: pdf/svg/pptx/view primitives; full RenderIR node algebra incomplete
- **L5026** — MotionIR
  - notes: reciplexa-motion MotionTrack Constant/Keyframes/Samples
- **L5030** — Backend lowering
  - notes: pdf/svg/pptx lowering; AE/edit-preserving path incomplete
- **L5042** — テスト
  - notes: phase12_motion TEST-IR-007 style; full IR-001..009 validator suite 意図的後回し

### `EVAL` — 4 partial

- **L1110** — 13.3 `EVAL-001` Strict lexical Core evaluator
  - notes: eval.rs CBV Core; also letrec/match/effects beyond min-Core v1
- **L1154** — 評価文脈
  - notes: evaluation contexts via Outcome/resume; not formal EC grammar
- **L1166** — 適合試験
  - notes: eval_tests + lang_kernel_suite EVAL/DYN-003; named EVAL corpus incomplete
- **L1206** — 解決後の最小Core
  - notes: resolve BindingMap + SyntaxNodeId on defs (lang_kernel_suite); eval still name-string Core

### `LEX` — 3 partial

- **L66** — 13.1 `LEX-001` Lossless lexer/CST
  - notes: rowan CST+unparse+shebang trivia; BOM専用node/仮想tokenは不足
- **L74** — 構文・字句
  - notes: lexer/kind/parse; Comment kind未使用・OPEN字句あり
- **L82** — 正常例・拒否例
  - notes: trivia保持はok; 単位suffixはinterim

### `RES` — 2 partial

- **L530** — 13.2.1 `RES-001` 名前解決とnamespace
  - notes: resolve_language_source + BindingMap; multi-ns/phase/import ambiguity incomplete
- **L534** — 概要・状態
  - notes: TEST-RES-C001 shadowing/unbound; Type/Module/Syntax ns + phase resolve 暫定のまま

### `ROW` — 2 partial

- **L2906** — 13.7 `ROW-001` Row-polymorphic records
  - notes: closed+OpenRecord+Lacks done; multi-tail deferred (plan ROW-001)
- **L2910** — 概要・状態
  - notes: closed+OpenRecord+Lacks done; multi-tail deferred (plan ROW-001)

### `KER` — 2 partial

- **L4330** — 13.10.1 `KER-001` Rust kernelとforeign primitive境界
  - notes: BuiltinOp arithmetic/compare + EffectHost; full Rust/FFI ABI deferred
- **L4334** — 概要・状態
  - notes: kernel ops in eval/check; TEST-KER-* / foreign validator absent

### `RSC` — 2 partial

- **L4338** — 13.10.2 `RSC-001` Resource、I/O、host-handler境界
  - notes: MemoryFsHost read-file/write-file; richer catalog/path safety deferred
- **L4342** — 概要・状態
  - notes: in-memory host for tests; resolve-font/load-image etc. not in language kernel

