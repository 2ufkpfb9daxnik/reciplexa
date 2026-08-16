# 第II部 conformance — `deferred` 一覧

## deferred とは何か

`deferred` は、`lang/part2-conformance.md` 上で**レビュー済みだが、いま実装適合を追わない**と印した見出しである。「未実装だから gap」ではなく、**意図的に後回し**（実装順序・依存・仕様の開き）であることを notes に残すためのステータス。したがって `gap`/`partial` の埋める優先度とは別枠で管理する。

**重要:** `deferred` は必ずしも「ユーザー未決定」ではない。レビュー時の分類であり、仕様 OPEN・依存待ち・実装順序による後回しも含む。

### 分類ごとの件数

- **合計**: 683
- **仕様未決定**: 60
- **意図的後回し**: 366
- **依存待ち**: 257
- **要確認**: 0

分類の基準:

- **仕様未決定** — `OPEN-*`、仕様本文の「未決定／保留」、またはまだ閉じない設計選択
- **意図的後回し** — 仕様は概ね確定しているが、実装順序（カーネル→PKG、multi-shot EFF 等）で後にする
- **依存待ち** — 他機能（PKG/MOD/ERR 等）や第III部以降の前提が先
- **要確認** — title/notes だけでは上記に切れないもの

---

## 仕様未決定（60）

### L427: 未決定
- **機能ブロック**: `LEX`
- **notes**: OPEN-SYN-002/OPEN-EDT-001 追跡
- **分類**: 仕様未決定

### L3483: 12.1 Cyclic data値
- **機能ブロック**: `DAT`
- **notes**: OPEN-GRAPH / cyclic values deferred
- **分類**: 仕様未決定

### L3509: 12.3 Sharingの非観測性
- **機能ブロック**: `DAT`
- **notes**: OPEN-GRAPH / cyclic values deferred
- **分類**: 仕様未決定

### L3515: 12.4 無限構造
- **機能ブロック**: `DAT`
- **notes**: OPEN-GRAPH / cyclic values deferred
- **分類**: 仕様未決定

### L7392: 10.4 Typed macro
- **機能ブロック**: `MAC`
- **notes**: typed macroはOPEN-MAC-TYPED
- **分類**: 仕様未決定

### L7696: 19.4 正式identity
- **機能ブロック**: `MAC`
- **notes**: 正式identityはOPEN-EDT
- **分類**: 仕様未決定

### L7702: 20. マクロの外部公開
- **機能ブロック**: `MAC`
- **notes**: 外部公開はv1制限/OPEN-MAC-PKG
- **分類**: 仕様未決定

### L7920: 22.1 `OPEN-MAC-EXT-001`
- **機能ブロック**: `MAC`
- **notes**: OPEN-MAC-EXT-001
- **分類**: 仕様未決定

### L7933: 22.2 `OPEN-MAC-PKG-001`
- **機能ブロック**: `MAC`
- **notes**: OPEN-MAC-PKG-001
- **分類**: 仕様未決定

### L7944: 22.3 `OPEN-MAC-PROC-001`
- **機能ブロック**: `MAC`
- **notes**: OPEN-MAC-PROC-001
- **分類**: 仕様未決定

### L7954: 22.4 `OPEN-MAC-TYPED-001`
- **機能ブロック**: `MAC`
- **notes**: OPEN-MAC-TYPED-001
- **分類**: 仕様未決定

### L7963: 22.5 `OPEN-MAC-CAP-001`
- **機能ブロック**: `MAC`
- **notes**: OPEN-MAC-CAP-001
- **分類**: 仕様未決定

### L7971: 22.6 `OPEN-EDT-001`
- **機能ブロック**: `MAC`
- **notes**: OPEN-EDT-001
- **分類**: 仕様未決定

### L15558: 21.8 ABI hash
- **機能ブロック**: `MOD`
- **notes**: AbiHash → OPEN-KER-001
- **分類**: 仕様未決定

### L15720: 22.1 `OPEN-PKG-001`
- **機能ブロック**: `MOD`
- **notes**: OPEN-PKG-001 — packages deferred
- **分類**: 仕様未決定

### L15740: 22.2 `OPEN-MAC-001`
- **機能ブロック**: `MOD`
- **notes**: macro-phase identity → MAC
- **分類**: 仕様未決定

### L15751: 22.3 `OPEN-KER-001`
- **機能ブロック**: `MOD`
- **notes**: OPEN-KER-001 ABI
- **分類**: 仕様未決定

### L15761: 22.4 `OPEN-MOD-REC-001`
- **機能ブロック**: `MOD`
- **notes**: recursive modules out of v1
- **分類**: 仕様未決定

### L15773: 22.5 `OPEN-MOD-FC-001`
- **機能ブロック**: `MOD`
- **notes**: first-class modules out of v1
- **分類**: 仕様未決定

### L15783: 22.6 `OPEN-MOD-GEN-001`
- **機能ブロック**: `MOD`
- **notes**: generative functors out of v1
- **分類**: 仕様未決定

### L17158: 22.1 `OPEN-BLD-001`
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 仕様未決定

### L17171: 22.2 `OPEN-PKG-FEAT-001`
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 仕様未決定

### L17181: 22.3 `OPEN-REG-001`
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 仕様未決定

### L17193: 22.4 `OPEN-KER-001`
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 仕様未決定

### L17204: 22.5 `OPEN-TST-001`
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 仕様未決定

### L17215: 22.6 `OPEN-ERR-001`
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 仕様未決定

### L17224: 22.7 `OPEN-CON-001`
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 仕様未決定

### L17260: Compiler-native package実装
- **機能ブロック**: `PKG`
- **notes**: native package swap deferred (OPEN-NATIVE-PKG)
- **分類**: 仕様未決定

### L17412: 0.2 直接の対象としないもの
- **機能ブロック**: `EDT`
- **notes**: collab/codec overrides → OPEN-EDT-*
- **分類**: 仕様未決定

### L18778: `OPEN-ERR-001`
- **機能ブロック**: `EDT`
- **notes**: OPEN-ERR-001 transferred / out of pre-PKG kernel
- **分類**: 仕様未決定

### L18787: `OPEN-MEM-001`
- **機能ブロック**: `EDT`
- **notes**: OPEN-MEM-001 transferred / out of pre-PKG kernel
- **分類**: 仕様未決定

### L18795: `OPEN-CON-001`
- **機能ブロック**: `EDT`
- **notes**: OPEN-CON-001 transferred / out of pre-PKG kernel
- **分類**: 仕様未決定

### L18804: `OPEN-IR-001`
- **機能ブロック**: `EDT`
- **notes**: OPEN-IR-001 transferred / out of pre-PKG kernel
- **分類**: 仕様未決定

### L18812: `OPEN-EDT-CODEC-001`
- **機能ブロック**: `EDT`
- **notes**: OPEN-EDT-CODEC-001 transferred / out of pre-PKG kernel
- **分類**: 仕様未決定

### L18819: `OPEN-EDT-COLLAB-001`
- **機能ブロック**: `EDT`
- **notes**: OPEN-EDT-COLLAB-001 transferred / out of pre-PKG kernel
- **分類**: 仕様未決定

### L18828: `OPEN-EDT-OVERRIDE-001`
- **機能ブロック**: `EDT`
- **notes**: OPEN-EDT-OVERRIDE-001 transferred / out of pre-PKG kernel
- **分類**: 仕様未決定

### L20401: `OPEN-MEM-001`
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L20410: `OPEN-CON-001`
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L20419: `OPEN-KER-001`
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L20427: `OPEN-TST-001`
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L20435: `OPEN-PKG-ENTRY-001`
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L20442: `OPEN-ERR-DIAG-001`
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L21061: 13.2 Capture
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し(OPEN-MEM-CONT): continuation capture 表現は最小
- **分類**: 仕様未決定

### L21098: 13.6 Escape
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し(OPEN-MEM-CONT): resume 範囲を越える escape 検査は後回し
- **分類**: 仕様未決定

### L21338: 20. Concurrencyへの接続
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(ASY)|仕様未決定: OPEN-CON-001 atomic RC / Send/Share
- **分類**: 仕様未決定

### L21339: 20.1 v1の範囲
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(ASY)|OPEN-CON-001: concurrency v1 scope- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 仕様未決定

### L21345: 20.2 共有値
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(ASY)|OPEN-CON-001: shared values across tasks- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 仕様未決定

### L21677: 31. メモリ観測API
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部)|OPEN-MEM-PROF-001: RC 非公開は §31.1 ok; budget/peak 観測 API は後回し
- **分類**: 仕様未決定

### L21691: 31.2 許可される情報
- **機能ブロック**: `MEM`
- **notes**: 依存待ち|OPEN-MEM-PROF-001: allowed memory observation API- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 仕様未決定

### L21701: 31.3 安定性
- **機能ブロック**: `MEM`
- **notes**: 依存待ち|OPEN-MEM-PROF-001: observation API stability- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 仕様未決定

### L21714: 32. GUI状態
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し(OPEN-GUI-STATE-001): DocumentSnapshot 明示モデル; 完全 GUI 状態は第V部
- **分類**: 仕様未決定

### L21719: 32.2 推奨モデル
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し(OPEN-GUI-STATE-001): Perceus 既定経路ではなく推奨モデル記述のみ
- **分類**: 仕様未決定

### L21908: `OPEN-MEM-CELL-001`
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L21917: `OPEN-MEM-BORROW-001`
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L21926: `OPEN-CON-001`
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L21935: `OPEN-KER-001`
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L21943: `OPEN-GUI-STATE-001`
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L21950: `OPEN-MEM-PROF-001`
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
- **分類**: 仕様未決定

