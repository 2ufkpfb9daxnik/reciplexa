# reciplexa / RPX 言語設計記録

> 文書状態: 設計記録（規範仕様策定前）
>
> 更新日: 2026-07-17
>
> 対象: 最終的に目指す RPX 言語、および現行 reciplexa 実装との対応

## 0. この文書の読み方

本書は、文書・組版・ベクター作図・スライド・アニメーションを統合する
reciplexa のための RPX 言語について、2026-07-17 までの設計会話を整理した
**言語設計記録**である。完成した規範仕様ではない。現在の実装を説明する箇所と、
最終目標を説明する箇所を区別する。

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

## 5. 暫定事項

- 正格 call-by-value、左から右の評価順序
- Lisp-1 の Value namespace
- `handle`を基本、`with`を糖衣構文とする
- effect operation を通常関数のように呼び、`use`を必須にしない
- ファイルの既定を code mode とし、`src` wrapper を最終的に不要にする
- `(effects ...)`による型内 effect row 構文
- rank-1、prenex polymorphism と bidirectional/local inference
- bounded dynamic を型ルートに置く Elixir 型の gradual boundary
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

## 9. 未決定事項（概要）

| ID | 優先度 | 問題 |
|---|---|---|
| `OPEN-EVAL-001` | 最優先 | Core calculus の完全な項、値、評価文脈 |
| `OPEN-EFF-001` | 最優先 | deep/shallow handler、one-shot/multi-shot resume |
| `OPEN-TYP-001` | 最優先 | bounded dynamic の evidence/cast/blame 意味論 |
| `OPEN-TYP-002` | 最優先 | semantic subtyping の実装可能な決定手続きの正確な範囲 |
| `OPEN-BND-001` | 最優先 | `letrec`初期化制約と`var` escape |
| `OPEN-SYN-001` | 高 | `src`/`doc`の最終名称と migration |
| `OPEN-SYN-002` | 高 | number、unit、string、comment の完全字句文法 |
| `OPEN-MAC-001` | 高 | マクロ phase、展開fuel、意図的capture API |
| `OPEN-MOD-001` | 高 | first-class module、recursive module の導入範囲 |
| `OPEN-EDT-001` | 高 | provenance の安定ID、競合、stale edit transaction |
| `OPEN-IR-001` | 高 | 色管理、フィルター、Backend capability の規範精度 |
| `OPEN-MEM-001` | 高 | GC、resource lifetime、continuation のメモリモデル |
| `OPEN-ERR-001` | 高 | 例外を algebraic effect として統一するか |
| `OPEN-CON-001` | 高 | 並行性、async、race/deadlock の仕様 |
| `OPEN-TST-001` | 中 | 言語内テストの最終表面構文 |

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

### 13.3 `EVAL-001` Strict lexical core evaluator

#### 概要・目的・状態

- lexical scope、closure、first-class function: `暫定`だが最終言語に不可欠。
- strict call-by-value: `暫定`。
- application/record field等の左から右評価: `暫定`。
- tail call保証: 過去案にあるが明示承認がなく`暫定`。
- hidden global stateを持たない: `確定`の設計原則。

#### 中心構文

`G-EVAL-001` — `暫定`

```ebnf
e ::= x | literal
    | (fn (x*) e+)
    | (let ((x e)*) e+)
    | (letrec ((x e)*) e+)
    | (if e e e)
    | (match e clause+)
    | (e e*)
    | record-expression
    | effect-expression
```

`val`、named function sugar、`with`、`var`、module syntaxはCoreへ脱糖する。

#### 判断と規則スケッチ

`T-FN-001` — `暫定`

```text
Γ, x1:T1, …, xn:Tn ⊢ body ⇒ U ! E
────────────────────────────────────
Γ ⊢ (fn ((x1:T1) … (xn:Tn)) body)
    ⇒ (fn (T1 … Tn) E U) ! <>
```

