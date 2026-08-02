# reciplexa / RPX 言語設計記録

> 文書状態: 設計記録（規範仕様策定前）
>
> 更新日: 2026-07-27
>
> 対象: 最終的に目指す RPX 言語、および現行 reciplexa 実装との対応

## 0. この文書の読み方

本書は、文書・組版・ベクター作図・スライド・アニメーションを統合する
reciplexa のための RPX 言語について、2026-07-27 までの設計会話を整理した
**言語設計記録**である。完成した規範仕様ではない。現在の実装を説明する箇所と、
最終目標を説明する箇所を区別する。

2026-07-27版では、`solve.md`の`OPEN-EVAL-001`、`OPEN-EFF-001`、`OPEN-TYP-001`、`OPEN-BND-001`、`OPEN-TYP-002`の解決内容を本文へ統合した。

各記述には次の状態を用いる。

| 状態 | 意味 |
|---|---|
| `確定` | 会話で明示的に採用された設計 |
| `暫定` | 有力案だが、詳細または最終承認が残る |
| `推定` | 既決事項から合理的に推測される |
| `未決定` | 仕様決定がまだない |
| `矛盾候補` | 既存実装または別の設計判断と両立しない可能性 |
| `棄却` | 採用しないことが明確になった案 |
| `情報不足` | 現在の資料だけでは判断できない |

「保証」「安全」という語は、特記しない限り設計目標を指す。形式証明または機械検証を
意味しない。現在、本文に記載するメタ理論上の性質はすべて未証明である。

### 0.1 規範性

- `確定`であっても、構文・推論規則が未決定なら、その未決定部分は規範的ではない。
- `暫定`、`推定`、`未決定`、`矛盾候補`は実装適合性の根拠にしてはならない。
- 現行実装に存在するだけの機能は、最終言語へ自動的に採用されない。
- 例示中の構文は、該当節で`確定`と明記されない限り説明用である。

### 0.2 ID体系

| 接頭辞 | 対象 |
|---|---|
| `DD-*` | 設計決定 |
| `LEX-*`, `SYN-*` | 字句・構文 |
| `EVAL-*`, `BND-*` | 評価・束縛 |
| `MAC-*` | マクロ |
| `TYP-*`, `ROW-*` | 型・レコード行 |
| `EFF-*` | エフェクト |
| `MOD-*`, `PKG-*` | モジュール・パッケージ |
| `EDT-*` | GUI・双方向編集 |
| `IR-*` | 描画中間表現 |
| `ERR-*`, `MEM-*` | エラー・メモリ |
| `TST-*` | テスト機構 |
| `G-*`, `T-*`, `E-*` | 構文規則・型付け規則・評価規則 |
| `CE-*` | 反例候補 |
| `TEST-*` | 適合試験 |
| `OPEN-*` | 未決定事項 |

## 1. 言語の概要

### 1.1 目的

`DD-001` — `確定`

RPX は、Word、PowerPoint、TeX、SATySFi、manim、Adobe Illustrator、
After Effects、Affinity 系アプリケーションに相当する用途を、個別アプリケーションの
固定された表現モデルへ閉じ込めず、型付き関数と必要最小限の hygienic macro により
ユーザー自身が構築できるようにする Lisp 系言語である。

核は「文書」「スライド」「アニメーション」を特別扱いしない。これらは RPX
パッケージとして構築される。Rust カーネルは、言語処理、低水準描画、リソース解決、
バックエンド境界、provenance を提供する。

### 1.2 想定利用者

`推定`

- コードと GUI の双方で文書・図・動画を制作したい利用者
- 独自の組版・作図・アニメーション語彙を作るパッケージ作者
- 再現可能なレポート、教材、スライド、図版、動画を生成する研究者・制作者
- 型・テストによる検証を利用しつつ、ライブ編集も行いたい利用者

具体的な対象熟練度、教育用途、アクセシビリティ要件は`未決定`である。

### 1.3 想定用途

`確定`

- 日本語を含む文書組版
- レポート、書籍、注釈、数式、図表
- ベクターグラフィックスとレイヤー編集
- スライドと段階表示
- 時間関数、キーフレーム、タイムラインによるアニメーション
- PDF、SVG、PNG、動画、PPTX 等への出力
- ソースと GUI の双方向同期
- パッケージによる新しい表現モデルの構築

個別ファイル形式の完全互換性は`未決定`であり、既存製品のクローンを目的とはしない。

## 2. 設計目標

| ID | 状態 | 目標 |
|---|---|---|
| `DD-002` | `確定` | Rust カーネルのプリミティブを最小にする |
| `DD-003` | `確定` | 拡張の主役は型付き関数とし、マクロは関数で表せない構文・束縛・評価制御・two-faced syntax に限定する |
| `DD-004` | `確定` | 最終的にすべてのマクロを hygienic にする |
| `DD-005` | `確定` | Elixir 系の集合論的漸進型と semantic subtyping を採用する |
| `DD-006` | `確定` | レコードに row polymorphism を統合する |
| `DD-007` | `確定` | Koka を参考に、row-polymorphic effect types と algebraic effect handlers を採用する |
| `DD-008` | `確定` | ML/OCaml 系の signature、opaque sealing、functor を持つ静的モジュールを採用する |
| `DD-009` | `確定` | GUI 編集を provenance と `Editable` 更新戦略によってソースへ反映する |
| `DD-010` | `確定`/`暫定` | 単一万能 RenderIR を採用しないことは`確定`。Domain/Visual/Motion/Render/Backend IR という具体的分割は`暫定` |
| `DD-011` | `確定` | 時間は隠れた副作用でなく `Time -> Picture` の明示入力とする |
| `DD-012` | `確定` | 外部リソースと I/O はエフェクトとハンドラの境界へ置く |
| `DD-013` | `確定` | ブラックボックステストとホワイトボックステストを共に言語・エコシステムで支える |
| `DD-014` | `暫定` | CLI/テストは厳格に失敗し、GUI は last-good-render を保持する |

### 2.1 優先関係

`暫定`

1. 表現力と拡張可能性
2. 意味の予測可能性、決定性、再現性
3. 型・効果・モジュールによる安全性と抽象化
4. ソースと GUI の双方向編集可能性
5. 実用性能とインクリメンタル再計算
6. 構文上の簡潔性
7. カーネル実装の容易性

これは厳密な全順序ではない。ライブ編集では完全な静的保証より応答性を優先する場合が
ある一方、出力・テストでは保証を優先する。このモード差の正確な境界は`未決定`である。

## 3. 設計原則

1. **小さい核**: Path、文字、画像、合成等の不可欠な機能だけを Rust に置く。
2. **関数第一**: 通常の抽象化・再利用・ドメイン語彙は関数で行う。
3. **明示的効果**: I/O、リソース、乱数、状態は型付きエフェクトとして追跡する。
4. **不変値を既定**: 可変状態は局所的・型付き・スコープ付きにする。
5. **意味と表示の分離**: 文書やスライドの意味構造と最終描画命令を分ける。
6. **決定性**: 同じ source、params、正規化済みresources、target、handler入力、
   runtime/backend versionから同じ意味結果を得ることを目標とする。
7. **lossless source**: 空白・コメントを含む CST と provenance を編集の基礎にする。
8. **抽象化尊重**: GUI、マクロ、dynamic、provenance が module sealing を破らない。
9. **テスト可能性**: 外部世界をハンドラへ隔離し、テストをオンメモリで実行可能にする。
10. **段階的実装**: 最終設計を記録しつつ、小さい動く縦切りを積み重ねる。

## 4. 確定事項

- 最小プリミティブと RPX パッケージ中心の構成
- 関数中心、hygienic macro 限定
- `val`による不変束縛、`var`による局所可変束縛、`fn`による関数
- `define`を最終表面構文として使用しない
- 集合論的漸進型、semantic subtyping、record row polymorphism
- RecordRow と EffectRow の kind 分離
- Koka 系 effect row と handler
- ML 系 module/signature/functor
- `Editable`の replace/offset/transform/lens 方針
- ページ・スライドのサイズと metadata を個別値に保持
- 型、record row、effect row の別名機構
- 時間を明示入力にする Signal モデル
- 正格call-by-value、operator-first・左から右の評価順序
- deep handlerとone-shot resumption
- bounded dynamic、cast evidence、dynamic-type-error
- `letrec`による自己再帰・相互再帰
- 独立した`(type ...)`型注釈とrank-1／prenex polymorphism
- 構文的value restriction
- bidirectional type checkingと三値algorithmic判定
- closed／open／row-polymorphic recordの区別
- regular・contractive・strictly positiveなrecursive data

## 5. 暫定事項

- Lisp-1 の Value namespace
- `handle`を基本、`with`を糖衣構文とする
- effect operation を通常関数のように呼び、`use`を必須にしない
- ファイルの既定を code mode とし、`src` wrapper を最終的に不要にする
- `(effects ...)`による型内 effect row 構文
- applicative functor を既定、generative application を明示
- layered RenderIR の具体的代数
- GUI の last-good-render

## 6. 推定事項

- 実行系は、型検査済み Core IR のインタプリタから開始し、将来コンパイルを追加する。
- Rust ホストは tracing/GC/resource table を持つ必要がある。
- public package API には型注釈を要求または強く推奨する。
- 3D は RPX パッケージで 2D Picture へ投影し、最終 RenderIR は 2D とする。
- package の top-level 初期化をempty effect rowに制限する案。totalityを要求するか、
  また停止性をどう検査するかは未決定。

## 7. 棄却済みの案

| ID | 案 | 理由 |
|---|---|---|
| `DD-X01` | circle、rect 等を最終カーネルプリミティブとして増やし続ける | 最小核と自由な表現モデルに反する |
| `DD-X02` | すべての拡張をマクロで行う | 型、テスト、provenance、双方向編集を壊しやすい |
| `DD-X03` | 型内の効果区切りに裸の`/`を使う | Lisp構文で位置が不明瞭、型差・除算・パスと衝突し得る |
| `DD-X04` | 計算値のGUI編集時に常に暗黙の`+`を挿入する | 型一般性がなく、意図不明・入れ子化・抽象化破壊を招く |
| `DD-X05` | すべてを一つの万能 RenderIR に押し込む | 文書意味、編集可能性、時間、描画正規化の要求が競合する |
| `DD-X06` | public 関数だけをテスト可能にする | ホワイトボックステストを不当に排除する |

## 8. 矛盾候補

### `CON-001`: `src`の現在実装と最終 code mode

- 現実装: ファイル全体をLisp modeで読み、`(src ...)`内では限定された
  `perform`/`handle`だけを型検査・実行する。
- 最終案: ファイル全体を code mode とし、`doc`だけで reader mode を切り替える。
- 状態: `矛盾候補`。
- 案A: 後方互換 wrapper として`src`を恒久的に残す。
- 案B: migration warning の後に削除する。
- 必要な決定: `OPEN-SYN-001`。

### `CON-002`: 現在の fail-fast と GUI last-good-render

- 現実装/設計メモ: parse/type error で不完全な木を描画しない。
- 最終案: GUI は直前の正常描画を保持する。
- 両立案: 新しい壊れた木を描画しないが、以前の正常 RenderIR は表示する。
- 状態: `暫定`。UI上の選択・編集対象の扱いは`未決定`。

### `CON-003`: 現行 shape IR と最終 Path 中心 IR

- 現実装: Rust enum に Circle、Rect、Ellipse 等が存在する。
- 最終案: VisualIR/RenderIR の最終プリミティブを Path 等に絞る。
- 両立案: 現在の scene を移行用 High-level SceneIR と見なし、後段で Path へ lower する。
- 状態: `矛盾候補`ではあるが段階移行可能。

### `CON-004`: `var`、effect handler、`Editable`

- `var`: 実行中の局所状態。
- `Editable`: source CST を更新する外部編集効果。
- 両者を同じ mutation と扱うと、再実行時の永続性と多重 resume の意味が衝突する。
- 決定: 同一視しない。`Editable`の厳密な effect/transaction 意味は`未決定`。

### `CON-005`: module sealing と provenance

- provenance が private 実装の source origin を外部へ漏らすと opaque sealing が破れる。
- 暫定制約: import package の private origin は read-only かつ非公開。
- 形式的な representation-independence は未証明・仕様不足。

### `CON-006`: 現行座標系と最終RenderIR候補

- 現行scene: 左下原点、x右/y上。
- 最終RenderIR候補: 左上原点、x右/y下。
- 影響: text baseline、rotation、hit test、GUI patch、provenance property。
- 解釈案A: 現行座標を最終仕様に維持する。
- 解釈案B: SceneIR→VisualIRで一度だけ座標変換し、source propertyとの対応を明示する。
- 状態: `矛盾候補`。決定は`OPEN-IR-001`。

### `CON-007`: 未知のdoc command

- 最終案: 未束縛`@foo`を名前解決エラーにする。
- 現行doc macro: 未知commandを通常textへflattenする場合がある。
- 解釈案A: warningを経てerrorへ移行する。
- 解釈案B: doc packageごとにunknown-command fallbackを宣言可能にする。
- 状態: `矛盾候補`。

## 9. 未決定事項と解決済み最優先項目

### 9.1 解決済み

| ID | 状態 | 統合先 |
|---|---|---|
| `OPEN-EVAL-001` | `解決済み` | 13.3 `EVAL-001` |
| `OPEN-EFF-001` | `解決済み` | 13.8 `EFF-001` |
| `OPEN-TYP-001` | `解決済み` | 13.6.1 `TYP-DYN-001` |
| `OPEN-BND-001` | `解決済み` | 13.4 `BND-001` |
| `OPEN-TYP-002` | `解決済み` | 13.6.2 `TYP-ALG-001` |

旧`OPEN-*`名は決定履歴の参照用に保持するが、未決定事項として扱わない。

### 9.2 現在の未決定事項

| ID | 優先度 | 問題 |
|---|---:|---|
| `OPEN-SYN-001` | 高 | `src`／`doc`の最終名称とmigration |
| `OPEN-SYN-002` | 高 | literal、number、unit、string、comment、型・宣言groupの完全文法 |
| `OPEN-DAT-001` | 高 | variant、pattern、match、網羅性、guard |
| `OPEN-MAC-001` | 高 | macro phase、展開fuel、意図的capture API |
| `OPEN-MOD-001` | 高 | first-class／recursive module、separate-compilation identity |
| `OPEN-EDT-001` | 高 | provenance stable ID、競合、stale transaction |
| `OPEN-IR-001` | 高 | 色、filter、Backend capabilityの規範精度 |
| `OPEN-MEM-001` | 高 | GC、resource lifetime、continuation、将来の`cell`／`ref` |
| `OPEN-ERR-001` | 高 | exception、cleanup、primitive failure、terminal taxonomy |
| `OPEN-CON-001` | 高 | concurrency、async、race／deadlock |
| `OPEN-KER-001` | 高 | ForeignValue、validator、trusted adapter ABI |
| `OPEN-TST-001` | 中 | testのSurface構文と許容effect row |

## 10. 用語集

| 用語 | 意味 |
|---|---|
| RPX | 本書で扱う最終目標の言語。名称自体は`暫定` |
| reciplexa | 現在の Rust プロジェクト/アプリケーション名 |
| code mode | S式を読む reader mode |
| doc mode | 通常テキストを読み、`@`でコードへ入る reader mode |
| CST | trivia を含み、元バイト列を再構成できる具象構文木 |
| Core | マクロ展開・名前解決後の小さい中心言語 |
| semantic subtyping | 型を値集合として解釈し、部分型を包含として定義する方式 |
| row | 明示部分以外のレコードフィールドまたはエフェクト列 |
| effect operation | ハンドラへ制御を移す型付き操作 |
| handler | effect operation の解釈を提供する値/構文 |
| provenance | 値・IR property と source CST の由来対応 |
| Editable | GUIからsource patchを生成できる値/更新戦略 |
| Surface | page、slide、artboard、frame、canvas の共通出力面 |
| VisualIR | 編集意味をある程度保持する視覚IR |
| RenderIR | リソース解決・文字整形済みの純粋2D描画IR |

### 10.1 Effect用語

- **effect signature environment**: effect／operationの静的signature環境。
- **effect environment**: 現在の式で利用可能または許容されるeffect要求の文脈。
- **expression effects**: 式を現在評価すると外側へ要求されるeffect。
- **function required effects**: 関数を呼び出すと外側へ要求されるeffect。
- **handled effects**: 現在のhandler／runnerが処理するeffect。
- **residual effects**: handler／runner適用後も外側へ残るeffect。

## 11. 記号一覧

| 記号 | 意味 |
|---|---|
| `Γ` | 値変数から型schemeへの型環境 |
| `Δ` | kind付き型変数、record row変数、effect row変数環境 |
| `M` | モジュール環境 |
| `Σ` | location/effect instanceを型付けするstore環境 |
| `H` | 実行時handler stack |
| `σ` | 実行時store |
| `e` | 式 |
| `v` | 値 |
| `T,U` | 値型 |
| `R` | RecordRow |
| `E` | EffectRow |
| `A <: B` | semantic subtyping |
| `A ≃ B` | 相互部分型による型等価 |
| `Γ ⊢ e ⇒ T ! E` | `e`が型`T`と効果`E`を合成する |
| `Γ ⊢ e ⇐ T ! E` | `e`を期待型`T`として検査し効果`E`を得る |
| `⟨e,σ,H⟩ → ⟨e',σ',H'⟩` | 1ステップ評価 |
| `[[T]]` | 型`T`の意味的な値集合 |

## 12. 機能一覧

| ID | 機能 | 状態 | 依存 | 主な影響先 |
|---|---|---|---|---|
| `LEX-001` | lossless lexer/CST | `確定` | なし | SYN, MAC, EDT |
| `SYN-001` | S式とtwo-faced reader | `確定`（名称は未決定） | LEX | MAC, TYP, PKG |
| `RES-001` | 名前解決とnamespace | `暫定` | SYN, MAC | TYP, MOD |
| `LIT-001` | literal、数値、単位、primitive | `未決定` | LEX | EVAL, TYP, IR |
| `DAT-001` | variant、pattern、match | `未決定` | SYN, TYP | EVAL, ROW |
| `EVAL-001` | strict lexical core evaluator | `暫定` | SYN | BND, EFF, TYP |
| `BND-001` | `val`/`var`/`let`/`fn` | `確定`（詳細は暫定） | EVAL, EFF | TYP, EDT |
| `MAC-001` | hygienic macro | `確定`（方式未決定） | LEX, SYN | MOD, TYP, EDT |
| `TYP-001` | gradual set-theoretic types | `確定` | EVAL | ROW, EFF, MOD |
| `ROW-001` | row-polymorphic records | `確定` | TYP | MOD, IR, PKG |
| `EFF-001` | algebraic effects/handlers | `確定`（操作構文等は暫定） | EVAL, TYP | BND, TST, PKG |
| `MOD-001` | ML-style modules | `確定`（詳細は暫定） | TYP, EFF, MAC | PKG, TST |
| `PKG-001` | package/domain libraries | `確定`（配布仕様未決定） | MOD | IR, TST |
| `KER-001` | Rust kernel/foreign primitive境界 | `確定`（ABI未決定） | TYP, EFF | PKG, IR |
| `RSC-001` | Resource/I/O/host boundary | `確定`（操作未決定） | EFF, KER | IR, TST |
| `EDT-001` | provenance/Editable | `確定`（取引意味論未決定） | LEX, EVAL, MOD | GUI, IR |
| `IR-001` | layered visual/motion/render IR | `暫定` | ROW, EFF, EDT | backend |
| `ERR-001` | errors and terminal states | `未決定` | EVAL, EFF, TYP | TST |
| `MEM-001` | runtime memory/resource model | `情報不足` | EVAL, EFF | safety |
| `ASY-001` | concurrency/async model | `未決定` | EVAL, EFF, MEM | safety, IR |
| `TST-001` | language and conformance tests | `確定`（構文未決定） | 全機能 | 全機能 |

### 12.1 依存関係の要約

```text
bytes → LEX-001 → SYN-001 → MAC-001 → name resolution
                                      ↓
                          TYP-001 + ROW-001 + EFF-001
                                      ↓
                                  typed Core
                                      ↓
                             EVAL-001 / BND-001
                                      ↓
                    Domain values → IR-001 → backend

MOD-001/PKG-001 は macro/type/runtime phase を横断する。
EDT-001 は CST、評価値、各IRを provenance side table で横断する。
TST-001 はすべての境界を観測する。
```

## 13. 各機能の詳細

本節では、要求された詳細項目を機能ごとに記録する。機能に該当しない項目は
「該当なし」と明記する。形式規則は設計スケッチであり、証明済みではない。

### 13.1 `LEX-001` Lossless lexer/CST

#### 概要・目的・状態

- 状態: `確定`。現行実装も `rowan` CST、trivia、mode stack を持つ。
- 目的: GUI編集後も空白・改行・コメント・ユーザーの記述順序を保持する。
- 利点: byte round-trip、局所patch、正確な診断、provenance。
- 欠点: ASTだけを扱う処理系よりメモリ・実装複雑性が増す。
- 代替: formatterによる全再生成。双方向編集の要求により`棄却`相当。

#### 構文・字句

現行の字句要素は括弧、`@`、brace、bracket、identifier、number、string、
text chunk、whitespace、newline、comment、error token である。

`G-LEX-001` — `暫定`

```ebnf
source      = trivia*, form*, trivia* ;
trivia      = whitespace | newline | comment ;
form        = atom | list | reader-form ;
list        = "(", trivia*, { form, trivia* }, ")" ;
atom        = identifier | number | string ;
```

identifier、number/unit、string escape、comment、Unicode normalization、BOM、
invalid UTF-8 の規範文法は`OPEN-SYN-002`。演算子の優先順位はない。

Lossless の最低要件は、受理された source について、

```text
unparse(parse(bytes)) = bytes
```

である。malformed input の error node を含むround-trip範囲は`暫定`。

#### 静的・動的意味

CSTそのものには型付け・動的評価を定義しない。spanはbyte offsetであることが現実装上の
事実だが、最終仕様ではencoding確定まで`暫定`。StableSyntaxIdの生成・編集後継承は
`OPEN-EDT-001`。

#### 正常例・拒否例

```lisp
; triviaを保持する
(val radius  40mm) ; comment
```

- 型: CST段階では該当なし。
- 期待: 空白二個とコメントを保持する。

```lisp
(val x 1
```

- 現行: parse error/error node。
- 最終仕様: editor CSTとして保持するか、parse失敗だけを返すか`未決定`。

#### Progress/Preservation・その他

- CoreのProgress/Preservationの対象外。
- parse/unparseのround-trip、incremental parseとfull parseの同値性が代替する性質。
- parser totality、resource bound、malformed UTF-8 は未証明。

#### 実装構造・処理

- Token: kind、byte span、元text。
- Green/Red CST: tokenを一度ずつ順序保持。
- ParseError: span、期待、実際。
- Mode stack: code/doc reader mode。
- 入力bytes → lexer token → parser events → CST。型検査はCSTを直接使用せず、
  expanded AST/Coreへ変換するのが最終案。

#### 相互作用・テスト

- MAC: macro expansion originを保持する。
- EDT: patchはtoken/node境界を検証する。
- MOD: imported package sourceは編集権限を持たない。
- `TEST-LEX-001`: valid sourceのbyte round-trip。
- `TEST-LEX-002`: malformed delimiterのerror recoveryとround-trip。
- `TEST-LEX-003`: incremental/full parse differential fuzz。
- `TEST-LEX-004`: Unicode、CRLF/LF、巨大nest、巨大token。

#### 未決定

- `OPEN-SYN-002`（高）: encoding、escape、comment、unit literal。
- `OPEN-EDT-001`（高）: stable node identity。

### 13.2 `SYN-001` S式とtwo-faced reader

#### 概要・目的・状態

- S式をcode faceとすること: `確定`。
- 通常テキストと`@` escapeを持つdoc face: `確定`。
- `doc`、`src`という名称: `未決定`。
- ファイル全体をcode modeとし`src`を不要にする案: `暫定`。

two-faced syntax は同じsourceからコードと文書を別言語として分離するものではない。
readerがdoc syntaxを通常のsyntax objectへ変換し、意味はimportされたパッケージが与える。

#### 構文

`G-SYN-001` — code mode（部分文法）

```ebnf
code-form = atom | "(", { code-form | trivia }, ")" ;
```

`G-SYN-002` — doc mode（現行構文からの暫定）

```ebnf
doc-form   = "(doc", { doc-item }, ")" ;
doc-item   = text-chunk | at-expression ;
at-expression
           = "@", identifier,
             [ "[", { code-form | trivia }, "]" ],
             [ "{", { doc-item }, "}" ]
           | "@(", { code-form | trivia }, ")" ;
```

`@foo(...)`、brace/bracketの複合規則、escape、literal `@` は`未決定`。

構文糖の例:

```lisp
(doc Hello @em{world})
```

は概念的に次のsyntax objectへreader expansionされる。

```lisp
(doc-node
  (text "Hello ")
  (call em (doc-node (text "world"))))
```

`doc-node`等の実名は`未決定`。

#### 静的意味

doc reader自身は`em`を解決しない。通常の名前解決でパッケージの`em`へ束縛する。
doc bodyの型（Inline、Block、Doc等）はパッケージsignatureによる。現行型検査器が
doc bodyを深く検査しないことは、最終仕様の保証ではない。

#### 動的意味

reader expansion後は通常Coreと同じ評価を受ける。doc modeに固有の実行時状態はない。
reader expansionの評価順序はsource orderを保存する。

#### 例

```lisp
(val radius 40mm)
(doc 半径は @show{radius} です。)
```

- 型: `show`のsignatureとdoc packageに依存し、現時点では`情報不足`。
- 期待: radiusの表示を含むDoc値。

拒否例:

```lisp
(doc @unknown{x})
```

- reader上は受理し得る。
- 名前解決で`unknown`が未束縛なら静的エラー。
- 診断はidentifier spanを指し、doc reader expansion traceを含むべき。

#### メタ理論・実装・テスト

- reader expansionがCore syntaxを生成することを示すelaboration correctnessが必要。
- macro hygieneとreader hygieneの関係は未決定。
- `TEST-SYN-001`: code/doc nested parse。
- `TEST-SYN-002`: reader expansion source order。
- `TEST-SYN-003`: doc identifier name resolution。
- `TEST-SYN-004`: code↔doc mode stack fuzz。
- `TEST-SYN-005`: reader expansion後のsource location。

### 13.2.1 `RES-001` 名前解決とnamespace

#### 概要・状態

名前解決段階が必要なことは`推定`ではなく、hygiene、module、型検査を成立させるための
必須要件である。具体的namespace構成は`暫定`。

- 通常値と関数は同じValue namespace（Lisp-1候補）。
- Type、Module、Syntaxは別namespace候補。
- effect operationはValue namespaceから通常関数のように参照する候補。
- bindingは名前文字列でなくstable BindingIdに解決する。
- lexical scopeを採用し、shadowingは内側bindingを優先する案。

phase-aware macro expansion中にSyntax/Module bindingを解決し、expansion後にruntime
Value bindingを最終解決する。単純な「全展開後に初めてすべて解決」は採用しない。

静的エラー:

- unbound identifier
- duplicate binder（pattern内の扱いは未決定）
- ambiguous import
- private/phase違反
- module cycle

診断は使用spanと候補binding/import originを示す。shadowing warningの有無は`未決定`。
動的意味は該当なし。resolved Coreでは未解決名を許さない。

実装構造: ScopeId、BindingId、Namespace、Phase、ImportEdge、Visibility、UseSite。

- `TEST-RES-001`: lexical shadowing。
- `TEST-RES-002`: phase-separated import。
- `TEST-RES-003`: ambiguous/private import。
- `TEST-RES-004`: hygiene下のbinding identity。
- `TEST-RES-005`: alpha-renaming不変。

Progressには直接影響しないが、unresolved nameをCoreへ残すとstuckを生む。名前解決の
完全性、separate compilation path identityは未証明・未決定。

### 13.2.2 `LIT-001` Literal、数値、単位、primitive

#### 状態

Number、String、Bool、symbol/keyword、色、単位付き量が必要であることは`暫定`。
正確なliteral grammar、numeric tower、overflow、equality、hashabilityは`未決定`。

候補:

```lisp
42
3.5
40mm
12pt
30deg
"text"
true
'symbol
```

単位は型付きquantityとし、`Length`と`Angle`を混同しない方針が有力だが、
dimension algebra、unit suffix tokenization、implicit conversionは`OPEN-SYN-002`。

未決定のprimitive semantics:

- integer精度、overflow
- float NaN/Inf、signed zero、丸め
- division by zero
- string indexingをbyte/code point/graphemeのどれにするか
- structural/identity equality
- collection bounds
- color literal

これらが未決定なためcanonical forms、Progress、backend determinismを完全には定義
できない。primitive domain failureをpanicにしてはならず、静的拒否、Result、effect、
または明示runtime errorのいずれかへ分類する必要がある。

- `TEST-LIT-001`: unit dimension mismatch。
- `TEST-LIT-002`: numeric boundary/overflow。
- `TEST-LIT-003`: Unicode string boundary。
- `TEST-LIT-004`: float canonical serialization。
- `TEST-LIT-005`: equality/hash consistency。

### 13.2.3 `DAT-001` Variant、pattern、match

#### 状態

不変list/vector/map/record、variant、pattern matchがパッケージ実装に必要という案は
`暫定`。完全なpattern grammarと網羅性方針は`未決定`。

必要候補:

```lisp
(match shape
  ((record (kind 'circle) (radius r)) ...)
  ((record (kind 'rect) (width w) (height h)) ...))
```

静的目標:

- pattern成功branchでoccurrence typeをintersectionによりrefineする。
- branch差をunionで合成する。
- unreachable patternをemptiness判定で警告/拒否する。
- non-exhaustive matchを静的拒否するか`MatchError`へするか未決定。
- guardのeffectと評価順序は未決定。

動的にはscrutineeを一度評価し、clauseをsource orderで試す案。binder scopeはguard/body。
repeated binder、or-pattern、view-patternは未決定。

`CE-DAT-001`: dynamic scrutineeやnegation型に対する不完全な網羅性判定を「安全」と
みなすとmatch failureがstuckになる。

- `TEST-DAT-001`: tag/record destructuring。
- `TEST-DAT-002`: occurrence refinement。
- `TEST-DAT-003`: exhaustive/unreachable。
- `TEST-DAT-004`: guard evaluation order/effects。
- `TEST-DAT-005`: reference matcherとdecision tree差分。

### 13.3 `EVAL-001` Strict lexical Core evaluator

> 統合元: `OPEN-EVAL-001`。同項目は解決済み。

#### 状態

`解決済み`

本決定は、RPXの最小Core v1について、項、値、評価順序、逐次評価および
評価文脈を固定するものである。