### L21988: 13.14.1 `ASY-001` Concurrency and async
- **機能ブロック**: `ASY`
- **notes**: ASY-001 未決定; runtime scheduler exists but language async unspecified
- **分類**: 仕様未決定

### L22039: Black/white box
- **機能ブロック**: `TST`
- **notes**: 仕様未決定: test-module/test-of white-box companion surface
- **分類**: 仕様未決定

---

## 意図的後回し（366）

### L1360: 12. 単位と色
- **機能ブロック**: `SYN`
- **notes**: 単位・色はPKG担当; suffixはinterim分割
- **分類**: 意図的後回し

### L1362: 12.1 単位
- **機能ブロック**: `SYN`
- **notes**: 40mm→Number+Ident; typed (mm 40)はPKG
- **分類**: 意図的後回し

### L1402: 12.2 色
- **機能ブロック**: `SYN`
- **notes**: #hexなし; 色ctorはPKG
- **分類**: 意図的後回し

### L2342: 18.7 Markup body
- **機能ブロック**: `SYN`
- **notes**: 意図的後回し: markup body 回復は汎用; 専用回復戦略は後回し
- **分類**: 意図的後回し

### L2425: 19. 適合例
- **機能ブロック**: `SYN`
- **notes**: 意図的後回し: examples/tests で主要適合; 仕様列挙の全網羅は後回し
- **分類**: 意図的後回し

### L2477: 20. 不適合例
- **機能ブロック**: `SYN`
- **notes**: 意図的後回し: 主要拒否は検出; markup 複数式など差のある葉は個別追跡
- **分類**: 意図的後回し

### L2520: Markup引数内の複数式
- **機能ブロック**: `SYN`
- **notes**: 意図的後回し(OPEN): markup 引数内複数式の厳密拒否は部分的
- **分類**: 意図的後回し

### L2897: 2.3 Constructor固有型
- **機能ブロック**: `DAT`
- **notes**: plan: full polymorphic ADT typing / ctor-specific types deferred
- **分類**: 意図的後回し

### L2930: 2.5 Runtime reflection
- **機能ブロック**: `DAT`
- **notes**: spec §2.5: data decls do not auto-emit runtime type descriptors; type-of-value / constructors-of are separate future items
- **分類**: 意図的後回し

### L2946: 3. 名前とnamespace
- **機能ブロック**: `DAT`
- **notes**: 意図的後回し: value-ns ctors は DataEnv; 完全な type-ns ctor 型の二重名前空間は後回し
- **分類**: 意図的後回し

### L2948: 3.1 型namespaceと値namespace
- **機能ブロック**: `DAT`
- **notes**: 意図的後回し: type/value namespace 分離の完全モデルは後回し
- **分類**: 意図的後回し

### L2966: 3.2 型名とconstructor名の同名禁止
- **機能ブロック**: `DAT`
- **notes**: 意図的後回し: 同名 type/ctor の厳密 dual-ns 強制は後回し
- **分類**: 意図的後回し

### L3106: 6. Constructor固有型
- **機能ブロック**: `DAT`
- **notes**: plan: full polymorphic ADT typing / ctor-specific types deferred
- **分類**: 意図的後回し

### L3128: 6.2 Subtyping
- **機能ブロック**: `DAT`
- **notes**: plan: full polymorphic ADT typing / ctor-specific types deferred
- **分類**: 意図的後回し

### L3137: 6.3 親型とのsealed union関係
- **機能ブロック**: `DAT`
- **notes**: plan: full polymorphic ADT typing / ctor-specific types deferred
- **分類**: 意図的後回し

### L3149: 6.4 表示上の単純化
- **機能ブロック**: `DAT`
- **notes**: plan: full polymorphic ADT typing / ctor-specific types deferred
- **分類**: 意図的後回し

### L3251: 7.5 値位置の明示型argument
- **機能ブロック**: `DAT`
- **notes**: DAT param typing still Dynamic; plan DAT-001 deferral
- **分類**: 意図的後回し

### L3259: 8. Variance
- **機能ブロック**: `DAT`
- **notes**: 意図的後回し: per-param cov/contra/invar/phantom 記録済（DataEnv.type_variances）; 部分型latticeへの完全反映は後回し
- **分類**: 意図的後回し

### L3403: 10.3 宣言kindの混在禁止
- **機能ブロック**: `DAT`
- **notes**: 意図的後回し: rec 内 kind 混在の厳密拒否は弱い
- **分類**: 意図的後回し

### L3944: 19.5 Disjoint pattern
- **機能ブロック**: `DAT`
- **notes**: 意図的後回し: disjoint-pattern の精密型付けは Dynamic/exhaustiveness 経由
- **分類**: 意図的後回し

### L3964: 20.3 結果型
- **機能ブロック**: `DAT`
- **notes**: 意図的後回し: arm 結果型 unify; GADT/refine は Dynamic 依存
- **分類**: 意図的後回し

### L4079: 21.6 Transparent export
- **機能ブロック**: `DAT`
- **notes**: MOD export/abstract data boundary; MOD-001 deferral
- **分類**: 意図的後回し

### L4085: 21.7 Abstract export
- **機能ブロック**: `DAT`
- **notes**: MOD export/abstract data boundary; MOD-001 deferral
- **分類**: 意図的後回し

### L4091: 21.8 一部constructor公開
- **機能ブロック**: `DAT`
- **notes**: MOD export/abstract data boundary; MOD-001 deferral
- **分類**: 意図的後回し

### L4788: 評価文脈
- **機能ブロック**: `EVAL`
- **notes**: 形式証明/意図的後回し: Outcome/resume 経由; 形式 EC 文法は後回し
- **分類**: 意図的後回し

### L4867: 適合試験
- **機能ブロック**: `EVAL`
- **notes**: 意図的後回し: eval_tests + lang_kernel_suite; 版付き named EVAL corpus は後回し
- **分類**: 意図的後回し

### L5016: 解決後の最小Core
- **機能ブロック**: `EVAL`
- **notes**: 意図的後回し: BindingMap+SyntaxNodeIdは解決相; eval CoreのBindingId昇格は後回し
- **分類**: 意図的後回し

### L5597: `DD-BND-011`: 多相再帰の禁止
- **機能ブロック**: `BND`
- **notes**: polymorphic recursion out of v1; plan BND deferral
- **分類**: 意図的後回し

### L5834: `DD-BND-018`: local state identity
- **機能ブロック**: `BND`
- **notes**: 意図的後回し/形式証明: local-state effect + Cell Rc identity実装済; formal scope identity代数は後回し
- **分類**: 意図的後回し

### L5907: `DD-BND-020`: non-escaping callback
- **機能ブロック**: `BND`
- **notes**: non-escaping callback typing deferred
- **分類**: 意図的後回し

### L5974: `DD-BND-022`: one-shot resumptionと局所state
- **機能ブロック**: `BND`
- **notes**: multi-shot/escapeable state deferred; one-shot+alive check only
- **分類**: 意図的後回し

### L6003: `DD-BND-023`: escape可能な状態との分離
- **機能ブロック**: `BND`
- **notes**: multi-shot/escapeable state deferred; one-shot+alive check only
- **分類**: 意図的後回し

### L6035: `DD-BND-024`: 型一般化point
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6064: `DD-BND-025`: 構文的value restriction
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6108: `DD-BND-026`: effectful function valueの一般化
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6145: `DD-BND-027`: capability captureによる一般化禁止
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6176: `DD-BND-028`: 明示的`forall`注釈
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6223: `DD-BND-029`: 一般化される変数
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6251: `DD-BND-030`: 一般化されないmetavariable
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6267: `DD-BND-031`: `letrec`とvalue restriction
- **機能ブロック**: `BND`
- **notes**: full generalization / value-restriction regime deferred (plan BND-001)
- **分類**: 意図的後回し

### L6292: `DD-BND-032`: relaxed value restrictionの保留
- **機能ブロック**: `BND`
- **notes**: relaxed value restriction reserved
- **分類**: 意図的後回し

### L7047: 3.6 Interfaceおよびmanifest
- **機能ブロック**: `MAC`
- **notes**: interface/manifestはPKG/OPEN
- **分類**: 意図的後回し

### L7223: 6.6 0個以上の反復
- **機能ブロック**: `MAC`
- **notes**: 0個以上`...`はv1対象外
- **分類**: 意図的後回し

### L7258: 7.3 Templateの構文妥当性
- **機能ブロック**: `MAC`
- **notes**: 意図的後回し: def時 var/DAG検査あり（check_template_vars）; template完全構文妥当性の網羅は後回し
- **分類**: 意図的後回し

### L7510: 14.3 概念的なidentity
- **機能ブロック**: `MAC`
- **notes**: 意図的後回し: MacroSourceMap SyntaxNodeId + BindingMap use-sites; Core文字列 BindingId 統一は後回し
- **分類**: 意図的後回し

### L7637: 18. エラー診断
- **機能ブロック**: `MAC`
- **notes**: 意図的後回し: 主要 expand 診断あり; 深い span/provenance 帰属は後回し
- **分類**: 意図的後回し

### L7664: 18.5 不正な展開結果
- **機能ブロック**: `MAC`
- **notes**: 意図的後回し: 不正展開結果は後段エラー依存
- **分類**: 意図的後回し

### L7668: 18.6 型エラー
- **機能ブロック**: `MAC`
- **notes**: 意図的後回し: 型エラーのマクロ起点帰属なし
- **分類**: 意図的後回し

### L7683: 19.2 診断例
- **機能ブロック**: `MAC`
- **notes**: 意図的後回し: expand provenance 記録あり; typed diagnostic 付着は後回し
- **分類**: 意図的後回し

