# 第II部 conformance — `deferred` 一覧

## deferred とは何か

`deferred` は、`lang/part2-conformance.md` 上で**レビュー済みだが、いま実装適合を追わない**と印した見出しである。「未実装だから gap」ではなく、**意図的に後回し**（実装順序・依存・仕様の開き）であることを notes に残すためのステータス。したがって `gap`/`partial` の埋める優先度とは別枠で管理する。

### 分類ごとの件数

- **合計**: 269
- **仕様未決定**: 56
- **意図的後回し**: 201
- **依存待ち**: 12
- **要確認**: 0

分類の見方:

- **仕様未決定** — `OPEN-*`、仕様本文の「未決定／保留」、またはまだ閉じない設計選択
- **意図的後回し** — 仕様は概ね確定しているが、実装順序（カーネル→PKG、multi-shot EFF 等）で後にする
- **依存待ち** — 他機能（PKG/MOD/ERR 等）や第III部以降の前提が先
- **要確認** — title/notes だけでは上記に切れないもの

---

## 仕様未決定（56）

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

### L6292: `DD-BND-032`: relaxed value restrictionの保留
- **機能ブロック**: `BND`
- **notes**: relaxed value restriction reserved
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

### L7919: 22. 移管先OPEN・下位項目・状態
- **機能ブロック**: `MAC`
- **notes**: OPEN移管カタログ
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

### L14477: 0.2 本項目が直接定めないもの
- **機能ブロック**: `MOD`
- **notes**: PKG/MAC/KER/recursive/1st-class/generative → OPEN
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

### L17157: 22. 移管先OPEN・下位項目・状態
- **機能ブロック**: `PKG`
- **notes**: OPEN transferred; deferred with PKG
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

### L17232: 22.8 下位項目
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

### L20399: 30. 移管先OPEN
- **機能ブロック**: `ERR`
- **notes**: OPEN transfer; deferred
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

### L21906: 34. 移管先OPEN
- **機能ブロック**: `MEM`
- **notes**: OPEN transfer; deferred
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

## 意図的後回し（201）

### L2897: 2.3 Constructor固有型
- **機能ブロック**: `DAT`
- **notes**: plan: full polymorphic ADT typing / ctor-specific types deferred
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

### L3166: 7. 型parameter推論
- **機能ブロック**: `DAT`
- **notes**: params in DataEnv.type_params; ctor typing still Dynamic (plan)
- **分類**: 意図的後回し

### L3182: 7.2 期待型からの推論
- **機能ブロック**: `DAT`
- **notes**: params in DataEnv.type_params; ctor typing still Dynamic (plan)
- **分類**: 意図的後回し

### L3200: 7.3 一部未確定のparameter
- **機能ブロック**: `DAT`
- **notes**: params in DataEnv.type_params; ctor typing still Dynamic (plan)
- **分類**: 意図的後回し

### L3222: 7.4 Value restriction
- **機能ブロック**: `DAT`
- **notes**: DAT param typing still Dynamic; plan DAT-001 deferral
- **分類**: 意図的後回し

### L3251: 7.5 値位置の明示型argument
- **機能ブロック**: `DAT`
- **notes**: DAT param typing still Dynamic; plan DAT-001 deferral
- **分類**: 意図的後回し

### L5597: `DD-BND-011`: 多相再帰の禁止
- **機能ブロック**: `BND`
- **notes**: polymorphic recursion out of v1; plan BND deferral
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

### L7223: 6.6 0個以上の反復
- **機能ブロック**: `MAC`
- **notes**: 0個以上`...`はv1対象外
- **分類**: 意図的後回し

### L10878: `DD-TYP-EFF-006`: EffectRow alias
- **機能ブロック**: `LIT`
- **notes**: effect-row polymorphism / aliases not full
- **分類**: 意図的後回し

### L13579: `DD-EFF-008`: return clause
- **機能ブロック**: `EFF`
- **notes**: return-clause / Handler<L,A,B,H> typing deferred (plan)
- **分類**: 意図的後回し

### L13634: `DD-EFF-009`: handler単位の結果型変換
- **機能ブロック**: `EFF`
- **notes**: return-clause / Handler<L,A,B,H> typing deferred (plan)
- **分類**: 意図的後回し

### L13875: Named/scoped effect instance
- **機能ブロック**: `EFF`
- **notes**: named/scoped effect instances not in kernel
- **分類**: 意図的後回し

### L14560: 2.3 Interface path
- **機能ブロック**: `MOD`
- **notes**: .rpi interface path not implemented
- **分類**: 意図的後回し