`letrec`、pattern match、record、effect handler、局所可変状態、dynamic cast、
primitive固有の失敗は最小Core v1には含めず、それぞれ別の設計事項として扱う。

---

#### `DD-EVAL-001`: 評価戦略と評価順序

`確定`

RPX Coreは、正格call-by-valueで評価する。

関数適用では、operatorを最初に評価し、その後、引数をsource orderで
左から右へ評価する。すべての引数が値になった後で関数本体を評価する。

以下の評価順序は、effect、diagnostic、provenanceおよび外部操作から
観測可能な言語仕様であり、実装によって変更してはならない。

1. 関数適用ではoperatorを最初に評価する。
2. 引数は左から右へ評価する。
3. `let`のinitializerをbodyより先に評価する。
4. `seq`の各式を左から右へ評価する。
5. `if`ではconditionを先に評価し、選択されたbranchだけを評価する。
6. 将来recordを追加する場合、field式はsource orderで評価する。
7. 将来handlerを追加する場合、handler式をbodyより先に評価する。

遅延評価はCoreの既定評価戦略にはしない。必要な場合は、明示的な関数、
データ型、effectまたはhygienic macroとして導入する。

---

#### `DD-EVAL-002`: 最小Coreの項

`確定`

最小Core v1の項を次のように定める。

```text
e ::= x
    | literal
    | fn(x1, ..., xn, e)
    | app(e, e1, ..., en)
    | let(x, e1, e2)
    | if(e1, e2, e3)
    | seq(e1, ..., en)       where n >= 1
````

各項の意味は次のとおりである。

* `x`: lexical variable reference
* `literal`: Core primitive valueのリテラル
* `fn`: lexical closureを生成する関数式
* `app`: 関数適用
* `let`: 単一の不変局所束縛
* `if`: Bool条件による分岐
* `seq`: 一個以上の式の逐次評価

`val`は最小Coreの式ではない。

* Surfaceの局所`val`を許す場合はCore `let`へelaborateする。
* top-level `val`はfileまたはmodule declarationとして別途elaborateする。
* named function sugarは`val`と`fn`へelaborateする。

次のものは最小Core v1に含めない。

* `letrec`
* `match`およびpattern
* recordとvariant
* `var`およびassignment
* effect operationとhandler
* module
* dynamic cast
* resource operation固有の項

これらを言語から棄却するのではなく、安定した最小Coreに対する拡張として
別途定義する。

***

#### `DD-EVAL-003`: `let`

`確定`

Core `let`は単一bindingだけを持つ。

```text
let(x, initializer, body)
```

`initializer`を現在のlexical environmentで評価し、その結果を`x`へ束縛した
環境で`body`を評価する。

`x`は自身の`initializer`から参照できない。

Surface `let`は一個以上のsequential bindingを持つことができる。

```lisp
(let ((x e1)
      (y e2)
      (z e3))
  body)
```

これは次の単一binding Core `let`の入れ子へ、左から右にelaborateする。

```lisp
(let ((x e1))
  (let ((y e2))
    (let ((z e3))
      body)))
```

したがって、先行するbindingは後続するinitializerおよびbodyから参照できる。

同一のSurface `let` binding list内では、同じ名前のbinderを重複してはならない。
意図的なshadowingは、明示的に入れ子にした`let`によって表現する。

parallel bindingは最初の言語仕様には導入しない。必要になった場合は、
通常の`let`とは異なる明示的構文として追加する。

Core `let`を関数適用へ完全には脱糖しない。これは、let-polymorphism、
value restriction、型・effect推論、diagnosticおよびprovenanceにおける
generalization pointを保持するためである。

***

#### `DD-EVAL-004`: 複数式bodyと`seq`

`確定`

仕様でbodyを持つと定められたSurface構文では、一個以上のbody expressionを許す。

複数のbody expressionはsource orderで左から右へ評価する。最後以外の式の値は
破棄し、最後の式の値をbody全体の結果とする。

Coreの`fn`、`let`等のbodyは常に単一式とする。複数のSurface body expressionは
Core `seq`へelaborateする。

Surface:

```lisp
(fn (x)
  (log x)
  (+ x 1))
```

Core:

```lisp
(fn (x)
  (seq
    (log x)
    (+ x 1)))
```

bodyが一式だけの場合、不要な`seq`は生成しない。

`seq`は一個以上の式を持つ。空の`seq`および空のSurface bodyは認めない。
何もしないbodyは明示的な`Unit`値を返す。

途中式では任意の型の値を破棄できる。非`Unit`の純粋な値が未使用である場合、
処理系はwarningを出してよいが、これをCoreの型エラーとはしない。

`seq(e1, ..., en)`の結果型は最後の式`en`の型であり、effectはすべての式の
effectをsource orderに従って合成したものとする。

***

#### `DD-EVAL-005`: `if`

`確定`

Core `if`は厳密に次の三式を持つ。

```text
if(condition, then_branch, else_branch)
```

`condition`は静的に`Bool`型でなければならない。

Number、String、Unit、record、collection、dynamicその他の値を、
暗黙にBoolへ変換しない。0、空文字列、空collection、null相当の値に
truthinessまたはfalsinessを与えない。

動的評価では、conditionを最初に評価する。

* 結果が`true`ならthen branchだけを評価する。
* 結果が`false`ならelse branchだけを評価する。
* 選択されなかったbranchは評価しない。

then branchの型が`T`、else branchの型が`U`の場合、`if`全体の型は
`union(T, U)`とする。

semantic subtypingによって`T <: U`なら、次が成立する。

```text
union(T, U) ≃ U
```

condition、then branchおよびelse branchのすべての静的effectを、
`if`式全体のeffectへ合成する。実行時には選択されたbranchのeffectだけが発生する。

***

#### `DD-TYP-IF-001`: 条件分岐による型の絞り込み

`確定`

RPXは、処理系が認識可能な型判定によるoccurrence typingを採用する。

不変な局所変数`x`がstatic型`S`を持ち、conditionが`x`の型`T`への所属を
判定する場合、各branchで`x`を次のように扱う。

```text
then branch: intersect(S, T)
else branch: diff(S, T)
```

`diff(S, T)`はstatic type fragmentにおいて次と型等価である。

```text
intersect(S, not(T))
```

初期仕様では、直接参照された不変局所変数だけを絞り込み対象とする。

次は本決定の対象外とする。

* `var`等の可変変数
* 任意のBool戻り値関数
* effectful expression
* property path
* 計算式の再評価を必要とする絞り込み
* bounded dynamicに対するruntime narrowing

型判定の表面構文、predicate typeおよび利用者定義predicateへの拡張方法は、
型システムの別項目で決定する。

***

#### 値

最小Core v1の値を次のように定める。

```text
v ::= primitive-value
    | closure(environment, parameters, body)
```

初期の`primitive-value`は少なくとも次を含む。

```text
Bool
Unit
Number
String
```

Numberの内部分類、精度、overflow、NaN、Infinity、単位付きquantityおよび
Stringのindexing単位は`OPEN-SYN-002`およびliteral/primitive仕様へ移管する。

将来のCore拡張により、次の値を追加できる。

```text
record-value
variant-value
handler-value
continuation/resumption
resource-handle
packed-module
dynamic value with evidence
```

ただし、追加時には対応する型付け規則、評価規則およびcanonical formsを
別途定義しなければならない。

***

#### Closureとlexical scope

`fn(x1, ..., xn, body)`を評価すると、現在のlexical environmentを捕捉した
closureを生成する。

```text
closure(environment, parameters, body)
```

closure生成自体はbodyを評価せず、runtime effectを発生させない。
bodyのeffectは関数型へ記録する。

closure適用時には、closure生成時のenvironmentを基礎とし、仮引数へ実引数の値を
束縛した環境でbodyを評価する。

同名の内側bindingは外側bindingをshadowする。名前解決後のCoreでは、
識別子は文字列ではなく一意な`BindingId`によって参照する。

***

#### 関数適用

関数適用`app(operator, argument1, ..., argumentN)`は次の順序で評価する。

1. operatorを値まで評価する。
2. argumentを左から右へ値まで評価する。
3. operatorの値が対応するarityのclosureであることを確認する。
4. 各parameterを対応するargument valueへ束縛する。
5. closureのbodyを評価する。

最小Coreでは固定arity関数だけを扱う。

arity mismatchは、通常のstatic fragmentでは型検査またはelaboration時に拒否する。
型検査済みのstatic Coreで、non-function applicationまたはarity mismatchが
動的に発生してはならない。

dynamic境界、foreign primitiveまたは不正なCoreに起因するarity mismatchの分類は、
`OPEN-TYP-001`、`OPEN-KER-001`または`OPEN-ERR-001`で決定する。

可変長引数、keyword引数、optional引数、partial applicationおよびcurryingの
表面構文は本決定に含めない。

***

#### 評価文脈

最小Core v1の評価文脈を概念的に次のように定める。

```text
E ::= []
    | app(E, e1, ..., en)
    | app(v, v1, ..., vi-1, E, ei+1, ..., en)
    | let(x, E, e)
    | if(E, e1, e2)
    | seq(v1, ..., vi-1, E, ei+1, ..., en)
```

この評価文脈は、次を保証する。

* operator-first
* argument left-to-right
* initializer-before-body
* sequence left-to-right
* condition-before-branch
* unselected branch non-evaluation

ここで`seq`の最後以外の値は、次の式へ進む際に破棄する。

実装はCEK、CEKS、bytecode VMまたは直接インタプリタのいずれを使用してもよい。
ただし、上記の観測可能な評価順序を保存しなければならない。

***

#### 最小Coreの終端状態

最小Core v1の評価は、次のいずれかとなる。

1. 値を返して正常終了する。
2. 無限にstepを続けて発散する。
3. 実装上のresource limitへ到達する。

well-typedでclosedな最小Core v1の項は、次の状態になってはならない。

* unbound variable
* non-function application
* arity mismatch
* non-Bool `if` condition
* unexplained stuck state
* undefined behavior

resource exhaustionのcatch可能性およびhost failureとしての分類は、
`OPEN-MEM-001`および`OPEN-ERR-001`へ移管する。

***

#### 未決定事項の移管

`OPEN-EVAL-001`を閉じるため、最小Core v1に含まれない事項を次へ移管する。

* `letrec`のscopeと初期化制約:
  `OPEN-BND-001`
* `var`のescapeおよびstate-handler elaboration:
  `OPEN-BND-001`
* pattern、match、網羅性、guard:
  `OPEN-DAT-001`
* primitiveのdomain error:
  `OPEN-ERR-001`
* effect operation、handler、resumption:
  `OPEN-EFF-001`
* bounded dynamicによるruntime failure:
  `OPEN-TYP-001`
* record/variant値と評価規則:
  `OPEN-DAT-001`および`OPEN-ROW-001`
* literal、数値およびStringの詳細:
  `OPEN-SYN-002`
* runtime resource exhaustion:
  `OPEN-MEM-001`および`OPEN-ERR-001`

これらは`OPEN-EVAL-001`の未解決部分とは数えず、各拡張の導入時に
最小Coreへ追加する評価規則として追跡する。

***

#### 適合試験

##### lexical closure

```lisp
(let ((x 10))
  (let ((f (fn (y) (+ x y))))
    (let ((x 20))
      (f 1))))
```

期待結果:

```text
11
```

##### sequential `let`

```lisp
(let ((x 1)
      (y (+ x 1)))
  y)
```

期待結果:

```text
2
```

##### duplicate binder

```lisp
(let ((x 1)
      (x 2))
  x)
```

期待結果:

```text
静的エラー: 同一Surface let内のduplicate binder
```

##### application order

```lisp
(f
  (log-and-return "a" 1)
  (log-and-return "b" 2))
```

期待される観測順序:

```text
a
b
```

##### sequence result

```lisp
(seq
  (log "first")
  (log "second")
  42)
```

期待される観測順序:

```text
first
second
```

期待結果:

```text
42
```

##### selected branch only

```lisp
(if true
    (log-and-return "then" 1)
    (log-and-return "else" 2))
```

期待される観測:

```text
then
```

期待結果:

```text
1
```

##### strict Bool condition

```lisp
(if 1
    10
    20)
```

期待結果:

```text
静的エラー: expected Bool, found Integer
```

##### union result

```lisp
(if condition
    42
    "unknown")
```

期待される型:

```text
union(Integer, String)
```

##### occurrence typing

```lisp
; x : union(Number, String)

(if (number? x)
    (+ x 1)
    (string-length x))
```

期待されるbranch環境:

```text
then: x : Number
else: x : String
```

***

#### 解決後の最小Core

```text
e ::= x
    | literal
    | fn(x1, ..., xn, e)
    | app(e, e1, ..., en)
    | let(x, e1, e2)
    | if(e1, e2, e3)
    | seq(e1, ..., en)       where n >= 1

v ::= primitive-value
    | closure(environment, parameters, body)
```

この最小Coreについて、評価戦略、値、評価文脈および主要なstuck状態が
特定されたため、`OPEN-EVAL-001`を解決済みとする。

### 13.4 `BND-001` `val`、`var`、`let`、`letrec`、`fn`

> 統合元: `OPEN-BND-001`。同項目は解決済み。

#### 状態

`解決済み`

本決定は、RPX v1における次の事項を固定する。

- 通常の不変束縛
- `letrec`の対象となる右辺
- 自己再帰と相互再帰
- 再帰関数の型推論
- `(type ...)`による明示的な型注釈
- 多相再帰の扱い
- `var`のscope、読出し、更新
- closureによる局所`var`の捕捉
- local state identityのescape制約
- one-shot resumptionと局所state
- `val`、`let`および`letrec`におけるvalue restriction

次はRPX v1の基本束縛規則には含めず、別項目または将来拡張へ移管する。

- 任意式による再帰束縛
- 再帰的recordおよび無限データ構造
- recursive module
- 多相再帰
- relaxed value restriction
- escape可能なheap cell／reference
- 一般的なnon-escaping callback型
- affine kind system
- multi-shot resumption下の局所state
- closure環境およびheap cellの具体的memory管理

---

#### `DD-BND-001`: 通常の不変束縛

`確定`

RPXでは、値の不変束縛に`val`を使用する。

```lisp
(val radius 40mm)
````

関数も通常の値として`val`へ束縛する。

```lisp
(val add
  (fn (x y)
    (+ x y)))
```

named function sugarを提供する場合、次の形式を許してよい。

```lisp
(val (add x y)
  (+ x y))
```

これはhygienicに次へelaborateする。

```lisp
(val add
  (fn (x y)
    (+ x y)))
```

`define`は最終Surface構文として使用しない。

Coreの不変な局所束縛は、単一bindingの`let`として表す。

```text
let(x, initializer, body)
```

評価順序は次のとおりである。

1. `initializer`を現在の環境で評価する。
2. 評価結果を`x`へ束縛する。
3. `x`が見える環境で`body`を評価する。

新しく導入される`x`は、自身の`initializer`から参照できない。

```lisp
(let ((x initializer))
  body)
```

`initializer`内に同名の`x`が現れた場合、それは外側のbindingを参照する。
外側に対応するbindingがなければ未束縛参照となる。

***

#### `DD-BND-002`: sequentialなSurface `let`

`確定`

Surfaceの`let`は一個以上のbindingを持つことができる。

```lisp
(let ((x 10)
      (y (+ x 1)))
  (+ x y))
```

Surfaceの複数binding `let`はsequentialとする。

先行するbindingは、後続するinitializerおよび`let`本体から参照できる。

上記は、単一binding Core `let`の入れ子へ左から右にelaborateする。

```lisp
(let ((x 10))
  (let ((y (+ x 1)))
    (+ x y)))
```

parallel bindingはRPX v1の通常の`let`には採用しない。

必要になった場合は、通常の`let`と異なる明示的構文またはlibrary macroとして
導入する。

同一のSurface `let` binding list内では、同じ名前のbinderを重複してはならない。

意図的なshadowingは明示的な入れ子で表現する。

***

#### `DD-BND-003`: `(type ...)`による型注釈

`確定`

RPXでは、値bindingおよび関数bindingの明示的な型注釈を、
独立した`(type ...)`形式で記述する。

基本形は次のとおりである。

```lisp
(type name type-expression)
```

例：

```lisp
(type radius length)

(val radius 40mm)
```

関数の型注釈：

```lisp
(type add
  (fn (int int) int))

(val add
  (fn (x y)
    (+ x y)))
```

named function sugarと組み合わせる場合も、型注釈は独立した`type` formとする。

```lisp
(type add
  (fn (int int) int))

(val (add x y)
  (+ x y))
```

次のようなbinder内埋込み型注釈は、RPX v1の標準構文には採用しない。

```lisp
; 採用しない
(val add
  : (fn (int int) int)
  (fn (x y)
    (+ x y)))
```

```lisp
; 採用しない
(fn ((x : int) (y : int)) ...)
```

`(type ...)`は、同じlexical scopeにある対応する値bindingの型を宣言する。

型annotationと値bindingは、名前解決後の同一`BindingId`へ対応づける。

単なる名前文字列の一致だけで関連付けてはならない。

***

#### `DD-BND-004`: 型注釈のscopeと対応関係

`確定`

`(type name type-expression)`は、その型宣言が属するlexical declaration group内の
対応する値bindingを注釈する。

top-level例：

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (fn (x) x))
```

局所例：

```lisp
(let-declarations
  ((type increment
     (fn () int))

   (val increment
     (fn ()
       1)))
  (increment))
```

`let-declarations`という名称および正確なSurface構文は`OPEN-SYN-002`または
束縛構文の後続決定で確定する。

規範上必要なのは、局所scopeにおいても次の二種類の宣言を同じdeclaration groupへ
置けることである。

```text
type annotation declaration
value binding declaration
```

`type`宣言だけが存在し、対応する値bindingが存在しない場合は静的エラーとする。

```lisp
(type missing-value int)
```

診断例：

```text
型注釈に対応する値bindingがありません。

name:
  missing-value
```

同一declaration group内で、一つの値bindingに複数の互換でない型注釈を与えてはならない。

```lisp
(type value int)
(type value str)

(val value 1)
```

は静的エラーとする。

同一の型注釈を重複して記述することも、原則としてduplicate declaration errorとする。

***

#### `DD-BND-005`: 型注釈は検査される

`確定`

`(type ...)`は処理系への無検査の仮定ではない。

値bindingに型注釈が存在する場合、型検査器はbinding initializerをその型に対して検査する。

```lisp
(type value int)

(val value 42)
```

は受理する。

```lisp
(type value str)

(val value 42)
```

は静的エラーとする。

概念的な規則：

```text
Γ ⊢ annotation-type : type
Γ ⊢ initializer ⇐ annotation-type ! E
──────────────────────────────────────
bindingはannotation-typeを持つ
```

型注釈は、initializerの実際の型と矛盾してはならない。

subtypingを許可する文脈では、initializerの型がannotation typeのsubtypeであれば
受理できる。

型注釈にdynamic boundaryが含まれる場合は、通常のbounded dynamic cast規則に従う。

型注釈は、expansive expressionを構文的valueへ変換しない。

したがって、型注釈を付けてもvalue restrictionを迂回できない。

***

#### `DD-BND-006`: `letrec`の対象

`確定`

RPX v1は、再帰関数を定義するための`letrec`を持つ。

`letrec`の各値bindingの右辺は、Surface elaboration後に構文的な`fn`でなければならない。

```lisp
(letrec ((factorial
           (fn (n)
             (if (= n 0)
                 1
                 (* n (factorial (- n 1)))))))
  (factorial 5))
```

次のような任意式による再帰束縛は認めない。

```lisp
(letrec ((x (+ x 1)))
  x)
```

```lisp
(letrec ((x x))
  x)
```

次も、initializerの結果型が関数であるという理由だけでは認めない。

```lisp
(letrec ((f (make-function f)))
  ...)
```

`make-function`の評価中に再帰bindingを参照する可能性があり、
初期化前参照を防げないためである。

RPX v1の`letrec`が直接扱うのは、再帰関数および相互再帰関数だけとする。

次は対象外とする。

* 一般の再帰的な値
* 再帰的record
* 再帰的handler value
* effectful initializer
* strictな無限データ構造
* 初期化前に値を読み出す再帰定義

***

#### `DD-BND-007`: `letrec`内の型注釈

`確定`

再帰関数についても、型注釈は独立した`(type ...)` formで記述する。

自己再帰関数の概念例：

```lisp
(letrec
  ((type factorial
     (fn (int) int))

   (val factorial
     (fn (n)
       (if (= n 0)
           1
           (* n (factorial (- n 1)))))))
  (factorial 5))
```

相互再帰の概念例：

```lisp
(letrec
  ((type even?
     (fn (int) bool))

   (type odd?
     (fn (int) bool))

   (val even?
     (fn (n)
       (if (= n 0)
           true
           (odd? (- n 1)))))

   (val odd?
     (fn (n)
       (if (= n 0)
           false
           (even? (- n 1))))))
  (even? 10))
```

`letrec`内のdeclaration groupは、少なくとも次を含められる。

```text
type annotation declaration
recursive function value declaration
```

型注釈は省略可能である。

無注釈の場合、型検査器が再帰関数の型を推論する。

一部の関数だけに型注釈を付けることも許す。

```lisp
(letrec
  ((type parse-block
     (fn (block-token) block))

   (val parse-block
     (fn (token)
       ...))

   (val parse-inline
     (fn (token)
       ...)))
  ...)
```

この場合、`parse-block`は注釈型に対して検査し、
`parse-inline`の型はグループ全体の制約から推論する。

***

#### `DD-BND-008`: `letrec`の実行意味

`確定`

`letrec`は、すべての再帰関数が同じ再帰環境を捕捉するように評価する。

概念的には次の循環環境を構成する。

```text
ρrec =
ρ[
  f1 ↦ closure(ρrec, parameters1, body1),
  ...
  fn ↦ closure(ρrec, parametersn, bodyn)
]
```

評価手順は次のとおりである。

1. 再帰関数グループ用の環境を作る。
2. 各関数について、その再帰環境を捕捉するclosureを作る。
3. すべての再帰binderを対応するclosureで初期化する。
4. すべての初期化完了後に`letrec`本体を評価する。

`fn`を評価してclosureを生成する時点では、関数本体を評価しない。

したがって、well-formedなRPX v1の`letrec`では、利用者コードから
未初期化bindingを観測できない。

***

#### `DD-BND-009`: 相互再帰

`確定`

RPX v1は、一つの`letrec` declaration group内で相互再帰を許す。

```lisp
(letrec
  ((val even?
     (fn (n)
       (if (= n 0)
           true
           (odd? (- n 1)))))

   (val odd?
     (fn (n)
       (if (= n 0)
           false
           (even? (- n 1))))))
  (even? 10))
```

同一グループ内のすべての再帰値binderは、次から参照できる。

* グループ内のすべての再帰関数本体
* `letrec`本体

型注釈宣言も、同じdeclaration group内の対応する値bindingを前方参照できる。

相互再帰を、関数内関数またはdispatcher関数への手動変換で代用させない。

一関数だけを含む`letrec`は通常の自己再帰となる。

moduleをまたぐ相互再帰とrecursive moduleは`OPEN-MOD-001`へ移管する。

***

#### `DD-BND-010`: 再帰関数の型推論

`確定`

無注釈の再帰グループでは、型検査器は各再帰関数へfreshな単相metavariableを割り当てる。

概念例：

```text
even? : a -> b
odd?  : c -> d
```

その後、グループ内のすべての関数本体を同じ仮型環境で検査する。

```text
Γ,
even? : a -> b,
odd?  : c -> d
⊢ even-body

Γ,
even? : a -> b,
odd?  : c -> d
⊢ odd-body
```

生成された制約をグループ単位で解く。

再帰グループの検査中、各再帰binderは単相として扱う。

グループの検査完了後に限り、通常のvalue restrictionとgeneralization規則に従って
rank-1一般化を行うことができる。

型検査器が型を決定できない場合、必要な関数への`(type ...)`注釈を要求する。

診断例：

```text
この再帰グループの型を局所推論だけでは決定できません。

次のbindingへ型注釈を追加してください:
  parse-inline

例:
  (type parse-inline
    (fn (inline-token) inline-result))
```

***

#### `DD-BND-011`: 多相再帰の禁止

`確定`

RPX v1は多相再帰を許可しない。

再帰グループ内では、すべての再帰呼出しが同じ単相型を共有する。

```text
再帰グループ内部:
monomorphic recursion

再帰グループ検査完了後:
許可されたrank-1 generalization
```

`(type ...)`による明示的な型注釈を付けても、多相再帰は有効化しない。

多相再帰は将来拡張として保留する。

***

#### `DD-BND-012`: `var`の基本意味

`確定`

RPX v1の`var`は、escape不能な局所可変状態を表す。

基本Surface構文は明示的なbodyを持つ。

```lisp
(var count 0
  (set count (+ count 1))
  count)
```

概念的なCoreまたはelaboration中間形：

```text
local-var(binding, initializer, body)
```

評価順序は次のとおりである。

1. `initializer`を外側の環境で評価する。
2. initializerが正常終了した後、freshなlocal state identityを生成する。
3. initializerの結果を局所stateの初期値とする。
4. bodyを局所state handlerのscope内で評価する。
5. bodyの結果を`var`式全体の結果とする。
6. scope終了時に局所stateを破棄する。

新しい`var` bindingは、自身のinitializerから参照できない。

initializerには通常のeffectを許す。

initializerが異常終了した場合はlocal state instanceを生成しない。

***

#### `DD-BND-013`: `var`の型注釈

`確定`

`var`についても、格納型を明示する場合は独立した`(type ...)` formを使用する。

概念例：

```lisp
(var-declarations
  ((type value
     (union int str))

   (var value 0))
  (set value "text")
  value)
```

正確なSurface構文名は後続の構文仕様で確定する。

必要な規範上の性質は次のとおりである。

* `type`宣言と`var` bindingを同じlexical declaration groupで関連付けられる
* 注釈がない場合はinitializerから格納型を推論する
* 注釈がある場合はinitializerをその型に対して検査する
* すべての`set`右辺も同じ格納型に対して検査する

単純な一binding形式へ型注釈を直接含める糖衣を将来追加する場合も、
意味上は独立した`type` declarationへelaborateする。

次のcolon形式は基本規範構文にはしない。

```lisp
; 採用しない
(var value : (union int str) 0
  ...)
```

***

#### `DD-BND-014`: `var`の格納型は固定

`確定`

`var`の格納型は、initializerまたは対応する`(type ...)`宣言から一度決定し、
その後は固定する。

無注釈例：

```lisp
(var value 0
  ...)
```

initializerが`int`なので、格納型は`int`となる。

後続の代入から格納型を自動的にunionへ拡張してはならない。

```lisp
(var value 0
  (set value "text")
  value)
```

は静的エラーである。

複数型を保存したい場合は、明示的な型注釈を使用する。

概念例：

```lisp
(var-declarations
  ((type value
     (union int str))

   (var value 0))
  (set value "text")
  value)
```

型注釈がある場合、initializerおよびすべての`set`右辺が注釈型へ適合しなければならない。

***

#### `DD-BND-015`: `var`の読出し

`確定`

Surface上では、`var`の現在値を通常の変数参照と同じ綴りで読み出す。

```lisp
(var count 0
  (+ count 1))
```

名前解決後のCoreでは、不変bindingの参照と局所`var`の読出しを区別する。

```text
不変binding:
read-immutable(binding-id)

局所var:
read-local-state(state-instance-id)
```

effect handlerへのelaboration後は、概念的に次へ変換できる。

```text
get<state-instance-id>
```

利用者へ明示的な`get`構文を要求しない。

***

#### `DD-BND-016`: `set`

`確定`

局所`var`の更新には特殊形式`set`を使用する。

```lisp
(set count value)
```

`set`は通常関数ではなく、第一引数を局所可変bindingとして名前解決する。

名前解決後の概念形：

```text
set-local-state(state-instance-id, value-expression)
```

評価順序は次のとおりである。

1. 更新対象bindingを解決する。
2. 右辺expressionを評価する。
3. 右辺の値が格納型へ適合することを確認する。
4. 局所stateを更新する。
5. `unit`を返す。

`set`の結果型は常に`unit`である。

不変な`val`または`let` bindingに`set`してはならない。

```lisp
(let ((x 10))
  (set x 20))
```

は静的エラーとする。

***

#### `DD-BND-017`: closureによる`var`のcapture

`確定`

内側のclosureが、外側の`var`をcaptureすることを許す。

```lisp
(var count 0
  (let ((increment
          (fn ()
            (set count (+ count 1))
            count)))
    (seq
      (increment)
      (increment))))
```

closureを`var`のscope内で作成し、scope内で使用することは有効である。

問題となるのはcapture自体ではなく、captureしたclosureがlocal stateのscope外へ
escapeすることである。

***

#### `DD-BND-018`: local state identity

`確定`

各`var`の評価時にfreshなlocal state identity`s`を生成する。

```text
var count
→ fresh local state identity s
```

`count`の読出しおよび更新は、概念的に次のoperationを使用する。

```text
get<s>
put<s>
```

`count`をcaptureするclosureは、型または内部capture setに`s`への依存を持つ。

概念的な関数型：

```text
unit ->{local-state<s>} int
```

`s`は生成元の`var` scopeに限定される。

異なる`var`評価から生成されたidentityは、同じsource bindingに由来していても異なる。

***

#### `DD-BND-019`: local state escapeの禁止

`確定`

local state identity`s`は、生成元`var`のscope外へescapeしてはならない。

少なくとも次をescapeとして扱う。

* `var`をcaptureしたclosureを結果として返す
* そのclosureを返却されるrecordまたはvariantへ格納する
* 外側のheap cellまたはmutable storeへ保存する
* moduleまたはglobal registryへ登録する
* public APIへexportする
* `any`または`dynamic any`へ変換してscope identityを消去する
* foreign runtimeへ渡す
* scope外で使用され得るcallbackとして登録する

概念的な条件：

```text
s ∉ free-scopes(result-type)
s ∉ residual-effects
s ∉ exported-captures
s ∉ external-store-effects
```

状態付きclosureを返す次の形式は静的エラーとする。

```lisp
(val make-counter
  (fn ()
    (var count 0
      (fn ()
        (set count (+ count 1))
        count))))
```

escape可能な状態が必要な場合は、将来の`cell`または`ref`機構を使用する。

***

#### `DD-BND-020`: non-escaping callback

`確定`

局所stateをcaptureしたclosureは、同じscope内で直接呼び出してよい。

また、呼出し先がそのclosureを保存せず、呼出し中だけ使用することを
型または組込み契約で保証できる場合は、non-escaping callbackとして渡してよい。

```lisp
(var count 0
  (for-each items
    (fn (item)
      (set count (+ count 1)))))