### L7720: 20.3 将来拡張
- **機能ブロック**: `MAC`
- **notes**: 将来のpkg公開
- **分類**: 意図的後回し

### L7724: 21. 適合試験
- **機能ブロック**: `MAC`
- **notes**: 意図的後回し: lang_kernel_suite TEST-LANG-MAC-001 等; 版付き全 MAC 試験は後回し
- **分類**: 意図的後回し

### L7900: 21.13 不適合試験 MAC-13：Interface
- **機能ブロック**: `MAC`
- **notes**: MAC-13 interface未（PKG）
- **分類**: 意図的後回し

### L7919: 22. 移管先OPEN・下位項目・状態
- **機能ブロック**: `MAC`
- **notes**: OPEN移管カタログ
- **分類**: 意図的後回し

### L8036: 13.6 `TYP-001` Gradual set-theoretic types
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し(TYP-ALG): Bounded Dynamic/casts/EffectRow/ROW fragment実装済; 完全集合論ソルバは後回し
- **分類**: 意図的後回し

### L8038: 概要・状態
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し(TYP-ALG): Dynamic/unify/EffectRow/cast fragment実装済; 完全集合論代数は後回し
- **分類**: 意図的後回し

### L8992: `DD-TYP-DYN-018`: cast provenance
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: CastProvenance struct 分離済; 完全 boundary ID 配線は後回し
- **分類**: 意図的後回し

### L9734: `DD-NAME-002`: namespace間の同綴り衝突
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: namespace 間同綴り衝突の完全警告は後回し
- **分類**: 意図的後回し

### L9840: 適合試験
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: lang_kernel_suite TYP+cast_tests; 版付き named TYP corpus は後回し
- **分類**: 意図的後回し

### L10176: namespace collision warning
- **機能ブロック**: `TYP`
- **notes**: namespace collision warnings deferred (NAME-002)
- **分類**: 意図的後回し

### L10340: 13.6.2 `TYP-ALG-001` Algorithmic型検査、semantic subtypingの判定範囲、型推論
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: 完全な semantic subtyping solver / worklist 代数（decide_subtype 三値 fragment は維持）
- **分類**: 意図的後回し

### L10489: `DD-TYP-ALG-003`: 診断分類
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: TypeDiagClass 全経路の emitter 配線（classify_decide の基本マップは維持）
- **分類**: 意図的後回し

### L10520: `annotation-required`
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: AnnotationRequired の診断 emitter 統合（ADT-08 文字列注釈要求は維持）
- **分類**: 意図的後回し

### L10556: `checker-resource-limit`
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: checker-resource-limit / solver step-budget emitter
- **分類**: 意図的後回し

### L10562: `unsupported-language-feature`
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: UnsupportedLanguageFeature 診断の利用点拡張
- **分類**: 意図的後回し

### L10639: `DD-TYP-ALG-006`: 決定的なsolver budget
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: 明示的 solver step budget（断片上は終了する）
- **分類**: 意図的後回し

### L10823: `DD-TYP-EFF-004`: effect-row polymorphism
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: EffectRow 全多相 / quantify（Fun EffectRow + infer_with_effects は維持）
- **分類**: 意図的後回し

### L10853: `DD-TYP-EFF-005`: handlerとrunnerによるeffect縮小
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: handle が residual から op 除去; return-clause 精密型付けは後回し
- **分類**: 意図的後回し

### L10878: `DD-TYP-EFF-006`: EffectRow alias
- **機能ブロック**: `TYP`
- **notes**: effect-row polymorphism / aliases not full
- **分類**: 意図的後回し

### L10915: `DD-TYP-EFF-007`: 注釈されたrequired effects
- **機能ブロック**: `TYP`
- **notes**: annotated required-effects surface deferred
- **分類**: 意図的後回し

### L10990: `DD-TYP-BOOL-002`: Surface negationの制限
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: unrestricted Surface negation の完全解法（Not 構文は維持）
- **分類**: 意図的後回し

### L11096: Function型
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: function intersection types / coherence（fixed-arity Fun は維持）
- **分類**: 意図的後回し

### L11201: `DD-TYP-FN-003`: function intersection
- **機能ブロック**: `TYP`
- **notes**: function intersection / coherence deferred past fixed-arity Fun
- **分類**: 意図的後回し

### L11227: `DD-TYP-FN-004`: function intersectionの適用可能性
- **機能ブロック**: `TYP`
- **notes**: function intersection applicability deferred
- **分類**: 意図的後回し

### L11277: `DD-TYP-FN-005`: branch specificity
- **機能ブロック**: `TYP`
- **notes**: branch specificity deferred with function intersection
- **分類**: 意図的後回し

### L11333: `DD-TYP-FN-006`: function intersectionのcoherence
- **機能ブロック**: `TYP`
- **notes**: function intersection coherence deferred
- **分類**: 意図的後回し

### L11339: 入力領域が互いに素
- **機能ブロック**: `TYP`
- **notes**: coherence case: disjoint domains
- **分類**: 意図的後回し

### L11347: 片方が他方を包含
- **機能ブロック**: `TYP`
- **notes**: coherence case: inclusion
- **分類**: 意図的後回し

### L11351: 入力領域が等価
- **機能ブロック**: `TYP`
- **notes**: coherence case: equivalent domains
- **分類**: 意図的後回し

### L11357: 入力領域が重なるが非比較
- **機能ブロック**: `TYP`
- **notes**: coherence case: overlapping incomparable
- **分類**: 意図的後回し

### L11373: `DD-TYP-FN-007`: union引数とdispatch
- **機能ブロック**: `TYP`
- **notes**: union-arg dispatch deferred with function intersection
- **分類**: 意図的後回し

### L11420: Row-polymorphic record
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し(ROW-001): OpenRecord 行多相; multi-tail は後回し
- **分類**: 意図的後回し

### L11591: `DD-TYP-ROW-008`: recordのBoolean演算
- **機能ブロック**: `TYP`
- **notes**: record Boolean combination complete fragment deferred
- **分類**: 意図的後回し

### L11626: Recursive data type
- **機能ブロック**: `TYP`
- **notes**: equi-recursive / contractiveness checker deferred
- **分類**: 意図的後回し

### L11628: `DD-TYP-REC-001`: recursive data type
- **機能ブロック**: `TYP`
- **notes**: recursive data type equi-checker deferred (DAT surface exists)
- **分類**: 意図的後回し

### L11644: `DD-TYP-REC-002`: equi-recursiveな利用者意味論
- **機能ブロック**: `TYP`
- **notes**: equi-recursive user semantics deferred
- **分類**: 意図的後回し

### L11664: `DD-TYP-REC-003`: contractiveness
- **機能ブロック**: `TYP`
- **notes**: contractiveness checker deferred
- **分類**: 意図的後回し

### L11699: `DD-TYP-REC-004`: strict positivity
- **機能ブロック**: `TYP`
- **notes**: strict positivity: DAT group positivity only (partial elsewhere)
- **分類**: 意図的後回し

### L11727: `DD-TYP-REC-005`: regularity
- **機能ブロック**: `TYP`
- **notes**: regularity checker deferred
- **分類**: 意図的後回し

### L11758: `DD-TYP-REC-006`: base constructor
- **機能ブロック**: `TYP`
- **notes**: base constructor discipline deferred
- **分類**: 意図的後回し

### L11786: `DD-TYP-REC-007`: recursive dataの完全判定範囲
- **機能ブロック**: `TYP`
- **notes**: full recursive decide fragment deferred
- **分類**: 意図的後回し

### L12208: `DD-TYP-FRAG-003`: C — 注釈を要求し得るfragment
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: AnnotationRequired class 予約; incompleteness 経路は薄い
- **分類**: 意図的後回し

### L12266: 共通constraint worklist
- **機能ブロック**: `TYP`
- **notes**: shared constraint worklist deferred; unify is direct
- **分類**: 意図的後回し

### L12268: `DD-TYP-SOLVER-001`: shared constraint worklist
- **機能ブロック**: `TYP`
- **notes**: shared constraint worklist deferred
- **分類**: 意図的後回し

### L12303: `DD-TYP-SOLVER-002`: solver間のconstraint生成
- **機能ブロック**: `TYP`
- **notes**: multi-solver constraint generation deferred
- **分類**: 意図的後回し

### L12382: `DD-TYP-SOLVER-003`: constraint処理の優先度
- **機能ブロック**: `TYP`
- **notes**: constraint priority schedule deferred
- **分類**: 意図的後回し

### L12409: `DD-TYP-SOLVER-004`: canonicalizationとmemoization
- **機能ブロック**: `TYP`
- **notes**: canonicalization/memoization deferred
- **分類**: 意図的後回し

### L12436: `DD-TYP-SOLVER-005`: solver終了状態
- **機能ブロック**: `TYP`
- **notes**: solver end-states folded into CheckError for now
- **分類**: 意図的後回し

### L12466: Resource limit
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: CheckerResourceLimit / resource-budget 実行経路
- **分類**: 意図的後回し

### L12472: `DD-TYP-SOLVER-006`: cast insertionとgeneralizationの順序
- **機能ブロック**: `TYP`
- **notes**: cast insertion vs generalization ordering deferred
- **分類**: 意図的後回し

### L12494: 型検査器の概念pipeline
- **機能ブロック**: `TYP`
- **notes**: full checker pipeline schedule deferred
- **分類**: 意図的後回し

### L12496: `DD-TYP-SOLVER-007`: checkerの全体処理順序
- **機能ブロック**: `TYP`
- **notes**: overall checker processing order deferred
- **分類**: 意図的後回し

### L12542: 適合試験
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: TYP + cast_tests; 版付き TYP-ALG corpus は後回し
- **分類**: 意図的後回し