closure生成自体は効果を発生せず、本体の効果は関数型へ入る。無注釈`fn`の
algorithmic inferenceではfresh metavariableを生成する。宣言的規則と推論algorithmは
別物である。

`T-APP-001` — `暫定`

```text
Γ ⊢ f ⇒ (fn (T1 … Tn) E U) ! Ef
Γ ⊢ ei ⇐ Ti ! Ei   (i=1..n)
────────────────────────────────────
Γ ⊢ (f e1 … en) ⇒ U ! join(Ef,E1,…,En,E)
```

`join`は順次実行のeffect upper boundであり、同じoperationを二度実行した回数を
duplicate labelとして数えない。Koka型のduplicate labelを導入するrow extension、
masking、handler eliminationとは別演算である。正確な規則は`EFF-001`で未決定。

`E-BETA-001` — `暫定`（CEK風の略記）

```text
⟨apply(closure(ρc,(x1…xn),body),(v1…vn)), ρ, σ, H, κ⟩
  → ⟨body, ρc[x1↦v1,…,xn↦vn], σ, H, κ⟩
```

#### 値・評価文脈

暫定値:

```text
v ::= literal | closure | record-value | variant-value
    | handler-value | packed-module? | resource-handle?
```

評価文脈はoperator、引数1..n、record field source orderを左から右に進む。
`if`はconditionだけを先に評価し、選ばれたbranchだけを評価する。

#### 例

```lisp
((fn (x) (+ x 1)) 41)
```

- 推定型: `Integer`
- 推定結果: `42`
- 関連: `T-FN-001`, `T-APP-001`, `E-BETA-001`

評価順序例:

```lisp
(f (log "a") (log "b"))
```

- 暫定観測: `"a"`のeffectが先。

#### エラー・反例

- 非関数適用: 静的拒否。dynamic境界ではcast/blame候補。
- arity mismatch: 静的拒否。dynamic境界の分類は`OPEN-TYP-001`。
- non-exhaustive match: exhaustivenessで拒否するかexplicit MatchError effectか`未決定`。
- 発散: 許容。強正規化を保証しない。
- zero division: effect/error/IEEE semanticsが`未決定`。

`CE-EVAL-001`: unrestricted `letrec x = x`を値として読み出すとstuckし得る。
関数値だけを再帰束縛可能にするか、初期化cellを型付きエラーにする必要がある。

#### Progress/Preservation

目標Progress:

> empty effect rowを持つwell-typed closed Core termは、値であるか、一歩進むか、
> 仕様に列挙された明示的終端エラーである。

unhandled effect、dynamic cast、resource operationを含む場合は前提を拡張する必要がある。
未証明。

目標Preservationの**形だけを示す未完成スケッチ**:

```text
WellTypedConfig(Δ,M,Σ, ⟨e,ρ,σ,H,κ⟩, T, E)
⟨e,ρ,σ,H,κ⟩ → C'
──────────────────────────────────────────
∃Σ',T'. RelatedWorld(Σ,Σ') ∧ T' <: T
       ∧ WellTypedConfig(Δ,M,Σ',C',T',E)
```

ここでは1 step中のeffect annotationを同じ`E`で保つ候補を示したが、handler reductionで
residual effectを縮小する定式化もあり得る。scoped store破棄があるため単純な
`Σ'⊇Σ`ではなく`RelatedWorld`を未定義のまま置いている。environment、store、handler、
continuationを含むconfiguration typing、置換、弱化、型代入、evaluation-context lemmaが
必要である。証明はない。

#### 実装・テスト

必要構造: resolved AST、typed Core、closure environment、value stack、handler stack、
source map。現行実装には一般評価器がなく、effect formの限定走査だけである。

- `TEST-EVAL-001`: beta reduction。
- `TEST-EVAL-002`: lexical captureとshadowing。
- `TEST-EVAL-003`: effectを使うleft-to-right litmus。
- `TEST-EVAL-004`: tail recursion。
- `TEST-EVAL-005`: reference evaluatorと将来compilerの差分試験。
- `TEST-EVAL-006`: typed random termのstep preservation経験試験。