```

これを許可するには、`for-each`がcallbackをscope外へ保存しないことを保証しなければならない。

RPX v1では次のいずれかに限定する。

* closureの直接呼出し
* 同じscope内の局所関数への引渡し
* コンパイラまたは標準ライブラリがnon-escapingと認識する高階関数
* 将来定義されるscoped parameter型を持つ関数

non-escaping性を証明できない一般の高階関数への引渡しは、保守的に拒否してよい。

***

#### `DD-BND-021`: local state effectの除去

`確定`

`var` body内の局所state操作は、fresh identity`s`に対応するlocal state effectを持つ。

```text
body :
a ! <local-state<s> | e>
```

`var`はこのeffectを内部handlerによって処理する。

local state identity`s`がscope外へescapeしない場合、`var`式全体から
`local-state<s>`を一層除去する。

```text
initializer :
t ! ei

body :
a ! <local-state<s> | e>

var expression :
a ! f
```

ここで`f`は、initializerのeffect`ei`とbodyの残余effect`e`を満たすambient rowである。

```text
ei ⊑ f
e  ⊑ f
```

局所stateに対応するeffectは外側へ残さない。

***

#### `DD-BND-022`: one-shot resumptionと局所state

`確定`

局所stateを含む計算がoperationによって中断され、one-shot resumptionとして
捕捉された場合、そのresumptionは同じlocal state instanceを保持する。

```text
operation発生時:
state = v

resume:
同じinstanceで再開

再開後:
state = vから継続
```

stateを複製しない。

resumptionを使用せず破棄した場合は、捕捉されたscopeをunwindし、
local state instanceを破棄する。

forwardした場合は、resumptionと共にlocal state scopeの継続責任を外側へ移譲する。

multi-shot resumption下でstateを共有するか複製するかは、multi-shot導入時まで保留する。

***

#### `DD-BND-023`: escape可能な状態との分離

`確定`

RPX v1の`var`はescape不能な局所状態だけを表す。

状態付きclosureを返す用途など、scope外へ生存する状態が必要な場合は、
`var`のescape制約を緩和して表現しない。

将来、必要に応じて次のような明示的heap state機構を導入できる。

```text
cell<a>
ref<a>
```

この機構は、少なくとも次を別途規定する。

* runtime identity
* heap lifetime
* read／write effect
* closure間の共有規則
* memory management
* equality／hashの可否
* serializationの可否
* GUI再評価時の寿命
* provenanceおよびsource versionとの関係

`cell`／`ref`の仕様は`OPEN-MEM-001`へ移管する。

***

#### `DD-BND-024`: 型一般化point

`確定`

RPX v1で型一般化を行う場所は原則として次とする。

* `val` binding
* `let` binding
* `letrec`グループの検査完了後

次は一般化pointではない。

* `type` annotation declarationそのもの
* 関数parameter
* `var` binding
* pattern binder
* operation clause parameter
* resumption binder
* named effect evidence
* local state identity
* resource capability

一般化はrank-1に限定する。

`(type ...)`宣言は、値の型を制約・公開するものであり、それ自体が新しいruntime bindingを
作るものではない。

***

#### `DD-BND-025`: 構文的value restriction

`確定`

RPX v1は、ML系に近い厳格な構文的value restrictionを採用する。

`val`または`let`のinitializerが、規定されたgeneralizable syntactic valueである場合に限り、
型変数、record-row変数およびeffect-row変数を一般化する。

評価結果が最終的にvalueになることだけでは不十分である。

```text
結果がvalue
≠
構文的value
```

generalizable syntactic valueには、少なくとも次を含める。

* literal
* `fn`
* handler literal
* generalizable valueだけからなる不変record
* generalizable valueをpayloadとするvariant constructor
* 評価時に利用者コードやeffectを実行しない不変value form

一般化しない式には、少なくとも次を含める。

* function application
* effectful computation
* `var`の読出し
* `set`
* handlerの適用
* dynamic import
* implicit cast
* explicit safe cast
* foreign validation
* resource取得
* その他のexpansive expression

`(type ...)`注釈を付けても、expansive expressionを構文的valueとして扱ってはならない。

***

#### `DD-BND-026`: effectful function valueの一般化

`確定`

`fn`の本体がeffectfulであっても、closure生成自体がpureであれば、
`fn`はgeneralizable syntactic valueである。

```lisp
(type log-value
  (forall ((a type))
    (fn (a)
        (effects log)
        a)))

(val log-value
  (fn (x)
    (log x)
    x))
```

次を区別する。

```text
closure生成時のeffect:
<>

関数呼出し時のlatent effect:
<log>
```

latent effectが存在することだけを理由に、関数値の一般化を禁止しない。

同様に、handler clauseがeffectfulでも、handler value生成自体がpureで、
scopedまたはaffineな値をcaptureしないならhandler literalを一般化できる。

***

#### `DD-BND-027`: capability captureによる一般化禁止

`確定`

構文的にはvalueであっても、次をcaptureする場合は一般化しない。

* local `var`
* local state identity
* affine resumption
* named/scoped effect evidence
* private resource capability
* source transaction authority
* その他、scopeまたは使用回数に制約を持つ値

例：

```lisp
(var count 0
  (let ((increment
          (fn ()
            (set count (+ count 1))
            count)))
    ...))
```

`increment`は構文的には`fn`だが、local state identityをcaptureするため単相とする。

型検査器は構文形だけでなく、valueのcapture setおよびscope依存性も確認する。

***

#### `DD-BND-028`: 明示的`forall`注釈

`確定`

明示的な多相型注釈も`(type ...)`内で記述する。

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (fn (x) x))
```

明示的`forall`注釈が存在する場合でも、initializerがvalue restrictionを満たさないなら、
その注釈によってgeneralizationを強制してはならない。

```lisp
(type generated-identity
  (forall ((a type))
    (fn (a) a)))

(val generated-identity
  (make-identity))
```

`make-identity`がfunction applicationである場合、initializerはexpansiveである。

このbindingを上記の多相型で受理してはならない。

診断例：

```text
このbindingは多相型として一般化できません。

理由:
  initializerが構文的valueではありません

annotation:
  forall a. a -> a
```

明示的注釈は推論を補助し、結果を検査するが、unsafeなgeneralizationを許可する機能ではない。

***

#### `DD-BND-029`: 一般化される変数

`確定`

一般化可能なbindingの型を`t`、外側の環境を`Γ`とする。

一般化する変数は概念的に次である。

```text
free-variables(t)
-
free-variables(Γ)
```

一般化対象をkindごとに区別する。

* 通常の型変数
* record-row変数
* effect-row変数

local scope identity、affine capability identity、private runtime authority等は
一般化対象にしない。

明示的な`forall`注釈がある場合は、そのquantifierのkindとscopeを検査し、
initializerの推論型または検査結果が注釈schemeへ適合することを確認する。

***

#### `DD-BND-030`: 一般化されないmetavariable

`確定`

一般化不可のbindingに未解決metavariableが存在する場合、それらを
monomorphicなweak metavariableとして扱う。

同一bindingのすべての使用箇所で同じmetavariableを共有する。

最初の使用で型が確定した後は、別の型へ再instantiateできない。

明示的な`(type ...)`宣言を後から追加して、既に別の型へ確定したweak metavariableを
不整合に一般化してはならない。

***

#### `DD-BND-031`: `letrec`とvalue restriction

`確定`

`letrec`グループの検査中、すべての再帰binderは単相である。

グループ内の全関数本体を検査し、制約を解決した後に限り、
グループ外向けの一般化を検討する。

各右辺は構文的`fn`なので、通常はgeneralizable valueになり得る。

ただし、次の場合は一般化しない。

* local stateをcaptureする
* scoped evidenceをcaptureする
* affine capabilityをcaptureする
* 外側環境に未一般化metavariableが現れる
* その他、通常のvalue restrictionを満たさない

対応する`(type ...)`宣言がある場合は、推論結果をその型schemeに対して検査する。

多相再帰は許可しない。

***

#### `DD-BND-032`: relaxed value restrictionの保留

`確定`

RPX v1はrelaxed value restrictionを導入しない。

varianceまたは型変数の出現位置に基づいて、expansive expressionの一部を追加で一般化することは
行わない。

必要性が確認された場合は、`OPEN-TYP-002`でsoundness、decidabilityおよび
semantic subtypingとの相互作用を検証したうえで将来拡張として検討する。

***

#### Coreおよびdeclaration elaboration

型注釈を含むSurface declaration groupを、名前解決と型検査のため次のように整理する。

Surface例：

```lisp
(type add
  (fn (int int) int))

(val add
  (fn (x y)
    (+ x y)))
```

declaration elaboration：

```text
1. `type add ...`からannotation declarationを作る。
2. `val add ...`からvalue binding declarationを作る。
3. 同一scopeの名前解決によって両者を同じBindingIdへ関連付ける。
4. annotation typeをkind checkする。
5. initializerをannotation typeに対して検査する。
6. value restrictionに基づいてgeneralization可否を判定する。
7. runtime Coreには値bindingだけを残す。
8. 型注釈はtyped Core metadataおよびdiagnostic provenanceへ保持する。
```

`type` annotation declarationはruntimeで評価されない。

したがって、Core evaluatorの項として独立した`type`式を持たせる必要はない。

ただし、typed Core nodeは次を保持してよい。

```text
BindingMetadata {
  binding-id,
  inferred-type,
  declared-type?,
  declaration-span,
  annotation-span?
}
```

***

#### 適合試験

##### top-level型注釈

```lisp
(type radius length)

(val radius 40mm)
```

期待：

```text
受理
radius : length
```

##### 関数型注釈

```lisp
(type add
  (fn (int int) int))

(val add
  (fn (x y)
    (+ x y)))
```

期待：

```text
受理
```

##### 注釈不一致

```lisp
(type value str)

(val value 42)
```

期待：

```text
静的エラー
```

##### 対応bindingのない型注釈

```lisp
(type missing int)
```

期待：

```text
静的エラー:
型注釈に対応する値bindingがない
```

##### 自己再帰の型注釈

```lisp
(letrec
  ((type factorial
     (fn (int) int))

   (val factorial
     (fn (n)
       (if (= n 0)
           1
           (* n (factorial (- n 1)))))))
  (factorial 5))
```

期待：

```text
120
```

##### 相互再帰の型注釈

```lisp
(letrec
  ((type even?
     (fn (int) bool))

   (type odd?
     (fn (int) bool))

   (val even?
     (fn (n)
       (if (= n 0)
           true
           (odd? (- n 1)))))

   (val odd?
     (fn (n)
       (if (= n 0)
           false
           (even? (- n 1))))))
  (even? 10))
```

期待：

```text
true
```

##### 一部だけ注釈

```lisp
(letrec
  ((type even?
     (fn (int) bool))

   (val even?
     (fn (n)
       (if (= n 0)
           true
           (odd? (- n 1)))))

   (val odd?
     (fn (n)
       (if (= n 0)
           false
           (even? (- n 1))))))
  (odd? 9))
```

期待：

```text
odd?の型をグループ制約から推論
結果はtrue
```

##### 任意式の再帰拒否

```lisp
(letrec
  ((val x
     (+ x 1)))
  x)
```

期待：

```text
静的エラー:
letrecの値binding initializerはfnでなければならない
```

##### 明示的多相型

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (fn (x) x))
```

期待：

```text
受理

identity :
forall a. a -> a
```

##### 注釈によるvalue restriction回避の拒否

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (make-identity))
```

`make-identity`がfunction applicationの場合の期待：

```text
静的エラー:
expansive initializerを明示注釈で多相一般化できない
```

##### local `var`

```lisp
(var count 0
  (set count (+ count 1))
  (set count (+ count 1))
  count)
```

期待：

```text
2
```

##### `set`の戻り値

```lisp
(var count 0
  (set count 1))
```

期待される結果型：

```text
unit
```

##### 格納型の固定

```lisp
(var value 0
  (set value "text")
  value)
```

期待：

```text
静的エラー:
expected int, found str
```

##### 明示union格納型

概念例：

```lisp
(var-declarations
  ((type value
     (union int str))

   (var value 0))
  (set value "text")
  value)
```

期待：

```text
受理
```

##### closure capture

```lisp
(var count 0
  (let ((increment
          (fn ()
            (set count (+ count 1))
            count)))
    (seq
      (increment)
      (increment))))
```

期待：

```text
2
```

##### closure escape

```lisp
(val make-counter
  (fn ()
    (var count 0
      (fn ()
        (set count (+ count 1))
        count))))
```

期待：

```text
静的エラー:
local state captured by escaping closure
```

##### effectful関数値の一般化

```lisp
(type log-value
  (forall ((a type))
    (fn (a)
        (effects log)
        a)))

(val log-value
  (fn (x)
    (log x)
    x))
```

期待：

```text
受理
```

***

#### 未解決事項の移管

##### `OPEN-SYN-002`

* top-levelおよび局所declaration groupの完全な構文
* `letrec`内で`type`と`val`を並べる正確な括弧構造
* `var`と対応する`type`宣言を置く局所構文
* `val (f x)` sugarの完全な構文
* `forall`、kind annotation、effect rowの表面構文
* declaration orderingとforward annotation
* duplicate annotation diagnostic
* `type` annotationとtype alias declarationの構文上の区別

##### `OPEN-TYP-002`

* 注釈付き／無注釈再帰グループのalgorithm
* annotation subsumption
* explicit `forall`のchecking
* annotationとprincipal typingの関係
* non-escaping callbackの判定
* generalizable syntactic valueの正確なalgorithmic分類
* semantic subtyping下のgeneralization
* checker limitationによる注釈要求

##### `OPEN-MEM-001`

* escape可能な`cell`／`ref`
* heap allocation
* closure環境のmemory管理
* shared mutable cell
* cyclic state
* stateful closureのidentity
* GUI再評価をまたぐstate lifetime

##### `OPEN-MOD-001`

* public valueへの`(type ...)`注釈要件
* signature内の`type`とvalue annotationの区別
* module間相互再帰
* recursive module
* exported closureのscope検査

##### 将来拡張

* 多相再帰
* lazy recursive value
* recursive record
* recursive handler value
* affine kind system
* 一般的なborrowed closure
* relaxed value restriction
* multi-shot state semantics

***

#### 解決後の基本原則

```text
不変値binding:
val

型注釈:
type

再帰関数binding:
letrec内のval

局所可変状態:
var

更新:
set
```

```text
型注釈:

(type name type-expression)

(val name initializer)
```

```text
型注釈は:
- 省略可能
- 対応する値bindingを検査する
- runtime bindingを作らない
- value restrictionを迂回しない
```

```text
letrec:
- initializerはfnのみ
- 自己再帰と相互再帰を許可
- グループ内にtype annotationを置ける
- グループ内部では単相
- 検査後にrank-1一般化可能
```

```text
var:
- 明示bodyを持つ
- initializerから新bindingは見えない
- 格納型はinitializerまたはtype annotationから決定
- setはunit
- closure captureは可能
- scope外escapeは禁止
```

```text
一般化:
- val／let／letrec検査後
- generalizable syntactic valueのみ
- affine／scoped capabilityをcaptureしないこと
- explicit type annotationは一般化条件を変更しない
```

以上により、RPX v1の再帰関数、相互再帰、独立した`(type ...)`型注釈、
局所可変状態、scope escapeおよび構文的value restrictionが特定されたため、
`OPEN-BND-001`を解決済みとする。

### 13.5 `MAC-001` Hygienic macro

#### 概要・目的・状態

すべての最終マクロをhygienicにすることは`確定`。用途を以下へ限定する。

1. 新しい束縛構文
2. 評価順序・回数の制御
3. 遅延評価やpattern syntax
4. two-faced reader syntax
5. 通常の関数適用で表現できない構文

通常の図形、組版、アニメーション抽象化は関数を優先する。

#### 構文

`define-syntax`という過去例は`暫定`であり、`define`回避方針と用語上衝突する。
候補:

```lisp
(syntax when
  ...)
```

または:

```lisp
(val-syntax when
  ...)
```

最終名称は`OPEN-MAC-001`。展開は型検査前。macroはsyntax objectを受け取りsyntax
objectを返し、lexical scope markとoriginを保持する。

#### 静的・動的意味

- expansionはcompile-time phase。
- macro自体は実行時値ではない。
- runtime valueを直接参照しない。
- import-for-syntaxとruntime importを分離する。
- 各macro expansion stepはfresh introduction scopeを作り、そこで導入されたbinderと
  その参照が適切なscope setを共有する。入力からコピーしたidentifierはuse-site
  contextを保持する。各identifierへ無関係なfresh scopeを個別付与してはならない。
- intentional capture APIは未決定。
- expansion termination/fuel、filesystem access、type-directed macroは未決定。

Macro expansion後のCoreだけを通常の型安全性定理の対象とする。Surface soundnessには、
expansion成功時にwell-scoped Coreを生成することが必要だが未証明。

#### 例・反例

```lisp
(when condition body)
```

が`if`へ展開されることは可能。導入したtemporaryが利用者の同名変数をcaptureしては
ならない。

`CE-MAC-001`: macroがmoduleのprivate type名をsyntax datumとして外部へ埋め込むと
sealingを破り得る。macro authorityとcertificateが必要。

#### 実装・テスト

必要構造: syntax object、scope set/mark、phase、binding identity、origin chain、
expansion trace、fuel counter。

- `TEST-MAC-001`: alpha-renaming不変。
- `TEST-MAC-002`: adversarial capture。
- `TEST-MAC-003`: nested macro source span。
- `TEST-MAC-004`: phase cycle拒否。
- `TEST-MAC-005`: expansion fuel。
- `TEST-MAC-006`: macro×sealed module attack。
- `TEST-MAC-007`: pretty diagnosticにexpansion stack。

現行のtext splice/fixed-point macroは移行実装であり、最終hygienic macro仕様ではない。

### 13.6 `TYP-001` Gradual set-theoretic types

#### 概要・状態

- 集合論的漸進型、semantic subtyping、bounded dynamic: `確定`。
- algorithmic checkerの完全判定範囲、三値判定、bidirectional typing: `確定`。
- `OPEN-TYP-001`および`OPEN-TYP-002`: `解決済み`。
- 本節は、従来の未決定記述を置き換える。

#### 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure

> 統合元: `OPEN-TYP-001`。同項目は解決済み。

##### 状態

`解決済み`

本決定は、RPX v1におけるbounded dynamicについて、次を固定する。

- `dynamic S`の意味
- static型および`any`との関係
- dynamic値の生成境界
- dynamic値をstatic型として使用する条件
- runtime castの挿入
- cast成功後の型
- cast failureの分類
- cast evidenceの基本構造
- record、variant、function、abstract typeのcast方針
- effectful function castの制限
- polymorphismとdynamic境界
- gradual guaranteeの設計目標
- 数値promotionとdynamic castの順序

次はRPX v1のbounded dynamicには含めず、将来拡張または別の`OPEN-*`へ移管する。

- 一般的なblame polarityとblame theorem
- polymorphic dynamic
- dynamic値から`forall`型へのcast
- gradual effect-row cast
- runtime effect monitoring
- dynamic handler cast
- dynamic resumption cast
- scoped／affine capabilityのdynamic化
- 一般的なanswer-type modification
- cast failureと例外effectの統一
- space-efficient cast calculusの完全な最適化規則
- gradual guaranteeの形式証明

---

##### `DD-TYP-DYN-001`: static型とgradual型の分離

`確定`

RPXの型を、概念上次の二つに分ける。

```text
S ::= static type

G ::= S
    | dynamic S
````

`S`はstatic semantic typeであり、少なくとも次を含む。

```text
S ::= any
    | never
    | base-type
    | union(S+)
    | intersect(S+)
    | not(S)
    | diff(S, S)
    | function-type
    | record-type
    | variant-type
    | type-variable
    | recursive-type
```

`dynamic S`は、static型`S`を上限として持つgradual型である。

`dynamic S`は次のどちらとも型等価ではない。

```text
dynamic S ≄ S
dynamic S ≄ any
```

static fragmentにおけるsemantic subtypingおよび型等価は、従来どおり値集合によって
定義する。

```text
S <: T
iff
[[S]] ⊆ [[T]]
```

```text
S ≃ T
iff
S <: T and T <: S
```

`dynamic S`はstatic semantic typeの値集合演算へ直接混入させない。

特に、次のような演算をstatic型代数の通常演算として定義してはならない。

```text
not(dynamic S)
diff(T, dynamic S)
```

dynamicを含むcompatibility、precision、cast insertionは、static semantic subtypingとは
別の判断として定義する。

***

##### `DD-TYP-DYN-002`: `dynamic S`の意味

`確定`

```text
dynamic S
```

は、次の意味を持つ。

> 実行時値がstatic型`S`の意味領域に属することは保証されているが、
> その値をどの程度精密なstatic型として使用できるかは、runtime情報に依存する。

したがって、次を不変条件とする。

```text
x : dynamic S
ならば
runtime value of x ∈ [[S]]
```

`dynamic S`は「おそらく`S`」または「未検証の値」を意味しない。

型保証のない値を`dynamic S`へ導入する場合は、dynamic値を生成する境界で、
値が実際に`S`へ属することを検証しなければならない。

例：

```text
dynamic(union(number, str))
```

は、実行時値が次のいずれかであることを保証する。

```text
int
f64
str
```

この上限と交わらない型として値を利用しようとした場合は、runtimeまで待たず
静的に拒否する。

***

##### `DD-TYP-DYN-003`: static top型`any`

`確定`

RPXは、static型代数のtop型として`any`を持つ。

```text
[[any]]
=
すべてのunrestricted RPX runtime value
```

すべてのunrestricted static型`S`について、次が成り立つ。

```text
S <: any
```

`any`はdynamic型ではなく、runtime castの暗黙挿入を許可しない。

```text
x : any
```

を型`T`が必要な位置で使用する場合、静的に`any <: T`が成立しない限り拒否する。

```lisp
; x : any

(+ x 1)
```

は静的エラーである。

これに対して、

```text
x : dynamic any
```

を数値として使用する場合は、runtime narrowingを挿入できる。

```text
any
≠
dynamic any
```

`any`はsemantic negationおよびdifferenceの基準となるstatic universeである。

```text
union(T, not(T)) ≃ any
intersect(T, not(T)) ≃ never
diff(any, T) ≃ not(T)
```

RPX v1の`any`が含むのは、通常のunrestricted valueだけとする。

次は、一般の`any`または`dynamic any`へupcastできない。

* affine resumption
* named/scoped effect evidence
* handler instance frame
* 局所resource capability
* private runtime authority
* raw foreign pointer
* その他、scopeまたは使用回数の制約を消去すると安全性が失われる値

これらは通常のunrestricted value universeの外側にある、専用の型付きcapabilityとして
扱う。

`any`という名称は表面型名として採用する。ただし、`any`が型検査回避機構ではなく
static top型であることを仕様と診断で明示する。

***

##### `DD-TYP-DYN-004`: `never`およびdynamicの正規形

`確定`

`never`はstatic型代数の空型とする。

```text
[[never]] = ∅
```

すべてのstatic型`S`について次が成り立つ。

```text
never <: S
union(S, never) ≃ S
intersect(S, never) ≃ never
```

`dynamic S`の上限`S`はstatic型だけに限定する。

次の型は表面型として認めない。

```text
dynamic(dynamic S)
```

複数のdynamic境界が連続した場合、型表現は一重の`dynamic S`へ正規化するが、
各境界のprovenanceは保持する。

```text
型:
dynamic S

境界履歴:
boundary-1
→ boundary-2
```

`dynamic never`には正常なruntime値が存在しないため、次へ正規化する。

```text
dynamic never
≃
never
```

***

##### `DD-TYP-DYN-005`: dynamic値をstatic型として使用する三段階判定

`確定`

```text
x : dynamic S
```

を、static型`T`が必要な位置で使用するとき、次の三段階で判定する。

###### 1. 上限全体が要求型へ含まれる場合

```text
S <: T
```

なら、runtime checkなしで使用を許可する。

```text
dynamic int
→ expected number
```

では、

```text
int <: number
```

なので、runtime narrowingは不要である。

###### 2. 上限と要求型が互いに素である場合

```text
intersect(S, T) ≃ never
```

なら、runtime checkは必ず失敗するため静的に拒否する。

```text
dynamic(union(number, str))
→ expected picture
```

かつ、

```text
intersect(union(number, str), picture)
≃ never
```

なら静的エラーとする。

###### 3. 一部だけ重なる場合

次を共に満たす場合、

```text
S </: T
intersect(S, T) ≄ never
```

runtime checkを伴うcastをtyped Coreへ挿入する。

```text
dynamic(union(number, str))
→ expected number
```

では、実行時値が`number`なら成功し、`str`なら失敗する。

この判定をまとめると次のとおりである。

```text
S <: T
  → runtime checkなし

intersect(S, T) ≃ never
  → 静的エラー

それ以外
  → runtime castを挿入
```

***

##### `DD-TYP-DYN-006`: cast成功後の型

`確定`

```text
dynamic S
```

をstatic型`T`へruntime checkし、検査に成功した場合、結果のstatic型は単なる`T`ではなく
次とする。

```text
intersect(S, T)
```

例：

```text
source bound:
union(int, str)

required type:
number
```

の場合、

```text
intersect(union(int, str), number)
≃ int
```

なので、cast成功後の値は`number`ではなく、より精密な`int`として扱える。

これは、元の上限が持っていたstatic情報をcastによって失わないためである。

利用箇所では次が成り立つ。

```text
intersect(S, T) <: T
```

したがって、要求型`T`の値として安全に使用できる。

***

##### `DD-TYP-DYN-007`: occurrence typingとの関係

`確定`

認識可能な型判定により、dynamic値の型をbranch内で絞り込む。

```text
x : dynamic S
```

について、型`T`への所属判定を行った場合、then branchでは、runtime検査が成功済みであるため
static型へ移行する。

```text
then branch:
x : intersect(S, T)
```

else branchでは、`T`でないことは分かるが、残余範囲内の正確な型は未確定であるため、
原則としてdynamicを維持する。

```text
else branch:
x : dynamic(diff(S, T))
```

`diff(S, T) ≃ never`なら、else branchは到達不能である。

例：

```text
x : dynamic(union(number, str))
```

```lisp
(if (number? x)
    (+ x 1)
    (str-length x))
```

branch環境は概念的に次となる。

```text
then:
x : number

else:
x : dynamic str
```

残余範囲が単一のruntime-check済み型として利用可能であることを型検査器が証明できる場合は、
else側もstatic型へ精密化してよい。

ただし、可変変数、effectful expression、property pathおよび再評価によって値が変化し得る
場所には、この単純な絞り込みを適用しない。

***

##### `DD-TYP-DYN-008`: static値からdynamic値への導入

`確定`

static型`T`の値を`dynamic S`へ導入できる条件は次である。

```text
T <: S
```

概念的なCore項を次とする。

```text
to-dynamic<S>(value)
```

型付けの概念形：

```text
value : T
T <: S
────────────────────────────
to-dynamic<S>(value) : dynamic S
```

この変換は失敗しない。

元の値が型検査済みであり、`T <: S`が静的に証明されているため、runtime membership
checkは不要である。

次の変換は静的に拒否する。

```text
T </: S
```

特に、static値からdynamic値への導入を、値の型を狭めるchecked castとして使用しては
ならない。

```text
to-dynamic:
失敗しないdynamic境界の生成

checked cast:
失敗し得る型のnarrowing
```

この二つを区別する。

***

##### `DD-TYP-DYN-009`: dynamic上限のwidening

`確定`

```text
x : dynamic S
```

について、

```text
S <: U
```

なら、runtime checkなしで次へwidenできる。

```text
dynamic S
→ dynamic U
```

概念的なCore項：

```text
widen-dynamic<U>(x)
```

この変換は元のdynamic boundary identityおよびprovenanceを保持する。

これに対して、

```text
dynamic S
→ dynamic U
```

で`S <: U`が成立しない場合は、暗黙のwideningとして認めない。

上限を狭める必要がある場合は、型判定またはchecked castを使用する。

checked castに成功した値は、原則として狭い`dynamic U`ではなく、確認済みのstatic型
`intersect(S, U)`として扱う。

***

##### `DD-TYP-DYN-010`: foreign値のdynamic導入

`確定`

型保証のないforeign runtime valueを`dynamic S`へ導入する場合、境界で`S`へのruntime
validationを必須とする。

概念的な操作：

```text
import-dynamic<S>(foreign-value)
```

型：

```text
ForeignValue
→ Result<dynamic S, boundary-error>
```

型`S`がruntime検査可能であることを次の判断で表す。

```text
RuntimeCheckable(S)
```

型付けの概念形：

```text
foreign : ForeignValue
RuntimeCheckable(S)
────────────────────────────────────────
import-dynamic<S>(foreign)
: Result<dynamic S, boundary-error>
```

validationに成功した値だけを`dynamic S`として公開する。

validationに失敗した場合は、後段のdynamic cast failureではなく、値の導入境界における
`boundary-error`とする。

safe RPXでは、未検証のforeign valueを`dynamic S`と仮定する操作を公開しない。

```text
unsafe-assume-dynamic<S>
```

相当の機能が必要な場合は、trusted kernelまたは監査済みforeign adapterに限定する。

***

##### `DD-TYP-DYN-011`: decoderとgradual foreign boundaryの分離

`確定`

外部bytesや構造化データをRPX値へ変換するdecoderと、foreign runtime objectをdynamic値として
導入する境界を区別する。

###### Static decoder

```text
decode<S>(bytes)
: Result<S, decode-error>
```

decoderは外部表現を完全に検証し、通常のstatic RPX値`S`へ再構築する。

###### Gradual foreign boundary

```text
import-dynamic<S>(foreign-value)
: Result<dynamic S, boundary-error>
```

foreign objectのidentityまたは動的表現を維持したまま、上限`S`だけを保証する。

JSON、設定ファイル、document source等の通常の外部データについては、可能な限りdecoderを使用し、
安易に`dynamic any`へ導入しない。

***

##### `DD-TYP-DYN-012`: runtime-checkableな型

`確定`

RPX v1では、型`S`がruntimeで安全に検査可能である場合だけ、foreign valueから
`dynamic S`を生成できる。

直接検査可能な型の候補は次である。

* `bool`
* `unit`
* `int`
* `f64`
* `str`
* runtime tagを持つ通常のprimitive型
* runtime identityを持つvariant
* 検査可能なfieldからなるrecord
* 有限union
* 検査可能な型からなるintersection
* moduleが正規のruntime identityを提供するnominal型

wrapperによって検査可能な型の候補は次である。

* fixed-arity function
* 検査可能な要素型を持つ一部のcollection
* foreign adapterが正規のcontractを提供するcallable

一般のforeign境界から生成できない型は次である。

* affine resumption
* named/scoped effect evidence
* handler instance
* private resource capability
* raw runtime authority
* moduleの認証なしでは生成できないopaque abstract type
* runtime検査不能な任意のsemantic negation
* scope identityの保持が必要な型

```text
¬RuntimeCheckable(S)
```

の場合、

```text
import-dynamic<S>(foreign)
```

を静的に拒否する。

***

##### `DD-TYP-DYN-013`: implicit cast failure

`確定`

型検査器が挿入したimplicit castがruntimeで失敗した場合、次の構造化された言語上の
異常終端を生成する。

```text
dynamic-type-error {
  dynamic-bound,
  required-type,
  refined-target,
  actual-runtime-type,
  introduction-boundary,
  cast-site,
  module-path,
  cast-trace
}
```

`dynamic-type-error`は通常のalgebraic effectではない。

次の扱いを採用する。

* effect rowへ追加しない
* 通常のRPX handlerからcatchできない
* Rust panicにしない
* undefined behaviorにしない
* compiler/runtime defectとは分類しない
* gradual typingによって許容された明示的な異常終端とする

CLIでは、cleanup後に構造化診断を表示し、非zero statusで終了する。

GUIでは、失敗した評価runを停止し、次を行う。

* cleanupを実行する
* そのrunのsource transactionをrollbackする
* 途中のRenderIRを破棄する
* last-good-renderを保持する
* introduction boundaryとcast siteの両方を診断表示する

***

##### `DD-TYP-DYN-014`: 明示的safe cast

`確定`

cast失敗を正常な分岐として処理したい場合は、implicit castではなく明示的なsafe castを
使用する。

単純に成否だけが必要な場合：

```text
try-cast<T> :
dynamic S
→ Option<intersect(S, T)>
```

失敗理由が必要な場合：

```text
check-cast<T> :
dynamic S
→ Result<intersect(S, T), cast-mismatch>
```

`intersect(S, T) ≃ never`である場合、safe castも成功不能なので静的に拒否してよい。

implicit castとexplicit safe castを区別する。

```text
implicit cast failure:
dynamic-type-errorでrunを停止