### L12574: 診断分類
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: TypeDiagClass 全診断 emitter（taxonomy + classify_decide は維持）
- **分類**: 意図的後回し

### L12656: EffectRow polymorphism
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: EffectRow polymorphism 適合試験 / 量化（EffectRow 本体は維持）
- **分類**: 意図的後回し

### L12722: Function subtyping
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: 基本 Fun 部分型; 完全 DD suite は後回し
- **分類**: 意図的後回し

### L12744: Function intersection coherence
- **機能ブロック**: `TYP`
- **notes**: function intersection coherence suite deferred
- **分類**: 意図的後回し

### L12770: 非比較なbranch overlap
- **機能ブロック**: `TYP`
- **notes**: incomparable branch overlap suite deferred
- **分類**: 意図的後回し

### L12786: Union引数の暗黙dispatch禁止
- **機能ブロック**: `TYP`
- **notes**: union-arg implicit dispatch forbidden suite deferred
- **分類**: 意図的後回し

### L12893: Recursive data
- **機能ブロック**: `TYP`
- **notes**: recursive data conformance suite deferred
- **分類**: 意図的後回し

### L12916: 非contractive再帰
- **機能ブロック**: `TYP`
- **notes**: non-contractive recursion suite deferred
- **分類**: 意図的後回し

### L12930: Non-regular recursion
- **機能ブロック**: `TYP`
- **notes**: non-regular recursion suite deferred
- **分類**: 意図的後回し

### L12948: Bidirectional checking
- **機能ブロック**: `TYP`
- **notes**: 意図的後回し: Fun/Record annotation checking あり; 完全 BIDI corpus は後回し
- **分類**: 意図的後回し

### L13022: Solver determinism
- **機能ブロック**: `TYP`
- **notes**: solver determinism suite deferred with worklist
- **分類**: 意図的後回し

### L13203: 13.7 `ROW-001` Row-polymorphic records
- **機能ブロック**: `ROW`
- **notes**: 意図的後回し(ROW-001): closed+OpenRecord+Lacks実装済; multi-tailは後回し
- **分類**: 意図的後回し

### L13226: 13.8 `EFF-001` Algebraic effects and handlers
- **機能ブロック**: `EFF`
- **notes**: 意図的後回し: deep one-shot + ambient + with/handler実装済; multi-shot/return句は後回し
- **分類**: 意図的後回し

### L13230: 状態
- **機能ブロック**: `EFF`
- **notes**: 意図的後回し: deep one-shot + ambient + with/handler; multi-shot/return句は後回し
- **分類**: 意図的後回し

### L13351: `DD-EFF-003`: resumptionの型とscope
- **機能ブロック**: `EFF`
- **notes**: 意図的後回し: one-shot resume 値あり; resume 精密型付けは interim Dynamic
- **分類**: 意図的後回し

### L13579: `DD-EFF-008`: return clause
- **機能ブロック**: `EFF`
- **notes**: return-clause / Handler<L,A,B,H> typing deferred (plan)
- **分類**: 意図的後回し

### L13634: `DD-EFF-009`: handler単位の結果型変換
- **機能ブロック**: `EFF`
- **notes**: return-clause / Handler<L,A,B,H> typing deferred (plan)
- **分類**: 意図的後回し

### L13681: `DD-EFF-010`: handler valueのrank-1多相性
- **機能ブロック**: `EFF`
- **notes**: 意図的後回し: HandlerValue は check 上 Dynamic; rank-1 多相 handler は後回し
- **分類**: 意図的後回し

### L13875: Named/scoped effect instance
- **機能ブロック**: `EFF`
- **notes**: named/scoped effect instances not in kernel
- **分類**: 意図的後回し

### L13931: `DD-EFF-014`: EffectRowの意味
- **機能ブロック**: `EFF`
- **notes**: 意図的後回し: thin EffectRow; handler 精密型付けは後回し
- **分類**: 意図的後回し

### L14013: `DD-EFF-015`: ambient effect rowと制約生成
- **機能ブロック**: `EFF`
- **notes**: 意図的後回し: ambient EffectRow 制約生成は薄い
- **分類**: 意図的後回し

### L14109: `DD-EFF-016`: handlerの型付け骨格
- **機能ブロック**: `EFF`
- **notes**: 意図的後回し: handler 型付け骨格は interim（op+fn）
- **分類**: 意図的後回し

### L14371: `DD-EFF-020`: cleanupとの接続要件
- **機能ブロック**: `EFF`
- **notes**: cleanup/finalization → ERR; plan deferral
- **分類**: 意図的後回し

### L14412: 13.9 `MOD-001` モジュール・シグネチャ・Functor・分割コンパイル
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: outer unit + import/link + .rpi実装済; signatures/functorsは後回し
- **分類**: 意図的後回し

### L14434: DD-001.2 中心的な決定
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: import as/only/rename+qualified + .rpi境界実装済; functors/signaturesは後回し
- **分類**: 意図的後回し

### L14477: 0.2 本項目が直接定めないもの
- **機能ブロック**: `MOD`
- **notes**: PKG/MAC/KER/recursive/1st-class/generative → OPEN
- **分類**: 意図的後回し

### L14573: 2.4 Path変更
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: path-rename / remapping API
- **分類**: 意図的後回し

### L14585: 3. 下位モジュール
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)
- **分類**: 意図的後回し

### L14586: 3.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)
- **分類**: 意図的後回し

### L14603: 3.2 下位モジュールのpath
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)
- **分類**: 意図的後回し

### L14609: 3.3 下位モジュールの外部ファイル化
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)
- **分類**: 意図的後回し

### L14620: 3.4 Module body
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)
- **分類**: 意図的後回し

### L14636: 4. 下位モジュールのscopeと純粋性
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested-module scope N/A until nested modules
- **分類**: 意図的後回し

### L14655: 4.2 親scopeの参照
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested parent-scope N/A until nested modules
- **分類**: 意図的後回し

### L14667: 4.3 後方参照
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: nested forward-ref N/A until nested modules
- **分類**: 意図的後回し

### L14685: 4.5 Top-level effect
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: unit body は Core として評価; module-level effect gate は未導入
- **分類**: 意図的後回し

### L14727: 5.3 内部表現
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: qualified binder は `alias/export` 文字列; formal ModuleId path IR は後回し
- **分類**: 意図的後回し

### L14737: 5.4 .
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: dot-qualified module refs; slash paths are primary
- **分類**: 意図的後回し

### L14811: 6.7 自動再公開
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: auto re-export of imports not in bind skeleton
- **分類**: 意図的後回し

### L14821: 7. Importと正式identity
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: alias は局所接頭辞; formal ModuleId identity 層は後回し
- **分類**: 意図的後回し

### L14870: 8.2 Wrapper
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14879: 8.3 .rpi内のimport
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14897: 8.5 Internal module
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14903: 8.6 Script／実行entry
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14907: 9. 推論シグネチャと抽象化境界
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14918: 9.3 Interface追加
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14924: 10. .rpiの値仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14925: 10.1 type
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14938: 10.2 実装
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14946: 10.3 型注釈の省略
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14952: 10.4 実装側にも型がある場合
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14958: 11. 型の公開方法
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14959: 11.1 抽象型仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14977: 11.2 Parameter付き抽象型
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14986: 11.3 Constructor公開data仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L14998: 11.4 実装との一致
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15011: 11.5 type-alias
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15025: 12. 名前付きシグネチャ
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15026: 12.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15033: 12.2 日本語上の意味
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15042: 12.3 .rpiとの違い
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15049: 12.4 Signature identity
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15055: 13. シグネチャ指定
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15056: 13.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15069: 13.2 適合判定
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15075: 13.3 追加構成要素
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15081: 13.4 抽象型identity
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15089: 13.5 実装内部のalias
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15103: 14. 下位モジュール仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15104: 14.1 名前付きシグネチャ
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15119: 14.2 インラインシグネチャ
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15128: 14.3 下位モジュール型の参照
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15134: 14.4 抽象型の独立性
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15142: 14.5 非公開型の漏出
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15151: 15. シグネチャの精緻化と型共有
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15152: 15.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15156: 15.2 抽象型の具体化
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15163: 15.3 別モジュールとの型共有
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15174: 15.4 日本語用語
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15185: 15.5 許可される精緻化
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15189: 15.6 禁止される変更
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15198: 15.7 Parameter付き型constructor
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15205: 15.8 下位モジュール内の型
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; implemented-features)
- **分類**: 意図的後回し

### L15212: 16. Functor
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15213: 16.1 概念
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15223: 16.2 基本構文
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15233: 16.3 Parameter
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15243: 16.4 結果シグネチャ
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15249: 16.5 Fixed arity
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15255: 16.6 通常値ではない
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15261: 17. Functor適用
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15262: 17.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15269: 17.2 複数引数
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15276: 17.3 名前付きモジュールのみ
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15282: 17.4 適用結果の再利用
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15294: 18. 適用的Functor
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15295: 18.1 基本規則
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15305: 18.2 同一入力
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15319: 18.3 異なる入力
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15333: 18.4 Aliasの影響
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15339: 18.5 生成的Functor
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / implemented-features)
- **分類**: 意図的後回し

### L15343: 19. 再公開とシグネチャ合成
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15344: 19.1 module-alias
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15353: 19.2 re-export
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15362: 19.3 Identity
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15368: 19.4 Dataの原子性
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15379: 19.5 include
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15388: 19.6 衝突
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15394: 19.7 実装moduleのinclude
- **機能ブロック**: `MOD`
- **notes**: module-alias/re-export/include not in bind skeleton
- **分類**: 意図的後回し

