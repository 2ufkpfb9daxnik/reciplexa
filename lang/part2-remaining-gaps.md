# Part II — remaining `gap` inventory

Generated from `part2-conformance.md` + `part2-conformance-stats.json`.
Grouped by parent L4 `XXX-001` feature block (file order). `LINE` = line in `part2-conformance.md`.

## Current stats

| metric | count |
|---|---:|
| `total` | 1589 |
| `unchecked` | 0 |
| `ok` | 356 |
| `partial` | 538 |
| `gap` | 205 |
| `deferred` | 355 |
| `meta` | 135 |

Attributed checklist gaps: **205** (matches `stats.gap`).

## Gaps per feature

| feature | gap | partial | gap+partial |
|---|---:|---:|---:|
| `LEX` | 0 | 3 | 3 |
| `SYN` | 0 | 24 | 24 |
| `RES` | 0 | 2 | 2 |
| `DAT` | 0 | 55 | 55 |
| `EVAL` | 0 | 7 | 7 |
| `BND` | 0 | 19 | 19 |
| `MAC` | 0 | 14 | 14 |
| `TYP` | 92 | 101 | 193 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 0 | 10 | 10 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 0 | 20 | 20 |
| `PKG` | 0 | 50 | 50 |
| `KER` | 0 | 2 | 2 |
| `EDT` | 75 | 56 | 131 |
| `IR` | 1 | 5 | 6 |
| `ERR` | 19 | 37 | 56 |
| `MEM` | 14 | 102 | 116 |
| `TST` | 4 | 27 | 31 |
| **total** | **205** | **538** | **743** |

## Remaining gaps (compact)

### `TYP` — 92 gaps