explicit safe cast failure:
OptionまたはResultとして利用者コードが処理
```

認識可能な型判定とoccurrence typingで十分な場合は、safe castより型判定を優先できる。

***

##### `DD-TYP-DYN-015`: cast evidence

`確定`

すべてのimplicit castを、typed Coreの明示的な`Cast`項へelaborateする。

```text
Cast(expression, evidence, cast-id)
```

cast evidenceは、runtime検査および成功後の型保証を表す内部証拠である。

conceptual evidence algebraを次のように定める。

```text
Evidence ::=
    Identity
  | Widen
  | TagCheck
  | UnionCheck
  | IntersectionCheck
  | RecordCheck
  | VariantCheck
  | FunctionGuard
  | NominalCheck
  | Compose
```

それぞれの意味は次のとおりである。

###### `Identity`

runtime検査が不要なcast。

###### `Widen`

dynamic上限の安全な拡張。

###### `TagCheck`

primitiveまたはruntime tagによる直接検査。

###### `UnionCheck`

複数候補のいずれかに属することの純粋な検査。

###### `IntersectionCheck`

すべての構成型の条件を満たすことの検査。

###### `RecordCheck`

必要field、field型およびrow条件の検査。

###### `VariantCheck`

constructor identityおよびpayload型の検査。

###### `FunctionGuard`

関数呼出時に引数および結果を検査するwrapper。

###### `NominalCheck`

moduleまたは型所有者が発行したruntime type identityによる検査。

###### `Compose`

二つのevidenceを順番に適用する意味的合成。

evidenceは通常のRPX値として公開しない。

次を提供しない。

* evidenceの取得
* evidenceの等値比較
* evidenceのhash
* evidenceのserialization
* private nominal identityの観測
* evidenceを利用したmodule sealingの回避

***

##### `DD-TYP-DYN-016`: cast evidenceの純粋性

`確定`

implicit cast evidenceの実行は純粋でなければならない。

implicit castは次を発生させてはならない。

* I/O
* Resource operation
* FileRead／FileWrite
* Network
* Random
* Clock
* SourceEdit
* 利用者定義の非制限effect
* handler stackの観測可能な変更

castは、次のいずれかだけを行う。

1. 値をそのまま返す。
2. runtime型または構造を検査する。
3. guarded wrapperを生成する。
4. `dynamic-type-error`で終了する。

型注釈または型精度の変更だけで、成功実行のeffect traceを変えてはならない。

***

##### `DD-TYP-DYN-017`: evidence compositionと最適化

`確定`

連続するcastの規範意味論は、まず`Compose`による逐次適用として定義する。

```text
Compose(e1, e2)
```

は、`e1`を適用し、成功した場合に`e2`を適用する。

implementationは、次をすべて保存できる場合だけevidenceを簡約してよい。

* 成功する値の集合
* 失敗する値の集合
* 成功後の型
* 値identityに関する仕様
* effect trace
* module abstraction
* dynamic導入境界
* cast使用箇所
* 利用者向け診断

許可される代表的な簡約は次である。

```text
Compose(Identity, e)
→ e
```

```text
Compose(e, Identity)
→ e
```

```text
Widen(S, T)
;
Widen(T, U)
→
Widen(S, U)
```

ただし、中間境界のprovenanceを失ってはならない。

一次値について、最終検査と診断履歴を保存できる場合は、連続するtag checkを最終的な
より狭い検査へ縮約してよい。

function evidenceは、RPX v1の規範意味論では順次wrapperとして維持する。

実装がfunction wrapperを融合する場合は、引数検査、結果検査、失敗境界および
呼出回数を保存しなければならない。

***

##### `DD-TYP-DYN-018`: cast provenance

`確定`

runtime検査用evidenceと、利用者診断用のcast provenanceを分離する。

```text
ExecutableEvidence {
  runtime検査に必要な最小情報
}
```

```text
CastProvenance {
  dynamic導入境界,
  中間widening,
  narrowing箇所,
  使用箇所,
  source type,
  target type,
  module path,
  source span
}
```

evidenceを最適化しても、cast provenanceを失ってはならない。

dynamic値の導入境界および各narrowing使用箇所には、一意なboundary IDまたはcast IDを
付与する。

private module内の型identityおよびrepresentationは、診断表示時にauthorityに従って
隠蔽する。

***

##### `DD-TYP-DYN-019`: recordおよびvariant cast

`確定`

不変recordについては、runtimeで次を検査できる。

* 値がrecordであること
* 必須fieldの存在
* 各field値の型
* open／closed row条件
* nominalまたはstructural identityの必要条件

open record型では、指定されていない追加fieldを許す。

closed record型に追加fieldを許すかどうかは`ROW-001`のclosed record意味論に従う。

recordが通常の不変RPX値である場合、一度検査したfield evidenceを再利用してよい。

foreign mutable objectをrecordとして公開する場合は、導入時の一度の検査だけで将来の
field型を保証してはならない。必要ならproxyまたはfield access guardを使用する。

variant castは、constructorの文字列名ではなく、一意なconstructor identityで検査する。

payloadを持つconstructorでは、payloadについて対応するevidenceを適用する。

***

##### `DD-TYP-DYN-020`: opaque abstract typeのcast

`確定`

opaque abstract typeは構造によってcastしてはならない。

異なるsealed moduleが所有するabstract typeは、内部表現が同じであっても別型である。

```text
module-a.t
≠
module-b.t
```

dynamic castでは、型所有者が発行した非公開のnominal runtime identityを使用する。

```text
NominalCheck(abstract-type-id)
```

一般のforeign boundaryまたはstructural validatorは、opaque abstract typeの値を
新しく生成できない。

abstract typeの正規の値を生成できるのは、少なくとも次に限る。

* 型所有moduleのconstructor
* 型所有moduleの認証済みdecoder
* 型所有moduleが発行したruntime evidence
* 許可されたmodule-private foreign adapter

dynamic cast、provenanceおよび診断は、abstract typeのprivate representationを
外部へ漏らしてはならない。

***

##### `DD-TYP-DYN-021`: fixed-arity function cast

`確定`

RPX v1は、具体的な単相fixed-arity function型へのdynamic castを許す。

source function型を次とする。

```text
(A1, ..., An) ->{Es} R
```

target function型を次とする。

```text
(B1, ..., Bn) ->{Et} U
```

function castを許可する基本条件は次である。

1. sourceとtargetがfixed-arity functionである。
2. arityが等しい。
3. 各`Bi`から`Ai`へのcastが定義可能である。
4. `R`から`U`へのcastが定義可能である。
5. latent effect rowが静的に判明している。
6. `Es`のeffect要求を`Et`が満たす。
7. scoped、affineまたはprivate capabilityをdynamic境界から漏らさない。

関数値をcastした時点では、次を確認する。

* 値がcallableである
* arityが一致する
* 必要なfunction metadataまたはadapterが存在する
* effect contractが静的に適合する

引数および結果の型はfunction call時にguardする。

***

##### `DD-TYP-DYN-022`: function引数の反変cast

`確定`

function guardでは、target側から受け取った各引数をsource functionの引数型へcastする。

```text
target argument:
Bi

source function requires:
Ai

cast direction:
Bi → Ai
```

これは関数引数の反変性に対応する。

概念的なwrapper：

```lisp
(fn (b1 ... bn)
  (let ((a1 (cast b1 a1-type))
        ...
        (an (cast bn an-type)))
    (source-function a1 ... an)))
```

`Bi <: Ai`ならruntime checkは不要である。

```text
intersect(Bi, Ai) ≃ never
```

なら、その関数castは成功不能なので静的に拒否する。

それ以外の場合は、実際の関数呼出時にruntime checkを行う。

***

##### `DD-TYP-DYN-023`: function結果の共変cast

`確定`

source functionの戻り値は、source result型`R`からtarget result型`U`へcastする。

```text
cast direction:
R → U
```

これは関数結果の共変性に対応する。

概念的なwrapper：

```lisp
(fn (arguments...)
  (let ((source-result
          (source-function converted-arguments...)))
    (cast source-result target-result-type)))
```

`R <: U`ならruntime checkは不要である。

```text
intersect(R, U) ≃ never
```

なら、関数castを静的に拒否する。

それ以外の場合は、source functionが戻った時点でruntime checkを行う。

引数または結果のguardが失敗した場合は`dynamic-type-error`とする。

***

##### `DD-TYP-DYN-024`: function arityの制限

`確定`

RPX v1のdynamic function castは、固定個数の位置引数を持つ関数だけを対象とする。

```text
() -> T
(A) -> T
(A, B) -> T
...
```

dynamic function castではarityの一致を必須とする。

次はRPX v1のdynamic function cast対象外とする。

* 可変長引数
* optional引数
* keyword引数
* overload
* 自動currying変換
* 自動partial application
* pattern parameter
* 動的に変化するforeign calling convention

これらを追加する場合は、call signatureとwrapper semanticsを個別に規定する。

***

##### `DD-TYP-DYN-025`: effectful function cast

`確定`

functionの値型引数および結果型にはdynamic castを許すが、latent effect rowはruntime cast
しない。

source functionのeffect rowを`Es`、target functionが許容するeffect rowを`Et`とする。

function castを許可するには、両方のrowが静的に判明し、次が成立しなければならない。

```text
Es ⊑ Et
```

ここで`⊑`は、source functionのeffect要求をtarget側のambient effect環境が満たせることを
表す。

例：

```text
source:
number ->{resource} str

target:
number ->{resource, log} str
```

は許可できる。

一方、次は拒否する。

```text
source:
number ->{resource, file-write} str

target:
number ->{resource} str
```

`file-write`がtarget側の許容範囲に含まれないためである。

effect rowが不明な関数を、具体的なeffectful function型へcastしてはならない。

```text
dynamic effect row
→ static effect row
```

のruntime監視およびgradual effect castは、RPX v1には導入しない。

参照実装は段階的に実装してよい。

```text
段階1:
pure fixed-arity function

段階2:
静的に既知のeffect rowを持つfixed-arity function

将来:
gradual effect typing
```

***

##### `DD-TYP-DYN-026`: dynamic境界を通れない制御値

`確定`

次の値は、通常のfunctionまたは`dynamic any`として扱ってはならない。

* handler value
* handler instance
* affine resumption
* named/scoped effect evidence
* private resource capability
* source transaction authority
* module-private runtime authority

特に、次を禁止する。

```text
Resume<R, B, E>
→ dynamic any
```

```text
dynamic function
→ Resume<R, B, E>
```

```text
Evidence<L, s>
→ dynamic any
→ scope外へescape
```

handler valueはfirst-classだが、通常のdynamic function castの対象ではない。

handler専用dynamic castはRPX v1には導入しない。

***

##### `DD-TYP-DYN-027`: polymorphismとdynamic境界

`確定`

RPX v1では、多相値を多相性を維持したままdynamic境界へ通さない。

rank-1 polymorphic valueをdynamic境界へ渡す場合、境界が要求する具体的な単相型へ
instantiateする。

例：

```text
identity :
forall a. a -> a
```

を次の境界へ渡す場合、

```text
dynamic(int -> int)
```

まず次へinstantiateする。

```text
int -> int
```

その後で通常のstatic injectionまたはfunction castを適用する。

境界型だけからinstantiationを決定できない場合、明示的な型注釈を要求する。

次を禁止する。

```text
dynamic function
→ forall a. a -> a
```

有限のruntime検査では、すべての型についての普遍量化を保証できないためである。

dynamic値から取り出せるfunction型は、具体的な単相fixed-arity function型に限定する。

effect-row polymorphicな値は、具体的なstatic EffectRowへinstantiateしてからdynamic境界へ
渡す。

record-row polymorphicな値は、具体的なRecordRowへinstantiateしてからdynamic境界へ渡す。

多相handler valueは一般のdynamic境界へ通さない。

typed Coreへ到達する前にdynamic境界を通る多相値を単相化し、runtime cast calculusは
原則として単相型だけを扱う。

***

##### `DD-TYP-DYN-028`: gradual guarantee

`確定`（設計目標、未証明）

RPXは、bounded dynamicを含む型precision関係を定義する。

precision関係はsemantic subtypingとは別の判断とする。

概念的に次の記号を用いる。

```text
G1 ⊑p G2
```

これは、`G2`が`G1`以上に精密であることを表す。

RPXはstatic gradual guaranteeおよびdynamic gradual guaranteeを設計目標とする。

###### Static gradual guarantee

型情報を不精密にしたことだけを理由として、以前型検査可能だったプログラムを
不必要に拒否しないことを目標とする。

ただし、次の制約は型precisionの低下によって消去できない。

* affine resumption
* scoped evidence
* module-private nominal identity
* resource capability
* handler authority
* effect-row安全境界

###### Dynamic gradual guarantee

精密な版と不精密な版が共に正常終了する場合、観測可能な結果が等しいことを目標とする。

最低限の観測対象は次である。

* return value
* handled effect trace
* generated artifact
* committed source transaction
* external output
* resource operationの意味的順序

型精度を上げた結果として、次は許容する。

* runtime castの削除
* `dynamic-type-error`の静的検出
* より早い型エラー
* より具体的なdynamic診断

型精度を変更しただけで、両方が正常終了する実行について次を変更してはならない。

* return value
* effectの順序または回数
* handler selection
* generated artifactの意味
* committed `source-edit`
* external output
* resource操作の意味的順序

次はRPX v1のgradual guarantee対象外とする。

* polymorphic dynamic
* gradual effect-row cast
* dynamic handler cast
* dynamic resumption cast
* scoped／affine capabilityのdynamic化
* 実行時間の一致
* allocation回数の一致
* 最大memory使用量の一致
* resource exhaustionの完全な一致

gradual guarantee、blame safetyおよび通常の型安全性は、別々の性質として扱う。

これらは現時点では未証明である。

***

##### `DD-TYP-NUM-001`: RPX v1の基本数値型

`確定`

RPX v1は、次の二つを別のprimitive型として持つ。

```text
int
f64
```

###### `int`

```text
int
= 任意精度の正確な符号付き整数
```

整数overflowを通常の`int`へ設けない。値の大きさは利用可能memoryによってのみ制限される。

###### `f64`

```text
f64
= IEEE 754 binary64 floating-point
```

`f64`は数学的な実数全体ではなく、有限精度の近似数値型である。

###### `number`

```text
number
≃
union(int, f64)
```

次が成り立つ。

```text
int <: number
f64 <: number
```

次は成り立たない。

```text
int <: f64
f64 <: int
```

`int`と`f64`は別のruntime値領域とする。

```text
int-value(1)
≠
f64-value(1.0)
```

数値比較によって等しい場合があっても、runtime表現型は異なる。

***

##### `DD-TYP-NUM-002`: 数値promotion

`確定`

`int`から`f64`への変換はsemantic subtypingではなくnumeric promotionとする。

混合算術では、原則として`int`を`f64`へpromotionし、結果を`f64`とする。

```text
int + int
→ int

f64 + f64
→ f64

int + f64
→ f64

f64 + int
→ f64
```

同様の規則を減算、乗算、比較等へ適用できる。

任意精度`int`から`f64`へのpromotionでは、値によって精度が失われる可能性がある。
これは数値仕様および診断で明示する。

通常の除算`/`は、RPX v1では次の型を基本とする。

```text
int / int
→ f64
```

整数除算および剰余は別の`int`専用operationとして提供する。

表面名は別途決定するが、概念的には次である。

```text
int-div : int × int -> int
mod     : int × int -> int
```

`real`という型名はRPX v1では導入せず、将来のnumeric tower拡張用に予約する。

`rational`、`decimal`、`f32`、固定幅整数等は将来拡張またはlibrary型として扱う。

***

##### `DD-TYP-NUM-003`: dynamic castとnumeric promotionの順序

`確定`

dynamic narrowingとnumeric promotionを一つの特殊castとして混在させない。

処理順序を次のように定める。

```text
1. dynamic値を、必要なstatic数値型へruntime narrowingする。
2. narrowingに成功した値へ通常のnumeric promotionを適用する。
```

例：

```text
x : dynamic int
expected:
f64
```

処理：

```text
1. xがint上限を満たすことを利用する。
2. int値をf64へnumeric promotionする。
```

例：

```text
x : dynamic number
expected:
f64
```

実行時値が`f64`の場合：

```text
1. f64としてnarrowing成功
2. promotion不要
```

実行時値が`int`の場合：

```text
1. number内のintとしてnarrowing
2. intからf64へnumeric promotion
```

実行時値が`number`の上限外である場合は、dynamic値の不変条件違反であり、
通常のcast failureではなく境界またはruntime defectとして扱う。

数値promotionの失敗、精度損失警告および有限性要件の詳細は、
`OPEN-SYN-002`および`LIT-001`へ移管する。

***

##### `DD-NAME-001`: 組込み型名と識別子の小文字規約

`確定`

RPXの通常識別子は、namespaceにかかわらず小文字で開始する。

この規則を次へ適用する。

* value
* function
* type
* type parameter
* record row
* effect row
* module
* signature
* effect
* operation
* constructor
* syntax／macro
* package内member

組込み型の主要な表記を次とする。

```text
any
never
bool
unit
str
int
f64
number
```

複数語の識別子には`kebab-case`を推奨する。

```text
source-edit
render-ir
last-good-render
resource-id
dynamic-type-error
```

略語も小文字で表記する。

```text
svg
pdf
pptx
ir
id
uri
```

大文字開始の通常識別子は拒否する。

```lisp
(val Page ...)
```

診断例：

```text
識別子は小文字で開始する必要があります。

found:
  Page

suggestion:
  page
```

型、値、module、effect、syntax等の区別は、大文字小文字ではなくnamespaceおよび
名前解決後の`BindingId`で行う。

constructorも小文字で表記する。

```text
some
none
ok
err
true
false
```

***

##### `DD-NAME-002`: namespace間の同綴り衝突

`確定`

異なるnamespaceに属する識別子は、型システム上は同じ綴りを持つことができる。

概念例：

```text
type namespace:
picture

value namespace:
picture
```

ただし、同一moduleまたは同一public APIで、複数namespaceに同じ綴りのpublic memberを
公開する場合は衝突警告を出す。

```text
warning:
public name `picture` is exported in multiple namespaces

namespaces:
  type
  value
```

この警告は、名前解決上の曖昧性がない場合でも、API検索、診断、documentationおよび
利用者の読みやすさを改善するために出す。

内部memberまたは意図的なfactory／constructor patternについて、warningを局所的に
抑制できる仕組みは将来の診断設定で定める。

namespace間同綴りを言語上の全面的なエラーにはしない。

***

##### dynamic typingのCore構文

RPX v1のtyped Coreへ、概念的に次を追加する。

```text
e ::= ...
    | ToDynamic(static-bound, expression, boundary-id)
    | WidenDynamic(target-bound, expression, boundary-id)
    | Cast(expression, evidence, cast-id)
    | TryCast(expression, evidence)
    | ImportDynamic(foreign-expression, validator, boundary-id)
    | FunctionGuard(function, signature-evidence, cast-id)
```

実際のCore表現では、`FunctionGuard`を`Cast` evidenceの一種として統合してもよい。

ただし、規範的意味は次を区別しなければならない。

```text
ToDynamic:
失敗しないstatic injection

WidenDynamic:
失敗しないdynamic upper-bound widening

Cast:
失敗時にdynamic-type-errorとなるimplicit narrowing

TryCast:
失敗をOption／Resultとして返す明示的safe cast

ImportDynamic:
foreign境界で上限を検証し、失敗をboundary-errorとして返す

FunctionGuard:
call時に引数および結果を検査する高階境界
```

***

##### dynamic typingの終端状態

RPXの評価結果へ、少なくとも次を追加する。

```text
EvaluationOutcome<A> =
    completed(A)
  | dynamic-type-error(DynamicTypeErrorInfo)
  | runtime-fault(RuntimeFaultInfo)
  | divergence
```

`dynamic-type-error`と`runtime-fault`を区別する。

```text
dynamic-type-error:
許可されたgradual castがruntime値に対して失敗した、
言語仕様上の明示的な異常終端

runtime-fault:
well-typed／well-linkedな実行では到達不能であるべき、
runtime、foreign boundaryまたは実装整合性の欠陥
```

`boundary-error`および明示的safe castの失敗は通常のRPX値として扱う。

***

##### 適合試験

###### static injection

```text
42 : int
```

を次へ導入する。

```text
dynamic number
```

期待：

```text
成功
runtime checkなし
```

###### invalid static injection

```text
picture
→ dynamic number
```

期待：

```text
静的エラー
```

###### dynamic widening

```text
dynamic int
→ dynamic number
```

期待：

```text
成功
runtime checkなし
元のboundary provenanceを保持
```

###### safe static use

```text
x : dynamic int
```

を`number`が必要な位置で使用する。

期待：

```text
int <: number
runtime checkなし
```

###### partial overlap

```text
x : dynamic(union(number, str))
```

を`number`が必要な位置で使用する。

期待：

```text
runtime Castを挿入
```

実体が`int`または`f64`：

```text
成功
```

実体が`str`：

```text
dynamic-type-error
```

###### disjoint use

```text
x : dynamic(union(number, str))
```

を`picture`が必要な位置で使用する。

期待：

```text
静的エラー
```

###### cast precision

```text
x : dynamic(union(int, str))
```

を`number`として検査する。

期待される成功後の型：

```text
int
```

###### `any`と`dynamic any`

```text
x : any
```

```lisp
(+ x 1)
```

期待：

```text
静的エラー
```

```text
y : dynamic any
```

```lisp
(+ y 1)
```

期待：

```text
runtime number checkを挿入
```

###### foreign ingress

```text
foreign value
→ import-dynamic<number>
```

実体が`int`：

```text
ok(dynamic number)
```

実体が`picture`：

```text
err(boundary-error)
```

###### implicit cast failure

```text
x : dynamic(union(number, str))
actual x = "hello"
```

```lisp
(+ x 1)
```

期待：

```text
dynamic-type-error
```

通常のeffect handlerによるcatch：

```text
不可
```

###### explicit safe cast

```text
try-cast<number>(x)
```

実体が`str`：

```text
none
```

評価run自体は継続する。

###### fixed-arity function cast

```text
source:
int ->{} str

target:
number ->{} str
```

期待：

```text
FunctionGuardを生成
number引数をcall時にintへ検査
```

`f64`を渡した場合：

```text
dynamic-type-error
```

###### function result cast

```text
source:
unit ->{} dynamic(union(number, str))

target:
unit ->{} number
```

期待：

```text
source結果をnumberとして検査
```

###### effect-compatible function cast

```text
source:
number ->{resource} str

target:
number ->{resource, log} str
```

期待：

```text
受理
```

###### effect-incompatible function cast

```text
source:
number ->{resource, file-write} str

target:
number ->{resource} str
```

期待：

```text
静的エラー
```

###### polymorphic value boundary

```text
identity:
forall a. a -> a
```

境界：

```text
dynamic(int -> int)
```

期待：

```text
a := intとしてinstantiate
単相化後にdynamic境界へ導入
```

境界：

```text
dynamic function
```

期待：

```text
具体的なinstantiationを決定できないため静的エラー
```

###### dynamicからforall

```text
dynamic function
→ forall a. a -> a
```

期待：

```text
静的エラー
```

###### numeric promotion

```text
x : dynamic number
actual x = int-value(2)
expected = f64
```

期待：

```text
number内のintとして確認
intからf64へpromotion
result = 2.0 : f64
```

###### namespace collision warning

同一moduleが次をpublic exportする。

```text
type namespace:
picture

value namespace:
picture
```

期待：

```text
名前解決は成功
public namespace collision warningを出す
```

***

##### 未解決事項の移管

`OPEN-TYP-001`を閉じるため、次を別項目へ移管する。

###### `OPEN-TYP-002`

* static semantic subtypingの実装可能な決定手続き
* emptiness判定の正確な範囲
* cast挿入が依存するalgorithmic approximation
* type／row／effect constraint solverの完全性
* checker limitationによる保守的拒否
* principal solutionの有無

###### `OPEN-SYN-002`／`LIT-001`

* `int`および`f64`の完全な字句文法
* 数値suffix
* hexadecimal／binary literal
* underscore separator
* NaN／Infinityの字面
* signed zero
* 浮動小数点の丸め
* `f64` equalityおよびhash
* intからf64への精度損失診断
* zero division
* unit literalのtokenization
* `i64`等の固定幅数値型

###### `OPEN-ERR-001`

* `dynamic-type-error`と他の異常終端の統合分類
* GUI／CLI診断の最終形式
* cleanup中にdynamic failureが発生した場合
* 複数failureの合成
* exception effectとの関係

###### `OPEN-MOD-001`

* abstract typeのruntime nominal identity
* separate compilationをまたぐidentity安定性
* plugin境界でのabstract type evidence
* signatureへruntime-checkabilityを記述する方法

###### `OPEN-KER-001`

* ForeignValue ABI
* runtime validator ABI
* trusted foreign adapter
* unsafe dynamic assumptionの権限
* foreign function effect契約の監査

###### 将来拡張

* polymorphic dynamic
* gradual effect typing
* handler cast
* resumption cast
* runtime effect monitor
* formal blame calculus
* blame theorem
* gradual guaranteeの形式証明
* space-efficient coercion calculus

***

##### 解決後の基本原則

```text
static top type:
any

static bottom type:
never

basic numeric types:
int
f64

numeric union:
number = int | f64

gradual type:
dynamic S
```

```text
dynamic SをTとして使用:

S <: T
  → checkなし

intersect(S, T) ≃ never
  → 静的エラー

それ以外
  → runtime Cast
```

```text
static Tからdynamic S:

T <: S
  → 安全な導入

それ以外
  → 静的エラー
```

```text
foreign valueからdynamic S:

RuntimeCheckable(S)
  → runtime validation

validation失敗
  → boundary-error
```

```text
implicit cast失敗:
dynamic-type-error

explicit safe cast失敗:
OptionまたはResult
```

```text
関数cast:

引数:
target → source

結果:
source → target

effect row:
runtime castせず、静的包含を要求
```

以上により、RPX v1のbounded dynamic、cast insertion、runtime failure、
function guard、polymorphism境界および基本的なgradual guaranteeの範囲が特定されたため、
`OPEN-TYP-001`を解決済みとする。

#### 13.6.2 `TYP-ALG-001` Algorithmic型検査、semantic subtypingの判定範囲、型推論

> 統合元: `OPEN-TYP-002`。同項目は解決済み。

`解決済み`

本決定は、RPX v1の宣言的型システムに対して、型検査器がどの範囲を自動的かつ完全に判定し、どの範囲で型注釈または保守的拒否を必要とするかを固定する。

本決定の対象は次のとおりである。

- 宣言的型関係とalgorithmic型判定の分離
- `proved`、`disproved`、`unknown`の三値判定
- 型エラー、注釈要求、checker limitationの区別
- checker soundness、completeness、terminationの目標範囲
- principal typeの保証範囲
- semantic subtypingの完全判定fragment
- singleton型
- fixed-arity functionのsubtyping
- function intersectionのcoherence
- effect environmentとrequired effects
- RecordRowの完全判定範囲
- recursive data typeの判定範囲
- bidirectional type checking
- 明示的`forall`とkind
- 型機能のA／B／C／D分類
- 共通constraint worklist
- 決定的なsolver resource budget

次はRPX v1のalgorithmic型検査には含めず、別項目または将来拡張へ移管する。

- algorithmic completenessをRPX全体へ保証すること
- RPX全体のprincipal type
- higher-rank polymorphism
- 多相再帰
- relaxed value restriction
- 一般のSurface `mu`型
- non-regular recursive type
- function型のSurface negation
- gradual effect typing
- dynamic handler cast
- dynamic resumption cast
- 暗黙のruntime overload dispatch
- 一般的なmulti-method
- unrestrictedなRecordRow／EffectRow tallying
- checkerのsoundness、completeness、terminationの形式証明

---

##### `DD-TYP-ALG-001`: 宣言的型関係とalgorithmic判定の分離

`確定`

RPXでは、宣言的なsemantic subtypingと、実装される型検査器のalgorithmic判定を区別する。

宣言的なstatic型の部分型関係は、型の意味的な値集合の包含として定義する。

```text
s <: t
iff
[[s]] ⊆ [[t]]
````

型等価は相互部分型として定義する。

```text
s ≃ t
iff
s <: t
and
t <: s
```

部分型関係は、型の差分の空性へ帰着できる。

```text
s <: t
iff
intersect(s, not(t)) ≃ never
```

これは宣言的な意味であり、型検査器がすべての型について完全に判定できることを意味しない。

algorithmic判定は、宣言的関係を近似・実装する独立した判断とする。

```text
decide-subtype(s, t)
decide-equivalent(s, t)
decide-empty(t)
```

***

##### `DD-TYP-ALG-002`: algorithmic判定の三値結果

`確定`

algorithmicな部分型、型等価、空性および関連する型判断は、内部的に次の三種類の結果を返す。

```text
proved
disproved
unknown
```

###### `proved`

要求された型関係が宣言的意味論のもとで成立することを、checkerが証明したことを表す。

```text
decide-subtype(int, number)
→ proved
```

###### `disproved`

要求された型関係が成立しないことを、checkerが証明したことを表す。