### L15400: 20. 正式identity
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: reciplexa-identity に opaque ID; MOD metadata への正式配線は後回し
- **分類**: 意図的後回し

### L15441: 20.5 TypeId
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: TypeId opaque は identity crate; MOD metadata 未配線
- **分類**: 意図的後回し

### L15447: 20.6 ConstructorId
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: ConstructorId opaque は identity crate; MOD metadata 未配線
- **分類**: 意図的後回し

### L15473: 20.9 Functor適用結果
- **機能ブロック**: `MOD`
- **notes**: functor apply identity N/A until functors
- **分類**: 意図的後回し

### L15492: 21. 分割コンパイルと適合試験
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: InterfaceHash / compiled interface metadata
- **分類**: 意図的後回し

### L15493: 21.1 Interface metadata
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: .rpi → interface metadata emitter
- **分類**: 意図的後回し

### L15523: 21.3 InterfaceHash
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: InterfaceHash absent
- **分類**: 意図的後回し

### L15530: 21.4 Hashに含めるもの
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: hash inputs N/A until InterfaceHash
- **分類**: 意図的後回し

### L15548: 21.6 Documentation hash
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: DocumentationHash absent
- **分類**: 意図的後回し

### L15552: 21.7 再コンパイル
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: incremental recompile by InterfaceHash
- **分類**: 意図的後回し

### L15594: 21.11 適合試験 MOD-03
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: MOD-03 conformance corpus
- **分類**: 意図的後回し

### L15621: 21.12 不適合試験 MOD-04
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: MOD-04 negative suite
- **分類**: 意図的後回し

### L15650: 21.13 適合試験 MOD-05
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: MOD-05 suite
- **分類**: 意図的後回し

### L15677: 21.14 適合試験 MOD-06
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: MOD-06 suite
- **分類**: 意図的後回し

### L15689: 21.15 不適合試験 MOD-07
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: MOD-07 suite
- **分類**: 意図的後回し

### L15699: 21.16 適合試験 MOD-08
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: MOD-08 suite
- **分類**: 意図的後回し

### L15708: 21.17 不適合試験 MOD-09
- **機能ブロック**: `MOD`
- **notes**: 意図的後回し: MOD-09 suite
- **分類**: 意図的後回し

### L16140: 4.3 表示名
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16149: 5. パッケージversion
- **機能ブロック**: `PKG`
- **notes**: 意図的後回し: version フィールド parse; 完全 semver 代数は後回し
- **分類**: 意図的後回し

### L16159: 5.2 Version要素
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16165: 5.3 互換性の一般原則
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16175: 5.4 Breaking changeの例
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16188: 5.5 Pre-release
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16229: 6.4 Interface対応
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16242: 6.5 Root数
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16255: 6.6 Root pathの制限
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16287: 7.3 内部モジュール
- **機能ブロック**: `PKG`
- **notes**: 意図的後回し: 非公開 cross-package import拒否済; 同一package内モジュールグラフは後回し
- **分類**: 意図的後回し

### L16297: 7.4 Internal moduleの.rpi
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16307: 7.5 Inline下位モジュール
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16313: 7.6 自動公開
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16342: 8.3 .rpi
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16348: 8.4 main
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16360: 8.6 実行契約
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16376: 9.2 必要な値
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16384: 9.3 単一ファイル制限
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16398: 9.5 Public API
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16428: 10.3 正式identity
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16443: 10.5 Aliasの重複
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16450: 11. Version constraint
- **機能ブロック**: `PKG`
- **notes**: 意図的後回し: 完全な version constraint solver（正確一致 / * は維持）
- **分類**: 意図的後回し

### L16469: 11.2 範囲指定
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16475: 11.3 初期演算子集合
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16485: 11.4 条件の結合
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16494: 11.5 Pre-release
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16499: 12.1 既定Registry
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16519: 12.3 Local packageの検証
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16525: 12.4 Supported source
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16533: 12.5 Git／URL
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16537: 12.6 Sourceの排他性
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16541: 13. Development dependency
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16542: 13.1 構文
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16548: 13.2 用途
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16559: 13.3 Public APIへの漏出
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16568: 13.4 Optional dependency
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16574: 14. Public dependency
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16575: 14.1 定義
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16582: 14.2 自動導出
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16588: 14.3 用途
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16604: 15.2 初回解決
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16610: 15.3 Pre-release
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16614: 15.4 統合
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16618: 15.5 分割
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16628: 16. 同一パッケージの複数version
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16629: 16.1 基本方針
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16639: 16.2 型identity
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16647: 16.3 直接依存での明示
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16663: 16.4 単一instance制約
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16681: 17.3 通常build
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16687: 17.4 初回build
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16704: 17.6 更新
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16710: 17.7 部分更新
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16717: 18.1 Package node
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16730: 18.2 Registry package
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16742: 18.4 Version control
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16748: 18.5 Offline build
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16760: 19. Package identityとcontent hash
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16761: 19.1 論理identityと内容identity
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16771: 19.2 Registry package
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16783: 19.3 Workspace／local package
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16793: 19.4 Content hash対象
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16813: 20. ワークスペース
- **機能ブロック**: `PKG`
- **notes**: 意図的後回し: workspace member discovery/build（workspace.rpxm stub parse は維持）
- **分類**: 意図的後回し

### L16839: 20.4 Member
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16843: 20.5 Member path
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16849: 20.6 Memberの外部配置
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16855: 20.7 Package名重複
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16859: 20.8 Nested workspace
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16865: 21. Workspace依存・リソース・適合試験
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L16872: 21.2 Member単独build
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16876: 21.3 Workspace member優先
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16884: 21.4 明示source
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16895: 21.5 Member version
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16899: 21.6 Member依存graph
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16905: 21.7 Workspace外path dependency
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16911: 21.8 Workspace identity
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16917: 21.9 Resource root
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16928: 21.10 Resource一覧
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16945: 21.11 Resource path
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16959: 21.12 Symbolic link
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16963: 21.13 Resource identity
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16973: 21.14 Resource参照
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16982: 21.15 Pure／effectfulの区別
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16989: 21.16 Resourceの外部公開
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L16995: 21.17 Resource hash
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L17001: 21.18 Generated resource
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L17005: 21.19 Test fixture
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
- **分類**: 意図的後回し

### L17009: 21.20 適合試験 PKG-01
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17028: 21.21 不適合試験 PKG-02
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17039: 21.22 適合試験 PKG-03
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17049: 21.23 不適合試験 PKG-04
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17059: 21.24 適合試験 PKG-05
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17069: 21.25 適合試験 PKG-06
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17082: 21.26 適合試験 PKG-07
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17094: 21.27 不適合試験 PKG-08
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17113: 21.28 適合試験 PKG-09
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17127: 21.29 不適合試験 PKG-10
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17137: 21.30 適合試験 PKG-11
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17147: 21.31 不適合試験 PKG-12
- **機能ブロック**: `PKG`
- **notes**: PKG conformance suite deferred / not wired
- **分類**: 意図的後回し

### L17157: 22. 移管先OPEN・下位項目・状態
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 意図的後回し

### L17232: 22.8 下位項目
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
- **分類**: 意図的後回し

### L17299: 13.10.1 `KER-001` Rust kernelとforeign primitive境界
- **機能ブロック**: `KER`
- **notes**: 意図的後回し: BuiltinOp + EffectHost; 完全 Rust/FFI ABI は後回し
- **分類**: 意図的後回し

### L17301: 概要・状態
- **機能ブロック**: `KER`
- **notes**: 意図的後回し: kernel ops は eval/check; TEST-KER-*/foreign validator は後回し
- **分類**: 意図的後回し

### L17326: 13.10.2 `RSC-001` Resource、I/O、host-handler境界
- **機能ブロック**: `RSC`
- **notes**: 意図的後回し: MemoryFsHost read-file/write-file実装済; 豊かなcatalog/path safetyは後回し
- **分類**: 意図的後回し

### L17328: 概要・状態
- **機能ブロック**: `RSC`
- **notes**: 意図的後回し: in-memory host は試験用; resolve-font/load-image 等は言語核外
- **分類**: 意図的後回し

### L18860: 13.12 `IR-001` Layered visual/motion/render IR
- **機能ブロック**: `IR`
- **notes**: 意図的後回し: backends+motion+view実装済; 完全 layered IR schemaは後回し
- **分類**: 意図的後回し

### L18875: SurfaceとArtifact
- **機能ブロック**: `IR`
- **notes**: 意図的後回し: scene/document surface あり; Artifact 代数は後回し
- **分類**: 意図的後回し

### L18889: RenderIR node algebra
- **機能ブロック**: `IR`
- **notes**: 意図的後回し: pdf/svg/pptx/view 原始あり; 完全 RenderIR node 代数は後回し
- **分類**: 意図的後回し

### L18949: Backend lowering
- **機能ブロック**: `IR`
- **notes**: 意図的後回し: pdf/svg/pptx lowering あり; AE/edit-preserving path は後回し
- **分類**: 意図的後回し

### L18981: テスト
- **機能ブロック**: `IR`
- **notes**: 意図的後回し: phase12_motion TEST-IR-007; 版付き IR-001..009 validator suite は後回し
- **分類**: 意図的後回し

### L19275: 5.4 Handlerの結果型
- **機能ブロック**: `ERR`
- **notes**: 意図的後回し: handler 結果型は共有 handle infer（interim）
- **分類**: 意図的後回し

### L20399: 30. 移管先OPEN
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
- **分類**: 意図的後回し

### L20476: 13.14 `MEM-001` Perceusメモリ管理・スコープ付きリソース・継続・メモリ予算
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し: reciplexa-mem Perceus IR（dup/drop/reuse/verify/lower）実装済; 既定evalのPerceus切替は後回し
- **分類**: 意図的後回し