### 13.4 `BND-001` `val`、`var`、`let`、`fn`

#### 概要・目的・状態

- `val`: 不変束縛、`確定`。
- `var`: Koka型の局所可変束縛、`確定`。細部は`未決定`。
- `let`: 局所scope、`確定`。具体構文は`暫定`。
- `fn`: lambda表面名、`確定`。
- `define`を使わない: `確定`。

#### 構文と脱糖

```lisp
(val radius 40mm)

(val add
  (fn (x y) (+ x y)))

(val (add x y)
  (+ x y))
```

最後は次へのhygienic sugar候補。

```lisp
(val add (fn (x y) (+ x y)))
```

局所束縛:

```lisp
(let ((x 10) (y 20))
  (+ x y))
```

`let`のbindingをparallelにするかsequentialにするかは`未決定`。必要なら`let*`を
library macroにする。

局所状態:

```lisp
(var count 0)
(set count (+ count 1))
```

assignment演算子/関数名は`未決定`。`var`は意味論上、fresh state instanceとhandlerへ
脱糖する設計目標である。

#### 静的意味

- `val`は再代入不可。
- `var`は宣言scope内だけで更新可能。
- polymorphic generalizationにvalue restrictionを採用する案は`暫定`である。
- effectful式、`var`、resumptionを含む束縛を無制限にgeneralizeしない。

`T-VAL-001A` — effectful monomorphic binding（`暫定`）:

```text
Γ ⊢ e ⇒ T ! E
────────────────────────────
Γ ⊢ val x = e extends Γ with x:T, and declaration has effect E
```

`T-VAL-001B` — generalization候補（`暫定`）:

```text
Γ ⊢ e ⇒ T ! <>
ᾱ = ftv(T) - ftv(Γ)
e は generalizable value
──────────────────────────
Γ ⊢ val x = e extends Γ with x:∀ᾱ.T
```

任意の許可されたeffectful式を`val`へ束縛することと、多相一般化することを区別する。
effectful valueのrelaxed value restrictionは`未決定`。

#### 動的意味・反例

`var`がmulti-shot continuation内にある場合、resume複製時に状態を共有するか複製するかで
結果が変わる。Kokaの意味へ従う方針だが、正確なelaborationは`OPEN-EFF-001`。

`CE-BND-001`: polymorphic mutable cell。

```lisp
(var x (fn (y) y))
; xを異なる型として更新・利用
```

無制限generalizationすると型不健全になり得る。value restrictionが必要。

#### テスト

- `TEST-BND-001`: `val`再代入拒否。
- `TEST-BND-002`: `var`scopeとshadowing。
- `TEST-BND-003`: closureがvarをcaptureする場合。
- `TEST-BND-004`: multi-shot resume×var。
- `TEST-BND-005`: polymorphic reference反例。
- `TEST-BND-006`: `val (f x)` sugarのhygiene。

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

#### 概要・目的・状態

集合論的漸進型を採用することは`確定`。Elixirのgradual set-theoretic types、
semantic subtyping、および関連研究を基礎とする。現在の固定builtin type checkerは
最終仕様の実装ではない。

#### 型文法

`G-TYP-001` — `暫定`

```text
S ::= Any | Never | Base
    | (union S+)
    | (intersect S+)
    | (not S)
    | (diff S S)
    | (fn (S*) (effects E) S)
    | RecordType
    | α
    | (forall ((α Type) (R RecordRow) (E EffectRow)) S)
    | μ α . S

G ::= S | (dynamic S)
```

`S`はstatic semantic type、`G`はgradual typeである。以下の型等価・集合意味論は
**dynamicを除くstatic fragmentだけ**に適用する。

```text
S ≃ U  iff  S <: U and U <: S
S <: U iff [[S]] ⊆ [[U]]
S <: U iff intersect(S, not(U)) is empty
```