```text
decide-subtype(str, number)
→ disproved
```

可能な場合は、反例となる型または値のwitnessを診断用に生成する。

###### `unknown`

型関係の意味は定義されているが、現在のchecker、設定およびresource budgetでは証明も反証もできなかったことを表す。

`unknown`を受理として扱ってはならない。

利用者プログラムの処理は次のとおりである。

```text
proved
→ 受理可能

disproved
→ type-errorとして拒否

unknown
→ checker-limitationとして拒否
```

最終的なcompile結果は`accepted`または`rejected`の二値である。

三値結果は、型検査器内部および診断分類のために使用する。

***

##### `DD-TYP-ALG-003`: 診断分類

`確定`

次を異なる診断として扱う。

```text
type-error
annotation-required
checker-limitation
checker-resource-limit
unsupported-language-feature
```

###### `type-error`

要求された型関係が成立しないことをcheckerが反証した場合に使用する。

```text
type-error

actual:
  str

expected:
  number

counterexample:
  "text"
```

###### `annotation-required`

式から型を一意に合成するための情報が不足しているが、期待型が与えられれば既知のalgorithmで検査できる場合に使用する。

```text
annotation-required

この式の型を決定するには型注釈が必要です。

binding:
  values

example:
  (type values
    (vector picture))
```

###### `checker-limitation`

型情報は与えられているが、現在のcheckerでは型関係を証明も反証もできない場合に使用する。

```text
checker-limitation

この型関係を現在のcheckerでは決定できません。

source:
  ...

target:
  ...

unresolved constraints:
  ...
```

###### `checker-resource-limit`

決定的なsolver budgetを超過した場合に使用する。

理論上のalgorithmic limitationと、実装設定によるresource limitを区別する。

###### `unsupported-language-feature`

RPX v1で意味またはalgorithmを提供しない機能を使用した場合に使用する。

```text
unsupported-language-feature:
  higher-rank polymorphism
```

***

##### `DD-TYP-ALG-004`: soundness、completeness、terminationの優先順位

`確定`

RPX v1のalgorithmic type checkerは、次の順に優先する。

1. algorithmic soundness
2. termination
3. 診断の有用性
4. 通常コードに対する推論能力
5. algorithmic completeness
6. principal type

目標とするsoundnessは次である。

```text
checkerがプログラムを受理する
→
宣言的型システムでも型付け可能
```

RPX v1の型システム全体について、次は保証しない。

```text
宣言的型システムで型付け可能
→
checkerが必ず受理する
```

checkerが証明できない型関係を、推測または任意選択によって受理してはならない。

対象fragmentとresource budgetの範囲で、checkerが有限stepで終了することを設計目標とする。

これらの性質は現時点では未証明である。

***

##### `DD-TYP-ALG-005`: principal type

`確定`

RPX v1は、言語全体についてprincipal typeを保証しない。

次の限定fragmentでは、principalまたは十分に一般的で安定した型を推論することを目標とする。

* rank-1／prenex polymorphism
* 構文的value restriction
* 単相再帰
* 通常のfixed-arity function
* checkerが完全に扱えるRecordRow
* checkerが完全に扱えるEffectRow
* 高度なnegationを含まない型
* coherentなfunction intersection
* 一意なconstraint solutionを持つ式

複数の非比較な型解が存在し、期待型によって一意化できない場合は、任意の一解を黙って選ばない。

```text
複数の非比較な解
→ annotation-required
```

独立した`(type ...)`宣言による期待型を要求する。

***

##### `DD-TYP-ALG-006`: 決定的なsolver budget

`確定`

型検査resource limitは、wall-clock timeoutではなく決定的なsolver step budgetを基本とする。

budget対象には少なくとも次を含められる。

* kind constraintの処理
* metavariable unification
* 型正規化step
* subtype obligation
* emptiness branch
* recursive type pairの展開
* RecordRow constraint
* EffectRow constraint
* tallying候補
* overload coherence check
* cast evidence生成
* scope／capability constraint

同じcompiler version、source、dependency signature、checker設定およびbudgetについて、同じ判定結果を得ることを目標とする。

budget超過は型不一致ではない。

```text
checker-resource-limit
```

として報告する。

***

## Effectに関するalgorithmic用語

##### `DD-TYP-EFF-001`: effect関連用語

`確定`

RPXでは、effectに関する次の概念を区別する。

```text
effect-signature-environment
effect-environment
expression-effects
function-required-effects
handled-effects
residual-effects
effect-row
```

###### `effect-signature-environment`

effect identityおよびoperation identityから、operationの引数型、結果型、effect identity等を取得する静的環境である。

概念記号として`Ξ`を使用できる。

###### `effect-environment`

現在の式を実行するときに、その文脈で利用可能または許容されるeffect要求を表す型検査上の環境である。

effect environment自体は、EffectRowによって表現される。

```text
effect environment:
<resource, log, file-write>
```

###### `expression-effects`

式を現在評価することで、外側へ要求される未処理effectである。

```lisp
(load-image resource-id)
```

のexpression effectsは、概念的に次である。

```text
<resource>
```

###### `function-required-effects`

関数値を呼び出したときに、外側へ要求されるeffectである。

```text
project ->{resource, log} picture
```

における`resource`と`log`である。

文献上latent effectと呼ばれる概念に相当するが、RPXの規範用語では`function-required-effects`を使用する。

###### `handled-effects`

現在のhandlerまたはrunnerが処理するeffectである。

###### `residual-effects`

handlerまたはrunner適用後も外側へ残るeffectである。

###### `effect-row`

上記のeffect要求を表現する型レベルのrow構造である。

effect environmentとeffect rowは同一概念ではない。

```text
effect-row:
型レベルの構造

effect-environment:
型検査文脈における役割
```

***

##### `DD-TYP-EFF-002`: 最小required effects

`確定`

式および関数には、外側から実際に必要となる未処理effectだけを記録する。

周囲のeffect environment全体を、式または関数のrequired effectsへコピーしてはならない。

```text
effect environment:
<resource, log, file-write>

expression required effects:
<log>
```

の場合、式の推論結果は`<log>`のままとする。

その式が現在のeffect environmentで実行可能である条件は次である。

```text
<log> ⊑ <resource, log, file-write>
```

通常のfirst-order functionには、原則として最小のclosed residual rowを推論する。

関係のないEffectRow変数を自動的に追加してはならない。

```text
str ->{log} picture
```

を、理由なく次へ一般化してはならない。

```text
forall e.
  str ->{log | e} picture
```

***

##### `DD-TYP-EFF-003`: 関数値とrequired effects

`確定`

関数値の生成、保存および受渡しだけでは、その関数のrequired effectsを現在のexpression effectsへ追加しない。

```lisp
(fn (id)
  (load-image id))
```

について：

```text
fn式のexpression effects:
<>

生成された関数のrequired effects:
<resource>
```

関数required effectsは、関数が実際に適用された場所でexpression effectsへ加わる。

***

##### `DD-TYP-EFF-004`: effect-row polymorphism

`確定`

高階関数は、引数関数のrequired effectsをEffectRow変数で抽象化できる。

概念型：

```text
forall a b e.
  ((a ->{e} b), vector<a>)
  ->{e}
  vector<b>
```

callbackのrequired effectsを、そのまま呼出側へ伝える。

高階関数自身がeffectを要求する場合は、row extensionとして追加する。

```text
forall a b e.
  ((a ->{e} b), a)
  ->{log | e}
  b
```

EffectRow変数へ、現在のeffect environment全体を不用意に代入してはならない。

***

##### `DD-TYP-EFF-005`: handlerとrunnerによるeffect縮小

`確定`

関数内部でhandlerまたはrunnerが完全に処理したeffectは、その関数のresidual effectsから除去する。

```text
body required effects:
<resource, cache, log>

handled effects:
<cache, log>

residual effects:
<resource>
```

公開関数型へ残すのは`<resource>`だけである。

packageはrunner関数を提供し、内部実装effectを処理して公開required effectsを縮小できる。

module signatureは、内部実装effectを公開せず、利用者が提供する必要のあるcapabilityだけを公開してよい。

***

##### `DD-TYP-EFF-006`: EffectRow alias

`確定`

複数のeffectをまとめた公開API用のaliasを提供できる。

構文名は別途確定するが、概念的には次のような型付きaliasである。

```lisp
; [Surface候補]
(effect-alias preview-host
  (effects
    resource
    font-service
    image-service
    log))
```

EffectRow aliasは単なる文字列集合ではない。

内部では解決済みのEffectIdおよび必要な型引数を持つ、正規のEffectRowとして扱う。

型検査時には展開できるが、IDEおよび診断ではalias名を通常表示として使用できる。

すべてのeffectを万能な`io`へまとめることは推奨しない。

```text
preview-host
export-host
editor-host
test-host
```

等、意味の異なるcapability境界を区別する。

***

##### `DD-TYP-EFF-007`: 注釈されたrequired effects

`確定`

関数型注釈にrequired effectsが記述されている場合、initializerから推論されたrequired effectsが注釈rowへ含まれることを要求する。

```text
inferred-required-effects
⊑
declared-required-effects
```

推論rowが注釈rowより大きい場合は型エラーとする。

注釈rowが推論rowより不必要に広い場合は受理できるが、過剰なcapability要求として警告してよい。

```text
warning:
declared required effects are broader than inferred

unused capability:
  log
```

***

## Static型のBoolean代数

##### `DD-TYP-BOOL-001`: 型のBoolean演算

`確定`

RPXのstatic型言語は、次のBoolean演算を持つ。

```text
union
intersection
negation
difference
```

概念形：

```text
union(s, t)
intersect(s, t)
not(s)
diff(s, t)
```

differenceは次と等価である。

```text
diff(s, t)
≃
intersect(s, not(t))
```

`not(s)`は、static universeである`any`から`s`の値を除いた型である。

```text
union(s, not(s)) ≃ any
intersect(s, not(s)) ≃ never
```

Boolean演算を`dynamic s`へ直接適用してはならない。

```text
not(dynamic s)
```

は認めない。

***

##### `DD-TYP-BOOL-002`: Surface negationの制限

`確定`

RPX v1のSurfaceで記述可能なnegationは、data-type fragmentへ限定する。

対象は少なくとも次である。

* primitive型
* 限定されたsingleton型
* nominal variant
* finite union
* finite intersection
* closed immutable recordの判定可能な範囲
* regular recursive dataのconstructor差分

次に対するSurface negationは認めない。

* function型
* effectful function型
* handler型
* dynamic型
* EffectRow
* resumption
* scoped／affine capability
* resource authority

EffectRowにsemantic negationを導入しない。

effectが存在しないことは、EffectRow固有のlacks constraintとして扱う。

```text
e lacks resource
```

checker内部では、semantic subtyping判定のため、Surfaceより広いnegation相当表現を使用してよい。

内部判定が完全fragmentを超える場合は`unknown`を返し得る。

***

## Singleton型

##### `DD-TYP-SINGLETON-001`: singleton型

`確定`

singleton型は、ちょうど一つのruntime値だけを含むstatic型である。

```text
[[42]]      = {42}
[["page"]]  = {"page"}
[[true]]    = {true}
```

RPX v1でsingleton型として使用できる対象は次とする。

* `true`
* `false`
* `unit`
* `int` literal
* `str` literal
* symbol／keyword literalを採用した場合のそのliteral
* payloadを持たないvariant constructor

関係例：

```text
42 <: int
"page" <: str
true <: bool
loading <: status
```

同じdomain内の異なるsingletonは互いに素である。

```text
intersect(42, 43) ≃ never
intersect("page", "slide") ≃ never
```

次のsingleton型はRPX v1では提供しない。

* 一般の`f64`値
* NaN
* record値
* collection値
* picture値
* closure
* handler
* resource handle
* dynamic値
* scoped／affine capability

`f64` singleton型は、NaN、signed zero、数値等値およびbitwise equalityの仕様が確定するまで保留する。

型位置に置かれたliteralをsingleton型として解釈できる。

```lisp
; [既決原則に基づくSurface候補]
(type page-kind
  (union "page" "slide" "artboard"))
```

***

## Function型

##### `DD-TYP-FN-001`: fixed-arity function

`確定`

RPX v1の基本関数型は、引数個数が静的に固定されたfixed-arity functionである。

```text
() -> t
(a) -> t
(a, b) -> t
```

次は別の関数型として扱う。

```text
(a, b) -> t
```

```text
a -> (b -> t)
```

currying、uncurryingおよびpartial applicationをsubtyping変換として暗黙に行わない。

可変長引数、optional引数、keyword引数、overloadされたcalling conventionは、v1の基本function subtyping対象外とする。

***

##### `DD-TYP-FN-002`: fixed-arity function subtyping

`確定`

source function型を次とする。

```text
(a1, ..., an) ->{e-source} r
```

target function型を次とする。

```text
(b1, ..., bn) ->{e-target} u
```

sourceがtargetのsubtypeである条件は次のとおりである。

```text
1. arityが等しい。

2. すべての引数位置iについて:
   bi <: ai

3. 結果型について:
   r <: u

4. required effectsについて:
   e-source ⊑ e-target
```

規則形：

```text
arity(ā) = arity(b̄)

∀i. bi <: ai

r <: u

e-source ⊑ e-target
────────────────────────────────────────
(ā) ->{e-source} r
<:
(b̄) ->{e-target} u
```

関数引数は反変、結果は共変である。

required effectsが少ない関数は、より多くのeffectを許容する関数型の場所で使用できる。

```text
int ->{} str
<:
int ->{log} str
```

逆は成立しない。

numeric promotionをfunction subtypingへ混入させない。

```text
int </: f64
```

なので、自動的に次を導かない。

```text
int -> int
<:
f64 -> f64
```

***

##### `DD-TYP-FN-003`: function intersection

`確定`

function intersectionは、一つの関数値が複数のfunction型契約を同時に満たすことを表す。

```lisp
; [既決原則に基づくSurface候補]
(type normalize
  (intersect
    (fn (int) int)
    (fn (f64) f64)))

(val normalize
  (fn (value)
    ...))
```

function intersection自体は、複数のruntime実装を型branchごとに登録・dispatchする機能ではない。

runtimeで呼び出される関数値は一つである。

異なる実装を自動選択するoverloadまたはmulti-methodは別機能とし、RPX v1では導入しない。

***

##### `DD-TYP-FN-004`: function intersectionの適用可能性

`確定`

branchの入力領域はparameter型のtupleとして定義する。

```text
branch:
(p1, ..., pn) ->{e} r

input(branch):
tuple(p1, ..., pn)
```

呼出し引数のstatic型tupleを`a`とする。

branchが適用可能である条件は次である。

```text
a <: input(branch)
```

引数型とparameter型が一部だけ交差することは、適用可能性として不十分である。

```text
argument:
number

parameter:
int
```

では：

```text
intersect(number, int) ≄ never
```

だが：

```text
number </: int
```

なので、そのbranchは適用可能ではない。

実行時値が偶然`int`である可能性を理由に、暗黙runtime dispatchを行わない。

***

##### `DD-TYP-FN-005`: branch specificity

`確定`

適用可能branchが複数ある場合、入力領域が最も狭いbranchを最具体branchとする。

branch`b1`が`b2`より入力上具体的である条件：

```text
input(b1) <: input(b2)
```

strictに具体的である条件：

```text
input(b1) <: input(b2)
and
input(b2) </: input(b1)
```

入力領域が包含関係にある場合、specific branchはgeneral branchの契約を強化しなければならない。

```text
input(specific) <: input(general)

result(specific) <: result(general)

required-effects(specific)
⊑
required-effects(general)
```

安全な例：

```text
general:
number ->{log} str

specific:
int ->{} "integer"
```

不正な例：

```text
general:
number ->{} str

specific:
int ->{file-write} picture
```

specific branchが結果型とrequired effectsの両方でgeneral branchの契約を破るため拒否する。

***

##### `DD-TYP-FN-006`: function intersectionのcoherence

`確定`

function intersectionを構成するすべてのbranch pairについてcoherenceを検査する。

###### 入力領域が互いに素

```text
intersect(input(b1), input(b2)) ≃ never
```

なら問題ない。

###### 片方が他方を包含

specific branchの結果型とrequired effectsがgeneral branchの契約を強化することを要求する。

###### 入力領域が等価

結果型とrequired effectsも等価である場合だけ、重複branchとして正規化できる。

それ以外はduplicate／incoherent branchとして拒否する。

###### 入力領域が重なるが非比較

```text
intersect(input(b1), input(b2)) ≄ never

input(b1) </: input(b2)

input(b2) </: input(b1)
```

なら、RPX v1ではfunction intersectionを拒否する。

source order、declaration order、import orderまたは期待結果型によってbranchを選ばない。

***

##### `DD-TYP-FN-007`: union引数とdispatch

`確定`

引数のstatic型がunionである場合、そのunion全体を受け取れるbranchだけを適用可能とする。

```text
branches:
int -> str
str -> int

argument:
int | str
```

では、どちらのbranchもunion全体を受け取れないためapplicationを拒否する。

unionの各構成要素を別branchへ暗黙分配し、runtime tag dispatchを自動生成してはならない。

必要な場合は、利用者が明示的な`match`またはtype predicateを使用する。

***

## RecordRow

##### `DD-TYP-ROW-001`: closed、open、row-polymorphic record

`確定`

次の三種類を区別する。

###### Closed record

明示されたfieldだけを持つ。

```text
{x : f64, y : f64}
```

###### Open record

明示fieldを少なくとも持ち、追加fieldを許すが、その構成を型関係として保持しない。

```text
{x : f64, y : f64, ...}
```

###### Row-polymorphic record

追加fieldの構成をRecordRow変数として保持する。

```text
{x : f64, y : f64, ..r}
```

open recordとrow variableを同一視しない。

入力に存在した未知fieldを出力へ保存する関数にはrow variableを使用する。

***

##### `DD-TYP-ROW-002`: closed recordの正確なshape

`確定`

closed recordは、明示されたfield集合だけを持つ。

```text
{x : int, y : str, z : bool}
</:
{x : int, y : str}
```

追加fieldを許す型が必要な場合はopen recordを使用する。

```text
{x : int, y : str, z : bool}
<:
{x : number, y : str, ...}
```

これにより、schema validation、serialization、dynamic record validation等で余分なfieldの扱いを明示できる。

***

##### `DD-TYP-ROW-003`: immutable field covariance

`確定`

通常のRPX recordは不変値である。

そのため、同一field集合または互換なopen shapeを持つrecord間でfield型を共変に扱える。

```text
{x : int}
<:
{x : number}
```

mutable record fieldは通常record機構には導入しない。

更新は新しいrecordを返すfunctional updateとして扱う。

***

##### `DD-TYP-ROW-004`: RecordRow変数とlacks制約

`確定`

明示fieldとRecordRow変数を組み合わせる場合、同一labelの重複を防ぐlacks制約を自動生成する。

```text
{x : f64, y : f64, ..r}
```

は内部的に次を要求する。

```text
r lacks x
r lacks y
```

RecordRowではduplicate fieldを許可しない。

RecordRow変数、EffectRow変数および通常型変数は別kindであり、相互に代入できない。

***

##### `DD-TYP-ROW-005`: field extensionとupdate

`確定`

field extensionとfield updateを別の型操作とする。

###### Extension

存在しないfieldを追加する。

```text
extension requires:
row lacks label
```

###### Update

既存fieldの値または型を置き換える。

```text
update requires:
label is present
```

同じSurface操作でextensionとupdateを曖昧に実行してはならない。

これにより、field名の誤記が意図しない新規fieldとして受理されることを防ぐ。

***

##### `DD-TYP-ROW-006`: row-preserving update

`確定`

未知のfield構成を入出力間で保存する関数を完全fragmentへ含める。

```text
forall a r.
  {title : a, ..r}
  × str
  ->
  {title : str, ..r}
```

入力のRecordRow変数`r`を、出力へ同一のrowとして保持する。

複数の独立row変数間の複雑な関係は完全fragment外とする。

***

##### `DD-TYP-ROW-007`: optional field

`確定`

optional fieldと、`option`値を持つrequired fieldを区別する。

```text
author? : str
```

は、field自体が存在しない可能性を表す。

```text
author : option<str>
```

は、fieldが必ず存在し、その値が`some`または`none`であることを表す。

optional fieldはRecordRow domain固有のabsence状態を持つ。

absenceは通常のruntime値ではなく、通常のunion型へ混入させない。

関係例：

```text
{author : str, ...}
<:
{author? : str, ...}
```

```text
{author? : str, ...}
</:
{author : str, ...}
```

field不在を表すにはrecord全体のnegationではなく、lacks constraintを優先する。

***

##### `DD-TYP-ROW-008`: recordのBoolean演算

`確定`

closed immutable recordの有限Boolean combinationについて、emptiness、subtyping、equivalenceおよびdisjointnessを完全判定fragmentへ含める。

同じfieldのintersectionはfield型のintersectionとして処理する。

```text
intersect(
  {x : number, ...},
  {x : int, ...}
)
≃
{x : int, ...}
```

field型のintersectionが`never`なら、record intersectionも`never`である。

record unionを、optional fieldを並べた単一recordへ一般には変換しない。

```text
{x : int} | {y : str}
```

は、次と等価ではない。

```text
{x? : int, y? : str}
```

一般のopen record negation、複数row変数を含むBoolean combinationでは`unknown`を返し得る。

***

## Recursive data type

##### `DD-TYP-REC-001`: recursive data type

`確定`

RPX v1はrecursive data typeを持つ。

再帰型は、主にvariant／data定義から生成する。

一般の`mu`型をSurfaceへ公開しない。

`data`等の正確なSurface構文は別途決定する。

checker内部では、recursive typeを有限の循環型graphとして表現してよい。

***

##### `DD-TYP-REC-002`: equi-recursiveな利用者意味論

`確定`

Surface利用者へ明示的な`fold`／`unfold`を要求しない。

再帰型とその一段展開を、型検査上equi-recursiveに扱う。

```text
list<a>
≃
nil | cons<a, list<a>>
```

実装では、型graph、pair memoizationおよびcoinductive subtype checkingを使用する。

内部実装が消去可能なfold／unfold evidenceまたはcastを持つことは許容するが、runtimeの利用者向け操作として観測可能にしてはならない。

***

##### `DD-TYP-REC-003`: contractiveness

`確定`

recursive data型の再帰参照はcontractiveでなければならない。

再帰参照へ到達するまでに、variant constructor、immutable data constructor等、値構造を一段形成する型constructorを通らなければならない。

許可：

```text
list<a>
=
nil | cons<a, list<a>>
```

拒否：

```text
a = a
```

```text
a = b
b = a
```

```text
a = int | a
```

単なるalias、union、intersectionまたはnegationだけを通る循環をRPX v1では認めない。

***

##### `DD-TYP-REC-004`: strict positivity

`確定`

recursive data型の自己参照はstrictly positiveな位置にだけ現れなければならない。

許可：

```text
tree<a>
=
leaf<a>
|
branch<tree<a>, tree<a>>
```

拒否：

```text
bad
=
make<bad -> int>
```

function parameter、negation、dynamic、resumption、handler、capability等を介したrecursive occurrenceをRPX v1では認めない。

***

##### `DD-TYP-REC-005`: regularity

`確定`

RPX v1はregular recursionだけを許す。

再帰参照は、原則として同じ型parameter構造へ戻らなければならない。

許可：

```text
list<a>
→ list<a>
```

拒否：

```text
nested<a>
→ nested<list<a>>
```

```text
flip<a, b>
→ flip<b, a>
```

non-regular recursionはv1では認めない。

***

##### `DD-TYP-REC-006`: base constructor

`確定`

recursive data定義は、有限のstrict値を構築可能な非再帰的base constructorを少なくとも一つ持たなければならない。

許可：

```text
list<a>
=
nil
|
cons<a, list<a>>
```

拒否：

```text
endless
=
next<endless>
```

empty型が必要な場合は既存の`never`を使用する。

***

##### `DD-TYP-REC-007`: recursive dataの完全判定範囲

`確定`

次を満たすregular recursive variantについて、subtyping、constructor差分、exhaustiveness、constructor disjointnessおよび共変parameter比較を完全fragmentへ含める。

* contractive
* strictly positive
* regular
* immutable
* base constructorを持つ
* 同じ型parameter構造へ再帰する

arbitrary recursive negation、recursive open record、functionを経由するrecursion等は、完全fragment外とする。

recursive immutable dataは`RuntimeCheckable`にできる。

dynamic castでは構造を再帰的に検査し、検査済みevidenceをcacheしてよい。

cyclic runtime valueはRPX v1のrecursive dataには含めない。

***

## Bidirectional type checking

##### `DD-TYP-BIDI-001`: bidirectional typing

`確定`

RPX v1はbidirectional type checkingを採用する。

型検査器は、次の二つの判断を持つ。

###### Synthesis

```text
Γ ⊢ e ⇒ t ! r
```

式`e`から値型`t`および最小required effects `r`を求める。

###### Checking

```text
Γ ⊢ e ⇐ t ! r
```

式`e`が期待型`t`へ適合するか検査し、最小required effects `r`を求める。

型注釈は独立した`(type ...)`宣言によって期待型を与える。

***

##### `DD-TYP-BIDI-002`: 型注釈付きbinding

`確定`

対応する`(type ...)`宣言が存在するbindingについて、initializerを宣言型に対してcheckする。

```lisp
(type add
  (fn (int int) int))

(val add
  (fn (x y)
    (+ x y)))
```

関数parameter型と結果型は期待関数型から伝播する。

型注釈は無検査の仮定ではなく、initializerが注釈型へ適合することを検証する。

型注釈はvalue restrictionを迂回しない。

***

##### `DD-TYP-BIDI-003`: 無注釈binding

`確定`

対応する`(type ...)`宣言がないbindingについて、initializerの型とrequired effectsを原則としてsynthesizeする。

```lisp
(val identity
  (fn (x) x))
```

単純な局所制約から一意の型を得られる場合は注釈なしで受理する。

```text
identity :
forall a.
  a -> a
```

一意に決まらない場合は、任意の解を選ばず`annotation-required`とする。

***

##### `DD-TYP-BIDI-004`: synthesis可能な式

`確定`

次を原則としてsynthesis可能とする。

* literal
* 変数参照
* function application
* field access
* record literal
* non-empty immutable collection
* payloadから型parameterを決められるvariant constructor
* 目標型が構文上明示されたsafe cast
* `let`／`val` initializer
* 単純な`fn`
* 単純なhandler literal

この一覧は閉じた列挙ではなく、各Surface formについて規則を別途定める。

***

##### `DD-TYP-BIDI-005`: checkingを優先する式

`確定`

次は期待型に対するcheckingを優先する。

* `(type ...)`付きbindingのinitializer
* `fn`に期待関数型がある場合
* empty collection
* function intersection型を持つ関数値
* 型parameterがpayloadだけでは決まらないconstructor
* 複雑なhandler literal
* 曖昧な再帰関数グループ
* 型が一意に決まらないRecordRow式

`fn`はchecking専用にはしない。

期待型がない場合でもfresh metavariableと局所制約を使用してsynthesisを試みる。

***

##### `DD-TYP-BIDI-006`: `if`と`match`

`確定`

`if`および`match`は、checking modeでは期待型を到達可能な各branchへ伝播する。

synthesis modeでは、到達可能な各branchの結果型のunionを合成する。

```text
then : s
else : t

result:
union(s, t)
```

unreachable branchは結果unionおよびrequired effectsへ影響させない。

各branchのrequired effectsを満たす最小のresidual EffectRowを求める。

実行時には一branchだけが実行されても、静的には到達可能な全branchを処理できるeffect environmentを要求する。

***

##### `DD-TYP-BIDI-007`: subtyping、promotion、dynamic cast

`確定`

式を期待型に対してcheckするとき、次の処理を概念上区別する。

```text
1. 直接の型等価／subtyping
2. 許可されたnumeric promotion
3. bounded dynamic cast
4. それ以外はtype-error
```

型`s`を合成し、期待型`t`がある場合の基本規則：

```text
Γ ⊢ e ⇒ s ! r
s <: t
────────────────
Γ ⊢ e ⇐ t ! r
```

`int`から`f64`はsubtypingではなくnumeric promotionである。

`dynamic s`から`t`への変換は、`OPEN-TYP-001`の三段階規則に従う。

dynamic castは、目標型が明確なchecking boundaryで挿入する。

***

##### `DD-TYP-BIDI-008`: annotation-requiredとchecker-limitation

`確定`

次を明確に区別する。

```text
型を決める情報が不足
→ annotation-required

必要情報はあるがalgorithmが判定不能
→ checker-limitation

型関係が成立しない
→ type-error
```

`(type ...)`注釈を追加しても同じ難しい型関係の判定が必要な場合、checker limitationは解消しない可能性がある。

***

## 明示的多相型

##### `DD-TYP-POLY-001`: 明示的`forall`

`確定`

明示的な多相型注釈では、すべての型変数を`forall`で明示的に束縛する。

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))
```

型注釈内の未束縛型名を暗黙量化しない。

次は静的エラーである。

```lisp
(type identity
  (fn (a) a))
```

```text
unbound type variable:
  a
```

無注釈bindingでは、value restrictionを満たす範囲でcheckerが自動generalizationできる。

```lisp
(val identity
  (fn (x) x))
```

```text
inferred:
forall a.
  a -> a
```

***

##### `DD-TYP-POLY-002`: `forall` binderのkind

`確定`

明示的`forall` binderにはkindを必須とする。

RPX v1で明示的に量化できるkindは少なくとも次とする。

```text
type
record-row
effect-row
```

例：

```lisp
(forall ((a type)
         (r record-row)
         (e effect-row))
  ...)
```

kindの異なる変数を相互に代入してはならない。

通常型変数、RecordRow変数およびEffectRow変数は、名前が同じでも別categoryである。

kindを省略する短縮構文はRPX v1の標準構文には採用しない。

***

##### `DD-TYP-POLY-003`: `forall`のscope

`確定`

`forall`で束縛された変数のscopeは、その`forall`のbodyだけとする。

同一型式内で、外側の量化変数を同名でshadowすることを禁止する。

別の型注釈で同じ綴りを使用することは許すが、名前解決後の型変数BindingIdは異なる。

未使用の量化変数は型として受理可能だが、warningを出す。

```text
warning:
quantified type variable `a` is not used
```

型等価判定ではalpha-renamingを無視する。

source上のbinder順序は、診断およびpretty printingのために保持する。

***

##### `DD-TYP-POLY-004`: rank-1／prenex制限

`確定`

RPX v1の多相性はrank-1／prenexに限定する。

bindingの型schemeの最外部にある`forall`だけを許す。

nested `forall`は最外部へ正規化する。

```text
forall a.
  forall b.
    t
```

を次へ正規化する。

```text
forall a b.
  t