### L20956: 10. var
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し: LocalVar escape check は eval; Perceus-var cell 統合は後回し
- **分類**: 意図的後回し

### L20966: 10.2 物理表現
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し: var 物理セルの独立 ownership クラスは mem IR 未
- **分類**: 意図的後回し

### L21047: 13. Continuation
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し: Resume/DiscardCont/Raise はあるが完全継続捕捉モデルは後回し
- **分類**: 意図的後回し

### L21728: 33. 適合試験
- **機能ブロック**: `MEM`
- **notes**: 意図的後回し: phase6_mem / mem tests あり; 版付きフル MEM-001 suite は後回し
- **分類**: 意図的後回し

### L21906: 34. 移管先OPEN
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
- **分類**: 意図的後回し

### L22074: 14.1 全体EBNF（未完成）
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: skeleton EBNF; 実文法は reciplexa-syntax
- **分類**: 意図的後回し

### L22128: 14.3 予約語
- **機能ブロック**: `TST`
- **notes**: 意図的後回し(OPEN): is_reserved_special_form/keywords あり; 予約語政策の最終確定は後回し
- **分類**: 意図的後回し

### L22134: 14.4 糖衣とCore
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: elaborate が主要糖衣をカバー; 表形式の完全対応表は後回し
- **分類**: 意図的後回し

### L22148: 15.1 Kind
- **機能ブロック**: `TST`
- **notes**: 形式証明/意図的後回し: kinds は CoreType/rows に暗黙; Module/Signature kind は後回し
- **分類**: 意図的後回し

### L22156: 15.2 共通判断
- **機能ブロック**: `TST`
- **notes**: 形式証明/意図的後回し: infer/check 判断は check.rs; module sig 判断は後回し
- **分類**: 意図的後回し

### L22168: 15.3 基本規則
- **機能ブロック**: `TST`
- **notes**: 形式証明/意図的後回し: T-VAR/if/record 等は checker; 規則全集は後回し
- **分類**: 意図的後回し

### L22222: 15.4 一般化
- **機能ブロック**: `TST`
- **notes**: 形式証明/意図的後回し: let 一般化は light; value restriction 完全形は後回し
- **分類**: 意図的後回し

### L22228: 15.5 Subtypingと制約解決
- **機能ブロック**: `TST`
- **notes**: 形式証明/意図的後回し: unify+subtype stubs; 完全制約ソルバは後回し
- **分類**: 意図的後回し

### L22236: 15.6 Module境界
- **機能ブロック**: `TST`
- **notes**: module signature checking deferred with MOD signatures
- **分類**: 意図的後回し

### L22267: 16.3 効果伝播
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: deep one-shot handlers; multi-shot 効果伝播は後回し
- **分類**: 意図的後回し

### L22384: 実装アーキテクチャ
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: vertical slice pipeline あり; 最終アーキテクチャ文書化は後回し
- **分類**: 意図的後回し

### L22386: 20.1 最終目標パイプライン
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: bytes→CST→elaborate→check→eval→lower あり; 位相隙間の閉包は後回し
- **分類**: 意図的後回し

### L22404: 20.2 必要データ構造
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: 主要構造あり; ModuleEnv/Typed Core 完全形は後回し
- **分類**: 意図的後回し

### L22460: 21.1 型検査器要件
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: typecheck_language_sourceあり; ModuleEnv/imported sigs完全形は後回し
- **分類**: 意図的後回し

### L22529: 22.5 Property/differential/fuzz
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: property/differential/fuzz harness (harden fuzz hook only)
- **分類**: 意図的後回し

### L22569: 段階2: 仕様が明確になった
- **機能ブロック**: `TST`
- **notes**: 形式プロセス/意図的後回し: 仕様は大部記述済; 文法/Core 穴の完全閉鎖は後回し
- **分類**: 意図的後回し

### L22585: 段階4: 適合試験を通過
- **機能ブロック**: `TST`
- **notes**: 形式プロセス/意図的後回し: 主要適合試験は通過; 版付きフル suite gate は後回し
- **分類**: 意図的後回し

### L22593: 段階5: 統合試験を通過
- **機能ブロック**: `TST`
- **notes**: multi-package/GUI/resource integration post-PKG
- **分類**: 意図的後回し

### L22601: 段階6: 差分・生成・fuzzを通過
- **機能ブロック**: `TST`
- **notes**: 意図的後回し: stage-6 generators/differential/fuzz gate
- **分類**: 意図的後回し

### L22618: 段階8: 実装と形式仕様の対応を確認
- **機能ブロック**: `TST`
- **notes**: 形式証明/意図的後回し: crate↔spec 対応は informal; 版付き correspondence report は後回し
- **分類**: 意図的後回し

---

## 依存待ち（257）

### L1986: 17.2 Coreとpackageの分担
- **機能ブロック**: `SYN`
- **notes**: 依存待ち(PKG): Core reader vs package constructors (circle/space/…)- notes: CST readerあり; package分担は暫定
- **分類**: 依存待ち

### L2129: 17.7 糖衣展開
- **機能ブロック**: `SYN`
- **notes**: 依存待ち(PKG): @name[…]/{} sugar expands to package constructors- notes: document/macro prototype; 完全sugar未
- **分類**: 依存待ち

### L8609: `DD-TYP-DYN-010`: foreign値のdynamic導入
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(KER/foreign): import-dynamic / ForeignValue boundary
- **分類**: 依存待ち

### L8660: `DD-TYP-DYN-011`: decoderとgradual foreign boundaryの分離
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(KER/foreign): decoder vs gradual foreign boundary separation
- **分類**: 依存待ち

### L8667: Static decoder
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(KER/foreign): static decoder Result<S, decode-error>
- **分類**: 依存待ち

### L8676: Gradual foreign boundary
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(KER/foreign): import-dynamic foreign boundary
- **分類**: 依存待ち

### L9054: `DD-TYP-DYN-020`: opaque abstract typeのcast
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(MOD/opaque): NominalCheck abstract-type sealing
- **分類**: 依存待ち

### L9301: `DD-TYP-DYN-026`: dynamic境界を通れない制御値
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(EFF/KER): control values (resume/handler/capability) barred from dynamic
- **分類**: 依存待ち

### L9339: `DD-TYP-DYN-027`: polymorphismとdynamic境界
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(DAT/poly): polymorphism × dynamic boundary
- **分類**: 依存待ち

### L9988: foreign ingress
- **機能ブロック**: `TYP`
- **notes**: 依存待ち(KER/foreign): foreign ingress
- **分類**: 依存待ち

### L10115: polymorphic value boundary
- **機能ブロック**: `TYP`
- **notes**: 依存待ち: polymorphic value dynamic boundary
- **分類**: 依存待ち

### L10147: dynamicからforall
- **機能ブロック**: `TYP`
- **notes**: 依存待ち: dynamic→forall boundary
- **分類**: 依存待ち

### L17355: 13.11 `EDT-001` 編集スナップショット・トランザクション・競合・由来情報
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): DocumentSnapshot/Transaction は言語側骨格あり; full GUI reconciliation/conflict/undo/codec は後回し
- **分類**: 依存待ち

### L17401: 0.1 本項目が扱う編集
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17477: 1.3 現在文書
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17505: 2.2 所有と参照の分離
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17524: 2.3 共有
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17548: 3. 文書treeの不変条件
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17552: 1. RootNodeIdがnode storeに存在する
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17553: 2. Rootは親を持たない
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17554: 3. Root以外の全ノードはちょうど一つの親を持つ
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17555: 4. 所有edgeにcycleがない
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17556: 5. 同じ親のchildren列内に同一NodeIdが重複しない
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17557: 6. 全ノードが同じDocumentIdに所属する
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17558: 7. 全ノードがRootから到達可能である
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17559: 8. 必須propertyが存在する
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17560: 9. Property値がNode kindのschemaに適合する
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17561: 10. 強いNodeId参照が有効な対象を指す
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17563: 3.1 到達不能ノード
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17578: 4. 識別子
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17579: 4.1 識別子の種類
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17653: 4.5 TransactionId
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17671: 5. 保存・複製・Fork
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17672: 5.1 通常保存
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17683: 5.2 Save As
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17693: 5.3 Duplicate／Fork
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17708: 5.4 内部参照の複製
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17718: 5.5 文書間参照
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17724: 6. Revision
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17725: 6.1 直線的な履歴
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17738: 6.2 Commitの直列化
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17772: 6.4 保存後のrevision
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17780: 7. Snapshotの保持
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17781: 7.1 不変性
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17792: 7.2 過去版の永久取得
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17798: 7.3 履歴の種類
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17811: 8. 編集トランザクション
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17812: 8.1 概念構造
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17822: 8.2 不変の第一級値
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17856: 9. 編集Operation
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17900: 9.3 SetProperty
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): SetLayout/SetText のみ; 汎用 SetProperty は GUI EDT
- **分類**: 依存待ち

### L17950: 9.5 RemoveChild
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L17999: 10.2 予約済みID
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18005: 10.3 Copy
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18011: 11. 適用前条件
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): UnknownNode/InvalidParent 等の基本検査あり; 完全な適用前条件プロトコルは GUI
- **分類**: 依存待ち

### L18037: 11.3 Base revision
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18043: 12. Stale transaction
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18044: 12.1 定義
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18048: 12.2 分類
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18063: 12.3 保守的な再適用
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18069: 12.4 無関係な変更
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18073: 13. 競合
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18074: 13.1 基本分類
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18091: 13.2 競合と不正トランザクション
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18109: 13.3 競合の収集
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18115: 13.4 自動併合
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18135: 14. 適用手順
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): 簡略 apply; 仕様9段プロトコルは GUI reconciliation
- **分類**: 依存待ち