関数のsubtypingは引数反変、結果共変、効果は少ない方がsubtypeという目標。

#### Gradual boundary

`(dynamic S)`は「実行時情報を含むがstatic上限はS」というbounded dynamic。
`Any`と同じではない。Elixir同様dynamicをrootへ持ち上げる案は`暫定`。
gradual fragmentにはprecision、consistency/consistent subtyping、evidence、cast insertion、
blame polarity、runtime checkを別判断として定義する必要がある。これらは
`OPEN-TYP-001`であり、`dynamic S`を単なる値集合としてsemantic negationへ入れない。

#### 推論

- rank-1/prenex: `暫定`。
- bidirectional/local inference: `暫定`。
- immutable syntactic valueだけgeneralize: `暫定`。
- recursive/public/higher-rank definitionにannotation: `暫定`。
- principal type、完全推論は保証しない見込みだが正式決定は`未決定`。
- constraintsはtype/record-row/effect-row kind別に生成・解決する。

#### 例

```lisp
(type-alias Scalar
  (union Integer Float Length Angle))
```

```lisp
(type scale
  (intersect
    (fn (Number Number) Number)
    (fn (Vector Number) Vector)
    (fn (Picture Number) Picture)))
```

intersection functionのapplication規則、overlap ambiguityは`未決定`。

型エラー:

```lisp
((fn (x) (+ x 1)) "text")
```

- `String`が`Number`のsubtypeでないため拒否。
- 診断はargument span、期待/実際、subtyping counterexampleを示すべき。

#### Progress/Preservation

Static fragmentでは通常の目標を置く。dynamic fragmentではcast failure/blameを
明示的な終端結果へ含める必要がある。semantic subtypingのdenotational soundness、
algorithmic subtypingのsoundness/completeness/terminationはすべて未証明。

必要補題:

- type substitution
- value substitution
- semantic subtyping reflexivity/transitivity
- cast/evidence composition
- canonical forms（union/intersectionを考慮）

#### 実装

必要構造:

- hash-consed type DAG
- Boolean normal form/BDD相当またはspecialized emptiness solver
- kinded metavariable
- constraint/tallying set
- recursive type cycle representation
- cast evidence
- typed ASTとdiagnostic provenance

#### テスト

- `TEST-TYP-001`: union/intersection/negation代数。
- `TEST-TYP-002`: subtyping reflexive/transitive。
- `TEST-TYP-003`: normalization idempotence。
- `TEST-TYP-004`: finite modelとのsubtyping differential。
- `TEST-TYP-005`: rejected emptinessのwitness探索。
- `TEST-TYP-006`: bounded dynamic narrowing。
- `TEST-TYP-007`: cast composition/blame。
- `TEST-TYP-008`: value restriction regression。
- `TEST-TYP-009`: occurrence typingとunreachable branch。

### 13.7 `ROW-001` Row-polymorphic records

#### 概要・状態

semantic subtyping上にrecord row polymorphismを統合することは`確定`。
RecordRowをEffectRowと分離することも`確定`。

#### 構文

```lisp
; closed
(record (x Number) (y Number))

; open
(record (x Number) ...)

; polymorphic tail
(record (x Number) (.. R))

; optional
(record (title String) (author? String) (.. R))
```

optional fieldは、record意味領域における「present with T」と「absent」の選択として
扱う。`Absent`は通常のruntime値型ではなくrecord-row固有のabsence atomであり、
runtimeの`None`/`Nil`とは異なる。説明上`T | Absent`と略記しても、通常値の
`union`へ混入させてはならない。

明示field `x`を持つ`(.. R)`には自動的に`R lacks x`制約を付ける。

#### 型規則・操作

代表型:

```lisp
(type set-title
  (forall ((A Type) (R RecordRow))
    (fn ((record (title A) (.. R)) String)
        (record (title String) (.. R)))))
```

record field評価順序はsource order（`暫定`）。duplicate literal fieldは静的拒否。
extension時のduplicateはupdateとextensionを別操作にして曖昧性を避ける案が`暫定`。