```

関数引数、関数結果、record fieldその他の内部位置にある`forall`は認めない。

次のようなhigher-rank型はRPX v1では禁止する。

```lisp
(type use-polymorphic
  (fn ((forall ((a type))
         (fn (a) a)))
      int))
```

***

## 完全性分類

##### `DD-TYP-FRAG-001`: A — 完全判定fragment

`確定`

次の問題形状について、checkerはresource budgetの範囲内で`proved`または`disproved`を返し、algorithmic limitationによる`unknown`を返さないことを目標とする。

* `any`、`never`
* primitive型
* 限定されたsingleton型
* nominal variant
* finite algebraic data type
* A-fragmentの有限union
* A-fragmentの有限intersection
* data-type fragmentの制限付きnegation
* data-type fragmentのdifference
* fixed-arity function subtyping
* coherent function intersection
* closed immutable record
* 単純なopen record
* 一個のRecordRow変数
* RecordRow lacks constraint
* field read
* field extension
* field update
* row-preserving update
* optional field
* closed EffectRow
* 一個のEffectRow tail
* duplicate-awareな単純EffectRow制約
* 一層単位のhandler elimination
* regular recursive variant
* regular recursive variantのexhaustiveness
* A-fragment上のbounded dynamic判定
* rank-1／prenex polymorphismの通常fragment
* 構文的value restriction
* 単相再帰
* explicit `forall`のkind checking

これは型constructor単位ではなく、問題形状単位の分類である。

***

##### `DD-TYP-FRAG-002`: B — `unknown`を返し得るfragment

`確定`

次は意味論上定義されるが、v1 checkerが`unknown`を返し得る。

* open recordのgeneral negation
* RecordRow変数を含む複雑なBoolean combination
* 複数の独立RecordRow変数
* optional fieldと複数row変数の複雑な組合せ
* recursive RecordRow
* 複数段のrow difference
* 複数の独立EffectRow tail
* named effect instanceを含む高度なrow制約
* 複雑なduplicate-label tallying
* 複数段のhandler subtraction
* arbitrary recursive typeのnegation
* recursive open record
* function型を含む内部の一般的Boolean combination
* row-polymorphic function intersection
* overlap判定がB-fragmentに依存するfunction intersection
* 高度なhandler型
* 複数の非比較なconstraint solution

B-fragmentで`unknown`が生じた場合は`checker-limitation`として拒否する。

***

##### `DD-TYP-FRAG-003`: C — 注釈を要求し得るfragment

`確定`

型関係の判定自体は可能だが、式から一意の型を合成する情報が不足している場合、`(type ...)`注釈を要求する。

代表例：

* empty collection
* 曖昧なnumeric operation
* function intersection型の発見
* 無注釈の複雑なhandler literal
* 曖昧な再帰関数グループ
* 複数の型候補を持つconstructor
* 型解が一意でないRecordRow式
* 型解が一意でないEffectRow式
* 期待型で一意化可能な高階関数
* public APIで推論結果を固定する必要がある場合

C-fragmentの診断は`annotation-required`とする。

***

##### `DD-TYP-FRAG-004`: D — RPX v1で禁止するfragment

`確定`

次をRPX v1では禁止する。

* 多相再帰
* higher-rank polymorphism
* 明示注釈内の暗黙量化
* 一般のSurface `mu`型
* non-contractive recursive type
* negative recursive occurrence
* non-regular recursive type
* base constructorを持たないrecursive data
* function型のSurface negation
* effectful function型のSurface negation
* handler型のSurface negation
* dynamic型のnegation
* EffectRowのsemantic negation
* gradual effect-row cast
* runtime effect monitoringによるfunction cast
* dynamic handler cast
* dynamic resumption cast
* scoped／affine capabilityのdynamic化
* RecordRowとEffectRowのkind混同
* RecordRowのduplicate field
* dynamic field labelを持つtyped record
* union引数の暗黙runtime overload dispatch
* 型branchごとの複数実装dispatch
* source orderによるfunction branch選択

これらは単なるchecker limitationではなく、v1のunsupported featureまたは静的違反とする。

***

## 共通constraint worklist

##### `DD-TYP-SOLVER-001`: shared constraint worklist

`確定`

RPX v1の型検査器は、通常型、semantic subtyping、RecordRow、EffectRow、kind、scopeおよびcapabilityを、単純な一方向pipelineとして独立処理しない。

すべてのconstraintを共通worklistで管理し、各専門solverが相互にconstraintを生成する方式を採用する。

概念的なconstraint category：

```text
constraint ::=
    kind-eq
  | type-eq
  | type-subtype
  | type-empty
  | type-inhabited
  | record-row-eq
  | record-row-includes
  | record-row-lacks
  | record-field-present
  | record-field-absent
  | effect-row-eq
  | effect-row-includes
  | effect-row-requires
  | effect-row-handles-one
  | overload-coherence
  | dynamic-cast-obligation
  | scope-non-escape
  | capability-usage
  | generalization-readiness
```

***

##### `DD-TYP-SOLVER-002`: solver間のconstraint生成

`確定`

各solverは担当constraintを処理し、必要に応じて別categoryのconstraintをworklistへ追加する。

RecordRowの例：

```text
record-row-eq(
  {x : int, ..r},
  {x : a, ..s}
)
```

から：

```text
type-eq(int, a)
record-row-eq(r, s)
record-row-lacks(r, x)
record-row-lacks(s, x)
```

を生成できる。

Function subtypingの例：

```text
type-subtype(
  int ->{resource} str,
  int ->{resource, log} str
)
```

から：

```text
type-subtype(int, int)
type-subtype(str, str)
effect-row-includes(
  <resource>,
  <resource, log>
)
```

を生成する。

Dynamic castの例：

```text
dynamic(union(int, str))
→ expected number
```

では、部分型と空性constraintを生成する。

```text
type-subtype(
  union(int, str),
  number
)
```

必要に応じて：

```text
type-empty(
  intersect(
    union(int, str),
    number
  )
)
```

を生成する。

***

##### `DD-TYP-SOLVER-003`: constraint処理の優先度

`確定`

共有worklistは固定の一方向pipelineではないが、constraint processingに決定的な優先度を設ける。

推奨優先度：

```text
1. kind constraints
2. 明白なtype equality
3. ordinary metavariable unification
4. RecordRow constraints
5. EffectRow constraints
6. semantic subtyping／emptiness
7. overload coherence
8. dynamic cast obligations
9. scope／capability constraints
10. generalization readiness
```

各solverが新しいconstraintを生成した場合、対応する優先度queueへ追加する。

同一優先度内では、source order、stable constraint IDまたは明示されたcanonical順序により決定的に処理する。

***

##### `DD-TYP-SOLVER-004`: canonicalizationとmemoization

`確定`

同一または等価なconstraintの無限再投入を防ぐため、すべてのconstraintをcanonical formへ正規化する。

処理済みconstraintおよび現在処理中のrecursive type pairをmemoizeする。

少なくとも次を考慮する。

* alpha-renamingされた型変数
* union／intersectionの順序
* 重複型要素
* EffectRowのduplicate label
* RecordRow field順序
* recursive type graph identity
* alias展開後の型identity
* source-independentなconstraint key

RecordRow field順序は型等価に影響させない。

EffectRowのduplicate labelは消去してはならない。

recursive type comparisonでは、処理中pairをcoinductive hypothesisとして利用する。

***

##### `DD-TYP-SOLVER-005`: solver終了状態

`確定`

共通worklistの処理結果を次のように分類する。

###### 成功

```text
worklistが空
かつ
必須metavariableが解決済み
かつ
scope／capability constraintが成立
かつ
generalization条件が成立
```

###### Type error

いずれかの必須constraintが`disproved`になった。

###### Annotation required

型関係は既知のalgorithmで検査可能だが、型合成に必要なmetavariableの解が一意に定まらない。

###### Checker limitation

意味の定義されたconstraintが残るが、v1 solverが`proved`または`disproved`へ到達できない。

###### Resource limit

決定的なsolver step budgetを超過した。

***

##### `DD-TYP-SOLVER-006`: cast insertionとgeneralizationの順序

`確定`

bounded dynamic castは、必要なsubtypingおよびemptiness constraintを解決した後に挿入する。

generalizationは、次を確認した後に行う。

* initializerの型とrequired effects
* 構文的value restriction
* capture set
* local state identity
* scoped／affine capability
* unresolved metavariable
* explicit `(type ...)` annotationとの整合性

scope escapeまたはcapability captureが後から判明した場合に、既に不正なgeneralizationを確定してはならない。

実装はgeneralization候補を一時的に作成し、scope／capability検査完了後にschemeを確定してよい。

***

## 型検査器の概念pipeline

##### `DD-TYP-SOLVER-007`: checkerの全体処理順序

`確定`

型検査器の概念的な処理順序を次とする。

```text
1. Surface declaration collection

2. lexical／namespace resolution

3. kind resolution

4. explicit `(type ...)` annotationのkind check

5. bidirectional type／effect constraint generation

6. shared constraint worklist processing
   - ordinary type constraints
   - RecordRow constraints
   - EffectRow constraints
   - semantic subtyping
   - emptiness
   - overload coherence

7. bounded dynamic cast obligationの解決

8. recursive group constraintの確定

9. value restrictionとgeneralization候補の作成

10. scope／capability escape checking

11. generalizationの確定

12. cast evidence生成と簡約

13. typed Core validation
```

実装は増分的またはworklist駆動で処理してよく、物理的に13回の完全走査を要求するものではない。

ただし、観測可能な判定結果および診断分類はこの依存関係に従わなければならない。

***

## 適合試験

##### 三値判定

```text
int <: number
```

期待：

```text
proved
```

```text
str <: number
```

期待：

```text
disproved
```

複雑な複数row変数とnegationの関係：

```text
checkerが完全fragment外ならunknownを返してよい
```

***

##### 診断分類

空vector：

```lisp
(val values
  (vector))
```

期待：

```text
annotation-required
```

複雑なopen record negation：

```text
必要な型情報がすべてあるがsolverが判断不能
```

期待：

```text
checker-limitation
```

明白な型不一致：

```lisp
(+ "text" 1)
```

期待：

```text
type-error
```

***

##### 最小required effects

```lisp
(val make-label
  (fn (label)
    (log label)
    (make-text label)))
```

期待される型：

```text
str ->{log} picture
```

周囲のeffect environmentに`resource`や`file-write`があっても、それらを関数型へ混入させない。

***

##### 関数値のeffect

```lisp
(val renderer
  (fn (project)
    (load-image project)))
```

`fn`式のexpression effects：

```text
<>
```

生成された関数のrequired effects：

```text
<resource>
```

***

##### EffectRow polymorphism

```text
map :
forall a b e.
  ((a ->{e} b), vector<a>)
  ->{e}
  vector<b>
```

pure callback：

```text
e := <>
```

`log` callback：

```text
e := <log>
```

***

##### Handlerによるeffect縮小

body：

```text
<resource, cache, log>
```

`cache`および`log`を処理するrunner適用後：

```text
<resource>
```

***

##### Singleton型

```text
42 <: int
"page" <: str
true <: bool
```

期待：

```text
proved
```

```text
intersect(42, 43)
```

期待：

```text
never
```

***

##### Function subtyping

```text
animal ->{} dog
<:
dog ->{log} animal
```

前提：

```text
dog <: animal
```

期待：

```text
proved
```

***

##### Function intersection coherence

```text
number ->{log} str
int ->{} "integer"
```

期待：

```text
coherent
```

```text
number ->{} str
int ->{file-write} picture
```

期待：

```text
incoherent function intersection
```

***

##### 非比較なbranch overlap

```text
(int | str) -> bool
(int | bool) -> str
```

期待：

```text
静的エラー:
overlapping incomparable function branches
```

***

##### Union引数の暗黙dispatch禁止

branches：

```text
int -> str
str -> int
```

argument：

```text
int | str
```

期待：

```text
no applicable branch
明示的なmatchを要求
```

***

##### Closed record

```text
{x : int, y : str, z : bool}
<:
{x : int, y : str}
```

両方closedの場合：

```text
disproved
```

targetがopenの場合：

```text
{x : int, y : str, z : bool}
<:
{x : number, y : str, ...}
```

期待：

```text
proved
```

***

##### Row preservation

```text
forall r.
  {title : a, ..r}
  ->
  {title : str, ..r}
```

入力：

```text
{
  title : symbol,
  author : str,
  page-count : int
}
```

期待出力型：

```text
{
  title : str,
  author : str,
  page-count : int
}
```

***

##### Optional field

```text
{author : str, ...}
<:
{author? : str, ...}
```

期待：

```text
proved
```

逆方向：

```text
disproved
```

***

##### Recursive data

```text
list<int>
<:
list<number>
```

前提：

```text
int <: number
```

期待：

```text
proved
有限のpair memoizationで終了
```

***

##### 非contractive再帰

```text
a = a
```

期待：

```text
unsupported／invalid recursive type
```

***

##### Non-regular recursion

```text
nested<a>
=
stop
|
more<nested<list<a>>>
```

期待：

```text
unsupported recursive type
```

***

##### Bidirectional checking

```lisp
(type values
  (vector picture))

(val values
  (vector))
```

期待：

```text
checking modeで受理
```

無注釈：

```lisp
(val values
  (vector))
```

期待：

```text
annotation-required
```

***

##### Explicit `forall`

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))
```

期待：

```text
受理
```

```lisp
(type identity
  (fn (a) a))
```

期待：

```text
unbound type variable:
  a
```

***

##### Kind error

```lisp
(forall ((r record-row))
  (fn (r) r))
```

RecordRowを通常値型位置で使用している場合：

```text
kind error
```

***

##### Solver determinism

同一source、compiler version、dependency signatureおよびchecker budgetで複数回検査する。

期待：

```text
同じconstraint処理結果
同じ診断分類
同じprimary span
同じsolver-budget結果
```

***

## 未解決事項の移管

##### `OPEN-SYN-002`

* `union`、`intersect`、`not`、`diff`の最終Surface構文
* singleton型literalの完全な字句規則
* `forall` binderの完全な括弧構造
* `data`／variant定義構文
* `effect-alias`の最終名称
* EffectRow tailの最終構文
* RecordRow tailの最終構文
* field access、extension、updateのSurface構文
* optional fieldのSurface構文
* explicit type ascriptionを設けるか
* safe castのSurface名称

##### `OPEN-DAT-001`

* variant／constructorの最終Surface構文
* pattern grammar
* pattern exhaustiveness policy
* guard
* literal pattern
* or-pattern
* recursive data definitionの完全な構文

##### `OPEN-EFF-001`の後続仕様

* EffectRow constraintの正確なformal rule
* duplicate labelのcanonical representation
* named effect instanceのalgorithmic比較
* effect aliasのmodule visibility
* host effect environmentとのlinking

##### `OPEN-MOD-001`

* public APIでの`(type ...)`必須要件
* signature内の明示的`forall`
* abstract typeのvariance
* separate compilation時のtype／effect identity
* imported checker metadataの信頼境界

##### `OPEN-ERR-001`

* `checker-limitation`
* `annotation-required`
* `checker-resource-limit`
* 複数constraint failureのprimary error選択
* IDEとCLIでの診断表示差

##### 将来拡張

* higher-rank polymorphism
* 多相再帰
* relaxed value restriction
* full semantic subtyping algorithm
* unrestricted type tallying
* general Surface `mu`
* non-regular recursion
* general function negation
* gradual effect typing
* multi-method
* runtime overload dispatch
* generalized scoped callback typing

***

## 解決後の基本原則

```text
宣言的型関係:
semantic subtyping

algorithmic判定:
proved / disproved / unknown
```

```text
proved:
受理可能

disproved:
type-error

unknown:
checker-limitation
```

```text
情報不足:
annotation-required

budget超過:
checker-resource-limit
```

```text
型検査方式:
bidirectional typing

synthesis:
式から型とrequired effectsを求める

checking:
期待型に対して式を検査する
```

```text
明示的多相型:
forall必須
kind必須

implicit generalization:
無注釈bindingでvalue restrictionの範囲内
```

```text
effect:
最小required effectsだけを推論
effect environment全体を型へ混入させない
handler／runnerで処理済みeffectを除去
```

```text
function subtyping:
引数反変
結果共変
required effectsが少ない方がsubtype
fixed arity一致
```

```text
function intersection:
一つの値に複数の型契約
暗黙runtime dispatchではない
非coherent overlapを拒否
```

```text
record:
closed／open／row-polymorphicを区別
immutable fieldは共変
extensionとupdateを分離
duplicate fieldは禁止
```

```text
recursive type:
data定義から生成
equi-recursive
contractive
strictly positive
regular
base constructor必須
```

```text
solver:
shared constraint worklist
solver間でconstraintを相互生成
canonicalizationとmemoization
決定的step budget
```

以上により、RPX v1におけるalgorithmic semantic subtyping、型推論、EffectRowおよびRecordRow制約、recursive data、多相性、完全判定fragment、保守的拒否およびsolver構造が特定されたため、`OPEN-TYP-002`を解決済みとする。

### 13.7 `ROW-001` Row-polymorphic records

#### 概要・状態

`確定`

RecordRowのalgorithmic意味論は、13.6.2の`DD-TYP-ROW-*`および`DD-TYP-FRAG-*`へ統合した。本節では中心原則を要約する。

- closed、open、row-polymorphic recordを区別する。
- closed recordは明示fieldだけを持つ正確なshapeである。
- open recordは明示fieldを少なくとも持ち、追加fieldを許す。
- row-polymorphic recordは追加fieldをRecordRow変数として保持する。
- 通常recordは不変であり、field型は共変に扱う。
- 明示fieldとrow tailを併用する場合、同名fieldの`lacks`制約を自動生成する。
- RecordRowではduplicate fieldを禁止する。
- extensionとupdateは別操作であり、前者は不在、後者は存在を要求する。
- optional fieldと`option`値を区別する。absenceはRecordRow固有である。
- 単一RecordRow変数によるfield保存・read・extension・update・optional fieldは完全判定fragmentに含める。
- 複数row変数、open record negation、複雑なBoolean combinationでは`unknown`を返し得る。
- RecordRowとEffectRowは別kind・別solver規則とする。

詳細は13.6.2の`DD-TYP-ROW-001`〜`DD-TYP-ROW-008`、`DD-TYP-FRAG-*`、`DD-TYP-SOLVER-*`を参照する。

### 13.8 `EFF-001` Algebraic effects and handlers

> 統合元: `OPEN-EFF-001`。同項目は解決済み。

#### 状態

`解決済み`

本決定は、RPX v1における次の事項を固定する。

- deep handler
- one-shot resumption
- operationの探索と自動伝播
- 明示的forward
- return clause
- handler単位の結果型変換
- duplicate labelを許すEffectRowの基本的意味
- ambient effectとnamed/scoped effect instance
- first-class handler value
- Core `handle`とSurface `with`
- 未処理effectの静的・動的な扱い

次の機能はRPX v1の基本handlerには含めず、将来拡張または別の
`OPEN-*`へ移管する。

- multi-shot resumption
- shallow handler
- first-class resumption
- 一般的なanswer-type modification
- 利用者向けの任意のeffect masking
- cleanup失敗を含む完全なfinalization意味論
- exception/error taxonomy
- 並行実行、cancellationおよびtask間resumption

---

#### `DD-EFF-001`: deep handler

`確定`

RPXの通常のeffect handlerはdeep handlerとする。

operationがnearest matching handlerによって処理され、operation clauseが
resumptionを再開した場合、再開された計算は同じhandlerの動的scope内で
引き続き実行される。

再開後の計算が同じeffectのoperationを再び実行した場合、そのoperationは
同じhandlerによって再び処理される。ただし、再開後により内側のmatching
handlerが設置された場合は、nearest matching handler規則に従って内側の
handlerが処理する。

概念的には、次のように動作する。

```text
operation発生
→ nearest matching handlerのoperation clause
→ resume
→ 現在のdeep handlerを再設置
→ 元の計算の残りを実行
````

deep handlerが複数のoperationを処理することは、同一resumptionを複数回
使うことを意味しない。各operationはそれぞれ別個のresumptionを生成する。

shallow handlerはRPX v1の公開言語には含めない。将来導入する場合は、
通常のhandlerとは異なる明示的な構文および型を持つ別機能として定義する。

***

#### `DD-EFF-002`: one-shot resumption

`確定`

RPXの通常のresumptionはone-shotとする。

正確には、resumptionは0回または1回だけ使用できるaffine capabilityである。

operation clauseはresumptionを次のいずれかの方法で扱う。

1. 一度だけresumeする。
2. 一度だけforwardする。
3. resumeもforwardもせず、捕捉された計算を破棄する。

同一resumptionを複数回resumeしてはならない。

同一resumptionをresumeした後にforwardしてはならない。また、forwardした後に
resumeまたは再度forwardしてはならない。

```lisp
; 許可
(op (argument resume)
  (resume value))
```

```lisp
; 許可
(op (argument resume)
  alternative-handler-result)
```

```lisp
; 拒否
(op (argument resume)
  (seq
    (resume first-value)
    (resume second-value)))
```

multi-shot resumptionは通常の`resume`の挙動として提供しない。

将来multi-shotを導入する場合は、少なくとも次を別途規定しなければならない。

* 継続を複製できる条件
* 局所状態を共有するか複製するか
* resource handleを含む継続を複製できるか
* `SourceEdit`等の外部effectを含む継続を複製できるか
* cleanupを何回実行するか
* 複製のmemory cost
* multi-shot性を型またはeffect rowで追跡する方法

multi-shotが別途導入されるまで、RPXの公開言語で利用できるresumptionは
one-shotだけとする。

***

#### `DD-EFF-003`: resumptionの型とscope

`確定`

resumptionは通常の関数型ではなく、専用のaffine型を持つ。

operationが次の型を持つとする。

```text
op : P ->{L} R
```

handler全体の結果型を`B`、再開中に外側へ残り得るeffect rowを`E`とすると、
operation clause内のresumptionは概念的に次の型を持つ。

```text
Resume<R, B, E>
```

各型引数の意味は次のとおりである。

* `R`: operation呼出地点へ返す値の型
* `B`: handler全体の結果型
* `E`: 再開された計算から外側へ残り得るeffect row

resumptionの再開は、通常の関数適用とは異なるCore項として扱う。

```text
resume(k, value)
```

型付けの概念形は次のとおりである。

```text
k : Resume<R, B, E>
value : R
────────────────────────
resume(k, value) : B ! E
```

`resume`へ渡す値の型はoperation結果型`R`である。一方、`resume`式そのものの
結果型はhandler全体の結果型`B`である。

resumptionはoperation clause内だけの特殊binderとし、RPX v1では通常の
first-class valueにしない。

次を禁止する。

* recordまたはcollectionへの格納
* moduleからのexport
* operation clauseからの返却
* 通常関数への引渡し
* closureによる捕捉
* polymorphic valueとしての一般化
* serialization
* 複製
* handlerのscope外へのescape

`resume`はtail positionだけには限定しない。したがって、再開後の結果に対して
後処理を行うことができる。

```lisp
(op (argument resume)
  (let ((result (resume value)))
    (postprocess result)))
```

各実行経路でresumptionが高々一回だけ消費されることを静的に検査する。
runtimeも防御的にresumptionの消費状態を保持し、型検査済みCoreの外部から
二重resumeが行われないことを検証する。

first-class affine resumptionは将来拡張として保留する。

***

#### `DD-EFF-004`: effect operationの呼出し

`確定`

effect operationは、通常の関数呼出しに近いSurface構文で呼び出す。

```lisp
(random)
(log "message")
(load-image resource-id)
```

`use`等の接頭辞は必須にしない。残す場合もSurface上の任意の糖衣とし、
Core operation項にはしない。

名前解決後のCoreでは、通常の関数適用とeffect operationを区別する。

```text
通常関数:
App(function, arguments)

effect operation:
Perform(operation_id, arguments)
```

operation identityは名前文字列ではなく、名前解決済みの一意な
`EffectId`および`OperationId`で識別する。

別moduleまたは別packageが同名のeffectやoperationを定義しても、identityが異なる
場合は別のoperationとして扱う。

***

#### `DD-EFF-005`: nearest matching handlerと自動伝播

`確定`

effect operationが発生した場合、runtimeはhandler stackを内側から外側へ探索し、
operation identityに一致する最初のhandler frameへ制御を移す。

handlerが処理対象として宣言していないeffectのoperationは、利用者による
明示的な記述なしに外側へ自動伝播する。

```text
operation発生
→ 最内側のhandlerが非対応なら通過
→ 次の外側のhandlerが非対応なら通過
→ nearest matching handlerが処理
```

自動伝播はoperationの再評価を意味しない。発生済みのoperation requestと捕捉された
継続を外側へ渡す。

通常のhandlerは、処理対象として宣言したeffectに属するすべてのoperationについて
clauseを持たなければならない。

対象effectのoperation clauseが不足している場合、それを暗黙のforwardとはみなさず、
静的エラーにする。

この規則により、operation clauseの書き忘れと、意図的な外側への委譲を区別する。

***

#### `DD-EFF-006`: 明示的forward

`確定`

handlerが現在処理中のoperationを外側へ委譲する場合、operation clause内で
明示的な`forward`を使用する。

```text
forward(k)
```

`forward`は次の意味を持つ。

1. 現在処理中のoperation requestを維持する。
2. 現在のhandler frameより外側から探索を再開する。
3. 次のnearest matching handlerへoperationとresumptionを委譲する。
4. resumptionを消費する。

`forward`は任意の送り先を指定できない。特定のhandlerやhost handlerへ直接飛ばしたり、
複数のmatching handlerを指定個数だけ飛び越えたりしてはならない。

```text
許可:
現在のhandler
→ 次の外側のmatching handler

禁止:
現在のhandler
→ 名前で指定した任意のhandler

禁止:
現在のhandler
→ matching handlerを二層飛び越す
```

forward後は同じresumptionをresumeまたは再度forwardできない。

handlerが新しいoperationを発生させることと、現在のoperationをforwardすることは
区別する。

```text
新しいoperationを呼ぶ:
新しいoperation requestと新しいresumptionを生成する。

forwardする:
現在のoperation requestと現在のresumptionを外側へ委譲する。
```

***

#### `DD-EFF-007`: handler clauseの実行scope

`確定`

operation clauseおよびreturn clause自身は、現在のhandler frameの外側にある
effect環境で実行する。

したがって、clause自身が現在処理中のeffectと同じeffectのoperationを発生させた場合、
現在のhandler自身ではなく、その外側のnearest matching handlerが処理する。

```text
元のbodyのoperation
→ 現在のhandler

operation clause自身が発生させたoperation
→ 現在のhandlerより外側を探索
```

これは、handler clause内で同じoperationを再発行したときの無限自己再帰を防ぎ、
handlerを段階的に合成するためである。

一方、operation clauseがresumptionをresumeした場合は、deep handlerの意味に従って
現在のhandlerを再設置し、元の計算の残りを実行する。

```text
operation clause自身の計算:
現在のhandlerより外側

resumeされた元の計算:
現在のhandlerを再設置した内側
```

return clauseにはresumptionは存在しない。bodyはreturn clauseの実行前に既に
正常終了している。

operation clauseがresumptionを使用せずに値を返した場合、暗黙のresumeは行わない。
捕捉された計算の残りは破棄する。

***

#### `DD-EFF-008`: return clause

`確定`

handlerは省略可能なreturn clauseを持つ。

return clauseは、handler対象bodyが正常終了したとき、その正常終了値を受け取って
handler全体の結果へ変換する。

bodyの正常終了型を`A`、handler全体の結果型を`B`、return clause自身のeffectを
`Hret`とすると、return clauseは概念的に次の型を持つ。

```text
A ->{Hret} B
```

return clauseを省略した場合は、次の恒等変換を補う。

```lisp
(return (value)
  value)
```

省略時は次を要求する。

```text
A ≃ B
```

handlerが結果型を変更する場合は、return clauseを明示しなければならない。

例として、Failureを`Result`へ変換するhandlerは次のように表せる。

```lisp
(handler Failure
  (return (value)
    (Ok value))

  (fail (error resume)
    (Err error)))
```

この場合、return clauseは概念的に次の型を持つ。

```text
A -> Result<A, Error>
```

operation clauseの結果型も、return clauseの結果型と同じ`B`でなければならない。

return clause自身はeffectを発生させてよい。そのeffectは現在のhandlerでは処理せず、
現在のhandlerより外側のhandlerへ伝播する。

***

#### `DD-EFF-009`: handler単位の結果型変換

`確定`

RPX v1は、handler単位でbodyの正常終了型`A`をhandler全体の結果型`B`へ変換することを
許す。

概念的なhandler型は次の形を持つ。

```text
Handler<L, A, B, H>
```

各要素の意味は次のとおりである。

* `L`: handlerが処理するeffect label
* `A`: handler対象bodyの正常終了型
* `B`: handler適用後の結果型
* `H`: return clauseおよびoperation clause自身が要求するeffect row

たとえば、結果型を変更しないhandlerは次の形になる。

```text
Handler<Random, A, A, <>>
```

Failureを`Result`へ変換するhandlerは次の形になる。

```text
Handler<
  Failure<Error>,
  A,
  Result<A, Error>,
  <>
>
```

これはhandlerの入口と出口における一つの`A -> B`変換である。

任意の式またはcontrol operationが周囲のanswer typeを段階的に変更する、
一般的なanswer-type modificationはRPX v1には導入しない。

一般的answer-type modification、answer-type polymorphismおよびその型推論は
将来拡張として保留する。

***

#### `DD-EFF-010`: handler valueのrank-1多相性

`確定`

handler valueは、通常の`val`一般化およびvalue restrictionに従ってrank-1多相に
なれる。

不変かつgeneralizableなhandler valueでは、body結果型、handler結果型および
effect-row変数を一般化できる。

結果型を変更しないRandom handlerは、概念的に次の型schemeを持ち得る。

```text
forall A E.
  Handler<Random, A, A, <>>
```

Failureを`Result`へ変換するhandlerは、概念的に次の型schemeを持ち得る。

```text
forall A E.
  Handler<
    Failure<Error>,
    A,
    Result<A, Error>,
    <>
  >
```

すべてのhandlerを無条件に一般化してはならない。

次はvalue restrictionの対象とする。

* effectfulな式によって生成されたhandler
* 局所可変状態を捕捉するhandler
* affine capabilityを捕捉するhandler
* scoped effect evidenceを不正にescapeさせるhandler
* generalizable valueに該当しないhandler生成式

handler一般化の最終的なalgorithmは、通常の`val`一般化および
`OPEN-BND-001`で決定するvalue restrictionと整合させる。

***

#### `DD-EFF-011`: first-class handler value

`確定`

handlerはfirst-class valueとする。

handler valueについて次を許可する。

* `val`への束縛
* 関数引数としての受渡し
* 関数からの返却
* record fieldへの格納
* moduleおよびpackage APIからの公開
* 不変なlexical environmentの捕捉

handler valueと、実行時に設置されたhandler instanceを区別する。

```text
handler value:
effectの解釈方法を表す再利用可能な値