- **L2254** — dynamic typingの終端状態
- **L2274** — safe static use
- **L2278** — partial overlap
- **L2282** — disjoint use
- **L2286** — cast precision
- **L2294** — foreign ingress
- **L2298** — implicit cast failure
- **L2302** — explicit safe cast
- **L2306** — fixed-arity function cast
- **L2310** — function result cast
- **L2314** — effect-compatible function cast
- **L2318** — effect-incompatible function cast
- **L2322** — polymorphic value boundary
- **L2326** — dynamicからforall
- **L2330** — numeric promotion
- **L2334** — namespace collision warning
- **L2374** — `DD-TYP-ALG-001`: 宣言的型関係とalgorithmic判定の分離
- **L2378** — `DD-TYP-ALG-002`: algorithmic判定の三値結果
- **L2382** — `proved`
- **L2386** — `disproved`
- **L2390** — `unknown`
- **L2394** — `DD-TYP-ALG-003`: 診断分類
- **L2398** — `type-error`
- **L2402** — `annotation-required`
- **L2406** — `checker-limitation`
- **L2410** — `checker-resource-limit`
- **L2414** — `unsupported-language-feature`
- **L2418** — `DD-TYP-ALG-004`: soundness、completeness、terminationの優先順位
- **L2422** — `DD-TYP-ALG-005`: principal type
- **L2426** — `DD-TYP-ALG-006`: 決定的なsolver budget
- **L2486** — `DD-TYP-EFF-007`: 注釈されたrequired effects
- **L2490** — Static型のBoolean代数
- **L2494** — `DD-TYP-BOOL-001`: 型のBoolean演算
- **L2498** — `DD-TYP-BOOL-002`: Surface negationの制限
- **L2502** — Singleton型
- **L2506** — `DD-TYP-SINGLETON-001`: singleton型
- **L2522** — `DD-TYP-FN-003`: function intersection
- **L2526** — `DD-TYP-FN-004`: function intersectionの適用可能性
- **L2530** — `DD-TYP-FN-005`: branch specificity
- **L2534** — `DD-TYP-FN-006`: function intersectionのcoherence
- **L2538** — 入力領域が互いに素
- **L2542** — 片方が他方を包含
- **L2546** — 入力領域が等価
- **L2550** — 入力領域が重なるが非比較
- **L2554** — `DD-TYP-FN-007`: union引数とdispatch
- **L2606** — `DD-TYP-ROW-007`: optional field
- **L2610** — `DD-TYP-ROW-008`: recordのBoolean演算
- **L2614** — Recursive data type
- **L2618** — `DD-TYP-REC-001`: recursive data type
- **L2622** — `DD-TYP-REC-002`: equi-recursiveな利用者意味論
- **L2626** — `DD-TYP-REC-003`: contractiveness
- **L2630** — `DD-TYP-REC-004`: strict positivity
- **L2634** — `DD-TYP-REC-005`: regularity
- **L2638** — `DD-TYP-REC-006`: base constructor
- **L2642** — `DD-TYP-REC-007`: recursive dataの完全判定範囲
- **L2686** — `DD-TYP-BIDI-008`: annotation-requiredとchecker-limitation
- **L2690** — 明示的多相型
- **L2694** — `DD-TYP-POLY-001`: 明示的`forall`
- **L2698** — `DD-TYP-POLY-002`: `forall` binderのkind
- **L2702** — `DD-TYP-POLY-003`: `forall`のscope
- **L2706** — `DD-TYP-POLY-004`: rank-1／prenex制限
- **L2710** — 完全性分類
- **L2714** — `DD-TYP-FRAG-001`: A — 完全判定fragment
- **L2718** — `DD-TYP-FRAG-002`: B — `unknown`を返し得るfragment
- **L2722** — `DD-TYP-FRAG-003`: C — 注釈を要求し得るfragment
- **L2726** — `DD-TYP-FRAG-004`: D — RPX v1で禁止するfragment
- **L2730** — 共通constraint worklist
- **L2734** — `DD-TYP-SOLVER-001`: shared constraint worklist
- **L2738** — `DD-TYP-SOLVER-002`: solver間のconstraint生成
- **L2742** — `DD-TYP-SOLVER-003`: constraint処理の優先度
- **L2746** — `DD-TYP-SOLVER-004`: canonicalizationとmemoization
- **L2750** — `DD-TYP-SOLVER-005`: solver終了状態
- **L2754** — 成功
- **L2758** — Type error
- **L2762** — Annotation required
- **L2766** — Checker limitation
- **L2770** — Resource limit
- **L2774** — `DD-TYP-SOLVER-006`: cast insertionとgeneralizationの順序
- **L2778** — 型検査器の概念pipeline
- **L2782** — `DD-TYP-SOLVER-007`: checkerの全体処理順序
- **L2790** — 三値判定
- **L2794** — 診断分類
- **L2814** — Singleton型
- **L2822** — Function intersection coherence
- **L2826** — 非比較なbranch overlap
- **L2830** — Union引数の暗黙dispatch禁止
- **L2846** — Recursive data
- **L2850** — 非contractive再帰
- **L2854** — Non-regular recursion
- **L2862** — Explicit `forall`
- **L2866** — Kind error
- **L2870** — Solver determinism

### `EDT` — 75 gaps