### L14859: 8. .rpiインターフェース
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14860: 8.1 役割
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14870: 8.2 Wrapper
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14879: 8.3 .rpi内のimport
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14891: 8.4 Public module
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14897: 8.5 Internal module
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14903: 8.6 Script／実行entry
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14907: 9. 推論シグネチャと抽象化境界
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14908: 9.1 .rpiなしの内部モジュール
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14912: 9.2 .rpiありのモジュール
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14918: 9.3 Interface追加
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14924: 10. .rpiの値仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14925: 10.1 type
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14938: 10.2 実装
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14946: 10.3 型注釈の省略
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14952: 10.4 実装側にも型がある場合
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14958: 11. 型の公開方法
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14959: 11.1 抽象型仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14977: 11.2 Parameter付き抽象型
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14986: 11.3 Constructor公開data仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L14998: 11.4 実装との一致
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15011: 11.5 type-alias
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15025: 12. 名前付きシグネチャ
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15026: 12.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15033: 12.2 日本語上の意味
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15042: 12.3 .rpiとの違い
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15049: 12.4 Signature identity
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15055: 13. シグネチャ指定
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15056: 13.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15069: 13.2 適合判定
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15075: 13.3 追加構成要素
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15081: 13.4 抽象型identity
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15089: 13.5 実装内部のalias
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15103: 14. 下位モジュール仕様
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15104: 14.1 名前付きシグネチャ
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15119: 14.2 インラインシグネチャ
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15128: 14.3 下位モジュール型の参照
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15134: 14.4 抽象型の独立性
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15142: 14.5 非公開型の漏出
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15151: 15. シグネチャの精緻化と型共有
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15152: 15.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15156: 15.2 抽象型の具体化
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15163: 15.3 別モジュールとの型共有
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15174: 15.4 日本語用語
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15185: 15.5 許可される精緻化
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15189: 15.6 禁止される変更
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15198: 15.7 Parameter付き型constructor
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15205: 15.8 下位モジュール内の型
- **機能ブロック**: `MOD`
- **notes**: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)
- **分類**: 意図的後回し

### L15212: 16. Functor
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15213: 16.1 概念
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15223: 16.2 基本構文
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15233: 16.3 Parameter
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15243: 16.4 結果シグネチャ
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15249: 16.5 Fixed arity
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15255: 16.6 通常値ではない
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15261: 17. Functor適用
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15262: 17.1 基本構文
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15269: 17.2 複数引数
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15276: 17.3 名前付きモジュールのみ
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15282: 17.4 適用結果の再利用
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15294: 18. 適用的Functor
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15295: 18.1 基本規則
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15305: 18.2 同一入力
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15319: 18.3 異なる入力
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15333: 18.4 Aliasの影響
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
- **分類**: 意図的後回し

### L15339: 18.5 生成的Functor
- **機能ブロック**: `MOD`
- **notes**: full ML functors deferred (MOD-001 / language-kernel-plan)
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

### L15473: 20.9 Functor適用結果
- **機能ブロック**: `MOD`
- **notes**: functor apply identity N/A until functors
- **分類**: 意図的後回し

### L16140: 4.3 表示名
- **機能ブロック**: `PKG`
- **notes**: PKG-001 deferred (post language-kernel)
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

### L16674: 17.2 役割
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

### L16698: 17.5 不整合
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
- **notes**: PKG-001 deferred (post language-kernel)
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

## 依存待ち（12）

### L1360: 12. 単位と色
- **機能ブロック**: `SYN`
- **notes**: 単位・色はPKG担当; suffixはinterim分割
- **分類**: 依存待ち

### L1362: 12.1 単位
- **機能ブロック**: `SYN`
- **notes**: 40mm→Number+Ident; typed (mm 40)はPKG
- **分類**: 依存待ち

### L1402: 12.2 色
- **機能ブロック**: `SYN`
- **notes**: #hexなし; 色ctorはPKG
- **分類**: 依存待ち

### L4079: 21.6 Transparent export
- **機能ブロック**: `DAT`
- **notes**: MOD export/abstract data boundary; MOD-001 deferral
- **分類**: 依存待ち

### L4085: 21.7 Abstract export
- **機能ブロック**: `DAT`
- **notes**: MOD export/abstract data boundary; MOD-001 deferral
- **分類**: 依存待ち

### L4091: 21.8 一部constructor公開
- **機能ブロック**: `DAT`
- **notes**: MOD export/abstract data boundary; MOD-001 deferral
- **分類**: 依存待ち

### L7047: 3.6 Interfaceおよびmanifest
- **機能ブロック**: `MAC`
- **notes**: interface/manifestはPKG/OPEN
- **分類**: 依存待ち

### L7720: 20.3 将来拡張
- **機能ブロック**: `MAC`
- **notes**: 将来のpkg公開
- **分類**: 依存待ち

### L7900: 21.13 不適合試験 MAC-13：Interface
- **機能ブロック**: `MAC`
- **notes**: MAC-13 interface未（PKG）
- **分類**: 依存待ち

### L14371: `DD-EFF-020`: cleanupとの接続要件
- **機能ブロック**: `EFF`
- **notes**: cleanup/finalization → ERR; plan deferral
- **分類**: 依存待ち

### L22236: 15.6 Module境界
- **機能ブロック**: `TST`
- **notes**: module signature checking deferred with MOD signatures
- **分類**: 依存待ち

### L22593: 段階5: 統合試験を通過
- **機能ブロック**: `TST`
- **notes**: multi-package/GUI/resource integration post-PKG
- **分類**: 依存待ち

## 要確認（0）

（なし）