handler instance:
特定のhandle適用によって生成される実行時frame
```

同じhandler valueを複数回設置した場合、設置ごとにfresh handler frameを生成する。

```text
同じhandler value h
├─ handler frame 1
└─ handler frame 2
```

handler valueが捕捉した通常の不変値は、通常のclosure captureと同じ規則で
各instanceから参照できる。

instance固有の可変状態をhandler valueそのものの基本機構には組み込まない。
instance固有状態が必要な場合は、局所stateおよびrunner関数によって構築する。

runtimeは最適化として局所stateをhandler frameへ融合してよいが、この最適化は
観測可能な意味を変更してはならない。

handler valueについて、次は提供しない。

* 一般的な等値比較
* hash
* serialization
* 捕捉環境の観測
* private module representationの観測
* handler instance identityの通常値としての取得

handlerが処理するeffect identityは静的に型へ現れなければならない。
実行時の文字列名だけで任意のeffectを処理する動的handlerは、RPX v1には導入しない。

***

#### `DD-EFF-012`: Core `handle`とSurface `with`

`確定`

Coreの`handle`は、handler valueと引数なし関数であるbody thunkを受け取る
専用の制御項とする。

```text
handle(handler_expression, body_thunk)
```

Surface上の概念形は次のとおりである。

```lisp
(handle handler-value
  (fn ()
    body))
```

RPXは正格call-by-valueであるため、bodyを通常引数として直接渡してはならない。
bodyを直接引数にすると、handler設置前にbodyが評価され、対象operationが未処理になる。

Surfaceでは`with`を糖衣構文として提供する。

```lisp
(with handler-value
  body-expression-1
  body-expression-2)
```

これはhygienicに次へelaborateする。

```lisp
(handle handler-value
  (fn ()
    (seq
      body-expression-1
      body-expression-2)))
```

Core `handle`の評価順序を次のように定める。

1. handler expressionを評価し、handler valueを得る。
2. body thunkを評価し、closureを得る。
3. handler valueからfresh handler frameを生成する。
4. handler frameを設置する。
5. body thunkを一度だけ起動する。
6. body中のoperationをhandler意味論に従って処理する。
7. bodyが正常終了した場合はreturn clauseを適用する。
8. handlerの動的scopeを終了する。
9. handler全体の結果を返す。

`handle`は通常の関数適用ではなく、Core専用項とする。

```text
App(function, arguments)
Handle(handler, body_thunk)
```

同じhandler valueを再利用することは、同じhandler instanceを再利用することを
意味しない。

***

#### `DD-EFF-013`: ambient effectとnamed/scoped effect instance

`確定`

RPXは、次の二種類のeffect利用形態を持つ。

1. ambient effect
2. named/scoped effect instance

##### Ambient effect

通常のoperationはambient effectとして呼び出す。

```lisp
(log "message")
(random)
(load-image resource-id)
```

operationは動的scope内のnearest matching handlerへ送られる。

ambient effectは、通常次の用途に使用する。

* Log
* Random
* Resource
* FileRead／FileWrite
* Failure
* host service

##### Named/scoped effect instance

同じeffectの複数handlerから呼出側が送り先を選択する必要がある場合、
named/scoped effect instanceを使用する。

概念例：

```lisp
(read state-a)
(read state-b)
```

または：

```lisp
(state-a.read)
(state-b.read)
```

表面構文は別途決定する。

named handlerの設置時に、freshなinstance identityと型付きevidenceを生成する。

```text
install named handler
→ fresh scope identity s
→ Evidence<L, s>
→ handler frame for L<s>
```

異なる設置から生成されたinstanceは、同じhandler valueを使用していても
異なるidentityを持つ。

```text
s1 ≠ s2
```

operationの送り先はevidenceによって明示する。

scoped instanceのidentityは、そのscope外へescapeしてはならない。
特に局所`var`、局所resource、transaction等に使用するinstanceは、
scope identityが結果型、外部closure、module exportまたは永続データへ漏れないことを
型検査する。

duplicate effect labelとnamed instance identityは別の機構である。

```text
duplicate effect label:
row polymorphismとnested handlerの型付けに使用する。

named instance:
同種の複数handlerからoperationの送り先を区別するために使用する。
```

***

#### `DD-EFF-014`: EffectRowの意味

`確定`

EffectRowはoperationの動的実行回数を数えるものではない。

次の計算が`log`を二回実行しても、通常必要なeffect要求は一層の`Log`である。

```lisp
(seq
  (log "a")
  (log "b"))
```

```text
実行回数:
2

effect要求:
<Log>
```

一つのdeep handlerが両operationを処理できる。

EffectRowは同一effect labelの重複を許す。

```text
<Log, Log | E>
```

重複は、operationの実行回数ではなく、row extensionの異なる由来、
nested handlerおよびeffect-polymorphicな計算を型付けするために保持する。

`<L | E>`は、effect row `E`へ一層の`L`を追加するrow extensionである。

`E`にも`L`が含まれている場合、重複を勝手に除去してはならない。

```text
E = <L, Resource>

<L | E>
= <L, L, Resource>
```

handlerは対象effectを一層だけ処理する。

```text
<L | E>
   ↓ handle L
E
```

重複がある場合も、一度ですべて除去してはならない。

```text
<L, L, Resource>
   ↓ handle L
<L, Resource>
```

次は不正である。

```text
<L, L, Resource>
   ↓ handle L
<Resource>
```

named instanceでは、instance identityをeffect labelの一部として扱う。

```text
State<s1> ≠ State<s2>
```

したがって、次は同一ラベルの重複ではなく、異なるeffect labelである。

```text
<State<s1>, State<s2>>
```

***

#### `DD-EFF-015`: ambient effect rowと制約生成

`確定`

複数の部分式のeffectを単純な集合和またはmultiset加算によって合成してはならない。

各計算にはambient effect rowを割り当て、各部分式はそのrowに対する
effect要求制約を生成する。

operationが次の型を持つとする。

```text
op : P ->{L} R
```

operation呼出しは、ambient row `E`について次を要求する。

```text
L ∈ E
```

同じ計算内で同じoperationを複数回呼び出した場合、同じeffect層が複数の
membership要求を満たしてよい。

```text
L ∈ E
L ∈ E

最小の通常解:
E = <L>
```

これを機械的に次へ変換してはならない。

```text
E = <L, L>
```

複数の部分式については、それぞれのeffect要求を満たす共通のambient rowを求める。

```lisp
(seq
  (log "start")
  (random)
  (log "finish"))
```

制約：

```text
Log ∈ E
Random ∈ E
Log ∈ E
```

通常の最小解：

```text
E = <Log, Random>
```

一方、effect-polymorphicなrowへ明示的に一層を追加する場合はrow extensionを使用する。

```text
<Log | E>
```

`E`にも`Log`が含まれている場合、その重複は維持する。

規範仕様では、曖昧な`join(E1, E2)`を基本演算としない。

代わりに、次の関係および制約を用いる。

```text
Requires(L, E)
```

`E`がeffect label `L`を少なくとも一層含むことを要求する。

```text
Includes(E1, E2)
```

`E2`が`E1`のeffect要求を満たすことを要求する。

```text
HandlesOne(L, Ebody, Erest)
```

`Ebody`が`<L | Erest>`と単一化可能であり、handlerが`L`を一層処理することを表す。

正確なrow unification algorithm、principal solutionおよびalgorithmic completenessは
`OPEN-TYP-002`と接続して別途検証する。

***

#### `DD-EFF-016`: handlerの型付け骨格

`確定`

effect operationを次のように表す。

```text
op : P ->{L} R
```

handler対象bodyの型を次のように表す。

```text
body : A ! <L | E>
```

Coreではbodyをthunkとして渡す。

```text
body_thunk : Unit ->{<L | E>} A
```

handler valueの概念型を次とする。

```text
Handler<L, A, B, H>
```

* `L`: 処理するeffect
* `A`: bodyの正常終了型
* `B`: handler全体の結果型
* `H`: handler clause自身のeffect要求

return clauseは概念的に次の型を持つ。

```text
A ->{Hret} B
```

各operation clauseは、operation引数とaffine resumptionを受け取る。

```text
argument : P
resume   : Resume<R, B, E>
```

operation clauseのbodyは次の型を持つ。

```text
B ! Hop
```

全clauseのeffect要求はhandler effect `H`によって満たされなければならない。

```text
Hret ⊑ H
Hop1 ⊑ H
...
HopN ⊑ H
```

handler適用の概念的な規則は次のとおりである。

```text
h    : Handler<L, A, B, H>
body : Unit ->{<L | E>} A

E ⊑ F
H ⊑ F
────────────────────────────────
handle(h, body) : B ! F
```

ここで`F`は、bodyから残るeffect `E`とhandler clause自身のeffect `H`の
両方を満たすambient rowである。

handlerはbodyの`L`を一層だけ処理する。

***

#### `DD-EFF-017`: 利用者向けmaskの禁止

`確定`

RPX v1は、一般利用者向けの任意のeffect masking機能を提供しない。

利用者コードは、動的scopeに設置された任意のhandlerを一時的に無効化したり、
指定したmatching handlerを飛び越えたりしてはならない。

次のような一般的機能は提供しない。

```lisp
(mask Log
  body)
```

```lisp
(skip-handler audit-handler
  body)
```

理由は次のとおりである。

* sandboxまたは監査handlerの回避を防ぐ
* module sealingとprivate不変条件を保護する
* handlerの送り先を型およびscopeから予測可能にする
* resource、transaction、SourceEdit等の安全境界を維持する
* named instanceと重複する低水準機能を避ける

operation clauseおよびreturn clauseの実行中に現在のhandler frameを探索対象から
除外する動作は、handler意味論の内部規則であり、first-classなmask operationとして
公開しない。

特定のhandler instanceを使用する必要がある場合はnamed/scoped effect instanceを使う。

現在処理中のoperationを外側へ委譲する場合は、operation clause内だけで使用できる
`forward`を使う。

将来、runtime、schedulerまたはforeign interfaceの実装にmask相当の操作が必要になった
場合は、通常のsafe RPXから分離されたtrusted internal primitiveとして検討する。

***

#### `DD-EFF-018`: residual effectと実行境界

`確定`

通常の関数および公開パッケージAPIは、残余effectを型に持つことができる。

```text
load-project :
Path ->{Resource, Failure} Project
```

このeffect rowは未処理runtime errorではなく、呼出側への型付き要求である。

private関数のeffectは、次のいずれかでなければならない。

1. パッケージ内部のhandlerで処理する。
2. 呼出側の関数型へ正確に伝播する。
3. effectを禁止する境界で静的に拒否する。

package初期化は原則としてempty effect rowを要求する。

```text
package initialization : <>
```

外部操作が必要なpackageは、import時に処理を実行せず、effectfulな公開関数として
提供する。

testは、許可されたtest handlerを適用した後にempty effect rowになることを要求する。

実行可能entry pointでは、次のいずれかを要求する。

```text
residual effects = <>
```

または：

```text
residual effects
⊆ 実行hostが明示的に提供するeffect
```

host handlerは実行前のlink段階で接続し、entry pointのeffect要求と照合する。

```text
typed entry point
+
typed host handlers
+
runtime/backend configuration
→ closed executable computation
```

必要なhost handlerが不足する場合、operationを実行するまで待たず、link errorとして
拒否する。

***

#### `DD-EFF-019`: 未処理effectの動的分類

`確定`

well-typedかつwell-linkedなclosed computationでは、未処理effectは発生してはならない。

runtimeは防御的検査として、handler stackおよびhost boundaryのいずれにも
対応handlerが存在しない場合を検出する。

この状態を構造化されたruntime faultとして扱う。

```text
UnhandledEffect {
  effect_id,
  operation_id,
  effect_name,
  operation_name,
  source_span,
  module_path,
  operation_origin,
  active_handler_stack,
  expected_host_capability,
  runtime_version
}
```

`UnhandledEffect`は通常のRPX effectではない。通常のRPX handlerからcatchできない
runtime faultとする。

一般的な`UnhandledEffect` handlerを言語内に提供してはならない。これを許すと、
operationごとに異なる戻り値型を無視してeffect systemを迂回できるためである。

runtimeは未処理effectを次として扱ってはならない。

* Rust panic
* undefined behavior
* 説明のないstuck状態
* 不正な任意値の返却

CLIでは構造化診断を表示し、非zero statusで終了する。

GUIでは失敗した評価runを破棄し、直前のlast-good-renderを保持する。
失敗runの途中RenderIR、SourceEdit、resource transaction等をcommitしてはならない。

外部処理の通常の失敗と`UnhandledEffect`を区別する。

```text
handlerが存在しない:
UnhandledEffect runtime fault

handlerは存在するが外部操作が失敗:
ResourceError等の通常のtyped failure
```

`UnhandledEffect`は、次のような型安全境界の破損を検出するための安全網である。

* foreign primitiveの誤実装
* runtimeまたはcompilerの欠陥
* ABI不整合
* 不正なserialized Core
* 壊れたcache
* unsafe plugin
* handler link構成の欠陥

目標性質は次のとおりである。

```text
well-typed
+
well-linked
+
valid foreign boundary
+
valid runtime configuration
────────────────────────────
UnhandledEffectへ到達しない
```

***

#### `DD-EFF-020`: cleanupとの接続要件

`確定`

resumptionをresumeせずに破棄する場合、捕捉された計算を単に破棄してはならない。

捕捉された計算の動的scope内にcleanup frameが存在する場合、runtimeは継続を
unwindし、必要なcleanupを実行しなければならない。

ただし、汎用cleanup／finalizationの完全な意味論は`OPEN-EFF-001`の中心仕様から
分離する。

`OPEN-EFF-001`では次だけを要求する。

```text
1. resumeされた継続:
   cleanupをまだ実行せず、計算へ復帰する。

2. 破棄された継続:
   復帰不能になったscopeをunwindし、cleanup frameを処理する。

3. 外側へ一時的に伝播したoperation:
   外側のhandlerがresumeする可能性がある間はcleanupしない。

4. forwardされた継続:
   cleanupの所有責任も外側へ移譲する。

5. cleanupを無視したままcontinuation segmentを破棄してはならない。
```

汎用`ensure(body, cleanup)`の詳細は、`OPEN-MEM-001`および`OPEN-ERR-001`へ移管する。

移管する事項は次のとおりである。

* cleanupの完全なCore構文と型規則
* cleanupのexactly-once保証
* cleanup自身が発生させるeffect
* bodyとcleanupが共に失敗した場合のエラー合成
* resource exhaustion
* process

### 13.9 `MOD-001` ML-style module system

#### 概要・状態

静的module、structural signature、transparent ascription、opaque sealing、functorを
採用することは`確定`。細部は`暫定`。

moduleはruntime recordではない。抽象型、compile-time syntax、分割コンパイル、
generativityを持つ。

#### 構文

```lisp
(signature Ordered
  (type T)
  (val compare (fn (T T) Ordering)))
```

```lisp
(module NumberOrder
  (seal Ordered)
  (type-alias T Number)
  (val compare (fn (x y) ...)))
```

- `(as Sig)`: transparent ascription。
- `(seal Sig)`: opaque ascription。
- signatureがexport listを兼ね、追加実装memberはprivate。

Functor:

```lisp
(functor MakeSet
  ((Order Ordered))
  (seal Set)
  ...)

(module NumberSet
  (apply MakeSet NumberOrder))
```

applicativeを既定、`fresh`によるgenerative applicationを明示する案は`暫定`。

#### 静的意味

- signature matchingはstructural。
- sealed abstract typeはpath-dependent nominal identityを得る。
- `where type`/sharing-type constraintを持つ。
- module dependencyは原則DAG。
- recursive moduleは将来明示機能、現在`未決定`。
- top-level module value initializationをempty effect rowに制限する案は`暫定`。
  一般再帰を許す言語でtotalityを要求するか、その停止性を検査する方法は`未決定`。
- Value、Type、Module、Syntax namespaceを分ける案は`暫定`。

module signatureにはvalue、type、type alias、row、effect、effect alias、submodule、
syntax exportを記述可能とする案が`暫定`。

#### 動的意味

moduleのtype identity、signature matching、functor identityは静的elaboration対象である。
一方、moduleのvalue member初期化はCoreの`let`/record等へelaborateしてruntime評価するか、
すべてsyntactic valueへ制限する必要がある。この選択は`未決定`であり、単に
「moduleはruntime項ではない」としてvalue初期化を消去してはならない。
first-class moduleは将来`pack/unpack`によるexistentialとして明示する案が`暫定`。

#### 反例

`CE-MOD-001`: dynamic castやprovenanceからsealed representationを観測できると抽象化が
破れる。

`CE-MOD-002`: applicative functorへ非stable expressionを入力した場合のtype identityが
不明。入力をstable module pathに制限するかgenerativeにする必要がある。

#### テスト

- `TEST-MOD-001`: structural matching。
- `TEST-MOD-002`: transparent/opaque type equality。
- `TEST-MOD-003`: private access rejection。
- `TEST-MOD-004`: applicative identity。
- `TEST-MOD-005`: explicit generativity。
- `TEST-MOD-006`: sharing constraint。
- `TEST-MOD-007`: macro/dynamic/provenance sealing attacks。
- `TEST-MOD-008`: separate compilation identity。
- `TEST-MOD-009`: dependency cycle rejection。

### 13.10 `PKG-001` Packages and domain libraries

#### 概要・状態

packageを配布・version・dependency単位、moduleを抽象化単位とすることは`確定`。
物理manifest、registry、version selectionは`未決定`。

想定package:

- Japanese typesetting
- document/TeX/SATySFi style layout
- vector design
- slides
- animation/timeline
- chart/diagram
- backend adapters

暫定manifest例:

```lisp
(package japanese-layout
  :version "1.0.0"
  :root JapaneseLayout
  :dependencies {core: "^1.0"})