### L18139: 1. TransactionIdを確認
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18140: 2. DocumentIdを確認
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18141: 3. Base revisionを比較
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18142: 4. Transaction-level preconditionを検査
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18143: 5. 現在snapshotから作業状態を作成
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): in-place mutate+rollback; 作業 snapshot 複製モデルは GUI
- **分類**: 依存待ち

### L18145: 7. 文書全体の不変条件を検査
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18147: 9. 新revisionとUndo情報を生成
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18185: 15.5 AlreadyApplied
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18191: 15.6 Transaction content hash
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18205: 16. UndoとRedo
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18206: 16.1 新revision
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18218: 16.2 Undo情報
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18234: 16.3 UndoToken
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18241: 16.4 Undo競合
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18247: 16.5 Redo
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18251: 16.6 履歴保持
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18265: 17.1 定義
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18278: 17.2 Optional metadata
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18296: 17.4 種類
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18305: 17.5 UserCreated
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18312: 17.6 SourceGenerated
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18327: 17.7 MacroGenerated
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18336: 17.8 Imported
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18342: 17.9 Copied
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18348: 17.10 Derived
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18362: 18. 派生ノードと逆編集
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18363: 18.1 編集可能性
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18379: 18.2 Provenanceと逆編集
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(EDT): reverse-edit from Provenance is GUI/EDT, not Part II language
- **分類**: 依存待ち

### L18385: 18.3 逆編集結果
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18392: 18.4 自動選択
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18396: 18.5 逆写像不能
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18409: 18.6 Stale provenance
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18415: 19. 派生ノードのID継承
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18416: 19.1 DerivationKey
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18426: 19.2 曖昧な対応
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18432: 19.3 NodeIdとの違い
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18436: 20. Provenanceの安全性
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(EDT): Provenance safety/export policy is EDT/security layer
- **分類**: 依存待ち

### L18437: 20.1 真正性
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18443: 20.2 Privacy
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18454: 20.3 書換え
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18460: 21. 公開API階層
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): Rust document API; RPX 第一級 EDT API は後回し
- **分類**: 依存待ち

### L18461: 21.1 純粋な第一級値
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): DocumentEdit 値はあるが RPX 第一級化は GUI/EDT
- **分類**: 依存待ち

### L18474: 21.2 状態付きhandle
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): DocumentSnapshot handle は GUI 経路
- **分類**: 依存待ち

### L18478: 21.3 抽象型
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18491: 21.4 Constructor付き公開data
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18500: 22. 高水準APIと低水準API
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18501: 22.1 高水準API
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): source_sync GUI edits
- **分類**: 依存待ち

### L18515: 22.2 低水準API
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): DocumentTransaction 低水準 API は GUI 層
- **分類**: 依存待ち

### L18529: 23. Pure処理とEffectful処理
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): doc tx 純関数性 vs host I/O の完全分離は GUI/RSC 境界
- **分類**: 依存待ち

### L18530: 23.1 Pure処理
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18538: 23.2 Effectful処理
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): source rewrite side effects via GUI sync
- **分類**: 依存待ち

### L18559: 24. 競合・不正・実行障害の分離
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): Outcome vs Error 骨格あり; 競合型の完全分離は GUI
- **分類**: 依存待ち

### L18573: 24.3 実行障害
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18590: 25. Undo履歴・Transaction履歴
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18591: 25.1 有限保持
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18600: 25.2 AlreadyApplied保証
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18606: 25.3 Undo不可
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18612: 26. 永続化
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18613: 26.1 標準保存
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18624: 26.2 編集履歴
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18630: 26.3 Version付きcodec
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18638: 27. 適合試験
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18835: 29. 最終状態
- **機能ブロック**: `EDT`
- **notes**: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)
- **分類**: 依存待ち

### L18993: 13.13 `ERR-001` 通常の失敗・Failure effect・後始末・Defect・最上位実行境界
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): raise/handle/or-raise/as-result + Never + DYN-005実装済; bracket/cleanup/defect境界は後回し
- **分類**: 依存待ち

### L19114: 2. resultおよび専用結果型
- **機能ブロック**: `ERR`
- **notes**: dedicated ERR result API beyond DAT result 依存待ち(第III部); user data result path ok
- **分類**: 依存待ち

### L19115: 2.1 用途
- **機能ブロック**: `ERR`
- **notes**: 専用結果型 API 依存待ち(第III部); DAT result/option usable
- **分類**: 依存待ち

### L19135: 2.2 専用結果型
- **機能ブロック**: `ERR`
- **notes**: dedicated ERR result constructors 依存待ち(第III部)
- **分類**: 依存待ち

### L19150: 2.3 複数errorの収集
- **機能ブロック**: `ERR`
- **notes**: multi-error collect language primitive 依存待ち(第III部)/OPEN
- **分類**: 依存待ち

### L19359: 7. 一つのFailure型への統合
- **機能ブロック**: `ERR`
- **notes**: single-failure-type-per-boundary enforcement 依存待ち(第III部)
- **分類**: 依存待ち

### L19468: 9. resultとFailureの選択指針
- **機能ブロック**: `ERR`
- **notes**: result vs Failure choice-policy auto-enforcement 依存待ち(第III部)/メタ指針
- **分類**: 依存待ち

### L19469: 9.1 resultを推奨する場合
- **機能ブロック**: `ERR`
- **notes**: result推奨指針はメタ; 言語強制なし 依存待ち(第III部)
- **分類**: 依存待ち

### L19496: 10. Resource cleanup
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: bracket primitive + release guarantees (Part III runtime)
- **分類**: 依存待ち

### L19497: 10.1 基本primitive
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: bracket acquire/use/release (Part III runtime)
- **分類**: 依存待ち

### L19515: 10.2 役割
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: bracket roles (Part III runtime)
- **分類**: 依存待ち

### L19525: 10.3 特別な保証
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: bracket release guarantees (Part III runtime)
- **分類**: 依存待ち

### L19535: 11. Acquire規則
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: acquire/release registration (Part III runtime)
- **分類**: 依存待ち

### L19536: 11.1 Release登録
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: acquire failure skip release (Part III runtime)
- **分類**: 依存待ち

### L19540: 11.2 Acquire failure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: partial acquire (Part III runtime)
- **分類**: 依存待ち

### L19552: 11.3 部分取得
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: nested bracket on partial acquire (Part III runtime)
- **分類**: 依存待ち

### L19558: 12. Useの正常終了
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: use normal completion + release (Part III runtime)
- **分類**: 依存待ち

### L19570: 13. Use中のFailure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: failure during use + release (Part III runtime)
- **分類**: 依存待ち

### L19581: 14. 一般Effectと継続
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: effect suspend + continuation cleanup (Part III runtime)
- **分類**: 依存待ち

### L19582: 14.1 一時中断
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: temporary effect suspend (Part III runtime)
- **分類**: 依存待ち

### L19594: 14.2 Resume
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: resume without release (Part III runtime)
- **分類**: 依存待ち

### L19598: 14.3 Discard
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: discard continuation cleanup (Part III runtime)
- **分類**: 依存待ち

### L19620: 14.5 継続escape
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: continuation escape cleanup (Part III runtime)
- **分類**: 依存待ち

### L19638: 15. Cleanup順序と回数
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: cleanup LIFO order (Part III runtime)
- **分類**: 依存待ち

### L19639: 15.1 LIFO
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: cleanup LIFO (Part III runtime)
- **分類**: 依存待ち

### L19653: 15.2 高々一回
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: cleanup at-most-once (Part III runtime)
- **分類**: 依存待ち

### L19668: 15.3 一つのRelease失敗
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: single release failure (Part III runtime)
- **分類**: 依存待ち

### L19676: 16. Cleanup中のFailure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: failure during cleanup (Part III runtime)
- **分類**: 依存待ち

### L19677: 16.1 Primary failure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: primary failure during cleanup (Part III runtime)
- **分類**: 依存待ち

### L19681: 16.2 Suppressed failure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: suppressed failure (Part III runtime)
- **分類**: 依存待ち

### L19702: 16.3 正常終了後のRelease failure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: release failure after success (Part III runtime)
- **分類**: 依存待ち

### L19709: 16.4 複数のSuppressed failure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: multiple suppressed failures (Part III runtime)
- **分類**: 依存待ち

### L19720: 16.5 通常Handlerへの公開
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: suppressed failure not in handler (Part III runtime)
- **分類**: 依存待ち

### L19734: 17. finally
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: finally (Part III runtime)
- **分類**: 依存待ち

### L19749: 18. Cancellation
- **機能ブロック**: `ERR`
- **notes**: full ERR Cancellation model beyond CancellationToken stub 依存待ち(第III部)
- **分類**: 依存待ち

### L19757: 18.2 Cleanup
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: cancellation cleanup (Part III runtime)
- **分類**: 依存待ち

### L19761: 18.3 詳細
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: cancellation details (Part III runtime)
- **分類**: 依存待ち

### L19772: 19. Defect
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: defect model (Part III outcome/runtime)
- **分類**: 依存待ち

### L19773: 19.1 定義
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: defect definition (Part III outcome/runtime)
- **分類**: 依存待ち

### L19788: 19.2 Effect row
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: defect not in effect row (Part III)
- **分類**: 依存待ち

### L19794: 19.3 再開
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: no defect resume (Part III)
- **分類**: 依存待ち

### L19798: 19.4 通常Handler
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: defect not caught by handler (Part III)
- **分類**: 依存待ち

### L19802: 19.5 Cleanup
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: defect cleanup (Part III)
- **分類**: 依存待ち