- **L4402** — 2.3 共有
- **L4438** — 7. 全ノードがRootから到達可能である
- **L4442** — 8. 必須propertyが存在する
- **L4446** — 9. Property値がNode kindのschemaに適合する
- **L4450** — 10. 強いNodeId参照が有効な対象を指す
- **L4454** — 3.1 到達不能ノード
- **L4478** — 4.5 TransactionId
- **L4486** — 5. 保存・複製・Fork
- **L4490** — 5.1 通常保存
- **L4494** — 5.2 Save As
- **L4498** — 5.3 Duplicate／Fork
- **L4502** — 5.4 内部参照の複製
- **L4506** — 5.5 文書間参照
- **L4526** — 6.4 保存後のrevision
- **L4538** — 7.2 過去版の永久取得
- **L4542** — 7.3 履歴の種類
- **L4602** — 10.2 予約済みID
- **L4606** — 10.3 Copy
- **L4622** — 11.3 Base revision
- **L4626** — 12. Stale transaction
- **L4630** — 12.1 定義
- **L4634** — 12.2 分類
- **L4638** — 12.3 保守的な再適用
- **L4642** — 12.4 無関係な変更
- **L4646** — 13. 競合
- **L4650** — 13.1 基本分類
- **L4654** — 13.2 競合と不正トランザクション
- **L4658** — 13.3 競合の収集
- **L4662** — 13.4 自動併合
- **L4670** — 1. TransactionIdを確認
- **L4674** — 2. DocumentIdを確認
- **L4678** — 3. Base revisionを比較
- **L4682** — 4. Transaction-level preconditionを検査
- **L4694** — 7. 文書全体の不変条件を検査
- **L4702** — 9. 新revisionとUndo情報を生成
- **L4726** — 15.5 AlreadyApplied
- **L4730** — 15.6 Transaction content hash
- **L4734** — 16. UndoとRedo
- **L4738** — 16.1 新revision
- **L4742** — 16.2 Undo情報
- **L4746** — 16.3 UndoToken
- **L4750** — 16.4 Undo競合
- **L4754** — 16.5 Redo
- **L4758** — 16.6 履歴保持
- **L4778** — 17.4 種類
- **L4782** — 17.5 UserCreated
- **L4786** — 17.6 SourceGenerated
- **L4790** — 17.7 MacroGenerated
- **L4794** — 17.8 Imported
- **L4798** — 17.9 Copied
- **L4802** — 17.10 Derived
- **L4806** — 18. 派生ノードと逆編集
- **L4810** — 18.1 編集可能性
- **L4818** — 18.3 逆編集結果
- **L4822** — 18.4 自動選択
- **L4826** — 18.5 逆写像不能
- **L4830** — 18.6 Stale provenance
- **L4834** — 19. 派生ノードのID継承
- **L4838** — 19.1 DerivationKey
- **L4842** — 19.2 曖昧な対応
- **L4846** — 19.3 NodeIdとの違い
- **L4854** — 20.1 真正性
- **L4858** — 20.2 Privacy
- **L4862** — 20.3 書換え
- **L4878** — 21.3 抽象型
- **L4882** — 21.4 Constructor付き公開data
- **L4930** — 24.3 実行障害
- **L4938** — 25. Undo履歴・Transaction履歴
- **L4942** — 25.1 有限保持
- **L4946** — 25.2 AlreadyApplied保証
- **L4950** — 25.3 Undo不可
- **L4954** — 26. 永続化
- **L4958** — 26.1 標準保存
- **L4962** — 26.2 編集履歴
- **L4966** — 26.3 Version付きcodec

### `IR` — 1 gaps

- **L5042** — テスト

### `ERR` — 19 gaps

- **L5450** — 22.9 Foreign adapter
- **L5454** — 22.10 Resource exhaustion
- **L5478** — 25. Entry pointと実行環境
- **L5482** — 25.1 Runtime capability
- **L5486** — 25.2 実行環境ごとのmain
- **L5490** — 26. 未処理Failure
- **L5494** — 26.1 原則
- **L5498** — 26.2 最終防御
- **L5502** — 26.3 Runtime default表示
- **L5506** — 27. 実行環境別の処理
- **L5510** — 27.1 CLI
- **L5514** — 27.2 GUI
- **L5518** — 27.3 Server
- **L5522** — 27.4 Plugin
- **L5526** — 27.5 Render job
- **L5534** — 28.1 構築と出力の分離
- **L5538** — 28.2 PrimaryとSuppressed
- **L5542** — 28.3 Libraryの責務
- **L5546** — 29. 適合試験

### `MEM` — 14 gaps

- **L5950** — 20. Concurrencyへの接続
- **L5962** — 20.3 Scoped Resource
- **L5990** — 22.1 通常Allocation
- **L6002** — 23. メモリ予算
- **L6018** — 24. 予算超過
- **L6034** — 25. 単一巨大Allocation
- **L6046** — 26. Continuation予算
- **L6058** — 27. Snapshot保持量
- **L6062** — 27.1 有効なSnapshot handle
- **L6074** — 28. 一般heap OOM
- **L6078** — 28.1 管理予算との区別
- **L6090** — 29. Reference count overflow
- **L6126** — 31. メモリ観測API
- **L6142** — 32. GUI状態

### `TST` — 4 gaps

- **L6206** — Black/white box
- **L6368** — 22.5 Property/differential/fuzz
- **L6402** — 段階6: 差分・生成・fuzzを通過
- **L6406** — 段階7: メタ理論が確認された