```

package外へはroot moduleから到達するpublic memberだけを公開する。lockfile、
content hash、resource manifestを用いて再現性を確保する案は`暫定`。

実行時I/Oをimport時に行わず、resource effectを明示する。

#### テスト

- `TEST-PKG-001`: dependency DAG/version lock再現性。
- `TEST-PKG-002`: private module非到達。
- `TEST-PKG-003`: compile-time/runtime dependency分離。
- `TEST-PKG-004`: package更新時のsignature compatibility。
- `TEST-PKG-005`: hermetic package build。

### 13.10.1 `KER-001` Rust kernelとforeign primitive境界

#### 概要・状態

Rustカーネルを最小にすることは`確定`。正確なABI、FFI、trusted primitive一覧は
`未決定`。カーネル候補は次に限定する。

- parser/type/effect/module runtimeに不可欠な操作
- Path rasterization、font shaping、image decode等の低水準・高性能処理
- resource/backend host adapter
- provenance/source transactionのtrusted metadata

circle、rect、組版規則、slide、timeline等のドメイン語彙はRPX packageへ置く。
Rust実装で高速化する場合も、型付きRPX関数と同じ観測意味を持つintrinsicとして公開する。

primitive signatureは通常の型/effect signatureで記述し、panic、unsafe pointer、
untracked I/OをRPXへ漏らさない。FFIが型を偽るとCore型安全性は成立しないため、
kernel validatorとunsafe監査はtrusted computing baseである。

動的failureは`ERR-001`の明示結果へ変換する。memory layout、calling convention、
plugin ABI、custom shaderは`未決定`。

- `TEST-KER-001`: RPX reference implementationとRust intrinsicの差分。
- `TEST-KER-002`: malformed foreign resultのvalidator。
- `TEST-KER-003`: panic/unsafe boundary fuzz。
- `TEST-KER-004`: primitive effect declarationと実動作の一致。

### 13.10.2 `RSC-001` Resource、I/O、host-handler境界

#### 概要・状態

font、image、file、network、random、clock等の外部世界をeffect/handlerへ置く方針は
`確定`。operation taxonomyとerror typeは`未決定`。

```text
resolve-font : FontQuery ->{Resource} Font
load-image   : ResourceId ->{Resource} Image
write-output : Path × Bytes ->{FileWrite} Unit
```

RenderIRは未解決pathを持たず、解決済みcontent-addressed resource IDだけを参照する。
本番handlerはfilesystem/network等を使用し、test handlerはmemory mapと固定seedを使う。

resource operationの順序はCoreの左から右評価に従う。cacheは観測結果、diagnostic、
provenanceを変えてはならない。network nondeterminism、timeout、cancellation、credential、
sandboxは`未決定`。

Progressではhost operationを「適切なhost handler下でstep可能なsuspension」と扱う候補。
外部失敗はpanic/stuckでなくtyped error resultまたはerror effectにする必要がある。

- `TEST-RSC-001`: memory/real handler API conformance。
- `TEST-RSC-002`: missing/corrupt resource。
- `TEST-RSC-003`: content hash/replay。
- `TEST-RSC-004`: concurrent cache determinism（並行性採用時）。
- `TEST-RSC-005`: unit testに未処理FileWriteが残る場合の拒否。

### 13.11 `EDT-001` Provenance and Editable

#### 概要・状態

`editable`を表面上の関数として使い、replace/offset/transform/lensでsource patchを
生成する方針は`確定`。通常の純粋関数だけではsource originを得られないため、
trusted compiler/runtime metadataのいずれかが必要である。どの層に置くかは`未決定`。

#### 構文

```lisp
(val radius
  (editable :id 'logo-radius 40mm))
```

更新戦略:

```lisp
(editable value :strategy replace)
(editable expression :strategy offset)
(editable expression :strategy transform)
(editable expression :lens custom-lens)
(editable expression :strategy readonly)
```

offsetは初回編集時に、意味のある構造へ変換し得る。

```lisp
(adjust
  :base (layout-position item)
  :offset (editable 12mm))
```

#### 意味

概念型:

```text
Editable A = {
  get: SourceSnapshot -> Result<A, EditError>,
  put: (SourceSnapshot, A) -> Result<PatchSet, EditError>,
  origin: SourceOrigin,
  id: EditableId
}
```

目標lens law:

```text
apply(s, put(s, get(s))) ≈ s
get(apply(s, put(s, a))) = a
apply(apply(s, put(s, a)), put(apply(s, put(s, a)), b))
  ≈ apply(s, put(s, b))
```

各式は`get`/`put`/`apply`が成功した場合だけ要求する部分lens lawである。
snapshotにはsource version、評価params/resources、macro transformer versionを含める
必要がある。format差、macro originを含むため`≈`の定義は`未決定`。

既定規則:

1. direct literalは置換。
2. 明示editable bindingは初期値を更新。
3. 一意かつ安全に可逆な式だけ逆伝播。
4. 曖昧なら勝手に変更せず候補を提示。
5. 選択したoffset/transform/lensをsourceへ記録。
6. macro-generated codeは、一意で権限のあるoriginと明示lensがある場合だけ展開元へ
   逆伝播する。それ以外は候補提示またはread-only失敗とし、生成物を直接編集しない。
7. imported sealed/private sourceは編集しない。

#### Effect、error、反例

source mutationは通常stateでなく`SourceEdit` effect/transactionとする案が`暫定`。

- stale origin
- overlapping patch
- deleted source
- ambiguous multi-origin
- read-only package

は明示的な編集失敗。分類は`ERR-001`で未決定。

`CE-EDT-001`: `x = left + width`をdragしたとき更新先は一意でない。
`CE-EDT-002`: macro expansion後spanを直接書換えると利用者sourceを破壊する。
`CE-EDT-003`: multi-shot resumeによるpatch重複。

#### メタ理論・実装

Coreの通常Preservationは外部source書換えを覆わない。versioned source snapshot間の
transaction safety、再parse・再typecheck成功を別性質として定義する必要がある。

必要構造: StableSyntaxId、Origin DAG、PropertyPath、EditableId、SourceVersion、
PatchSet、conflict set、module authority、lens registry。

#### テスト

- `TEST-EDT-001`: replace/offset/transform。
- `TEST-EDT-002`: lens laws property test。
- `TEST-EDT-003`: stale/overlap conflict。
- `TEST-EDT-004`: macro origin。
- `TEST-EDT-005`: formatting後identity。
- `TEST-EDT-006`: sealed package非編集。
- `TEST-EDT-007`: GUI edit→parse→typecheck→render round-trip。
- `TEST-EDT-008`: ambiguous update UI contract。

### 13.12 `IR-001` Layered visual/motion/render IR

#### 概要・状態

単一万能IRを採用しないことは`確定`。以下の層名、schema、座標系、色・filter詳細は
最後の提案に対する明示承認がないため`暫定`とする。

```text
Domain IR
→ DocumentIR / SlideIR / SceneIR / MotionIR
→ VisualIR
→ RenderIR
→ target BackendIR
```

#### SurfaceとArtifact

```text
Artifact =
  StaticDocument { surfaces: Vector Surface }
| MotionDocument { surface, timeline, audio }

Surface = {
  id, kind, extent, view_box, background, root, metadata
}
```

各page/slide/artboardが個別size/metadataを持つ。

#### RenderIR node algebra

```text
NodePayload =
  Empty
| Group { children: Vector<Node>, isolated: Bool, knockout: Bool }
| Path {
    contours: Vector<Contour>,
    fill: Option<Fill>,
    stroke: Option<Stroke>
  }
| GlyphRun { font, glyphs, unicode, clusters, paint, direction, language }
| Image { resource, source_rect, destination_rect, sampling }
| Clip { paths: Vector<ClipPath>, child: Node }
| Mask { mask: Node, mode: Alpha | Luminance, child: Node }
| Filter { graph: FilterGraph, region: Rect, child: Node }
| Instance { definition: DefId }

Contour = {
  start: Point,
  segments: Vector<LineTo | CubicTo>,
  closed: Bool
}
```

共通property:

```text
Node = {
  id, transform, opacity, blend, visible, payload
}
```

`Contour.start`がMoveTo、`closed`がCloseを表す。最終segmentはLineToとCubicTo。
circle/rect/arc/quadはlowerする。fill rule、stroke width/cap/join/dash、paintは
Path payloadの型付きfieldに置く。Group/Clip/Mask/Filterのtree edgeとInstanceの
definition-reference edgeを区別し、definition-reference graphをacyclicにする。
PaintはSolid、Linear/Radial/Mesh gradient、Pattern。
ColorはsRGBだけでなくlinear sRGB、Display P3、Gray、CMYK、Lab、ICC、Spotを
表せる案。正確な色変換規範は`OPEN-IR-001`。

TextはVisualIRでUnicode/styleを持ち、RenderIRではfont resource、glyph id、position、
cluster mapping、元Unicodeを持つGlyphRunへshapeする。

Resourceはpathでなくcontent-addressed IDを参照する。

#### MotionIR

```text
Track<A> =
  Constant A
| Keyframes (Vector (Keyframe A))
| Samples { rate, values }
```

RPX意味上のAnimationは`Time -> Picture`。認識可能なsignalだけをexactなtrackへ変換する。
任意関数は、明示された有限時間区間とtarget frame timesでsampleする。keyframe fittingは
sample rateと許容誤差を伴う近似であり、一般には意味保存変換ではない。strict modeでは
近似を拒否できる必要がある。

#### Backend lowering

backendはcapability setを宣言し、次の順でlowerする。

1. そのまま保持
2. Path等へ等価展開
3. 複数primitiveへ分解
4. subtreeを局所rasterize
5. warning付き近似
6. strict mode error

PPTX/AEの編集可能性を保つ場合は上位VisualIR/MotionIRから直接変換する。

#### 不変条件

- finite numberのみ。NaN/Inf禁止。
- definition graphはacyclic。
- resource IDは解決済み。
- Group child orderがz-order。
- RenderIRはI/O、未解決type、macro、Editable実行を含まない。
- provenance/semanticsはside table。
- 2D、左上origin、x右/y下、`parent × local` transform案は`暫定`。

#### メタ理論・反例

型Preservationだけではpixel等価を示せない。各loweringにobservational equivalenceまたは
許容誤差仕様が必要。

`CE-IR-001`: group opacityをchildへ分配するとblend結果が変わる。
`CE-IR-002`: glyphをpath化すると検索・accessibilityを失う。
`CE-IR-003`:単一RenderIRからPPTX/AEへ出すと編集意味を失う。

#### テスト

- `TEST-IR-001`: validator fuzz（NaN、cycle、missing resource）。
- `TEST-IR-002`: transform composition metamorphic。
- `TEST-IR-003`: group/blend/opacity golden。
- `TEST-IR-004`: text cluster/search round-trip。
- `TEST-IR-005`: color/profile conformance。
- `TEST-IR-006`: backend capability fallback。
- `TEST-IR-007`: animation sample determinism。
- `TEST-IR-008`: optimization前後のrender equivalence。
- `TEST-IR-009`: provenance side table保持。

### 13.13 `ERR-001` Errors and terminal states

#### 状態

分類の必要性は`確定`、各言語機構への割当は多くが`未決定`。

| 分類 | 状態・意味 |
|---|---|
| 正常終了 | 値またはartifactを返す |
| 発散 | 許容。停止性を保証しない |
| 明示的例外 | effectとして実装する案が`暫定` |
| 回復可能runtime error | resource failure、dynamic cast等。正確な型は未決定 |
| panic | runtime invariant破壊。利用者入力で起こしてはならない |
| stuck | well-typed closed handled Coreでは許さない設計目標 |
| undefined behavior | safe RPXには設けない設計目標（`暫定`） |
| implementation-dependent | resource limit、浮動小数の一部、backend差。範囲未決定 |
| compiler/runtime crash | 仕様上許容結果ではなくimplementation defect |
| resource exhaustion | host failure。catch可能性は未決定 |
| external failure | Resource/IO effectのerror resultまたはexception effect |

zero division、array bounds、non-exhaustive match、invalid cast、device lossの具体分類は
`OPEN-ERR-001`。

#### 診断要件

すべての静的診断はprimary span、期待/実際、binding origin、macro expansion stack、
module path、型/effect/row constraint traceを必要に応じて持つ。複数エラー回復の保証は
`未決定`。

#### テスト

- `TEST-ERR-001`: 各終端分類の識別。
- `TEST-ERR-002`: user inputでpanicしないfuzz。
- `TEST-ERR-003`: macro/type/module origin診断。
- `TEST-ERR-004`: resource failureをhandlerで回復。

### 13.14 `MEM-001` Memory and resource model

#### 状態

`情報不足`。Rustホストがmemory safeであることだけではRPX言語のメモリ意味論は決まらない。

未決定:

- GC、reference counting、arena、hybrid
- closure/continuation lifetime
- weak reference/finalizer
- first-class `ref`
- resource handleのclose/dispose
- cyclic value
- foreign resourceとdevice loss
- memory limit

`var`は局所state handlerとして意味づけるため、物理allocation方式を観測不能にする目標。
resource lifetimeはeffect handler/lexical scopeで管理する案が`推定`。

Progress/Preservationにはtyped store `Σ`が必要。memory safety、race freedom、
deadlock freedomは現時点で`仕様不足により判定不能`。

テスト候補:

- `TEST-MEM-001`: closure/continuation retention stress。
- `TEST-MEM-002`: resource scope cleanup。
- `TEST-MEM-003`: cyclic structure。
- `TEST-MEM-004`: allocation failure分類。

### 13.14.1 `ASY-001` Concurrency and async

#### 状態

`未決定`。現時点の言語設計はthread、task、async/await、channel、schedulerを規定しない。
Koka型handlerからasyncをlibraryとして構築できる可能性はあるが、採用済みではない。

未決定事項:

- deterministic schedulerの有無
- continuationをthread間でresumeできるか
- `var`、`ref`、Editable、resource cacheの共有
- cancellationとstructured concurrency
- race/deadlockを型で排除するか
- frame renderingの並列化が観測可能か

したがってrace freedom、deadlock freedomは`仕様不足により判定不能`。実装が内部で
並列renderしても、仕様上の評価順序とartifact決定性を保存しなければならない。

- `TEST-ASY-001`: 将来のscheduler replay。
- `TEST-ASY-002`: cancellation時resource cleanup。
- `TEST-ASY-003`: SourceEditの直列化。
- `TEST-ASY-004`: parallel renderとsequential renderの同値。

### 13.15 `TST-001` Tests and conformance

#### 概要・状態

テストを製品の一部として重視し、validity/defect、black-box/white-box、unit/component/
integrationを併用することは`確定`。言語内表面構文は`未決定`。

暫定例:

```lisp
(test "circle area"
  (assert-equal
    (area (circle 10mm))
    expected))
```

handler付きhermetic test:

```lisp
(test "deterministic random"
  (with test-random
    (assert-equal (jitter 100) 125)))
```

test bodyから許可されない未処理effectが残れば静的拒否する。許可rowの指定方法は
`未決定`。

#### Black/white box

- 別test moduleはpublic signatureだけを見る。
- `test-module`/`test-of` companionは同一package内private memberを見られる。
- white-box testはprivateを再exportできず、製品artifactへ含めない。

#### 既存実装

現行Rust testsはvalidity/defect区分、CST round-trip、macro、固定type checker、
effect skeleton、lowering、GUI sync、PDF/SVG/PPTXを経験的に検査する。ただし最終言語の
型安全性や意味論を検証したものではない。

#### テスト原則

各conformance caseは可能な限り以下を記録する。

```text
source bytes
CST
reader/macro expansion
resolved Core
inferred type/effect
evaluation trace
provenance graph
Visual/Motion/Render IR
backend result
```

#### メタ理論

テストは証明の代替ではない。typed random reductionで反例が出ないことをPreservationの
証明と呼ばない。反対に、反例が一つあれば仕様または実装の不健全性を示し得る。

## 14. 統合された構文

### 14.1 全体EBNF（未完成）

以下は決定済み概念を接続する骨格であり、完全な規範文法ではない。

```ebnf
source        = trivia*, top-form*, trivia* ;
top-form      = form ;
form          = literal
              | identifier
              | list
              | reader-form ;
list          = "(", trivia*, { form, trivia* }, ")" ;

reader-form   = doc-form ;
doc-form      = "(", "doc", { doc-item }, ")" ;
doc-item      = text-chunk | at-expression ;

special-form  = fn-form | val-form | var-form | let-form | letrec-form
              | if-form | match-form
              | eff-form | handler-form | handle-form | with-form
              | type-form | alias-form
              | module-form | signature-form | functor-form
              | syntax-form ;

fn-form       = "(", "fn", "(", { identifier }, ")", { form }, ")" ;
val-form      = "(", "val", identifier, form, ")" ;
var-form      = "(", "var", identifier, form, ")" ;
let-form      = "(", "let", "(", { binding }, ")", { form }, ")" ;
binding       = "(", identifier, form, ")" ;
if-form       = "(", "if", form, form, form, ")" ;
```

`special-form`は具象parserの`form`から直接到達する別delimiter構文ではない。すべて
`list`としてparseした後、phase-aware binding resolutionによりlist headがCore special
form、macro、通常applicationのどれかに識別される。このため上の`special-form`生成規則は
list内容のshapeを説明する補助文法である。

未確定穴:

- literal/token grammar
- pattern/match grammar
- exact type grammar tokenization
- macro transformer grammar
- module refinement/sharing syntax
- doc escapeの完全形
- quote/quasiquote
- record literal/update/remove
- sequence/body empty/nonempty rule

### 14.2 優先順位・結合

S式にはinfix優先順位・結合規則を設けない。`/`、`+`等もlist headのidentifierである。
unit suffixを数値tokenへ含める場合の結合は`OPEN-SYN-002`。

### 14.3 予約語

readerが認識する`doc`等を除き、特殊形式名をlexically reservedにするか、
binding identityで認識するかは`未決定`。hygienic macroとmodule shadowingのため、
単なる文字列比較にしないことを推奨するが未確定。

### 14.4 糖衣とCore

| Surface | Core候補 |
|---|---|
| `(val (f x) body)` | `(val f (fn (x) body))` |
| `let` | lambda applicationまたはCore let |
| `var` | fresh state effect + scoped handler |
| `with h body` | `(handle h (fn () body))` |
| doc reader | syntax object constructors/function calls |
| pattern function clauses | `match` |
| `test` | test registryへのtyped value（詳細未決定） |

## 15. 統合された静的意味論

### 15.1 Kind

```text
K ::= Type | RecordRow | Effect | EffectRow | Module | Signature | Syntax
```

`Effect`を独立kindにするか、effect labelを別categoryにするかは`暫定`。

### 15.2 共通判断

```text
Δ; M; Γ ⊢ e ⇒ T ! E
Δ; M; Γ ⊢ e ⇐ T ! E
Δ ⊢ T : Type
Δ ⊢ R : RecordRow
Δ ⊢ E : EffectRow
Δ ⊢ T <: U
M ⊢ ModuleImpl : Signature
```

### 15.3 基本規則

`T-VAR-001`

```text
x : ∀ᾱ.T ∈ Γ     U = instantiate(∀ᾱ.T)
──────────────────────────────────────────
Δ;M;Γ ⊢ x ⇒ U ! <>
```

`T-SUB-001`

```text
Γ ⊢ e ⇒ T ! E     T <: U
──────────────────────────
Γ ⊢ e ⇐ U ! E
```

`T-IF-001`

```text
Γ ⊢ c ⇐ Bool ! Ec
Γc,true ⊢ t ⇒ T ! Et
Γc,false ⊢ f ⇒ U ! Ef
────────────────────────────────────────
Γ ⊢ (if c t f) ⇒ union(T,U) ! join(Ec,Et,Ef)
```

`Γc,*`はoccurrence typingによるrefinement。predicate grammarは未決定。

`T-RECORD-001`

```text
Γ ⊢ ei ⇒ Ti ! Ei  (source order, distinct labels)
──────────────────────────────────────────────
Γ ⊢ {li=ei} ⇒ {li:Ti} ! join(Ei)
```

`T-PERFORM-001`

```text
op : A ->{L} B     Γ ⊢ e ⇐ A ! E
────────────────────────────────
Γ ⊢ (op e) ⇒ B ! extend(L,E)
```

`extend`はeffect labelをambient rowへ要求する制約を表す略記であり、operationの
動的実行回数をmultisetへ加算するものではない。同じoperationの逐次二回実行は、通常、
一つのambient label要求で表せる。duplicate labelはnested handler、masking、
row-polymorphic unificationのために別途現れ得る。正確なKoka型row unificationと
eliminationは`OPEN-EFF-001`。

handler ruleはdeep/shallow等が未決定のため規範化できない。

### 15.4 一般化

厳格な構文的value restrictionを`確定`とする。generalization pointは`val`、`let`および`letrec`検査後である。
module signature境界では明示quantifierを保持する。`var`、effectful computation、
resumptionを含む項は原則generalizeしない。

### 15.5 Subtypingと制約解決

- semantic subtypingを宣言的関係とする。
- algorithmはemptiness/normalization/tallyingへ帰着する。
- RPX全体のalgorithmic completenessは保証しない。A-fragmentに限って完全判定を目標とする。
- algorithmが保守的に拒否する場合、診断で「仕様上不正」と「checker限界」を区別する
  必要がある。

### 15.6 Module境界

opaque sealing後のabstract typeは、元representationとのsubtyping/型等価を外部で利用
できない。dynamic cast、pattern、serialization、provenanceにも同じ制約が必要。

## 16. 統合された動的意味論

### 16.1 構成

```text
Configuration = ⟨e, ρ, σ, H, κ⟩
```

- `ρ`: lexical environment
- `σ`: local state/effect instance store
- `H`: handler stack
- `κ`: continuation/evaluation context

実装がCEK/CEKS machineを使うか、直接interpreterを使うかは実装依存にできるが、
観測可能な評価順序を変えてはならない。

### 16.2 評価順序

`暫定`

1. operator
2. arguments left-to-right
3. record field expressions source order
4. selected `if`/`match` branchのみ
5. handler expressionを先に評価し、その後body

### 16.3 効果伝播

operationはnearest matching handlerまでcontinuationをcaptureする。unmatched operationは
外側へ伝播する。host boundaryで未処理の場合の分類は`ERR-001`で未決定。

### 16.4 SourceEdit

GUI editは通常評価と別transactionである。暫定処理:

```text
snapshot
→ Editable.put
→ PatchSet validation
→ CST patch
→ parse/expand/typecheck
→ 成功ならcommit、失敗ならrollback/diagnostic
```

preview last-good-renderはcommit失敗時に保持する。

### 16.5 観測可能な振る舞い

最低限:

- return value
- handled external output
- generated artifact
- source transaction result
- deterministic diagnostic

allocation address、module representation、private provenance、内部optimizationは観測不能と
する目標。

## 17. エラーと停止状態の分類

| 状態 | Progress上の扱い | 許容 | 例 |
|---|---|---|---|
| value/artifact | 正常終端 | はい | RenderPackage |
| divergence | 無限step | はい | 一般再帰 |
| handled operation suspension | step可能 | はい | Resource request |
| explicit language error | 明示終端またはeffect | 未決定 | MatchError |
| dynamic blame | 明示終端候補 | 暫定 | invalid cast |
| external failure | handlerへ値/effect | はい | missing font |
| resource exhaustion | host failure | 暫定 | OOM |
| panic/crash | 仕様結果でない | いいえ | compiler bug |
| stuck | 型安全fragmentで禁止目標 | いいえ | non-function apply |
| undefined behavior | 安全RPXでは禁止目標 | いいえ | 該当なしを目指す |
| implementation-dependent | 明示範囲だけ | 未決定 | float/backend差 |

配列境界、zero division、null相当、pattern failureは型/effect設計が未決定。
データ競合・deadlockは並行性仕様がないため`情報不足`。

## 18. 機能間相互作用

| 組合せ | 分類 | 必要事項 |
|---|---|---|
| set types × record rows | 制約付きで両立可能 | semantic record model、Absent、tallying |
| set types × effects | 仕様追加が必要 | effectful arrowのsubtypingとnegation |
| dynamic × abstract types | 反例候補あり | castがrepresentationを漏らさない |
| rank-1 polymorphism × var | 制約付きで両立可能 | value restriction |
| handlers × var | 仕様追加が必要 | multi-shot state semantics |
| duplicate effect rows × nested handler | 仕様追加が必要 | multiplicity-aware subtraction |
| macro × module sealing | 反例候補あり | phase authority/certificate |
| macro × provenance | 仕様追加が必要 | origin chain、generated edit policy |
| Editable × sealing | 制約付きで両立可能 | owner authority、private origin遮蔽 |
| Editable × multi-shot effect | 反例候補あり | one-shot/transaction idempotence |
| Visual optimization × provenance | 仕様追加が必要 | side table remapping |
| doc reader × macro | 仕様追加が必要 | reader/expander phase order |
| module initialization × I/O | 両立困難 | top-level effect禁止を推奨 |
| MotionIR × arbitrary function | 制約付きで両立可能 | sampling/baking fallback |
| backend fallback × visual equality | 仕様追加が必要 | tolerance/capability policy |

## 19. メタ理論上の性質

| 性質 | 現状 | 注記 |
|---|---|---|
| Progress | `仕様不足により判定不能` | Core、handler、dynamic error未確定 |
| Preservation | `仕様不足により判定不能` | store/effect/cast規則未確定 |
| 型安全性 | `未検討` | Progress+Preservationの形式証明なし |
| 決定性 | `証明スケッチのみ`にも未到達 | pure Coreは目標。外部effectはhandler依存 |
| 型検査の決定可能性 | `反例候補あり` | 推論範囲を制限する必要 |
| checker soundness | `未検討` | 最終checker未実装 |
| checker completeness | `意図的に保証しない`可能性 | 正式決定は未決定 |
| principal type | `未決定` | set types/subtypingで一般に困難 |
| 正規化 | `意図的に保証しない` | 一般再帰・発散を許す |
| 合流性 | `該当範囲未決定` | deterministic CBVならreduction path固定 |
| pattern網羅性 | `未決定` | semantic subtyping emptinessを利用可能 |
| parametricity | `反例候補あり` | dynamic/effects/typecaseで制約 |
| effect safety | `仕様不足により判定不能` | empty row/host boundaryを要定義 |
| memory safety | `仕様不足により判定不能` | safe Rustは言語証明ではない |
| race freedom | `未検討` | concurrency未仕様 |
| deadlock freedom | `未検討` | concurrency未仕様 |
| module abstraction | `証明スケッチのみ`にも未到達 | sealing攻撃面を列挙した段階 |
| lowering correctness | `テストによる経験的確認のみ`（現実装） | 最終IRの同値定義なし |

目標Progressの前提:

1. closed Core term
2. well-kinded、well-typed
3. empty residual effect row、または全operationにhost handler
4. valid store
5. dynamic cast failure等を明示終端に含める

目標Preservationに必要な補題:

- weakening/exchange
- value substitution
- type/row/effect substitution
- store extension
- evaluation context replacement
- handler/resumption substitution
- cast evidence composition
- module elaboration/sealing correctness

現時点の検討では、限定されたpure STLC相当部分以外について明確な安全性証明はない。
反例が列挙されていない部分も、形式的または機械的に証明されたわけではない。

## 20. 実装アーキテクチャ

### 20.1 最終目標パイプライン

```text
bytes
→ tokens
→ lossless CST
→ reader expansion
→ phase-aware macro expansion + compile-time binding/module resolution
→ runtime binding resolution済み Core AST
→ type/effect/module checking
→ typed Core IR
→ evaluation
→ Domain IR
→ Visual/Motion IR
→ resolved RenderIR
→ BackendIR/artifact
```

### 20.2 必要データ構造

| 構造 | 目的・不変条件 | 生成→利用 |
|---|---|---|
| Token | kind/span/raw text、重複/欠落なし | lexer→parser |
| Lossless CST | 全token/triviaを順序保持 | parser→editor/reader |
| SyntaxObject | scopes、phase、origin | reader/macro→resolver |
| Resolved AST | identifierがbinding IDを持つ | resolver→type checker |
| Type DAG | kind付き、hash-cons、recursive cycle安全 | checker |
| ConstraintSet | type/record/effect kindを混同しない | inference→solver |
| Typed Core | 各nodeにtype/effect/source | checker→evaluator |
| ModuleEnv | signature、visibility、abstract identity | resolver/checker/loader |
| Runtime Value | closure、record、handler等 | evaluator |
| Store/HandlerStack | typed location/effect frame | evaluator |
| Provenance DAG | source→value→IR property | all stages→GUI |
| Domain/Visual/Motion/Render IR | 各層不変条件 | evaluator→backend |
| Diagnostic | primary/related spans、trace | all failure paths |

### 20.3 現行crateとの対応

| 現行crate | 現在の役割 | 最終仕様との差 |
|---|---|---|
| `reciplexa-syntax` | lexer/parser/rowan CST/edit | 最終lexer/CSTの出発点 |
| `reciplexa-macro` | text splice macro/doc layout | hygienic syntax object方式へ移行必要 |
| `reciplexa-types` | fixed builtin structural checker | 集合論的型/推論/row/module未実装 |
| `reciplexa-effect` | perform/handle skeleton | 一般評価器・effect row・continuation未実装 |
| `reciplexa-lower` | CST→scene、GUI source sync | provenance/lensと層別IRへ整理必要 |
| `reciplexa-scene` | shape enum | High-level移行IRとして利用可能 |
| `reciplexa-view` | flatten/hit test/layout | VisualIR projection候補 |
| `reciplexa-pdf/svg/pptx` | backend | capability lowering契約未実装 |
| `reciplexa` | pipeline/CLI | 最終phase orchestrator候補 |
| `reciplexa-gui` | egui editor | last-good/provenance transactionへ移行 |

現行pipelineは概ね source→expand→fixed typecheck→effects(exportのみ)→lower→backend。
これは最終評価意味論ではなく、動く縦切り実装である。

## 21. 仕様と実装の対応

| 段階 | 入力→出力 | 不変条件 | 失敗 | 必須試験 |
|---|---|---|---|---|
| lexer | bytes→tokens | spanで入力を被覆 | invalid token | LEX round-trip/fuzz |
| parser | tokens→CST | token/trivia保存 | parse diagnostics | incremental differential |
| reader | CST→syntax objects | origin/mode保存 | reader error | code/doc nesting |
| macro/phase resolver | syntax→expanded syntax | hygiene/phase/origin、compile-time binding ID | cycle/fuel/unbound | capture/alpha/module |
| runtime resolver | expanded syntax→resolved AST | runtime参照にbinding ID | unbound/visibility | shadow/module |
| checker | AST→typed Core | type/effect/kind整合 | static diagnostic | generated terms |
| desugar | surface→Core | 意味・origin保存 | invalid sugar | differential |
| evaluator | Core→values/Domain IR | CBV/effect semantics | explicit runtime result | reference traces |
| IR lower | Domain→Render layers | validator通過 | capability/resource | metamorphic/golden |
| backend | Render/upper IR→artifact | target contract | external/device | conformance |
| module loader | package→module env | lock/signature/phase | dependency/version | reproducibility |
| GUI edit | event→source transaction | authority/version/lens | conflict/rollback | round-trip |

最適化はtyped CoreまたはIRごとに、前後の観測可能意味を保存しなければならない。
デバッグ情報は全段階のorigin chainを保持する。

### 21.1 型検査器要件

- 入力: resolved AST、ModuleEnv、imported signatures。
- 出力: typed Core、inferred schemes/effects、constraints解決証跡。
- 宣言的意味とalgorithmを分離する。
- soundness、completeness、terminationを別々に評価する。
- occurs checkをtype/row/effect metavariableに行う。
- recursive typeは明示`μ`と推論cycleを区別する。
- error recoveryで不正な型を正しいものとして利用しない。
- primary errorとcascade diagnosticを区別する。
- algorithmic limitationによる拒否を可能なら明示する。

RPX全体の完全性は保証しない。A-fragmentとB-fragmentの境界は13.6.2で確定している。

## 22. テスト計画

### 22.1 構文

| ID | 対象 | 内容 |
|---|---|---|
| `TEST-SYN-C001` | LEX/SYN | valid parse/unparse byte round-trip |
| `TEST-SYN-C002` | LEX/SYN | malformed input recovery |
| `TEST-SYN-C003` | SYN | code/doc nesting and escapes |
| `TEST-SYN-C004` | SYN/MAC | reader→macro phase ordering |
| `TEST-SYN-C005` | LEX | Unicode、line ending、深いnest fuzz |
| `TEST-SYN-C006` | SYN | no infix precedence ambiguity |

### 22.2 静的意味

| ID | 対象 | 内容 |
|---|---|---|
| `TEST-STA-001` | EVAL/BND | lexical scope、shadowing、unbound |
| `TEST-STA-002` | TYP | type algebra and subtyping |
| `TEST-STA-003` | ROW | open/closed/polymorphic record |
| `TEST-STA-004` | EFF | row inference、duplicate label、handling |
| `TEST-STA-005` | MOD | signature matching/sealing/sharing |
| `TEST-STA-006` | TYP/BND | value restriction |
| `TEST-STA-007` | TYP | dynamic boundary and blame |
| `TEST-STA-008` | MAC | expanded binding identity |
| `TEST-STA-009` | TST | test effect gate |
| `TEST-STA-010` | diagnostics | error span/constraint/expansion trace |

### 22.3 動的意味

| ID | 対象 | 内容 |
|---|---|---|
| `TEST-DYN-001` | EVAL | application/record left-to-right order |
| `TEST-DYN-002` | EVAL | closure and recursion |
| `TEST-DYN-003` | EFF | nested handlers/resume/forward |
| `TEST-DYN-004` | BND/EFF | var under continuation |
| `TEST-DYN-005` | ERR | explicit error terminal classification |
| `TEST-DYN-006` | Resource | mock/real handler equivalence at API |
| `TEST-DYN-007` | determinism | fixed seed/time/resources replay |

### 22.4 統合

| ID | 内容 |
|---|---|
| `TEST-INT-001` | source→CST→Core→value→RenderIR→artifact |
| `TEST-INT-002` | multi-file module/functor/package |
| `TEST-INT-003` | black-box/white-box test visibility |
| `TEST-INT-004` | macro×module sealing |
| `TEST-INT-005` | Editable×macro×module authority |
| `TEST-INT-006` | doc package×Japanese layout×resource mock |
| `TEST-INT-007` | animation signal→MotionIR→sample/video |
| `TEST-INT-008` | per-page/per-slide size and metadata |
| `TEST-INT-009` | backend capability fallback strict/non-strict |
| `TEST-INT-010` | preview failure preserves last-good-render |

### 22.5 Property/differential/fuzz

| ID | Property |
|---|---|
| `TEST-PROP-001` | parse/unparse identity |
| `TEST-PROP-002` | incremental parse = full parse |
| `TEST-PROP-003` | well-typed generated Core step does not become unexplained stuck |
| `TEST-PROP-004` | step前後の型/effect関係（経験検査） |
| `TEST-PROP-005` | declarative/reference evaluator = optimized implementation |
| `TEST-PROP-006` | type normalization idempotence |
| `TEST-PROP-007` | subtyping finite-model differential |
| `TEST-PROP-008` | row permutation/extension laws |
| `TEST-PROP-009` | lens laws |
| `TEST-PROP-010` | IR validation survives arbitrary valid generation |
| `TEST-PROP-011` | optimization前後render equivalence within tolerance |
| `TEST-PROP-012` | interpreter/compiler observational equivalence |

Golden image/PDF testには、font bytes、color profile、backend version、resolution、toleranceを
固定する。単なる画像一致を意味論全体の証明としない。

## 23. 完成判定基準

### 段階1: 設計案が記録された

- 成果物: 本書、状態ラベル、feature/rule/open ID。
- 必須: 既決/未決/現実装を区別。
- 許容: 形式規則未完成。
- リスク: 例示構文が実質的既成事実になる。
- 次条件: 最優先OPEN項目の意思決定。

### 段階2: 仕様が明確になった

- 成果物: 完全grammar、Core calculus、type/effect/module rules、error分類。
- 必須: 全規範用語と観測可能意味。
- 許容: proof未完。
- リスク: algorithmが宣言仕様を実装できない。
- 次条件: executable reference semanticsを作れること。

### 段階3: 参照実装が動く

- 成果物: parser、expander、resolver、reference checker/evaluator、IR validator。
- 必須: 小さいconformance corpus。
- 許容: 最適化、全backend、GUI未完。
- リスク: reference implementation自体の誤り。
- 次条件: trace可能なend-to-end実行。

### 段階4: 適合試験を通過

- 成果物: normative conformance suiteとversioned期待値。
- 必須: 正常・拒否・error分類。
- 許容: 大規模性能。
- リスク: test coverage外の仕様齟齬。
- 次条件: 全MUST ruleに少なくとも正負試験。

### 段階5: 統合試験を通過

- 成果物: multi-package、GUI、resource、複数backend fixture。
- 必須: cross-feature matrixの高リスク項目。
- 許容: すべての既存製品形式との互換。
- リスク: backend差、resource環境差。
- 次条件: hermetic replay可能。

### 段階6: 差分・生成・fuzzを通過

- 成果物: generators、shrinkers、reference differential harness、crash corpus。
- 必須: parser、checker、evaluator、IR、Editable。
- 許容: 反例がないことを証明とは呼ばない。
- リスク: generator bias。
- 次条件: 既知counterexample zero、coverage基準達成。

### 段階7: メタ理論が確認された

- 成果物: 固定Coreに対するProgress/Preservation、subtyping/checker theorem、
  effect/module abstraction proofまたは機械検証。
- 必須: theorem statementと実装範囲の一致。
- 許容: backend pixel correctnessを別定理に分離。
- リスク: proof calculusと実装desugaringの乖離。
- 次条件: elaboration correspondence。

### 段階8: 実装と形式仕様の対応を確認

- 成果物: phaseごとのrefinement/correspondence、versioned spec、適合報告。
- 必須: optimizer/backendを含む観測同値または明示fallback契約。
- 許容: 実装依存範囲だけの差。
- リスク: foreign library/hardware。
- 完了: 規範仕様の保証範囲について「正しく実装」と主張可能。

## 24. 既知の問題

1. 最優先だったCore、handler、bounded dynamic、binding、algorithmic checkerの主要設計は解決されたが、形式規則と機械検証は未完成である。
2. variant／pattern／matchの完全Surface構文と網羅性policyが未固定である。
3. literal、数値字句、unit、float edge case、primitive domain errorが未固定である。
4. semantic subtyping、RecordRow、EffectRow、recursive dataの実装複雑性は高く、A-fragmentのalgorithmic completenessは未証明である。
5. shared constraint worklistの停止性、canonicalization、budget calibrationは実装・検証が必要である。
6. hygienic macroとmodule sealing／provenanceのauthorityが未設計である。
7. `var`のescape制約は確定したが、将来の`cell`／`ref`とmemory modelは未決定である。
8. cleanup、external failure、dynamic-type-error、runtime faultの最終統合分類が未決定である。
9. Editableのstale／concurrent transactionが未設計である。
10. layered IRのpixel／temporal equivalenceが未定義である。
11. memory／concurrency modelが未定義である。
12. 現行実装は最終pipelineより大幅に単純で、migrationが必要である。

## 25. 未証明の性質

以下はすべて未証明である。

- parser round-tripの全入力範囲
- reader/macro elaboration correctness
- hygiene
- Progress、Preservation、型安全性
- semantic subtyping algorithmのsoundness/completeness/termination
- gradual guarantee、blame theorem
- value restrictionの十分性
- handler/effect row safety
- module representation independence
- Editable lens/transaction law
- IR lowering/optimization correctness
- backend間の描画同値性
- memory/race/deadlock safety

## 26. 追加で決める必要がある事項

### 解決済み

- `OPEN-EVAL-001`: 最小Core、CBV、評価文脈、`let`、`seq`、`if`。
- `OPEN-EFF-001`: deep handler、one-shot resumption、forward、return、EffectRow。
- `OPEN-TYP-001`: bounded dynamic、cast evidence、dynamic failure、数値基礎。
- `OPEN-BND-001`: `letrec`、相互再帰、`var` escape、value restriction、`(type ...)`。
- `OPEN-TYP-002`: algorithmic判定、bidirectional typing、RecordRow、recursive data、solver worklist。

### 高

1. `OPEN-SYN-002`: literal、comment、escape、unit、型構文、declaration group。
2. `OPEN-DAT-001`: variant、constructor、pattern、match、guard、網羅性。
3. `OPEN-MAC-001`: phase、hygiene、capture、fuel。
4. `OPEN-EDT-001`: stable ID、source transaction、authority。
5. `OPEN-MEM-001`: GC、reference、resource、continuation lifetime。
6. `OPEN-ERR-001`: exception、cleanup、runtime terminal taxonomy。
7. `OPEN-IR-001`: color、filter、timing、backend tolerance。
8. `OPEN-MOD-001`: first-class／recursive module、separate-compilation identity。
9. `OPEN-KER-001`: foreign boundary ABIとvalidator。

### 中

10. `OPEN-SYN-001`: `doc`／`src`名称とcompatibility。
11. `OPEN-TST-001`: test syntax、allowed effect row。

### 低

12. qualified nameの表記。
13. `syntax`、`macro`等のSurface keyword。
14. diagnostic wording／style。

## 27. 次に行うべき作業

1. 解決済みCore／effect／dynamic／binding／checker判断をreference semanticsへ落とす。
2. `OPEN-SYN-002`を解決し、`(type ...)`、`forall`、kind、declaration group、literalの完全文法を固定する。
3. `OPEN-DAT-001`を解決し、regular recursive data、constructor、pattern、match、網羅性をSurfaceへ接続する。
4. shared constraint worklist、A／B／C／D fragment、三値判定を持つreference checkerを実装する。
5. deep one-shot handler、forward、return、duplicate-aware EffectRowをreference evaluatorへ追加する。
6. bounded dynamicのCast evidence、provenance、dynamic-type-errorをtyped Coreへ追加する。
7. `var`をfresh local-state instanceとhandlerへelaborateし、escape検査を実装する。
8. module calculusとmacro phaseをCore外のelaborationとして固定する。
9. Editable transactionをsource version付きの独立状態機械として定義する。
10. VisualIR／RenderIR schemaとvalidatorを独立仕様に分離する。
11. 各解決済み`OPEN-*`に対応する正負conformance testを追加する。

## 付録A: 現行実装の位置づけ

現行 reciplexa は、lossless CST、two-faced syntax、固定macro、固定builtin type checker、
限定effect runner、shape scene、GUI/CST同期、PDF/SVG/PPTX出力を持つ実用的な縦切りである。
最終言語に必要な一般 evaluator、集合論的型、row polymorphism、一般handler、
ML module、hygienic macro、layered IRは未実装である。

現行Rust testsは重要な回帰資産であるが、本書の最終言語仕様への適合試験ではない。
移行中は、現在のexampleを同じ視覚結果へlowerできるcompatibility packageを用意するのが
望ましい（`推定`）。

## 付録B: 参考設計系譜

- Glisp: 自由なLisp表現、GUIと式の連動
- SATySFi: 型付き関数型組版、code/text二面
- Elixir gradual set-theoretic types: bounded dynamic、occurrence typing
- Semantic subtyping / Polymorphic Records for Dynamic Languages
- Koka: row-polymorphic effects、handlers、局所`var`
- Standard ML / OCaml: signature、opaque sealing、functor
- Racket/Scribble: reader/macro phase、文書構文
- Functional Geometry / Haskell diagrams: 図形の純粋代数
- Fran/FRP: 時間を明示入力とするanimation

これらは設計根拠であり、RPXが各言語の仕様へ完全準拠することを意味しない。