### L19808: 20. Fault boundary
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: fault boundary (Part III runtime)
- **分類**: 依存待ち

### L19809: 20.1 定義
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: fault boundary definition (Part III)
- **分類**: 依存待ち

### L19823: 20.2 一般公開
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: fault boundary not user API (Part III)
- **分類**: 依存待ち

### L19827: 20.3 処理
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: fault boundary processing (Part III)
- **分類**: 依存待ち

### L19838: 20.4 継続条件
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: fault boundary continuation (Part III)
- **分類**: 依存待ち

### L19851: 21. Terminal failure
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: terminal failure (Part III)
- **分類**: 依存待ち

### L19852: 21.1 定義
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: terminal failure definition (Part III)
- **分類**: 依存待ち

### L19866: 21.2 通常Handler
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: terminal failure vs handler (Part III)
- **分類**: 依存待ち

### L19870: 21.3 Cleanup
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: terminal failure cleanup (Part III)
- **分類**: 依存待ち

### L19874: 21.4 可能な最小処理
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: terminal failure minimal handling (Part III)
- **分類**: 依存待ち

### L19887: 22.1 Assertion
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: assertion→defect wiring (Part III)
- **分類**: 依存待ち

### L19897: 22.2 Match
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: match exhaustiveness defect (Part III)
- **分類**: 依存待ち

### L19904: 22.3 Dynamic cast
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: cast failure classification (Part III gradual)
- **分類**: 依存待ち

### L19911: 22.4 Index access
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: index access failure (Part III)
- **分類**: 依存待ち

### L19918: 22.5 Arithmetic overflow
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: arithmetic overflow policy (Part III)
- **分類**: 依存待ち

### L19934: 22.6 Division by zero
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: division by zero policy (Part III)
- **分類**: 依存待ち

### L19950: 22.8 Validator
- **機能ブロック**: `ERR`
- **notes**: 依存待ち: validator defect (Part III)
- **分類**: 依存待ち

### L19957: 22.9 Foreign adapter
- **機能ブロック**: `ERR`
- **notes**: classify_foreign_adapter done; Terminal/fault boundary 依存待ち(第III部)
- **分類**: 依存待ち

### L19964: 22.10 Resource exhaustion
- **機能ブロック**: `ERR`
- **notes**: classify_resource_exhaustion done; job budget/Terminal OOM 依存待ち(第III部)
- **分類**: 依存待ち

### L20007: 23.3 権限・機密性
- **機能ブロック**: `ERR`
- **notes**: permission/secrecy fields present; host redaction 依存待ち(第III部)
- **分類**: 依存待ち

### L20039: 25. Entry pointと実行環境
- **機能ブロック**: `ERR`
- **notes**: RequiredEffects⊆Provided entry matrix 依存待ち(第III部)
- **分類**: 依存待ち

### L20040: 25.1 Runtime capability
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): entry RequiredEffects ⊆ ProvidedEffects capability check
- **分類**: 依存待ち

### L20078: 25.2 実行環境ごとのmain
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): CLI/GUI/Server/Worker distinct main contracts
- **分類**: 依存待ち

### L20097: 26. 未処理Failure
- **機能ブロック**: `ERR`
- **notes**: Application sink/retry for unhandled Failure 依存待ち(第III部)
- **分類**: 依存待ち

### L20098: 26.1 原則
- **機能ブロック**: `ERR`
- **notes**: Application-boundary policy for unhandled Failure 依存待ち(第III部)
- **分類**: 依存待ち

### L20108: 26.2 最終防御
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): final Failure sink cleanup + JobResult conversion
- **分類**: 依存待ち

### L20119: 26.3 Runtime default表示
- **機能ブロック**: `ERR`
- **notes**: full Error→Diagnostic explain API 依存待ち(第III部); default FailureReport display present
- **分類**: 依存待ち

### L20136: 27. 実行環境別の処理
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): env-specific Failure/Defect/Cancel/Terminal taxonomy
- **分類**: 依存待ち

### L20137: 27.1 CLI
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): CLI exit-status taxonomy
- **分類**: 依存待ち

### L20159: 27.2 GUI
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部|第V部): GUI command/render fault boundaries
- **分類**: 依存待ち

### L20168: 27.3 Server
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): Server request isolation
- **分類**: 依存待ち

### L20174: 27.4 Plugin
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部|第V部): Plugin invocation fault boundary
- **分類**: 依存待ち

### L20181: 27.5 Render job
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部|第V部): Render job fault boundary
- **分類**: 依存待ち

### L20190: 28. Diagnostic
- **機能ブロック**: `ERR`
- **notes**: 依存待ち(第III部): reciplexa-diagnostic + FailureDiagnosticBundle; ERR §28 主モデルは後回し
- **分類**: 依存待ち

### L20208: 28.2 PrimaryとSuppressed
- **機能ブロック**: `ERR`
- **notes**: primary→suppressed order present; cleanup wiring 依存待ち(第III部)
- **分類**: 依存待ち

### L20577: 1. メモリとResourceの分離
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): values vs bracket resources 方針のみ; bracket 表面は後回し
- **分類**: 依存待ち

### L20594: 1.2 外部Resource
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: external Resource lifetime via bracket (Part III)- notes: policy: values vs resources; RSC/bracket separation incomplete
- **分類**: 依存待ち

### L20684: 3.2 Surface所有権注釈
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: Surface ownership annotations (none in v1 core path)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L20708: 3.3 Trusted boundary
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: Trusted foreign ownership boundary metadata- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L20940: 9.4 Closure identity
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: closure identity observability (Part III / full MEM)
- **分類**: 依存待ち

### L21135: 15. Scoped Resource handle
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: scoped resource handle + bracket (Part III)
- **分類**: 依存待ち

### L21136: 15.1 隠れたscope
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: hidden resource scope / bracket (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21153: 15.2 bracketの概念型
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: bracket concept type surface (Part III)
- **分類**: 依存待ち

### L21168: 15.3 Escape禁止
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: scoped resource escape ban (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21182: 15.4 Scope内Closure
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: scope closure capture rules (Part III)
- **分類**: 依存待ち

### L21188: 15.5 独立した結果値
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: independent result vs handle (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21201: 16. Resource API
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: resource API surface (Part III)
- **分類**: 依存待ち

### L21202: 16.1 With-style API
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: with-style resource API (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21210: 16.2 低水準API
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: low-level resource API (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21216: 16.3 Release責任
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: release responsibility model (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21223: 16.4 自発的な無効化
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: voluntary invalidation API (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21296: 19. Foreign boundary
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: foreign ownership boundary (Part III)
- **分類**: 依存待ち

### L21297: 19.1 Ownership metadata
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: foreign ownership metadata (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21320: 19.2 Borrowed契約違反
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: borrowed foreign contract (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21330: 19.3 Owned移送
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: owned foreign transfer (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21360: 20.3 Scoped Resource
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部|ASY): scoped resource must-not-send across tasks
- **分類**: 依存待ち

### L21366: 21. SnapshotとPerceus
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: snapshot + Perceus integration (Part III)
- **分類**: 依存待ち

### L21367: 21.1 構造共有
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: snapshot structural sharing + Perceus (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21380: 21.2 解放
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: snapshot release with Perceus (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21386: 21.3 一意性
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: snapshot uniqueness (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21390: 21.4 観測不能
- **機能ブロック**: `MEM`
- **notes**: 依存待ち: snapshot observability (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21424: 23. メモリ予算
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): job-increment memory budget model
- **分類**: 依存待ち

### L21425: 23.1 適用単位
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): memory budget unit- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21440: 23.2 Job増分方式
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): job-increment budget- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21459: 23.3 Commit時の移管
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): commit-time budget transfer- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21475: 24. 予算超過
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): failure resource-exhausted on budget exceed
- **分類**: 依存待ち

### L21476: 24.1 型付きFailure
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): resource-exhausted Failure- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21492: 24.2 処理
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): budget-exceed handling- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21506: 24.3 予約領域
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): reserved budget region- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21512: 25. 単一巨大Allocation
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): huge-allocation precheck classification
- **分類**: 依存待ち

### L21513: 25.1 事前検査
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): huge allocation precheck- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21522: 25.2 分類
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): huge allocation classification- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21538: 26. Continuation予算
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): continuation capture budget
- **分類**: 依存待ち

### L21539: 26.1 課金対象
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): continuation capture budget- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21552: 26.2 超過時
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): continuation budget exceed- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21564: 27. Snapshot保持量
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部|第V部): snapshot retention budget / refuse new handles
- **分類**: 依存待ち

### L21565: 27.1 有効なSnapshot handle
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部|第V部): snapshot handle retention limit- notes: DocumentSnapshot holds while handle live; retention-limit refuse 依存待ち(第III部|第V部)
- **分類**: 依存待ち

### L21571: 27.2 保持policy
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部|第V部): snapshot retention policy- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21584: 27.3 履歴破棄
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部|第V部): history discard policy- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21588: 28. 一般heap OOM
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): process heap OOM vs managed Failure Terminal path
- **分類**: 依存待ち

### L21589: 28.1 管理予算との区別
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): managed budget vs Terminal OOM distinction
- **分類**: 依存待ち

### L21593: 28.2 分類
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): OOM classification- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L21600: 28.3 Cleanup
- **機能ブロック**: `MEM`
- **notes**: 依存待ち(第III部): OOM cleanup path- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
- **分類**: 依存待ち

### L22272: 16.4 SourceEdit
- **機能ブロック**: `TST`
- **notes**: 依存待ち(第V部): syntax edit + source_sync; 完全 SourceEdit 計算は GUI
- **分類**: 依存待ち

---

## 要確認（0）