row Boolean combinationは型検査器内部で扱い、表面公開は`未決定`。

#### 例

```lisp
(type-alias (Page R:RecordRow)
  (record
    (size Size)
    (content Picture)
    (metadata Metadata)
    (.. R)))
```

`resize-page`は未知の`japanese-layout`や`crop-marks` fieldを保存する。

#### 反例・メタ理論

`CE-ROW-001`: open record `{x:Number,...}`をrow変数と同一視すると、入出力間で余分な
field型を保持できない。

`CE-ROW-002`: negationとoptional/Absentの補集合を通常値領域で計算すると不正確になる。
record field domainにAbsentを含むquasi-constant function semanticsが必要。

subtyping/tallyingは Castagna/Peyrot 系理論を参考にするが、RPXに対する証明はない。

#### テスト

- `TEST-ROW-001`: field順序permutation。
- `TEST-ROW-002`: extension/update/removal型保存。
- `TEST-ROW-003`: lacks constraint。
- `TEST-ROW-004`: optionalとpresent Noneの区別。
- `TEST-ROW-005`: union/intersection/negation record。
- `TEST-ROW-006`: polymorphic extra field preservation。
- `TEST-ROW-007`: duplicate label rejection。

### 13.8 `EFF-001` Algebraic effects and handlers

#### 概要・状態

- effect定義名を`eff`にする: `確定`。
- Kokaの重複可能effect rowとscoped handlerを主要な参照設計とする: `確定`。
  Kokaとの意味論的・構文的互換性は`未決定`。
- operation発生を`use`必須にする: 検討後、通常関数呼出しを推奨したが最終承認が
  明示されていないため`暫定`。
- `handle`を基本、`with`をscope sugarとする: `暫定`。
- RecordRowと異なる、重複ラベル可能なEffectRow: `確定`。

#### 構文

```lisp
(eff Random
  (random () Number))
```

推奨operation call:

```lisp
(random)
```

`(use random)`は、残すなら上記への任意sugarであり、Core formにはしない。

handler value:

```lisp
(val random-handler
  (handler Random
    (random (resume)
      (resume 0.5))))
```

基本適用:

```lisp
(handle random-handler
  (fn () (make-picture)))
```

scope sugar:

```lisp
(with random-handler
  (make-picture))
```

は前者へ脱糖する。

型内構文:

```lisp
(fn (Request)
    (effects Resource Log (.. E))
    Response)
```

`| E`の代わりに`(.. E)`を使う案は、S式で句読点を減らすための`暫定`案。

#### 型・動的意味

operation型:

```text
op : A ->{Eff} B
```

operation実行はnearest matching handlerまでdelimited continuationをcaptureする。
handlerがhandled labelをrowから除去し、handler clause自身のeffectを加える。

未決定:

- deepかshallowか
- one-shotかmulti-shotか
- return/finally clause
- forwarding/masking
- answer type polymorphism
- duplicate labelの正確なrow subtraction
- named effect instance/generativity

これらが未決定のため、完全な小ステップ規則は記載できない。

#### 例・反例

テスト用resource:

```lisp
(with memory-resources
  (render project))
```

closed test bodyに未処理`FileWrite`が残る場合は型エラーにする設計目標。

`CE-EFF-001`: duplicate labelを集合として消去すると、nested同名handlerを一度で全消去し、
scoped handlingが壊れる。

`CE-EFF-002`: multi-shot continuationと外部`SourceEdit`を組み合わせると、同じpatchが複数回
適用され得る。one-shot制約またはtransaction idempotenceが必要。

#### Progress/Preservation

empty effect rowのclosed termだけを通常Progress対象にする。非empty rowは適切なhost
handler下で進む、またはunhandled operationを明示終端とする必要がある。
handler preservation theorem、resumption substitution lemmaが必要。未証明。

#### 実装・テスト

必要構造: effect identity、operation table、duplicate-aware effect row、
handler frame、continuation segment、resume capability、source span。

- `TEST-EFF-001`: nested same-label handler。
- `TEST-EFF-002`: forwarding/rehandling。
- `TEST-EFF-003`: duplicate row unification。
- `TEST-EFF-004`: effect polymorphism。
- `TEST-EFF-005`: left-to-right operation order。
- `TEST-EFF-006`: state×multi-shot。
- `TEST-EFF-007`: unhandled effect診断。
- `TEST-EFF-008`: `with`と`handle`のobservational equivalence。

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

value restrictionを採用する方針は`暫定`。generalization pointはimmutable `val`。
module signature境界では明示quantifierを保持する。`var`、effectful computation、
resumptionを含む項は原則generalizeしない。

### 15.5 Subtypingと制約解決

- semantic subtypingを宣言的関係とする。
- algorithmはemptiness/normalization/tallyingへ帰着する。
- 宣言的完全性を実装algorithmが満たすかは`未決定`。
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

完全性は集合論的型、polymorphism、rows、effectsの組合せで意図的に諦める可能性が高いが、
範囲は`OPEN-TYP-002`。

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

1. 最終Core calculusが未固定。
2. handler semanticsとduplicate effect rowの除去規則が未固定。
3. gradual evidence/blameが未設計。
4. semantic subtyping＋row polymorphism＋effectful arrowの実装複雑性が高い。
5. hygienic macroとmodule sealing/provenanceのauthorityが未設計。
6. `var`とmulti-shot continuationが不健全または直感に反する可能性。
7. Editableのstale/concurrent transactionが未設計。
8. layered IRのpixel/temporal equivalenceが未定義。
9. memory/concurrency modelが未定義。
10. 現行実装は最終pipelineより大幅に単純で、migrationが必要。

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

### 最優先

1. `OPEN-EFF-001`: deep/shallow、one/multi-shot、answer type、duplicate handling。
2. `OPEN-TYP-001`: bounded dynamic、cast evidence、blame。
3. `OPEN-EVAL-001`: Core term/value/context、match、letrec、primitive error。
4. `OPEN-BND-001`: varのescapeとstate handler elaboration。

### 高

5. `OPEN-MAC-001`: phase/hygiene/capture/fuel。
6. `OPEN-EDT-001`: stable ID、source transaction、authority。
7. `OPEN-TYP-002`: checkerが完全性を諦める範囲。
8. `OPEN-MEM-001`: GC/reference/resource lifetime。
9. `OPEN-ERR-001`: exception/error taxonomy。
10. `OPEN-IR-001`: color/filter/timing/backend tolerance。
11. `OPEN-MOD-001`: first-class/recursive module、separate compilation identity。

### 中

12. `OPEN-SYN-001`: `doc`/`src`名称とcompatibility。
13. `OPEN-SYN-002`: literal/comment/escape/units。
14. `OPEN-TST-001`: test syntax、allowed effect row。

### 低

15. qualified nameを`M.x`、`M/x`のどちらにするか。
16. `syntax`、`macro`等の表面keyword。
17. diagnostic wording/style。

## 27. 次に行うべき作業

1. 最小Core calculusを別文書で固定する。`fn/val/let/letrec/match`とCBVだけから始める。
2. Kokaを基準に`EFF-001`のhandler semanticsを選び、Coreへ追加する。
3. bounded dynamicを除くstatic set-theoretic fragmentを先に形式化する。
4. closed/open recordからrow-polymorphic recordへ段階的に拡張する。
5. `var`を選択したhandler semanticsへelaborateし、multi-shot反例を検査する。
6. module calculusとmacro phaseをCore外のelaborationとして固定する。
7. Editable transactionをsource version付きの独立状態機械として定義する。
8. VisualIR/RenderIR schemaとvalidatorを独立仕様に分離する。
9. 本書の各`OPEN-*`解決時に状態を更新し、決定履歴を追記する。

---

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
