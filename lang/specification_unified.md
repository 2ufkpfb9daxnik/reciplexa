# reciplexa / RPX 統合仕様書

> **文書状態**: 実装着手のための統合設計指針  
> **統合日**: 2026-08-08  
> **入力文書**: 添付された `specification.md` と `solve.md`  
> **対象**: RPX言語、reciplexa編集基盤、Runtime、Toolchain、Backend、Package、GUI、Native境界

## 0. この文書の権威と運用規則

本書は、reciplexaおよびRPXの新しい設計に関する**第一の設計指針**である。実装者は、局所的に実装しやすいという理由だけで、本書の背景、目的、要件、外部仕様、確定済み設計決定を変更してはならない。

本書と実装が食い違う場合は、実装を本書へ合わせるか、背景、目的、要件、外部仕様、影響範囲を再検討して本書を先に更新する。コードだけを変更して本書を古いまま残すことは禁止する。未決定事項は暗黙の挙動として固定せず、明示的な設計決定として本書へ追加する。

この方針は、[「ドキュメントに固執せよ」](https://gfngfn.github.io/ja/posts/2022-06-18-be-a-documentation-geek/)が示す、背景、目的、要件、外部仕様を最初に明らかにし、想定読者、結論先行、大枠から詳細、用語定義を一貫させるという原則を採用したものである。

### 0.1 文書変更の規則

- **設計変更は文書変更を伴う**。公開動作、型、Effect、Failure、Lifetime、Capability、永続化形式、適合条件を変える変更は、該当節を更新する。
- **未決定事項を推測で固定しない**。一時実装なら実験的状態、破棄条件、依存禁止範囲を記録する。
- **用語を勝手に増やさない**。既存用語で表現できない場合は用語集へ定義を追加する。
- **内部実装から外部仕様を逆算しない**。背景、目的、要件、外部仕様から内部設計を導く。
- **適合試験を設計の一部とする**。重要な契約には検証方法を対応付ける。

### 0.2 想定読者

本書は、言語処理系、編集、描画、Backend、Package、GUI、Native境界、適合試験、Migration、Securityを実装またはレビューする人を対象とする。会話履歴を前提とせず、一般的でない語は初出または用語集で定義する。

### 0.3 読み方

1. 第I部で背景、目的、要件、外部仕様、用語を確認する。
2. 第II部で基礎仕様を確認する。
3. 第III部でOPEN項目の解決済み決定を確認する。
4. 実装時には対象節だけでなく、Identity、Outcome、Snapshot、Transaction、Lifetime、Diagnosticの横断規則も確認する。

---

# 第I部 背景・目的・要件・外部仕様

## 1. 背景

文書、スライド、ベクター画像、アニメーション、組版、GUI編集、成果物生成は、通常、異なるアプリケーションと内部表現へ分断される。その結果、利用者が新しい表現モデル、編集操作、出力形式、検証規則を必要としても、既存アプリケーションがあらかじめ提供する機能へ制約されやすい。

RPXとreciplexaは、これらを固定機能の集合ではなく、型付き言語、Package、編集Transaction、段階化IR、Backend Contractとして構成可能にする。言語だけでなく、Package、Runtime、編集、GUI、永続化、Backend、Native、Diagnostic、Testを一つの系として設計する。

```text
RPX Source
↓
Parser・Macro・Module・Type・Effect
↓
Evaluation・Task・Resource・Capability
↓
Editable Document・Transaction・GUI
↓
Domain IR・Visual IR・Motion IR・Render IR
↓
Backend Planning・Emission・Verification
↓
PDF・SVG・PPTX・画像・動画・Preview等のArtifact
```

## 2. 目的

1. 文書、スライド、図形、アニメーションを、固定アプリケーション内部モデルではなくPackageと型付き関数で構成可能にする。
2. GUI編集とProgram生成を、同じDocument IdentityとTransaction上で接続する。
3. Portableな意味を正本としながら、Native実装、GPU、OS機能、既存Libraryを安全に利用する。
4. Failure、Cancellation、Defect、Terminal Failureを区別し、cleanupを含めて予測可能に扱う。
5. 文書、Package、Build、Artifactを再現可能かつ検証可能にする。
6. Backend差を暗黙劣化で隠さず、Capability、Output Profile、Planning、Loss、Verificationとして明示する。
7. 長期保存、Schema Migration、未知Extension、部分復旧に耐える。
8. Subsystemを独立実装しても全体契約が矛盾しない設計指針を提供する。

## 3. 大域要件

- **言語**: 評価順、束縛、型、Effect、Failure、Module、Packageの意味を明示する。
- **実行**: Taskを構造化ConcurrencyのScopeへ所属させ、ResourceとCapabilityにOwner、Lifetime、cleanup契約を持たせる。
- **編集**: 文書変更をRevisionを持つ原子的Transactionとし、Stable Node Identityを保存、Undo、GUI、Diagnostic、Migrationへ共通利用する。
- **出力**: Backend能力と今回許容するLossを分離し、事前PlanningとArtifact Verificationを必須とする。
- **永続化**: Portable Snapshot、Transaction Log、Resource Manifestを分離し、Capability、Secret、Task、Native pointerを保存しない。
- **Toolchain**: Package resolution、Build、Test、Diagnosticを決定的にし、BudgetExceededをUnsupportedと区別する。
- **SecurityとPrivacy**: Secretを保存・表示せず、Capabilityを最小権限に縮小し、未信頼Codeを隔離可能にする。

## 4. 外部仕様の概要

- 利用者はRPX Source、Package Manifest、Target、Entry Profile、Runtime Profile、依存Requirementを記述する。
- ToolchainはPackageを解決し、型、Effect、Capability、Target条件を検査してArtifactを生成する。
- EditorはPortable Documentを開き、Stable Node IDを参照するTransactionをcommitする。
- GUIは宣言的DescriptionからMounted instanceへReconcileし、Stable Keyと互換性に基づきStateを継承する。
- 出力要求はOutput Profileを指定し、Backend Plannerが表現方法とLossを事前決定する。
- Native実装はPortable ContractのProviderであり、Runtime ProfileがPolicyとConformance evidenceから選ぶ。
- Compiler、Runtime、Editor、Backend、Package、Testは同じ構造化Diagnostic schemaを利用する。

## 5. 規範用語

- **必須とする**: 適合実装が満たさなければならない。
- **禁止する**: 適合実装が行ってはならない。
- **許可する**: 実装またはProfileが選択してよい。
- **推奨する**: 特段の理由がなければ採用し、採用しない場合は理由と影響を記録する。
- **正本**: 意味を決定する権威ある表現。
- **規範的**: 適合性判断に使用できる。
- **説明用**: 理解の補助であり、それだけでは構文や動作を確定しない。

## 6. 用語集

- **reciplexa**: RPXを用いて文書、図形、スライド、アニメーション等を生成・編集する言語処理系とソフトウェア基盤。
- **RPX**: reciplexaで使用する型付きLisp系言語。
- **Contract（契約）**: 型、Effect、Failure、Lifetime、Capability、同値性等からなる外部仕様。
- **Portable（移植可能）**: 特定OS、Device、Native ABI、Process内pointerへ依存しない規範表現または実装。
- **Native implementation（ネイティブ実装）**: OS、Device、外部Library、機械語Binary等を利用してPortable Contractを実現する実装。
- **ABI**: Binary間のcalling convention、data layout、symbol、ownership等の低水準契約。
- **Effect（作用）**: 計算が必要とする、または外部へ及ぼす作用を型として表したもの。
- **Capability（権限能力）**: ResourceやOperationを使用する権限を表すScopeとLifetimeを持つ値。
- **Failure（失敗）**: Programの公開契約内で起こり得る型付きの回復可能な失敗。
- **Cancellation（取消し）**: 結果が不要になった、または親Scopeから停止された制御結果。
- **Defect（欠陥）**: 本来成立すべき不変条件またはContractの違反。
- **Terminal Failure（終端的失敗）**: RuntimeまたはProcessの健全性を保証できない状態。
- **Outcome（結果区分）**: Completed、Failed、Cancelled、Defected、Aborted等の終端状態。
- **Identity（同一性識別子）**: 対象が同じ論理対象かを判定する安定識別情報。
- **Revision（版識別子）**: 同じ論理対象の特定時点の状態を識別する情報。
- **Snapshot（スナップショット）**: 特定Revisionで固定された入力、Profile、Resource、文書状態等の不変記録。
- **Transaction（トランザクション）**: 検証、commit、rollback境界を持つ原子的状態遷移。
- **IR**: Sourceと最終Artifactの間で意味を表す中間表現。
- **Backend**: Render IR等をPDF、SVG、PPTX、画像、動画、Previewへ変換する実装。
- **Output Profile（出力プロファイル）**: 必ず保持する性質、許可する変換、Tolerance、Preference、Failure Policyを表す設定。
- **Lowering（低水準化）**: 高水準表現を意味を保ちながら低水準表現へ変換すること。
- **Approximation（近似）**: 規定Error modelとTolerance内で意味を近似すること。
- **Raster fallback（ラスター代替）**: Vector、Text、Effect等を画像へ変換する代替方法。
- **Conformance Test（適合試験）**: 実装がContractを満たすことを検査する試験。
- **Differential Test（差分比較試験）**: 複数実装を同じ入力で比較する試験。
- **Provenance（来歴）**: Source、IR、Plan、Artifact間の由来と変換経路。
- **Stable Key（安定キー）**: GUI再構築を越えて同じState ownerを識別するKey。
- **Reconciliation（照合更新）**: 旧GUI instanceと新descriptionを照合し、State継承やMount等を計画・commitする処理。
- **Opaque（不透明）**: 内容の意味を解釈せず、境界とIntegrityだけを検査して保持する状態。
- **Migration（移行）**: 有効な旧Schemaの値を有効な新Schemaへ変換すること。
- **Recovery（復旧）**: 不正または不完全な保存Dataから安全な部分を救出すること。
- **Budget（予算上限）**: 探索、Memory、Operation数、再試行等を必ず停止させる構造的上限。
- **Quarantine（隔離停止）**: 問題のある実装やinstanceを再選択から除外する状態。

英語を残す場合は初出で日本語の意味を併記する。一般的な日本語で明確なら日本語を優先し、型名、識別子、規範enum、既存技術用語として英語が正本なら英語を用いる。

## 7. 横断設計原則

1. IdentityとRevisionを分離する。Content hash、Memory address、配列index、Source行番号を論理Identityにしない。
2. 結果へ影響する暗黙環境をSnapshot化する。
3. Document編集、Output生成、GUI commit、Native初期化等をTransactionとして扱い、半端な状態を公開しない。
4. Task、Resource、Capability、GUI State、Native instance、CallbackにOwnerとLifetimeを持たせる。
5. FailureとDefectを混同しない。
6. Portableな意味を正本とする。
7. 暗黙Fallbackを禁止する。
8. 候補列挙順やThread schedulingで規範結果を変えず、探索にBudgetを設ける。
9. Diagnosticを構造化し、表示文字列を正本にしない。

## 8. 解決済みOPEN項目

| ID | 根拠 | 位置 |
|---|---|---|
| `OPEN-BACKEND-PROFILE-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-CON-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-ERR-DIAG-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-GUI-STATE-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-IR-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-KER-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-NATIVE-PKG-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-PKG-ENTRY-001` | `solve.md`に独立した決定本文あり | 第III部 |
| `OPEN-TST-001` | `solve.md`に独立した決定本文あり | 第III部 |

## 9. 後続確認が必要なOPEN識別子

以下は入力文書中で参照されたが、添付`solve.md`に同名の独立決定本文がない識別子である。これは自動的に未決定を意味しない。第II部で解決済み、部分解決済み、棄却、または別IDへ統合済みの可能性がある。実装着手前に本文上の状態を確認し、未決定なら完全に決定する。

- **BLD系**: `OPEN-BLD-001`
- **BND系**: `OPEN-BND-001`
- **CON系**: `OPEN-CON-CHANNEL-001`, `OPEN-CON-PAR-001`
- **DAT系**: `OPEN-DAT-001`, `OPEN-DAT-001A`, `OPEN-DAT-001B`, `OPEN-DAT-001C`, `OPEN-DAT-001D`, `OPEN-DAT-001E`, `OPEN-DAT-001F`, `OPEN-DAT-001G`, `OPEN-DAT-001H`, `OPEN-DAT-001I`
- **DERIVE系**: `OPEN-DERIVE-001`
- **DYNAMIC系**: `OPEN-DYNAMIC-001`
- **EDT系**: `OPEN-EDT-001`, `OPEN-EDT-CODEC-001`, `OPEN-EDT-COLLAB-001`, `OPEN-EDT-OVERRIDE-001`
- **EFF系**: `OPEN-EFF-001`
- **ERR系**: `OPEN-ERR-001`
- **EVAL系**: `OPEN-EVAL-001`
- **GADT系**: `OPEN-GADT-001`
- **GRAPH系**: `OPEN-GRAPH-001`
- **IR系**: `OPEN-IR-3D-001`, `OPEN-IR-SIM-001`
- **KER系**: `OPEN-KER-CODEC-001`
- **LAZY系**: `OPEN-LAZY-001`
- **MAC系**: `OPEN-MAC-001`, `OPEN-MAC-001A`, `OPEN-MAC-001B`, `OPEN-MAC-001C`, `OPEN-MAC-001D`, `OPEN-MAC-001E`, `OPEN-MAC-001F`, `OPEN-MAC-001G`, `OPEN-MAC-001H`, `OPEN-MAC-CAP-001`, `OPEN-MAC-EXT-001`, `OPEN-MAC-PKG-001`, `OPEN-MAC-PROC-001`, `OPEN-MAC-TYPED-001`
- **MEM系**: `OPEN-MEM-001`, `OPEN-MEM-BORROW-001`, `OPEN-MEM-CELL-001`, `OPEN-MEM-PROF-001`
- **MOD系**: `OPEN-MOD-001`, `OPEN-MOD-001A`, `OPEN-MOD-001B`, `OPEN-MOD-001C`, `OPEN-MOD-001D`, `OPEN-MOD-001E`, `OPEN-MOD-001F`, `OPEN-MOD-001G`, `OPEN-MOD-001H`, `OPEN-MOD-001I`, `OPEN-MOD-FC-001`, `OPEN-MOD-GEN-001`, `OPEN-MOD-REC-001`
- **NATIVE系**: `OPEN-NATIVE-ABI-SCHEMA-001`, `OPEN-NATIVE-SEC-001`
- **PKG系**: `OPEN-PKG-001`, `OPEN-PKG-001A`, `OPEN-PKG-001B`, `OPEN-PKG-001C`, `OPEN-PKG-001D`, `OPEN-PKG-001E`, `OPEN-PKG-001F`, `OPEN-PKG-ENTRY-SURFACE-001`, `OPEN-PKG-FEAT-001`, `OPEN-PKG-HOT-RELOAD-001`, `OPEN-PKG-MANIFEST-SURFACE-001`
- **REG系**: `OPEN-REG-001`
- **ROW系**: `OPEN-ROW-001`
- **SEM系**: `OPEN-SEM-001`
- **SYN系**: `OPEN-SYN-001`, `OPEN-SYN-002`, `OPEN-SYN-002A`, `OPEN-SYN-002B`, `OPEN-SYN-002C`, `OPEN-SYN-002D`, `OPEN-SYN-002E`, `OPEN-SYN-002F`, `OPEN-SYN-002G`
- **TEXT系**: `OPEN-TEXT-JA-001`, `OPEN-TEXT-LAYOUT-001`
- **TST系**: `OPEN-TST-CONFORMANCE-001`, `OPEN-TST-RUNNER-001`, `OPEN-TST-SNAPSHOT-001`, `OPEN-TST-SURFACE-001`
- **TYP系**: `OPEN-TYP-001`, `OPEN-TYP-002`

## 10. 実装開始前の完了条件

- 対象機能の背景、目的、要件、外部仕様が本文にある。
- 用語が定義されている。
- Identity、Revision、Snapshot、Lifetime、Outcome、Diagnosticとの関係が明らかである。
- 未決定事項を暗黙既定値で埋めていない。
- Failure、Cancellation、Defect、BudgetExceededを区別している。
- Privacy、Capability、Resource ownershipを確認している。
- 適合試験または検証方法がある。
- 文書と実装を同じReview単位で変更する。

---

# 第II部 基礎仕様・設計記録

> この文書は、言語仕様、編集エンジン、Runtimeおよびtoolchainの合意済み設計を一体化した統合版である。各OPEN項目は現在の独立した課題として記述し、設計経緯より現在有効な契約、例、適合条件および残課題を優先する。

> 文書状態: 設計記録（規範仕様策定前）
>
> 更新日: 2026-07-27
>
> 対象: 最終的に目指す RPX 言語、および現行 reciplexa 実装との対応

### 0. この文書の読み方

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

#### 0.1 規範性

- `確定`であっても、構文・推論規則が未決定なら、その未決定部分は規範的ではない。
- `暫定`、`推定`、`未決定`、`矛盾候補`は実装適合性の根拠にしてはならない。
- 現行実装に存在するだけの機能は、最終言語へ自動的に採用されない。
- 例示中の構文は、該当節で`確定`と明記されない限り説明用である。

#### 0.2 ID体系

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

### 1. 言語の概要

#### 1.1 目的

`DD-001` — `確定`

RPX は、Word、PowerPoint、TeX、SATySFi、manim、Adobe Illustrator、
After Effects、Affinity 系アプリケーションに相当する用途を、個別アプリケーションの
固定された表現モデルへ閉じ込めず、型付き関数と必要最小限の hygienic macro により
ユーザー自身が構築できるようにする Lisp 系言語である。

核は「文書」「スライド」「アニメーション」を特別扱いしない。これらは RPX
パッケージとして構築される。Rust カーネルは、言語処理、低水準描画、リソース解決、
バックエンド境界、provenance を提供する。

#### 1.2 想定利用者

`推定`

- コードと GUI の双方で文書・図・動画を制作したい利用者
- 独自の組版・作図・アニメーション語彙を作るパッケージ作者
- 再現可能なレポート、教材、スライド、図版、動画を生成する研究者・制作者
- 型・テストによる検証を利用しつつ、ライブ編集も行いたい利用者

具体的な対象熟練度、教育用途、アクセシビリティ要件は`未決定`である。

#### 1.3 想定用途

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

### 2. 設計目標

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

#### 2.1 優先関係

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

### 3. 設計原則

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

### 4. 設計状態の要約

#### 4.1 解決済みの主要領域

- 最小Core、正格CBV、左から右の評価順序
- `val`、escape不能な局所`var`、`fn`、再帰binding、独立した型注釈
- 集合論的漸進型、semantic subtyping、RecordRow、EffectRow、bounded dynamic
- deep handler、one-shot continuation、forward、return clause
- code mode、字句・literal・宣言group・markup reader
- sealed ADT、constructor固有型、pattern、網羅的`match`
- ファイル単位module、`.rpi`、signature、適用的Functor、分割コンパイル
- package manifest、lockfile、workspace、package resource
- 同一コンパイル単位内の最小衛生的式マクロ
- 編集Snapshot、Transaction、競合、Undo、Provenance
- `option`／`result`／`failure E`／Defect／Terminal failureの分類
- `bracket`によるcleanup
- Perceus方式の精密参照カウント、scoped resource、メモリ予算

#### 4.2 主要未決定領域

- `OPEN-CON-001`: 構造化Concurrency、Cancellation、task間共有、atomic reference count
- `OPEN-KER-001`: ForeignValue、validator、trusted adapter ABI、ownership metadata
- `OPEN-IR-001`: Domain／Visual／Motion／Render IRの規範schemaとbackend tolerance
- `OPEN-TST-001`: test構文、実行境界、許容Effect
- `OPEN-PKG-ENTRY-001`: CLI／GUI／Server／Worker entry profile
- `OPEN-NATIVE-PKG-001`: package APIとcompiler-native implementationの境界

#### 4.3 v1で意図的に導入しないもの

一般`cell`／`ref`、一般heap cycle、一般borrow、multi-shot continuation、first-class／recursive／generative module、procedural／typed／package公開macro、CRDT／OTはv1へ含めない。

### 5. 規範性と実装自由度

公開構文、静的意味、動的意味、observable behavior、公開API契約および適合試験は規範的である。内部データ構造、IDのbit表現、hash、cache、index、物理的in-place更新、memory layoutは、規範的意味を保つ限り実装依存とする。

### 6. 実装方針

- 型検査済みCoreから、Effect／handler／cleanupを明示的制御フローへloweringする。
- 通常値はPerceus方式で管理し、一意性に基づくreuseを意味保存最適化として行う。
- 外部Resourceの意味的解放は`bracket`が担当する。
- 公開package APIとcompiler intrinsicの意味を分離し、native化しても通常importとpackage identityを維持する。

### 7. 棄却済みの主要案

- Domain primitiveを最終kernelへ無制限に追加する。
- すべての拡張をmacroで行う。
- 型内のEffect区切りに裸の`/`を使う。
- すべてを一つの万能RenderIRへ押し込む。
- `src` wrapperを正規構文として残す。
- Tracing GCを通常値の標準memory managerとする。

### 8. 現行実装との主要差分

- `src`固有pipelineを廃止し、top-level codeを通常pipelineへ送る。
- 構造化文章readerは`markup`を正規形とし、未知commandを静的errorにする。
- 現行shape enumは高水準Scene IRとして扱え、最終IRへのlowering対象とする。
- 限定Effect runnerはdeep one-shot handlerと型付きEffectRowへ置き換える。
- Memory管理はPerceus Core passとownership verifierを備える構成へ移行する。

### 9. OPEN項目

#### 9.1 解決済み

`OPEN-EVAL-001`、`OPEN-EFF-001`、`OPEN-TYP-001`、`OPEN-BND-001`、`OPEN-TYP-002`、`OPEN-SYN-001`、`OPEN-SYN-002`、`OPEN-DAT-001`、`OPEN-MOD-001`、`OPEN-PKG-001`、`OPEN-MAC-001`、`OPEN-EDT-001`、`OPEN-ERR-001`、`OPEN-MEM-001`は解決済みである。旧IDは決定履歴と適合試験の追跡に使用する。

#### 9.2 高優先度

- `OPEN-CON-001`: Task scope、Cancellation、Failure伝播、値の移送、Perceusのthread共有対応
- `OPEN-KER-001`: trusted foreign boundary、validator、ABI、pinned／owned／shared値
- `OPEN-IR-001`: IR schema、座標、色、filter、timing、backend capability
- `OPEN-TST-001`: test Surface、Fault boundary、Failure／Defect期待値

#### 9.3 中優先度

- `OPEN-PKG-ENTRY-001`: Entry profileと`main`の正確な型
- `OPEN-ERR-DIAG-001`: Diagnostic schema、privacy、source/effect trace
- `OPEN-EDT-CODEC-001`: 文書・Transaction codecとmigration
- `OPEN-GUI-STATE-001`: Stable Keyと再評価時のmodel継承
- `OPEN-NATIVE-PKG-001`: compiler-native packageの選定基準、置換条件、互換性検証

#### 9.4 将来項目

一般`cell`／`ref`、borrow、cycle collector、macro拡張、package feature、registry protocol、build step、共同編集、派生node override、first-class／recursive／generative module、GADT、lazy、runtime reflection等は、必要性が確認された時点に独立項目として検討する。

### 10. 用語集

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

#### 10.1 Effect用語

- **effect signature environment**: effect／operationの静的signature環境。
- **effect environment**: 現在の式で利用可能または許容されるeffect要求の文脈。
- **expression effects**: 式を現在評価すると外側へ要求されるeffect。
- **function required effects**: 関数を呼び出すと外側へ要求されるeffect。
- **handled effects**: 現在のhandler／runnerが処理するeffect。
- **residual effects**: handler／runner適用後も外側へ残るeffect。

### 11. 記号一覧

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

### 12. 機能一覧

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

#### 12.1 依存関係の要約

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

### 13. 各機能の詳細

本節では、要求された詳細項目を機能ごとに記録する。機能に該当しない項目は
「該当なし」と明記する。形式規則は設計スケッチであり、証明済みではない。

#### 13.1 `LEX-001` Lossless lexer/CST

##### 概要・目的・状態

- 状態: `確定`。現行実装も `rowan` CST、trivia、mode stack を持つ。
- 目的: GUI編集後も空白・改行・コメント・ユーザーの記述順序を保持する。
- 利点: byte round-trip、局所patch、正確な診断、provenance。
- 欠点: ASTだけを扱う処理系よりメモリ・実装複雑性が増す。
- 代替: formatterによる全再生成。双方向編集の要求により`棄却`相当。

##### 構文・字句

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

##### 静的・動的意味

CSTそのものには型付け・動的評価を定義しない。spanはbyte offsetであることが現実装上の
事実だが、最終仕様ではencoding確定まで`暫定`。StableSyntaxIdの生成・編集後継承は
`OPEN-EDT-001`。

##### 正常例・拒否例

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

##### Progress/Preservation・その他

- CoreのProgress/Preservationの対象外。
- parse/unparseのround-trip、incremental parseとfull parseの同値性が代替する性質。
- parser totality、resource bound、malformed UTF-8 は未証明。

##### 実装構造・処理

- Token: kind、byte span、元text。
- Green/Red CST: tokenを一度ずつ順序保持。
- ParseError: span、期待、実際。
- Mode stack: code/doc reader mode。
- 入力bytes → lexer token → parser events → CST。型検査はCSTを直接使用せず、
  expanded AST/Coreへ変換するのが最終案。

##### 相互作用・テスト

- MAC: macro expansion originを保持する。
- EDT: patchはtoken/node境界を検証する。
- MOD: imported package sourceは編集権限を持たない。
- `TEST-LEX-001`: valid sourceのbyte round-trip。
- `TEST-LEX-002`: malformed delimiterのerror recoveryとround-trip。
- `TEST-LEX-003`: incremental/full parse differential fuzz。
- `TEST-LEX-004`: Unicode、CRLF/LF、巨大nest、巨大token。

##### 未決定

- `OPEN-SYN-002`（高）: encoding、escape、comment、unit literal。
- `OPEN-EDT-001`（高）: stable node identity。

#### 13.2 `SYN-001` Code mode・字句・Surface構文・markup reader

##### 統合方針

RPX source fileの既定reader modeはcode modeとする。Top-levelには宣言を直接記述し、`src` wrapperは使用しない。構造化文章へ入るreader formは、用途を文書だけへ限定しない`markup`とする。旧案の`doc`というreader-level名称は規範構文へ含めない。

```lisp
(type title str)
(val title "Report")
(val body
  (markup
    @heading(@(show title))))
```

Reader modeは入れ子を扱い、source order、source span、mode transitionおよびprovenanceをlossless CSTとsyntax objectへ保持する。Markup commandの意味はreaderが決めず、通常の名前解決・型検査へ渡す。未知commandは静的な名前解決errorであり、plain textへ暗黙変換しない。


##### 0. 結論

`OPEN-SYN-002`は、以下の方針で**解決済み**とする。

この項目で確定する範囲は次のとおり。

* source encoding、改行、BOM、shebang
* 空白と構造化コメント
* 識別子、予約形式、演算子名
* 数値・文字列・真偽値・unit・bytes
* symbol／keywordを導入しない方針
* 宣言グループ
* 関数・list・tuple・recordの基本構文
* 型構文
* 汎用的な構造化文章領域である`markup`
* 構文エラー回復
* lossless CSTへのsource情報の保存

単位、色、画像、URL、markup commandなどの**意味と型はpackageが定義する**。ただし、それらを記述するために必要な一般構文は本項目で確定する。

***

##### 1. Source fileと文字コード

###### 1.1 文字コード

RPX sourceはUTF-8だけを受理する。

```text
source encoding:
UTF-8
```

不正なUTF-8はlexerより前のsource decoding errorとする。

```text
invalid UTF-8:
source decoding error
```

通常の文字列も、妥当なUTF-8として表現可能なUnicode scalar value列だけを保持する。

***

###### 1.2 BOM

UTF-8 BOMはsource先頭に限って許可する。

```text
- byte offset 0に限る
- lossless CSTへ保持する
- RPXの意味には影響しない
- shebangとは併用できない
```

BOMとshebangが同時に存在した場合はsource errorとする。

***

###### 1.3 改行

次の改行形式を受理する。

```text
LF
CRLF
CR
```

行番号の計算上はいずれも一改行として扱う。

一方、lossless CSTではsourceに書かれた元の改行bytesを保持する。

***

###### 1.4 Source span

内部のsource spanはUTF-8 byte offsetによる半開区間とする。

```text
[start, end)
```

行・列番号は診断表示用にsource mapから算出する。

Unicode code point offsetや表示上のcolumn幅を、規範的なsource identityには使用しない。

***

###### 1.5 Shebang

Unix系環境での直接実行用に、任意のshebangを許可する。

```text
##!/usr/bin/env rpx
```

規則：

```text
- byte offset 0から始まる
- 第一行に限る
- RPXの型、module、effect、評価結果には影響しない
- lossless CSTへ保持する
- BOMとは併用できない
```

***

##### 2. 空白とコメント

###### 2.1 Code modeの空白

code modeでは、少なくとも次を空白として認識する。

```text
space
tab
form feed
newline
```

空白はtoken境界を作るが、通常はsemantic ASTには渡さない。

lossless CSTではすべて保持する。

***

###### 2.2 構造化コメント

コメント構文は次の一種類とする。

```lisp
(// コメント)
```

複数行も許可する。

```lisp
(//
  この範囲はコメントです。

  (val old-value 10)

  (render old-value))
```

コメント内部の内容は次の処理対象にならない。

```text
- macro展開
- 名前解決
- 型検査
- effect推論
- 評価
- 通常の未使用warning
```

***

###### 2.3 コメントの入れ子

構造化コメントは入れ子可能とする。

```lisp
(//
  外側のコメント

  (//
    内側のコメント)

  外側の続き)
```

readerはコメント終了位置を判断するため、コメント内部で次だけを追跡する。

```text
- 丸括弧の対応
- 文字列の開始と終了
- 文字列delimiter
- 入れ子の構造化コメント
```

コメント内の通常コードをparse、名前解決、型検査してはならない。

***

###### 2.4 コメント内部の括弧

コメント内部でも括弧対応を必須とする。

```lisp
(//
  (val x
    (+ 1 2)))
```

は有効。

括弧対応が壊れた範囲を任意に囲めるraw commentは、v1では導入しない。

必要性が確認された場合に、別のraw comment形式として将来検討する。

***

###### 2.5 採用しないコメント

次は導入しない。

```text
// 行末コメント
; 行末コメント
(* ... *)
```

裸の`//`を採用しないことで、`/`や演算子構文との競合を避ける。

コメントは原則として独立したformとして配置する。

***

###### 2.6 CSTでの保持

コメントは削除せず、専用nodeとしてlossless CSTへ保存する。

概念的には次の情報を持つ。

```text
StructuredComment {
  opening-span,
  content-span,
  closing-span,
  raw-bytes
}
```

通常ASTやtyped Coreからは除外する。

***

##### 3. 識別子

###### 3.1 Unicode識別子

通常識別子にはUnicodeを許可する。

```lisp
(val radius 40mm)

(val 半径 40mm)

(val 面積
  (* 幅 高さ))
```

識別子の基礎規則にはUnicodeの次の分類を使う。

```text
XID_Start
XID_Continue
```

日本語の漢字、ひらがな、カタカナなどは使用可能とする。

***

###### 3.2 Unicode正規化

識別子は名前解決前にNFC正規化する。

```text
元の綴り:
lossless CSTへ保持

NFC正規化後:
名前解決、重複判定、BindingId生成に使用
```

同じscopeで、異なるraw spellingがNFC正規化後に一致する場合はduplicate binderとする。

通常文字列は自動的にNFC正規化しない。

***

###### 3.3 先頭文字

caseを持つ文字では、小文字開始だけを許可する。

```text
report        許可
report-title  許可
半径          許可
レポート      許可

Report        拒否
REPORT        拒否
42-pages      拒否
-page         拒否
_report       拒否
```

caseを持たない日本語文字等は識別子の先頭へ使用できる。

型、constructor、moduleなども、大文字開始によって分類しない。名前のcategoryはnamespace、binding kind、BindingIdによって区別する。

***

###### 3.4 Kebab-case

複数語の区切りにはhyphenを使う。

```text
report-title
last-good-render
source-edit
```

制約：

```text
- 先頭に置けない
- 末尾に置けない
- 連続させられない
```

単独の`-`はoperator identifierとして別に扱う。

***

###### 3.5 Underscore

単独の`_`だけをwildcardとして予約する。

```lisp
(fn (_ value)
  value)
```

`_`はbindingを作らず、参照もできない。

通常識別子の一部には使用しない。

```text
report_title  拒否
_value        拒否
value_        拒否
```

数値separatorの`_`は別規則として許可する。

***

###### 3.6 `?`と`!`

通常識別子の末尾に限り、`?`または`!`を一つ付けられる。

```text
empty?
valid?
commit!
flush!
```

途中や重複は認めない。

```text
is?empty   拒否
value??    拒否
commit!!   拒否
valid?!    拒否
```

`?`と`!`は命名慣習であり、型やeffectの意味を直接持たない。

***

###### 3.7 不可視文字

識別子では次を禁止する。

```text
- bidi control
- zero-width space
- zero-width joiner
- zero-width non-joiner
- その他の不可視format文字
```

見た目が紛らわしい識別子や、不自然なscript混在はwarning対象にできる。

***

##### 4. Package名とmodule path component

通常の局所識別子にはUnicodeを許可するが、package名とmodule path componentはASCII lowercase kebab-caseに限定する。

```text
許可:
report
report-layout
japanese-typesetting
graphics2

拒否:
Report
日本語組版
report_layout
-report
report-
report--layout
```

`/`は通常識別子には含めず、module path用に予約する。

`.`も通常識別子には含めず、module仕様で用途を決定する。

***

##### 5. Operator identifier

v1ではoperator identifierを固定一覧とする。

```text
+
-
*
/
=
<
>
<=
>=
!=
```

`and`、`or`、`not`は通常識別子として提供する。

自由な記号列による利用者定義operatorは導入しない。

RPXにはinfix構文がないため、operatorもprefix applicationとして使う。

```lisp
(+ 1 2)
```

```lisp
(<= x y)
```

***

##### 6. Core特殊形式

初期のCore特殊形式として次を予約する。

```text
markup
fn
val
type
type-alias
local
rec
let
letrec
if
seq
var
set
handle
with
```

Core特殊形式はshadowing禁止とする。

```lisp
(val if value)
```

は静的エラー。

通常のpackage関数や利用者bindingは、通常のscope規則に従ってshadowing可能とする。

将来使用する可能性がある名前を、必要になる前から大量に予約しない。

***

##### 7. 数値リテラル

###### 7.1 整数

`int`は任意精度の符号付き整数とする。

```lisp
0
42
123456789012345678901234567890
```

基数：

```text
42       10進
0b101010 2進
0o52     8進
0x2a     16進
```

基数prefixは小文字だけを正式構文とする。

16進digitの`a`〜`f`は大小文字を許可する。

```lisp
0xff
0xFF
0xFf
```

***

###### 7.2 先頭ゼロ

不要な先頭ゼロを持つ10進整数は拒否する。

```text
許可:
0
7
70
0o7

拒否:
00
007
0123
```

固定桁の識別文字列等は文字列で表す。

***

###### 7.3 負数

負号の直後に空白なしで数字が続く場合、負の数値literalとして読める。

```lisp
-42
-3.5
-0xff
```

通常の符号反転も別に許可する。

```lisp
(- value)
```

したがって次は構文木上で異なる。

```text
-42:
負数literal

(- 42):
符号反転application
```

***

###### 7.4 数字separator

数字間のunderscoreをseparatorとして許可する。

```lisp
1_000_000
0b1010_1100
0xff_ff_ff
3.141_592
```

underscoreは値へ影響しない。

次は拒否する。

```text
_1000
1000_
1__000
0x_ff
1_.5
1._5
1e_10
```

***

###### 7.5 `f64`

次を`f64` literalとする。

```lisp
0.0
3.14
-2.5
1e10
1e+10
1e-10
1.5e3
```

小数点の前後には数字を要求する。

```text
許可:
0.5
1.0

拒否:
.5
1.
```

指数記号は小文字`e`を正式構文とする。

16進浮動小数点literalはv1では導入しない。

***

###### 7.6 非有限値

NaNとInfinityをliteralとして提供しない。

```text
nan:
literalではない

infinity:
literalではない
```

`f64` literalが有限範囲を超える場合は静的エラーとする。

ゼロへunderflowする場合はwarning対象とする。

`-0.0`は正のゼロへ勝手に正規化せず保持する。

***

##### 8. 文字列

###### 8.1 基本方針

RPXの文字列literalでは、backslash escapeを使用しない。

```text
\:
通常の文字
```

すべての文字列はUnicode `str`値を作る。

通常文字列：

```lisp
"短い文字列"
```

長い文字列、引用符を含む文字列、複数行文字列：

```lisp
"""
複数行の文字列
"""
```

***

###### 8.2 文字列delimiter

一個の`"`は短い一行文字列を囲む。

```lisp
"Report"
```

三個以上の`"`は可変長delimiterとして使用できる。

```lisp
"""She said "hello"."""
```

内容に三個連続する引用符がある場合は、外側のdelimiterを増やせる。

```text
開始delimiterと終了delimiterでは、
同数の引用符を使用する。
```

二個の引用符は、空文字列との曖昧性を避けるためdelimiter長として使用しない。

```lisp
""
```

は空文字列。

***

###### 8.3 複数行文字列

三個以上の引用符で囲んだ文字列には物理改行を含められる。

```lisp
"""
第一段落です。

第二段落です。
"""
```

規則：

```text
- 開始delimiter直後の改行は値へ含めない
- 終了delimiter直前の改行は値へ含めない
- 内容中の物理改行はruntimeではLFへ正規化する
- 元の改行bytesはlossless CSTへ保持する
```

終了delimiterのindentationをbaselineとして、各非空行から同じprefixを除去する。

baselineより浅い非空行は字句エラーとする。

***

###### 8.4 特殊文字

文字列escapeは導入しない。

特殊文字は、通常の値や関数で生成する。

```lisp
newline
tab
carriage-return
nul
```

任意のUnicode scalar valueは次で生成できる。

```lisp
(unicode 0x3002)
```

`unicode`は有効なUnicode scalar valueなら一文字分の`str`を返す。

実行時値の場合は不正code pointを`result`で表す。

compile-time定数が明らかに不正なら静的エラーにできる。

***

###### 8.5 `str`の意味

`str`は次の意味を持つ。

```text
- 妥当なUTF-8
- 不変
- 所有された値として振る舞う
- NUL終端ではない
- 内部NULを保持可能
- bytesとは別型
- 自動Unicode正規化を行わない
```

具体的な物理表現は言語仕様へ固定しない。

Rust実装では`Arc<str>`等を利用できるが、将来のsmall-string optimization、interning、rope等を妨げない。

任意の整数による暗黙文字indexは提供しない。

```text
byte
Unicode scalar value
grapheme cluster
```

を必要に応じて明示的に区別する。

***

##### 9. Boolとunit

真偽値literal：

```lisp
true
false
```

型関係：

```text
true  <: bool
false <: bool

bool ≃ union(true, false)
```

unit値：

```lisp
unit
```

空括弧`()`をunit値にはしない。

```text
unit : unit
```

何も返さない関数やbodyでも、必要なら`unit`を明示する。

***

##### 10. Symbolとkeyword

runtime symbol literalとkeyword literalはv1では導入しない。

```text
'circle:
不採用

:circle:
不採用
```

役割分担：

```text
型付きの列挙値:
variant constructor

外部名・metadata:
strまたは専用nominal型

record field:
静的field label

macro:
syntax object

named argument:
必要になった時点で別途設計
```

`circle`、`heading`、`portrait`等は言語組込みではなく、packageがconstructorとして定義する。

***

##### 11. Bytes

`bytes`は、0〜255の値からなる不変byte列とする。

```text
str:
valid UTF-8

bytes:
任意のbyte列
```

暗黙変換は行わない。

```lisp
(encode-utf8 text)
```

```text
str -> bytes
```

```lisp
(decode-utf8 data)
```

```text
bytes -> result<str, utf8-decode-error>
```

小さいbyte列は専用字句literalではなく、constructorで作る。

```lisp
(bytes 0x00 0xff 0x2a)
```

範囲外の静的要素はcompile-time error。

大きなbinaryはsourceへ埋め込まず、resource systemを使用する。

```text
resource:
外部データへの参照・authority

bytes:
実際に読み込まれた内容
```

***

##### 12. 単位と色

###### 12.1 単位

RPXは、数値直後の空白なしsuffix構文を提供できる。

```lisp
40mm
2s
30deg
```

概念的には型付きunit constructorの適用である。

```lisp
(mm 40)
```

具体的な単位、結果型、変換規則はpackageが定義する。

```text
mm:
length package

s:
time package

deg:
angle package

fps:
motion package
```

一般的なdimension algebraはv1の言語核へ導入しない。

`%`、`em`、device pixel等の文脈依存量は固定倍率の絶対単位と同一視しない。

物理表現、fixed-point精度、canonical unit等は単位package／runtime仕様で引き続き確定する。

***

###### 12.2 色

色を言語組込みliteralにしない。

```text
##ff0000:
RPX組込みliteralではない
```

color packageがconstructorを提供する。

```lisp
(srgb8 255 0 0)
```

```lisp
(srgb 1.0 0.0 0.0)
```

```lisp
(oklch lightness chroma hue)
```

CSS形式が必要ならpackage parserを使う。

```lisp
(parse-css-color "#ff0000")
```

色空間、alpha、ICC profile、gamut mapping、補間空間等はcolor packageとrender pipelineの責任とする。

***

##### 13. Source fileの宣言グループ

###### 13.1 Top-level

通常のsource fileのtop-levelは、順序付き宣言グループとする。

top-levelへ自由な実行式を置かない。

```lisp
(type title str)

(val title "Report")
```

effectfulな処理は、明示的な関数またはentry pointへ置く。

package import時に隠れたI/Oを発生させない。

***

###### 13.2 型注釈

値bindingへの型注釈は独立した宣言として書く。

```lisp
(type title str)

(val title "Report")
```

binder内の型注釈構文は導入しない。

```text
type annotation:
対応bindingより前

隣接:
必須ではない

同一declaration group:
必須

重複注釈:
禁止
```

通常implementation groupでは、対応するbindingを持たない型注釈はエラー。

***

###### 13.3 `val`

通常の`val`は逐次scopeを持つ。

```lisp
(val x 1)

(val y
  (+ x 1))
```

後方参照、自己参照は許可しない。

```lisp
(val y
  (+ x 1))

(val x 1)
```

は`x`未束縛。

注釈なしの`val`は許可し、型推論とvalue restrictionに従う。

***

###### 13.4 `rec`

自己再帰・相互再帰には明示的な`rec`宣言グループを使う。

```lisp
(rec
  (type even?
    (fn int bool))

  (type odd?
    (fn int bool))

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
```

規則：

```text
- group内のbindingは相互可視
- val右辺はelaboration後にfnでなければならない
- group検査中は単相
- polymorphic recursionは導入しない
- 検査完了後にvalue restrictionに従ってgeneralize可能
```

***

###### 13.5 `local`

局所宣言グループには`local`を使う。

```lisp
(local
  (val x 1)
  (val y (+ x 1))
  (+ x y))
```

文法：

```text
(local declaration* result-expression)
```

最後の式は必須。

内部的にはsequential `let`へ変換できる。

局所`rec`も許可する。

***

###### 13.6 重複binding

同じscopeでのduplicate binderは禁止。

```lisp
(val value 1)
(val value 2)
```

内側scopeでの通常shadowingは許可する。

top-level `var`は禁止する。

top-level `val` initializerはpureでなければならない。

***

##### 14. 関数

###### 14.1 値構文

関数値はparameter listを明示する。

```lisp
(fn (x y)
  (+ x y))
```

0引数：

```lisp
(fn ()
  unit)
```

***

###### 14.2 関数型

関数型には、引数型を囲う追加の括弧を置かない。

```lisp
(fn int int)
```

```text
引数:
int

戻り値:
int
```

```lisp
(fn int int int)
```

```text
引数:
int, int

戻り値:
int
```

effect付き：

```lisp
(fn str unit
  (effects console))
```

pure functionでは`effects`節を省略する。

***

###### 14.3 Fixed arity

```lisp
(fn int int int)
```

は2引数のfixed-arity functionであり、curried functionではない。

Curried functionは明示的に入れ子にする。

```lisp
(fn int
  (fn int int))
```

自動部分適用はv1では導入しない。

```text
引数不足:
arity error

引数過剰:
arity error
```

部分適用が必要なら明示的な`fn`を作る。

***

##### 15. List、tuple、record

###### 15.1 List

値：

```lisp
(list 1 2 3)
```

型：

```lisp
(list int)
```

空list：

```lisp
(list)
```

List値は不変。

異種要素から期待型なしで自動的なunion型を生成しない。異種listが必要なら型注釈を要求する。

***

###### 15.2 Tuple

値：

```lisp
(tuple 1 "title" true)
```

型：

```lisp
(tuple int str bool)
```

Tuple arityは2以上とする。

```text
0要素:
unitを使用

1要素:
元の型をそのまま使用
```

Tuple位置参照はcompile-timeに確定するindexを使う。

***

###### 15.3 Record

値：

```lisp
(record
  (title "Report")
  (page-count 10))
```

型：

```lisp
(record
  (title str)
  (page-count int))
```

field labelは通常変数ではなく、構造的なLabelIdとして扱う。

同じrecord内のduplicate fieldは禁止。

fieldのsource順序はCSTへ保持するが、recordのsemantic equalityはfield順序に依存しない。

***

###### 15.4 Field access

```lisp
(field report title)
```

`title`は評価される式ではなく静的field label。

***

###### 15.5 Updateとextension

既存fieldだけを書き換える。

```lisp
(record-update report
  (title "Updated"))
```

存在しないfieldだけを追加する。

```lisp
(record-extend report
  (author "Unknown"))
```

暗黙の上書きや、更新とextensionを混ぜるspread構文は導入しない。

***

##### 16. 型構文

###### 16.1 型適用

```lisp
(list int)
```

```lisp
(option str)
```

```lisp
(result str utf8-decode-error)
```

一般形：

```text
(type-constructor type-argument ...)
```

***

###### 16.2 `forall`

```lisp
(forall ((a type))
  (fn a a))
```

kind：

```text
type
record-row
effect-row
```

例：

```lisp
(forall ((a type)
         (e effect-row))
  (fn a a
    (effects e)))
```

***

###### 16.3 集合論的型

```lisp
(union int str)
```

```lisp
(intersect
  (fn int int)
  (fn str str))
```

```lisp
(not int)
```

```lisp
(diff number int)
```

基本的な正規化：

```text
(union)        ≃ never
(intersect)    ≃ any
(union t)      ≃ t
(intersect t)  ≃ t
```

***

###### 16.4 Dynamic

境界付きdynamic型を使用する。

```lisp
(dynamic any)
```

```lisp
(dynamic number)
```

裸の`dynamic`は正式型として必須にせず、必要ならaliasとして提供する。

***

###### 16.5 Open record

Closed record：

```lisp
(record
  (title str))
```

Open record：

```lisp
(record
  (title str)
  (row r))
```

`row`は最後に一つだけ置ける。

Optional field：

```lisp
(record
  (title str)
  (optional subtitle str))
```

これは次とは異なる。

```lisp
(record
  (title str)
  (subtitle (option str)))
```

***

###### 16.6 Effect row

```lisp
(fn str unit
  (effects console resource))
```

Effect application：

```lisp
(fn int int
  (effects (state int)))
```

Effect-row変数：

```lisp
(forall ((e effect-row))
  (fn str unit
    (effects console e)))
```

Surface上の同一effectの重複記述は禁止する。

空effect rowは省略する。明示的な`(effects)`は受理してもよいが、formatterは省略形へ正規化できる。

***

###### 16.7 型alias

```lisp
(type-alias report
  (record
    (title str)
    (page-count int)))
```

`type-alias`はtransparent aliasであり、新しいnominal identityを作らない。

Recursive type aliasはv1では禁止する。再帰dataは`data` declarationで定義する。

***

##### 17. `markup` reader

###### 17.1 位置づけ

文書専用の`doc`特殊形式は導入しない。

代わりに、構造化された文章・字幕・スライド本文・UI text等を記述する汎用reader形式として`markup`を導入する。

```lisp
(markup
  これは構造化された文章です。)
```

動画や図形全体を`markup`で記述する必要はない。

動画内の字幕、テロップ、title、説明文等で必要に応じて利用できる。

***

###### 17.2 Coreとpackageの分担

Core readerが知るもの：

```text
- markup領域の開始と終了
- text
- horizontal whitespace
- soft break
- paragraph break
- markup command
- code argument
- nested markup body
- embedded code
- source span
```

Core readerが知らないもの：

```text
- strongが太字か
- linkがURLか
- imageが何を表示するか
- headingが文書見出しか
- captionが動画字幕か
```

commandの意味と型はpackageが定義する。

***

###### 17.3 Markup command

基本形：

```lisp
@name
```

```lisp
@name[code-expression]
```

```lisp
@name(markup-body)
```

```lisp
@namemarkup-body
```

例：

```lisp
@page-number
```

```lisp
@space[20mm]
```

```lisp
@strong(重要)
```

```lisp
@link公式サイト
```

```lisp
@imageキャプション
```

***

###### 17.4 `[]`

角括弧内には通常のRPX code expressionをちょうど一つ書く。

```lisp
@image[logo]
```

```lisp
@link外部サイト
```

```lisp
@image[
  (record
    (source logo)
    (width 40mm))
](プロジェクトのロゴ)
```

空の`[]`は禁止。

複数の値が必要ならtupleまたはrecordへまとめる。

code引数は最大一つとする。

***

###### 17.5 `()`

`@name(...)`の丸括弧内は入れ子可能なmarkup bodyとして読む。

```lisp
@strong(
  この中には@term(専門用語)があります。
)
```

markup bodyは最大一つとする。

空bodyは許可する。

```lisp
@at()
```

本文なしの`@name`とは区別する。

***

###### 17.6 任意のcode埋込み

通常RPX codeを埋め込む場合は次を使う。

```lisp
@(show page-count)
```

```lisp
@(if detailed?
     detailed-content
     short-content)
```

`@(...)`内は通常のRPX S式として読む。

***

###### 17.7 糖衣展開

```lisp
@space[20mm]
```

は概念的に：

```lisp
(space 20mm)
```

へ変換される。

```lisp
@strong(重要)
```

は概念的に：

```lisp
(strong
  <markup-fragment "重要">)
```

へ変換される。

```lisp
@imageキャプション
```

は概念的に：

```lisp
(image
  path
  <markup-fragment "キャプション">)
```

へ変換される。

`@(space 20mm)`も意味上は同じ通常applicationを挿入できる。

正準styleでは、単純なmarkup commandは`@name[...]`形式を推奨する。

***

###### 17.8 `@at()`

文字としての`@`は、標準markup commandである`at`を使う。

```lisp
(markup
  user@at()example.com)
```

`@at()`は形式上、空のmarkup bodyを受け取る通常commandである。

reader専用の特殊escapeにはしない。

```text
@@:
不採用
```

`at`は標準markup packageまたはmarkup preludeが提供する。

```lisp
@code["@"]
```

は、`@`をcode表記として装飾する別用途である。

***

###### 17.9 丸括弧

Markup body内ではASCII丸括弧の対応を追跡する。

```lisp
@strong(
  RPX (prototype language) is statically typed.
)
```

対応の取れた括弧は通常textとして保持できる。

対応しない丸括弧を文章として示す必要がある場合、Core専用escapeは追加せず、`code`や`verbatim`等のpackage commandを使用する。

```lisp
@code[")"]
```

日本語の全角括弧`（ ）`は通常文字であり、delimiterではない。

***

###### 17.10 改行と空白

単一の物理改行は`SoftBreak`として保持する。

```lisp
(markup
  これは
  日本語です。)
```

Core readerは、改行をspaceへ変換するか削除するかを決めない。

空行は`ParagraphBreak`として保持する。

```lisp
(markup
  第一段落です。

  第二段落です。)
```

Horizontal whitespaceも早期に一つへ潰さず、範囲を保持する。

言語、用途、prose／verbatim等に応じた最終処理はpackageが行う。

***

###### 17.11 Markupの型

`markup`はexpected typeに応じてelaborate可能なsyntax formとする。

期待型がある場合：

```text
document-inline
caption-content
slide-heading
UI text
```

等のpackage固有型へelaborateできる。

期待型がない場合は、汎用的な`markup-fragment`型を既定とする。

Markup commandは通常のvalue bindingとして名前解決し、そのparameter型からmarkup bodyの期待型を得る。

***

##### 18. 構文エラー回復

###### 18.1 基本原則

構文エラー時にもlossless CSTを返す。

使用する回復node：

```text
MissingToken
UnexpectedToken
ErrorNode
```

parserは意味のある式や値を勝手に補わない。

局所的で確実な閉じdelimiterだけを仮想挿入できる。

***

###### 18.2 不足した閉じ括弧

EOF等で`)`が不足している場合、仮想`MissingToken`)\`を挿入する。

複数不足している場合はopen delimiter stackに基づいて補う。

診断は可能な限り一つにまとめ、各開始位置を関連情報として示す。

***

###### 18.3 余分な閉じ括弧

対応する`(`を持たない`)`は削除せず、`UnexpectedToken`として保存する。

そのtokenを意味解析から無視し、後続formの解析を続ける。

***

###### 18.4 未終了文字列

一行文字列が閉じられないまま改行へ到達した場合、行末へ仮想`"`を挿入する。

複数行文字列が未終了の場合は、EOFまでを文字列として保持し、EOFに仮想終了delimiterを置く。

複数行文字列の途中で通常codeへ勝手に復帰しない。

***

###### 18.5 未終了コメント

構造化コメントが未終了の場合、EOFまでをコメントとして保持し、EOFに仮想`)`を置く。

コメント内部に見える宣言へ勝手に復帰しない。

***

###### 18.6 Markupの`[]`

一つのcode expressionを正常に読み終えた後、`]`が不足していると判断できる場合は、その位置へ仮想`]`を挿入できる。

`[]`内に複数のcode expressionがある場合は、argument全体を`ErrorNode`にする。

最初の式だけを採用して残りを捨ててはならない。

診断では、recordまたはtupleへまとめる修正を提案できる。

***

###### 18.7 Markup body

Markup bodyの`)`が不足している場合、delimiter stackに従って最も内側の未終了formを処理する。

EOFまたは確実な外側同期点で仮想`)`を挿入する。

***

###### 18.8 不正な`@`

`@`の後にcommand名または`(`がない場合、`@`だけを`UnexpectedToken`として保持する。

後続textの解析は継続する。

診断例：

```text
expected a markup command name or `(` after `@`

examples:
  @strong(text)
  @(show value)

to write the `@` character:
  @at()
```

***

###### 18.9 未知command

```lisp
@strog(重要)
```

は構文としては正常。

未知command名はparser errorではなく、名前解決errorとして扱う。

```text
unbound markup command:
  strog

did you mean:
  strong
```

***

###### 18.10 型検査のcascade抑制

式位置の`ErrorNode`には内部専用のerror typeを与える。

error typeは任意の期待型へ暫定適合できるが、正常な型として外部へ公開しない。

これにより、一つのsyntax errorから大量の二次的型エラーが発生することを防ぐ。

***

###### 18.11 Formatter

構文エラーを含むsourceについて、formatterは仮想tokenを勝手に実体化しない。

```text
正常node:
format可能

ErrorNode:
元bytesを保持

MissingToken:
sourceへ自動挿入しない
```

修正は明示的なcode actionとして提供できる。

```text
Quick fix:
Insert missing `)`
```

***

##### 19. 適合例

```lisp
##!/usr/bin/env rpx

(// Report definition)

(type report-title str)

(val report-title
  "進捗報告")

(type make-heading
  (fn str markup-fragment))

(val make-heading
  (fn (title)
    (markup
      @strong(@(show title)))))

(type report-options
  (record
    (title str)
    (page-size page-size)
    (optional subtitle str)))

(val options
  (record
    (title report-title)
    (page-size a4)))

(type body markup-fragment)

(val body
  (markup
    @heading(概要)

    この文書は@strong(型付き)の制作言語で生成されます。

    詳細は@link仕様書を参照してください。

    @image[
      (record
        (source architecture-diagram)
        (width 120mm))
    ](RPXの処理pipeline)

    連絡先はuser@at()example.comです。))
```

***

##### 20. 不適合例

###### 不要な先頭ゼロ

```lisp
(val value 007)
```

###### 大文字開始識別子

```lisp
(val Report "...")
```

###### Underscoreを含む識別子

```lisp
(val report_title "...")
```

###### 重複field

```lisp
(record
  (title "First")
  (title "Second"))
```

###### 1要素tuple

```lisp
(tuple 1)
```

###### 自動部分適用

```lisp
(val increment
  (add 1))
```

`add`が2引数fixed-arity関数ならarity error。

###### Markup引数内の複数式

```lisp
@imageキャプション
```

###### 文字としての`@`に`@@`を使用

```lisp
user@@example.com
```

`@@`は正式escapeではない。次を使う。

```lisp
user@at()example.com
```

***

##### 21. 本項目で意図的に確定しない事項

`OPEN-SYN-002`の外へ回す事項は次のとおり。

```text
- data／variantの最終構文
- patternとmatch
- constructor pattern
- exhaustiveness
- module、signature、functor
- import／export
- package manifest
- named／optional function argument
- variadic function
-一般的な部分適用糖衣
- 利用者定義operator
- 一般的なdimension algebra
- 単位の正確な内部表現
- fixed-point精度
- color managementの詳細
- ICC profile
- stream API
- resource declarationの最終構文
- markup package固有command
- markup-fragmentから各domain型への具体的elaboration protocol
```

***

##### 22. 状態

```text
OPEN-SYN-002:
RESOLVED
```

下位項目：

```text
OPEN-SYN-002A
文字コード・空白・コメント
→ RESOLVED

OPEN-SYN-002B
識別子・予約形式
→ RESOLVED

OPEN-SYN-002C
リテラル
→ RESOLVED

OPEN-SYN-002D
宣言グループ
→ RESOLVED

OPEN-SYN-002E
型構文
→ RESOLVED

OPEN-SYN-002F
markup reader
→ RESOLVED

OPEN-SYN-002G
構文エラー回復
→ RESOLVED
```

#### 13.2.1 `RES-001` 名前解決とnamespace

##### 概要・状態

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

#### 13.2.3 `DAT-001` 代数的データ型・constructor・pattern・match

##### DD-001 決定概要

###### DD-001.1 状態

```text
Status:
RESOLVED

Scope:
代数的データ型
値constructor
constructor固有型
pattern
match
網羅性検査
再帰data型
相互再帰data型
抽象化境界
```

###### DD-001.2 中心的な決定

```text
- 代数的データ型の宣言には`data`を使う
- `type`は値bindingへの型注釈として維持する
- `type-alias`はtransparent aliasに使う
- data型はsealedである
- constructorは通常のvalue namespaceに置く
- constructor固有型を型namespaceに生成する
- constructor適用はconstructor固有型を返す
- constructor固有型は親data型のsubtypeである
- match caseは`pattern -> expression`形式で書く
- 非網羅matchと到達不能caseは静的エラーにする
- 再帰data型にはstrict positivityを要求する
- 通常のdata値は不変、strict、非循環である
```

###### DD-001.3 宣言名

通常のADTには`data`を使用する。

```lisp
(data option
  ((a type))

  none
  (some a))
```

`adt`は予約語にしない。GADTを導入する場合も、原則として`data`宣言の拡張として設計する。

```text
type:
値bindingへの型注釈

type-alias:
transparentな型alias

data:
sealed nominal ADT
```

***

##### 0. 適用範囲

###### 0.1 本項目が定めるもの

本項目は次を規定する。

```text
- data宣言のSurface構文
- 型parameter
- value constructor
- constructor固有型
- constructorの型parameter推論
- recursive data
- mutually recursive data
- patternの種類
- matchの構文
- patternによるbinding
- 型精緻化
- exhaustiveness
- unreachable case
- sealed constructor集合
- abstract export時の意味
- data値の評価と不変性
```

###### 0.2 本項目が定めないもの

次は別のOPEN項目へ移管する。

```text
- module signatureの最終構文
- import／export構文
- GADT
- existential constructor
- open variant
- derive
- 自動等値性
- serialization
- runtime reflection
- proper tail call
- ownership／borrow
- lazy value
- mutable cyclic graph
```

***

##### 1. `data`宣言

###### 1.1 Parameterを持たないdata型

```lisp
(data alignment
  left
  center
  right
  justify)
```

これは新しいnominal型`alignment`と、4つのconstructorを定義する。

###### 1.2 Parameterを持つdata型

```lisp
(data option
  ((a type))

  none
  (some a))
```

Parameter節では、parameter名とkindを組にする。

```text
(a type)
```

複数parameterの例：

```lisp
(data result
  ((value-type type)
   (error-type type))

  (ok value-type)
  (err error-type))
```

###### 1.3 Parameterなしの場合

Parameterがないdata宣言では、空のparameter節を書かない。

適合：

```lisp
(data alignment
  left
  center
  right)
```

不適合：

```lisp
(data alignment
  ()

  left
  center
  right)
```

###### 1.4 Constructor payload

Constructorは0個以上の位置payload型を持てる。

```lisp
(data shape
  (circle point length)
  (rectangle point size)
  (path path-data))
```

複雑なpayloadについては、1個のrecord型をpayloadとして使用できる。

```lisp
(data shape
  (circle
    (record
      (center point)
      (radius length))))
```

***

##### 2. `data`宣言が生成するbinding

###### 2.1 親type constructor

```lisp
(data option
  ((a type))

  none
  (some a))
```

は型constructor`option`を生成する。

```text
option:
type -> type
```

具体型：

```lisp
(option int)
```

```lisp
(option str)
```

###### 2.2 Value constructor

値namespaceには次を生成する。

```text
none
some
```

概念型：

```text
none:
forall a.
option<a>

some:
forall a.
a -> option<a>
```

###### 2.3 Constructor固有型

型namespaceにはconstructorごとのcase typeを生成する。

```lisp
(none int)
```

```lisp
(some int)
```

関係：

```text
(none int) <: (option int)
(some int) <: (option int)
```

###### 2.4 Sealed constructor集合

`data`宣言は、網羅性検査に使用するsealed constructor集合を生成する。

```text
option<a>
=
none<a>
または
some<a>
```

他packageはconstructorを追加できない。

###### 2.5 Runtime reflection

`data`宣言は、通常のRPX値としてruntime type descriptorを自動生成しない。

次のような機能は別項目とする。

```lisp
(type-of-value value)
```

```lisp
(constructors-of option)
```

***

##### 3. 名前とnamespace

###### 3.1 型namespaceと値namespace

Constructor名は値位置と型位置で異なる意味を持つ。

値位置：

```lisp
(some 42)
```

型位置：

```lisp
(some int)
```

値位置ではconstructor application、型位置ではconstructor固有型である。

###### 3.2 型名とconstructor名の同名禁止

次は拒否する。

```lisp
(data box
  ((a type))

  (box a))
```

型位置の`(box int)`が親型とcase typeのどちらか判別できないためである。

代わりに異なる名前を使う。

```lisp
(data boxed
  ((a type))

  (box a))
```

###### 3.3 Constructor名の重複

同じdata宣言内で同じconstructor名を複数回定義できない。

```lisp
(data result
  (ok int)
  (ok str))
```

これはoverloadではなくduplicate constructor errorとする。

###### 3.4 Constructor identity

Constructorは文字列ではなく`ConstructorId`によって識別する。

異なるpackageの同名constructorは異なるidentityを持つ。

```text
graphics/circle
diagram/circle
```

***

##### 4. Nullary constructor

###### 4.1 宣言

Payloadを持たないconstructorは裸のidentifierで宣言する。

```lisp
(data alignment
  left
  center
  right)
```

###### 4.2 値

Nullary constructorは値そのものである。

```lisp
left
```

次のようには書かない。

```lisp
(left)
```

`(left)`は0引数関数適用と解釈される。

###### 4.3 関数ではない

Nullary constructorは0引数関数ではない。

常に`none`を返す関数が必要なら明示的に書く。

```lisp
(fn (_)
  none)
```

***

##### 5. Payload constructor

###### 5.1 値構築

```lisp
(circle center 20mm)
```

Constructorは通常のfixed-arity pure function valueとして使用できる。

###### 5.2 Constructorの高階利用

Payload constructorは高階関数へ渡せる。

```lisp
(map some values)
```

`values : list<int>`なら、`some`は使用位置で次へinstance化される。

```text
int -> option<int>
```

###### 5.3 Arity

宣言されたpayload数と適用時の引数数は一致しなければならない。

```lisp
(data shape
  (circle point length))
```

適合：

```lisp
(circle center radius)
```

不適合：

```lisp
(circle center)
```

```lisp
(circle center radius extra)
```

***

##### 6. Constructor固有型

###### 6.1 精密な結果型

Constructor適用は親型ではなく、constructor固有型をprincipal typeとして返す。

```lisp
(some 42)
```

精密型：

```lisp
(some int)
```

親型：

```lisp
(option int)
```

###### 6.2 Subtyping

```text
some<int> <: option<int>
none<int> <: option<int>
```

したがって、親型を要求する場所へ明示変換なしで渡せる。

###### 6.3 親型とのsealed union関係

Constructorが可視なscopeでは、親型をconstructor固有型のsealed unionとして扱う。

```text
option<int>
≃
union(
  none<int>,
  some<int>)
```

###### 6.4 表示上の単純化

内部推論ではconstructor固有型を保持するが、通常のhoverや公開APIでは親型へ簡略表示できる。

```text
内部:
some<int>

通常表示:
option<int>

詳細表示:
option<int>, refined as `some`
```

***

##### 7. 型parameter推論

###### 7.1 Payloadからの推論

```lisp
(some 42)
```

から：

```text
a = int
```

を推論する。

###### 7.2 期待型からの推論

```lisp
(type value
  (option str))

(val value
  none)
```

期待型から：

```text
none<str>
```

としてinstance化する。

###### 7.3 一部未確定のparameter

```lisp
(ok 42)
```

では成功値型は`int`と分かるが、error型は未確定である。

Pureな`val`なら未確定parameterをgeneralizeできる。

```lisp
(val success
  (ok 42))
```

概念型：

```text
forall error-type.
result<int, error-type>
```

###### 7.4 Value restriction

Mutableまたはgeneralize不能な位置では、未確定parameterを量化しない。

不適合：

```lisp
(local
  (var result
    (ok 42))

  result)
```

`error-type`を決定できないため、型注釈を要求する。

適合：

```lisp
(local
  (type result
    (result int render-error))

  (var result
    (ok 42))

  result)
```

###### 7.5 値位置の明示型argument

Constructorへ値位置で明示的な型argumentを渡す構文は導入しない。

型を指定する場合は、期待型または型注釈を使う。

***

##### 8. Variance

###### 8.1 自動推論

Varianceは利用者が記述せず、constructor payloadにおける型parameterの出現位置から自動推論する。

###### 8.2 共変

```lisp
(data option
  ((a type))

  none
  (some a))
```

`a`は正の位置だけに現れるため共変である。

```text
cat <: animal
なら
option<cat> <: option<animal>
```

###### 8.3 反変

```lisp
(data consumer
  ((a type))

  (consumer
    (fn a unit)))
```

`a`は関数引数位置に現れるため反変である。

###### 8.4 不変

```lisp
(data channel
  ((a type))

  (channel
    (fn unit a)
    (fn a unit)))
```

`a`は正と負の両方へ現れるため不変である。

###### 8.5 Phantom parameter

Constructor payloadへ現れないparameterはphantomとして扱う。

```lisp
(data identifier
  ((domain type))

  (identifier int))
```

Phantom parameterは許可するがwarning対象とする。

***

##### 9. 再帰data型

###### 9.1 自己再帰

```lisp
(data tree
  ((a type))

  empty

  (node
    a
    (tree a)
    (tree a)))
```

定義中の型名をconstructor payloadから参照できる。

###### 9.2 Strict positivity

定義中のdata型はstrictly positiveな位置にのみ出現できる。

適合：

```lisp
(data tree
  ((a type))

  empty
  (node a (tree a) (tree a)))
```

不適合：

```lisp
(data bad
  (bad
    (fn bad unit)))
```

`bad`が関数引数位置に現れるため、strict positivity違反である。

###### 9.3 関数戻り値位置

定義中の型が関数戻り値に現れることは、positivity上は許可できる。

```lisp
(data stream-node
  ((a type))

  (stream-node
    a
    (fn unit (stream-node a))))
```

ただし、serializationや自動deriveの可否は別に判断する。

***

##### 10. 相互再帰data型

###### 10.1 `rec` group

相互再帰data型は、明示的な`rec` groupで宣言する。

```lisp
(rec
  (data expression
    (literal int)
    (sequence (list statement)))

  (data statement
    (evaluate expression)
    (return expression)))
```

###### 10.2 Group内の可視性

同じ`rec` group内のdata型名は相互に可視である。

###### 10.3 宣言kindの混在禁止

同じ`rec` groupへ`data`と`val`を混在させない。

適合：

```lisp
(rec
  (data first ...)
  (data second ...))
```

適合：

```lisp
(rec
  (val first
    (fn (...) ...))

  (val second
    (fn (...) ...)))
```

不適合：

```lisp
(rec
  (data first ...)
  (val second
    (fn (...) ...)))
```

###### 10.4 Group全体のpositivity

Mutually recursive groupでは、group内の全data型についてまとめてstrict positivityを検査する。

***

##### 11. Data値の実行意味

###### 11.1 不変性

Data値は常に不変である。

Constructor payloadを後から破壊的に書き換えることはできない。

###### 11.2 Strict評価

Constructor payloadは、constructor値を構築する前に左から右へ評価する。

```lisp
(node
  (compute-root)
  (compute-left)
  (compute-right))
```

評価順序：

```text
1. compute-root
2. compute-left
3. compute-right
4. node構築
```

###### 11.3 Effect

Constructor自体はpureである。

Constructor適用式のeffectは、payload式のeffectから生じる。

###### 11.4 途中失敗

Payload評価が途中で失敗した場合、残りのpayloadを評価せず、data値も構築しない。

***

##### 12. 非循環性とsharing

###### 12.1 Cyclic data値

通常のSurface codeから、自己参照するcyclic data値を構築できない。

不適合：

```lisp
(rec
  (val infinite
    (node 1 infinite infinite)))
```

Value-level `rec`の右辺は`fn`に限る。

###### 12.2 Structural sharing

完成済みの不変部分値は共有できる。

```lisp
(local
  (val subtree
    (node 5 empty empty))

  (node 10 subtree subtree))
```

###### 12.3 Sharingの非観測性

Structural sharingの有無をpointer identityとして観測できない。

一般的な`physical-equal?`をdata値へ提供しない。

###### 12.4 無限構造

無限stream等はcyclic dataではなく、thunkや関数を明示的に含む有限値として表現する。

```lisp
(data stream
  ((a type))

  (stream
    a
    (fn unit (stream a))))
```

***

##### 13. Patternの適用場所

###### 13.1 v1の制限

Patternは`match` case内で使用する。

`val` binderや関数parameterへ一般patternを導入しない。

適合：

```lisp
(match pair
  (tuple first second ->
    (+ first second)))
```

不採用：

```lisp
(val (tuple first second)
  pair)
```

```lisp
(fn ((tuple first second))
  (+ first second))
```

###### 13.2 Binder

Constructor payload位置の裸identifierはbinderとして扱う。

```lisp
(some item ->
  item)
```

`item`はcase body内だけで有効である。

###### 13.3 Wildcard

```lisp
_
```

は任意値へ一致し、bindingを作らない。

###### 13.4 Duplicate binder

同じpattern内で同じ名前を複数回束縛できない。

```lisp
(tuple value value ->
  ...)
```

は静的エラー。

`_`は複数回使用可能である。

***

##### 14. `match`構文

###### 14.1 基本構文

```lisp
(match subject-expression
  (pattern ->
    result-expression)

  (pattern ->
    result-expression))
```

###### 14.2 Payload constructor

```lisp
(match value
  (some item ->
    item)

  (none ->
    default))
```

###### 14.3 複数payload

```lisp
(match shape-value
  (circle center radius ->
    (render-circle center radius))

  (rectangle origin rectangle-size ->
    (render-rectangle origin rectangle-size))

  (path data ->
    (render-path data)))
```

###### 14.4 `->`

`->`はmatch branch専用の予約tokenである。

通常operatorではなく、値として参照できない。

###### 14.5 Case body

矢印の右側には一つの式だけを書く。

複数処理は`seq`または`local`でまとめる。

```lisp
(some item ->
  (seq
    (log item)
    (render item)))
```

***

##### 15. Patternの種類

###### 15.1 Nullary constructor pattern

```lisp
(none ->
  default)
```

Case先頭の裸identifierはnullary constructorとして解決する。

###### 15.2 Payload constructor pattern

```lisp
(some item ->
  item)
```

Constructor headは`ConstructorId`として名前解決する。

###### 15.3 Wildcard pattern

```lisp
(_ ->
  fallback)
```

###### 15.4 Catch-all binder

対象値全体を束縛する場合は`bind` patternを使う。

```lisp
(bind value ->
  (handle-other value))
```

`bind`はpattern特殊形式とする。

###### 15.5 Literal pattern

v1では次を許可する。

```text
整数literal
文字列literal
true
false
unit
```

例：

```lisp
(match format
  ("pdf" ->
    render-pdf)

  ("svg" ->
    render-svg))
```

`f64` literal patternは導入しない。

***

##### 16. Nested pattern

###### 16.1 Nested payload constructor

```lisp
(match value
  (some (some item) ->
    item)

  (some inner ->
    fallback)

  (none ->
    default))
```

###### 16.2 Nested tuple

```lisp
(match value
  (some (tuple first second) ->
    (+ first second))

  (none ->
    0))
```

###### 16.3 Nested nullary constructor

Constructor payload位置の裸identifierはbinderとして読むため、nested nullary constructor patternはv1では直接提供しない。

次のようにnested `match`を使用する。

```lisp
(match outer
  (some inner ->
    (match inner
      (none ->
        first-result)

      (some value ->
        second-result)))

  (none ->
    outer-default))
```

***

##### 17. Tuple pattern

###### 17.1 基本形

```lisp
(match pair
  (tuple first second ->
    (+ first second)))
```

###### 17.2 Arity

Pattern arityは入力tuple型と一致しなければならない。

###### 17.3 Binding型

入力型：

```lisp
(tuple int str)
```

Pattern：

```lisp
tuple count title
```

Binding：

```text
count : int
title : str
```

###### 17.4 Refutability

Tuple自体の型が一致していても、内部にrefutable patternがあればtuple pattern全体もrefutableである。

***

##### 18. Record pattern

###### 18.1 基本形

```lisp
(match report
  (record
    (title title)
    (page-count count)
  ->
    (render-summary title count)))
```

###### 18.2 Partial decomposition

指定したrequired fieldだけを分解する。

追加fieldは許可する。

```lisp
(record
  (title title))
```

は、`title`以外のfieldを持つrecordにも一致する。

###### 18.3 Closed pattern

Field集合の完全一致を要求するclosed record patternはv1では導入しない。

###### 18.4 Required fieldのみ

Patternへ書けるのは、入力型で存在と型が静的に保証されたrequired fieldだけである。

Optional fieldは直接分解しない。

###### 18.5 Optional field

Optional fieldはfield access結果を`option`としてmatchする。

```lisp
(match (field report subtitle)
  (none ->
    no-subtitle)

  (some subtitle ->
    (render-subtitle subtitle)))
```

###### 18.6 Unknown row field

Open row内にあるか不明なfieldをrecord patternへ書けない。

Binder型を静的に決定できないためである。

***

##### 19. Pattern typing

###### 19.1 三つの結果

Pattern checkerは各patternから次を計算する。

```text
Covered:
そのpatternが覆う型

Bindings:
case bodyへ導入される名前と型

Residual:
一致しなかった値の型
```

###### 19.2 Constructor pattern

入力：

```lisp
(option int)
```

Pattern：

```lisp
some item
```

結果：

```text
Covered:
(some int)

Bindings:
item : int

Residual:
(none int)
```

###### 19.3 Literal pattern

入力：

```lisp
(union "pdf" "svg")
```

Pattern：

```lisp
"pdf"
```

結果：

```text
Covered:
"pdf"

Residual:
"svg"
```

###### 19.4 Wildcard

```text
Covered:
入力型全体

Bindings:
なし

Residual:
never
```

###### 19.5 Disjoint pattern

入力型とのintersectionが`never`なら、そのcaseは静的に到達不能である。

```text
this pattern cannot match the input type
```

***

##### 20. `match`の型・effect・評価

###### 20.1 対象式

`match`対象式は一度だけ評価する。

###### 20.2 Case順序

Caseをsource orderで検査し、最初に一致したcaseを選択する。

###### 20.3 結果型

期待型がある場合、各到達可能case bodyをその型で検査する。

期待型がない場合、到達可能case body型のunionを結果型とする。

```lisp
(match value
  (none ->
    "unknown")

  (some item ->
    item))
```

`item : int`なら結果型は：

```lisp
(union str int)
```

###### 20.4 `never`

`never`を返すcaseは、他branchの結果型を汚染しない。

```text
union(never, t) ≃ t
```

###### 20.5 Effect

`match`全体のeffectは次の和である。

```text
対象式のeffect
+
全到達可能case bodyのeffect
```

Pattern照合自体はpureとする。

***

##### 21. 網羅性・公開・適合試験

###### 21.1 網羅性

Case処理前の残余型を対象型とする。

各caseについて：

```text
case-space =
intersection(remaining, pattern-type)
```

処理後：

```text
remaining =
diff(remaining, pattern-type)
```

最後に：

```text
remaining = never
```

なら網羅的である。

###### 21.2 非網羅match

非網羅matchは静的エラー。

```lisp
(match value
  (some item ->
    item))
```

診断例：

```text
non-exhaustive match

missing case:
  none
```

###### 21.3 到達不能case

既に前のcaseで覆われたcaseは静的エラー。

```lisp
(match value
  (_ ->
    default)

  (some item ->
    item))
```

###### 21.4 空match

Caseを持たない`match`は、対象型が`never`の場合に限って許可する。

###### 21.5 Sealed data

通常の`data`はsealedとする。

他packageからconstructorを追加できない。

公開data型へのconstructor追加はbreaking changeとして扱う。

###### 21.6 Transparent export

型と全constructorを公開する。

外部利用者は構築、pattern match、網羅性検査を行える。

###### 21.7 Abstract export

親型identityだけを公開し、constructorをすべて隠せる。

外部利用者は内部constructorでmatchできない。

###### 21.8 一部constructor公開

一部constructorだけの公開はv1では導入しない。

###### 21.9 適合試験 ADT-01

```lisp
(data option
  ((a type))

  none
  (some a))

(type value
  (option int))

(val value
  (some 42))
```

期待結果：

```text
success
valueの内部精密型:
some<int>

指定型:
option<int>
```

###### 21.10 適合試験 ADT-02

```lisp
(match value
  (some item ->
    item)

  (none ->
    0))
```

期待結果：

```text
success
result type:
int
```

###### 21.11 適合試験 ADT-03

```lisp
(match value
  (some item ->
    item))
```

期待結果：

```text
static error:
non-exhaustive match

missing:
none
```

###### 21.12 適合試験 ADT-04

```lisp
(match value
  (_ ->
    0)

  (some item ->
    item))
```

期待結果：

```text
static error:
unreachable match case
```

###### 21.13 適合試験 ADT-05

```lisp
(data tree
  ((a type))

  empty
  (node a (tree a) (tree a)))
```

期待結果：

```text
success
recursive occurrence is strictly positive
```

###### 21.14 不適合試験 ADT-06

```lisp
(data bad
  (bad
    (fn bad unit)))
```

期待結果：

```text
static error:
recursive occurrence of `bad` is not strictly positive
```

###### 21.15 適合試験 ADT-07

```lisp
(val success
  (ok 42))
```

期待結果：

```text
success
generalized type:
forall e. result<int, e>
```

###### 21.16 不適合試験 ADT-08

```lisp
(local
  (var success
    (ok 42))

  success)
```

期待結果：

```text
static error:
cannot infer ungeneralized error type

suggestion:
add a type annotation
```

###### 21.17 適合試験 ADT-09

```lisp
(match report
  (record
    (title title)
  ->
    title))
```

入力型：

```lisp
(record
  (title str)
  (page-count int))
```

期待結果：

```text
success
title : str
```

###### 21.18 不適合試験 ADT-10

```lisp
(match report
  (record
    (subtitle subtitle)
  ->
    subtitle))
```

`subtitle`がoptional fieldの場合の期待結果：

```text
static error:
optional fields cannot be directly decomposed by record patterns

suggestion:
match the result of field access
```

***

##### 22. 移管先OPEN・下位項目・状態

###### 22.1 `OPEN-SYN-002`への追補

`->`をmatch branch separator用の予約tokenとして追補する。

```text
->
```

は通常operator identifierではない。

###### 22.2 `OPEN-MOD-001`

次を移管する。

```text
- data型のexport構文
- transparent export
- abstract export
- constructor import
- package qualification
- signature内のabstract type
- module境界でのcase型可視性
```

###### 22.3 `OPEN-SEM-001`

次を移管する。

```text
- proper tail call
- 一般再帰のstack safety
- stack overflowのfailure分類
- 関数呼出しstackの観測性
```

###### 22.4 `OPEN-DERIVE-001`

次を移管する。

```text
- 構造的等値性の自動導出
- hash
- ordering
- debug表示
- serialization
- clone／copy能力
```

###### 22.5 `OPEN-GADT-001`

次を将来項目として移管する。

```text
- constructorごとのresult type
- GADT
- existential constructor parameter
- pattern matchによる型等式導入
- typed AST
```

宣言名として`gadt`を新設するか、`data`を拡張するかはこの項目で決める。

###### 22.6 `OPEN-DYNAMIC-001`

次を移管する。

```text
- anyからのruntime type test
- dynamic cast
- dynamic値へのconstructor pattern
- cast failure
- RuntimeCheckable制約
```

`match`自体へ暗黙のdynamic castを含めない。

###### 22.7 `OPEN-LAZY-001`

次を移管する。

```text
- lazy
- force
- memoized thunk
- 無限stream
- 内部mutationを伴う遅延評価
```

###### 22.8 `OPEN-GRAPH-001`

必要になった場合に次を移管する。

```text
- cyclic graph
- node identity
- graph serialization
- mutable graph
- cycle-aware traversal
```

通常data値の循環参照としては実装しない。

###### 22.9 下位項目

```text
OPEN-DAT-001A
data宣言と型parameter
→ RESOLVED

OPEN-DAT-001B
constructorとconstructor固有型
→ RESOLVED

OPEN-DAT-001C
varianceとphantom parameter
→ RESOLVED

OPEN-DAT-001D
recursive／mutually recursive data
→ RESOLVED

OPEN-DAT-001E
pattern構文とpattern typing
→ RESOLVED

OPEN-DAT-001F
match構文・型・effect
→ RESOLVED

OPEN-DAT-001G
exhaustivenessとunreachable case
→ RESOLVED

OPEN-DAT-001H
sealed／abstract data境界
→ RESOLVED

OPEN-DAT-001I
runtime data semantics
→ RESOLVED
```

###### 22.10 最終状態

```text
OPEN-DAT-001:
RESOLVED
```

本解決により、RPXは次を型安全に表現できる。

```text
- 列挙型
- option／result
- recursive tree
- mutually recursive AST
- constructor固有型
- constructorによる型精緻化
- 網羅的pattern matching
- abstract data representation
- immutable structural sharing
```

#### 13.3 `EVAL-001` Strict lexical Core evaluator

> 統合元: `OPEN-EVAL-001`。同項目は解決済み。

##### 状態

`解決済み`

本決定は、RPXの最小Core v1について、項、値、評価順序、逐次評価および
評価文脈を固定するものである。

`letrec`、pattern match、record、effect handler、局所可変状態、dynamic cast、
primitive固有の失敗は最小Core v1には含めず、それぞれ別の設計事項として扱う。

---

##### `DD-EVAL-001`: 評価戦略と評価順序

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

##### `DD-EVAL-002`: 最小Coreの項

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

##### `DD-EVAL-003`: `let`

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

##### `DD-EVAL-004`: 複数式bodyと`seq`

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

##### `DD-EVAL-005`: `if`

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

##### `DD-TYP-IF-001`: 条件分岐による型の絞り込み

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

##### 値

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

##### Closureとlexical scope

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

##### 関数適用

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

##### 評価文脈

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

##### 最小Coreの終端状態

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

##### 未決定事項の移管

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

##### 適合試験

###### lexical closure

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

###### sequential `let`

```lisp
(let ((x 1)
      (y (+ x 1)))
  y)
```

期待結果:

```text
2
```

###### duplicate binder

```lisp
(let ((x 1)
      (x 2))
  x)
```

期待結果:

```text
静的エラー: 同一Surface let内のduplicate binder
```

###### application order

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

###### sequence result

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

###### selected branch only

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

###### strict Bool condition

```lisp
(if 1
    10
    20)
```

期待結果:

```text
静的エラー: expected Bool, found Integer
```

###### union result

```lisp
(if condition
    42
    "unknown")
```

期待される型:

```text
union(Integer, String)
```

###### occurrence typing

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

##### 解決後の最小Core

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

#### 13.4 `BND-001` `val`、`var`、`let`、`letrec`、`fn`

> 統合元: `OPEN-BND-001`。同項目は解決済み。

##### 状態

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

##### `DD-BND-001`: 通常の不変束縛

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

##### `DD-BND-002`: sequentialなSurface `let`

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

##### `DD-BND-003`: `(type ...)`による型注釈

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

##### `DD-BND-004`: 型注釈のscopeと対応関係

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

##### `DD-BND-005`: 型注釈は検査される

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

##### `DD-BND-006`: `letrec`の対象

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

##### `DD-BND-007`: `letrec`内の型注釈

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

##### `DD-BND-008`: `letrec`の実行意味

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

##### `DD-BND-009`: 相互再帰

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

##### `DD-BND-010`: 再帰関数の型推論

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

##### `DD-BND-011`: 多相再帰の禁止

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

##### `DD-BND-012`: `var`の基本意味

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

##### `DD-BND-013`: `var`の型注釈

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

##### `DD-BND-014`: `var`の格納型は固定

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

##### `DD-BND-015`: `var`の読出し

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

##### `DD-BND-016`: `set`

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

##### `DD-BND-017`: closureによる`var`のcapture

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

##### `DD-BND-018`: local state identity

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

##### `DD-BND-019`: local state escapeの禁止

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

##### `DD-BND-020`: non-escaping callback

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

##### `DD-BND-021`: local state effectの除去

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

##### `DD-BND-022`: one-shot resumptionと局所state

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

##### `DD-BND-023`: escape可能な状態との分離

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

##### `DD-BND-024`: 型一般化point

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

##### `DD-BND-025`: 構文的value restriction

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

##### `DD-BND-026`: effectful function valueの一般化

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

##### `DD-BND-027`: capability captureによる一般化禁止

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

##### `DD-BND-028`: 明示的`forall`注釈

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

##### `DD-BND-029`: 一般化される変数

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

##### `DD-BND-030`: 一般化されないmetavariable

`確定`

一般化不可のbindingに未解決metavariableが存在する場合、それらを
monomorphicなweak metavariableとして扱う。

同一bindingのすべての使用箇所で同じmetavariableを共有する。

最初の使用で型が確定した後は、別の型へ再instantiateできない。

明示的な`(type ...)`宣言を後から追加して、既に別の型へ確定したweak metavariableを
不整合に一般化してはならない。

***

##### `DD-BND-031`: `letrec`とvalue restriction

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

##### `DD-BND-032`: relaxed value restrictionの保留

`確定`

RPX v1はrelaxed value restrictionを導入しない。

varianceまたは型変数の出現位置に基づいて、expansive expressionの一部を追加で一般化することは
行わない。

必要性が確認された場合は、`OPEN-TYP-002`でsoundness、decidabilityおよび
semantic subtypingとの相互作用を検証したうえで将来拡張として検討する。

***

##### Coreおよびdeclaration elaboration

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

##### 適合試験

###### top-level型注釈

```lisp
(type radius length)

(val radius 40mm)
```

期待：

```text
受理
radius : length
```

###### 関数型注釈

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

###### 注釈不一致

```lisp
(type value str)

(val value 42)
```

期待：

```text
静的エラー
```

###### 対応bindingのない型注釈

```lisp
(type missing int)
```

期待：

```text
静的エラー:
型注釈に対応する値bindingがない
```

###### 自己再帰の型注釈

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

###### 相互再帰の型注釈

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

###### 一部だけ注釈

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

###### 任意式の再帰拒否

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

###### 明示的多相型

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

###### 注釈によるvalue restriction回避の拒否

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

###### local `var`

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

###### `set`の戻り値

```lisp
(var count 0
  (set count 1))
```

期待される結果型：

```text
unit
```

###### 格納型の固定

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

###### 明示union格納型

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

###### closure capture

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

###### closure escape

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

###### effectful関数値の一般化

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

##### 未解決事項の移管

###### `OPEN-SYN-002`

* top-levelおよび局所declaration groupの完全な構文
* `letrec`内で`type`と`val`を並べる正確な括弧構造
* `var`と対応する`type`宣言を置く局所構文
* `val (f x)` sugarの完全な構文
* `forall`、kind annotation、effect rowの表面構文
* declaration orderingとforward annotation
* duplicate annotation diagnostic
* `type` annotationとtype alias declarationの構文上の区別

###### `OPEN-TYP-002`

* 注釈付き／無注釈再帰グループのalgorithm
* annotation subsumption
* explicit `forall`のchecking
* annotationとprincipal typingの関係
* non-escaping callbackの判定
* generalizable syntactic valueの正確なalgorithmic分類
* semantic subtyping下のgeneralization
* checker limitationによる注釈要求

###### `OPEN-MEM-001`

* escape可能な`cell`／`ref`
* heap allocation
* closure環境のmemory管理
* shared mutable cell
* cyclic state
* stateful closureのidentity
* GUI再評価をまたぐstate lifetime

###### `OPEN-MOD-001`

* public valueへの`(type ...)`注釈要件
* signature内の`type`とvalue annotationの区別
* module間相互再帰
* recursive module
* exported closureのscope検査

###### 将来拡張

* 多相再帰
* lazy recursive value
* recursive record
* recursive handler value
* affine kind system
* 一般的なborrowed closure
* relaxed value restriction
* multi-shot state semantics

***

##### 解決後の基本原則

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

#### 13.5 `MAC-001` 最小式マクロ・展開・衛生性
##### DD-001 決定概要
###### DD-001.1 状態
Status:
RESOLVED

Scope:
利用者定義マクロ
式マクロ
マクロ展開段階
マクロパターン
末尾反復
名前衝突防止
展開順序
展開上限
マクロのscope
展開結果の由来情報

###### DD-001.2 設計原則

Reciplexaでは、通常の抽象化に第一級関数を優先する。

値の受渡しで表現できる処理:
通常関数を使用する

評価順序・binding・構文構造の変更が必要な処理:
必要な場合に限りマクロを使用する


マクロは言語の中心的な抽象化機能ではなく、関数では表現できない限定的な構文糖衣に使用する。

###### DD-001.3 中心的な決定
- v1では式マクロだけを提供する
- マクロは同一コンパイル単位内だけで使用できる
- 一つのマクロは一つの入力patternだけを持つ
- マクロpattern変数には`$`を付ける
- 固定arityを基本とする
- 必要な場合のみ末尾の一個以上反復`...+`を許可する
- マクロは定義より後ろでのみ使用できる
- 自己再帰・相互再帰マクロを禁止する
- マクロ展開は型検査前に行う
- マクロは型情報を参照しない
- マクロは既定で衛生的である
- 意図的な名前capture APIは提供しない
- 展開には有限の予算を適用する
- マクロの別ファイル・別パッケージへの公開はv1では行わない
- 宣言マクロ、型マクロ、patternマクロ、procedural macroはv1では行わない

##### 0. 適用範囲
###### 0.1 本項目が定めるもの

本項目は次を規定する。

- マクロと通常関数の役割分担
- マクロ宣言のSurface構文
- マクロ呼出し
- マクロpattern変数
- 固定arity
- 末尾反復
- テンプレートへの構文挿入
- マクロのscope
- マクロ展開の段階
- 展開順序
- 名前衝突防止
- 展開予算
- エラー診断
- 展開構文のprovenance

###### 0.2 本項目が定めないもの

次は将来項目へ移管する。

- 宣言マクロ
- 型位置マクロ
- pattern位置マクロ
- module構成要素を生成するマクロ
- 複数rule
- 0個以上の反復
- procedural macro
- typed macro
- マクロのパッケージ間公開
- macro dependency
- import-macro
- .rpiのmacro仕様
- 意図的capture
- compiler plugin

##### 1. マクロと通常関数
###### 1.1 通常関数の優先

通常関数で表現できる処理にマクロを使用しない。

次のような処理は通常関数として定義する。

(clamp value minimum maximum)

(render-with-style document style)

(map transform values)


通常関数には次の利点がある。

- 引数と戻り値を直接型検査できる
- 第一級値として受け渡せる
- 高階関数へ渡せる
- 通常の関数合成を利用できる
- 実行時の呼出しとして追跡できる
- 展開によるコード膨張がない

###### 1.2 マクロが適する場合

マクロは、通常関数では同じ意味を保持できない場合に使用する。

典型例は条件付き評価である。

(when ready?
  (render document))


これをstrictな通常関数として実装すると、ready?の結果にかかわらず(render document)が先に評価される。

マクロなら次へ展開できる。

(if ready?
    (render document)
    unit)


したがって、マクロの主な用途は次とする。

- 条件付き評価
- 短絡評価
- 局所bindingを伴う式糖衣
- 本文の評価を遅延・制御する式糖衣

##### 2. マクロの基本構文
###### 2.1 宣言形式
(macro macro-name
  (macro-pattern ...)
  ->
  expression-template)

###### 2.2 固定arityの例
(macro unless
  ($condition $expression)
  ->
  (if $condition
      unit
      $expression))


利用：

(unless failed?
  (continue))


展開：

(if failed?
    unit
    (continue))

###### 2.3 可変長本文の例
(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))


利用：

(when ready?
  (prepare)
  (render document))


展開：

(if ready?
    (seq
      (prepare)
      (render document))
    unit)

###### 2.4 ->

->はマクロ入力patternと展開templateの区切りとして使用する。

左側:
呼出しに一致させる構文pattern

右側:
一致時に生成する式template


->は通常の値operatorではない。

##### 3. 式マクロ
###### 3.1 使用可能な位置

利用者定義マクロは式位置でのみ使用できる。

適合：

(val result
  (when ready?
    (render document)))


適合：

(fn (value)
  (unless invalid?
    (process value)))

###### 3.2 展開結果

式マクロは、ちょうど一つのRPX式へ展開しなければならない。

複数処理を生成する場合はseqまたはlocalで一つの式へまとめる。

(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))

###### 3.3 宣言位置

次のように、マクロによってtype、val、dataなどを生成する機能はv1では提供しない。

(define-enum-with-name
  alignment
  left
  center
  right)

###### 3.4 型位置

次のような型生成マクロはv1では認めない。

(type value
  (generated-type ...))

###### 3.5 Pattern位置

次のようなpattern生成マクロはv1では認めない。

(match value
  ((generated-pattern ...) ->
    result))

###### 3.6 Interfaceおよびmanifest

利用者定義マクロは次では使用できない。

- .rpi
- package.rpxm
- workspace.rpxm
- rpx.lock


公開interfaceとパッケージ構成は、マクロ展開に依存させず明示的に記述する。

##### 4. マクロpattern変数
###### 4.1 基本表記

マクロ呼出しから構文を受け取る変数には$を付ける。

$condition

$expression

$body

###### 4.2 通常identifierとの区別
$name:
マクロpattern変数

name:
通常のRPX identifierまたはtemplate由来identifier


通常のRPX identifierは$で開始できない。

これにより、マクロ変数と通常の名前を字句上区別する。

###### 4.3 受け取るもの

マクロpattern変数は実行時の値ではなく、入力構文を受け取る。

呼出し：

(unless failed?
  (continue))


では、次の構文が束縛される。

$condition:
failed?という構文

$expression:
(continue)という構文


この時点では、どちらも評価されない。

###### 4.4 Pattern変数の重複

同一pattern内で、同じマクロpattern変数を複数回束縛できない。

不適合：

(macro same
  ($value $value)
  ->
  $value)


診断例：

duplicate macro pattern variable:
  $value

###### 4.5 Template内での再利用

Patternで一度束縛した変数は、template内で複数回使用できる。

(macro duplicate
  ($expression)
  ->
  (tuple $expression $expression))


ただし、同じ入力式を複数回挿入すると実行時にも複数回評価され得る。

一度だけ評価する必要がある場合は、衛生的な局所bindingを生成する。

##### 5. 固定arity
###### 5.1 基本方針

固定数の入力で足りるマクロは、固定arityとする。

(macro unless
  ($condition $expression)
  ->
  ...)

###### 5.2 引数不足
(unless failed?)


は静的エラーである。

macro `unless` expects 2 arguments, but received 1

###### 5.3 引数過剰
(unless failed?
  first-expression
  second-expression)


も静的エラーである。

macro `unless` expects 2 arguments, but received 3

###### 5.4 通常関数との整合

マクロも通常関数と同様に、明示されたarityを持つ。

ただし、マクロの入力は値ではなく構文である。

##### 6. 末尾反復
###### 6.1 一個以上の入力

一個以上の末尾構文を受け取る場合は...+を使用する。

$body ...+

###### 6.2 Templateでの展開

Patternで反復束縛した構文列は、template内で...を使って展開する。

(seq $body ...)

###### 6.3 使用例
(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))


呼出し：

(when ready?
  first-expression
  second-expression)


対応：

$condition:
ready?

$body:
first-expression
second-expression

###### 6.4 反復位置

可変長反復はpattern引数列の末尾にだけ置ける。

適合：

($condition $body ...+)


不適合：

($prefix ...+ $last)

###### 6.5 反復数

一つのマクロpatternに含められる可変長反復は一つだけである。

###### 6.6 0個以上の反復

0個以上を表すpatternの...はv1では提供しない。

本文が不要な場合は、固定arityの別マクロを定義する。

##### 7. Template
###### 7.1 Template変数

Template内で使用できる$nameは、同じマクロpatternで束縛されたものだけである。

不適合：

(macro example
  ($input)
  ->
  (+ $input $missing))


診断例：

unbound macro pattern variable:
  $missing

###### 7.2 通常identifier

Template内の$を持たないidentifierは、通常のRPX構文を生成する。

(if $condition
    $expression
    unit)


ここでifとunitはマクロpattern変数ではない。

###### 7.3 Templateの構文妥当性

展開template自体が構文として不正な場合、マクロ定義位置で静的エラーにする。

入力挿入後にのみ判明する構文不整合は、マクロ呼出し位置を主位置として診断する。

###### 7.4 評価回数

Templateへ同じ入力構文を複数回挿入すると、展開後の実行時評価も複数回発生し得る。

(macro twice
  ($expression)
  ->
  (+ $expression $expression))


Effectfulな式を一度だけ評価する必要がある場合は、局所bindingを生成する。

##### 8. Scope
###### 8.1 コンパイル単位内限定

v1のマクロは、定義された同一コンパイル単位内だけで使用できる。

- 別ファイルからimportできない
- 別パッケージへ公開できない
- .rpiから公開できない

###### 8.2 宣言順序

マクロは定義位置より後ろでのみ使用できる。

適合：

(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))

(val result
  (when ready?
    (render document)))


不適合：

(val result
  (when ready?
    (render document)))

(macro when
  ...)

###### 8.3 下位モジュール

下位モジュールは、定義位置より前にある親モジュールのマクロを使用できる。

(macro when
  ($condition $body ...+)
  ->
  ...)

(module rendering
  (val render-if-ready
    (fn (document)
      (when ready?
        (render document)))))

###### 8.4 子scopeから親scope

マクロscopeは通常の逐次的な字句scopeに従う。

後続宣言や兄弟下位モジュールでまだ定義されていないマクロは参照できない。

##### 9. マクロ名と呼出し
###### 9.1 呼出し構文

マクロ呼出しは、通常のS式適用と同じ表面形を持つ。

(when condition
  expression)

###### 9.2 判別

呼出しheadがマクロbindingへ解決された場合、そのformをマクロ呼出しとして展開する。

呼出しheadが通常の値bindingへ解決された場合、通常関数適用として扱う。

###### 9.3 名前衝突

同一scopeで、マクロと通常値が同じlocal spellingを持ち、呼出しが曖昧になることを禁止する。

ambiguous call head:
  when

both a macro and a value are visible

###### 9.4 名前変更

マクロ名のrenameは、同一コンパイル単位内の対応する呼出しを通常のrename refactoring対象とする。

マクロは外部公開されないため、v1では公開API identityを持たない。

##### 10. 展開段階
###### 10.1 処理順序

概念的なコンパイル順序は次とする。

##### 1. Sourceをreaderで読む
##### 2. Lossless CSTを構築する
##### 3. 逐次scopeに従ってマクロ定義を認識する
##### 4. 式マクロを展開する
##### 5. 通常の名前解決を行う
##### 6. 型検査する
##### 7. IRへ変換する
##### 8. 実行可能artifactを生成する

###### 10.2 型検査前展開

マクロは通常の型検査より前に展開する。

展開後の式全体を通常の型checkerが検査する。

###### 10.3 型情報

マクロは次を参照できない。

- 入力式の推論型
- 期待型
- overload解決結果
- effect推論結果
- pattern網羅性情報

###### 10.4 Typed macro

型検査結果を参照して構文生成するtyped macroはv1では導入しない。

##### 11. 展開順序
###### 11.1 Source order

マクロ定義と通常宣言の可視性はsource orderに従う。

###### 11.2 外側からの展開

マクロ呼出しを検出した場合、その呼出しを展開し、展開結果を同じ位置で再度検査する。

展開結果に先行定義済みの別マクロ呼出しが含まれる場合、さらに展開できる。

###### 11.3 決定性

展開順序をhash mapの反復順、thread schedulingまたはfilesystem順へ依存させない。

同じsourceと同じ処理系versionからは、同じ展開結果を得なければならない。

###### 11.4 展開後の再検査

展開結果が再びマクロ呼出しである場合、通常式になるまで展開を繰り返す。

ただし、展開予算を超えてはならない。

##### 12. 再帰マクロ
###### 12.1 自己再帰

マクロが直接自分自身を展開結果へ含めることを禁止する。

不適合：

(macro forever
  ($value)
  ->
  (forever $value))

###### 12.2 相互再帰

複数マクロによる相互再帰も禁止する。

macro-a
→ macro-b

macro-b
→ macro-a

###### 12.3 先行マクロの利用

後から定義されたマクロが、既に定義済みの別マクロを利用することはできる。

(macro unless
  ($condition $body ...+)
  ->
  ...)

(macro unless-ready
  ($body ...+)
  ->
  (unless ready?
    $body ...))

###### 12.4 参照graph

マクロ参照graphはDAGでなければならない。

DAG:
有向非巡回graph
循環のない依存関係

##### 13. 衛生性
###### 13.1 定義

マクロの衛生性とは、マクロが生成した名前と利用側の名前が、偶然同じ綴りであるだけで衝突しない性質である。

通常の説明では、次の語を使用する。

マクロの衛生性
マクロ展開時の名前衝突防止

###### 13.2 基本保証
- 入力構文のidentifierは利用側contextを維持する
- Templateが新しく導入したbinderはfresh identityを持つ
- Template内の定義側参照は定義側bindingへ解決する
- 利用側の同名bindingを偶然取り込まない

###### 13.3 Fresh identity

マクロが生成する局所binderは、展開ごとに異なる内部identityを持つ。

Source上の綴りが同じでも、別展開で生成されたbinderは別bindingである。

###### 13.4 入力構文の再挿入

Pattern変数から受け取ったidentifierをtemplateへ挿入する場合、元の名前解決contextを維持する。

利用側で別bindingを意味していたidentifierを、マクロ定義側の同名bindingへ変更してはならない。

##### 14. 衛生性の例
###### 14.1 マクロ定義
(macro evaluate-once
  ($expression)
  ->
  (local
    (val temporary
      $expression)

    temporary))

###### 14.2 利用
(local
  (val temporary 100)

  (evaluate-once
    (+ temporary 1)))

###### 14.3 概念的なidentity
利用側:
temporary#user

マクロ生成:
temporary#expansion


展開後の意味は概念的に次となる。

(local
  (val temporary#user 100)

  (local
    (val temporary#expansion
      (+ temporary#user 1))

    temporary#expansion))


##user等の表記は説明用であり、利用者がsourceへ書くものではない。

##### 15. 意図的capture
###### 15.1 v1の方針

利用側の名前を綴りから検索して意図的に取り込む一般APIは提供しない。

intentional capture:
意図的な名前取り込み

v1:
不採用

###### 15.2 Binder名の明示

マクロが利用側で使われるbinder名を必要とする場合、呼出し側から明示的に受け取る。

概念例：

(with-context context
  (render context))


隠れたcontextを生成するのではなく、入力構文として受け取ったidentifierをbinder位置で使用する。

###### 15.3 理由
- 呼出しだけから導入名が分かる
- 名前衝突が予測可能
- Refactoringが容易
- 隠れたbinding依存を避けられる

##### 16. 純粋性
###### 16.1 展開入力

マクロ展開結果は、次だけから決まる。

- マクロ定義
- マクロ呼出しの入力構文
- 先行して定義されたマクロ環境
- 言語およびマクロ仕様version

###### 16.2 禁止される外部入力

マクロ展開から次へアクセスできない。

- filesystem
- package resource
- network
- 環境変数
- 時刻
- 乱数
- process
- ForeignValue
- editor state
- compiler内部の非公開情報

###### 16.3 通常値

通常のvalや関数を、マクロ展開中に実行できない。

Macro phaseとruntime phaseは分離する。

###### 16.4 再現性

同じ入力に対する展開は、環境や実行machineによらず同じ構文結果を生成しなければならない。

##### 17. 展開予算
###### 17.1 基本原則

Compilerはマクロ展開へ有限の展開予算を適用する。

仕様本文では「展開予算」または「展開上限」と呼ぶ。

###### 17.2 監視対象

実装は少なくとも次を制限できる。

- 展開回数
- 展開の入れ子深さ
- 生成構文node数

###### 17.3 具体値

具体的な上限値は言語仕様へ固定せず、toolchain policyとして管理する。

ただし、準拠処理系は有限上限を持たなければならない。

###### 17.4 上限超過

上限超過は静的エラーとする。

macro expansion limit exceeded


診断には、可能な範囲で次を含める。

- マクロ名
- 呼出し位置
- 展開chain
- 超過した制限種別

###### 17.5 Sourceからの無制限化

通常RPX sourceから展開予算を無制限に変更できない。

悪意あるsourceまたは依存関係が制限を無効化できてはならない。

##### 18. エラー診断
###### 18.1 Pattern不一致

固定arityまたは末尾反復の条件を満たさない場合、マクロ呼出し位置で診断する。

macro invocation does not match its pattern

###### 18.2 引数数
macro:
  unless

expected:
  2 arguments

received:
  1 argument

###### 18.3 ...+

一個以上必要な末尾入力が空の場合：

macro `when` requires at least one body expression

###### 18.4 未束縛pattern変数

Template内で未束縛の$nameを参照した場合、マクロ定義位置でエラーにする。

###### 18.5 不正な展開結果

展開結果が式として不正な場合、主診断位置をマクロ呼出しにし、マクロ定義およびtemplate位置を補助情報として示す。

###### 18.6 型エラー

展開後の式に型エラーがある場合、通常の型診断に加えて、その式がどのマクロ呼出しから生成されたかを示す。

##### 19. Provenance
###### 19.1 保持する情報

生成構文には次の由来情報を保持する。

- マクロ定義位置
- マクロ呼出し位置
- 入力構文のsource span
- 展開templateのsource span
- 展開chain

###### 19.2 診断例
type error in syntax generated by macro `when`

macro invocation:
  current-file.rpx:42

macro definition:
  current-file.rpx:10

###### 19.3 Source map

Formatter、IDE、定義移動およびエラー表示のため、生成構文から元sourceへの対応を失わない。

###### 19.4 正式identity

v1では宣言マクロを導入しないため、マクロが公開DefinitionIdを生成することはない。

局所binderについてのみ、展開ごとのfresh identityを割り当てる。

##### 20. マクロの外部公開
###### 20.1 v1の制限

マクロを別コンパイル単位または別パッケージへ公開できない。

- macro-dependenciesなし
- import-macroなし
- .rpiのmacro仕様なし
- compiled macro artifactなし

###### 20.2 標準構文

複数パッケージで共通に必要な必須構文は、次のいずれかとして提供する。

- Core特殊形式
- 通常関数
- 標準libraryの通常値

###### 20.3 将来拡張

再利用可能なマクロpackageが必要になった場合、macro dependencyとinterfaceの設計を別項目で追加する。

##### 21. 適合試験
###### 21.1 適合試験 MAC-01：固定arity
(macro unless
  ($condition $expression)
  ->
  (if $condition
      unit
      $expression))

(val result
  (unless failed?
    (continue)))


期待結果：

success

展開:
(if failed?
    unit
    (continue))

###### 21.2 不適合試験 MAC-02：引数不足
(unless failed?)


期待結果：

static error:
macro `unless` expects 2 arguments, but received 1

###### 21.3 不適合試験 MAC-03：引数過剰
(unless failed?
  first
  second)


期待結果：

static error:
macro `unless` expects 2 arguments, but received 3

###### 21.4 適合試験 MAC-04：末尾反復
(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))

(when ready?
  (prepare)
  (render document))


期待結果：

success

展開:
(if ready?
    (seq
      (prepare)
      (render document))
    unit)

###### 21.5 不適合試験 MAC-05：空の...+
(when ready?)


期待結果：

static error:
macro `when` requires at least one body expression

###### 21.6 不適合試験 MAC-06：反復が末尾以外
(macro invalid
  ($prefix ...+ $last)
  ->
  $last)


期待結果：

static error:
variable-length macro repetition must appear at the end of the pattern

###### 21.7 不適合試験 MAC-07：重複pattern変数
(macro duplicated
  ($value $value)
  ->
  $value)


期待結果：

static error:
duplicate macro pattern variable:
  $value

###### 21.8 不適合試験 MAC-08：未束縛template変数
(macro invalid
  ($input)
  ->
  (+ $input $missing))


期待結果：

static error:
unbound macro pattern variable:
  $missing

###### 21.9 適合試験 MAC-09：衛生的binder
(macro evaluate-once
  ($expression)
  ->
  (local
    (val temporary
      $expression)

    temporary))

(local
  (val temporary 100)

  (evaluate-once
    (+ temporary 1)))


期待結果：

success

利用側temporaryと、
マクロ生成temporaryは別binding

###### 21.10 不適合試験 MAC-10：定義前使用
(val result
  (when ready?
    (render document)))

(macro when
  ($condition $body ...+)
  ->
  ...)


期待結果：

static error:
macro is not defined at this source position:
  when

###### 21.11 不適合試験 MAC-11：自己再帰
(macro forever
  ($value)
  ->
  (forever $value))


期待結果：

static error:
recursive macro expansion is not permitted

###### 21.12 不適合試験 MAC-12：宣言位置
(define-values first second)


define-valuesが式マクロとして定義されており、宣言位置で使用された場合：

static error:
expression macro cannot be used in declaration position

###### 21.13 不適合試験 MAC-13：Interface
// module.rpi

(when condition
  specification)


期待結果：

static interface error:
user-defined macros are not permitted in interface files

###### 21.14 不適合試験 MAC-14：展開上限

展開chainが処理系の有限上限を超えた場合：

static error:
macro expansion limit exceeded

##### 22. 移管先OPEN・下位項目・状態
###### 22.1 `OPEN-MAC-EXT-001`

次を将来項目として移管する。

- 複数rule
- 0個以上の反復
- 複数反復
- 構文分類付きpattern変数
- 宣言マクロ
- 型マクロ
- patternマクロ
- module itemマクロ

###### 22.2 `OPEN-MAC-PKG-001`

次を将来項目として移管する。

- マクロのパッケージ間公開
- macro-dependencies
- import-macro
- .rpiのmacro仕様
- compiled macro artifact
- macro packageのinterface hash

###### 22.3 `OPEN-MAC-PROC-001`

次を将来項目として移管する。

- procedural macro
- syntax object操作API
- 独自診断API
- procedural macro sandbox
- 実行step・memory・time制限

###### 22.4 `OPEN-MAC-TYPED-001`

次を将来項目として移管する。

- typed macro
- 型検査後のcode generation
- derive
- 型情報を利用した展開

###### 22.5 `OPEN-MAC-CAP-001`

次を将来項目として移管する。

- 意図的capture
- 非衛生的identifier生成
- 利用側scopeの明示操作

###### 22.6 `OPEN-EDT-001`

次を移管する。

- マクロ呼出しsiteのstable provenance
- 展開結果とsource editの対応
- 展開構文に対するrename
- stale transaction

###### 22.7 下位項目

```text
OPEN-MAC-001A
マクロの最小Surface構文
→ RESOLVED

OPEN-MAC-001B
Pattern変数・固定arity・末尾反復
→ RESOLVED

OPEN-MAC-001C
Scope・宣言順序・再帰禁止
→ RESOLVED

OPEN-MAC-001D
展開段階・展開順序
→ RESOLVED

OPEN-MAC-001E
衛生性・名前衝突防止
→ RESOLVED

OPEN-MAC-001F
展開予算・診断
→ RESOLVED

OPEN-MAC-001G
Identity・provenance
→ RESOLVED

OPEN-MAC-001H
外部公開範囲
→ RESOLVED
```

###### 22.8 最終状態

```text
OPEN-MAC-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- 第一級関数を中心とする通常の抽象化
- 関数では表現できない評価制御向けの限定的な式マクロ
- 単純な一pattern構文
- 固定arityと末尾反復
- 型検査前の決定的な展開
- 自動的な名前衝突防止
- 隠れた名前captureを持たない安全な展開
- 再帰しない有限な展開
- マクロ依存をパッケージ境界へ持ち込まない単純なv1設計

#### 13.6 `TYP-001` Gradual set-theoretic types

##### 概要・状態

- 集合論的漸進型、semantic subtyping、bounded dynamic: `確定`。
- algorithmic checkerの完全判定範囲、三値判定、bidirectional typing: `確定`。
- `OPEN-TYP-001`および`OPEN-TYP-002`: `解決済み`。
- 本節は、従来の未決定記述を置き換える。

##### 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure

> 統合元: `OPEN-TYP-001`。同項目は解決済み。

###### 状態

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

###### `DD-TYP-DYN-001`: static型とgradual型の分離

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

###### `DD-TYP-DYN-002`: `dynamic S`の意味

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

###### `DD-TYP-DYN-003`: static top型`any`

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

###### `DD-TYP-DYN-004`: `never`およびdynamicの正規形

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

###### `DD-TYP-DYN-005`: dynamic値をstatic型として使用する三段階判定

`確定`

```text
x : dynamic S
```

を、static型`T`が必要な位置で使用するとき、次の三段階で判定する。

# 1. 上限全体が要求型へ含まれる場合

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

# 2. 上限と要求型が互いに素である場合

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

# 3. 一部だけ重なる場合

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

###### `DD-TYP-DYN-006`: cast成功後の型

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

###### `DD-TYP-DYN-007`: occurrence typingとの関係

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

###### `DD-TYP-DYN-008`: static値からdynamic値への導入

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

###### `DD-TYP-DYN-009`: dynamic上限のwidening

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

###### `DD-TYP-DYN-010`: foreign値のdynamic導入

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

###### `DD-TYP-DYN-011`: decoderとgradual foreign boundaryの分離

`確定`

外部bytesや構造化データをRPX値へ変換するdecoderと、foreign runtime objectをdynamic値として
導入する境界を区別する。

# Static decoder

```text
decode<S>(bytes)
: Result<S, decode-error>
```

decoderは外部表現を完全に検証し、通常のstatic RPX値`S`へ再構築する。

# Gradual foreign boundary

```text
import-dynamic<S>(foreign-value)
: Result<dynamic S, boundary-error>
```

foreign objectのidentityまたは動的表現を維持したまま、上限`S`だけを保証する。

JSON、設定ファイル、document source等の通常の外部データについては、可能な限りdecoderを使用し、
安易に`dynamic any`へ導入しない。

***

###### `DD-TYP-DYN-012`: runtime-checkableな型

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

###### `DD-TYP-DYN-013`: implicit cast failure

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

###### `DD-TYP-DYN-014`: 明示的safe cast

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

###### `DD-TYP-DYN-015`: cast evidence

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

# `Identity`

runtime検査が不要なcast。

# `Widen`

dynamic上限の安全な拡張。

# `TagCheck`

primitiveまたはruntime tagによる直接検査。

# `UnionCheck`

複数候補のいずれかに属することの純粋な検査。

# `IntersectionCheck`

すべての構成型の条件を満たすことの検査。

# `RecordCheck`

必要field、field型およびrow条件の検査。

# `VariantCheck`

constructor identityおよびpayload型の検査。

# `FunctionGuard`

関数呼出時に引数および結果を検査するwrapper。

# `NominalCheck`

moduleまたは型所有者が発行したruntime type identityによる検査。

# `Compose`

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

###### `DD-TYP-DYN-016`: cast evidenceの純粋性

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

###### `DD-TYP-DYN-017`: evidence compositionと最適化

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

###### `DD-TYP-DYN-018`: cast provenance

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

###### `DD-TYP-DYN-019`: recordおよびvariant cast

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

###### `DD-TYP-DYN-020`: opaque abstract typeのcast

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

###### `DD-TYP-DYN-021`: fixed-arity function cast

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

###### `DD-TYP-DYN-022`: function引数の反変cast

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

###### `DD-TYP-DYN-023`: function結果の共変cast

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

###### `DD-TYP-DYN-024`: function arityの制限

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

###### `DD-TYP-DYN-025`: effectful function cast

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

###### `DD-TYP-DYN-026`: dynamic境界を通れない制御値

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

###### `DD-TYP-DYN-027`: polymorphismとdynamic境界

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

###### `DD-TYP-DYN-028`: gradual guarantee

`確定`（設計目標、未証明）

RPXは、bounded dynamicを含む型precision関係を定義する。

precision関係はsemantic subtypingとは別の判断とする。

概念的に次の記号を用いる。

```text
G1 ⊑p G2
```

これは、`G2`が`G1`以上に精密であることを表す。

RPXはstatic gradual guaranteeおよびdynamic gradual guaranteeを設計目標とする。

# Static gradual guarantee

型情報を不精密にしたことだけを理由として、以前型検査可能だったプログラムを
不必要に拒否しないことを目標とする。

ただし、次の制約は型precisionの低下によって消去できない。

* affine resumption
* scoped evidence
* module-private nominal identity
* resource capability
* handler authority
* effect-row安全境界

# Dynamic gradual guarantee

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

###### `DD-TYP-NUM-001`: RPX v1の基本数値型

`確定`

RPX v1は、次の二つを別のprimitive型として持つ。

```text
int
f64
```

# `int`

```text
int
= 任意精度の正確な符号付き整数
```

整数overflowを通常の`int`へ設けない。値の大きさは利用可能memoryによってのみ制限される。

# `f64`

```text
f64
= IEEE 754 binary64 floating-point
```

`f64`は数学的な実数全体ではなく、有限精度の近似数値型である。

# `number`

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

###### `DD-TYP-NUM-002`: 数値promotion

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

###### `DD-TYP-NUM-003`: dynamic castとnumeric promotionの順序

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

###### `DD-NAME-001`: 組込み型名と識別子の小文字規約

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

###### `DD-NAME-002`: namespace間の同綴り衝突

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

###### dynamic typingのCore構文

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

###### dynamic typingの終端状態

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

###### 適合試験

# static injection

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

# invalid static injection

```text
picture
→ dynamic number
```

期待：

```text
静的エラー
```

# dynamic widening

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

# safe static use

```text
x : dynamic int
```

を`number`が必要な位置で使用する。

期待：

```text
int <: number
runtime checkなし
```

# partial overlap

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

# disjoint use

```text
x : dynamic(union(number, str))
```

を`picture`が必要な位置で使用する。

期待：

```text
静的エラー
```

# cast precision

```text
x : dynamic(union(int, str))
```

を`number`として検査する。

期待される成功後の型：

```text
int
```

# `any`と`dynamic any`

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

# foreign ingress

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

# implicit cast failure

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

# explicit safe cast

```text
try-cast<number>(x)
```

実体が`str`：

```text
none
```

評価run自体は継続する。

# fixed-arity function cast

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

# function result cast

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

# effect-compatible function cast

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

# effect-incompatible function cast

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

# polymorphic value boundary

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

# dynamicからforall

```text
dynamic function
→ forall a. a -> a
```

期待：

```text
静的エラー
```

# numeric promotion

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

# namespace collision warning

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

###### 未解決事項の移管

`OPEN-TYP-001`を閉じるため、次を別項目へ移管する。

# `OPEN-TYP-002`

* static semantic subtypingの実装可能な決定手続き
* emptiness判定の正確な範囲
* cast挿入が依存するalgorithmic approximation
* type／row／effect constraint solverの完全性
* checker limitationによる保守的拒否
* principal solutionの有無

# `OPEN-SYN-002`／`LIT-001`

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

# `OPEN-ERR-001`

* `dynamic-type-error`と他の異常終端の統合分類
* GUI／CLI診断の最終形式
* cleanup中にdynamic failureが発生した場合
* 複数failureの合成
* exception effectとの関係

# `OPEN-MOD-001`

* abstract typeのruntime nominal identity
* separate compilationをまたぐidentity安定性
* plugin境界でのabstract type evidence
* signatureへruntime-checkabilityを記述する方法

# `OPEN-KER-001`

* ForeignValue ABI
* runtime validator ABI
* trusted foreign adapter
* unsafe dynamic assumptionの権限
* foreign function effect契約の監査

# 将来拡張

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

###### 解決後の基本原則

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

##### 13.6.2 `TYP-ALG-001` Algorithmic型検査、semantic subtypingの判定範囲、型推論

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

###### `DD-TYP-ALG-001`: 宣言的型関係とalgorithmic判定の分離

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

###### `DD-TYP-ALG-002`: algorithmic判定の三値結果

`確定`

algorithmicな部分型、型等価、空性および関連する型判断は、内部的に次の三種類の結果を返す。

```text
proved
disproved
unknown
```

# `proved`

要求された型関係が宣言的意味論のもとで成立することを、checkerが証明したことを表す。

```text
decide-subtype(int, number)
→ proved
```

# `disproved`

要求された型関係が成立しないことを、checkerが証明したことを表す。

```text
decide-subtype(str, number)
→ disproved
```

可能な場合は、反例となる型または値のwitnessを診断用に生成する。

# `unknown`

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

###### `DD-TYP-ALG-003`: 診断分類

`確定`

次を異なる診断として扱う。

```text
type-error
annotation-required
checker-limitation
checker-resource-limit
unsupported-language-feature
```

# `type-error`

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

# `annotation-required`

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

# `checker-limitation`

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

# `checker-resource-limit`

決定的なsolver budgetを超過した場合に使用する。

理論上のalgorithmic limitationと、実装設定によるresource limitを区別する。

# `unsupported-language-feature`

RPX v1で意味またはalgorithmを提供しない機能を使用した場合に使用する。

```text
unsupported-language-feature:
  higher-rank polymorphism
```

***

###### `DD-TYP-ALG-004`: soundness、completeness、terminationの優先順位

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

###### `DD-TYP-ALG-005`: principal type

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

###### `DD-TYP-ALG-006`: 決定的なsolver budget

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

### Effectに関するalgorithmic用語

###### `DD-TYP-EFF-001`: effect関連用語

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

# `effect-signature-environment`

effect identityおよびoperation identityから、operationの引数型、結果型、effect identity等を取得する静的環境である。

概念記号として`Ξ`を使用できる。

# `effect-environment`

現在の式を実行するときに、その文脈で利用可能または許容されるeffect要求を表す型検査上の環境である。

effect environment自体は、EffectRowによって表現される。

```text
effect environment:
<resource, log, file-write>
```

# `expression-effects`

式を現在評価することで、外側へ要求される未処理effectである。

```lisp
(load-image resource-id)
```

のexpression effectsは、概念的に次である。

```text
<resource>
```

# `function-required-effects`

関数値を呼び出したときに、外側へ要求されるeffectである。

```text
project ->{resource, log} picture
```

における`resource`と`log`である。

文献上latent effectと呼ばれる概念に相当するが、RPXの規範用語では`function-required-effects`を使用する。

# `handled-effects`

現在のhandlerまたはrunnerが処理するeffectである。

# `residual-effects`

handlerまたはrunner適用後も外側へ残るeffectである。

# `effect-row`

上記のeffect要求を表現する型レベルのrow構造である。

effect environmentとeffect rowは同一概念ではない。

```text
effect-row:
型レベルの構造

effect-environment:
型検査文脈における役割
```

***

###### `DD-TYP-EFF-002`: 最小required effects

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

###### `DD-TYP-EFF-003`: 関数値とrequired effects

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

###### `DD-TYP-EFF-004`: effect-row polymorphism

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

###### `DD-TYP-EFF-005`: handlerとrunnerによるeffect縮小

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

###### `DD-TYP-EFF-006`: EffectRow alias

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

###### `DD-TYP-EFF-007`: 注釈されたrequired effects

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

### Static型のBoolean代数

###### `DD-TYP-BOOL-001`: 型のBoolean演算

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

###### `DD-TYP-BOOL-002`: Surface negationの制限

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

### Singleton型

###### `DD-TYP-SINGLETON-001`: singleton型

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

### Function型

###### `DD-TYP-FN-001`: fixed-arity function

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

###### `DD-TYP-FN-002`: fixed-arity function subtyping

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

###### `DD-TYP-FN-003`: function intersection

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

###### `DD-TYP-FN-004`: function intersectionの適用可能性

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

###### `DD-TYP-FN-005`: branch specificity

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

###### `DD-TYP-FN-006`: function intersectionのcoherence

`確定`

function intersectionを構成するすべてのbranch pairについてcoherenceを検査する。

# 入力領域が互いに素

```text
intersect(input(b1), input(b2)) ≃ never
```

なら問題ない。

# 片方が他方を包含

specific branchの結果型とrequired effectsがgeneral branchの契約を強化することを要求する。

# 入力領域が等価

結果型とrequired effectsも等価である場合だけ、重複branchとして正規化できる。

それ以外はduplicate／incoherent branchとして拒否する。

# 入力領域が重なるが非比較

```text
intersect(input(b1), input(b2)) ≄ never

input(b1) </: input(b2)

input(b2) </: input(b1)
```

なら、RPX v1ではfunction intersectionを拒否する。

source order、declaration order、import orderまたは期待結果型によってbranchを選ばない。

***

###### `DD-TYP-FN-007`: union引数とdispatch

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

### RecordRow

###### `DD-TYP-ROW-001`: closed、open、row-polymorphic record

`確定`

次の三種類を区別する。

# Closed record

明示されたfieldだけを持つ。

```text
{x : f64, y : f64}
```

# Open record

明示fieldを少なくとも持ち、追加fieldを許すが、その構成を型関係として保持しない。

```text
{x : f64, y : f64, ...}
```

# Row-polymorphic record

追加fieldの構成をRecordRow変数として保持する。

```text
{x : f64, y : f64, ..r}
```

open recordとrow variableを同一視しない。

入力に存在した未知fieldを出力へ保存する関数にはrow variableを使用する。

***

###### `DD-TYP-ROW-002`: closed recordの正確なshape

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

###### `DD-TYP-ROW-003`: immutable field covariance

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

###### `DD-TYP-ROW-004`: RecordRow変数とlacks制約

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

###### `DD-TYP-ROW-005`: field extensionとupdate

`確定`

field extensionとfield updateを別の型操作とする。

# Extension

存在しないfieldを追加する。

```text
extension requires:
row lacks label
```

# Update

既存fieldの値または型を置き換える。

```text
update requires:
label is present
```

同じSurface操作でextensionとupdateを曖昧に実行してはならない。

これにより、field名の誤記が意図しない新規fieldとして受理されることを防ぐ。

***

###### `DD-TYP-ROW-006`: row-preserving update

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

###### `DD-TYP-ROW-007`: optional field

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

###### `DD-TYP-ROW-008`: recordのBoolean演算

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

### Recursive data type

###### `DD-TYP-REC-001`: recursive data type

`確定`

RPX v1はrecursive data typeを持つ。

再帰型は、主にvariant／data定義から生成する。

一般の`mu`型をSurfaceへ公開しない。

`data`等の正確なSurface構文は別途決定する。

checker内部では、recursive typeを有限の循環型graphとして表現してよい。

***

###### `DD-TYP-REC-002`: equi-recursiveな利用者意味論

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

###### `DD-TYP-REC-003`: contractiveness

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

###### `DD-TYP-REC-004`: strict positivity

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

###### `DD-TYP-REC-005`: regularity

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

###### `DD-TYP-REC-006`: base constructor

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

###### `DD-TYP-REC-007`: recursive dataの完全判定範囲

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

### Bidirectional type checking

###### `DD-TYP-BIDI-001`: bidirectional typing

`確定`

RPX v1はbidirectional type checkingを採用する。

型検査器は、次の二つの判断を持つ。

# Synthesis

```text
Γ ⊢ e ⇒ t ! r
```

式`e`から値型`t`および最小required effects `r`を求める。

# Checking

```text
Γ ⊢ e ⇐ t ! r
```

式`e`が期待型`t`へ適合するか検査し、最小required effects `r`を求める。

型注釈は独立した`(type ...)`宣言によって期待型を与える。

***

###### `DD-TYP-BIDI-002`: 型注釈付きbinding

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

###### `DD-TYP-BIDI-003`: 無注釈binding

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

###### `DD-TYP-BIDI-004`: synthesis可能な式

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

###### `DD-TYP-BIDI-005`: checkingを優先する式

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

###### `DD-TYP-BIDI-006`: `if`と`match`

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

###### `DD-TYP-BIDI-007`: subtyping、promotion、dynamic cast

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

###### `DD-TYP-BIDI-008`: annotation-requiredとchecker-limitation

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

### 明示的多相型

###### `DD-TYP-POLY-001`: 明示的`forall`

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

###### `DD-TYP-POLY-002`: `forall` binderのkind

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

###### `DD-TYP-POLY-003`: `forall`のscope

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

###### `DD-TYP-POLY-004`: rank-1／prenex制限

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

### 完全性分類

###### `DD-TYP-FRAG-001`: A — 完全判定fragment

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

###### `DD-TYP-FRAG-002`: B — `unknown`を返し得るfragment

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

###### `DD-TYP-FRAG-003`: C — 注釈を要求し得るfragment

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

###### `DD-TYP-FRAG-004`: D — RPX v1で禁止するfragment

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

### 共通constraint worklist

###### `DD-TYP-SOLVER-001`: shared constraint worklist

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

###### `DD-TYP-SOLVER-002`: solver間のconstraint生成

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

###### `DD-TYP-SOLVER-003`: constraint処理の優先度

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

###### `DD-TYP-SOLVER-004`: canonicalizationとmemoization

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

###### `DD-TYP-SOLVER-005`: solver終了状態

`確定`

共通worklistの処理結果を次のように分類する。

# 成功

```text
worklistが空
かつ
必須metavariableが解決済み
かつ
scope／capability constraintが成立
かつ
generalization条件が成立
```

# Type error

いずれかの必須constraintが`disproved`になった。

# Annotation required

型関係は既知のalgorithmで検査可能だが、型合成に必要なmetavariableの解が一意に定まらない。

# Checker limitation

意味の定義されたconstraintが残るが、v1 solverが`proved`または`disproved`へ到達できない。

# Resource limit

決定的なsolver step budgetを超過した。

***

###### `DD-TYP-SOLVER-006`: cast insertionとgeneralizationの順序

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

### 型検査器の概念pipeline

###### `DD-TYP-SOLVER-007`: checkerの全体処理順序

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

### 適合試験

###### 三値判定

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

###### 診断分類

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

###### 最小required effects

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

###### 関数値のeffect

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

###### EffectRow polymorphism

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

###### Handlerによるeffect縮小

body：

```text
<resource, cache, log>
```

`cache`および`log`を処理するrunner適用後：

```text
<resource>
```

***

###### Singleton型

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

###### Function subtyping

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

###### Function intersection coherence

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

###### 非比較なbranch overlap

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

###### Union引数の暗黙dispatch禁止

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

###### Closed record

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

###### Row preservation

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

###### Optional field

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

###### Recursive data

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

###### 非contractive再帰

```text
a = a
```

期待：

```text
unsupported／invalid recursive type
```

***

###### Non-regular recursion

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

###### Bidirectional checking

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

###### Explicit `forall`

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

###### Kind error

```lisp
(forall ((r record-row))
  (fn (r) r))
```

RecordRowを通常値型位置で使用している場合：

```text
kind error
```

***

###### Solver determinism

同一source、compiler version、dependency signatureおよびchecker budgetで複数回検査する。

期待：

```text
同じconstraint処理結果
同じ診断分類
同じprimary span
同じsolver-budget結果
```

***

### 未解決事項の移管

###### `OPEN-SYN-002`

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

###### `OPEN-DAT-001`

* variant／constructorの最終Surface構文
* pattern grammar
* pattern exhaustiveness policy
* guard
* literal pattern
* or-pattern
* recursive data definitionの完全な構文

###### `OPEN-EFF-001`の後続仕様

* EffectRow constraintの正確なformal rule
* duplicate labelのcanonical representation
* named effect instanceのalgorithmic比較
* effect aliasのmodule visibility
* host effect environmentとのlinking

###### `OPEN-MOD-001`

* public APIでの`(type ...)`必須要件
* signature内の明示的`forall`
* abstract typeのvariance
* separate compilation時のtype／effect identity
* imported checker metadataの信頼境界

###### `OPEN-ERR-001`

* `checker-limitation`
* `annotation-required`
* `checker-resource-limit`
* 複数constraint failureのprimary error選択
* IDEとCLIでの診断表示差

###### 将来拡張

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

### 解決後の基本原則

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

#### 13.7 `ROW-001` Row-polymorphic records

##### 概要・状態

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

#### 13.8 `EFF-001` Algebraic effects and handlers

> 統合元: `OPEN-EFF-001`。同項目は解決済み。

##### 状態

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

##### `DD-EFF-001`: deep handler

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

##### `DD-EFF-002`: one-shot resumption

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

##### `DD-EFF-003`: resumptionの型とscope

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

##### `DD-EFF-004`: effect operationの呼出し

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

##### `DD-EFF-005`: nearest matching handlerと自動伝播

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

##### `DD-EFF-006`: 明示的forward

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

##### `DD-EFF-007`: handler clauseの実行scope

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

##### `DD-EFF-008`: return clause

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

##### `DD-EFF-009`: handler単位の結果型変換

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

##### `DD-EFF-010`: handler valueのrank-1多相性

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

##### `DD-EFF-011`: first-class handler value

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

##### `DD-EFF-012`: Core `handle`とSurface `with`

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

##### `DD-EFF-013`: ambient effectとnamed/scoped effect instance

`確定`

RPXは、次の二種類のeffect利用形態を持つ。

1. ambient effect
2. named/scoped effect instance

###### Ambient effect

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

###### Named/scoped effect instance

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

##### `DD-EFF-014`: EffectRowの意味

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

##### `DD-EFF-015`: ambient effect rowと制約生成

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

##### `DD-EFF-016`: handlerの型付け骨格

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

##### `DD-EFF-017`: 利用者向けmaskの禁止

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

##### `DD-EFF-018`: residual effectと実行境界

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

##### `DD-EFF-019`: 未処理effectの動的分類

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

##### `DD-EFF-020`: cleanupとの接続要件

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

#### 13.9 `MOD-001` モジュール・シグネチャ・Functor・分割コンパイル
##### DD-001 決定概要
###### DD-001.1 状態
Status:
RESOLVED

Scope:
コンパイル単位
外側モジュール
下位モジュール
インポート
インターフェースファイル
シグネチャ
抽象型
シグネチャ指定
シグネチャの精緻化
型共有
Functor
再エクスポート
正式identity
分割コンパイル

###### DD-001.2 中心的な決定
- 一つの実装source fileを一つのcompilation unitとする
- 各compilation unitは一つの外側モジュールを定義する
- ファイル内にはinline下位モジュールを定義できる
- 下位モジュールを別ファイルから読み込む機能はv1では導入しない
- 通常のモジュール依存グラフはDAGとする
- importは通常形式、as、only、個別asを組み合わせられる
- aliasや再エクスポートは正式identityを変更しない
- .rpiを明示インターフェースファイルとする
- .rpiがない内部モジュールでは実装からシグネチャを推論する
- パッケージ外へ公開するモジュールには.rpiを要求する
- 名前付きシグネチャを導入する
- シグネチャ適合は構造的に判定する
- 抽象型のidentityはモジュールごとにnominalとする
- 適用的Functorを採用する
- 生成的Functor、再帰モジュール、第一級モジュールはv1では導入しない
- interface metadataとInterfaceHashを生成する

##### 0. 適用範囲
###### 0.1 本項目が定めるもの

本項目は次を規定する。

- source fileとcompilation unitの関係
- 外側モジュールと下位モジュール
- モジュールpath
- importと名前解決
- .rpiインターフェース
- 推論シグネチャ
- 名前付きシグネチャ
- 抽象型とconstructor公開data
- シグネチャ指定
- 下位モジュール仕様
- シグネチャの精緻化
- 型共有
- Functor
- module-alias
- re-export
- signature include
- 正式identity
- interface metadata
- 分割コンパイル

###### 0.2 本項目が直接定めないもの

次は別のOPEN項目へ移管する。

- package manifestの具体的形式
- version constraint
- lockfile
- package feature
- resource root
- macro phase
- ABIの完全な規則
- trusted adapter ABI
- 再帰モジュール
- 第一級モジュール
- 生成的Functor
- destructive signature substitution
- open variant

##### 1. コンパイル単位
###### 1.1 基本単位

一つの実装source fileは、一つのコンパイル単位を定義する。

compilation unit
=
個別に解析・型検査・コンパイル可能な単位


各コンパイル単位は、一つの外側モジュールとして振る舞う。

###### 1.2 外側モジュールのwrapper

Source file内には、外側モジュールの明示wrapperを書かない。

次のようにはしない。

(module graphics/color
  ...)


ファイル自身が外側モジュールの本体である。

###### 1.3 一ファイル内の外側モジュール数

一つのsource fileに複数の独立した外側モジュールを定義できない。

次のような構成は不採用とする。

(module graphics/color
  ...)

(module graphics/image
  ...)

###### 1.4 一ファイル一モジュールとの違い

正確な設計は次である。

一ファイル:
一つのコンパイル単位
一つの外側モジュール

ファイル内部:
複数のinline下位モジュールを定義可能

##### 2. モジュールpathとファイルpath
###### 2.1 Source root

パッケージは一つ以上のsource rootを宣言する。

具体的なmanifest構文はOPEN-PKG-001へ移管する。

###### 2.2 Module path

Source rootからの相対pathをモジュールpathとする。

src/color/conversion.rpx


は概念的に次のモジュールpathを持つ。

color/conversion

###### 2.3 Interface path

対応する実装ファイルとインターフェースファイルは、同じモジュールpathを持つ。

実装:
src/color/conversion.rpx

interface:
interface/color/conversion.rpi


両者は別モジュールではなく、同じModuleIdの実装部分と仕様部分である。

###### 2.4 Path変更

モジュールpathを変更した場合、v1では新しいModuleIdを生成する。

color/conversion
→ rendering/color-conversion


は、原則として公開API上のモジュール移動・改名である。

Identityを維持する必要がある場合は、旧pathにmodule aliasを残す。

##### 3. 下位モジュール
###### 3.1 基本構文

ファイル内にはinline下位モジュールを定義できる。

(module parsing
  (data state
    initial
    reading
    failed)

  (type parse
    (fn str parse-result))

  (val parse
    (fn (source)
      ...)))

###### 3.2 下位モジュールのpath

外側モジュールがdocument/readerなら、この下位モジュールは概念的に次のpathを持つ。

document/reader/parsing

###### 3.3 下位モジュールの外部ファイル化

次のように、本文なしのmodule宣言によって別fileを探索する機能はv1では導入しない。

(module parsing)


下位モジュールはinlineのみとする。

大きくなった場合は、独立したコンパイル単位へ移動する。

###### 3.4 Module body

下位モジュールの本体は、通常の順序付き宣言グループである。

置けるものには次が含まれる。

- type
- type-alias
- val
- data
- rec
- nested module
- signature
- functor
- 将来のeffect宣言

##### 4. 下位モジュールのscopeと純粋性
###### 4.1 Importの位置

importはコンパイル単位の先頭import領域だけに置ける。

下位モジュール内のimportは認めない。

不適合：

(module parsing
  (import text/utf8)

  ...)


診断例：

imports are only permitted in the compilation-unit import section

###### 4.2 親scopeの参照

下位モジュールは、定義位置より前にある親モジュールのbindingを参照できる。

適合：

(val default-limit 100)

(module parsing
  (val limit
    default-limit))

###### 4.3 後方参照

定義位置より後ろにある親bindingは参照できない。

不適合：

(module parsing
  (val limit
    default-limit))

(val default-limit 100)

###### 4.4 新しいscope

下位モジュールは新しい宣言scopeを作る。

内側bindingは、通常のscope規則に従って外側bindingをshadowできる。

###### 4.5 Top-level effect

外側モジュールと下位モジュールのtop-level値initializerはpureでなければならない。

不適合：

(module configuration
  (val current
    (read-resource config-file)))


Effectfulな処理は関数内へ置く。

###### 4.6 モジュールは通常値ではない

通常モジュールをvalへ格納したり、通常関数へ渡したりできない。

不適合：

(val selected
  parsing)

##### 5. 修飾参照
###### 5.1 区切り記号

モジュールおよび構成要素の修飾には/を使う。

parsing/parse

graphics/color/srgb8

parsing/state

###### 5.2 除算との区別

除算operatorは単独の/である。

(/ left right)


修飾名は複数componentからなる名前pathとして解析する。

###### 5.3 内部表現

graphics/color/srgb8を一つの文字列識別子として扱わず、概念的に次のpath nodeとして保持する。

QualifiedName {
  graphics,
  color,
  srgb8
}

###### 5.4 .

.はモジュール修飾には使用しない。

小数点以外の一般的な.用途は引き続き未割当とする。

##### 6. Import
###### 6.1 通常import
(import graphics/color)


これはgraphics/colorへの依存を宣言し、正式pathによる修飾参照を可能にする。

graphics/color/black


暗黙のcolor別名は生成しない。

###### 6.2 モジュール別名
(import graphics/color as color)


利用：

color/black


colorは現在のコンパイル単位内だけで有効なmodule aliasである。

###### 6.3 選択的インポート
(import graphics/color only black white srgb8)


利用：

black
white
(srgb8 255 0 0)

###### 6.4 個別項目の改名
(import graphics/color only black as blk)


導入される局所名はblkである。

blk


blackという無修飾名は導入されない。

###### 6.5 asとonlyの併用

モジュール別名と選択的インポートは同時に使用できる。

(import graphics/color as color only
  black as blk
  white)


利用可能な参照：

color/red
color/black
blk
white

###### 6.6 Import item

概念文法：

import-item :=
  name
  | name as local-name

###### 6.7 自動再公開

Importは構成要素を自動的に再公開しない。

import:
内部利用

re-export:
外部への再公開

##### 7. Importと正式identity
###### 7.1 Aliasの効果

Module aliasは新しいModuleIdを作らない。

graphics/color
color


が同じresolved moduleを指すなら、正式identityは同じである。

###### 7.2 選択的import

選択的importは新しい値や型を作らない。

black
color/black
graphics/color/black


が同じ定義を指すなら、すべて同じBindingIdへ解決する。

###### 7.3 Source spelling

利用者が書いたpath、alias、局所名、source spanはCSTとprovenanceへ保持する。

意味解析:
正式identityを使用

編集・診断:
元の綴りとsource spanを使用

###### 7.4 同じidentityの重複import

同じ正式identityを複数経路からimportした場合は、意味上同じ定義として扱う。

異なるidentityが同じ局所名へ入る場合は衝突エラーとする。

##### 8. .rpiインターフェース
###### 8.1 役割

.rpiは、対応する.rpxモジュールの公開シグネチャを記述する。

.rpx:
実装

.rpi:
外部契約

###### 8.2 Wrapper

.rpi全体を暗黙のシグネチャ本体とする。

次のようなwrapperは不要である。

(signature module-name
  ...)

###### 8.3 .rpi内のimport

.rpi内でもimportを許可する。

(import graphics/color as color)

(type render
  (fn color/color bytes))


Interface importは型・シグネチャ・effectなどの名前解決に使われ、実行時初期化を発生させない。

###### 8.4 Public module

パッケージ外へ公開されるモジュールには.rpiを要求する。

公開モジュールの指定方法はOPEN-PKG-001へ移管する。

###### 8.5 Internal module

パッケージ内部だけで使うモジュールでは.rpiを省略できる。

.rpiがない場合、実装から内部シグネチャを推論する。

###### 8.6 Script／実行entry

Scriptおよびexecutable entryには.rpiを要求しない。

##### 9. 推論シグネチャと抽象化境界
###### 9.1 .rpiなしの内部モジュール

.rpiがない内部モジュールでは、全top-level構成要素からシグネチャを推論し、同じパッケージ内から利用可能にする。

###### 9.2 .rpiありのモジュール

.rpiが存在する場合、同じパッケージの別モジュールからも.rpi外の構成要素へアクセスできない。

.rpiはパッケージ外向けの飾りではなく、実際の抽象化境界である。

###### 9.3 Interface追加

既存の内部モジュールに後から.rpiを追加した場合、.rpiに記載されていない構成要素は他モジュールから見えなくなる。

これは意図的な抽象化強化であり、内部参照に対してbreakingになり得る。

##### 10. .rpiの値仕様
###### 10.1 type

RPXのtypeは、新しい型を定義するものではなく、値bindingの型契約である。

(type parse-url
  (fn str
    (result url url-error)))


.rpiでは次を意味する。

parse-urlという値を公開し、その型は指定された型である。

###### 10.2 実装

.rpxには対応するvalを置く。

(val parse-url
  (fn (source)
    ...))

###### 10.3 型注釈の省略

.rpiに型がある公開値について、.rpx側の重複したtype宣言は省略できる。

Compilerは.rpiの型を実装の期待型として使用する。

###### 10.4 実装側にも型がある場合

.rpxにもtype宣言を書くことはできる。

この場合、.rpiと.rpxの型は同値でなければならない。

##### 11. 型の公開方法
###### 11.1 抽象型仕様
(abstract-type url)


外部へ公開されるもの：

- urlという型identity
- parameter数
- parameter kind


外部へ公開されないもの：

- constructor
- record表現
- alias先
- runtime layout

###### 11.2 Parameter付き抽象型
(abstract-type collection
  ((a type)))


外部では次のように使える。

(collection int)

###### 11.3 Constructor公開data仕様

.rpiにdataを記載した場合、型と全constructorを公開する。

(data shape
  (circle point length)
  (rectangle point size)
  (path path-data))


外部利用者は値の構築、pattern match、網羅性検査を行える。

###### 11.4 実装との一致

Constructor公開data仕様と実装のdataは、次について一致しなければならない。

- 型名
- parameter数
- parameter kind
- constructor集合
- constructor名
- payload数
- payload型
- 再帰構造

###### 11.5 type-alias

Interface内の：

(type-alias title str)


は、titleとstrの透明な型等式を外部へ公開する。

title ≃ str


別のnominal型identityが必要なら、実装でdataを定義し、interfaceではabstract-typeとして公開する。

##### 12. 名前付きシグネチャ
###### 12.1 基本構文
(signature ordered
  (abstract-type element)

  (type compare
    (fn element element int)))

###### 12.2 日本語上の意味
signature:
モジュール仕様
モジュール契約

named signature:
名前付きシグネチャ
再利用可能なモジュール契約

###### 12.3 .rpiとの違い
.rpi:
特定のコンパイル単位に対応する匿名シグネチャ

(signature name ...):
複数のモジュールやFunctorで再利用できる名前付きシグネチャ

###### 12.4 Signature identity

名前付きシグネチャには正式なSignatureIdを与える。

ただし、シグネチャ適合はSignatureIdの一致ではなく、要求構造に基づいて判定する。

##### 13. シグネチャ指定
###### 13.1 基本構文
(module integer-order
  implements ordered

  ...)


日本語では次のように説明する。

signature ascription:
シグネチャ指定
モジュール契約の適用

###### 13.2 適合判定

シグネチャ適合は構造的に行う。

モジュールは、シグネチャが要求する型・値・下位モジュールを適合する形で提供しなければならない。

###### 13.3 追加構成要素

実装側は、シグネチャにない追加の構成要素を持てる。

ただし、シグネチャ指定後は、それらを外部から隠す。

###### 13.4 抽象型identity

同じシグネチャを満たす別モジュールの抽象型は、原則として別identityを持つ。

module-a/element
≠
module-b/element

###### 13.5 実装内部のalias

実装内部で：

(type-alias element int)


としていても、適用シグネチャが：

(abstract-type element)


なら、外部からelement = intだとは分からない。

##### 14. 下位モジュール仕様
###### 14.1 名前付きシグネチャ
(signature parser-api
  (type parse
    (fn str parse-result)))


親シグネチャ：

(signature document-api
  (module parser
    implements parser-api))


これは、parser-apiを満たすparser下位モジュールを要求する。

###### 14.2 インラインシグネチャ

短い、その場限りの仕様には無名シグネチャを使える。

(module metadata
  (signature
    (type author
      (fn document str))))

###### 14.3 下位モジュール型の参照

公開された下位モジュールの型componentは修飾名で参照できる。

parser/state

###### 14.4 抽象型の独立性

同じparser-apiを満たしても、異なるモジュールの抽象型は同一ではない。

fast-parser/state
≠
strict-parser/state

###### 14.5 非公開型の漏出

公開シグネチャから、非公開モジュールまたは非公開型を参照できない。

public signature exposes a private type


として静的エラーにする。

##### 15. シグネチャの精緻化と型共有
###### 15.1 基本構文
(refine signature-expression
  refinement ...)

###### 15.2 抽象型の具体化
(refine set-api
  (type-alias element int))


これは、set-apiのelement型がintと同一であるという型等式を追加する。

###### 15.3 別モジュールとの型共有
(refine set-api
  (type-alias element ordering/element))


意味：

結果signatureのelement
=
ordering/element

###### 15.4 日本語用語
signature refinement:
シグネチャの精緻化
シグネチャの具体化

type sharing:
型共有

type equality constraint:
型同一性制約

###### 15.5 許可される精緻化

v1では、抽象型componentへの型等式追加だけを許可する。

###### 15.6 禁止される変更

refineで次を行えない。

- 値の型を別の型へ変更する
- constructorを追加・削除する
- 具体型を矛盾する別型へ変更する
- 仕様項目を削除する

###### 15.7 Parameter付き型constructor

Parameter数とkindが一致する場合、型constructor同士の等式を認める。

(refine collection-api
  (type-alias collection list))

###### 15.8 下位モジュール内の型

修飾型pathも精緻化対象にできる。

(refine storage-api
  (type-alias key/element str))

##### 16. Functor
###### 16.1 概念

Functorは、モジュールを受け取り、新しいモジュールを生成するモジュールレベルの関数である。

通常関数:
値 → 値

Functor:
モジュール → モジュール

###### 16.2 基本構文
(functor make-set
  (ordering implements ordered)

  returns
    (refine set-api
      (type-alias element ordering/element))

  declaration ...)

###### 16.3 Parameter

Functor parameterはモジュールparameterであり、名前と入力シグネチャを持つ。

Functor本体では次のように参照する。

ordering/element

ordering/compare

###### 16.4 結果シグネチャ

Functorは結果シグネチャを明示する。

Functor本体に追加helperがあっても、結果シグネチャにないものは適用結果の外部から見えない。

###### 16.5 Fixed arity

Functorはfixed arityとする。

引数不足および引数過剰は静的エラーである。

###### 16.6 通常値ではない

Functorをvalへ保存したり、通常関数へ渡したりできない。

高階Functorはv1では導入しない。

##### 17. Functor適用
###### 17.1 基本構文
(module integer-set
  apply make-set integer-order)


これはmake-setをinteger-orderへ適用し、結果モジュールをinteger-setへ束縛する。

###### 17.2 複数引数
(module result
  apply make-map key-order value-api)


各引数は対応する入力シグネチャを満たさなければならない。

###### 17.3 名前付きモジュールのみ

Functor引数は、正式ModuleIdを持つ名前付きモジュールpathに限定する。

無名inline moduleや第一級モジュール値は渡せない。

###### 17.4 適用結果の再利用

Functor適用結果を別Functorへ渡す場合は、一度名前付きモジュールへ束縛する。

適合：

(module integer-set
  apply make-set integer-order)

(module serialized-set
  apply make-serializer integer-set)

##### 18. 適用的Functor
###### 18.1 基本規則

v1のFunctorは適用的である。

同じFunctorId
+
同じ引数ModuleId列
→
同じ結果型identity

###### 18.2 同一入力
(module set-a
  apply make-set integer-order)

(module set-b
  apply make-set integer-order)


では、対応する生成型は同一である。

set-a/set
=
set-b/set

###### 18.3 異なる入力
(module integer-set
  apply make-set integer-order)

(module string-set
  apply make-set string-order)


では、結果型は異なる。

integer-set/set
≠
string-set/set

###### 18.4 Aliasの影響

Import aliasや再エクスポートpathはFunctor結果identityに影響しない。

解決済みFunctorIdとModuleIdを使用する。

###### 18.5 生成的Functor

適用ごとに新しい型identityを作る生成的Functorはv1では導入しない。

##### 19. 再公開とシグネチャ合成
###### 19.1 module-alias

既存モジュールを下位モジュールとして公開する。

(module-alias color graphics/color)


新しいModuleIdを作らず、元モジュールのidentityを維持する。

###### 19.2 re-export

既存モジュールの構成要素を現在のモジュール直下で再公開する。

(re-export graphics/color only
  black
  white
  srgb8 as make-srgb8)

###### 19.3 Identity

再公開は新しい値・型・constructorを作らない。

追加されるのは公開pathだけである。

###### 19.4 Dataの原子性

Data型に関係する公開は、次を一つのgroupとして扱う。

- 親data型
- 全constructor
- constructor固有型


個別constructorだけを再エクスポートする機能はv1では導入しない。

###### 19.5 include

シグネチャ内で別シグネチャの構成要素を取り込める。

(signature extended-api
  (include base-api)

  ...)

###### 19.6 衝突

異なるidentityを持つ同名componentがincludeまたはre-exportで衝突した場合は静的エラー。

同じ正式identityなら統合可能である。

###### 19.7 実装moduleのinclude

実装モジュールへ別モジュールの全構成要素を無修飾で取り込む一般的なincludeはv1では導入しない。

通常は修飾importを使う。

##### 20. 正式identity
###### 20.1 PackageInstanceId

依存解決後の特定のパッケージ実体を識別する。

概念的には次を含む。

- package名
- 解決済みversion
- source identity
- resolutionまたはcontent hash


具体形式はOPEN-PKG-001へ移管する。

###### 20.2 ModuleId
ModuleId =
PackageInstanceId
+
正規化済みmodule path

###### 20.3 DefinitionId

Named declarationの正式identityは概念的に次から作る。

DefinitionId =
ModuleId
+
namespace
+
正規化済み宣言名


Source byte offsetは使用しない。

###### 20.4 BindingId

値bindingの正式identityである。

Import aliasや再エクスポートで変化しない。

###### 20.5 TypeId

Nominal型定義の正式identityである。

type-aliasは新しいTypeIdを生成しない。

###### 20.6 ConstructorId
ConstructorId =
親TypeId
+
constructor名


Constructor順序だけには依存させない。

Runtime tag番号とは区別する。

###### 20.7 SignatureId
SignatureId =
ModuleId
+
signature名


適合判定そのものは構造的に行う。

###### 20.8 FunctorId
FunctorId =
ModuleId
+
functor名

###### 20.9 Functor適用結果
FunctorApplicationKey =
FunctorId
+
引数ModuleId列


結果内の型identityは、これに結果definition pathを加えて決める。

###### 20.10 Rename

公開宣言のrenameは、原則として新しいidentityを生成する。

互換性を維持する場合は、旧名をaliasまたは再エクスポートとして残す。

###### 20.11 Source span

Source spanは正式identityへ含めず、provenanceとして別に保持する。

##### 21. 分割コンパイルと適合試験
###### 21.1 Interface metadata

.rpiまたは推論シグネチャからcompiled interface metadataを生成する。

最低限、次を含む。

- ModuleId
- 公開BindingIdと型
- 公開TypeId
- 抽象型のkindとparameter
- 公開dataのconstructor集合
- ConstructorIdとpayload型
- 公開下位モジュール
- 公開シグネチャ
- 公開Functor
- effect row
- 型共有制約
- 再エクスポートpath
- metadata format version

###### 21.2 非公開情報

外部metadataへ次を含めない。

- private binding
- private constructor
- abstract typeの内部表現
- private nested module
- private helper

###### 21.3 InterfaceHash

正規化済みの公開シグネチャから意味上のinterface hashを生成する。

InterfaceHash =
hash(normalized public signature)

###### 21.4 Hashに含めるもの
- 正式identity
- 公開型
- 公開effect
- constructor集合
- abstract／constructor公開の区別
- 下位モジュール仕様
- Functor signature
- 型共有制約

###### 21.5 Hashに含めないもの
- whitespace
- comment
- indentation
- private implementation
- import alias spelling
- 公開APIに影響しないsource span

###### 21.6 Documentation hash

型検査用のSemanticInterfaceHashと、文書生成用のDocumentationHashを分離できる。

###### 21.7 再コンパイル

依存先の実装が変わってもInterfaceHashが同じなら、依存モジュールの再型検査を省略できる。

InterfaceHashが変わった場合、依存モジュールを再型検査する。

###### 21.8 ABI hash

Binary representationやcalling conventionに関するAbiHashは、semantic interface hashと分離する。

詳細はOPEN-KER-001およびbackend仕様へ移管する。

###### 21.9 適合試験 MOD-01
(import graphics/color)


期待結果：

graphics/colorが依存moduleとして解決される
暗黙aliasは生成されない
graphics/color/blackで参照可能

###### 21.10 適合試験 MOD-02
(import graphics/color as color only
  black as blk
  white)


期待結果：

color/black:
有効

blk:
有効

white:
有効

black:
無修飾では未束縛

###### 21.11 適合試験 MOD-03
// url.rpi

(abstract-type url)

(type parse-url
  (fn str
    (result url url-error)))

// url.rpx

(data url
  (validated-url internal-url-data))

(val parse-url
  (fn (source)
    ...))


期待結果：

success

外部からurl型を参照可能
validated-urlは不可視
.rpiの型をval実装の期待型として使用

###### 21.12 不適合試験 MOD-04

.rpi：

(data option
  ((a type))

  none
  (some a))


.rpx：

(data option
  ((a type))

  none
  (some a)
  unknown)


期待結果：

static error:
implementation data representation does not match interface

extra constructor:
unknown

###### 21.13 適合試験 MOD-05
(signature ordered
  (abstract-type element)

  (type compare
    (fn element element int)))

(module integer-order
  implements ordered

  (type-alias element int)

  (val compare
    (fn (left right)
      ...))

  (val helper
    ...))


期待結果：

integer-orderはorderedを満たす
helperはmodule内部で利用可能
helperは外部から不可視
elementは外部では抽象型

###### 21.14 適合試験 MOD-06
(module set-a
  apply make-set integer-order)

(module set-b
  apply make-set integer-order)


期待結果：

set-a/setとset-b/setは同じTypeId

###### 21.15 不適合試験 MOD-07
module-a imports module-b
module-b imports module-a


期待結果：

static error:
cyclic module dependency

###### 21.16 適合試験 MOD-08
(module-alias palette graphics/color)


期待結果：

paletteはgraphics/colorと同じModuleIdを参照
内部TypeId・BindingId・ConstructorIdを維持

###### 21.17 不適合試験 MOD-09
(signature public-api
  (type expose
    (fn private-module/internal-type str)))


private-moduleまたはinternal-typeが非公開なら、期待結果：

static error:
public signature exposes a private type

##### 22. 移管先OPEN・下位項目・状態
###### 22.1 `OPEN-PKG-001`

次を移管する。

- package manifest
- package名とversion
- dependency alias
- version constraint
- package source
- lockfile
- PackageInstanceIdの具体形式
- source root
- interface root
- public module一覧
- executable entry
- script package
- local path package
- 複数version共存
- package feature

###### 22.2 `OPEN-MAC-001`

次を移管する。

- macro phase
- macro import
- macro生成DefinitionId
- hygiene
- stable generated identity
- intentional capture

###### 22.3 `OPEN-KER-001`

次を移管する。

- ABI hash
- ForeignValue
- trusted adapter ABI
- runtime metadata
- validator boundary

###### 22.4 `OPEN-MOD-REC-001`

将来項目として次を移管する。

- recursive module
- signature-only recursion
- module cycle initialization
- cyclic TypeId group


v1のmodule dependency graphはDAGとする。

###### 22.5 `OPEN-MOD-FC-001`

将来項目として次を移管する。

- first-class module
- module pack／unpack
- existential package
- runtime module selection
- first-class moduleとFunctorの相互作用

###### 22.6 `OPEN-MOD-GEN-001`

将来項目として次を移管する。

- generative Functor
- 適用ごとのfresh TypeId
- capability生成
- anonymous module argument

###### 22.7 下位項目

```text
OPEN-MOD-001A
コンパイル単位・外側module・下位module
→ RESOLVED

OPEN-MOD-001B
import・alias・選択的import・修飾参照
→ RESOLVED

OPEN-MOD-001C
.rpi・推論signature・抽象型
→ RESOLVED

OPEN-MOD-001D
名前付きsignature・signature ascription
→ RESOLVED

OPEN-MOD-001E
下位module仕様・signature refinement・型共有
→ RESOLVED

OPEN-MOD-001F
Functor
→ RESOLVED

OPEN-MOD-001G
module-alias・re-export・signature include
→ RESOLVED

OPEN-MOD-001H
正式identity・separate compilation
→ RESOLVED

OPEN-MOD-001I
recursive／first-class moduleのv1範囲判定
→ RESOLVED
```
###### 22.8 最終状態

```text
OPEN-MOD-001:
RESOLVED
```

本解決により、RPXは次を提供する。

- ファイル単位の分割コンパイル
- inline下位モジュール
- 明示的で柔軟なimport
- 人間向けの.rpi契約
- 実装からのシグネチャ推論
- 抽象型による表現隠蔽
- constructor公開data仕様
- 再利用可能な名前付きシグネチャ
- 構造的なシグネチャ適合
- 型共有
- 適用的Functor
- identityを維持する再公開
- 安定した型・値・constructor identity
- interface hashによる差分コンパイル

#### 13.10 `PKG-001` パッケージmanifest・依存解決・ワークスペース・リソース
##### DD-001 決定概要
###### DD-001.1 状態
Status:
RESOLVED

Scope:
パッケージmanifest
パッケージ名とversion
source／interface root
公開モジュール
実行エントリ
スクリプト
依存宣言
依存先source
version制約
依存解決
lockfile
PackageInstanceId
複数version
ワークスペース
パッケージリソース

###### DD-001.2 中心的な決定
- パッケージmanifestにはpackage.rpxmを使用する
- Manifestは評価されない制限付きRPXデータ形式とする
- パッケージ名はASCII lowercase kebab-caseとする
- パッケージversionはmajor.minor.patchを基礎とする
- source rootは一つとし、既定値を"src"とする
- interface rootは省略可能な一つとし、既定値を"interface"とする
- 公開モジュールはmanifestで明示する
- 公開モジュールには.rpiを要求する
- 実行エントリはmanifestで明示し、.rpiを要求しない
- Manifestなしのscriptは単一ファイルに限定する
- 依存には局所的な依存別名を与える
- Version単独指定は完全一致として扱う
- Registry依存とlocal path依存をv1で扱う
- Lockfileにはrpx.lockを使用する
- 通常buildはlockfileを暗黙更新しない
- 通常の初回解決では最高の安定versionを選ぶ
- 必要な場合は同じパッケージの複数versionを共存させる
- ワークスペースにはworkspace.rpxmを使用する
- ワークスペース全体で一つのrpx.lockを共有する
- リソースはパッケージ内の論理identityとして扱う
- Resource rootの既定値を"resources"とする
- 配布対象resourceはmanifestで明示する
- 生成source・生成resource・任意build stepは別項目へ移管する

##### 0. 適用範囲
###### 0.1 本項目が定めるもの

本項目は次を規定する。

- パッケージとモジュールの関係
- package.rpxmの形式
- パッケージ名
- パッケージversion
- source root
- interface root
- public module
- internal module
- executable entry point
- manifestなしscript
- dependency alias
- dependency source
- version constraint
- development dependency
- dependency resolution
- rpx.lock
- PackageInstanceId
- 複数version共存
- workspace.rpxm
- workspace member
- workspace共通lockfile
- package resource
- resource root
- resource identity
- 配布resource一覧

###### 0.2 本項目が直接定めないもの

次は別のOPEN項目へ移管する。

- Registry protocol
- パッケージ公開・署名・失効
- Git／任意URL dependency
- optional dependency
- feature system
- platform条件付きdependency
- mainの最終的な型
- test discoveryとtest fixture
- 任意build script
- generated source
- generated resource
- ABI hash
- trusted adapter
- resource decoderのtrusted boundary
- runtime resource lifetime

##### 1. パッケージ
###### 1.1 定義

パッケージは、複数のモジュール、外部依存、リソースおよび実行エントリをまとめる配布単位である。

package
├─ package manifest
├─ source modules
├─ interface files
├─ public modules
├─ internal modules
├─ entry points
├─ resources
└─ dependencies

###### 1.2 モジュールとの違い
モジュール:
名前空間、型抽象化、分割コンパイルの単位

パッケージ:
配布、version付け、依存解決の単位


一つのパッケージは複数の外側モジュールを含められる。

###### 1.3 パッケージの種類

パッケージを排他的なlibraryまたはexecutableへ分類しない。

public moduleがある:
libraryとして利用可能

entry pointがある:
実行programを生成可能


一つのパッケージは両方を持てる。

##### 2. パッケージmanifest
###### 2.1 ファイル名

パッケージmanifestのファイル名は次とする。

package.rpxm


拡張子の役割：

.rpx:
RPX実装source

.rpi:
RPX interface

.rpxm:
RPX manifest

###### 2.2 Package root

package.rpxmが存在するdirectoryをpackage rootとする。

package-root/
├─ package.rpxm
├─ src/
├─ interface/
└─ resources/

###### 2.3 制限付きRPX形式

ManifestはRPXに似たS式を使用するが、RPXプログラムとして評価しない。

Manifestでは次を禁止する。

- 関数適用
- val／data等の通常宣言
- 名前解決
- macro展開
- effect実行
- 外部ファイルの動的読込み
- 環境変数の暗黙参照
- 条件分岐による構成変更

###### 2.4 静的schema

Manifest parserは規定されたfieldとliteralだけを受理する。

不適合：

(package document
  (val version
    "1.0.0"))


適合：

(package document
  version "1.0.0")

###### 2.5 未知field

未知fieldは無視せず静的エラーにする。

unknown package field:
  soruce-root

did you mean:
  source-root

##### 3. Manifestの基本構文
###### 3.1 最小形
(package document
  format-version 1
  version "1.0.0")

###### 3.2 明示形
(package document
  format-version 1
  version "1.0.0"

  source-root "src"
  interface-root "interface"
  resource-root "resources")

###### 3.3 Fieldの括弧

単一値fieldは括弧で囲まない。

version "1.0.0"
source-root "src"


複数項目または入れ子構造を持つfieldは括弧でまとめる。

(public-modules
  document
  document/query)

###### 3.4 Format version
format-version 1


はmanifest schemaのversionであり、パッケージrelease versionとは異なる。

format-version:
manifest形式のversion

version:
パッケージreleaseのversion


未対応format versionは明確に拒否する。

##### 4. パッケージ名
###### 4.1 基本規則

パッケージ名はASCII lowercase kebab-caseとする。

適合：

document
graphics-kit
japanese-typesetting
render2
layout-engine


不適合：

Document
graphics_kit
日本語組版
-graphics
graphics-
graphics--kit

###### 4.2 用途

正式パッケージ名は次に使用される。

- Registry identity
- Dependency宣言
- Lockfile
- Cache
- PackageInstanceId
- CLI表示

###### 4.3 表示名

Unicodeを含む人間向け表示名は、将来の任意fieldとして追加できる。

display-name "日本語組版"


表示名は正式identityに影響しない。

##### 5. パッケージversion
###### 5.1 基本形式

Versionはmajor.minor.patchを基礎とする。

version "1.2.3"


文字列literalとして記述し、専用version parserで検証する。

###### 5.2 Version要素
1.2.3
│ │ └─ patch
│ └─── minor
└───── major

###### 5.3 互換性の一般原則
major:
互換性を壊す変更

minor:
後方互換な機能追加

patch:
後方互換な修正

###### 5.4 Breaking changeの例

少なくとも次は公開APIのbreaking changeである。

- 公開bindingの削除またはrename
- 公開値の非互換な型変更
- 公開effect rowの拡大
- 公開dataへのconstructor追加・削除
- constructor payloadの変更
- 公開abstract TypeIdの変更
- Functor signatureの非互換変更
- public module pathの削除または変更

###### 5.5 Pre-release

次はpre-release versionである。

2.0.0-alpha.1
2.0.0-beta.2
2.0.0-rc.1


Pre-releaseは依存条件で明示的に要求された場合だけ自動選択対象とする。

##### 6. Source rootとinterface root
###### 6.1 Source root

実装sourceを探索する基準directoryを一つ指定できる。

source-root "src"


省略時の既定値：

src

###### 6.2 Module path

Source rootからの相対pathがmodule pathになる。

src/document/parser.rpx
→ document/parser

###### 6.3 Interface root

.rpiを探索する基準directoryを省略可能な一つとして指定できる。

interface-root "interface"


省略時の既定値：

interface

###### 6.4 Interface対応
src/document/parser.rpx

interface/document/parser.rpi


は同じmodule path：

document/parser


に対応する。

###### 6.5 Root数

v1では次に限定する。

source root:
一つ

interface root:
省略可能な一つ


複数rootと探索順序は導入しない。

###### 6.6 Root pathの制限

Root pathはpackage root内の相対pathでなければならない。

禁止：

- 絶対path
- `..` component
- symbolic linkによるpackage外脱出

##### 7. 公開モジュール
###### 7.1 Manifest構文
(public-modules
  document
  document/query)


public-modulesに書かれた外側モジュールだけが、別パッケージからimportできる。

###### 7.2 .rpi必須

公開モジュールには、対応する実装とinterfaceの両方を要求する。

src/document.rpx
interface/document.rpi


Interfaceがない場合：

public module has no interface file:
  document

###### 7.3 内部モジュール

public-modulesに書かれていないコンパイル単位は内部モジュールである。

同じパッケージ内:
利用可能

別パッケージ:
import不可

###### 7.4 Internal moduleの.rpi

内部モジュールでは.rpiは任意である。

.rpiなし:
実装から内部シグネチャを推論

.rpiあり:
.rpiを同一パッケージ内でも抽象化境界として適用

###### 7.5 Inline下位モジュール

Manifestのpublic-modulesが対象とするのは、ファイルに対応する外側モジュールである。

Inline下位モジュールの公開範囲は、その外側モジュールの.rpiが決める。

###### 7.6 自動公開

.rpiが存在するだけでは自動的にpublic moduleにしない。

.rpi:
モジュールの見せ方

public-modules:
パッケージ外へ見せるか


を分離する。

##### 8. 実行エントリ
###### 8.1 Manifest構文
(entry-points
  document-cli
  preview-server)


一つのパッケージは複数の実行エントリを持てる。

###### 8.2 Entry module

各entry pointは、source root内の外側モジュールpathを表す。

document-cli
→ src/document-cli.rpx

###### 8.3 .rpi

Entry moduleには.rpiを要求しない。

同じmoduleがpublic moduleも兼ねる場合は、public moduleとして.rpiが必要である。

###### 8.4 main

Entry moduleはmainという値を提供しなければならない。

mainの正確な型、許容effect、引数、exit statusおよびasync対応は実行意味論・エラー・並行性の各項目へ移管する。

###### 8.5 一module一entry

v1では、一つのentry moduleに一つのmainを対応させる。

複数programが必要な場合は、薄いentry moduleを複数作る。

###### 8.6 実行契約

Entry contractはlibrary向け.rpiとは別に検査する。

同じmoduleがpublicでも、mainを.rpiへ書くことは要求しない。

##### 9. Manifestなしscript
###### 9.1 基本形

単一の.rpxファイルは、manifestなしでscriptとして実行できる。

report.rpx


Compilerは一時的な暗黙script packageとして扱う。

###### 9.2 必要な値

Scriptはmainを提供する。

(val main
  (fn ()
    ...))

###### 9.3 単一ファイル制限

Manifestなしscriptは単一ファイルに限定する。

別の.rpxファイルを暗黙に同一packageとして探索しない。

複数ファイルが必要になった場合はpackage.rpxmを作成する。

###### 9.4 外部dependency

Standalone scriptは、標準library以外の外部dependencyを持たない。

外部dependencyを使用する場合は、packageまたはworkspaceのmanifestとlockfileを使用する。

###### 9.5 Public API

Manifestなしscriptはpublic moduleを持たず、.rpiを要求しない。

##### 10. 依存宣言
###### 10.1 基本構文
(dependencies
  (graphics
    package graphics-kit
    version ">=1.2.0 <2.0.0")

  (markup
    package rpx-markup
    version "2.1.0"))

###### 10.2 Dependency alias

各依存項目の先頭名は、現在のパッケージ内で使う依存別名である。

graphics:
依存別名

graphics-kit:
正式パッケージ名


Source：

(import graphics/color as color)

###### 10.3 正式identity

依存別名はPackageInstanceIdに含めない。

同じresolved packageを別aliasで参照しても、正式identityは同じである。

###### 10.4 正式パッケージ名

package fieldには正式パッケージ名を書く。

package graphics-kit


パッケージ名は専用name grammarを持つため、文字列literalにしない。

###### 10.5 Aliasの重複

同じdependency aliasを複数回定義できない。

duplicate dependency alias:
  graphics

##### 11. Version constraint
###### 11.1 完全一致

単一versionは完全一致として扱う。

version "1.2.3"


許可：

1.2.3


拒否：

1.2.4
1.3.0
2.0.0

###### 11.2 範囲指定

明示的な比較演算子で範囲を記述する。

version ">=1.2.0 <2.0.0"

###### 11.3 初期演算子集合

v1では次を許可する。

=
>
>=
<
<=

###### 11.4 条件の結合

空白で並べた複数条件はANDとする。

>=1.2.0 <2.0.0


OR条件はv1では導入しない。

###### 11.5 Pre-release

Pre-releaseは、constraintがpre-release versionを明示的に含む場合だけ候補にする。

##### 12. Dependency source
###### 12.1 既定Registry

pathまたは明示sourceがないdependencyは、既定registryから解決する。

(graphics
  package graphics-kit
  version ">=1.2.0 <2.0.0")


実際に使用したregistry identityはlockfileへ記録する。

###### 12.2 Local path
(theme
  package document-theme
  version "0.4.0"
  path "../document-theme")


Pathは現在のpackage.rpxmがあるdirectoryからの相対pathとして解釈する。

###### 12.3 Local packageの検証

Path先のmanifestにある正式パッケージ名が、dependencyのpackage fieldと一致しなければならない。

Version条件がある場合、path先のversionも条件を満たさなければならない。

###### 12.4 Supported source

v1で正式に扱う依存sourceは次とする。

- 既定registry
- local path
- workspace member

###### 12.5 Git／URL

Git repository、branch、commitおよび任意URL dependencyはv1では導入しない。

###### 12.6 Sourceの排他性

一つの依存項目で複数のsourceを同時指定できない。

##### 13. Development dependency
###### 13.1 構文
(development-dependencies
  (testing
    package rpx-testing
    version "1.0.0"))

###### 13.2 用途

Development dependencyは次に使用する。

- test
- benchmark
- 開発tool


通常のlibrary利用者の依存graphには含めない。

###### 13.3 Public APIへの漏出

Development dependencyの型、effect、signatureまたはFunctorを公開.rpiへ露出できない。

public interface depends on a development-only package


として静的エラーにする。

###### 13.4 Optional dependency

Optional dependencyおよびfeature連動依存はv1では導入しない。

将来OPEN-PKG-FEAT-001へ移管する。

##### 14. Public dependency
###### 14.1 定義

依存先の型その他のidentityが、自パッケージの公開.rpiに現れる場合、その依存は公開依存である。

(type create
  (fn markup/fragment document))

###### 14.2 自動導出

Public dependencyをmanifestへ手動指定させない。

Compilerが.rpiから自動的に導出し、interface metadataへ記録する。

###### 14.3 用途

Public dependency情報は次に利用できる。

- 互換性検査
- Documentation
- Registry metadata
- Dependency更新の影響分析

##### 15. 依存解決
###### 15.1 Manifestの役割

Manifestは受理可能なversion・source条件を記述する。

具体的な依存graphはlockfileで固定する。

###### 15.2 初回解決

Lockfileがない場合、条件を満たす候補から依存graphを解決する。

通常は、条件を満たす最高の安定versionを選ぶ。

###### 15.3 Pre-release

Pre-releaseは明示的に要求された場合だけ選択する。

###### 15.4 統合

同じ正式パッケージ名、同じsourceおよび互換な条件について、一つのversionで全要求を満たせる場合は、一つのpackage instanceへ統合する。

###### 15.5 分割

一つのversionで全条件を満たせない場合、通常のpure packageでは複数のPackageInstanceIdへ分割できる。

###### 15.6 決定性

同じmanifest、registry状態およびlockfile状態からは、同じ解決結果を得なければならない。

候補の選択およびtie-break規則を決定的にする。

##### 16. 同一パッケージの複数version
###### 16.1 基本方針

依存graph内で、同じ正式パッケージ名の複数version共存を許可する。

graphics-kit 1.5
graphics-kit 2.1


は別のPackageInstanceIdを持つ。

###### 16.2 型identity

同じmodule path・型名を持っていても、package instanceが異なれば別の型である。

graphics-kit@1/color
≠
graphics-kit@2/color

###### 16.3 直接依存での明示
(dependencies
  (graphics-v1
    package graphics-kit
    version "1.5.0")

  (graphics-v2
    package graphics-kit
    version "2.1.0"))


Source：

(import graphics-v1/color as old-color)
(import graphics-v2/color as new-color)

###### 16.4 単一instance制約

Trusted adapter、process-global plugin等で複数instanceを禁止する必要がある場合、その制約はOPEN-KER-001へ移管する。

##### 17. Lockfile
###### 17.1 ファイル名

Lockfileのファイル名は次とする。

rpx.lock

###### 17.2 役割
package.rpxm:
依存の許容条件

rpx.lock:
完全な解決済み依存graph

###### 17.3 通常build

Lockfileが存在し、manifestと整合する場合、記録済みgraphをそのまま使用する。

通常buildはlockfileを変更しない。

###### 17.4 初回build

Lockfileがない場合：

依存解決
→ lockfile生成
→ build


を行う。

###### 17.5 不整合

Lockfileのversionがmanifest条件を満たさない場合、通常buildでは自動更新せずエラーにする。

lockfile is not consistent with the package manifest

###### 17.6 更新

Version変更は明示的なdependency update操作によってのみ行う。

CLIの具体的な構文は別項目へ移管する。

###### 17.7 部分更新

一つの依存だけを更新する操作を許容する。

Resolverは、固定可能なlock entryを維持し、必要な範囲だけ再解決する。

##### 18. Lockfileの内容
###### 18.1 Package node

各解決済みpackage nodeについて、少なくとも次を記録する。

- 正式package名
- exact version
- source identity
- artifact checksum
- package content hash
- dependency edges
- dependency alias
- PackageInstanceId生成に必要な情報

###### 18.2 Registry package

Registry packageではartifact checksumを必須とする。

Checksum不一致は重大な完全性エラーである。

###### 18.3 Local path package

Local path packageは開発中の可変sourceとして扱う。

Source編集によってlockfile破損とはしないが、再コンパイル対象にはする。

###### 18.4 Version control

Workspaceのrpx.lockはversion controlへ含めることを推奨する。

Library作者のlockfileは、そのlibraryを利用する別workspaceの解決graphへ強制適用しない。

###### 18.5 Offline build

Offline buildでは：

- networkへ接続しない
- registry indexを更新しない
- lockfileを変更しない
- cache内artifactだけを使う


必要artifactがなければ明示的に失敗する。

##### 19. Package identityとcontent hash
###### 19.1 論理identityと内容identity

次を分離する。

LogicalPackageInstanceId:
依存graph上のpackage nodeの同一性

PackageContentHash:
現在のpackage内容の完全性とcache identity

###### 19.2 Registry package

概念的には次を使用する。

PackageInstanceId:
正式package名
+ exact version
+ source identity

PackageContentHash:
registry artifactの内容hash

###### 19.3 Workspace／local package

Workspace内のsource編集だけで、全TypeIdを毎回変更しない。

PackageInstanceId:
workspace dependency graph上の論理node

PackageContentHash:
編集に応じて変更

###### 19.4 Content hash対象

少なくとも次を含む。

- package.rpxm
- .rpx
- .rpi
- 配布resource
- 必要なpackage metadata


次は含めない。

- file mtime
- 所有者情報
- 絶対path
- editor temporary file
- build output
- OS固有metadata

##### 20. ワークスペース
###### 20.1 定義

ワークスペースは、複数のローカルパッケージを一つの開発単位として扱う仕組みである。

パッケージ:
配布・version付け・依存の単位

ワークスペース:
複数パッケージの共同開発単位

###### 20.2 Manifest

ワークスペースmanifestのファイル名は次とする。

workspace.rpxm

###### 20.3 基本構文
(workspace
  format-version 1

  (members
    "document-core"
    "document-render"
    "document-cli"))

###### 20.4 Member

各memberは独自のpackage.rpxmを持たなければならない。

###### 20.5 Member path

Member pathはworkspace rootからの明示的な相対pathとする。

Directory globはv1では導入しない。

###### 20.6 Memberの外部配置

原則としてworkspace root内のpackageだけをmemberにできる。

Workspace外packageはpath dependencyとして扱う。

###### 20.7 Package名重複

同一workspace内で正式パッケージ名を重複させられない。

###### 20.8 Nested workspace

ワークスペースの入れ子を禁止する。

一つのpackageは高々一つのworkspaceに所属する。

##### 21. Workspace依存・リソース・適合試験
###### 21.1 共通lockfile

Workspace全体でrootのrpx.lockを共有する。

Member directory内に個別lockfileを置かない。

###### 21.2 Member単独build

一つのmemberだけをbuildする場合も、workspace rootのlockfileを使用する。

###### 21.3 Workspace member優先

通常dependencyの正式package名とversion条件に適合するworkspace memberが存在する場合、source指定がなければworkspace memberを優先する。

(document
  package document-core
  version ">=1.2.0 <2.0.0")

###### 21.4 明示source
source workspace


はworkspace memberを必須にする。

source registry


はworkspace memberを無視し、registryから解決する。

###### 21.5 Member version

Workspace memberもpackage versionを持ち、依存側のversion条件を満たさなければならない。

###### 21.6 Member依存graph

Workspace member間の依存graphもDAGでなければならない。

Cycleは静的構成エラーとする。

###### 21.7 Workspace外path dependency

開発時には許可できるが、package publication時には禁止する。

Release／reproducible modeで警告またはエラーにできる。

###### 21.8 Workspace identity

v1では明示的な永続Workspace UUIDを導入しない。

絶対filesystem pathをPackageInstanceIdへ直接含めない。

###### 21.9 Resource root

パッケージは一つのresource rootを持てる。

resource-root "resources"


省略時の既定値：

resources

###### 21.10 Resource一覧

配布対象resourceはmanifestで明示する。

(resources
  "styles/default.css"
  "images"
  "fonts/body.woff2"
  "locale")


File指定はそのfileだけを含める。

Directory指定は、その配下の通常fileを再帰的に含める。

Globはv1では導入しない。

###### 21.11 Resource path

Resource pathはresource rootからの正規化済み相対pathである。

禁止：

- 絶対path
- `..`
- 空component
- package root外への脱出


規範的separatorは/とする。

###### 21.12 Symbolic link

Resourceとしてsymbolic linkを禁止する。

###### 21.13 Resource identity
PackageResourceId
=
PackageInstanceId
+
normalized resource path


異なるpackageの同じpathは別resourceである。

###### 21.14 Resource参照

Sourceからは次の特殊形式で参照する。

(resource "styles/default.css")


これは文字列やOS pathではなく、package-resource型の静的handleを生成する。

###### 21.15 Pure／effectfulの区別
(resource "path"):
pureなresource identityの構築

resource内容の読込み:
resource effectを要求

###### 21.16 Resourceの外部公開

別パッケージのresource pathを直接参照できない。

必要な場合、所有パッケージが.rpiを通じてpackage-resource値を公開する。

###### 21.17 Resource hash

ResourceはPackageContentHashへ含める。

各resourceのcontent hashもcompiled metadataへ保存できる。

###### 21.18 Generated resource

生成resource、生成sourceおよび任意build stepはOPEN-BLD-001へ移管する。

###### 21.19 Test fixture

Test fixtureはOPEN-TST-001へ移管する。

###### 21.20 適合試験 PKG-01
(package document
  format-version 1
  version "1.0.0")


期待結果：

success

source-root:
src

interface-root:
interface

resource-root:
resources

###### 21.21 不適合試験 PKG-02
(package Document
  format-version 1
  version "1.0.0")


期待結果：

static manifest error:
package name must be ASCII lowercase kebab-case

###### 21.22 適合試験 PKG-03
(public-modules
  document
  document/query)


対応する.rpxと.rpiが存在する場合：

success

###### 21.23 不適合試験 PKG-04
(public-modules
  document/query)


interface/document/query.rpiが存在しない場合：

static manifest error:
public module has no interface file

###### 21.24 適合試験 PKG-05
(entry-points
  document-cli
  preview-server)


両moduleが存在し、mainを提供する場合：

success

###### 21.25 適合試験 PKG-06
(dependencies
  (graphics
    package graphics-kit
    version ">=1.2.0 <2.0.0"))


期待結果：

graphicsをdependency aliasとして登録
条件を満たす最高の安定versionを初回解決
resolved packageをlockfileへ記録

###### 21.26 適合試験 PKG-07
(dependencies
  (theme
    package document-theme
    version "0.4.0"
    path "../document-theme"))


Path先の名前とversionが一致する場合：

success

###### 21.27 不適合試験 PKG-08

Lockfile：

graphics-kit 1.7.3


Manifest条件：

>=2.0.0 <3.0.0


期待結果：

build error:
lockfile is not consistent with the package manifest

lockfileは自動更新しない

###### 21.28 適合試験 PKG-09
(workspace
  format-version 1

  (members
    "document-core"
    "document-render"))


両directoryに有効なpackage.rpxmがあり、package名が重複しない場合：

success
workspace rootのrpx.lockを共有

###### 21.29 不適合試験 PKG-10
(resources
  "../secret.txt")


期待結果：

static manifest error:
resource path escapes the package resource root

###### 21.30 適合試験 PKG-11
(val logo
  (resource "images/logo.png"))


Resourceがmanifest一覧に含まれ、実在する場合：

success
logo : package-resource

###### 21.31 不適合試験 PKG-12
(val logo
  (resource "images/missing.png"))


期待結果：

build error:
package resource does not exist

##### 22. 移管先OPEN・下位項目・状態
###### 22.1 `OPEN-BLD-001`

次を移管する。

- 宣言的build step
- generated source
- generated resource
- code generator
- build sandbox
- build input／output declaration
- 非決定的buildの拒否
- build cache

###### 22.2 `OPEN-PKG-FEAT-001`

次を将来項目として移管する。

- optional dependency
- package feature
- feature unification
- platform条件
- feature集合とPackageInstanceId

###### 22.3 `OPEN-REG-001`

次を移管する。

- Registry protocol
- package upload
- package署名
- owner／namespace
- package yanking／失効
- checksum配布
- immutable release

###### 22.4 `OPEN-KER-001`

次を移管する。

- trusted adapter package
- process-global package制約
- 複数instance禁止条件
- ABI hash
- resource validator
- ForeignValue

###### 22.5 `OPEN-TST-001`

次を移管する。

- test entry
- test-only module
- development dependencyの可視性
- test fixture
- test resource
- 許容effect row

###### 22.6 `OPEN-ERR-001`

次を移管する。

- mainのfailure型
- exit status
- terminal failure
- cleanup

###### 22.7 `OPEN-CON-001`

次を移管する。

- async main
- cancellation
- concurrent entry point

###### 22.8 下位項目

```text
OPEN-PKG-001A
manifest・package名・version・root
→ RESOLVED

OPEN-PKG-001B
公開module・entry point・script
→ RESOLVED

OPEN-PKG-001C
dependency宣言・source・version条件
→ RESOLVED

OPEN-PKG-001D
dependency resolution・lockfile・PackageInstanceId
→ RESOLVED

OPEN-PKG-001E
workspace
→ RESOLVED

OPEN-PKG-001F
package resource
→ RESOLVED
```

##### Compiler-native package実装

頻繁に利用され、性能上の根拠がある抽象化は、通常package APIを保ったままcompilerまたはRuntimeのnative implementationへ置換できる。ただし、native implementationだけを直接特権APIとして公開しない。

- 意味上の正本はpackageの公開`.rpi`と適合試験である。
- 利用者は通常の`import`でpackageを読み込む。
- Native対象は原則として一つの独立packageへ分離する。
- CompilerはPackageInstanceId、module path、BindingId／TypeIdおよびEffect契約を維持する。
- Native版とportable版は同じobservable behaviorの適合試験を通過しなければならない。
- Native化の有無で型、Effect、Failure、Resource lifetime、source identityを変えてはならない。
- 未対応backendではportable版へfallbackできなければならない。
- 対象選定、version negotiation、ABI、bootstrap、fallbackは`OPEN-NATIVE-PKG-001`で確定する。

###### 22.9 最終状態

```text
OPEN-PKG-001:
RESOLVED
```



本解決により、RPXは次を提供する。

- 静的で評価されないpackage manifest
- 明示的なpublic module境界
- 軽量なinternal module開発
- 複数entry point
- 単一ファイルscript
- 明示的なdependency alias
- Registry／local／workspace dependency
- 厳密なversion constraint
- 再現可能なlockfile build
- 複数package versionの共存
- 共有lockfileを持つworkspace
- 安定したPackageInstanceId
- パッケージ所有のresource identity
- Resourceの明示的な配布範囲
- 生成処理を通常package解決から分離する安全な基盤

#### 13.10.1 `KER-001` Rust kernelとforeign primitive境界

##### 概要・状態

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

#### 13.10.2 `RSC-001` Resource、I/O、host-handler境界

##### 概要・状態

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

#### 13.11 `EDT-001` 編集スナップショット・トランザクション・競合・由来情報
##### DD-001 決定概要
###### DD-001.1 状態
Status:
RESOLVED

Scope:
編集可能な文書モデル
不変スナップショット
文書・ノード・トランザクションの識別
編集トランザクション
適用前条件
旧版トランザクション
競合検出
Undo／Redo
由来情報
派生ノード
逆編集
編集エンジンの公開API

###### DD-001.2 位置付け

OPEN-EDT-001は、主としてRPX言語の構文仕様ではなく、Reciplexa編集エンジンの規範的なソフトウェア仕様である。

本項目では、異なる実装でも揃えるべき意味論と公開APIを定める。一方、内部データ構造やIDのbit表現などは実装依存とする。

規範的に定めるもの:
- スナップショットの不変性
- ノード識別子の継続性
- トランザクションの原子性
- 旧版編集の再検証
- 競合分類
- Undoの意味
- 由来情報と逆編集の基本原則
- 公開API上の型と結果分類

実装依存とするもの:
- IDの具体的なbit表現
- スナップショットの内部データ構造
- Hash algorithm
- 履歴の具体的な保持上限
- 永続形式
- Network wire protocol
- Cacheおよびindex構造

##### 0. 適用範囲
###### 0.1 本項目が扱う編集

本項目が正規の編集対象とするのは、永続的なノード識別子を持つ編集可能な文書・シーンモデルである。

対象:
- 文書ノード
- シーンノード
- ノードのproperty
- 所有関係
- 明示的なノード参照

###### 0.2 直接の対象としないもの

次は本項目の正規編集対象ではない。

- RPXソーステキスト
- Lossless CST
- 型検査済みAST
- コンパイラ内部の一時IR
- Backend artifact


ソースコードのrenameやimport追加などは、IDE・refactoring仕様へ移管する。

ただし、次の識別情報は共有できる。

- PackageInstanceId
- ModuleId
- DefinitionId
- BindingId
- TypeId
- SyntaxNodeId
- マクロ展開provenance

##### 1. 編集モデルの基本原則
###### 1.1 不変値と継続的identity

通常の文書ノード値は不変とする。

一方、内容が変化しても同じ段落・図形・セクションとして追跡する必要があるため、各ノードは安定したNodeIdを持つ。

NodeValue:
不変値

NodeId:
版をまたいで同じ編集対象を追跡する識別子


例：

revision 10:
P -> Paragraph("Hello")

revision 11:
P -> Paragraph("Hello world")


値は置き換わっているが、PというNodeIdは維持される。

###### 1.2 スナップショット

文書状態は、不変のDocumentSnapshotとして表す。

概念的には次の構成を持つ。

DocumentSnapshot {
  document-id,
  revision,
  root-node-id,
  node-store,
  provenance-index
}


スナップショット取得後に現在文書が更新されても、取得済みスナップショットの論理内容は変わらない。

###### 1.3 現在文書

変化していく現在文書はDocumentHandleによって表す。

DocumentSnapshot:
特定revisionの不変状態

DocumentHandle:
現在revisionへ接続された状態付きhandle

##### 2. 文書の所有構造
###### 2.1 単一rootの所有tree

文書の親子関係は、単一rootを持つ所有treeに限定する。

- Rootは一つ
- Rootは親を持たない
- Root以外の各到達可能ノードは、ちょうど一つの親を持つ
- 所有関係のcycleは禁止


例：

document
└─ page
   ├─ paragraph
   └─ image

###### 2.2 所有と参照の分離

ノードの構造的な包含関係は所有edgeで表す。

構造上の親子ではない関連は、property内の明示的なNodeId参照として表す。

例：

TableOfContentsEntry {
  target: HeadingNodeId
}

Annotation {
  target: ParagraphNodeId
}


参照先ノードを所有しているわけではない。

###### 2.3 共有

同じノードを複数の親が所有することは禁止する。

共有が必要なものは次のいずれかで表す。

- 不変値の共有
- package-resource等のhandle共有
- NodeIdによる明示参照
- 別々のinstance nodeから共有定義を参照

###### 2.4 規範的な親子情報

所有関係の正本は、親ノードのchildren列とする。

規範データ:
親から子への所有edge

派生index:
子から親への逆index


実装は性能のために親indexを保持できるが、意味上の正本ではない。

##### 3. 文書treeの不変条件

確定済みスナップショットは、次をすべて満たさなければならない。

##### 1. RootNodeIdがnode storeに存在する
##### 2. Rootは親を持たない
##### 3. Root以外の全ノードはちょうど一つの親を持つ
##### 4. 所有edgeにcycleがない
##### 5. 同じ親のchildren列内に同一NodeIdが重複しない
##### 6. 全ノードが同じDocumentIdに所属する
##### 7. 全ノードがRootから到達可能である
##### 8. 必須propertyが存在する
##### 9. Property値がNode kindのschemaに適合する
##### 10. 強いNodeId参照が有効な対象を指す

###### 3.1 到達不能ノード

確定済みスナップショットでは、Rootから到達不能なノードを認めない。

ただし、トランザクションの仮適用中は、一時的なdetached nodeを許可する。

Transaction適用中:
一時的detached nodeを許可

Commit時:
全ノードの到達可能性を要求


Clipboardや一時作業領域は、文書スナップショットとは別の所有領域にする。

##### 4. 識別子
###### 4.1 識別子の種類
DocumentId:
一つの文書系列を識別する

NodeId:
文書系列内の一つのノードを識別する

TransactionId:
一つの編集要求を識別する

Revision:
文書系列内の版を識別する

PropertyId:
ノードschema内のpropertyを識別する


意味上のノードaddressは次である。

NodeAddress =
DocumentId + NodeId

###### 4.2 Opaque型

各IDは内部表現を隠したopaque型とする。

許可される基本操作：

- 等値比較
- Hashing
- 検証済みserialization
- Debug表示


許可しない操作：

- 数値演算
- 内部componentの分解
- 順序へ意味を持たせる比較
- 自由な文字列からの無検証生成


IDを知っていることは、編集権限を持つことを意味しない。

###### 4.3 DocumentId

新規文書作成時に発行する。

NewDocument
→ 新DocumentId
→ revision 0
→ Root NodeId発行


保存pathやファイル名とは独立したidentityである。

通常の保存、ファイル名変更、保存先変更では維持する。

###### 4.4 NodeId

必須保証：

- 同一文書系列内で一意
- 内容変更で維持
- Moveで維持
- Copyで新規発行
- 削除後に再利用しない
- Serialize／deserializeで維持


Node kindもNodeIdの生存期間中は不変とする。

Kind変更は旧ノードの削除と新ノードの作成として表す。

###### 4.5 TransactionId

トランザクション作成者が発行する。

同じ要求のnetwork retry等では、同じTransactionIdを再利用する。

同一ID + 同一content hash:
AlreadyApplied

同一ID + 異なるcontent hash:
TransactionIdentityConflict

###### 4.6 ID表現

UUID、中央連番、ActorIdとsequenceなどの具体形式は固定しない。

各IDは、将来のoffline生成を妨げないopaque表現とする。

##### 5. 保存・複製・Fork
###### 5.1 通常保存

通常保存では、次を維持する。

- DocumentId
- NodeId
- 現在revision


再読込後も同じ文書系列として扱う。

###### 5.2 Save As

通常のSave Asは、同じ文書identityを別の保存先へ書く操作とする。

DocumentId:
維持

保存path:
変更

###### 5.3 Duplicate／Fork

独立した文書を作る場合は、明示的なDuplicateまたはFork操作を使う。

- 新しいDocumentId
- 全NodeIdを新規発行
- 新しいrevision系列


対応表を結果として返せる。

CloneMap {
  old-node-id -> new-node-id
}

###### 5.4 内部参照の複製

複製元文書内のNodeId参照は、対応する新NodeIdへ書き換える。

元:
Link L1 -> Heading H1

複製:
Link L2 -> Heading H2

###### 5.5 文書間参照

v1の規範モデルでは、通常のNodeId参照を同じDocumentId内に限定する。

文書間参照は、将来の外部anchor仕様へ移管する。

##### 6. Revision
###### 6.1 直線的な履歴

一つの文書は、一本の単調増加するrevision列を持つ。

revision 0
    ↓ T1
revision 1
    ↓ T2
revision 2


Branchとmergeはv1では導入しない。

###### 6.2 Commitの直列化

複数actorは同じsnapshotからトランザクションを並行生成できる。

ただし、同一DocumentIdへのcommitは一つずつ直列化する。

Transaction生成:
並行可能

Transaction適用:
直列化

Transaction内部:
原子的

###### 6.3 Revision増加

意味上の変更を伴うトランザクションが成功した場合だけ、revisionを増加させる。

Applied:
revision + 1

Rejected:
変更なし

AlreadyApplied:
変更なし

AppliedNoChange:
revision変更なし


Revisionはwraparoundしてはならない。

###### 6.4 保存後のrevision

通常保存・再読込ではrevisionを維持する。

履歴がcompactされても、revision番号を巻き戻さない。

Revision番号だけで適用可能性を判断せず、操作ごとの適用前条件を再検証する。

##### 7. Snapshotの保持
###### 7.1 不変性

有効なsnapshot handleが存在する間、そのスナップショットの論理内容を参照できなければならない。

内部実装は次を自由に選べる。

- 永続データ構造
- Copy-on-write
- Checkpointと差分
- Operation logからの再構築

###### 7.2 過去版の永久取得

Revision番号だけを指定して、任意の過去snapshotを永久に取得できることは保証しない。

明示的に保持されたsnapshot handleだけが、その寿命中の利用を保証される。

###### 7.3 履歴の種類

次の履歴は別々の保持policyを持てる。

- Snapshot history
- Transaction history
- Undo history
- Tombstone metadata
- Audit log


具体的な保持上限は実装依存とする。

##### 8. 編集トランザクション
###### 8.1 概念構造
EditTransaction {
  transaction-id,
  document-id,
  base-revision,
  transaction-preconditions,
  ordered-operations,
  origin
}

###### 8.2 不変の第一級値

EditTransactionは不変の第一級値とする。

- 関数へ渡せる
- 関数から返せる
- Previewできる
- 保存・送信の対象にできる
- Commit前に検査できる


ただし、内部表現は公開せず、抽象型として提供する。

###### 8.3 原子性

トランザクションは全体が一括して成功または失敗する。

全操作成功:
すべて適用

一つでも失敗:
何も適用しない


部分成功を認めない。

###### 8.4 Operation順序

Operationは記載順に作業スナップショットへ仮適用する。

各操作後の一時状態が最終不変条件を満たす必要はない。

トランザクション末尾で文書全体の不変条件を検査する。

##### 9. 編集Operation

v1の基本Operationは次とする。

CreateNode
DeleteNode
SetProperty
InsertChild
RemoveChild
MoveNode

###### 9.1 CreateNode
CreateNode {
  node-id,
  kind,
  initial-properties
}


検査内容：

- NodeIdが未使用
- Node kindが有効
- 必須propertyが存在
- Property値がschemaへ適合


作成直後のdetached状態は、同一トランザクション内に限り認める。

###### 9.2 DeleteNode
DeleteNode {
  node-id,
  expected-parent,
  expected-position,
  expected-subtree-fingerprint?
}


対象ノードと所有subtree全体を削除する。

Rootは削除できない。

Subtree外から強い参照がある場合、同一トランザクション内で参照を解消しない限り拒否する。

###### 9.3 SetProperty
SetProperty {
  node-id,
  property-id,
  expected-old-value,
  replacement
}


意味：

none -> some:
property追加

some -> some:
property変更

some -> none:
property削除


必須propertyは削除できない。

期待旧値と現在値が異なれば競合とする。

###### 9.4 InsertChild
InsertChild {
  parent-id,
  child-id,
  position
}


子位置は整数indexではなくanchor方式を使う。

ChildPosition :=
  First
  | Last
  | Before(NodeId)
  | After(NodeId)


検査内容：

- 親と子が存在
- 子が未所有
- Anchorが有効
- 親schemaが子kindを許可
- Cycleを生成しない

###### 9.5 RemoveChild

親子関係を一時的に解除する低水準Operationとする。

通常の高水準APIではMoveNodeを優先する。

確定時にdetached nodeが残る場合、トランザクションを拒否する。

###### 9.6 MoveNode
MoveNode {
  node-id,
  expected-old-parent,
  expected-old-position,
  new-parent,
  new-position
}


Moveでは次を維持する。

- NodeId
- Node kind
- Property
- 所有subtree
- 非所有参照
- Provenance


禁止：

- Rootの移動
- 自身の子孫への移動
- 異なるDocumentIdへの直接移動
- Schemaに適合しない親への移動

##### 10. NodeIdの予約
###### 10.1 発行方式

正式なNodeIdは、文書編集contextに属するallocatorから、トランザクション構築前に予約する。

ID予約:
effectful

Transaction組立て:
pure

Commit:
effectful

###### 10.2 予約済みID

トランザクションが失敗または破棄された場合でも、予約済みNodeIdを再利用しない。

欠番は問題にしない。

###### 10.3 Copy

Copyは既存NodeIdを再利用せず、新しいNodeIdを予約して内容を複製する。

MoveとCopyは明確に分ける。

##### 11. 適用前条件
###### 11.1 Operation固有条件

対象固有の条件はOperationへ直接含める。

例：

SetProperty:
期待旧値

MoveNode:
期待旧親・旧位置

DeleteNode:
期待親・位置・任意fingerprint

###### 11.2 Transaction全体の条件

複数Operationに共通する大域条件だけを、transaction-level preconditionとして持つ。

例：

- 文書schema version
- 特定の大域状態
- 外部resource revision

###### 11.3 Base revision

base-revisionは、トランザクションがどのsnapshotから作られたかを示す。

Base revisionが現在revisionと同じであることを、適用の絶対条件にはしない。

##### 12. Stale transaction
###### 12.1 定義

現在revisionより古いrevisionを基に作成されたトランザクションを、旧版トランザクションまたはstale transactionと呼ぶ。

###### 12.2 分類
Fresh:
base = current

StaleApplicable:
base < current
かつ全ての適用前条件が成立

StaleConflicted:
base < current
かつ一つ以上の条件が不成立

InvalidFuture:
base > current

###### 12.3 保守的な再適用

Stale transactionを別の内容へ自動変換する高度なmergeは行わない。

同じOperationを現在状態へ再検証し、そのまま適用できる場合だけ適用する。

###### 12.4 無関係な変更

別ノードまたは独立propertyへの変更だけが行われている場合、旧版トランザクションを適用できる。

##### 13. 競合
###### 13.1 基本分類
EditConflict =
  DocumentMismatch
  | FutureRevision
  | TargetMissing
  | TargetAlreadyExists
  | NodeKindMismatch
  | PropertyValueMismatch
  | ParentMismatch
  | PositionMismatch
  | AnchorMissing
  | OwnershipConflict
  | CycleWouldBeCreated
  | NodeStillReferenced
  | SubtreeChanged
  | TransactionIdentityConflict

###### 13.2 競合と不正トランザクション

適用不成立を次の二種類に分ける。

Conflict:
作成時には妥当だったが、
現在状態の変化により適用不能

InvalidTransaction:
形式、型、schemaまたは基本制約に違反


概念的な拒否理由：

EditRejection =
  Conflicted(List<EditConflict>)
  | Invalid(EditValidationError)

###### 13.3 競合の収集

独立に検査可能な競合は、可能な限り一度に収集する。

先行Operationの失敗によって後続Operationが意味を失う場合、派生的な失敗は抑制する。

###### 13.4 自動併合

自動併合する範囲：

- 異なるNodeIdへの独立変更
- 同一ノードの独立propertyへの変更
- 同一propertyを同じ値へ変更する冗長操作


競合として拒否する範囲：

- 同一propertyへの異なる変更
- 削除済みノードへの操作
- 同一ノードの異なる親への移動
- 同一anchor・同一側への順序依存挿入
- 親子構造が両立しない変更


CRDTや高度な共同編集mergeはv1では導入しない。

##### 14. 適用手順

トランザクションは次の順序で処理する。

##### 1. TransactionIdを確認
##### 2. DocumentIdを確認
##### 3. Base revisionを比較
##### 4. Transaction-level preconditionを検査
##### 5. 現在snapshotから作業状態を作成
##### 6. Operationを順番に仮適用
##### 7. 文書全体の不変条件を検査
##### 8. 成功時だけcommit
##### 9. 新revisionとUndo情報を生成


一つでも拒否理由があれば、現在スナップショットを変更しない。

##### 15. 適用結果
###### 15.1 結果型
ApplyResult =
  Applied
  | AppliedNoChange
  | Rejected
  | AlreadyApplied

###### 15.2 Applied

次を含む。

- TransactionId
- 旧revision
- 新revision
- 新snapshot
- UndoToken

###### 15.3 AppliedNoChange

意味上の変更がなかったことを表す。

Revisionは増加しない。

###### 15.4 Rejected

次を含む。

- TransactionId
- Base revision
- Current revision
- EditRejection

###### 15.5 AlreadyApplied

同じTransactionIdと同じ内容のトランザクションが既に適用済みであることを表す。

編集を再適用しない。

###### 15.6 Transaction content hash

TransactionIdの再送判定では、意味に影響する内容からhashを作る。

- DocumentId
- Base revision
- 適用前条件
- Operation列
- Operation順序
- NodeId
- PropertyId
- 旧値・新値
- Anchor

##### 16. UndoとRedo
###### 16.1 新revision

Undoは過去revisionへ巻き戻す処理ではない。

逆トランザクションを現在状態へ適用し、新revisionを作る。

T1:
revision 10 -> 11

Undo T1:
revision 11 -> 12

###### 16.2 Undo情報

適用成功時に、逆操作に必要な情報を内部記録する。

CreateNode:
作成NodeId

DeleteNode:
削除subtree

SetProperty:
旧値

MoveNode:
旧親と旧位置

###### 16.3 UndoToken

標準公開APIでは、内部の逆トランザクションそのものではなく、opaqueなUndoTokenを返す。

UndoToken:
特定の適用結果をundoする権利・参照

###### 16.4 Undo競合

Undoも通常のトランザクションと同じ適用前条件を検査する。

他の変更を暗黙に消すような強制Undoは行わない。

###### 16.5 Redo

Redoも新しいTransactionIdを持つ新トランザクションとして実行する。

###### 16.6 履歴保持

Undo履歴は有限にできる。

- 最大件数
- 最大容量
- Checkpoint以前の破棄


具体値は実装依存とする。

Undo可能性は照会できなければならない。

##### 17. Provenance
###### 17.1 定義

Provenanceは、ノードがどこから生成されたかを示す由来情報である。

NodeId:
ノードの同一性

Provenance:
ノードの生成元・導出経路


Provenanceを変更してもNodeIdは変わらない。

###### 17.2 Optional metadata

Provenanceはoptionalとする。

欠落していてもノードは有効だが、生成元への移動や逆編集は利用できない。

###### 17.3 構造

Provenanceは共有可能なDAGとして管理できる。

NodeId
→ ProvenanceId
→ ProvenanceRecord
→ parent ProvenanceId


Provenance graphにcycleを認めない。

###### 17.4 種類
Provenance =
  UserCreated
  | SourceGenerated
  | MacroGenerated
  | Imported
  | Copied
  | Derived

###### 17.5 UserCreated

利用者操作によって直接作成されたノード。

- 作成TransactionId
- Actor等の任意補助情報

###### 17.6 SourceGenerated

RPX sourceの評価から生成されたノード。

記録候補：

- PackageInstanceId
- ModuleId
- DefinitionId
- SyntaxNodeId
- Source revision


絶対filesystem pathは規範的provenanceへ保存しない。

###### 17.7 MacroGenerated

次を区別して記録する。

- マクロ呼出し位置
- マクロ定義位置
- Template内構造path
- 入力由来構文かtemplate由来構文か

###### 17.8 Imported

別文書または外部データから取り込まれたノード。

Import後は新しいNodeIdを持ち、通常は独立編集可能とする。

###### 17.9 Copied

同一文書内のcopyまたは文書forkによって作られたノード。

元NodeIdとの関係を記録するが、identityは共有しない。

###### 17.10 Derived

Layout、filter、geometry処理等から派生したノード。

DerivedOrigin {
  producer-id,
  input-node-addresses,
  input-revisions,
  derivation-key?
}


複数入力を持てる。

##### 18. 派生ノードと逆編集
###### 18.1 編集可能性

ノードまたはノード層を次のように分類できる。

Editable:
直接編集可能

SourceMapped:
逆写像できる編集だけ可能

DerivedReadOnly:
参照可能だが直接編集不可

Ephemeral:
内部処理専用

###### 18.2 Provenanceと逆編集

Provenanceが存在するだけでは、逆編集可能とはみなさない。

生成結果への編集を上位モデルへ変換するには、明示的な逆編集mapperが必要である。

###### 18.3 逆編集結果
ReverseEditResult =
  Mapped
  | NotRepresentable
  | StaleOrigin
  | Ambiguous

###### 18.4 自動選択

複数の逆写像候補がある場合、v1では自動選択しない。

###### 18.5 逆写像不能

v1では次の二つだけを採用する。

逆編集可能:
上位modelへのEditTransactionへ変換

逆編集不能:
編集を拒否


OverrideとDetachは将来項目とする。

###### 18.6 Stale provenance

派生結果が古い入力revisionから生成されている場合、逆編集を拒否する。

再生成後に再試行する必要がある。

##### 19. 派生ノードのID継承
###### 19.1 DerivationKey

派生処理が一意かつ安定したDerivationKeyを提供できる場合、再生成後の対応ノードにNodeIdを継承できる。

DerivationKey {
  producer-id,
  source-node-id,
  local-role
}

###### 19.2 曖昧な対応

再生成前後の対応が曖昧な場合、新しいNodeIdを発行する。

誤った同一視より、ID変更を選ぶ。

###### 19.3 NodeIdとの違い

DerivationKeyはNodeIdそのものではなく、再生成時の対応候補を探すための補助keyである。

##### 20. Provenanceの安全性
###### 20.1 真正性

Provenanceは追跡・説明用metadataであり、デジタル署名や権限証明ではない。

外部入力のprovenanceは偽造されている可能性がある。

###### 20.2 Privacy

Provenance export時には次のpolicyを選択できる。

- Preserve
- Summarize
- Remove


ローカル絶対pathや秘密の内部情報を外部文書へ漏らしてはならない。

###### 20.3 書換え

一般利用者が任意のProvenanceを自由に作成・変更するAPIは提供しない。

Provenanceは編集エンジンおよび検証済みimporterが管理する。

##### 21. 公開API階層
###### 21.1 純粋な第一級値
DocumentId
NodeId
TransactionId
Revision
DocumentSnapshot
EditTransaction
EditConflict
EditRejection
ApplyResult
Provenance
ChildPosition

###### 21.2 状態付きhandle
DocumentHandle
UndoToken

###### 21.3 抽象型

次は内部表現を公開しない。

- document-snapshot
- document-handle
- edit-transaction
- document-id
- node-id
- transaction-id
- provenance
- undo-token

###### 21.4 Constructor付き公開data

利用者が分岐処理する必要があるため、次はconstructorを公開する。

- apply-result
- edit-rejection
- edit-conflict
- reverse-edit-result

##### 22. 高水準APIと低水準API
###### 22.1 高水準API

通常利用者には型付き編集関数を優先提供する。

例：

change-paragraph-text
move-section
replace-image-resource
insert-paragraph


これらはスナップショットから現在値を読み、適切な適用前条件を自動設定する。

###### 22.2 低水準API

Generic inspector、importer、plugin等のために低水準builderを提供できる。

ただし、EditTransactionの内部recordを直接構築させない。

- Builder APIを利用
- 作成時に検証
- Commit時にも完全再検証

###### 22.3 信頼境界

Network、plugin、serialization等から受け取ったトランザクションは信用せず、commit境界で完全に再検証する。

##### 23. Pure処理とEffectful処理
###### 23.1 Pure処理
- Snapshotの照会
- Nodeの照会
- EditTransactionの構築
- Transactionの事前検証
- Conflictの解析
- Provenanceの照会

###### 23.2 Effectful処理
- 現在snapshotの取得
- NodeIdの予約
- Transactionのcommit
- Undo／Redo
- Save／Load
- 履歴compaction

###### 23.3 Effectの暫定分類
document-read:
現在文書の観測

document-edit:
ID予約、Commit、Undo、Redo

storage:
永続化


最終的なeffect名とresource lifetimeはOPEN-MEM-001およびOPEN-ERR-001へ移管する。

##### 24. 競合・不正・実行障害の分離
###### 24.1 正常な結果
- Applied
- AppliedNoChange
- AlreadyApplied

###### 24.2 意味的拒否
ApplyResult.Rejected
├─ Conflicted
└─ Invalid


これは通常値として返す。

###### 24.3 実行障害

次はApplyResultへ混ぜない。

- Storage failure
- Permission failure
- Cancellation
- Resource exhaustion
- Internal fault


これらはeffect failureとしてOPEN-ERR-001で規定する。

###### 24.4 競合は例外ではない

旧版編集や同一propertyへの並行変更は通常運用で発生し得るため、例外やterminal failureとして扱わない。

##### 25. Undo履歴・Transaction履歴
###### 25.1 有限保持

次の履歴は有限保持を許可する。

- Undo情報
- 適用済みTransactionId
- Tombstone metadata
- 過去snapshot

###### 25.2 AlreadyApplied保証

Document transaction serviceがTransactionIdを既知として保持している範囲では、同じトランザクションを二重適用しない。

永久的なexactly-once保証はv1では要求しない。

###### 25.3 Undo不可

Undo情報が破棄済みの場合、明示的なUndoUnavailableを返す。

部分的なUndoを暗黙実行しない。

##### 26. 永続化
###### 26.1 標準保存

標準的な文書保存では、少なくとも次を保存する。

- DocumentId
- 現在revision
- Root NodeId
- NodeId
- Node内容
- 必要なschema version

###### 26.2 編集履歴

Transaction履歴、Undo履歴、TombstoneおよびAudit logは任意の別journalとして保存できる。

標準文書形式へ必須で埋め込まない。

###### 26.3 Version付きcodec

DocumentSnapshotやEditTransactionの内部表現を汎用serializationへ直接公開しない。

永続化にはversion付きの専用codecを使用する。

具体的なbinary・text形式は別項目へ移管する。

##### 27. 適合試験
EDT-01：ノード内容の変更
Revision 10:
Node P = Paragraph("Hello")

Transaction:
P.textを"Hello world"へ変更


期待結果：

- NodeId Pを維持
- Revision 11を生成
- 旧snapshotは不変

EDT-02：Move
PをSection AからSection Bへ移動


期待結果：

- PのNodeIdを維持
- Pのsubtreeを維持
- 旧親と新親のchildrenを更新

EDT-03：Copy
Pを複製


期待結果：

- 新しいNodeId Qを発行
- PとQは独立編集可能
- QのprovenanceにP由来を記録可能

EDT-04：Cycle
Node Aの子孫Bの下へAを移動


期待結果：

Rejected:
CycleWouldBeCreated

EDT-05：旧版だが独立
T1:
Revision 10を基にP.textを変更

先行T2:
Q.colorだけを変更


期待結果：

T1のpreconditionが成立するため、
Revision 11へ再検証して適用可能

EDT-06：Property競合
T1:
P.textを"A"から"B"へ変更

先行T2:
P.textを"A"から"C"へ変更


期待結果：

Rejected:
PropertyValueMismatch

EDT-07：原子性

複数Operationのうち一つが競合した場合：

- どのOperationも確定snapshotへ反映しない
- Revisionを増加させない

EDT-08：Transaction再送

同じTransactionIdと同じ内容を再送：

AlreadyApplied


同じTransactionIdと異なる内容：

Rejected:
TransactionIdentityConflict

EDT-09：参照中ノードの削除

別ノードから強く参照されるPを、参照を処理せず削除：

Rejected:
NodeStillReferenced

EDT-10：Undo競合
T1:
A -> B

T2:
B -> C

Undo T1:
B -> Aを試行


期待結果：

現在値はCなので、
PropertyValueMismatchとして拒否

EDT-11：文書複製

文書D1をD2へDuplicate：

- 新しいDocumentId
- 全NodeIdを新規発行
- 内部NodeId参照を再対応
- CloneMapを生成可能

EDT-12：Stale provenance

Revision 18から生成された派生ノードを、元文書がRevision 20の時点で逆編集：

StaleOrigin

EDT-13：逆写像不能

一つの生成元から多数のノードが生成され、特定ノードだけの編集を一意に元へ戻せない場合：

NotRepresentable
または
Ambiguous


自動的なsource変更を行わない。

##### 28. 移管先OPEN

###### `OPEN-ERR-001`

- Storage failure
- Permission failure
- Cancellation
- Terminal failure
- Cleanup
- Commit failureの伝播

###### `OPEN-MEM-001`

- DocumentHandleのlifetime
- SnapshotHandleの保持
- document-read／document-edit effect
- ID allocator
- 履歴dataの解放

###### `OPEN-CON-001`

- Commit queueの公平性
- 複数actor
- Network retry
- 永続的な重複排除
- Offline collaboration
- CRDT／OT

###### `OPEN-IR-001`

- Node kindとProperty schema
- Document modelとcanonical IRの境界
- 派生IR
- ProducerId
- DerivationKey

###### `OPEN-EDT-CODEC-001`

- 文書保存形式
- Transaction wire format
- Schema migration
- Provenance export policy

###### `OPEN-EDT-COLLAB-001`

- Branch
- Merge
- ActorId
- Offline transaction
- 同位置挿入の決定的順序
- 共同編集履歴

###### `OPEN-EDT-OVERRIDE-001`

- 派生ノードoverride
- Detach
- Override再適用
- 再生成時のoverride追跡

##### 29. 最終状態

```text
OPEN-EDT-001:
RESOLVED
```


本解決により、Reciplexa編集エンジンは次を提供する。

- 不変スナップショット
- 単一rootの所有tree
- 安定した文書・ノードidentity
- 直線的なrevision履歴
- 原子的な編集トランザクション
- 旧版編集の保守的な再適用
- 構造化された競合結果
- 二重適用の防止
- 新revisionとしてのUndo／Redo
- 生成元を追跡するprovenance
- 明示的な逆編集可能性
- 派生ノードへの安全な編集制限
- 第一級関数から扱える不変Transaction
- 状態変更と意味的競合と実行障害の分離

#### 13.12 `IR-001` Layered visual/motion/render IR

##### 概要・状態

単一万能IRを採用しないことは`確定`。以下の層名、schema、座標系、色・filter詳細は
最後の提案に対する明示承認がないため`暫定`とする。

```text
Domain IR
→ DocumentIR / SlideIR / SceneIR / MotionIR
→ VisualIR
→ RenderIR
→ target BackendIR
```

##### SurfaceとArtifact

```text
Artifact =
  StaticDocument { surfaces: Vector Surface }
| MotionDocument { surface, timeline, audio }

Surface = {
  id, kind, extent, view_box, background, root, metadata
}
```

各page/slide/artboardが個別size/metadataを持つ。

##### RenderIR node algebra

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

##### MotionIR

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

##### Backend lowering

backendはcapability setを宣言し、次の順でlowerする。

1. そのまま保持
2. Path等へ等価展開
3. 複数primitiveへ分解
4. subtreeを局所rasterize
5. warning付き近似
6. strict mode error

PPTX/AEの編集可能性を保つ場合は上位VisualIR/MotionIRから直接変換する。

##### 不変条件

- finite numberのみ。NaN/Inf禁止。
- definition graphはacyclic。
- resource IDは解決済み。
- Group child orderがz-order。
- RenderIRはI/O、未解決type、macro、Editable実行を含まない。
- provenance/semanticsはside table。
- 2D、左上origin、x右/y下、`parent × local` transform案は`暫定`。

##### メタ理論・反例

型Preservationだけではpixel等価を示せない。各loweringにobservational equivalenceまたは
許容誤差仕様が必要。

`CE-IR-001`: group opacityをchildへ分配するとblend結果が変わる。
`CE-IR-002`: glyphをpath化すると検索・accessibilityを失う。
`CE-IR-003`:単一RenderIRからPPTX/AEへ出すと編集意味を失う。

##### テスト

- `TEST-IR-001`: validator fuzz（NaN、cycle、missing resource）。
- `TEST-IR-002`: transform composition metamorphic。
- `TEST-IR-003`: group/blend/opacity golden。
- `TEST-IR-004`: text cluster/search round-trip。
- `TEST-IR-005`: color/profile conformance。
- `TEST-IR-006`: backend capability fallback。
- `TEST-IR-007`: animation sample determinism。
- `TEST-IR-008`: optimization前後のrender equivalence。
- `TEST-IR-009`: provenance side table保持。

#### 13.13 `ERR-001` 通常の失敗・Failure effect・後始末・Defect・最上位実行境界
##### DD-001 決定概要
###### DD-001.1 状態
Status:
RESOLVED

Scope:
optionとresult
型付きFailure effect
一般の再開可能Effectとの区別
Failure handler
resultとの明示的変換
Resource cleanup
bracket
継続破棄時の後始末
Primary／suppressed failure
Defect
Terminal failure
Fault boundary
Entry pointの最上位失敗処理
Diagnostic

###### DD-001.2 既存仕様との関係

本項目は、既に決定されているエフェクト・ハンドラの基礎仕様を変更しない。

既存仕様から、少なくとも次を前提とする。

- Effectはeffect rowによって型へ記録される
- Handlerはdeep handlerである
- 継続はone-shotである
- Operation探索規則が存在する
- Handlerには正常終了を扱うreturn clauseがある
- Handlerによって処理されたEffectはeffect rowから除かれる
- 一般Effectの継続は高々一回だけresumeできる


本項目で新たに定めるのは、既存のエフェクト機構上に構築する次の規則である。

- 非再開型のFailure effect
- resultとの使い分け
- 後始末保証
- DefectとTerminal failureの分類
- 実行境界での最終処理


現行の設計文書でも、deep handler、one-shot continuation、operation探索、return clause、effect rowなどは既決定の基礎部分として整理され、例外・cleanup・fault分類が未決定事項として残されていた。

##### 0. 設計原則
###### 0.1 失敗を一種類に統合しない

Reciplexaでは、すべての不成功を単一の「例外」として扱わない。

次の五種類を区別する。

1. option
   理由を必要としない通常の欠如

2. resultまたは専用data型
   理由を伴う通常の不成功

3. failure E
   現在の計算経路を中断する、型付きで回復可能な失敗

4. defect
   プログラムまたは処理系の論理的不変条件違反

5. terminal failure
   Runtime全体を安全に継続できない致命的障害

###### 0.2 判断基準
失敗を通常のデータとして扱う:
option／result／専用data型

処理境界まで非局所的に脱出する:
failure E

本来成立すべき内部前提が破られた:
defect

Runtimeの健全性を保証できない:
terminal failure

###### 0.3 公開APIと内部実装

resultとfailure Eの選択は、公開APIか内部実装かだけでは決めない。

次の意味上の違いによって判断する。

result:
失敗を値として返し、呼出し側が通常分岐として扱う

failure E:
現在の計算を中断し、外側の処理境界へ制御を移す


公開APIではresultを既定として推奨するが、処理全体の中断がAPIの本質である場合は、failure Eを公開してよい。

##### 1. option
###### 1.1 用途

optionは、理由を伴わない通常の欠如に使う。

例：

(type find-node
  (fn document-snapshot node-id
    (option document-node)))


適する例：

- 検索対象が存在しない
- 任意propertyが未設定
- Cacheに値がない
- 最初の一致がない

###### 1.2 不適切な用途

利用者が失敗理由を必要とする場合、optionではなくresultまたは専用data型を使用する。

##### 2. resultおよび専用結果型
###### 2.1 用途

resultは、失敗を通常値として保存・変換・分岐したい場合に使用する。

(type parse
  (fn str
    (result document parse-error)))


適する用途：

- Parsing
- Validation
- Dynamic cast
- Checked index access
- Checked arithmetic
- 複数errorの蓄積
- Retry候補の提示
- 利用者がその場で処理する不成功

###### 2.2 専用結果型

成功・失敗の二分だけでは表現不足の場合、専用data型を定義する。

編集トランザクションの結果は、その例である。

ApplyResult =
  Applied
  | AppliedNoChange
  | Rejected
  | AlreadyApplied


編集競合は通常運用で発生し得るため、failure EではなくApplyResult.Rejectedとして返す。

###### 2.3 複数errorの収集

最初の一件で中断せず複数の問題を集めたい場合、resultまたは専用validation型を使用する。

(type validate-document
  (fn document-snapshot
    (result validated-document
            (non-empty-list validation-error))))

##### 3. failure E
###### 3.1 定義

failure Eは、型Eの値を伴って現在の計算経路を中断する、組込みの型parameter付きEffectである。

failure E:
E型のerrorによって計算を中断できるEffect


例：

(data decode-error
  invalid-header
  unexpected-end
  (invalid-byte int))

(type decode-document
  (fn bytes document
    (effects
      (failure decode-error))))

###### 3.2 正常経路と失敗経路
正常終了:
宣言された戻り値を返す

Failure:
戻り値を返さず、外側のFailure handlerへ制御を移す

###### 3.3 Error payload

Eは通常のRPX型である。

特別な例外基底classや、全errorを統合する動的な例外objectは導入しない。

Error型は通常のdata等で定義する。

##### 4. Failureの発生
###### 4.1 raise

Failureを発生させる標準operationをraiseとする。

概念型：

(type raise
  (forall ((e type))
    (fn e never
      (effects
        (failure e)))))


使用例：

(raise
  (invalid-byte value))

###### 4.2 never

neverは値を一つも持たない空の型である。

raise:
正常経路では戻らない

戻り型:
never


neverは任意の型のsubtypeとして扱える。

never <: T


したがって、次の式全体はdocument型を持てる。

(if valid?
    document
    (raise invalid-header))


ただし、式全体のeffect rowにはfailure decode-errorが残る。

###### 4.3 基礎機構

raiseは独立した例外Runtimeを導入せず、既存のエフェクトoperation発生機構を使用する。

表面上は通常の適用に近いが、意味上は組込みの非再開型operationである。

##### 5. Failure handler
###### 5.1 非再開性

Failure handlerはerror値だけを受け取る。

(handle
  (decode-document input)

  (failure error ->
    fallback-document))


一般の再開可能Effectのhandler節とは異なり、Failure handlerには継続変数を渡さない。

一般Operation:
引数とone-shot continuationを受け取る

Failure:
errorだけを受け取り、continuationを公開しない

###### 5.2 Resume禁止

Failure発生地点からの再開は禁止する。

再開可能な通知や問い合わせが必要な場合は、failure Eではなく通常のEffect operationを定義する。

###### 5.3 内部実装

Runtime内部では既存のエフェクトハンドラ機構へloweringできるが、Failureの継続は利用者コードへ公開しない。

###### 5.4 Handlerの結果型

return節を省略する場合、正常終了値とFailure節の結果は共通の結果型へ適合しなければならない。

(handle
  (decode-document input)

  (failure error ->
    fallback-document))

正常結果:
document

Failure節:
document

handle式:
document


異なる型へ変換する場合は、既存のreturn節を明示する。

(handle
  (decode-document input)

  (return document ->
    (ok document))

  (failure error ->
    (err error)))

##### 6. Failureとeffect row
###### 6.1 型への明示

処理されていないFailureは、関数のeffect rowへ必ず現れなければならない。

(type load-document
  (fn package-resource document
    (effects
      resource
      (failure document-load-error))))


暗黙例外を認めない。

###### 6.2 Handlerによる除去

Failure handlerがfailure Eを処理した場合、対応するEffectを結果rowから除去する。

処理前:
{resource, failure decode-error}

処理:
failure decode-error

処理後:
{resource}

###### 6.3 Handler節自身のEffect

Handler節の評価中に発生したEffectは、handle式全体のeffect rowへ残る。

(handle
  computation

  (failure error ->
    (log-error error)))


log-errorがconsole Effectを持つ場合、handle式もconsole Effectを持つ。

###### 6.4 Handler節内の新しいFailure

Handler節内で新たに発生したFailureは、現在のhandlerではなく外側のhandlerへ伝播する。

handle対象内のfailure:
現在handlerが処理

handler節内の新failure:
外側handlerへ伝播


これにより、回復処理中の失敗が同じhandlerへ無限再入することを防ぐ。

##### 7. 一つのFailure型への統合
###### 7.1 基本指針

一つの処理領域では、原則として一つの公開Failure payload型へ統合する。

次のような型を乱用しない。

(effects
  (failure decode-error)
  (failure resource-error)
  (failure permission-error))


代わりに上位error型を定義する。

(data document-load-error
  (decode decode-error)
  (resource resource-error)
  (permission permission-error))

(type load-document
  (fn package-resource document
    (effects
      (failure document-load-error))))

###### 7.2 Error変換

下位のFailureは、境界で上位error型へ明示的に変換する。

(handle
  (decode-image image-bytes)

  (failure error ->
    (raise
      (image-failure error))))

処理前:
failure image-error

処理後:
failure document-load-error

###### 7.3 位置付け

この指針は、一般EffectRowの多重labelやnamed instanceの能力を禁止するものではない。

Failure APIの可読性・型合成・ハンドラ設計を単純にするための規範的指針である。

##### 8. resultとFailureの変換
###### 8.1 暗黙変換の禁止

次を自動変換しない。

resultのerr
→ failure

failure
→ resultのerr


制御フローを変更する変換は、コード上で明示する。

###### 8.2 resultからFailure

標準補助操作として、概念的なor-raiseを提供する。

(type or-raise
  (forall ((a type)
           (e type))
    (fn (result a e) a
      (effects
        (failure e)))))


意味：

ok value:
valueを返す

err error:
raise error

###### 8.3 Failureからresult

既存のhandlerとreturn clauseを使用する。

(handle
  computation

  (return value ->
    (ok value))

  (failure error ->
    (err error)))


高階APIとして提供する場合は、CBVによる事前評価を防ぐため無引数関数を受け取る。

概念型：

(type as-result
  (forall ((a type)
           (e type))
    (fn
      (fn unit a
        (effects
          (failure e)))
      (result a e))))

##### 9. resultとFailureの選択指針
###### 9.1 resultを推奨する場合
- 呼出し側がその場で分岐する
- 失敗を保存・変換する
- 複数errorを集積する
- 検索またはvalidation
- Dynamic cast
- Checked arithmetic
- 編集競合
- 利用者が修正して再試行する通常結果

###### 9.2 Failureを認める場合
- 深い呼出し階層から脱出する
- 処理領域全体を中断する
- 中間関数がerrorを転送するだけになる
- 外側handlerで一括したpolicyを適用する
- Loader、decoder、job等の処理単位を失敗させる

###### 9.3 公開API

公開APIでもFailureを使用できる。

ただし、Failure型はeffect rowへ明示し、利用者がhandlerを設置できるようにする。

すべてのAPIについてresult版とFailure版の両方を自動的に提供する必要はない。

正準APIを一つ決め、実需がある場合のみ変換用wrapperを追加する。

##### 10. Resource cleanup
###### 10.1 基本primitive

後始末の基礎primitiveとしてbracketを採用する。

公開形は高階関数に見えるAPIとし、内部ではRuntimeまたは標準handlerのprimitiveとして実装する。

概念例：

(bracket
  (fn ()
    (open-resource path))

  (fn (handle)
    (process-resource handle))

  (fn (handle)
    (close-resource handle)))

###### 10.2 役割
acquire:
Resourceを取得する

use:
Resourceを利用する

release:
Resourceを解放する

###### 10.3 特別な保証

通常の高階関数とは異なり、次の経路でreleaseを保証する。

- useの正常終了
- failureによる中断
- 継続の破棄
- 構造化されたcancellation
- Unwind可能なdefect

##### 11. Acquire規則
###### 11.1 Release登録

releaseはacquireが正常値を返した直後に登録する。

###### 11.2 Acquire failure

acquireが正常値を返す前にFailureを発生させた場合、対応するreleaseは呼ばない。

acquire開始
↓
failure
↓
resource未取得
↓
releaseしない

###### 11.3 部分取得

Acquire内部で複数段階のresource取得が必要な場合、Acquire自身が内側のbracketを使用する。

外側bracketは部分取得状態を推測しない。

##### 12. Useの正常終了

useが正常終了した場合、次の順序を保証する。

1. useが結果を生成
2. releaseを実行
3. releaseが成功
4. useの結果を外側へ返す


bracketが正常に戻った時点で、resourceの解放は完了している。

##### 13. Use中のFailure

useがFailureを発生させた場合、次を行う。

1. 元のFailureを一時保持
2. releaseを実行
3. release成功後、元のFailureを再伝播


元のFailure発生地点から処理を再開しない。

##### 14. 一般Effectと継続
###### 14.1 一時中断

useの途中で再開可能な一般Effectが発生しても、その時点ではreleaseしない。

一般Effectをperform
↓
Handlerへ制御移動
↓
継続はSuspended
↓
Resourceは保持

###### 14.2 Resume

Handlerが継続をresumeした場合、bracket scopeとresource lifetimeを維持したまま計算を続ける。

###### 14.3 Discard

Handlerが継続を再開せず破棄した場合、中断された計算のbracket scopeを終了し、内側から外側へreleaseする。

###### 14.4 継続状態

概念的な状態：

Suspended
Resumed
Discarded
Consumed


許可される遷移：

Suspended -> Resumed -> Consumed
Suspended -> Discarded -> Consumed


Consumed後の再resumeまたは再discardは禁止する。

###### 14.5 継続escape

v1では、one-shot continuationをhandler節の外へescapeさせない。

禁止：

- Recordへの保存
- Heap cellへの格納
- Closureへ捕捉して返す
- Handler節外での遅延resume


Handler節を抜けるまでに、継続はresumeまたはdiscardされなければならない。

これによりcleanup時点を構造的に決定する。

既存仕様により、より厳しい継続所有規則が存在する場合は、その規則を優先する。

##### 15. Cleanup順序と回数
###### 15.1 LIFO

Nested bracketは、resource取得順の逆順にreleaseする。

取得:
A
B
C

解放:
C
B
A

###### 15.2 高々一回

各release actionは高々一回だけ実行する。

概念状態：

Registered
Running
Completed

Registered -> Running -> Completed


Completed後に再実行しない。

###### 15.3 一つのRelease失敗

内側のreleaseが失敗しても、残る外側のreleaseを続行する。

release Cが失敗
→ release Bを試行
→ release Aを試行

##### 16. Cleanup中のFailure
###### 16.1 Primary failure

計算を最初に中断させたFailureをprimary failureとする。

###### 16.2 Suppressed failure

Primary failureの後、後始末中に追加で発生したFailureをsuppressed failureとして記録する。

例：

Use:
decode-error

Release:
close-error


結果：

Primary:
decode-error

Suppressed:
close-error

###### 16.3 正常終了後のRelease failure

Useが正常終了し、Releaseだけが失敗した場合：

Release failureがprimary failureになる
Useの正常結果は返さない

###### 16.4 複数のSuppressed failure

Nested cleanupで複数のreleaseが失敗した場合、実際のcleanup実行順で記録する。

Primary:
E0

Suppressed:
E1
E2

###### 16.5 通常Handlerへの公開

v1の通常Failure handlerには、primary error Eだけを渡す。

Suppressed failureは構造化されたRuntime診断metadataとして保持し、次から観測可能にする。

- 最上位診断
- Log
- Debugger
- Defect／failure report


通常の回復ロジックから異種のcleanup failureを精密に処理したいAPIは、releaseを明示的なresultとして扱う別APIを提供できる。

##### 17. finally

一般的な後始末用にfinally相当の補助APIを提供できる。

ただし、独立したCore primitiveにはせず、値を取得しないbracketとして構築する。

bracket:
取得したresourceをreleaseへ渡す

finally:
取得値のない後始末


Coreの後始末primitiveはbracket系へ集約する。

##### 18. Cancellation
###### 18.1 分類

Cancellationは通常Failure、Defect、Terminal failureのいずれとも同一視しない。

Cancellation:
外部要求による構造化された計算中断

###### 18.2 Cleanup

Cancellationではbracket cleanupを実行する。

###### 18.3 詳細

Cancellationを一般Effect、専用signal、またはConcurrency Runtimeの操作として表すかはOPEN-CON-001へ移管する。

本項目では次だけを確定する。

- CancellationはDefectではない
- CancellationはTerminal failureではない
- Cancellationは構造化unwindを実行する
- Releaseを保証する

##### 19. Defect
###### 19.1 定義

Defectは、本来成立すべきプログラム上または処理系上の論理的不変条件が破られた状態である。

例：

- Internal assertion違反
- 網羅的matchの実行時不一致
- 証明済みcastの失敗
- 証明済みindex条件の違反
- One-shot continuationの二重resume
- Validator自身の契約違反
- 型検査済みIRの内部矛盾
- 所有treeの不変条件破壊

###### 19.2 Effect row

Defectは通常のeffect rowへ現れない。

Defectを通常のFailure代わりに使用してはならない。

###### 19.3 再開

Defect発生地点からのresumeは禁止する。

###### 19.4 通常Handler

通常のhandleではDefectを捕捉できない。

###### 19.5 Cleanup

Runtimeとcleanup stackが健全であることを確認できるDefectでは、構造化unwindとbracket cleanupを行う。

健全性を確認できない場合はTerminal failureへ昇格する。

##### 20. Fault boundary
###### 20.1 定義

Fault boundaryは、Defectの影響を一つの計算単位へ隔離する、Runtimeまたはtrusted hostが設置する境界である。

適用例：

- Entry point
- GUI command
- Render job
- Server request
- Worker
- Plugin invocation
- Test case

###### 20.2 一般公開

通常の利用者コードが任意位置へFault boundaryを設置し、Defectを通常errorとして扱う一般APIは提供しない。

###### 20.3 処理

Fault boundaryはDefect発生時に次を行う。

1. 中断された継続を破棄
2. Unwind可能ならcleanupを実行
3. 未commitの作業状態を破棄
4. DefectReportを生成
5. 該当jobをDefectedとして終了
6. 健全な外側Runtimeへ制御を戻す

###### 20.4 継続条件

外側処理を継続できるのは、少なくとも次を保証できる場合だけである。

- Runtime内部構造が健全
- Cleanup stackが健全
- 共有stateへの部分commitがない
- Memory safetyが破られていない
- Defect発生jobのresourceを隔離できる


保証できない場合はTerminal failureとする。

##### 21. Terminal failure
###### 21.1 定義

Terminal failureは、Runtimeまたはprocessを安全に継続できない障害である。

例：

- Runtime memory構造の破損
- Stack／continuation表現の破損
- Cleanup stackの破損
- Kernelの致命的不変条件破壊
- 安全な回復不能のOut-of-memory
- Processの強制終了
- 電源断

###### 21.2 通常Handler

通常HandlerおよびFault boundaryから回復できない。

###### 21.3 Cleanup

完全なcleanupを保証しない。

###### 21.4 可能な最小処理

実装が安全に実行できる場合のみ、次を行う。

- 事前確保済み領域による最小診断
- Host／OSへの終了通知
- Watchdogへの報告
- ProcessまたはRuntime instanceの停止


通常の利用者定義formatting、文書保存、任意cleanup、通常JobResultの返却は保証しない。

##### 22. 個別事例の分類
###### 22.1 Assertion
利用者入力の検査:
result／failure

内部不変条件の検査:
違反時はDefect


Assertionを公開入力validationの代替として使用しない。

###### 22.2 Match
静的に網羅的と証明されたmatchの不一致:
Defect

意図的な部分match:
default節または明示的なresult／failureを要求

###### 22.3 Dynamic cast
通常のcast不一致:
option／result

成功すると静的に保証された内部castの失敗:
Defect

###### 22.4 Index access
一般のchecked access:
option／result

内部で範囲内と保証されたaccessの違反:
Defect

###### 22.5 Arithmetic overflow
数学的int:
Overflowなし

固定幅checked演算:
result

固定幅wrap演算:
型の通常意味としてwrap

証明済み範囲条件の違反:
Defect


Build modeによってoverflow意味論を変えない。

###### 22.6 Division by zero
一般除算:
resultまたは明示Failure

non-zero型を受け取る除算:
正常値

non-zero保証の破壊:
Defect

###### 22.7 Continuationの二重resume
Defect


Runtime内部構造の健全性を失った場合はTerminal failureへ昇格する。

###### 22.8 Validator
外部入力が検証不合格:
result／failure

Validator自身の契約違反:
Defect

###### 22.9 Foreign adapter
安全に検出できる契約違反:
Defectとしてadapter jobを停止

Memory safetyまたはRuntime整合性の破壊:
Terminal failure

###### 22.10 Resource exhaustion
局所的quota不足:
failure resource-error

File descriptorやGPU buffer不足:
failure resource-error

一般heapの回復不能OOM:
Terminal failureを許容

##### 23. DefectReport
###### 23.1 内容

Fault boundaryは構造化されたDefectReportを生成する。

概念的な内容：

DefectReport {
  kind,
  safe-message,
  source-origin,
  stack-trace,
  effect-handler-trace,
  transaction-id?,
  document-id?,
  suppressed-cleanup-failures,
  runtime-version
}

###### 23.2 安全性

Defect report生成は、利用者定義の一般showや複雑なformattingへ依存しない。

通常は次の最小情報を使用する。

- Defect kind
- 固定または検証済みmessage
- 正規化されたsource identity
- 制限されたstack情報


値の詳細dumpはdebug policyで明示的に有効化する。

###### 23.3 権限・機密性

Defect reportには機密情報が含まれ得るため、出力policyを実行環境が管理する。

##### 24. ジョブ結果

Fault boundaryを持つ実行単位では、結果を次のように分類できる。

JobResult<A, E> =
  Completed(A)
  | Failed(E)
  | Cancelled
  | Defected(DefectReport)


意味：

Completed:
正常完了

Failed:
型付きの予想可能な失敗

Cancelled:
外部要求による中止

Defected:
プログラム上の欠陥


Terminal failureでは、通常JobResultを返すところまで到達しない。

##### 25. Entry pointと実行環境
###### 25.1 Runtime capability

Entry pointのrequired effect rowは、実行環境が提供するEffectの部分集合でなければならない。

RequiredEffects(main)
⊆
ProvidedEffects(entry-environment)


例：

CLI environment:
console
resource
storage
clock
failure sink

GUI environment:
window
input
render
resource
storage
failure sink

Server environment:
network
storage
clock
request failure sink


未提供Effectが残る場合は静的エラーとする。

entry point requires an unsupported effect:
  window

###### 25.2 実行環境ごとのmain

すべてのEntry pointへ一種類のmain型を強制しない。

CLI entry:
exit statusを返し得る

GUI entry:
application lifecycleを開始する

Server entry:
serverまたはserviceを開始する

Worker entry:
job loopを開始する


最上位Failure処理は、各Entry environmentの契約に従う。

##### 26. 未処理Failure
###### 26.1 原則

Application固有のFailureは、Application境界で処理することを推奨する。

- 利用者向けdiagnostic
- Retry policy
- Exit status
- GUI dialog
- HTTP response

###### 26.2 最終防御

Entry environmentが対応するFailure sinkを提供する場合、mainのeffect rowに未処理failure Eを残せる。

Runtimeは最終防御として次を行う。

- 安全な最小診断
- Entryまたはjobを失敗扱いにする
- Cleanup完了を待つ
- Entry environmentに対応する終了結果へ変換

###### 26.3 Runtime default表示

任意のError型へ複雑な自動文字列化を要求しない。

Runtime default handlerは、可能な範囲で次だけを使用する。

- Error型の正式identity
- 安全なconstructor名
- Source位置
- Effect trace


詳細な利用者向け説明は、Error型ごとの明示的な変換関数が提供する。

(type explain-document-error
  (fn document-error diagnostic))

##### 27. 実行環境別の処理
###### 27.1 CLI
正常終了:
0相当

Application failure:
Applicationが明示した非ゼロstatus

未処理Failure:
Runtime標準の非ゼロstatus

Defect:
Failureとは区別された異常終了status

Cancellation:
Host規約または専用status

Terminal failure:
Runtime／OS依存の異常終了


具体的な整数値はtoolchainまたはhost規約へ移管する。

###### 27.2 GUI
- Command単位でFailureを通常結果へ変換
- Render job単位でFault boundaryを設置
- Plugin invocation単位でFault boundaryを設置
- 一操作の失敗でevent loop全体を終了しない


Runtime最終handlerは、処理漏れによるApplication全体の破損を防ぐ最後の境界とする。

###### 27.3 Server
- Request単位でFailureをresponseへ変換
- Request単位でDefectを隔離
- 他requestは健全なら継続
- Terminal failureではprocessを停止

###### 27.4 Plugin
- Plugin invocation単位でFault boundary
- 未commit transactionを破棄
- Plugin所有resourceをcleanup
- DefectReportを生成
- Hostが健全なら継続

###### 27.5 Render job

不変スナップショットを入力とし、出力をcommit前の一時成果物として扱う。

Render defect:
成果物を破棄
DefectReportを返す
Editor本体は健全なら継続

##### 28. Diagnostic
###### 28.1 構築と出力の分離
Diagnostic construction:
構造化されたdiagnostic値を生成

Diagnostic sink:
表示・保存・送信先を決定


Sinkの例：

- Terminal stderr
- GUI notification
- Log file
- Test reporter
- HTTP response
- Telemetry

###### 28.2 PrimaryとSuppressed

Primary failureを最初に表示し、suppressed cleanup failureを追加情報として表示する。

例：

Document load failed:
  invalid document header

Additional failure during cleanup:
  failed to close resource

###### 28.3 Libraryの責務

Libraryは最終的な出力先や終了方法を決めない。

Errorの意味と、必要に応じてDiagnosticへの変換関数を提供する。

##### 29. 適合試験
ERR-01：通常のresult

Dynamic castが不一致の場合：

result／optionとして返す
Defectにしない

ERR-02：Failureの型
(type decode
  (fn bytes document
    (effects
      (failure decode-error))))


Handlerなしで呼び出す関数は、同じFailureをeffect rowへ含めなければならない。

ERR-03：Failure handler
(handle
  (decode bytes)

  (failure error ->
    fallback))


期待結果：

- Failure節へ継続を渡さない
- failure decode-errorを結果rowから除去

ERR-04：Failureからresult
(handle
  computation

  (return value ->
    (ok value))

  (failure error ->
    (err error)))


期待結果：

result value-type error-type

ERR-05：Acquire failure

acquireが失敗した場合：

releaseを実行しない

ERR-06：Use failure

useがFailureを発生した場合：

- releaseを一回実行
- release成功後に元Failureを再伝播

ERR-07：Effectの一時中断

一般Effect発生後、継続がresumeされた場合：

resourceをreleaseせず、
bracket scopeを維持して続行

ERR-08：継続破棄

一般Effectの継続がdiscardされた場合：

継続内のbracketを内側から外側へrelease

ERR-09：Nested cleanup

取得順：

A
B
C


解放順：

C
B
A

ERR-10：Cleanup failure
Use:
E0

Release B:
E1

Release A:
E2


期待結果：

Primary:
E0

Suppressed:
E1, E2

Release Aまで実行

ERR-11：正常終了後のRelease failure
Use:
正常値R

Release:
E


期待結果：

EをPrimary failureとして伝播
Rは返さない

ERR-12：通常cast不一致
result／option

ERR-13：証明済みcastの失敗
Defect

ERR-14：One-shot継続の二重resume
Defect


Runtime健全性を保証できない場合：

Terminal failureへ昇格

ERR-15：Fault boundary

Plugin内でUnwind可能なDefectが発生：

- Plugin計算を中止
- Cleanupを実行
- 未commit状態を破棄
- DefectReportを返す
- Hostは健全なら継続

ERR-16：Terminal failure

Runtime memory構造が破損：

- 通常handlerで回復しない
- Cleanupを保証しない
- 最小診断後にRuntimeを停止

ERR-17：Entry effect検査

CLI Entry pointがwindow Effectを要求し、CLI環境が提供しない場合：

static error:
entry point requires an unsupported effect

ERR-18：編集競合

TransactionがPropertyValueMismatchで拒否された場合：

ApplyResult.Rejectedとして返す
failure effectにしない

ERR-19：Storage障害

Transaction commit中に保存装置へ接続できない場合：

ApplyResult.Rejectedではなく、
failure document-service-errorとして扱う

##### 30. 移管先OPEN

###### `OPEN-MEM-001`

- bracketのRuntime表現
- Cleanup stack
- SnapshotHandleのlifetime
- Resource handleの所有権
- Continuationとresource lifetime
- Releaseのcleanup-safe Effect

###### `OPEN-CON-001`

- Cancellationの表現
- Structured concurrency
- Task境界
- Commit queue
- Continuationの非同期利用
- Fault boundaryとworker

###### `OPEN-KER-001`

- Trusted adapterの契約
- Validator boundary
- ForeignValue
- Adapter defectとterminal failureの境界
- Kernel crash containment

###### `OPEN-TST-001`

- Test caseごとのFault boundary
- Failureの期待
- Defectの期待
- Cleanup検査
- Entry environmentのtest double

###### `OPEN-PKG-ENTRY-001`

- CLI／GUI／Server／Worker entry profile
- mainの正確な型
- Exit status
- Entry environmentのProvidedEffects

###### `OPEN-ERR-DIAG-001`

- Diagnostic schema
- Source trace
- Effect trace
- Primary／suppressed表示
- Privacy policy

##### 31. 最終状態

```text
OPEN-ERR-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- option／result／Failure／Defect／Terminal failureの明確な分類
- Effect rowへ現れる型付きFailure
- 再開不能なFailure handler
- resultとFailureの明示的変換
- 暗黙例外を持たない失敗型
- bracketによるResource cleanup
- 継続破棄時の後始末
- Nested cleanupのLIFO保証
- Primary／suppressed failure
- 通常Handlerから分離されたDefect
- Runtime管理のFault boundary
- Terminal failureの明確な限界
- Entry environmentによるEffect capability検査
- CLI／GUI／Server／Plugin／Render jobごとの障害隔離
- Diagnostic構築と出力先の分離

#### 13.14 `MEM-001` Perceusメモリ管理・スコープ付きリソース・継続・メモリ予算
##### DD-001 決定概要
###### DD-001.1 状態
Status:
RESOLVED

Scope:
通常値の自動メモリ管理
Perceus方式の精密参照カウント
dup／drop／reuse
Closure環境
One-shot continuation
Failure unwind
Resource handle
bracketとの責務分離
メモリ予算
Allocation failure
Reference count overflow
GC時点の観測可能性

###### DD-001.2 既存仕様との関係

本項目は、既に決定されている次の仕様を前提とする。

- 通常値は原則として不変
- valは不変binding
- varはescape不能な局所可変状態
- varの物理的な配置方法は観測不能
- 一般的なescape可能cell／refは未導入
- Effect handlerはdeep handler
- Continuationはone-shot
- Failureは再開不能
- bracketは正常終了・Failure・Cancellation・継続破棄時にcleanupする
- 文書Snapshotは不変値
- 文書の所有構造はtree


旧版の設計記録では、varのescape禁止は決定済みである一方、GC方式、closure環境、continuation、resource lifetime、escape可能なcell／ref、循環値、GUI再評価時の状態寿命などがOPEN-MEM-001へ残されていた。

###### DD-001.3 中心的な決定
- 通常値の自動メモリ管理にはPerceus方式を採用する
- Perceusは精密参照カウントとreuse解析を行う
- Effect／handler／bracketを明示的制御フローへloweringした後に適用する
- 利用者へdup、drop、reuse、参照カウントを公開しない
- 一般heap上の強参照cycleはv1では構築不能とする
- 一般的なescape可能cell／refはv1では導入しない
- varはescape不能な局所状態のままとする
- 外部resourceの意味的解放はPerceusではなくbracketが担当する
- Scoped resource handleはbracket外へescapeできない
- 一般的なborrow system、weak reference、利用者定義finalizerはv1では導入しない
- 通常allocationはeffect rowへ現さない
- 明示的な予算超過はfailure resource-exhaustedとする
- 回復不能な一般heap OOMはterminal failureを許容する

##### 0. 用語
###### 0.1 Perceus

Perceusは、関数型Coreに対して精密な参照カウント命令を挿入し、参照の一意性を利用してメモリ領域の再利用を行うコンパイル方式である。

代表的な内部操作は次である。

dup:
参照を共有する

drop:
不要になった所有参照を解放する

reuse:
一意な不要領域を新しい値の構築へ再利用する


Perceusは、循環のないプログラムにおいて不要参照を保持しないことを目標とし、参照が一意なら不変データの領域を再利用して、純粋な関数型コードを内部的にin-place実行できる。

###### 0.2 自動メモリ管理

本仕様における自動メモリ管理は、Tracing GCではなく、Perceus方式の精密参照カウントを中心とする。

自動メモリ管理:
Perceus方式

Tracing GC:
標準方式として採用しない

明示的free:
利用者へ提供しない

###### 0.3 Resource

Resourceは、単なるheap上の値ではなく、意味的な取得・利用・解放時点を持つ外部対象である。

例：

- File handle
- Socket
- GPU buffer
- Window
- Database transaction
- Foreign library object
- OS process
- Device context

##### 1. メモリとResourceの分離
###### 1.1 通常値

次はPerceusによる自動メモリ管理の対象である。

- data値
- record
- list
- str
- bytes
- Closure環境
- Handler環境
- Continuationの内部表現
- DocumentSnapshot
- Provenance DAG
- Resource handleのRPX側wrapper

###### 1.2 外部Resource

外部Resource本体の解放時点は、参照カウントが0になった時点ではなく、bracketのscope終了によって決める。

Perceus:
RPX値のmemoryを管理する

bracket:
外部Resourceの意味的lifetimeを管理する

###### 1.3 基本原則
Memory reclamation:
Perceus

Semantic resource cleanup:
bracket


File handleのwrapperが最後にdropされたことを、File closeの正規の意味的契機とはしない。

##### 2. Perceusによる自動メモリ管理
###### 2.1 利用者から見える意味

通常のRPX値について、次を保証する。

- 利用者は明示的にfreeしない
- 到達不能または所有権が消費された値は自動的に解放可能
- 参照カウント値を観測できない
- 物理addressを観測できない
- 共有の有無を観測できない
- reuseの成立・不成立を観測できない
- 物理的なin-place更新によって意味が変わらない

###### 2.2 回収時点

Perceusは正確な最終使用位置でdropを挿入することを目標とする。

ただし、利用者が特定値の物理的な解放時点へ依存するAPIは提供しない。

意味上:
不要参照を保持しない

利用者API:
解放時点への直接依存を許さない

###### 2.3 物理identity

通常の不変値にpointer identityを公開しない。

同じ内容のrecord:
同じallocationかどうかを観測不能

同じlist:
構造共有されているか観測不能

同じstr:
internされているか観測不能


継続identityが必要な領域では、専用のopaque IDを使う。

- DocumentId
- NodeId
- TransactionId
- Resource固有のidentity

##### 3. Compilation pipeline
###### 3.1 適用順序

Perceusは、Effectと制御移動を明示化した後のCore IRへ適用する。

1. Source解析
2. Macro展開
3. 名前解決
4. 型・Effect検査
5. 高水準構文のdesugar
6. Effect handler／Failure／bracketのlowering
7. Closure conversion
8. 明示的制御フローCoreの生成
9. 最終使用・共有解析
10. dup／drop挿入
11. Ownership verifier
12. Reuse解析
13. Reuse特殊化
14. Reuse verifier
15. Backend lowering


Kokaでも、Effect等を明示的な制御フローを持つ内部Coreへ変換した後にPerceusを適用する方式が用いられている。

###### 3.2 Surface所有権注釈

通常のRPXソースへ次の所有権注釈を要求しない。

owned
borrowed
shared
consume
dup
drop


通常利用者は、不変値と第一級関数を通常どおり使用する。

所有権はコンパイラが次から推論する。

- 変数の使用回数
- 最終使用位置
- 制御フロー
- Closure capture
- Continuation capture
- Effect lowering結果
- Foreign interface metadata

###### 3.3 Trusted boundary

所有権情報を明示的に記述できるのは、主に次のtrusted metadataである。

- Kernel primitive
- Foreign adapter
- Resource API
- Backend ABI

##### 4. 所有権Core IR
###### 4.1 必須のCore要素

Perceus適用前のCoreは、少なくとも次を明示する。

Let
Call
TailCall
Construct
Deconstruct
Match
Closure
Branch
Join
Return
Continuation capture
Resume
Discard
Handler frame
RegisterCleanup
RunCleanup
Raise
ForeignCall


Perceus適用後は、さらに次を含む。

Dup
Drop
ReuseCandidate
ConstructReuse

###### 4.2 評価順序

既存仕様のCBVかつ左から右の評価順序を維持する。

Perceusによるdropやreuseの挿入は、利用者から観測できる評価順序およびEffect順序を変更してはならない。

##### 5. dup
###### 5.1 意味

同じ値を複数の生存経路から使用する必要がある場合、コンパイラはdupを挿入する。

dup x


参照カウントを増加させ、複数の所有参照を成立させる。

###### 5.2 挿入条件

構文上の複数利用ごとに機械的に挿入するのではなく、所有権・borrow・最終使用解析に基づいて必要な場合だけ挿入する。

###### 5.3 利用者からの不可視性

dupの有無は利用者から観測できない。

参照カウント増加を明示的なEffectとして扱わない。

##### 6. drop
###### 6.1 意味

値が以後どの制御経路からも使用されない地点で、コンパイラはdropを挿入する。

drop x


参照カウントが0になった場合、次を行う。

1. 値が保持するfieldをdrop
2. Closureなら捕捉環境をdrop
3. Constructor領域を解放またはreuse候補化

###### 6.2 最終使用位置

dropは字句scope末尾ではなく、正確な最終使用位置へ挿入できる。

(local
  (val large-value
    (build-large-value))

  (process large-value)

  (unrelated-work))


概念的には次となる。

build large-value
process large-value
drop large-value
unrelated-work

###### 6.3 制御フロー

各制御経路で、所有参照は次のいずれかを満たさなければならない。

- 別の処理へ移送される
- 最終的にdropされる

##### 7. 分岐とJoin point
###### 7.1 排他的分岐

排他的分岐では、実際に通る経路が一つであるため、各branchへ所有権を移送できる。

condition
├─ true  -> xを左branchへ移送
└─ false -> xを右branchへ移送


分岐前に常にdupする必要はない。

###### 7.2 Join時の整合

Join pointへ到達する全経路は、各値について整合する所有状態を提供しなければならない。

適合：

Branch A:
owned x

Branch B:
owned x

Join:
owned x


適合：

Branch A:
xをdrop

Branch B:
xをdrop

Join:
xなし


所有状態が一致しない場合、コンパイラが必要なdup・dropを挿入するか、内部IR不正として拒否する。

##### 8. Reuse
###### 8.1 位置付け

reuseは、参照カウントの正しさに必要な機構ではなく、意味保存最適化である。

dup／drop:
正しさに必要

reuse:
性能最適化

###### 8.2 一意性

不要になったconstructorが一意参照されている場合、その領域を同等のsize classを持つ新constructorへ再利用できる。

共有されている場合は通常allocationへfallbackする。

Perceusのreuse解析は、不変データの一意性を利用し、純粋な関数型プログラムを内部的にin-place実行できるようにする。

###### 8.3 Reuse不成立

Reuse候補が共有されていた場合：

- Failureにしない
- Defectにしない
- 通常の新規allocationへfallbackする

###### 8.4 非保証

次を言語仕様として保証しない。

- 特定の値が必ずin-place更新される
- 特定のallocationが必ず省略される
- Compiler versionを越えて同じreuse判断になる
- Sourceの小変更後も同じreuse判断になる

###### 8.5 Reuse禁止値

次は原則としてreuse対象外とする。

- Foreign resource wrapper
- Pinned memory
- 外部APIへaddressを渡した値
- Foreign-owned値
- Runtime metadata
- Continuation stack segment
- Cleanup実行中のframe
- ABI上固定されたmemory


概念的な分類：

ReuseClass =
  Reusable
  | NonReusable
  | Pinned
  | ForeignOwned

##### 9. Closure環境
###### 9.1 表現

Closureは、codeと捕捉環境を持つ値として扱う。

Closure {
  code,
  environment
}

###### 9.2 生成

Closure生成時：

- 捕捉値をClosure環境へ移送する
- 外側でも必要な値にはdupを挿入する

###### 9.3 解放

Closureが不要になった場合：

- 捕捉環境内の値をdrop
- Closure領域を解放

###### 9.4 Closure identity

Closureの物理identityや等値比較を一般公開しない。

Closure equality:
提供しない

Closure address:
公開しない

###### 9.5 Scoped値のcapture

varまたはscoped resourceを捕捉するClosureは、そのscope内でのみ使用できる。

そのClosureがscope外へescapeする場合は静的エラーとする。

##### 10. var
###### 10.1 既存意味論

varはescape不能な局所可変状態である。

- 変数scope外へ直接返せない
- EscapeするClosureへ捕捉できない
- Module stateへ保存できない
- 一般共有heap stateにならない

###### 10.2 物理表現

利用者から見える意味が同じなら、実装は次を選べる。

- Stack slot
- Handler-local state
- Mutable frame
- SSA変換
- Heap上の局所slot

###### 10.3 Perceusとの関係

通常の不変値とClosure環境はPerceusで管理する。

varの局所storageは、一般の共有参照カウント付きobjectである必要はない。

##### 11. cell／ref
###### 11.1 v1の方針

一般的なescape可能cell／refはv1では導入しない。

- 共有可変heap stateなし
- Stateful closureの一般機構なし
- 任意の循環参照構築なし

###### 11.2 理由

一般cell／refは次を要求する。

- Heap identity
- Alias規則
- 共有可変状態
- Concurrency memory model
- Race規則
- Cycle処理
- Serialization規則
- GUI再評価時のidentity


Perceusの基本的なgarbage-free保証はcycle-freeなプログラムを前提とし、mutationで循環参照が作られる場合は別の処理が必要になる。

###### 11.3 将来拡張

将来導入する場合はOPEN-MEM-CELL-001で次を検討する。

- Cycleの静的禁止
- Region制約
- Weak reference
- Cycle collector
- Ownership制約
- Handler-local state限定
- 明示的cycle分断

##### 12. 再帰型と循環値
###### 12.1 再帰data型

再帰data型は許可する。

(data tree
  leaf
  (branch tree tree))

###### 12.2 循環する実行時値

通常の不変値から、直接自己参照するheap cycleを構築する機能はv1では提供しない。

再帰型:
許可

有限の不変再帰値:
許可

一般heap上の強参照cycle:
v1では構築不能

###### 12.3 論理的なID参照

NodeId等のopaque IDによる論理参照cycleは、heap object間の強いruntime pointer cycleとは区別する。

文書の所有edgeはtreeであり、非所有参照のcycle可否は各schemaで規定する。

##### 13. Continuation
###### 13.1 表現

One-shot continuationは内部的な所有値として表す。

概念構造：

Continuation {
  stack-segment,
  captured-values,
  handler-frames,
  cleanup-frames
}

###### 13.2 Capture

Continuation生成時：

- 再開に必要な値をContinuationへ移送
- Handler側でも必要な値にdupを挿入
- Cleanup frameをContinuationへ関連付ける

###### 13.3 Resume

Resume時：

- Continuationの所有権を再開先へ移送
- Captured valuesを復元
- Handler／cleanup frameを再接続
- Continuationを消費

###### 13.4 Discard

Discard時：

1. Cleanup frameをLIFOで実行
2. Captured valuesをdrop
3. Continuation objectを解放

###### 13.5 One-shot

Continuationは高々一回だけ消費できる。

Suspended -> Resumed
Suspended -> Discarded


二重resumeまたは二重discardはDefectである。

Runtimeの所有状態が破損している可能性がある場合はTerminal failureへ昇格する。

###### 13.6 Escape

v1ではContinuationをhandler節外へescapeさせない。

Handler節の終了時までに、Continuationはresumeまたはdiscardされなければならない。

未消費ならRuntimeがdiscardする。

##### 14. Failure unwind
###### 14.1 明示的なunwind

raiseは通常の戻り経路を持たない。

Failure loweringでは次を明示する。

- どのframeを離れるか
- どのlocal値をdropするか
- どのcleanupを実行するか
- Error値をどのhandlerへ移送するか

###### 14.2 Dropの欠落禁止

Failure handlerへ直接jumpするだけではならない。

離脱する全frameについて、必要なdropとcleanupを行う。

###### 14.3 Handler節

Handler節では少なくとも次が生存する。

- Operation引数
- One-shot continuation
- Handlerの捕捉環境


Handler節のすべての制御経路で、Continuationが確実に消費されなければならない。

##### 15. Scoped Resource handle
###### 15.1 隠れたscope

bracketは呼出しごとにfreshなresource scopesを生成する。

概念的なhandle型：

resource-handle<s, R>

s:
Resourceの有効scope

R:
File、Socket、GPU buffer等のResource種別


利用者へsを直接記述させない。

###### 15.2 bracketの概念型
bracket:
  acquire : unit -> resource<s, R>
  use     : resource<s, R> -> A
  release : resource<s, R> -> unit
  result  : A


制約：

freshなsは、bracketの結果Aへ現れてはならない


これは内部的にはrank-2相当のscope生成規則として扱える。

###### 15.3 Escape禁止

次を禁止する。

- Handleをbracketの結果として返す
- Handleを含むrecordやdataを返す
- Handleをlistやoptionへ包んで返す
- Handleをdynamicへ封入して逃がす
- 外側stateへ格納する
- Module-level bindingへ格納する
- EscapeするClosureへ捕捉する
- Effect operationのpayloadとして外側handlerへ送る
- 別taskへ送る

###### 15.4 Scope内Closure

Scoped handleをClosureへ捕捉すること自体は許可する。

ただし、そのClosureをresource scope外へescapeさせてはならない。

###### 15.5 独立した結果値

Resourceから生成された値がResource scopeへ依存しない場合は、外へ返せる。

例：

(with-file path
  (fn (file)
    (read-all file)))


read-allが独立した所有bytesを返すなら、結果はscope外で有効である。

##### 16. Resource API
###### 16.1 With-style API

通常利用者向けには、Resource種別ごとのwith-style APIを優先する。

(with-file path
  (fn (file)
    (read-all file)))

###### 16.2 低水準API

Trusted・高度APIとして、汎用bracket、acquire、release pairを提供できる。

通常利用者に手動closeを要求するAPIを標準形としない。

###### 16.3 Release責任
Handle wrapper memory:
Perceus

External resource release:
bracket cleanup frame

###### 16.4 自発的な無効化

Scope内でも、外部要因でResourceが使用不能になることはある。

- Network切断
- GPU device loss
- 外部process終了


Resource操作は必要に応じてfailure resource-errorを持つ。

Scope型はuse-after-releaseを防止するが、外部障害まで排除しない。

##### 17. Borrowed view
###### 17.1 所有値とBorrowed view

Resourceからcopy・decodeされた独立所有値はscope外へ返せる。

Resource内部memoryを直接参照するviewは、Resource scopeへ依存する。

概念型：

borrowed-bytes<s>

###### 17.2 v1の方針

一般利用者向けのborrow systemはv1では導入しない。

- Read borrow
- Mutable borrow
- Alias検査
- Borrowed Closure


は将来項目へ移管する。

Zero-copyが必要なKernel内部APIでは、trustedなscoped viewを使用できる。

##### 18. Weak referenceとFinalizer
###### 18.1 Weak reference

一般利用者向けWeak referenceはv1では提供しない。

理由：

- 値の消滅がメモリ管理時点へ依存する
- プログラム挙動がPerceus内部判断へ依存する
- 将来cell／cycleと複雑に相互作用する


Runtime内部cacheで使用することはできるが、RPXプログラムの意味を変えてはならない。

###### 18.2 利用者定義Finalizer

利用者が任意コードをdrop時に実行するFinalizerは提供しない。

- 実行順序
- Failure
- Resource復活
- Cycle
- Foreign ownership


が複雑になるためである。

###### 18.3 Resource安全網

RuntimeがResource wrapperの最終drop時に安全網としてreleaseを試行することは許可する。

ただし、正しいプログラムはそれへ依存してはならない。

必須解放はbracketで行う。

##### 19. Foreign boundary
###### 19.1 Ownership metadata

Foreign callの各引数は、trusted metadataによって少なくとも次に分類する。

Borrowed:
呼出し中だけ参照し、外部側は保持しない

Owned:
外部側へ所有権を移す

Shared:
外部側が呼出し後も参照を保持する

Copied:
外部表現へ複製する


戻り値も次のように分類する。

- Owned result
- Scoped borrowed view
- Foreign resource handle

###### 19.2 Borrowed契約違反

Foreign側がBorrowed値を呼出し後も保持した場合：

安全に検出可能:
Defect

Memory safetyを破壊:
Terminal failure

###### 19.3 Owned移送

Owned引数を外部側へ移した後、RPX側はその値をdropしてはならない。

外部側が最終的な解放責任を負う。

詳細はOPEN-KER-001へ移管する。

##### 20. Concurrencyへの接続
###### 20.1 v1の範囲

通常のPerceus参照カウントは、単一thread内では非atomic操作を使用できる。

すべての値へ最初からatomic reference countを要求しない。

###### 20.2 共有値

将来、thread間共有を導入する場合は次が必要になる。

- Send規則
- Share規則
- Thread共有可能性
- Atomic reference count
- Shared値のreuse禁止または制約


Perceus関連の実装・研究でも、thread共有される値と共有されない値を区別し、必要な場合だけatomic参照カウントを使うことが重要になる。

詳細はOPEN-CON-001へ移管する。

###### 20.3 Scoped Resource

Scoped resource handleを別taskへ送信できないものとする。

Task共有可能なResourceは、専用のmanaged service handleとして別途設計する。

##### 21. SnapshotとPerceus
###### 21.1 構造共有

不変DocumentSnapshotはrevision間で内部構造を共有できる。

Revision 10:
Root -> A, B, C

Revision 11:
Root -> A, B', C


AとCは複数snapshotから参照される。

###### 21.2 解放

古いsnapshotが不要になると、そのrootをdropする。

他snapshotから共有されている部分は参照カウントが残るため保持される。

###### 21.3 一意性

共有が解消され一意になった構造は、将来の更新処理でreuse候補になり得る。

###### 21.4 観測不能

Snapshotがどの程度構造共有されているかは利用者から観測できない。

##### 22. メモリ割当とEffect
###### 22.1 通常Allocation

通常heap allocationをeffect rowへ記録しない。

Construct
Closure生成
String生成
Snapshot更新
Continuation生成


は、通常の計算意味論を実現するRuntime動作である。

###### 22.2 Perceus操作

次もeffect rowへ現れない。

- Reference count更新
- dup
- drop
- reuse
- Allocationからreuseへの置換

###### 22.3 理由

通常allocationをEffectにすると、ほぼすべての関数がallocation Effectを持ち、Effect rowの情報価値が低下する。

不変値の生成は、外部状態を変更する意味的Effectとは区別する。

##### 23. メモリ予算
###### 23.1 適用単位

明示的なメモリ予算を、隔離可能な実行単位へ設定できる。

例：

- Render job
- Import job
- Export job
- Plugin invocation
- Server request
- Test case
- Document operation
- Worker

###### 23.2 Job増分方式

v1では、予算課金にjob増分方式を採用する。

Job開始前から存在する共有入力:
原則として課金しない

Jobが新規に割り当てた値:
Job予算へ課金

Job内で解放された値:
課金を戻す

Job結果としてcommitされた値:
永続所有領域へ課金を移管


この値は厳密なprocess heap使用量ではなく、そのJobが生み出した増分量を表す。

###### 23.3 Commit時の移管

Jobが生成した値をDocumentやCacheへcommitするとき、課金を移管する。

Commit前:
Job budget

Commit成功:
Document／cache budget

Commit失敗:
Job unwindでdrop


移管先の予算が不足する場合、commitを原子的に拒否する。

##### 24. 予算超過
###### 24.1 型付きFailure

管理されたメモリ予算を超える場合は、型付きFailureとする。

failure resource-exhausted


概念的なerror型：

(data resource-exhausted
  (memory-budget-exceeded memory-budget-info)
  (allocation-too-large allocation-size-info)
  (continuation-budget-exceeded continuation-budget-info)
  (snapshot-retention-limit snapshot-limit-info)
  (external-resource-limit external-limit-info))

###### 24.2 処理

予算超過時：

1. 対象allocationを実行しない
2. resource-exhaustedを発生
3. Continuationをdiscard
4. bracket cleanupを実行
5. Job所有rootをdrop
6. JobをFailedとして終了


共有Runtimeおよび別Jobは健全なら継続できる。

###### 24.3 予約領域

予算超過の報告とcleanupを安全に行うため、Runtimeは最小予約領域を持てる。

予約領域も利用できない場合はTerminal failureである。

##### 25. 単一巨大Allocation
###### 25.1 事前検査

巨大なallocationを実行する前に、次を検査する。

- Size計算overflow
- 実装上限
- Job予算
- Resource種別ごとの上限

###### 25.2 分類
外部入力による不正size:
resultまたはfailure

設定上限超過:
failure allocation-too-large

内部で証明済みのsize条件違反:
Defect

安全に報告不能なallocator failure:
Terminal failure


外部入力から読み取ったdimensionを、検証せずallocation sizeへ使用してはならない。

##### 26. Continuation予算
###### 26.1 課金対象

Continuation生成時、次を予算へ課金する。

- Continuation object
- Captured stack segment
- Handler frame
- Cleanup frame
- 新たに割り当てた補助構造


既に同じJobへ課金済みの捕捉値本体を、物理allocationなしに二重課金しない。

###### 26.2 超過時

Continuation捕捉が予算を超える場合：

- 通常Continuation生成を中止
- 最小resource-exhaustedを構築
- 安全なFailure boundaryへ移動
- Cleanupをunwind


安全なFailure処理に必要な領域も確保できない場合はTerminal failureである。

##### 27. Snapshot保持量
###### 27.1 有効なSnapshot handle

有効なSnapshot handleが存在する間、その論理内容を保持する。

予算不足を理由に、既存handleを暗黙に無効化しない。

###### 27.2 保持policy

次は所有serviceごとに個別の保持policyを持つ。

- Undo履歴
- Transaction履歴
- Preview snapshot
- Render job snapshot
- Plugin snapshot


予算不足時には、新しいhandleの発行、新しいJob開始、追加履歴保持を拒否できる。

###### 27.3 履歴破棄

保持policyによる過去履歴の破棄は、現在Snapshotの意味を変更しない。

##### 28. 一般heap OOM
###### 28.1 管理予算との区別

Process全体のheapが枯渇し、Failure値やcleanup処理に必要な領域も確保できない場合、通常のfailure resource-exhaustedとしての回復を保証しない。

###### 28.2 分類
安全に隔離・報告可能:
failure resource-exhausted

安全なunwindを保証不能:
terminal failure

###### 28.3 Cleanup

Terminal OOMでは、完全なcleanupを保証しない。

事前確保領域による最小診断のみを試行できる。

##### 29. Reference count overflow
###### 29.1 Wraparound禁止

参照カウントをwraparoundさせてはならない。

Wraparoundは生存中の値の早期解放につながり、memory safetyを破壊する。

###### 29.2 実装

十分広いcountを使用し、増加時にoverflowを検出する。

###### 29.3 分類
Objectの所有整合性を維持して隔離可能:
Defect

既にownership状態が不明:
Terminal failure


標準仕様としてsaturating countによる永久leakには移行しない。

###### 29.4 その他の内部不変条件

次もDefectまたはTerminal failureである。

- Countが負になる
- Drop済みobjectを再drop
- Reuse tokenの二重消費
- Shared objectを一意としてreuse
- Foreign ownership状態の矛盾


CompilerのOwnership verifierにより、可能な限り実行前に排除する。

##### 30. Verification
###### 30.1 Ownership verifier

dup／drop挿入後に独立したVerifierを実行する。

検査事項：

1. 各所有参照が全経路でdropまたは移送される
2. 共有地点に必要なdupがある
3. drop後に使用されない
4. 移送後に元参照を使用しない
5. Branch mergeの所有状態が整合する
6. Continuationが高々一回消費される
7. Cleanup frameが高々一回実行される
8. Foreign ownership契約が守られる

###### 30.2 Reuse verifier

Reuse最適化後に次を検査する。

- Reuse対象が一意
- ReuseClassがReusable
- Size／layout条件を満たす
- Pinned／Foreign値を再利用しない
- Reuse前後で観測可能な意味を維持する

###### 30.3 Verifier failure

Verifier failureは利用者のfailure Eではない。

Compiler defect


としてコンパイルを中止する。

不正なBackend codeを生成してはならない。

##### 31. メモリ観測API
###### 31.1 非公開情報

通常RPXコードへ次を公開しない。

- 手動dup／drop
- Reference count
- Reuse token
- Heap address
- 強制free
- 任意objectの厳密size
- Perceus内部queue
- 値の一意性

###### 31.2 許可される情報

次は診断・管理用途として提供できる。

- 設定済みbudget
- 概算残量
- Peak category
- 予算超過情報
- Debug／profiling統計

###### 31.3 安定性

Profiling値は診断用であり、プログラム意味論上の安定値とは保証しない。

- Compiler version
- Backend
- Reuse判断
- Alignment
- Header layout


によって変化し得る。

##### 32. GUI状態
###### 32.1 暗黙cellへの非依存

GUI再評価をまたぐ状態を、Closure環境や暗黙cellの物理寿命へ依存させない。

###### 32.2 推奨モデル
- GUI modelは明示的な不変値
- Updateはmessageまたはtransaction
- 再評価時の状態継承はStable Key
- 一時Closure環境は再評価時に破棄可能


詳細はOPEN-GUI-STATE-001へ移管する。

##### 33. 適合試験
MEM-01：基本的なdrop
(local
  (val value
    (build-large-value))

  (process value)

  (unrelated-work))


期待結果：

valueはprocess後に使用されないため、
unrelated-workより前にdrop可能

MEM-02：共有値
(tuple value value)


期待結果：

必要な共有参照についてdupを挿入
値を早期dropしない

MEM-03：排他的分岐
(if condition
    (consume-left value)
    (consume-right value))


期待結果：

実際に通るbranchへ所有権を移送
不要な事前dupを要求しない

MEM-04：Reuse成功

一意なconstructorを分解して同等sizeのconstructorを再構築する場合：

領域をreuse可能
観測可能な結果は通常allocationと同一

MEM-05：Reuse不成立

値が共有されている場合：

新規allocationへfallback
Failureにしない

MEM-06：Closure capture

不変値を捕捉するClosure：

Closure環境をPerceusで管理
Closureのdrop時に捕捉値をdrop

MEM-07：Scoped handle escape
(with-file path
  (fn (file)
    file))


期待結果：

static error:
resource handle escapes its bracket scope

MEM-08：Closure経由のescape
(with-file path
  (fn (file)
    (fn ()
      (read-all file))))


期待結果：

static error:
escaping closure captures a scoped resource

MEM-09：独立結果
(with-file path
  (fn (file)
    (read-all file)))


read-allが独立したbytesを返す場合：

success

MEM-10：Continuation resume

Resource scope内で一般Effectが発生し、Continuationがresumeされた場合：

- Resourceをreleaseしない
- Cleanup frameを復元
- Continuationを一回消費

MEM-11：Continuation discard

Continuationがdiscardされた場合：

- CleanupをLIFO実行
- Captured valuesをdrop
- Continuationを解放

MEM-12：Failure unwind

Resource scope内でFailureが発生：

- Cleanupを実行
- 離脱frameの通常値をdrop
- Error値をhandlerへ移送

MEM-13：Cycle

一般不変値を使って直接自己参照cycleを作ろうとする場合：

v1では表現不可

MEM-14：予算超過

Job予算を超えるallocation要求：

failure resource-exhausted
Cleanup後にJobを終了
別Jobは健全なら継続

MEM-15：Commit予算不足

Job結果をDocumentへcommitする際、Document budgetが不足：

- Commitを原子的に拒否
- 現在Snapshotを変更しない
- Job一時値をdrop

MEM-16：巨大Allocation

外部入力が実装上限を超える巨大bufferを要求：

failure allocation-too-large


Size計算自体が内部前提に反してoverflowした場合：

Defect

MEM-17：一般heap OOM

Failure値やcleanup用領域も確保できない場合：

Terminal failureを許容

MEM-18：Reference count overflow

参照カウントが実装上限を超える場合：

- Wraparoundしない
- 隔離可能ならDefect
- 所有整合性不明ならTerminal failure

MEM-19：Ownership verifier

drop後の値使用を含む内部Core：

Compiler defectとして拒否
Backend codeを生成しない

MEM-20：Foreign Borrowed違反

Foreign側がBorrowed引数を呼出し後も保持：

安全に検出可能:
Defect

Memory safety破壊:
Terminal failure

##### 34. 移管先OPEN

###### `OPEN-MEM-CELL-001`

- Escape可能cell／ref
- Stateful closure
- Heap identity
- Cycle
- Cycle collector
- Shared mutable state

###### `OPEN-MEM-BORROW-001`

- 一般borrowed view
- Read borrow
- Mutable borrow
- Alias規則
- Borrowed Closure
- Zero-copy API

###### `OPEN-CON-001`

- Thread間Send
- Shared immutable value
- Atomic reference count
- Structured concurrency
- Cancellation
- Task間Resource共有

###### `OPEN-KER-001`

- Foreign ownership metadata
- Borrowed／Owned／Shared／Copied
- Pinned memory
- ForeignValue
- Adapter contract

###### `OPEN-GUI-STATE-001`

- Stable Key
- GUI再評価
- Model stateの継承
- Component lifetime

###### `OPEN-MEM-PROF-001`

- Profiling API
- Peak memory
- Allocation category
- Reuse統計
- Debug ownership trace

##### 35. 最終状態

```text
OPEN-MEM-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- Perceus方式の精密参照カウント
- Effect lowering後のownership解析
- 自動的なdup／drop挿入
- 一意性に基づくreuse
- Functional But In-Place最適化
- 利用者から観測不能な物理共有と更新
- Closure環境の自動管理
- One-shot continuationの所有権管理
- Failure unwind時のdropとcleanup
- bracketとPerceusの責務分離
- Scope外へescapeできないResource handle
- v1での一般cell／ref・cycle・Weak referenceの不採用
- Job単位のメモリ予算
- 型付きの予算超過Failure
- 回復不能OOMのTerminal failure分類
- Ownership verifierとReuse verifier


次に検討する項目は、複数task、Cancellation、thread共有値、atomic reference countおよびFault boundaryを接続する **OPEN-CON-001「並行性・構造化Concurrency・Cancellation」**とする。

#### 13.14.1 `ASY-001` Concurrency and async

##### 状態

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

#### 13.15 `TST-001` Tests and conformance

##### 概要・状態

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

##### Black/white box

- 別test moduleはpublic signatureだけを見る。
- `test-module`/`test-of` companionは同一package内private memberを見られる。
- white-box testはprivateを再exportできず、製品artifactへ含めない。

##### 既存実装

現行Rust testsはvalidity/defect区分、CST round-trip、macro、固定type checker、
effect skeleton、lowering、GUI sync、PDF/SVG/PPTXを経験的に検査する。ただし最終言語の
型安全性や意味論を検証したものではない。

##### テスト原則

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

##### メタ理論

テストは証明の代替ではない。typed random reductionで反例が出ないことをPreservationの
証明と呼ばない。反対に、反例が一つあれば仕様または実装の不健全性を示し得る。

### 14. 統合された構文

#### 14.1 全体EBNF（未完成）

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

#### 14.2 優先順位・結合

S式にはinfix優先順位・結合規則を設けない。`/`、`+`等もlist headのidentifierである。
unit suffixを数値tokenへ含める場合の結合は`OPEN-SYN-002`。

#### 14.3 予約語

readerが認識する`doc`等を除き、特殊形式名をlexically reservedにするか、
binding identityで認識するかは`未決定`。hygienic macroとmodule shadowingのため、
単なる文字列比較にしないことを推奨するが未確定。

#### 14.4 糖衣とCore

| Surface | Core候補 |
|---|---|
| `(val (f x) body)` | `(val f (fn (x) body))` |
| `let` | lambda applicationまたはCore let |
| `var` | fresh state effect + scoped handler |
| `with h body` | `(handle h (fn () body))` |
| doc reader | syntax object constructors/function calls |
| pattern function clauses | `match` |
| `test` | test registryへのtyped value（詳細未決定） |

### 15. 統合された静的意味論

#### 15.1 Kind

```text
K ::= Type | RecordRow | Effect | EffectRow | Module | Signature | Syntax
```

`Effect`を独立kindにするか、effect labelを別categoryにするかは`暫定`。

#### 15.2 共通判断

```text
Δ; M; Γ ⊢ e ⇒ T ! E
Δ; M; Γ ⊢ e ⇐ T ! E
Δ ⊢ T : Type
Δ ⊢ R : RecordRow
Δ ⊢ E : EffectRow
Δ ⊢ T <: U
M ⊢ ModuleImpl : Signature
```

#### 15.3 基本規則

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

#### 15.4 一般化

厳格な構文的value restrictionを`確定`とする。generalization pointは`val`、`let`および`letrec`検査後である。
module signature境界では明示quantifierを保持する。`var`、effectful computation、
resumptionを含む項は原則generalizeしない。

#### 15.5 Subtypingと制約解決

- semantic subtypingを宣言的関係とする。
- algorithmはemptiness/normalization/tallyingへ帰着する。
- RPX全体のalgorithmic completenessは保証しない。A-fragmentに限って完全判定を目標とする。
- algorithmが保守的に拒否する場合、診断で「仕様上不正」と「checker限界」を区別する
  必要がある。

#### 15.6 Module境界

opaque sealing後のabstract typeは、元representationとのsubtyping/型等価を外部で利用
できない。dynamic cast、pattern、serialization、provenanceにも同じ制約が必要。

### 16. 統合された動的意味論

#### 16.1 構成

```text
Configuration = ⟨e, ρ, σ, H, κ⟩
```

- `ρ`: lexical environment
- `σ`: local state/effect instance store
- `H`: handler stack
- `κ`: continuation/evaluation context

実装がCEK/CEKS machineを使うか、直接interpreterを使うかは実装依存にできるが、
観測可能な評価順序を変えてはならない。

#### 16.2 評価順序

`暫定`

1. operator
2. arguments left-to-right
3. record field expressions source order
4. selected `if`/`match` branchのみ
5. handler expressionを先に評価し、その後body

#### 16.3 効果伝播

operationはnearest matching handlerまでcontinuationをcaptureする。unmatched operationは
外側へ伝播する。host boundaryで未処理の場合の分類は`ERR-001`で未決定。

#### 16.4 SourceEdit

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

#### 16.5 観測可能な振る舞い

最低限:

- return value
- handled external output
- generated artifact
- source transaction result
- deterministic diagnostic

allocation address、module representation、private provenance、内部optimizationは観測不能と
する目標。

### 17. エラーと停止状態の分類

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

### 18. 機能間相互作用

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

### 19. メタ理論上の性質

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

### 20. 実装アーキテクチャ

#### 20.1 最終目標パイプライン

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

#### 20.2 必要データ構造

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

#### 20.3 現行crateとの対応

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

### 21. 仕様と実装の対応

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

#### 21.1 型検査器要件

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

### 22. テスト計画

#### 22.1 構文

| ID | 対象 | 内容 |
|---|---|---|
| `TEST-SYN-C001` | LEX/SYN | valid parse/unparse byte round-trip |
| `TEST-SYN-C002` | LEX/SYN | malformed input recovery |
| `TEST-SYN-C003` | SYN | code/doc nesting and escapes |
| `TEST-SYN-C004` | SYN/MAC | reader→macro phase ordering |
| `TEST-SYN-C005` | LEX | Unicode、line ending、深いnest fuzz |
| `TEST-SYN-C006` | SYN | no infix precedence ambiguity |

#### 22.2 静的意味

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

#### 22.3 動的意味

| ID | 対象 | 内容 |
|---|---|---|
| `TEST-DYN-001` | EVAL | application/record left-to-right order |
| `TEST-DYN-002` | EVAL | closure and recursion |
| `TEST-DYN-003` | EFF | nested handlers/resume/forward |
| `TEST-DYN-004` | BND/EFF | var under continuation |
| `TEST-DYN-005` | ERR | explicit error terminal classification |
| `TEST-DYN-006` | Resource | mock/real handler equivalence at API |
| `TEST-DYN-007` | determinism | fixed seed/time/resources replay |

#### 22.4 統合

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

#### 22.5 Property/differential/fuzz

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

### 22.6 横断適合試験

- `INT-ERR-MEM-01`: Resource scope内のFailureでcleanup後にcaptured valueをdropする。
- `INT-EFF-MEM-01`: Resource scope内のEffectをresumeした場合、Resource scopeを維持する。
- `INT-EFF-MEM-02`: Continuation discardでcleanupとPerceus dropを一度だけ実行する。
- `INT-EDT-MEM-01`: Transaction commit失敗時に現在Snapshotを保ち、作業値をdropする。
- `INT-EDT-ERR-01`: 編集競合を`ApplyResult.Rejected`とし、`failure E`へ変換しない。
- `INT-PKG-MEM-01`: Package resource identityはpure、内容取得はResource Effect、wrapper memoryはPerceus管理とする。
- `INT-PKG-NATIVE-01`: Portable版とnative版が同じ公開型・Effect・Failure・適合結果を持つ。

### 23. 完成判定基準

#### 段階1: 設計案が記録された

- 成果物: 本書、状態ラベル、feature/rule/open ID。
- 必須: 既決/未決/現実装を区別。
- 許容: 形式規則未完成。
- リスク: 例示構文が実質的既成事実になる。
- 次条件: 最優先OPEN項目の意思決定。

#### 段階2: 仕様が明確になった

- 成果物: 完全grammar、Core calculus、type/effect/module rules、error分類。
- 必須: 全規範用語と観測可能意味。
- 許容: proof未完。
- リスク: algorithmが宣言仕様を実装できない。
- 次条件: executable reference semanticsを作れること。

#### 段階3: 参照実装が動く

- 成果物: parser、expander、resolver、reference checker/evaluator、IR validator。
- 必須: 小さいconformance corpus。
- 許容: 最適化、全backend、GUI未完。
- リスク: reference implementation自体の誤り。
- 次条件: trace可能なend-to-end実行。

#### 段階4: 適合試験を通過

- 成果物: normative conformance suiteとversioned期待値。
- 必須: 正常・拒否・error分類。
- 許容: 大規模性能。
- リスク: test coverage外の仕様齟齬。
- 次条件: 全MUST ruleに少なくとも正負試験。

#### 段階5: 統合試験を通過

- 成果物: multi-package、GUI、resource、複数backend fixture。
- 必須: cross-feature matrixの高リスク項目。
- 許容: すべての既存製品形式との互換。
- リスク: backend差、resource環境差。
- 次条件: hermetic replay可能。

#### 段階6: 差分・生成・fuzzを通過

- 成果物: generators、shrinkers、reference differential harness、crash corpus。
- 必須: parser、checker、evaluator、IR、Editable。
- 許容: 反例がないことを証明とは呼ばない。
- リスク: generator bias。
- 次条件: 既知counterexample zero、coverage基準達成。

#### 段階7: メタ理論が確認された

- 成果物: 固定Coreに対するProgress/Preservation、subtyping/checker theorem、
  effect/module abstraction proofまたは機械検証。
- 必須: theorem statementと実装範囲の一致。
- 許容: backend pixel correctnessを別定理に分離。
- リスク: proof calculusと実装desugaringの乖離。
- 次条件: elaboration correspondence。

#### 段階8: 実装と形式仕様の対応を確認

- 成果物: phaseごとのrefinement/correspondence、versioned spec、適合報告。
- 必須: optimizer/backendを含む観測同値または明示fallback契約。
- 許容: 実装依存範囲だけの差。
- リスク: foreign library/hardware。
- 完了: 規範仕様の保証範囲について「正しく実装」と主張可能。

### 24. 既知の問題

1. 主要Surface、型、Effect、module、package、編集、失敗、memory設計は解決済みだが、形式規則と機械検証は未完成である。
2. Semantic subtyping、RecordRow、EffectRow、recursive data、constraint solverの性質は未証明である。
3. Perceus pass、Effect lowering、ownership／reuse verifierの参照実装が必要である。
4. Structured concurrency、foreign boundary、layered IR、test、native package置換は未決定である。
5. 現行実装は最終pipelineより単純であり、段階移行が必要である。

### 25. 未証明の性質

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

### 26. 追加で決める必要がある事項

#### 高

1. `OPEN-CON-001`: Structured concurrency、Cancellation、Task failure、値の移送、atomic reference count。
2. `OPEN-KER-001`: ForeignValue、validator、trusted adapter ABI、ownership metadata。
3. `OPEN-IR-001`: Domain／Visual／Motion／Render IR、色、filter、timing、backend tolerance。
4. `OPEN-TST-001`: Test構文、Fault boundary、Failure／Defect、許容Effect。

#### 中

5. `OPEN-PKG-ENTRY-001`: Entry profile。
6. `OPEN-ERR-DIAG-001`: Diagnostic schemaとprivacy。
7. `OPEN-EDT-CODEC-001`: 文書・Transaction codecとmigration。
8. `OPEN-GUI-STATE-001`: Stable Keyとstate継承。
9. `OPEN-NATIVE-PKG-001`: Native packageの選定、独立package原則、fallback、version／ABI契約。

#### 将来

一般`cell`／`ref`、borrow、cycle、macro package／procedural macro、package feature、registry公開、任意build step、CRDT／OT、派生node override、GADT、lazy、runtime reflection。

### 27. 次に行うべき作業

1. Reference parser、resolver、type/effect checkerを整備する。
2. Deep one-shot handler、Failure、bracket、continuation discardをreference evaluatorへ実装する。
3. Effect lowering、closure conversion、Perceus `dup`／`drop` insertion、ownership verifierを実装する。
4. Reuse passとreuse verifierを追加する。
5. 編集Snapshot／Transactionの参照状態機械と適合試験を実装する。
6. `OPEN-CON-001`をTask scopeから順に解決する。
7. `OPEN-KER-001`、`OPEN-IR-001`、`OPEN-TST-001`を順に確定する。
8. Native package候補とportable版同値性要件を`OPEN-NATIVE-PKG-001`で定義する。
9. 各解決済み項目の正負適合試験と横断試験を追加する。

# 第III部 OPEN項目の解決済み決定

### OPEN-CON-001 構造化Concurrency・Task・Cancellation・Task間共有
#### DD-CON-001 決定概要
##### DD-CON-001.1 状態
Status:
RESOLVED

Scope:
TaskとThreadの区別
構造化Concurrency
Task scope
Task handle
TaskResult
spawn／await／cancel
Fail-fastとCollect-all
Cancellation
Effect環境
Task間の値の移送
Perceusとの接続
Schedulerの観測可能な保証
決定的なTest Scheduler
公開Task package

##### DD-CON-001.2 既存仕様との関係

本項目は、既に決定されている次の仕様を前提とする。

- RPXは正格CBVで、同一計算内の評価順序は左から右
- Effect handlerはdeep handler
- Continuationはone-shot
- 通常Failureはfailure Eで表す
- CancellationはFailureおよびDefectと区別する
- bracketは正常終了・Failure・Cancellation・継続破棄時にcleanupする
- 通常値はPerceus方式で管理する
- varはescape不能な局所可変状態
- 一般的なescape可能cell／refはv1では導入しない
- Scoped resource handleはscope外へescapeできない
- DocumentSnapshotとEditTransactionは不変値
- Documentへのcommitは直列化される

#### 0. 基本方針
##### 0.1 Concurrencyの目的

Concurrencyは、複数の処理を時間的に重ねて進めるために使用する。

想定用途：

- GUIを停止させずにRenderingする
- 複数のResource読込みを進める
- PreviewをBackgroundで生成する
- ExportやImportを独立したJobとして実行する
- 複数Fileを一括変換する
- PluginやRender Jobを隔離する

##### 0.2 v1の範囲

v1では、構造化されたTaskの最小機能を提供する。

- Task scope
- 子Taskの開始
- Task結果の回収
- Cancellation
- Fail-fastな並行処理
- 全結果を収集する並行処理
- Schedulerとの協調
- Task単位のFault isolation


次はv1へ含めない。

- OS Threadの直接生成
- Thread IDの安定した公開
- Mutex／Semaphore
- 一般Channel
- Actor
- 共有可変cell
- Detached task
- Task priority
- Schedulerの直接制御
- 分散Task
- Realtime保証

#### 1. TaskとThread
##### 1.1 Task

Taskは、一つの独立した計算単位である。

Taskは開始後、次のいずれかの終了状態へ到達する。

- 正常完了
- 型付きFailure
- Cancellation
- Defect

##### 1.2 Thread

Threadは、TaskをCPU上で実行する実装手段である。

Task:
言語・Runtime上の計算単位

Thread:
Taskを物理的に実行する手段


一つのThreadが複数Taskを切り替えてもよく、複数Threadが複数Taskを並列実行してもよい。

##### 1.3 配置の非公開性

通常のspawnは、別Threadでの実行を保証しない。

同一Thread上の論理的Concurrency:
許可

複数Thread上の並列実行:
安全な場合にRuntimeが選択可能


TaskがどのThreadまたはCPU coreで実行されたかは、通常のRPXプログラムから観測できない。

#### 2. 構造化Concurrency
##### 2.1 親子関係

すべての子Taskは、作成時のTask scopeへ所属する。

親Task
└─ Task scope
   ├─ 子Task A
   ├─ 子Task B
   └─ 子Task C


Task scopeは、所属する全子Taskの寿命に責任を持つ。

##### 2.2 子Taskの放置禁止

Task scopeは、未終了の子Taskを残したまま終了しない。

Scope bodyが終了しようとした時点で未完了の子Taskがある場合、Runtimeは次を行う。

1. 未完了の子TaskへCancellationを要求
2. 子Taskからその子孫へCancellationを伝播
3. 全子Taskのcleanup完了を待つ
4. 全Task handleを終了状態へ移す
5. Task scopeを終了する

##### 2.3 Detached task

親scopeを離れて独立して動き続けるDetached taskは、v1の通常APIでは提供しない。

ProcessやApplication全体に所属する長寿命serviceは、trusted RuntimeまたはHostが別の上位scopeとして管理する。

#### 3. Task scope
##### 3.1 抽象型

Task scopeは、概念的に隠れたscope parametersを持つ。

TaskScope<s>


利用者はsを直接記述しない。

##### 3.2 Scope生成

概念的な公開API：

(with-task-scope
  (fn (scope)
    body))


内部的には、呼出しごとにfreshなsを生成する。

##### 3.3 結果制約

Task scopeのsは、with-task-scopeの結果型へ現れてはならない。

概念型：

with-task-scope :
  (forall s.
    TaskScope<s> -> A
    effects {task, e})
  -> A
  effects {task, e}

##### 3.4 Escape禁止

次を禁止する。

- TaskScope<s>をscope外へ返す
- TaskScope<s>を外側stateへ保存する
- TaskScope<s>をdynamicへ封入して逃がす
- TaskScope<s>をescapeするClosureへ捕捉する
- TaskScope<s>を別scopeまたは別Taskへ送る

#### 4. Task handle
##### 4.1 抽象型

Task handleは概念的に次の型を持つ。

Task<s, A, E>


各parameterの意味：

s:
所属するTask scope

A:
正常完了時の値

E:
型付きFailureのpayload型

##### 4.2 内部Effect row

子Task内部で使用した通常Effect rowは、Task handleの型parameterには保存しない。

たとえば子計算が次を要求するとする。

() -> Image
effects {
  resource,
  clock,
  failure ImageError
}


resourceとclockは子Task実行環境内で処理され、Task handleは次となる。

Task<s, Image, ImageError>


ただし、子Taskのrequired effectsを無視するわけではない。spawn時に、子Taskの実行環境が必要なEffectを提供可能か静的に検査する。

##### 4.3 Escape禁止

Task handleは所属scope外へescapeできない。

禁止：

- Task handleをscopeの結果として返す
- 外側record、list、dataへ保存する
- Module-level bindingへ保存する
- dynamicへ封入して逃がす
- EscapeするClosureへ捕捉する
- 別のTask scopeへ送る

##### 4.4 一回限りの回収

Task handleの終了結果は高々一回だけ回収できる。

Pending
↓ await／await-result
Consumed


二重回収は、静的に判定可能なら静的エラーとする。実行時まで残った場合はDefectとする。

##### 4.5 等値性等

通常のTask handleには次を提供しない。

- 構造的等値
- Identity比較
- Hashing
- Serialization
- 安定したTask ID


Toolingは診断用の一時的なdebug identityを使用できるが、通常プログラムの意味には使用させない。

#### 5. Taskの終了結果
##### 5.1 TaskResult

低水準Task APIは、Taskの終了を通常data値として返す。

概念型：

(data (task-result a e)
  (completed a)
  (failed e)
  (cancelled cancellation-info)
  (defected defect-report))

##### 5.2 Completed
Completed(A):
Taskが正常に値Aを返した

##### 5.3 Failed
Failed(E):
Task内部でfailure Eが未処理のままTask境界へ到達した


Task境界はfailure Eを捕捉し、Failed(E)へ変換する。

##### 5.4 Cancelled
Cancelled(CancellationInfo):
Cancellation要求により構造化unwindを完了した

##### 5.5 Defected
Defected(DefectReport):
Task-localなDefectをFault boundaryが隔離した


Runtimeまたはscope全体の健全性を保証できないDefectは、TaskResultへ変換せずTerminal failureとする。

#### 6. Cancellation情報
##### 6.1 理由

Cancellationは通常Failureと区別する。

概念的な理由：

(data cancellation-reason
  user-requested
  parent-cancelled
  scope-closed
  sibling-failed
  timeout)

##### 6.2 CancellationInfo

CancellationInfoは少なくとも理由を保持できる。

CancellationInfo {
  reason
}


Task ID、実時刻、Thread ID等を規範的な公開情報にはしない。

##### 6.3 Cleanup failure

Cancellation中に発生した通常のcleanup failureは、Taskの主終了状態をFailedへ変更せず、suppressed diagnostic metadataとして記録する。

Primary termination:
Cancelled

Suppressed:
cleanup failure


Resource ownershipやRuntimeの不変条件が破られた場合はDefectまたはTerminal failureへ昇格する。

#### 7. spawn
##### 7.1 意味

spawnはTask scopeへ所属する子Taskを作成する。

概念型：

spawn :
  TaskScope<s>
  -> (() -> A
      effects {child-effects, failure E})
  -> Task<s, A, E>
  effects {task}

##### 7.2 遅延された計算

spawnへ渡す計算は無引数関数とする。

(spawn scope
  (fn ()
    (load-image resource)))


RPXがCBVであるため、計算を無引数関数へ包むことで、spawn前の事前評価を防ぐ。

##### 7.3 Effect検査

spawn時に次を検査する。

RequiredEffects(child-computation)
⊆
ProvidedEffects(child-task-environment)


提供不能なEffectがあれば静的エラーとする。

Task requires an Effect unavailable
in its execution environment:
  window

##### 7.4 親TaskへのEffect

spawn自体が親Taskで発生させるEffectはtaskである。

子Task内部で処理されるresource、clock等を親Taskのeffect rowへ単純に加えない。

##### 7.5 子Failure

子Task内部のfailure Eは、spawn時には親へ伝播しない。

spawn:
Task handleを返す

Task完了:
Failed(E)として保存

await:
必要に応じて親へ再伝播

#### 8. await-result
##### 8.1 型
await-result :
  Task<s, A, E>
  -> TaskResult<A, E>
  effects {task}

##### 8.2 意味
- Taskが未完了なら現在TaskをSuspendedにする
- Runtimeは別の実行可能Taskを進める
- 対象Task完了後に現在Taskを再開する
- Task handleを消費する
- TaskResultを通常値として返す

##### 8.3 Failure

await-resultは、子Taskのfailure Eを親のFailureとして再発生させない。

詳細な終了状態を通常値として扱いたい場合に使用する。

#### 9. await
##### 9.1 型

概念型：

await :
  Task<s, A, E>
  -> A
  effects {
    task,
    failure E
  }

##### 9.2 終了状態の変換
Completed(A):
Aを返す

Failed(E):
親Taskでfailure Eを再発生

Cancelled:
親TaskへCancellationを伝播

Defected:
外側Fault boundaryへDefectを再通知


Cancellationは一般EffectRowの公開labelへ追加せず、task Effectの実行契約として扱う。

##### 9.3 使い分け
終了状態を個別処理する:
await-result

正常値を取得し、Failure等を伝播する:
await

#### 10. cancel
##### 10.1 型
cancel :
  Task<s, A, E>
  -> unit
  effects {task}

##### 10.2 意味

cancelはTaskを直ちに破壊せず、Cancellation要求を記録する。

cancel:
要求

await-result:
終了確認

##### 10.3 Handle消費

cancelはTask handleを消費しない。

(seq
  (cancel task)
  (await-result task))


のように、Cancellation後のcleanup完了を確認できる。

##### 10.4 冪等性

Cancellation要求は冪等とする。

未要求Taskへのcancel:
要求を設定

既にCancelling:
追加変化なし

既に終了:
追加変化なし


Consumed済みhandleは再使用できない。

#### 11. Cancellation
##### 11.1 協調的な中止

Cancellationは任意の機械命令地点でTaskを強制破壊する操作ではない。

Cancellation要求
↓
安全なCancellation pointで観測
↓
構造化unwind
↓
Cleanup
↓
Cancelled

##### 11.2 Cancellation point

少なくとも次をCancellation pointとする。

- await
- await-result
- yield
- check-cancelled
- 非同期I/O待機
- Timer待機
- Runtimeが挿入する安全なsafepoint

##### 11.3 長時間の純粋計算

長時間の純粋計算へ対応するため、次を併用する。

- Libraryが明示的にcheck-cancelledを呼ぶ
- Compilerがloop back-edge等へ安全なsafepointを挿入できる


任意命令地点での強制停止は行わない。

##### 11.4 通常Handlerからの分離

Cancellationは、通常利用者が一般handleで捕捉して握り潰せるEffectにはしない。

実装上は非再開型の制御移動として既存handler機構を利用できるが、処理責任はTask Runtime境界に限定する。

##### 11.5 Effect row

Cancellationは公開EffectRowの独立labelへ載せない。

TaskはCancellationされ得る:
task Effectの契約

Cancellationが通常Failureである:
否

#### 12. Cancellationの伝播
##### 12.1 親から子

親TaskがCancellationを観測した場合、所属する全子TaskへCancellationを伝播する。

親
├─ 子A
│  └─ 孫A1
└─ 子B


終了順：

1. 子A・子BへCancellation
2. 孫A1へCancellation
3. 子孫Taskのcleanup
4. 子Task終了
5. 親Taskのcleanup
6. 親Task終了

##### 12.2 子から親

子Taskが単独でCancelledになっても、親Taskを自動的にcancelしない。

親または高水準combinatorが、そのCancelled結果をどのように扱うか決める。

##### 12.3 Scope終了

Task scope bodyが終了するときに残っている未完了Taskには、scope-closedを理由としてCancellationを要求する。

#### 13. Cleanup
##### 13.1 Cancellation時

Cancellationを観測したTaskは、次の手順で終了する。

1. 子TaskへCancellationを伝播
2. 子Taskの終了を待つ
3. bracket cleanupをLIFOで実行
4. Task所有rootをPerceusでdrop
5. Cancelledとして終了

##### 13.2 Cleanup中のmask

Cleanup中にCancellationを再観測して後始末を中断してはならない。

Cleanup区間:
Cancellation観測を一時的に延期

Cancellation要求:
消去しない

##### 13.3 利用者によるmask

通常利用者が任意の長時間Cancellation maskを作るAPIは、v1では提供しない。

Runtimeとbracketの限定されたcleanup区間だけがmaskを使用できる。

#### 14. yieldとcheck-cancelled
##### 14.1 yield

概念型：

yield :
  unit -> unit
  effects {task}


意味：

- 現在TaskをRunnableへ戻す
- Schedulerへ次Task選択の機会を返す
- Cancellation要求を確認する
- 再開後にunitを返す


yieldは特定の別Taskが必ず実行されることを保証しない。

##### 14.2 check-cancelled

概念型：

check-cancelled :
  unit -> unit
  effects {task}


Cancellation要求がなければunitを返す。

要求があれば構造化Cancellationを開始する。

#### 15. Fail-fast：all
##### 15.1 用途

allは、全ての子計算が成功しなければ一つの結果を構築できない場合に使用する。

例：

- 複数pageから一つのPDFを作る
- Font、Image、Layoutを全て準備する
- 一つのBuild artifactを複数工程で作る

##### 15.2 概念型

同一結果型のcollectionについて：

all :
  List<() -> A
       effects {child-effects, failure E}>
  -> List<A>
  effects {
    task,
    failure E
  }

##### 15.3 意味
全Task成功:
入力順のList<A>を返す

一つがFailed:
未完了兄弟Taskをcancel
全Taskのcleanup完了を待つ
Primary Failureをraise

一つがCancelled:
未完了兄弟Taskをcancel
現在TaskもCancelled

Scope-level Defect:
全兄弟をcancel
全体をDefected

##### 15.4 Primary Failure

Runtimeは最初に観測したFailureの時点でCancellationを開始する。

外側へ返すPrimary Failureは、全Task終了後、入力順で最初にFailedとなったTaskから選ぶ。

Cancellation開始:
最初に観測したFailure

外部へ返すPrimary:
入力順で最初のFailed Task


その他のFailureとcleanup failureはsuppressed metadataとして保持する。

#### 16. Collect-all：collect
##### 16.1 用途

collectは、各処理の結果が独立しており、一つのFailureで他の処理を止めるべきでない場合に使用する。

例：

- 複数Fileの一括変換
- 複数DocumentのValidation
- 独立したAssetのImport

##### 16.2 概念型
collect :
  List<() -> A
       effects {child-effects, failure E}>
  -> List<TaskResult<A, E>>
  effects {task}

##### 16.3 意味
一つの通常Failure:
他Taskをcancelしない

全Task終了:
入力順でTaskResultを返す

親TaskのCancellation:
全子Taskをcancel

Scope-level Defect:
全子Taskをcancel


Task-localに安全に隔離可能なDefectは、Defectedとして個別結果に含められる。

#### 17. Collection API
##### 17.1 Map形式

入力collectionと関数を受け取る高水準APIを提供できる。

collect-map :
  List<X>
  -> (X -> A
      effects {child-effects, failure E})
  -> List<TaskResult<A, E>>
  effects {task}


同様にall-mapを提供できる。

##### 17.2 同時実行数

大量Taskの無制限生成を避けるため、同時進行数を制限できるAPIを提供する。

collect-map-limit :
  PositiveInt
  -> List<X>
  -> (X -> A
      effects {child-effects, failure E})
  -> List<TaskResult<A, E>>
  effects {task}


規則：

limit = 1:
順次実行

limit = N:
最大N Taskを同時進行

limit = 0:
型または入力検査で拒否


同時実行数はThread数を意味しない。

##### 17.3 空入力
all([]):
[]

collect([]):
[]


空入力は正常成功とする。

##### 17.4 結果順

Taskの実際の開始・切替・完了順とは無関係に、結果は入力順で返す。

#### 18. 異なる結果型のTask

allとcollectは、同一型のcollectionを基本とする。

異なる結果型のTaskは、低水準APIで明示的に組み合わせる。

(with-task-scope
  (fn (scope)
    (val font-task
      (spawn scope load-font))

    (val image-task
      (spawn scope load-image))

    (record
      (font (await font-task))
      (image (await image-task)))))


all2、all3等をCore仕様へ大量に追加しない。

ApplicativeなTask abstractionは将来のLibrary設計にできる。

#### 19. Taskへ渡せる値
##### 19.1 Closure

子Taskとして実行するClosureは、捕捉値をTask環境へ移送または共有する。

##### 19.2 所有権移送

親Taskが以後使用しない値は、子Taskへ所有権を移送できる。

親 owns x
↓
子へtransfer
↓
子 owns x

##### 19.3 共有

親と子の両方が使用する不変値は共有できる。

Perceusは必要なdupを挿入する。

##### 19.4 捕捉禁止

v1では、次を子Task Closureへ捕捉できない。

- var
- Scoped resource handle
- TaskScope
- Task handle
- One-shot continuation
- Thread-affine GUI handle
- 非Task-safeなForeign値

##### 19.5 Dynamic

dynamic Sは、上限型SがTask境界で安全と証明できる場合だけTaskへ渡せる。

dynamic int:
候補

dynamic str:
候補

dynamic any:
原則として禁止


dynamicによってscope・Resource・sendability検査を迂回してはならない。

#### 20. Scoped Resource
##### 20.1 Task capture

v1では、親Taskで取得したscoped resource handleを子Taskへ捕捉させない。

不適合：

(with-file path
  (fn (file)
    (spawn scope
      (fn ()
        (read-all file)))))


推奨：

(spawn scope
  (fn ()
    (with-file path
      (fn (file)
        (read-all file)))))


Resourceの取得・利用・解放を同じTaskへ閉じ込める。

##### 20.2 将来拡張

Scope包含・Thread affinity・borrowを型検査できる将来版では、限定的なResource captureを検討できる。

v1では単純な禁止規則を採用する。

#### 21. Document編集との関係
##### 21.1 Task-safeな値
DocumentSnapshot:
Task間で共有可能

EditTransaction:
Task間で移送可能

DocumentId／NodeId:
Task間で利用可能

##### 21.2 現在文書

DocumentHandleは現在状態への接続を表すため、既定ではTask Closureへ直接捕捉させない。

##### 21.3 編集手順

子Taskは次の手順を使用する。

1. 不変Snapshotを受け取る
2. 計算・解析を行う
3. EditTransactionをpureに構築する
4. EditTransactionを結果として親またはserviceへ返す
5. Document serviceがcommitを直列化する


これにより、Task間の共有可変文書状態を導入せずに並行編集計算を行える。

#### 22. 子TaskのEffect環境
##### 22.1 原則

子Taskは、親Taskのhandler stack全体を暗黙に継承しない。

Task開始時に、Runtimeが子Task用のEffect環境を構築する。

##### 22.2 分類

EffectまたはCapabilityを次の4種類として扱う。

Reinstalled:
子Task用にRuntimeが新しく設置

Inherited:
安全に子Taskへ引き継げる

Explicit:
明示Capabilityとして渡す必要がある

Prohibited:
子Taskで使用できない

##### 22.3 Reinstalled

少なくとも次は子Task用に再設置する。

- Scheduler context
- Cancellation context
- Task-local memory budget
- Fault boundary
- Task-local diagnostic context

##### 22.4 Inherited

次は、handler実装がTask-safeである場合に引き継げる。

- 読み取り専用Configuration
- Clock service
- Package resource loader
- 安全なLogging service
- 読み取り専用Module／Package environment

##### 22.5 Explicit

権限が強いものは、明示Capabilityとして子Taskへ渡す。

例：

- Network access
- Storage write
- Process起動
- Document commit service
- Backend service

##### 22.6 Prohibited

次を子Taskへ引き継がない。

- varの局所State handler
- 親のscoped Resource handler
- 親のone-shot continuation
- Task scope
- Transaction適用途中の一時handler
- Thread専用GUI handler

##### 22.7 Effect検査

子Taskのrequired effectsは、子Task環境が提供するEffectの部分集合でなければならない。

RequiredEffects(child)
⊆
ProvidedEffects(child-task-environment)

#### 23. Task境界でのEffect処理
##### 23.1 生のEffectを親へ転送しない

子Task内の未処理Effectを、時間的に独立した親Taskの現在handlerへ直接forwardしない。

子Taskの通常Effectは子Task環境内で処理されなければならない。

##### 23.2 終了変換

Task境界は次を行う。

正常return:
Completed(A)

failure E:
Failed(E)

Cancellation:
Cancelled

Task-local Defect:
Defected(DefectReport)


親へ渡るのは生のEffectではなくTaskResultである。

#### 24. Perceusとの接続
##### 24.1 同一Thread

同一Thread上のTask間では、通常の非atomicなPerceus参照カウントを使用できる。

##### 24.2 所有権移送

値を親から子へ完全に移送する場合、移送後に親はその値を使用しない。

Thread A owns x
↓ transfer
Thread B owns x

##### 24.3 Thread間共有

親子が同じ値を別Threadから参照する場合、その値をShared表現へ昇格させる。

Local:
一つのThreadだけから参照可能

Shared:
複数Threadから参照可能


Shared値には、atomic参照カウントまたは同等のthread-safeな管理を使用する。

##### 24.4 昇格
Local -> Shared:
許可

Shared -> Local:
v1では規範的に要求しない


内部一意性解析による最適化は許可するが、利用者から観測可能にはしない。

##### 24.5 Reuse

Thread間で共有されている値は、一意性が確認できない限りreuseしてはならない。

##### 24.6 通常spawn

通常spawnは別Thread配置を要求しないため、非sendableだがTask-safeな計算を同一Threadで動かす実装余地を持つ。

ただし、var、scoped Resource、Continuation等の捕捉禁止規則は同一Threadの場合にも維持する。

#### 25. Scheduler
##### 25.1 同一Task内

同一Task内では、既存のCBV・左から右の評価順序を維持する。

first
↓
second
↓
third

##### 25.2 Task間

異なるTask間の、次の順序は原則として保証しない。

- 実行開始順
- Task切替順
- Effect発生の相対順
- 完了順

##### 25.3 明示的依存

Task間に順序が必要な場合、await等で明示的な依存関係を作る。

Task Aをawait
↓
Task Bを開始

##### 25.4 await時

await中のTaskはSuspendedとなり、無関係な実行可能Taskの進行を不必要に妨げない。

##### 25.5 弱い進行保証

Runtimeが継続して稼働し、Taskが実行可能であり続け、より高位の終了条件がない場合、そのTaskを意図的に永久放置してはならない。

次は保証しない。

- 一定時間以内の実行
- 一定Turn以内の実行
- Task間の実行割合
- Realtime deadline
- 完了時間の上限

##### 25.6 Priority

利用者指定Task priorityはv1の公開APIへ含めない。

#### 26. Safepoint
##### 26.1 役割

Safepointでは、Runtimeが安全に次を行える。

- Cancellation確認
- Task切替
- Memory budget確認
- Fault確認

##### 26.2 候補
- Loop back-edge
- 再帰呼出し
- Function call境界
- Allocation
- Effect operation
- await
- yield

##### 26.3 挿入

Compilerは安全な位置へsafepointを挿入できる。

具体的頻度は実装依存とする。

Safepointの挿入によって、Cancellationがない正常実行の意味、同一Task内の評価順序、Effect順序を変えてはならない。

#### 27. Blocking操作
##### 27.1 標準待機操作

標準Task対応APIは、待機中に無関係なTaskの進行を不必要に停止させてはならない。

可能な場合：

- TaskをSuspendedにする
- OS非同期I/Oを使用する
- Blocking callを専用workerへ移す

##### 27.2 Foreign call

Foreign callのblocking性、Cancellation可能性、Thread affinityはOPEN-KER-001で宣言する。

標準Schedulerは、任意の未注釈Foreign callがnon-blockingであるとは仮定しない。

#### 28. Test Scheduler
##### 28.1 目的

並行処理の再現可能なTestのため、決定的Schedulerを提供可能にする。

##### 28.2 規定可能な動作

一例として、Test Schedulerは次を使用できる。

Runnable queue:
FIFO

spawn:
queue末尾へ追加

yield:
現在Taskをqueue末尾へ戻す

await:
現在TaskをSuspendedへ移す

Task完了:
待機Taskを規定順でRunnableへ戻す


正確なTest APIはOPEN-TST-001で定める。

##### 28.3 仮想Clock

Test環境では実Clockを仮想Clockへ置き換えられる。

- 明示的に時間を進める
- Timer起床順を再現する
- Timeoutを実時間へ依存させない

##### 28.4 Effect環境

Hermetic Test Runtimeは、必要に応じて次を提供する。

- Deterministic Scheduler
- Virtual Clock
- Seeded Random
- In-memory Resource handler
- Mock Network
- Captured Diagnostic sink

##### 28.5 Trace

Schedulerはdebug・test用途に切替traceを記録・再生できる。

Traceの具体形式は通常プログラムの意味論ではなく、tooling仕様とする。

#### 29. 公開package
##### 29.1 位置付け

Task APIはCore特殊形式として多数追加せず、compiler-nativeな標準packageとして提供する。

公開意味:
通常の型付きpackage API

実装:
Rust Runtime intrinsic

Test:
Test Scheduler handlerへ差替え可能

##### 29.2 Package名

正式な標準namespaceが確定するまで、本仕様ではtask packageと呼ぶ。

候補例：

std/task

##### 29.3 Effect名

Task操作が要求する公開Effect名はtaskとする。

例：

(type render-pages
  (fn document render-output
    (effects
      task
      render
      (failure render-error))))

##### 29.4 抽象型

少なくとも次を抽象型または内部scope付き型とする。

TaskScope<s>
Task<s, A, E>
DefectReport


TaskResult<A, E>、CancellationInfoおよびCancellationReasonは、安全な公開dataとして提供できる。

#### 30. 推奨する最小API
with-task-scope
spawn
await-result
await
cancel
yield
check-cancelled
all
collect
all-map
collect-map
collect-map-limit


正確な関数名は標準package編成時に調整できるが、意味論は本項目に従う。

#### 31. Task状態機械

概念的な状態：

Created
Runnable
Running
Suspended
Cancelling
Completed
Failed
Cancelled
Defected
Consumed


代表的遷移：

Created
-> Runnable
-> Running


待機：

Running
-> Suspended
-> Runnable


正常終了：

Running
-> Completed
-> Consumed


Failure：

Running
-> Failed
-> Consumed


Cancellation：

Created／Runnable／Running／Suspended
-> Cancelling
-> Cancelled
-> Consumed


Defect：

Running
-> Defected
-> Consumed


終了状態からRunningへ戻る遷移は不正である。

#### 32. DefectとTerminal failure
##### 32.1 Task-local Defect

Runtimeの健全性を維持して隔離できるDefectは、Task-local Fault boundaryで処理する。

- 子Taskを終了
- Cleanupを実行
- TaskResult.Defectedを生成

##### 32.2 Scope-level Defect

Task scopeまたは共有Runtime状態へ影響するDefectでは、全子Taskをcancelし、scope全体をDefectedとして終了する。

##### 32.3 Terminal failure

次の場合はTerminal failureとする。

- Scheduler状態の整合性を保証できない
- 同じContinuationを二重resumeし、所有状態が不明
- Completed Taskを再実行した
- Atomic reference count等のmemory safetyが破損
- Cleanup stackの整合性が不明


Terminal failureでは通常のTaskResultを返すことを保証しない。

#### 33. 適合試験
CON-01：正常Task
spawn
-> Completed(A)
-> await-result


期待結果：

Completed(A)を返し、handleをConsumedにする

CON-02：Task handle escape

Task scopeからTask handleを返す。

期待結果：

static error:
Task handle escapes its Task scope

CON-03：二重await

同じTask handleを二回await-resultする。

期待結果：

静的に検出可能ならstatic error
そうでなければDefect

CON-04：Scope終了時の未回収Task

Scope bodyが未完了Taskを残して終了する。

期待結果：

- TaskへCancellation
- Cleanup完了を待つ
- 子Taskを残さずscope終了

CON-05：allのFail-fast

入力3件のうち2件目がFailed。

期待結果：

- 未完了兄弟をcancel
- 全cleanup完了を待つ
- 入力順でPrimary Failureを選ぶ

CON-06：collect

入力3件のうち2件目がFailed。

期待結果：

- 1件目と3件目を続行
- 入力順のTaskResultを返す

CON-07：結果順

完了順がC、A、Bであっても、入力順A、B、Cで結果を返す。

CON-08：親Cancellation

親Taskをcancelする。

期待結果：

- 全子孫Taskへ伝播
- 子孫cleanup
- 親cleanup
- 親がCancelled

CON-09：子Cancellation

子Taskだけをcancelする。

期待結果：

- 子TaskはCancelled
- 親Taskを自動cancelしない

CON-10：Cancellation中cleanup

Cancellation後のrelease中に通常Failureが発生。

期待結果：

- Taskの主結果はCancelled
- Cleanup failureをsuppressedとして記録

CON-11：var capture

varを捕捉するClosureをspawnへ渡す。

期待結果：

static error

CON-12：Scoped Resource capture

親で取得したFile handleを子Task Closureへ捕捉する。

期待結果：

static error

CON-13：ResourceをTask内部で取得

子Task内部でwith-fileを実行する。

期待結果：

正常に実行可能
Task終了前にFileをrelease

CON-14：Task Effect不足

子Taskがwindow Effectを要求し、子Task環境が提供しない。

期待結果：

static error:
unavailable Effect in Task environment

CON-15：DocumentSnapshot共有

親と子Taskが同じ不変Snapshotを読む。

期待結果：

安全に共有
文書内容を変更しない

CON-16：EditTransaction生成

子TaskがSnapshotからEditTransactionを構築して返す。

期待結果：

正常完了
CommitはDocument serviceが直列化

CON-17：await中の進行

Task AがTask Bをawaitし、Task CがRunnable。

期待結果：

Task AをSuspended
Task Cが進行可能

CON-18：yield

実行可能な別Taskがある状態でyieldする。

期待結果：

Schedulerへ選択機会を返す
ただし特定Taskの実行は保証しない

CON-19：決定的Test Scheduler

同じspawn、yield、await、Clock操作列を再実行する。

期待結果：

同じ規定Task切替traceを再現

CON-20：Scheduler Defect

Consumed済みContinuationを再開しようとする。

期待結果：

隔離可能ならDefect
所有状態が不明ならTerminal failure

#### 34. 関連する未決定項目

##### `OPEN-KER-001`

- Foreign callのblocking性
- Cancellation可能性
- Thread affinity
- Owned／Borrowed／Shared
- Worker Threadへの移送
- Native async I/O

##### `OPEN-TST-001`

- Test SchedulerのSurface
- Virtual Clock
- Scheduler trace
- Failure／Cancellation／Defectの期待構文

##### `OPEN-PKG-ENTRY-001`

- Application root Task scope
- CLI／GUI／ServerのRuntime profile
- Process終了時のTask cleanup

##### `OPEN-MEM-PROF-001`

- Taskごとのmemory利用
- Local／Shared参照カウント統計
- SchedulerとContinuation保持量

##### `OPEN-CON-CHANNEL-001`

- Channel
- Stream
- Backpressure
- Task間message passing
- Select

##### `OPEN-CON-PAR-001`

- 明示的なCPU並列API
- Sendable／Shareableの公開表現
- Worker pool
- Parallel collection

#### 35. 最終状態

```text
OPEN-CON-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- 親子関係を持つ構造化Task
- Task scope外へescapeしないTask handle
- Completed／Failed／Cancelled／Defectedの明確な区別
- 協調的Cancellation
- Cancellation時のbracket cleanup
- Fail-fastなall
- 全結果を収集するcollect
- 入力順で安定した結果
- Taskへ持ち込める値とEffectの制限
- Perceusの所有権移送とthread共有への接続
- 配置を固定しない論理的Concurrency
- 弱い進行保証
- Safepointとyield
- 決定的なTest Schedulerと仮想Clock
- Compiler-nativeな標準Task package

### OPEN-KER-001 Kernel・Native package・Foreign boundary・Trusted Adapter ABI
#### DD-KER-001 決定概要
##### DD-KER-001.1 状態
Status:
RESOLVED

Scope:
Kernelの責務
Native package
Portable fallback
Foreign dataとopaque handle
Validation boundary
Native representation
Foreign ownership
Resource lifetime
Task Runtimeとの接続
Blocking／Cancellation
Executor affinity
Callback／Reentrancy
Rust panic／unsafe
Native Adapter ABI
Package／Binding／Type identity
Typed Value Builder
Async completion
Native packageの初期化・終了
Security policy
Conformance test

##### DD-KER-001.2 既存仕様との関係

本項目は、既に決定されている次の仕様を前提とする。

- Domain機能は可能な限りRPX packageとして実装する
- 通常値はPerceus方式で自動管理する
- 外部Resourceの意味的寿命はbracketで管理する
- Effect handlerはdeep handler
- Continuationはone-shot
- Failureはfailure Eとして型へ記録する
- DefectとTerminal failureを通常Failureから区別する
- Taskは構造化Concurrencyに従う
- Cancellationは協調的な構造化中断である
- Native packageは通常package APIを維持する
- PackageInstanceId、ModuleId、BindingId、TypeIdを利用できる
- Module sealingとabstract type identityを維持する
- 一般的なcell／ref、borrow system、weak referenceはv1へ含めない

#### 0. 基本原則
##### 0.1 Kernelの目的

Kernelは、RPXの型付き世界を、Rust Runtime、OS、外部ライブラリ、BackendおよびNative packageへ安全に接続する最小限の信頼層である。

KernelはDomain概念を無制限に追加する場所ではない。

Kernelが担当するもの:
- Runtime valueとの安全な接続
- Effect／Continuation Runtime
- Perceus
- Task Scheduler
- Resource gateway
- Foreign call gateway
- Validation boundary
- Native Adapter ABI
- Typed Value Builder
- Fault isolation

通常RPX packageが担当するもの:
- Document
- Slide
- Circle
- Chart
- Animation
- Layout
- Image処理の高水準API
- Font処理の高水準API
- Backendの公開抽象化

##### 0.2 Native packageの正式な許可

頻繁に利用される機能、性能上の必要性が高い機能、OSや外部ライブラリとの接続が必要な機能には、RustによるNative実装を認める。

特に、標準packageはNative実装を持てる。

例:
- task
- resource
- image
- font
- path
- text shaping
- compression
- PDF／SVG／PPTX backend
- GUI
- network


Native実装の存在は、機能をKernel primitiveへ移すことを意味しない。

公開上は通常のpackage APIを維持する。

#### 1. Package APIとNative実装
##### 1.1 三層構造

Native対応packageを次の三層に分ける。

Package API:
利用者から見える型、Effect、Failure、module、documented behavior

Portable implementation:
通常のRPXで記述された参照実装またはfallback

Native implementation:
Rust、OS API、外部library等を使用する実装

##### 1.2 意味上の正本

意味上の正本は、Native実装ではなく次である。

- Packageの公開signature
- 規範的な意味
- Resource／Cancellation契約
- Conformance test


Native版は、その契約を実装する一つの手段である。

##### 1.3 呼出し側からの不可視性

利用者は通常のimportと関数呼出しを使用する。

(import std/image)

(val image
  (image/decode source-bytes))


Portable版かNative版かによって、次を変えてはならない。

- 公開型
- Required Effect
- Failure型
- Resource lifetime
- Cancellation semantics
- Scope規則
- Binding identity
- 規範的な結果


次の差は許容する。

- 実行時間
- Allocation回数
- Memory使用量
- SIMD利用
- Worker数
- Cache
- Perceus reuseの成立

#### 2. Native化の単位
##### 2.1 登録単位

Native実装の登録単位はpackageとする。

NativePackageImplementation
├─ Binding A
├─ Binding B
├─ Validator
├─ Codec
└─ Representation family

##### 2.2 置換単位

実際のNative置換はbinding単位で行える。

std/image:
decode   -> Native
resize   -> Native
encode   -> Native
metadata -> Portable

##### 2.3 独立package原則

意味的に独立し、Native実装、ABIまたはCapabilityを個別管理する価値がある機能は、原則として独立packageへ分離する。

推奨:
std/task
std/image
std/font
std/resource

避ける:
一つの巨大なstd/nativeへ全機能を集約


Native binding一個ごとにpackageを分ける必要はない。

#### 3. Native実装の分類

Native bindingを次の二種類に分類する。

OptionalAcceleration:
Portable版があり、Native版は高速化

NativeRequired:
Host capabilityまたはNative実装なしでは機能を提供できない

##### 3.1 Optional acceleration

Native版が利用できなければPortable版へfallbackする。

例：

- 文字列検索
- 画像resize
- 数値kernel
- Compression

##### 3.2 Native required

Native実装が必要な機能は、要件を明示する。

例：

- OS Window
- Native File gateway
- GPU driver
- Platform clipboard


利用不能な場合は、構造化されたpackage load errorまたはCapability unavailableを返す。

#### 4. 信頼レベル

Native codeを次の信頼層に分ける。

1. Runtime intrinsic
2. Trusted standard native package
3. Explicitly authorized workspace native package
4. Third-party native package
5. Portable RPX package

##### 4.1 Runtime intrinsic

次のようなRuntime中核だけが使用する。

- Perceus内部
- Continuation
- Scheduler
- Cleanup stack
- Typed Core lowering


Runtime-private ABIへアクセスできる。

##### 4.2 標準Native package

Toolchainとともにbuild、配布およびtestされる。

安定したAdapter ABIを使用し、Runtime-private ABIへ原則アクセスしない。

##### 4.3 Workspace Native package

ManifestまたはHost policyによる明示許可を要求する。

##### 4.4 第三者Native package

v1では既定で自動実行しない。

将来対応では、少なくとも次を必要とする。

- Origin
- Artifact hash
- Signature
- Host capability
- ABI compatibility
- Platform target
- Isolation policy

#### 5. Foreign値の分類
##### 5.1 外部データ

内容を検査して通常のRPX値へ変換できるもの。

- File bytes
- JSON
- Image data
- Font data
- Network message
- Document file

##### 5.2 外部オブジェクト

外部側に実体があり、通常dataへ完全変換しないもの。

- File descriptor
- Socket
- GPU texture
- Native window
- Font face
- Database connection


外部データにはValidation boundaryを使用する。

外部オブジェクトには型付きopaque handleとResource lifetimeを使用する。

#### 6. 未検証値
##### 6.1 通常RPXコードへの非公開

未検証の外部表現を、次の形で通常RPXコードへ公開しない。

- 万能ForeignValue
- Raw pointer
- Untyped native object
- dynamic anyによるResource包装


未検証状態はtrusted adapter内部に閉じ込める。

##### 6.2 Adapter内部表現

Adapter内部では、Rust型等として表現できる。

RawForeignValue
UnvalidatedImage
NativePointer
OsHandle


これらはRPXの通常型ではない。

##### 6.3 dynamicとの区別
dynamic S:
通常のRPX値に関するgradual typing

Foreign handle:
外部Resourceへのopaque capability

Unvalidated external data:
Adapter内部状態


dynamicをForeign Resource、ownershipまたはvalidationの代替にしない。

#### 7. Validation boundary
##### 7.1 定義

Validation boundaryは、未検証の外部表現から、宣言された型の不変条件を満たすRPX値へ移る境界である。

Untrusted representation
↓
Representation validation
↓
Type validation
↓
Semantic validation
↓
Policy validation
↓
Typed RPX value

##### 7.2 Representation validation

次を検査する。

- Length
- Alignment
- Tag
- Pointer validity
- Encoding
- Integer overflow

##### 7.3 Type validation
- Constructor
- Field数
- Field型
- Record shape
- Required property

##### 7.4 Semantic validation
- Dimensionが有効
- Buffer長が一致
- Path command列が有効
- Document所有treeがacyclic
- Resource状態が有効

##### 7.5 Policy validation
- Memory budget
- File size limit
- Capability policy
- Sandbox policy
- Decompression ratio

#### 8. Validator API
##### 8.1 原則

Validatorの正準的な公開結果は、原則としてresultとする。

validate:
Input -> result<Validated, ValidationError>


理由：

- 外部入力不正は通常予想される
- Errorを値として表示・蓄積できる
- 複数errorを収集できる


LoaderやDecoderは、必要なら明示的にFailureへ変換する。

(or-raise
  (validate-image bytes))

##### 8.2 純粋性

可能な限り次を分離する。

Acquire／Read:
Effectful

Parse／Validate:
Pure

Register／Commit:
Effectful

##### 8.3 Handle validator

Live handleの検査はRuntime tableやexecutorへ依存し得るため、pureである必要はない。

通常RPXコードへ直接公開せず、adapter call境界で内部的に実施できる。

#### 9. Validator authority
##### 9.1 Type identityとの関連付け

Validatorは文字列名ではなく次へ関連付ける。

PackageInstanceId
ModuleId
TypeId
ValidatorVersion

##### 9.2 登録権限

Validatorを登録できる主体を次へ限定する。

- TypeIdを定義したpackage
- Manifestで正式指定されたNative implementation
- Toolchain同梱Runtime intrinsic


依存packageが、別packageのsealed abstract type用Validatorを勝手に登録できない。

##### 9.3 成功後の保証

Validator成功後、少なくとも次を保証する。

- Runtime tagと宣言型の一致
- Constructor／fieldの整合
- 型固有不変条件
- Scopeの整合
- Ownership状態
- Adapter identity
- ABI version
- Perceus管理情報

#### 10. Typed Value Builder
##### 10.1 基本方針

Native packageはRuntime内部layoutを直接操作せず、型付きBuilder APIを使用してRPX値を構築する。

候補操作：

build-unit
build-bool
build-int
build-f64
build-str
build-bytes
build-list
build-tuple
build-record
build-variant
build-abstract-native-value
build-resource-handle

##### 10.2 Builder検査
- Runtime identity
- TypeId
- ConstructorId
- Field数
- Field型
- Scope
- Ownership
- RepresentationId
- Memory budget

##### 10.3 Transactional construction

複雑な構築はBuilder session内で行う。

Builder session
├─ 一時所有
├─ Field追加
├─ Validation
└─ Commit／Abort


Commit前に失敗した場合、全ての一時値をcleanupする。

CommitまたはAbort後のsessionは再利用できない。

##### 10.4 Error分類
外部入力不正:
Validation error

Memory budget不足:
resource-exhausted

Native実装とmetadataの矛盾:
Defect

Runtime内部破損:
Terminal failure

#### 11. Abstract typeとNative representation
##### 11.1 Native表現の許可

正式に登録されたNative implementationは、自packageのabstract typeをNative固有表現で実装できる。

公開型:
image

Portable表現:
RPX data

Native表現:
Rust buffer／GPU object等

##### 11.2 条件
- PackageInstanceId一致
- TypeId一致
- Public API hash一致
- ABI version一致
- Validator authorityあり
- Typed Builder使用

##### 11.3 Representation family

表現を次で識別する。

RepresentationIdentity {
  PackageInstanceId,
  TypeId,
  RepresentationId,
  RepresentationAbiVersion
}

##### 11.4 Binding分類
RepresentationNeutral:
内部表現へ依存しない

RepresentationAware:
特定RepresentationIdへ依存する


Representation-aware binding群は同じ表現familyへ整合しなければならない。

#### 12. Representation変換
##### 12.1 Unsafe reinterpretの禁止

Portable表現のpointerをNative表現として読み替える等の暗黙reinterpretを禁止する。

##### 12.2 Converter

表現間変換は正式なconverterとして登録する。

MoveConverter:
元値を消費

CopyConverter:
元値を維持


Shared backing converterはv1の一般機構へ含めない。

##### 12.3 Failure
入力内容／Resource上の変換失敗:
resultまたはfailure conversion-error

Representation metadata不一致:
Defect

ABI不一致:
Native implementationのLoad拒否

#### 13. Foreign handle
##### 13.1 用途別抽象型

一般的なRaw foreign pointerを公開しない。

用途別abstract typeを使用する。

file-handle<s>
socket<s>
gpu-buffer<s>
font-face<s>

##### 13.2 Handle検査

少なくとも次を検査する。

- nullではない
- 正しいadapter
- 正しいResource kind
- 正しいRuntime instance
- 正しいABI version
- 有効なscope
- Open状態
- Executor affinity

##### 13.3 偽造禁止

通常RPXコードは次を行えない。

- intからhandleを作る
- pointerを取得する
- Handle headerを書き換える
- Type／Adapter identityを偽装する
- 内部表現をserializeする

#### 14. Foreign ownership contract

各引数と結果は、ownership metadataを持つ。

Borrowed
Owned
Shared
Copied
Scoped


通常RPX利用者はこれらを手書きしない。

Trusted adapter metadataとCompiler／Runtimeが処理する。

#### 15. Borrowed
##### 15.1 意味

Foreign call中だけ値を使用し、Call終了後は保持しない。

RPX owns x
↓
ForeignがCall中だけ参照
↓
Call終了
↓
RPXは引き続きxを使用可能

##### 15.2 禁止
- Global stateへの保存
- 非同期operationへの直接保存
- Callback Closureへの捕捉
- Call後のpointer使用


Call後も必要ならOwned、SharedまたはCopiedを使用する。

##### 15.3 違反
安全に検出可能:
Defect

Use-after-free等の可能性:
Terminal failure

#### 16. Owned
##### 16.1 意味

Call開始時点で所有権をForeign側へ移送する。

Call前:
RPX owns x

Call後:
Foreign owns x


RPX側は以後その値を使用できない。

##### 16.2 FailureとCancellation

Owned引数はCallの成功、FailureまたはCancellationにかかわらず、RPXへ暗黙には戻らない。

再試行可能なAPIでは、専用結果型によって値を明示的に返す。

SubmitResult<A, E> =
  Submitted
  | Rejected(A, E)

##### 16.3 Foreign側の責任

Foreign側は最終的に次のいずれかを行う。

- drop
- 別ownerへ移送
- 正式な結果としてRPXへ返す
- Resource cleanupへ移す

#### 17. Shared
##### 17.1 意味

RPXとForeign側がCall後も同じ値を保持する。

##### 17.2 Retain token

Shared参照にはopaqueなretain tokenを使用する。

retain
↓
Foreign保持
↓
release token
↓
Perceus drop


Tokenは高々一回だけreleaseできる。

##### 17.3 Thread共有

別Threadで保持する場合はPerceus Shared表現とatomic参照カウント等を使用する。

##### 17.4 利用制限

Sharedは必要な場合だけ使用する。

可能ならBorrowed、OwnedまたはCopiedを優先する。

#### 18. Copied
##### 18.1 意味

Foreign側へ独立表現を複製する。

RPX value
↓ copy
Foreign value


両者の寿命は独立する。

##### 18.2 解放責任

Foreign copyはForeign側が解放する。

##### 18.3 利点と位置付け

Copy costはあるが、ABI安定性と安全性が高い。v1ではzero-copyよりCopiedを選ぶことを許容する。

#### 19. Scoped
##### 19.1 意味

特定scope内だけ有効なviewまたはhandle。

ForeignView<s, T>

##### 19.2 Escape

scope sの外へ返す、保存する、Callbackへ保持させる、Taskへ送ることを禁止する。

##### 19.3 v1での制限

一般borrow systemは導入しないため、Scoped viewは限定されたtrusted APIへ限る。

標準一般APIはOwned結果またはResource handleを優先する。

#### 20. Foreign result

結果を次のように分類する。

OwnedResult
ForeignResource
ScopedResult
SharedResult

##### 20.1 OwnedResult

RPX側が新しい値として所有し、通常値ならPerceus管理へ入れる。

##### 20.2 ForeignResource

Acquire成功時にopaque handleを構築し、bracket cleanupを登録する。

##### 20.3 ScopedResult

既存Resource scopeへ依存する。結果型にscopeが残り、escapeできない。

##### 20.4 SharedResult

Package固有のmanaged handleとして設計する。万能Shared foreign objectは提供しない。

#### 21. Resource lifetime
##### 21.1 責任分離
RPX wrapper memory:
Perceus

外部Resource:
bracket／adapter cleanup

##### 21.2 Acquireと登録

Acquire成功とcleanup登録を不可分に扱う。

Acquire
↓
Handle構築
↓
Cleanup登録
↓
RPXへ公開


公開後にcleanup責任を未登録の状態を作らない。

##### 21.3 状態
Open
-> Closing
-> Closed


Releaseは高々一回だけ実行する。

##### 21.4 Release failure

既存のPrimary／suppressed規則に従う。

Body正常 + Release failure:
Release failureがPrimary

Body Failure + Release failure:
Body FailureがPrimary

Cancellation + Release failure:
Cancelledが主状態

#### 22. Partial construction
##### 22.1 Commit point

Adapter内部で、成功結果公開前にcommit pointを設ける。

Prepare
↓
Validate
↓
Commit
↓
Publish

##### 22.2 Failure
Commit前:
Adapterが部分構築物をcleanup

Commit後:
結果またはbracketがcleanup責任を持つ


中間状態を通常RPXコードへ公開しない。

#### 23. Execution contract

各Native bindingは次を宣言する。

ExecutionClass
CancellationClass
ExecutorAffinity
CallbackPolicy
PanicPolicy
IsolationClass

#### 24. ExecutionClass
Inline
MayBlock
CpuBound
Async

##### 24.1 Inline

Scheduler executor上で直接実行してよい短時間処理。

##### 24.2 MayBlock

I/Oや外部library待機等により長時間戻らない可能性がある。原則としてblocking worker等へ隔離する。

##### 24.3 CpuBound

長時間CPUを占有し得る。CPU用worker pool等へ送る。

##### 24.4 Async

外部operationを開始し、完了通知でTaskを再開する。

具体的な時間閾値は仕様化しない。

#### 25. CancellationClass
Cancellable
Cooperative
NonCancellable

##### 25.1 Cancellable

外部cancel APIを用いて停止要求を送れる。

##### 25.2 Cooperative

Native処理がCancellation tokenを定期確認する。

##### 25.3 NonCancellable

処理完了まで停止できない。Cancellation要求は記録し、Call終了後に結果を破棄してTaskをCancelledにする。

##### 25.4 停止確認

Cancel要求と停止完了を分離する。

外部operation停止確認前にBorrowed値、Shared token、Callback Closure等を解放しない。

#### 26. Executor affinity
Any
SchedulerLocal
MainUI
Dedicated(ExecutorId)

##### 26.1 Any

任意の適切なworker上で実行可能。

##### 26.2 SchedulerLocal

現在Scheduler executor上でのみ実行する。原則として短いInline処理へ限定する。

##### 26.3 MainUI

UI executorでのみ実行可能。

##### 26.4 Dedicated

GPU、Foreign VM、Database等の専用executorで実行する。

OS Thread IDを通常RPX APIへ公開しない。

#### 27. IsolationClass
InProcess
WorkerThread
WorkerProcess

##### 27.1 InProcess

信頼された小さなRust実装等に使用する。

##### 27.2 WorkerThread

MayBlockまたはCpuBound処理をScheduler threadから隔離する。

##### 27.3 WorkerProcess

信頼度の低いcodec、C／C++ library、crashしやすいPlugin等をprocess単位で隔離できる。

Thread内crashがHost process全体を壊し得る場合に使用する。

#### 28. Execution契約の整合性

明らかに不適切な組合せをLoad時に拒否する。

例：

MayBlock + SchedulerLocal:
原則拒否

CpuBound + MainUI:
原則拒否

Async + Dedicated:
許可候補

Inline + MainUI:
短時間UI操作として許可候補


例外的に必要な組合せは、明示的なtrusted overrideと適合試験を要求する。

#### 29. Cancellationと完了の競合

Foreign operationの主結果は一度だけ確定する。

Pending
├─ Complete
└─ AcceptCancellation

##### 29.1 Cancellationが先

後から届いた結果をTaskへ返さない。

所有権契約に従い、次を行う。

- RPX値をdrop
- ForeignResourceをrelease
- Shared tokenをrelease
- Callbackを解除

##### 29.2 完了が先

正常結果またはFailureをTaskへ渡す。

後から来たCancellation要求は終了状態を変更しない。

#### 30. Callback
##### 30.1 Policy

通常Native adapterでは次だけを認める。

None
Queued

##### 30.2 Queued callback

Foreign threadからRPX Closureを直接実行しない。

Foreign event
↓
Runtime queue
↓
Payload validation
↓
適切なTask／executor
↓
RPX Closure実行

##### 30.3 Registration token

Callback登録にはopaque tokenを使用する。

CallbackRegistrationToken


登録時にClosureをShared retainし、解除後にreleaseする。

##### 30.4 Unregister
1. 新規callback受付停止
2. 実行中callbackの終了またはpolicy確定
3. Foreign登録を解除
4. Closure tokenをrelease

##### 30.5 Capture制約

Callback Closureは次を捕捉できない。

- var
- Scoped Resource
- TaskScope
- Task handle
- One-shot continuation
- Thread-affine値
- 非Shared対応値

#### 31. Reentrancy

通常Adapter ABIでは、Foreign call中の同期的RPX Callbackを禁止する。

理由：

- Borrowed lifetime
- Handler stack
- Builder transaction
- Cleanup
- Perceus ownership
- Transaction state


Runtime intrinsicだけが非公開の専用契約で同期Reentrancyを利用できる。

#### 32. Rust panicとunsafe
##### 32.1 Panic

Rust panicを通常のfailure Eへ変換しない。

契約内の外部失敗:
failure E

Rust panic:
Defect候補

##### 32.2 Panic境界

Native call境界でpanicを隔離可能な場合は、所有権とRuntimeの健全性を確認する。

健全性維持:
Defect

健全性不明:
Terminal failure

##### 32.3 Panic policy
NoPanicContract
PanicSafe
AbortOnly


標準Native packageはNoPanicContractを目標とする。

##### 32.4 Unsafe
- unsafeをAdapter境界内へ限定
- safety conditionを文書化
- Raw pointerをRPXへ公開しない
- PanicをFFI境界外へunwindさせない
- Undefined behaviorを正規意味論に含めない

#### 33. Adapter ABIの二層構造
##### 33.1 Adapter ABI

Native packageが使用するversion付きの比較的安定した境界。

- Package descriptor
- Version negotiation
- Capability negotiation
- Opaque value handle
- Typed Builder
- Ownership contract
- Execution contract
- Completion token
- Callback registration

##### 33.2 Runtime-private ABI

Toolchain内部専用であり、互換性を保証しない。

- Perceus header
- Reference count field
- Reuse token
- Continuation frame
- Cleanup stack
- Scheduler queue
- Typed Core
- Runtime Type DAG pointer


一般Native packageへ公開しない。

#### 34. ABI安定性
##### 34.1 v1の方針

version付きopaque Adapter ABIを採用するが、長期binary互換性を過度には約束しない。

保証:
明示されたABI majorとtoolchain互換範囲

許容:
Toolchain更新時のNative package再build

非保証:
Runtime-private layoutの互換性

##### 34.2 互換性レベル
Source API compatibility
Adapter ABI compatibility
Toolchain-private compatibility


Source API compatibilityを最優先する。

#### 35. ABI version
AbiVersion {
  major,
  minor
}

##### 35.1 Major

互換性を壊す変更で増加する。

Major不一致時はNative実装をLoadしない。

##### 35.2 Minor

後方互換な機能追加に使う。

RuntimeがNative packageの要求するminorおよびCapabilityを提供できる場合に使用可能。

##### 35.3 Implementation version

Bug fixや性能改善はABI versionとは別のimplementation versionで管理する。

#### 36. Runtime capability

Native packageは必要Capabilityを列挙する。

例：

typed-value-builder-v1
borrowed-bytes-view-v1
shared-value-token-v1
async-completion-v1
cancellation-token-v1
queued-callback-v1
main-ui-executor-v1
native-representation-v1


条件：

RequiredCapabilities(native)
⊆
ProvidedCapabilities(runtime)


不足時：

OptionalAcceleration:
Portable fallback

NativeRequired:
Load failure

#### 37. Native package descriptor
##### 37.1 Entry point

Native packageは単一の登録entrypointを公開する。

概念的名称：

rpx-native-package-entry

##### 37.2 Descriptor
NativePackageDescriptor {
  descriptor-size,
  descriptor-version,
  ABI-version,
  implementation-version,
  package-identity,
  public-API-hash,
  required-runtime-capabilities,
  supplied-capabilities,
  binding-descriptors,
  representation-descriptors,
  validator-descriptors,
  codec-descriptors
}

##### 37.3 拡張性

各descriptorは次を持てる。

- Struct size
- Struct version
- Flags


未知fieldを勝手に解釈しない。

所有権やExecution classの未知enum値を既知値へ暗黙変換しない。

#### 38. Identity
##### 38.1 Package
PackageInstanceId

##### 38.2 Binding
BindingIdentity {
  PackageInstanceId,
  ModuleId,
  BindingId
}

##### 38.3 Type
TypeIdentity {
  PackageInstanceId,
  ModuleId,
  TypeId
}

##### 38.4 Representation
RepresentationIdentity {
  PackageInstanceId,
  TypeId,
  RepresentationId,
  RepresentationAbiVersion
}


文字列名やRust symbol名だけで対応付けない。

#### 39. Public API hash
##### 39.1 計算対象
- 公開module identity
- BindingId
- TypeId
- 関数型
- Required Effect
- Failure型
- Abstract type identity
- 公開constructor
- Resource／scope制約
- Task境界制約
- Representation要件

##### 39.2 除外対象
- コメント
- 空白
- Source path
- Private binding
- Portable実装本体
- Diagnostic wording

##### 39.3 構造照合

Hash一致だけでなく、Load時に構造化metadataも照合する。

#### 40. Binding descriptor
NativeBindingDescriptor {
  binding-identity,
  function-signature,
  entrypoint,
  ownership-contract,
  execution-contract,
  representation-requirements,
  required-runtime-capabilities,
  optional-or-required
}


Rust symbol名はdescriptor内部のentrypointにすぎず、公開RPX identityではない。

#### 41. Opaque Value handle
##### 41.1 基本

Native packageはRuntime valueへの生pointerを受け取らず、opaque handleを使用する。

RpxValueHandle

##### 41.2 Runtime所属

Handleは一つのRuntime instanceに所属する。

別Runtimeでの使用を禁止する。

##### 41.3 操作

Runtimeのchecked APIを通じて行う。

- Primitiveの読出し
- Value kindの検査
- FieldのBorrowed access
- Typed Builder
- Owned transfer
- Shared retain／release

#### 42. Typed Borrowed view

性能が必要な型に、Call中限定のread-only viewを提供できる。

BorrowedUtf8View
BorrowedBytesView
BorrowedIntArrayView
BorrowedFloatArrayView


規則：

- Call中だけ有効
- Runtimeがpointerとlengthを検証
- Call後の保持禁止
- Async operationへの直接保存禁止
- Mutable viewは一般ABIへ含めない


Call後も必要ならOwnedまたはCopiedへ変換する。

#### 43. Async completion ABI
##### 43.1 Token
AsyncCompletionToken


One-shotである。

##### 43.2 状態
Pending
Completed
Cancelled
Consumed

##### 43.3 通知
complete-success(token, owned-result)
complete-failure(token, owned-error)
complete-cancelled(token)


一度だけ通知できる。

##### 43.4 Resume

Native threadからContinuationを直接resumeしない。

Runtime queueへ通知し、SchedulerがTaskを再開する。

##### 43.5 Late result

Cancellation後に到着した結果はTaskへ返さず、ownership契約に従ってdropまたはreleaseする。

#### 44. Cancellation token ABI

Operation限定のopaque tokenを使用する。

Native側は次を行える。

- Cancellation要求の確認
- Cancel hookの登録


次は行えない。

- Cancellation解除
- 別TaskのCancellation
- Tokenのoperation外保持
- Runtime内部状態への直接アクセス

#### 45. Error ABI

Rust固有errorをそのままRPXへ返さない。

公開Failure型へ明示変換する。

Rust error
↓
Adapter mapping
↓
Typed Builder
↓
公開Failure値


想定外の内部errorを、文字列化して通常Failureへ偽装しない。

契約内:
Failure

Adapter不具合:
Defect

Memory safety不明:
Terminal failure

#### 46. Process isolation
##### 46.1 Worker process

信頼度の低い外部libraryはWorker processへ隔離できる。

##### 46.2 値の受渡し

Process境界では次を使用する。

- Version付きcodec
- 明示的なShared memory protocol
- Validation


Runtime内部pointerやNative representationのraw dumpを送らない。

##### 46.3 Crash分類
外部serviceの停止:
Service failure候補

Toolchain同梱Native実装のcrash:
Defect

Host process自体の整合性喪失:
Terminal failure

#### 47. SerializationとCodec

Opaque Native valueのmemory表現をそのまま保存しない。

Package定義のversion付きcodecを使う。

Value
↓ encode
Portable representation
↓ save／send


再読込時：

Portable representation
↓ decode／validate
Typed value


CodecはPackageInstanceId、TypeId、CodecVersionへ関連付ける。

#### 48. Cross compilation
##### 48.1 HostとTarget
Build-host adapter
Target-runtime adapter


を区別する。

通常のTarget Native packageをBuild host上で実行しない。

##### 48.2 Sidecar metadata

Native artifactには静的に読めるdescriptor metadataを添付できる。

Native binary
+
Hashed／signed sidecar descriptor


Runtime Load時にbinary descriptorと照合できる。

#### 49. Native package初期化
##### 49.1 Describe

Metadataを返す段階。

原則として次を行わない。

- Network access
- Window作成
- Background Thread開始
- User Resource読込み
- Callback登録

##### 49.2 Initialize

Runtime capabilityを受け取り、必要な状態を構築する。

Partial failure時には確保済み状態をcleanupする。

##### 49.3 Arbitrary top-level effect

Native library load時の任意副作用を許可しない。

初期化EffectはRuntimeが明示的に管理する。

#### 50. ShutdownとUnload
##### 50.1 Shutdown順序
1. 新規Call受付停止
2. Async operationの終了・Cancellation
3. Callback解除
4. Native Resource release
5. Shared token release
6. Package local state shutdown

##### 50.2 Unload

v1ではNative libraryの物理的な動的unloadを保証しない。

Runtime instanceの生存中はlibraryをloadしたままにできる。

論理shutdownと物理unloadを区別する。

#### 51. Security policy
##### 51.1 ABIと許可の分離

ABI互換であっても、そのNative codeを実行してよいとは限らない。

##### 51.2 確認項目
- Origin
- Artifact hash
- Signature
- Requested Capability
- Platform target
- Isolation policy
- Workspace／Host permission

##### 51.3 v1既定
Toolchain同梱標準Native:
許可

Workspace Native:
明示許可

Registry由来Native:
既定では実行しない、または将来対応

#### 52. Diagnostic

Native実装を使用できない理由を構造化して報告する。

- ABI major mismatch
- Required runtime capability unavailable
- Public API hash mismatch
- Binding signature mismatch
- Ownership contract mismatch
- Execution contract unsupported
- Representation conflict
- Missing required binding
- Native artifact unavailable
- Security policy rejection


OptionalAccelerationがPortable版へfallbackした場合、通常の実行errorにはしない。

Verboseまたはdebug modeでは理由を表示できる。

#### 53. 適合試験
KER-01：未検証値の非公開

Native adapterがRaw foreign valueを通常RPX値として返そうとする。

期待結果：

拒否

KER-02：Validator成功

有効な外部入力をValidatorへ渡す。

期待結果：

宣言型の不変条件を満たすRPX値を返す

KER-03：Validator失敗

不正な画像dimension等を渡す。

期待結果：

ValidationError
Defectにはしない

KER-04：Validator contract違反

成功を返した値が型不変条件を満たさない。

期待結果：

Defect
Memory safety不明ならTerminal failure

KER-05：Sealed type authority

別packageがsealed abstract type用Validatorを登録する。

期待結果：

Load時に拒否

KER-06：Borrowed保持違反

Foreign側がBorrowed値をCall後も保持する。

期待結果：

検出可能ならDefect
Use-after-freeの可能性があればTerminal failure

KER-07：Owned移送

Owned値をForeignへ渡した後、RPX側で再利用する。

期待結果：

静的拒否またはDefect

KER-08：Owned Call failure

Owned引数を受け取ったCallがFailureで終了する。

期待結果：

値をRPXへ暗黙に戻さない
Foreign側がcleanup責任を持つ

KER-09：Shared release

Foreign側が保持終了時にShared tokenを一回releaseする。

期待結果：

正常drop


二回release：

Defect

KER-10：Partial construction

Builder commit前にFailure。

期待結果：

全一時値をcleanup
部分値を公開しない

KER-11：Resource acquire

Acquire成功。

期待結果：

Handle公開前にcleanup登録済み

KER-12：Late async result

Cancellation確定後にAsync結果が到着。

期待結果：

Taskへ返さない
結果をdrop／release
二重resumeしない

KER-13：Callback unregister race

Callback実行中にunregisterする。

期待結果：

実行中Callbackが安全に終了後、
Closure Shared tokenをrelease

KER-14：Reentrant callback

通常adapterがForeign call中に同期Callbackする。

期待結果：

契約違反として拒否

KER-15：MayBlock

MayBlock bindingを呼ぶ。

期待結果：

無関係Taskを不必要に停止させない

KER-16：Affinity違反

MainUI限定bindingを不適切なexecutorから直接呼ぶ。

期待結果：

静的拒否、dispatch、または安全なRuntime拒否

KER-17：Rust panic

Native bindingがpanicする。

期待結果：

通常Failureへ変換しない
隔離可能ならDefect
不可能ならTerminal failure

KER-18：ABI mismatch

ABI majorが異なるNative package。

期待結果：

Native実装をLoadしない
OptionalならPortable fallback

KER-19：Capability不足

必要CapabilityがRuntimeにない。

期待結果：

Optionalならfallback
RequiredならLoad failure

KER-20：Public API mismatch

Public API hashまたは構造化signatureが一致しない。

期待結果：

Native bindingを接続しない

KER-21：Representation mismatch

値とbindingのRepresentationIdが一致しない。

期待結果：

正式converter、fallback、またはLoad拒否
Unsafe reinterpretをしない

KER-22：Builder型不一致

Native adapterが不正なconstructor fieldを渡す。

期待結果：

Builderが拒否
Adapter Defectとして分類

KER-23：Runtime instance mismatch

Runtime AのValue handleをRuntime Bで使用する。

期待結果：

Defect

KER-24：Native／Portable同値性

同じ規範入力をPortable版とNative版へ与える。

期待結果：

Package仕様が定める同値条件を満たす

#### 54. 関連する未決定項目

##### `OPEN-NATIVE-ABI-SCHEMA-001`

- Descriptorの具体的binary schema
- 整数bit幅
- C ABI型
- Rust SDK trait
- Symbol decoration
- Sidecar形式

##### `OPEN-NATIVE-SEC-001`

- 第三者Native package署名
- Registry policy
- Sandbox
- Capability permission UI

##### `OPEN-KER-CODEC-001`

- Codec version
- Migration
- Cross-process serialization
- Shared memory protocol

##### `OPEN-CON-PAR-001`

- 明示的CPU並列API
- Public Sendable／Shareable制約
- Worker pool policy

##### `OPEN-TST-001`

- Fake executor
- ABI race test
- Native／Portable differential test
- Panic／process crash test

#### 55. 最終状態

```text
OPEN-KER-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- 最小Kernel
- 通常packageとして見えるRust Native implementation
- Binding単位のNative置換
- Portable fallback
- 未検証外部値を閉じ込めるValidation boundary
- Package権限に基づくValidator
- Typed Value Builder
- Opaque Foreign handle
- Borrowed／Owned／Shared／Copied／Scoped ownership
- bracketによる外部Resource管理
- Blocking／CPU／Async実行契約
- CancellationとLate resultの安全な処理
- Executor affinity
- Queued Callback
- Reentrancy制限
- Panic／unsafeのFault分類
- Native representation family
- Version付きopaque Adapter ABI
- Package／Binding／Typeの安定identity
- Capability negotiation
- Native codeのSecurity policy
- Comprehensive conformance tests

### OPEN-IR-001 Layered IR・増分コンパイル・描画意味論・Motion・Backend境界
#### DD-IR-001 決定概要
##### 状態
Status:
RESOLVED

Scope:
- Layered IR
- 差分コンパイルと増分評価
- Geometryと座標系
- 2D StackingとCompositing
- Paint、Color、Alpha、Blend
- 多言語Text基盤
- Time、Signal、Keyframe
- Nested CompositionとTime Transform
- Visual Processing DAG
- Backend Capability
- Fallback、Planning、Emission
- IR拡張・公開・Validation

#### 0. 基本方針

Reciplexaの描画系は、RPXコードから直接Backend命令を生成する単一変換にはしない。

次のLayered IRを基本構造とする。

RPX Source
↓
Typed Core IR
↓
Package固有Domain IR
↓
Layout IR
↓
Visual IR
↓
Evaluated Visual IR
↓
Compositing IR
↓
Resolved Geometry／Render IR
↓
Backend Planning IR
↓
Backend Emission IR
↓
Artifact


各IRを設ける目的は、IRの数を減らすことではなく、各変換を次の性質へ限定することである。

- 単純
- 局所的
- 検証可能
- Cache可能
- 差分更新可能
- 並列実行可能
- Backendから分離可能


全IRを毎回完全にmaterializeすることは要求しない。必要に応じて、遅延view、fragment、Page、Frame、Tile、chunkとして生成できるものとする。

#### 1. 性能を設計要件とする

性能は将来の実装最適化だけの問題ではなく、IR設計上の必須要件とする。

対象とする規模には、少なくとも次を含む。

- 数千Pageの長大文書
- 数百〜数千Slideの資料
- 数十万Text node
- 多数のFont、画像、動画Resource
- 多数Layerを持つComposition
- Frameごとに変化するAnimation
- Filter、Mask、Blendを含む映像処理
- 高解像度Previewと動画Export


各処理段階は、次を可能にしなければならない。

- 変更されていないsubtreeの再利用
- 部分的な再Layout
- Page／Slide／Frame／Tile単位の処理
- Resource decodeの遅延
- Node単位のCache
- Static subtreeのFrame間共有
- Task Runtimeによる並列化
- Stale結果のCommit拒否
- Memory budgetに基づくCache eviction

#### 2. 差分コンパイルと増分処理

SourceからArtifactまでを、差分更新可能な依存pipelineとして設計する。

Source edit
↓
Incremental parse
↓
差分Macro展開
↓
差分名前解決・型検査
↓
Typed Core更新
↓
Domain IR更新
↓
Layout更新
↓
Visual／Compositing fragment更新
↓
Backend再計画
↓
Artifact更新


次の三処理を区別する。

差分コンパイル:
SourceからTyped Coreまで

増分評価:
Typed CoreからDomain／Visual IRまで

差分Rendering:
Render fragment、Tile、Backend出力の更新


各段階は概念的に次を追跡できなければならない。

- Stable identity
- Revision
- Content fingerprint
- Dependency set
- Change set
- Provenance
- Diagnostics


差分処理は常に使用する必要はない。小規模な入力では全体再構築を選択できる。

Incremental result
=
Full rebuild result


を必須条件とする。

#### 3. IR追加の基準

IRの層数を最小化すること自体を目標にしない。

次のいずれかが成立する場合、独立IRを設ける価値がある。

- 前後で成立する不変条件が異なる
- 独立したValidatorを配置できる
- 独立したCache境界になる
- 情報を意図的に失う段階である
- 増分更新の粒度が異なる
- 異なるexecutorやResourceを必要とする
- 複数Backendから共有される
- 専用最適化passが必要である


単なるfield名変更だけの変換には、独立IRを設けない。

#### 4. Domain IR

Domain IRは、文書・Slide・Chart・Diagram等の高水準な意味を保持する。

- Document
- Section
- Paragraph
- Slide
- Table
- Chart
- Figure
- Caption
- Semantic group
- Accessibility structure


Domain IRは一つの万能schemaにはしない。各packageが固有Domain IRを定義できる。

Document IR
Slide IR
Chart IR
Diagram IR
Math IR


各Domain IRは、最終的に共通Visual IRまたは対応するLayout IRへloweringできなければならない。

巨大なDomain構造を即座に完全展開せず、次を許可する。

- Lazy collection
- Virtualized sequence
- Repeated template
- Shared style
- Master／instance
- Query-backed data

#### 5. Layout IR

Layout前後を独立した段階として扱う。

Layout前には次を保持できる。

- Flow
- Constraint
- Intrinsic size
- Relative length
- Style
- Inline object
- Page／Column条件


Layout後には次を確定する。

- 位置
- Size
- Line break
- Page break
- Baseline
- Column placement
- Continuation state


長文では、変更位置から再Layoutし、新旧のLayout stateが一致するstabilization point以降を再利用できるようにする。

#### 6. Visual IR

Visual IRはBackend非依存の意味的な視覚構造である。

基本要素には次を含む。

- Group
- Shape
- Path
- Text
- Image
- Transform
- Clip
- Mask
- Paint
- Opacity
- Blend
- Filter／Visual Processor


Visual IRは不変かつ階層的とする。

親のTransform、Opacity、Style等を全子へ早期に焼き込まず、局所propertyとして保持する。

VisualNode {
  geometry,
  local-transform,
  paint,
  children,
  stacking,
  provenance
}


これにより、親propertyだけの変更で全subtreeを再構築することを避ける。

#### 7. 座標空間

少なくとも次の座標空間を区別する。

Local space
Parent space
Composition space
Viewport space
Device space


共通2D座標系は次とする。

原点:
左上

+x:
右

+y:
下


Layout IR、Visual IR、Compositing IR、Render IRはこの規則へ統一する。

Backend固有座標系への変換はBackend loweringで行う。

内部IRでは空間identityを保持し、概念的に次を区別できるものとする。

Point<Local>
Point<Composition>
Transform<Local, Parent>

#### 8. 長さ・角度・Transform

規範的な長さ単位はlogical pointとする。

1 inch = 72 logical points


相対単位はLayout段階で解決する。

- %
- em
- logical px


Visual IR以降では、原則として解決済みlogical pointを使用する。

角度は次の規則とする。

0度:
+x方向

正の角度:
時計回り


TransformはSurfaceに記述された順に適用する。

T1
→ T2
→ T3


Visual IRでは編集可能なTransform componentを保持し、Render側ではAffine matrixへ正規化できる。

position
anchor
scale
rotation
skew


2D Transformに前後関係を含めない。

2D Transform:
Geometry配置

Stacking:
2D前後関係

3D position.z:
3D幾何学的Depth

#### 9. 2D Stacking

2Dの前後関係の正本は、Stacking context内の子sequenceとする。

先頭:
奥

末尾:
手前


コード記述順は、既定のStacking sequenceを生成する。

ただし、唯一の順序指定方法にはしない。

補助的に整数のstack-levelを使用できる。

小さいstack-level:
奥

大きいstack-level:
手前

同じstack-level:
sequence順


既定値は0とする。

stack-levelは幾何学的なz座標ではない。

Groupは必要に応じて独立Stacking contextを形成し、子のstack-levelは親contextを越えない。

#### 10. ShapeとPath

Visual IRでは高水準Shapeを保持できる。

- Line
- Rectangle
- RoundedRectangle
- Circle
- Ellipse
- Polygon
- Polyline
- Arc
- Path


Resolved Geometry IRでは、共通Pathへloweringする。

Render Pathの基本命令は次とする。

MoveTo
LineTo
CubicTo
Close


Quadratic curveやArcは必要に応じてCubicへ変換する。

Pathは複数subpathを持てる。

Fill ruleはGeometryとは分離する。

NonZero
EvenOdd


Path storageはchunk化・構造共有・部分更新可能でなければならない。

#### 11. Bounds

Boundsを一種類に統一しない。

GeometryBounds
PaintBounds
EffectBounds
VisibleBounds
ConservativeBounds


状態は次とする。

Bounds =
  Empty
  | Finite(Rect)
  | Unbounded
  | Unknown


ConservativeBoundsは、実際の影響範囲を必ず包含しなければならない。

Actual affected region
⊆
ConservativeBounds


過大評価は許可するが、過小評価は禁止する。

差分再描画の基本Dirty regionは次とする。

DirtyRegion =
  union(old-bounds, new-bounds)


Layer順変更、Blend、Filter、Isolationについては、影響するCompositing groupの安全な範囲へ拡張する。

#### 12. Geometry ToleranceとHit testing

Geometry近似はQualityPolicyとdevice-space errorに基づく。

次を区別する。

Geometry approximation tolerance
Bounds safety margin
Hit-test tolerance
Tessellation tolerance


低品質Previewの近似結果をFinal出力へ使用してはならない。

Hit testingは次の段階で行う。

1. Spatial index
2. ConservativeBounds
3. Clip／Mask判定
4. Geometry判定
5. 手前から候補を選択
6. Provenanceから編集対象へ変換


描画順が奥から手前であるのに対し、Hit testingは手前から奥へ行う。

#### 13. PaintとColor

GeometryとPaintを分離する。

v1の基本Paintは次とする。

- None
- SolidColor
- LinearGradient
- RadialGradient


基準色空間はsRGBとする。

合成計算の既定working spaceはLinear sRGBとする。

公開Color:
Straight alphaのsRGB

内部Compositing:
Premultiplied alphaのLinear sRGB


Alphaは0..1を基本範囲とする。

内部Filter計算では有限な範囲外値を許可できるが、出力変換時に明示的なmappingを行う。

#### 14. Compositing

既定合成演算はsource-overとする。

次を区別する。

Paint alpha
Node opacity
Group opacity


Group opacityは、子へ個別適用するのではなく、Group内を合成した後の結果へ適用する。

必要な場合はIsolated groupを形成する。

標準Layer処理順は次とする。

1. GeometryをPaint
2. 子を合成
3. Filterを適用
4. Maskを適用
5. Layer opacityを適用
6. Blend modeで親へ合成


ClipはGeometry制約、MaskはCompositing操作として分離する。

#### 15. Compositing IR

Compositing IRを独立層とする。

概念要素は次のとおりとする。

- PaintGeometry
- DrawText
- DrawImage
- Sequence
- Clip
- Mask
- Filter
- IsolatedGroup
- Composite


この段階で次を解決する。

- Back-to-front順
- Stacking context
- Isolation boundary
- Group opacity
- Blend mode
- Mask
- Filter順序
- 中間surfaceの必要性


意味的Groupを無条件に平坦化してはならない。

#### 16. 多言語Text基盤

共通Text基盤は日本語専用にはしない。

次の仮定を禁止する。

- 一文字は一Glyph
- 一Unicode scalarは一表示文字
- Textは常に左から右
- 空白だけが単語境界
- Glyph順は論理Text順と同じ


共通処理は次の段階に分ける。

Unicode Text
↓
Unicode Analysis IR
↓
Shaping Plan IR
↓
Shaped Text IR
↓
Line Break Plan IR
↓
Line Layout IR
↓
Positioned Text IR


LanguageとScriptを別に保持する。

language
script
direction
writing-mode


Writing modeは少なくとも次を含む。

HorizontalTB
VerticalRL
VerticalLR

#### 17. Text RunとShaping

次を区別する。

Source order
Logical order
Visual order
Glyph storage order


Grapheme clusterとShaping clusterを同一視しない。

Grapheme:
編集・Cursor操作上の単位

Shaping cluster:
Glyphと元Text rangeの対応単位


Shaping Run内では、少なくとも次を一定とする。

- Font
- Font size
- Script
- Language
- Direction
- Writing mode
- Font feature
- Variation axes


Shaping結果は元Unicode text、Text range、Cluster mappingを失ってはならない。

#### 18. 言語package拡張

言語固有規則を共通Render IRへ埋め込まない。

共通Text Layoutは、次の拡張protocolを提供できる。

TextClassifier
LineBreakProvider
HyphenationProvider
JustificationProvider
AnnotationLayoutProvider
GlyphOrientationProvider
FontFallbackPolicy


日本語組版、ルビ、禁則、縦中横等は日本語向けpackageで実装する。

JLReqは、日本語組版packageの主要な規範資料として参照する。共通Text IRは、その実装に必要なUnicode text、縦横方向、注釈関係、Cluster、Line layout情報を失わない。JLReqは主としてJIS X 4051に基づく日本語組版要件を扱うため、低水準Glyph描画仕様ではなく、日本語向けText／Document Layout profileの参照資料として位置付ける。

#### 19. Resource

Visual IRではNative handleを直接保持せず、安定Resource identityを参照する。

ResourceId
ContentHash
Revision
Metadata


対象には次を含む。

- Font
- Image
- Video
- Audio
- Color profile
- Pattern


ImageやVideoは必要時にdecodeし、Preview／Final品質を分離する。

Backend固有ResourceはBackend cacheで管理する。

共通IR:
Resource identity

Native Backend:
Decoded image、GPU texture、Font face等

#### 20. Time

時間型は次のように分離する。

Time:
符号付きの正確な時刻

Duration:
非負の時間長

TimeOffset:
符号付きの時間差

FrameRate:
正の有理数

FrameIndex:
0-basedの非負整数


規範的な時間は有理時間とする。

実装はtime base付き整数tickを使用できる。

時間区間は半開区間とする。

[start, end)


無期限は数値InfinityではなくUnboundedで表す。

Frameの基準評価時刻はFrame開始時刻とする。

TimeからFrameへの変換では、丸めpolicyを明示する。

Floor
Ceil
Nearest
Exact

#### 21. Signal

Signal<A>は解析可能なpureな時間依存値である。

一般のTime -> A関数と完全には同一視しない。

基本分類は次とする。

- Constant
- Keyframed
- Mapped
- Combined
- Sampled
- Selected


通常値AはConstant Signalとして自動的に使用できる。

Signal評価は次を満たさなければならない。

- Pure
- 決定的
- 同じSnapshotとTimeなら同じ結果
- 暗黙I/Oを行わない
- 暗黙の現在時刻を読まない


Randomや外部入力はseedまたはSnapshotとして明示する。

Signal依存関係はv1ではDAGとし、循環を拒否する。

#### 22. Keyframe

Keyframe列は時刻順の非空sequenceとする。

Keyframe<A> {
  identity,
  time,
  value,
  outgoing-interpolation
}


同一Signal・同一時刻のKeyframeは高々一つとする。

Keyframeが一つだけの場合、意味上はConstantと等価である。

最初のKeyframeより前と最後のKeyframeより後は、端の値をHoldする。

隣接Keyframeの補間区間は次とする。

[t0, t1)


補間policyは前Keyframeが所有する。

標準補間は次とする。

Hold
Linear
CubicBezierEasing
Steps


時間Easingと空間上のMotion pathを分離する。

#### 23. 型別補間

補間能力は型ごとのprotocolとして扱う。

補間可能な候補：

- Number
- Length
- Point
- Vector
- Opacity
- Color
- Angle
- Transform component


離散型はHoldまたはStepsを使用する。

- Boolean
- Text
- Font identity
- Resource identity
- Blend mode
- Stack level


Angleには次の補間policyを持たせる。

Shortest
Clockwise
CounterClockwise
Unwrapped


Colorの既定補間はLinear sRGBかつPremultiplied Alphaとする。

TransformはAffine matrix全体ではなく、position、anchor、scale、rotation等を個別に補間する。

Path morphは専用機能とする。

#### 24. Time Transform

親Timelineと子Timelineを区別する。

Temporal Placementは次を持つ。

- Parent active range
- Child source range
- Time transform
- Range policy


Time Transformは任意関数ではなく解析可能な専用IRとする。

標準要素は次とする。

Identity
Offset
Scale
Reverse
Freeze
Compose


範囲policyは次を扱える。

Transparent
Hold
Loop
PingPong
Failure


LoopはEuclidean moduloを使用する。

空のsource rangeへのLoopはValidation errorとする。

可変速度は単純な速度Signalの乗算ではなく、Time Remapとして表す。

動画Frame選択と音声のpitch処理はTime Transformとは分離する。

#### 25. Visual Processing DAG

Blur、色補正、Mask、Composite等の処理依存関係を、型付きVisual Processing DAGとして表す。

これは言語のEffect handlerとは別概念である。

公開上のVisual Processorは通常のpackage関数として見える。

(blur source radius)


Compilerは必要に応じてTyped Semantic DAGへloweringする。

DAGはv1ではacyclicとする。

Feedback、Simulation、Previous-frame参照は専用機構として将来扱う。

#### 26. Processing Node契約

Nodeは概念的に次を持つ。

- Stable identity
- Typed input
- Typed output
- Parameters
- Bounds function
- Required input region
- Time behavior
- Tileability
- Cache contract
- Execution contract
- Fallback contract


PortとParameterを区別する。

Port:
他Nodeの出力

Parameter:
Static値またはSignal


Graph topologyと時間依存Parameterを分離する。

Parameterだけの変化で、DAG全体をFrameごとに再構築しない。

#### 27. Processing DAG評価

出力からのdemand-driven evaluationを基本とする。

現在必要な出力へ到達しないNodeは評価しない。

Node処理は原則としてpureであるため、Dead node除去を安全に行える。

Boundsは入力から出力へ前方伝播する。

必要入力領域は、要求出力領域から入力側へ後方伝播する。

Tileabilityは次へ分類する。

Local
Overlapping
Global
NonTileable


Node cacheは次の単位を持てる。

- Node
- Time
- Time interval
- Spatial region
- Tile
- Quality
- Backend representation


差分無効化は概念的に次の軸で行う。

Node × Time × Spatial region

#### 28. Semantic DAGとExecution Plan

利用者の意味構造と最適化後の実行構造を分ける。

Semantic DAG:
RPXコード、GUI、Provenance上の処理

Execution Plan:
Fusion、Task分割、Backend選択、Cache計画


隣接Nodeは、意味を保つ限り融合できる。

ColorMatrix A
→ ColorMatrix B


を一つの実行operationへ融合できる。

ただし、GUI上の個別Node、Failure origin、Provenanceを失ってはならない。

#### 29. GUI表現

言語を正本とする方針を維持する。

同じ意味構造に対し、次のviewを提供できる。

Code:
正本、抽象化、Module、Test

Property inspector:
個別parameterの編集

Processing Stack:
直列処理の編集

Processing Graph View:
分岐、合流、共有、依存解析

Timeline／Curve:
時間依存propertyの編集


Graph上のNode位置は意味論に含めない。

Graph GUIは、Visual Processing DAGの一つの表示・編集方法であり、必須のSource形式ではない。

#### 30. Backend境界

Render IRまでをBackend非依存の規範的描画意味とする。

対象Backendには次を含む。

- PDF
- SVG
- PPTX
- Raster
- GPU Preview
- Video


各BackendはCapabilityを構造化して宣言する。

- Geometry
- Paint
- Text
- Image
- Compositing
- Filter
- Animation
- Accessibility
- Interactivity
- Document structure


対応水準は次とする。

Native
EquivalentLowering
Approximate
RasterFallback
Unsupported


単純なBooleanだけでCapabilityを表さない。

#### 31. Backend PlanningとEmission

Capability判定は描画hot loopではなくPlanning段階で完了させる。

Render IR
↓
Capability Snapshot
↓
Backend Planning IR
↓
Backend Emission IR
↓
Artifact


Backend Planning IRは、各subtreeの実現方法を確定する。

- Native vector
- Native text
- Geometry lowering
- Font embedding
- Raster fallback
- Resource embedding


Backend Emission IRは、形式固有のObject、命令、part、command等を表す。

#### 32. Fallback Policy

次の基本policyを持つ。

Strict
Compatible
Editable
BestEffort

Strict

意味を十分保持できなければFailureとする。

Compatible

規定済みの等価loweringまたは局所Raster化を許可する。

Editable

見た目の完全一致より、出力先で編集可能な構造を優先する。

BestEffort

Preview等で警告付き近似を許可する。

Output policyは次の価値を別軸として扱える。

- Visual fidelity
- Editability
- Accessibility
- Searchability
- Interactivity
- Performance
- Artifact size

#### 33. Raster Fallback

非対応機能は、可能な限り最小subtreeだけをRaster化する。

Page全体:
可能なら維持

非対応Filter group:
Raster island


Raster islandは次を持つ。

- Bounds
- Resolution
- Color space
- Alpha mode
- Quality policy
- Time identity


Backdrop依存のBlendやFilterでは、正しいCompositing結果を得るためRaster範囲を拡張する。

低解像度Preview用Raster cacheをFinal出力へ使用しない。

#### 34. Text Fallback

Text Backend出力は次の段階を持つ。

1. Native Unicode text
2. Positioned glyph＋Font embedding
3. Glyph outline
4. Raster image


Output policyに基づいて選択する。

Accessibility重視:
Native text／Positioned glyph

Visual fidelity重視:
Positioned glyph／Outline

Editability重視:
Backend native text

Preview:
Rasterを許可可能


Font substitutionをFinal出力で黙って行わない。

#### 35. Streamingと増分Backend

長大なArtifactでは、全Render IRを同時に保持しない。

Planning pass
↓
Page／Slide／Frame単位のEmission
↓
Finalize


必要に応じて次を全体Planningで収集する。

- Font subset
- Shared image
- Cross reference
- Page count
- Link destination
- Outline


差分更新では、変更Page、Slide、Frame、Resourceだけを再計画できるものとする。

最終ファイル形式が全bytesの再生成を要求する場合でも、Backend計算自体の差分再利用を可能にする。

#### 36. IR公開と拡張

Domain IRはpackage固有で自由に定義できる。

共通Visual IRとRender IRは、安全性・Backend互換性・最適化のため、原則として閉じた規範constructor集合を持つ。

拡張が必要な場合は、version付きExtension nodeを登録できる。

Extensionには少なくとも次を要求する。

- Extension identity
- Version
- Type signature
- Validator
- Bounds function
- Required input region
- Tileability
- Time behavior
- Cacheability
- Backend requirement
- Portable fallbackまたはNativeRequired宣言


通常packageが未検証のRender IR constructorを直接作ることは許可しない。

Render IRはTyped BuilderまたはValidatorを介して構築する。

#### 37. IR VersionとSerialization

各永続化可能IRにはversionを持たせる。

IRIdentity
IRVersion
SchemaVersion


Runtime内部cacheにはtoolchain-private表現を使用できる。

長期保存・process間転送にはversion付きportable codecを使用する。

内部memory dump:
禁止

Portable serialization:
Validatorを通して復元


古いIR versionは、明示Migration passによって新versionへ変換する。

Migration不能な場合は、元Sourceまたは上位IRから再構築する。

#### 38. ValidationとFailure

各IR境界にValidatorを配置する。

Domain Validator
Layout Validator
Visual Validator
Compositing Validator
Render Validator
Backend Validator


通常Failureには次を含む。

- Resource resolution failure
- Font resolution failure
- Unsupported backend feature
- Memory budget exceeded
- Approximation tolerance exceeded
- Output profile violation
- Encoder failure


Defectには次を含む。

- Validated IR内のcycle
- Known Boundsの過小評価
- 解決済みResourceの欠落
- Validated Render IR内のNaN
- Capability PlanとBackend実装の矛盾
- Cache key不一致による誤再利用


RuntimeのMemory safetyや全体整合性が不明な場合はTerminal failureとする。

#### 39. 適合試験

少なくとも次を検証する。

- Incremental compileとFull compileの同値性
- Visual subtreeの構造共有
- Transform適用順
- Stack-levelとsequence順
- Group opacityとIsolation
- Boundsの保守性
- Dirty region
- Tile overlap
- Path近似Tolerance
- 多言語ShapingとCluster対応
- BidiのLogical／Visual mapping
- Static SignalのFrame間共有
- Keyframe境界
- Time TransformのLoop／Reverse／Freeze
- Processing DAGのcycle拒否
- Node差分無効化
- Node cacheとFull evaluationの同値性
- Semantic DAGとExecution Planの同値性
- Backend Capability planning
- Fallback policy
- Raster islandの安全性
- Text fallbackの情報保持
- Page／Slide／Frame streaming
- Portable／Native Backendの適合性

#### 40. 性能原則
PERF-IR-01:
変更されていないsubtreeの再構築を要求しない

PERF-IR-02:
各loweringをsubtree単位でcache可能にする

PERF-IR-03:
Page／Slide／Frame／Tile単位で要求可能にする

PERF-IR-04:
全IRの完全materializationを要求しない

PERF-IR-05:
Resourceをstable identityとrevisionで共有可能にする

PERF-IR-06:
時間非依存subtreeをFrame間で再利用可能にする

PERF-IR-07:
BoundsとDependency summaryを取得可能にする

PERF-IR-08:
Backend capabilityをhot loop前に解決する

PERF-IR-09:
Cache evictionによって規範結果を変えない

PERF-IR-10:
並列化によって規範結果を変えない

PERF-IR-11:
Stale Jobを現在revisionへCommitしない

PERF-IR-12:
Provenanceを共有・圧縮可能なidentityで保持する

#### 41. 公開APIと実装

共通IRは通常の型付きpackageとして公開できる。

性能上重要な処理にはRust Native implementationを許可する。

- Text shaping
- Font処理
- Image decode
- Video decode
- Geometry処理
- Tessellation
- Rasterization
- PDF／PPTX Backend
- GPU Backend


Native実装はOPEN-KER-001のTrusted Adapter ABIに従う。

公開意味:
通常package

実装:
Portable RPX／Rust Native／GPU／Worker process


Native版とPortable版は、各packageが定める適合条件を満たさなければならない。

#### 42. 関連する後続項目

##### `OPEN-TEXT-LAYOUT-001`

- 共通Text Layout protocolの正確な型
- Font fallback
- Shaping
- Bidi
- Line breaking
- Justification

##### `OPEN-TEXT-JA-001`

- JLReq参照範囲
- 日本語文字クラス
- 禁則
- 約物
- ルビ
- 縦中横
- 行調整

##### `OPEN-IR-3D-001`

- 3D Scene
- Camera
- Lighting
- Depth
- 2.5D Layer
- 3Dと2D Compositingの接続

##### `OPEN-IR-SIM-001`

- Feedback
- Previous frame
- Simulation state
- Stateful processor
- Fixed-step evaluation

##### `OPEN-BACKEND-PROFILE-001`

- PDF profile
- SVG profile
- PPTX profile
- GPU profile
- Accessibility requirements

#### 43. 最終状態

```text
OPEN-IR-001:
RESOLVED
```


本解決により、Reciplexaの描画系は次の性質を持つ。

- 多層かつ検証可能なIR
- SourceからArtifactまでの差分処理
- 不変値と構造共有
- 大規模文書・Slideへの部分Layout
- Frame・Tile単位の動的Rendering
- 階層的な2D Stacking
- 3D depthとの明確な分離
- Backend非依存のGeometryとCompositing
- 多言語対応のText基盤
- 正確な時間・Signal・Keyframe
- Nested CompositionのTime Transform
- 型付きVisual Processing DAG
- Code／Property／Stack／Graph／Timelineの複数view
- Backend Capabilityに基づくPlanning
- 明示的なFallback policy
- Portable／Native Backendの共存
- 高速な増分評価とCache

### OPEN-TST-001 Test意味論・決定的Test Runtime・Property Test・適合試験
#### DD-TST-001 決定概要
##### 状態
Status:
RESOLVED

Scope:
- Test対象と成果物制作の分離
- Module-local Test
- External Contract Test
- Test-only declaration
- Test宣言と探索
- Test identity
- 依存置換
- Test double
- Test Runtime
- Test isolation
- 決定的Scheduler
- Virtual Clock
- Seeded Random
- TestOutcome
- Assertion
- Property-based testing
- Shrinking
- Differential testing
- Schedule exploration
- Public Test API
- Trusted Conformance API
- 増分BuildとTest選択

#### 0. 基本原則

ReciplexaのTest機構は、すべてのRPXコードへTest作成を要求するものではない。

特に、文書、スライド、図、Animationなどの具体的成果物を記述する本文については、通常のUnit Testを要求しない。

成果物本文
↓
Compile
↓
Validate
↓
Preview
↓
Build／Export


具体的成果物を生成して確認すること自体を、その制作過程における主要な検証と位置付ける。

一方、次についてはTestを記述できるものとする。

- 再利用可能な関数
- Module
- Library package
- Parser
- Validator
- Layout algorithm
- Visual Processor
- Backend
- Native implementation
- Compiler
- Runtime
- Application entry
- Package間の統合


Testは明示的にTestとして宣言されたbindingだけを対象とする。

通常の関数、成果物binding、unitを返す関数、特定の名前を持つ関数が、自動的にTestとして認識されることはない。

#### 1. Testと他の検証機構の分離

次を明確に区別する。

User-authored Test:
利用者が期待条件を明示して検査する

Validation:
Compiler／Runtimeが不正状態を自動的に拒否する

Artifact Check:
生成成果物に利用者固有の要件を課す

Lint:
不正ではないが望ましくない状態を報告する

Preview:
人間が成果物の見た目や内容を確認する

Export Verification:
生成Artifactの形式的整合性を検査する


成果物本文へUser-authored Testを書かない場合でも、型検査、IR Validation、Resource検査、Backend Validation等は行う。

Testを書かない
≠
Validationを行わない

#### 2. Testの対象と所属

Testの対象とTestの所属を分離する。

Test対象
- Binding
- Module
- Package内の複数Module
- 複数Package
- Application
- Compiler
- Runtime
- Native implementation
- Backend

Test所属
- Module-local
- Package external
- Workspace integration
- Documentation
- Toolchain conformance


PackageはTestのBuild、依存管理、探索、配布における上位単位であるが、Test対象をPackage全体に限定しない。

#### 3. Test分類

Testの主要分類を次とする。

TestKind =
  ModuleLocal
  | ExternalContract
  | PackageIntegration
  | WorkspaceIntegration
  | Application
  | Documentation
  | Conformance


この分類はTestの規模だけでなく、次を決定する。

- 可視性
- Compile environment
- Test-only declarationへのアクセス
- Runtime profile
- Isolation class
- Test doubleの注入権限
- Build artifact

#### 4. Module-local Test

Module-local Testは、対象Moduleの内部実装を検査するWhite-box Testである。

対象:
- private helper
- private type
- 内部constructor
- Module内部の不変条件
- 公開前の中間表現


Module-local Testは、論理的に対象Moduleの抽象化境界内部へ所属する。

Module M
├─ Public binding A
├─ Private binding B
├─ Private type T
└─ Module-local Tests


Module-local Testからは、対象Moduleのpublicおよびprivate memberへアクセスできる。

公開interfaceだけを対象とするBlack-box Testと、内部実装を対象とするWhite-box Testの双方には異なる役割があり、Moduleの抽象化境界とTestの境界が常に一致するとは限らない。

#### 5. Private値のescape禁止

Module-local Testがprivate memberへアクセスしても、privateな型、値、constructor authorityをTest scope外へ公開してはならない。

許可:
- private値の構築
- private関数の呼出し
- 内部状態の比較
- 内部不変条件のAssertion

禁止:
- private値をTest runnerへ返す
- private型を別Moduleへ公開する
- private constructorを他Testへ移送する
- Module sealingを迂回する


Test runnerへ返るのは、構造化されたTestOutcome、Diagnostic、Attachment identity等に限定する。

#### 6. External Contract Test

External Contract Testは、対象ModuleまたはPackageを外部利用者と同じ可視性で検査するBlack-box Testである。

見える:
- Public module
- Public binding
- Public type
- Public constructor
- Public Effect
- Public Failure

見えない:
- Private binding
- Private type representation
- Test-only helper
- Native representation
- Runtime-private identity


同じRepositoryまたはPackage内に物理的に配置されていても、CompilerはExternal Contract Testを論理的に外部Moduleとして型検査する。

Rustにおいても、Source近傍のUnit Testはprivate itemへアクセスできる一方、Integration Testは外部crateと同様に公開APIを検査する。この区別は、Module-local TestとExternal Contract Testの設計上の参考とする。

#### 7. Integration Test
Package Integration Test

一つのPackage内の複数Moduleを組み合わせて検査する。

Parser
+
Validator
+
Layout
+
Renderer


Package内部interfaceへのアクセスが必要なTestと、外部公開interfaceだけを使うTestは、可視性metadataによって区別する。

Workspace Integration Test

複数Packageの接続を検査する。

text-layout
+
font
+
language-specific-layout
+
PDF backend


Workspace Integration Testは、通常、Test専用PackageまたはTest targetへ所属し、対象Packageのprivate memberへはアクセスしない。

#### 8. Application Test

Application Testは、Application entry pointと最上位Runtime環境を検査する。

- CLI argument
- Configuration
- Document初期化
- GUI起動
- Root Task scope
- Startup Failure
- Graceful shutdown
- Exit status


Application Testの詳細なentry契約はOPEN-PKG-ENTRY-001で定める。

#### 9. Documentation Test

Documentation Testは、文書内で明示されたCode例を検査する。

DocumentationTestKind =
  Compile
  | Run
  | ExpectFailure
  | Render
  | Ignore


通常の成果物本文をDocumentation Testとして自動実行しない。

Documentation内でTest対象として明示された例だけをTestDescriptorへ変換する。

RustのDocumentation Testは、文書中のCode例を抽出し、compileまたは実行する仕組みを持つ。Reciplexaでも、公開APIの使用例を検証する仕組みとして参考にする。

#### 10. Conformance Test

Conformance Testは、言語・Runtime・標準package・Native implementation・Backend等の規範契約を検査する。

- Incremental compileとFull compileの同値性
- Portable実装とNative実装の同値性
- Task Cancellation
- Resource ownership
- Native Adapter ABI
- Backend Capability
- Raster fallback
- IR Validator


Conformance Testは、通常Testより強い権限を必要とする場合があるため、Trusted Conformance層へ所属させる。

#### 11. Artifact Check

Artifact Checkは通常Testとは別分類とする。

ArtifactCheckKind =
  Structural
  | Layout
  | Resource
  | Accessibility
  | BackendCompatibility
  | VisualRegression
  | Custom


例：

- Page数が規定以内である
- 内容がPageからはみ出していない
- Link切れがない
- 画像解像度が基準以上である
- 指定Fontだけを使用している
- Accessibility情報が存在する
- Raster fallbackが発生していない


Artifact Checkは任意であり、通常の文書・Slide・Animation本文へ記述を要求しない。

実行基盤やReportはTest機構と共有できるが、意味上はUnit／Contract Testと区別する。

#### 12. Test-only declaration

Module-local Testでは、次のTest専用宣言を定義できる。

- Fixture
- Test data
- Fake
- Comparison helper
- Generator
- Internal constructor helper


Test-only declarationは通常Buildから除外する。

通常Build:
Production declarationのみ

Test Build:
Production declaration
+
必要なTest-only declaration


Test-only declarationは公開.rpiへ含めない。

Test-only codeの追加または変更によって、本体bindingのidentityや公開API hashを変化させない。

#### 13. Test Buildによる本体置換の禁止

Test BuildはProduction declarationへ宣言を追加できるが、本番bindingを暗黙に置換してはならない。

許可:
Test helperを追加する

禁止:
本番関数を同名Test関数へ置換する
Test時だけ本番関数の意味を変更する
Test時だけ別のprivate実装へ差し替える


依存実装を差し替える場合は、Effect handler、Module substitutionまたはImplementation selectionを使用する。

#### 14. Test宣言

Testは意味上、通常の型付きbindingとTest metadataから構成する。

Typed Test Binding
+
TestDescriptor


Surfaceには簡潔なtest宣言を構文糖衣として提供できる。

(test addition-works
  (assert-equal
    (+ 1 2)
    3))


Typed Coreでは、通常bindingとTestDescriptorへloweringする。

Rustも、通常関数へ#[test]属性を付けてTest harnessへ登録する方式を採る。この「Test bodyは通常言語で記述し、Test性をmetadataとして付加する」点を参考にする。

Testの最終的なSurface構文は別項目で確定する。

#### 15. Test entryの型

Test entryの意味上の型は、概念的に次とする。

TestContext -> unit
effects {
  test,
  required-test-effects...
}


Surfaceでは通常、TestContext引数を省略できる。

CompilerはTest bindingの型とRequired Effectを検査し、TestDescriptorへ記録する。

例：

Pure Test:
effects {test}

Task Test:
effects {test, task}

Clock Test:
effects {test, task, clock}

Resource Test:
effects {test, resource}


RunnerはRequired Effectから必要なTest Runtime profileを構築する。

#### 16. test Effect

Assertion failure、Test skip、Observation、Test section等は、通常Programのfailure Eとは分離する。

概念的なtest Effect operationは次を含み得る。

- assertion-failed
- record-observation
- attach-diagnostic
- attach-artifact
- skip-test
- enter-section
- leave-section


Assertion APIは通常packageで定義し、不一致をtest Effectを通じてTest Runtimeへ報告する。

Program Failure:
対象Programの公開意味

Test Failure:
Test期待条件の不一致

Defect:
契約違反

Terminal failure:
回復不能


これらを混同しない。

#### 17. Test identity

Test identityは、表示名だけではなく、次から構成する。

TestIdentity {
  PackageInstanceId,
  ModuleId,
  BindingId,
  optional CaseId
}


表示名は別に保持する。

Stable identity:
機械的識別

Display name:
人間向け名称

Qualified display name:
Package／Moduleを含む表示


Parameterized TestやProperty Testの個別caseは、TestIdentityとCaseIdで識別する。

#### 18. TestDescriptor

CompilerはTestごとに構造化されたTestDescriptorを生成する。

TestDescriptor {
  test-identity,
  display-name,
  test-kind,
  visibility-mode,
  binding-type,
  required-effects,
  required-capabilities,
  isolation-class,
  resource-requirements,
  tags,
  source-origin,
  dependency-summary,
  timeout-policy,
  execution-budget,
  ignored-state,
  parameterization,
  implementation-policy
}


Descriptorは静的に読み取れるmetadataとし、一覧表示やTest選択のために任意のTest codeを実行する必要がないようにする。

#### 19. Test探索

Test runnerは、Compilerが生成したTestDescriptor indexからTestを探索する。

Source
↓
Incremental compile
↓
Module TestDescriptor fragment
↓
Package Test index
↓
Workspace Test index
↓
Runner selection


Runnerが毎回Workspace全体のSourceをparseしてTestを探す方式にはしない。

Test探索を次へ依存させない。

- Binding名の接頭辞
- 戻り型
- Directory名だけ
- File名だけ


明示Test metadataを正本とする。

#### 20. Test選択

TestDescriptorに基づき、次の条件でTestを選択できる。

- Workspace
- Package
- Module
- Test identity
- Display name
- Test kind
- Tag
- Isolation class
- Required capability
- Implementation policy
- Source変更との関係


具体的なCLI Filter構文はTooling仕様へ分離する。

#### 21. Disabled・Ignored・Skipped

次を区別する。

Disabled:
Build／Discovery段階で除外

Ignored:
Testとして存在するが通常実行では選択しない

Skipped:
選択後に実行条件を満たさず実行しない


SkippedはPassedではない。

CI profileは、Skipを許可するか、Test suite failureとして扱うかを指定できる。

#### 22. Parameterized Test

同一Test logicを複数の明示caseへ適用できる。

ParameterizedTest<Input> {
  cases,
  body
}


各caseは独立CaseIdを持つ。

TestIdentity
+
CaseId


Caseの表示値そのものをIdentityへ使用しない。

大きなDocument、画像、非hashable値等をcaseとして扱えるよう、CaseIdと表示内容を分離する。

#### 23. 依存置換の基本方針

任意のbindingをTest実行中に名前でpatchする方式は原則として採用しない。

理由は次のとおりである。

- InliningやFusionと矛盾する
- Binding identityとCacheが不安定になる
- Native実装との対応が壊れる
- Test時だけProgram意味が変わる
- private実装への過度な結合を促す


差し替え可能なのは、依存境界として明示されたものに限定する。

- Effect service
- Module parameter
- Capability
- Backend implementation
- Resource provider

#### 24. Effect Test handler

外部作用・Runtime serviceはEffect Test handlerで置換する。

本番:
clock Effect
→ System Clock handler

Test:
clock Effect
→ Virtual Clock handler

本番:
resource Effect
→ File Resource handler

Test:
resource Effect
→ In-memory Resource handler


対象ProgramのSourceとRequired Effectは変更しない。

HandlerだけをTest Runtime profileで差し替える。

#### 25. Module substitution

純粋な抽象処理や業務的依存は、型付きModule substitutionで置換する。

本番:
DatabaseRepository

Test:
InMemoryRepository


Test implementationは本番依存と同じModule signatureを満たさなければならない。

RustにおけるtraitとDependency Injectionの利用は、抽象依存へ別実装を供給する方式として参考になる。

小さな依存は関数引数や明示recordで渡し、環境全体の依存はModule compositionまたはRuntime profileで選択できる。

#### 26. Implementation selection

同じ公開意味に対するPortable／Native／CPU／GPU等の実装差は、MockではなくImplementation selectionとして扱う。

ImplementationPolicy =
  PortableOnly
  | NativeOnly(ImplementationId)
  | PreferNative
  | Differential


External Contract TestまたはConformance Testを、複数実装へ適用できる。

#### 27. Test double

Test doubleを次へ分類する。

Stub:
決められた値を返す

Fake:
簡易だが動作する代替実装

Spy:
呼出しや結果を記録する

Mock:
期待操作を事前宣言して検査する

FaultInjector:
Failure、遅延、Cancellation等を注入する

Simulator:
状態遷移や外部環境を模擬する


既定の推奨順は次とする。

1. Fakeと最終状態の検査
2. Spyによる必要な観測
3. Protocol上必要なInteraction Mock


Strict Mockを通常の業務Testの既定にはしない。

RustのMockallは、trait等からMock型を生成し、引数matcher、呼出し回数、順序、戻り値を設定する方式を提供する。Reciplexaでは同種の機能を型付きTest doubleとして提供できるが、任意binding patchには依存しない。

#### 28. Interaction expectation

Interaction Testでは、次を検査できる。

- Call count
- Argument constraint
- Ordering
- Partial ordering
- Lifetime
- No-more-calls
- Eventually


並行処理では、全操作の完全順序より部分順序を優先する。

Acquire before Use
Release after all Use
Commit after A and B


意味のないTask間順序を固定しない。

#### 29. Interaction履歴

呼出し履歴は構造化eventとして記録する。

InteractionEvent {
  sequence-id,
  logical-time,
  scheduler-step,
  task-id,
  service-id,
  operation-id,
  arguments-summary,
  outcome,
  resource-identity,
  source-origin
}


巨大値や機密値を無制限に保存しない。

Operationごとに記録policyを指定できる。

- Full value
- Structural summary
- Stable identity
- Hash
- Redacted
- Not recorded

#### 30. Test isolation

Test isolationを次の階層に分ける。

IsolationClass =
  Logical
  | Runtime
  | Process
  | Sandbox

Logical

軽量なPure Test向け。

Runtime

独立Scheduler、Clock、Resource table等を必要とするTest向け。

Process

Process-global state、Main-thread API、GPU context、Native panic等を隔離する。

Sandbox

信頼度の低いNative code、Codec、Security Test等へ使用する。

cargo-nextestがTestごとにprocessを分ける理由には、process-global state、Main-thread限定API、GPU context、環境変数、強制終了の隔離が含まれる。Reciplexaではこれを参考にしつつ、すべてのPure Testを常に別processにせず、必要隔離を階層化する。

RunnerはTestが宣言した最低Isolationより強いIsolationを選べるが、弱いIsolationへ下げてはならない。

#### 31. Test root Task scope

各Testへ独立したroot Task scopeを作成する。

Test
└─ Root Task scope
   ├─ Child Task A
   └─ Child Task B


Test body終了時に未完了Taskが残っている場合、既定ではTest failureとする。

Runtimeは次を行う。

1. Task leakを記録
2. Root Task scopeへCancellation
3. 子孫Taskへ伝播
4. Cleanup完了を待つ
5. 終了しない場合はIsolation boundaryを停止


構造化ConcurrencyのCancellation自体を検査するTestでは、明示的な内側scopeを観測対象にできる。

#### 32. 決定的Scheduler

Test Runtimeの既定Schedulerは、同じ入力、seed、Runtime Snapshotから同じTask選択順を生成する。

基本方針は次とする。

- Runnable queueは安定順序
- spawnは決定的にqueueへ追加
- yieldは規定位置へ戻す
- completionは安定したEvent順で処理
- HashMap列挙順やOS Thread順へ依存しない


Test Schedulerは次のModeを持つ。

SchedulerMode =
  DeterministicDefault
  | Replay
  | Randomized
  | Explore

#### 33. Scheduler trace

重要なTask eventを構造化traceとして記録する。

SchedulerEvent {
  event-id,
  logical-step,
  task-id,
  event-kind,
  related-resource-id,
  virtual-time,
  source-origin
}


Traceは次を含み得る。

- Task spawn
- Task run
- yield
- await
- completion
- Cancellation request
- Cleanup
- Timer event
- Native completion


Failure時にTraceを保存し、Replay modeで再現できるようにする。

#### 34. Virtual Clock

Test RuntimeはVirtual Clockを提供する。

ClockMode =
  Manual
  | AutoAdvance

Manual

Test bodyが明示的にClockを進める。

AutoAdvance

Runnable Taskが存在せず、Timerだけが残る場合に、次のTimer時刻へ自動的に進める。

Virtual Clockは、言語の正確なTime、Duration、TimeRangeを使用する。

#### 35. Timeout

次の三種類を区別する。

VirtualTimeout:
Program意味上のTimeout

ExecutionBudgetExceeded:
Task stepや探索数の上限超過

WallClockTimeout:
Host watchdogによる停止


Wall-clock watchdogはRPX上のClockとは独立し、Native hangやRuntime deadlockへ対する最終防御とする。

#### 36. Seeded Random

Random入力は次から決定的に導出する。

RunSeed
+
TestIdentity
+
CaseIdentity
+
StreamIdentity
→ Random stream


Testの物理実行順やworker数でRandom入力が変化しないようにする。

並行TaskではTask identityごとにStreamを分割する。

Input seed
Scheduler seed
Fault seed


を独立して管理する。

#### 37. ResourceとNative completion

Test用Resource serviceはIn-memory providerまたはScripted providerとして実装できる。

- Read
- Write
- Create
- Delete
- Metadata
- Failure injection
- Delayed completion
- Cancellation


Native callbackやAsync completionは、Taskを直接再開せずTest Scheduler queueへEventとして登録する。

これにより、次を再現できる。

- CompletionがCancellationより先
- CancellationがCompletionより先
- Cancellation後のLate result
- Callback解除中の発火

#### 38. Test終了とLeak検査

TestOutcomeはTest bodyの終了だけで確定しない。

終了処理は次の順序とする。

1. Test bodyの暫定Outcomeを記録
2. Root Task scopeを閉じる
3. 未完了Taskを検査
4. 必要ならCancellation
5. Callback受付を停止
6. Async operationを解決・破棄
7. Resource cleanup
8. Test double expectationを検査
9. Leak検査
10. Primary／suppressed failureを整理
11. 最終TestOutcomeを確定


Leak分類には次を含む。

- TaskLeak
- ResourceLeak
- CallbackLeak
- SharedTokenLeak
- CompletionTokenLeak
- BuilderSessionLeak
- NativeOperationLeak
- PendingExpectation

#### 39. TestOutcome

Test全体の結果を次へ分類する。

TestOutcome =
  Passed
  | Failed(TestFailureReport)
  | Skipped(SkipReason)
  | TimedOut(TimeoutReport)
  | Defected(DefectReport)
  | Aborted(InfrastructureAbort)

Passed

Assertion、Cleanup、Leak検査をすべて通過した。

Failed

Testは実行できたが期待条件を満たさなかった。

Skipped

必要条件がなく実行されなかった。

TimedOut

Virtual timeout、Execution budgetまたはWall-clock watchdogに達した。

Defected

契約違反やRuntime Defectが発生した。

Aborted

Test実行基盤が結果を信頼可能な形で確定できなかった。

#### 40. 対象処理のOutcome

対象Programの結果はTestOutcomeと分離する。

SubjectOutcome<A, E> =
  Success(A)
  | Failure(E)
  | Cancelled
  | Defect(DefectReport)


例として、期待された型付きFailureが発生した場合は次となる。

SubjectOutcome:
Failure(expected-error)

TestOutcome:
Passed


Assertion failureを対象Programのfailure Eへ混ぜない。

#### 41. TestFailure

TestFailureの基本分類を次とする。

TestFailureKind =
  ValueMismatch
  | PredicateNotSatisfied
  | UnexpectedSuccess
  | UnexpectedFailure
  | FailureMismatch
  | UnexpectedCancellation
  | MissingCancellation
  | InteractionMismatch
  | DiagnosticMismatch
  | SnapshotMismatch
  | VisualMismatch
  | TaskLeak
  | ResourceLeak
  | TimeoutExpectationMismatch
  | ConformanceMismatch

#### 42. Assertion

Assertionは通常packageで定義し、不一致を構造化Mismatchとしてtest Effectへ渡す。

基本Assertionには次を含む。

- 真偽条件
- Equality
- Inequality
- Ordering
- Pattern
- Collection
- Approximate equality
- Expected Failure
- Expected Cancellation
- Expected Diagnostic
- Interaction expectation


Assertionの失敗を通常FailureやDefectとして扱わない。

#### 43. EqualityとDiff

型ごとのEqualityとDiagnostic用Diffを区別する。

Equatable<A> {
  equal:
    A -> A -> bool
}

Diffable<A> {
  diff:
    A -> A -> Diff
}


すべての型に万能なpointer identity比較を提供しない。

次の型には専用比較を要求する。

- Function
- Task handle
- Resource handle
- Signal
- Native object
- GPU Resource
- Document
- 浮動小数点


Diff計算にはBudgetを持たせる。

DiffBudget {
  max-depth,
  max-items,
  max-bytes,
  max-computation
}

#### 44. 近似比較

浮動小数点、Geometry、Color、Pixel等には専用の近似比較を使用する。

Tolerance =
  Absolute
  | Relative
  | Combined
  | ULP
  | DomainSpecific


単一の既定Toleranceをすべての用途へ適用しない。

Geometry:
logical pointまたはdevice-space error

Color:
component差または知覚色差

Pixel:
位置差・色差・許容pixel数

#### 45. Expected Failure・Cancellation・Defect
Expected Failure

型付きFailureの型、constructor、payload patternを検査する。

Expected Cancellation

Cancellation要求だけでなく、TaskがCancelledとなりCleanupを完了したことを検査する。

Expected Defect

主としてTrusted Conformance Testへ限定する。

Expected Terminal failure

ProcessまたはSandbox isolationで実行し、親Runnerが終了種別、Diagnostic、Crash category等を検査する。

#### 46. Diagnostic比較

Diagnosticの表示文字列全体ではなく、構造化情報を比較する。

Diagnostic {
  code,
  severity,
  primary-origin,
  related-origins,
  category,
  arguments,
  notes,
  rendered-message
}


通常のDiagnostic Testでは次を比較する。

- Diagnostic code
- Severity
- Source origin
- 重要argument
- Related diagnostic


表示文全体の完全一致はDiagnostic renderer自身のTestへ限定する。

#### 47. Fail-fastと収集

Assertion制御を次へ分ける。

require:
失敗時に現在のTest sectionを中断

check:
失敗を記録して継続


前提条件にはrequireを使い、互いに独立した複数fieldの検査等にはcheckを使用できる。

複数Assertion failureは、集合として一つのTest failure reportへまとめる。

#### 48. Primaryとsuppressed failure

Test bodyのFailureとCleanup Failureが同時に発生した場合、次の規則を適用する。

Test bodyが既にFailure:
Test body FailureをPrimary

Cleanup／Leak:
suppressed


Test bodyが成功していた場合、CleanupまたはLeak FailureをPrimaryとする。

Runtimeの既存Primary／suppressed規則と整合させる。

#### 49. Snapshot Test

Snapshot Testは補助機能として提供する。

適切な対象：

- Diagnostic tree
- Normalized IR
- Stable JSON／XML
- Generated SVG
- Layout summary


不適切になりやすい対象：

- 生pointerを含むDebug出力
- 非決定的Map順序
- Timestamp入りArtifact
- 巨大Document全体
- 環境依存Font出力


Snapshot mismatch時に期待値を自動上書きしない。

Mismatch
↓
Actualを別Artifactとして保存
↓
差分確認
↓
明示的accept


CIでは自動acceptを禁止する。

#### 50. Visual Diff

Visual Diffは固定したBackend、Color profile、Quality policy、Font environmentで実行する。

比較方式は次を含む。

- ExactPixel
- PerChannelTolerance
- PerceptualDifference
- RegionMask
- StructuralVisualCheck


Visual snapshotを使用する前に、Bounds、Text内容、Layout tree等の構造的Assertionを優先する。

#### 51. Property-based testing

Property Testは通常Testを多数caseについて実行する上位形式とする。

Property<A> {
  generator,
  body,
  shrink-policy,
  case-budget,
  discard-budget,
  seed-policy
}


Property-based testingは、入力範囲について性質を記述し、Frameworkがedge caseを含む入力を生成する。通常の手書きTestを置き換えるものではなく、補完するものとして扱う。

#### 52. Generator

Generator<A>は、Choice streamを消費して有効なAを決定的に構築する。

同じGenerator identity
同じGenerator version
同じChoice trace
同じConfiguration
→ 同じ値


Generatorを単純な非決定的Random関数と同一視しない。

Generatorは型単位ではなく値領域単位を基本とする。

int generator
positive-length generator
valid-port generator
color-component generator


型ごとのCanonical Generatorは便利な既定として自動導出できるが、明示Generatorを常に許可する。

#### 53. Discard

Propertyの前提を満たさないcaseはDiscardとする。

Discard:
検査対象外

Failure:
有効caseがPropertyを破った


Discard数にはBudgetを設ける。

有効caseを十分に生成できない場合は、Property反例ではなくGeneration configuration failureとして報告する。

#### 54. Shrinking

Failureを維持したまま入力を簡略化する処理をShrinkingとする。

ShrinkingはGeneratorと整合した構造を基本とする。

Proptestは、生成とShrinkingを型単位ではなく値生成Strategy単位で定義し、制約を維持しながら反例を簡略化する。

Shrinkingは、定義された簡略化順序とBudget内で、より単純な反例を探索する。

数学的な大域最小反例の発見は保証しない。

#### 55. Choice traceとFailure persistence

Property Test失敗時には、Seedだけでなく次を保存する。

- Test identity
- Property version
- Generator identity
- Generator version
- Random algorithm version
- Run seed
- Case seed
- Choice trace
- Serialized counterexample
- Failure fingerprint
- Schedule trace
- Fault script


次回実行時には、保存済みRegression caseを新規Random caseより先に実行する。

Proptestも、発見済み失敗caseのSeedを永続化し、後続実行で再生するFailure persistenceを備える。

#### 56. Failure fingerprint

Shrinking中に別のFailureへ変化することを防ぐため、Failure identityを構造化する。

FailureFingerprint {
  outcome-kind,
  failure-code,
  assertion-id,
  source-origin-class,
  defect-category
}


既定では、同じFailure fingerprintを維持するShrink候補だけを採用する。

#### 57. Stateful Property Test

状態を持つ対象については、操作列とReference Modelを使う。

Reference Model
↓ operation sequence
Expected state

System Under Test
↓ same operations
Actual state


対象例：

- Resource lifetime
- Document editor
- Undo／Redo
- Task scope
- Callback registration
- Incremental compiler session


有効操作列と不正操作Testを分ける。

#### 58. Differential testing

同一入力を複数実装または処理経路へ与え、結果を比較する。

Input
├─ Implementation A
└─ Implementation B


対象例：

- Portable vs Native
- Interpreter vs Compiled
- Unoptimized vs Optimized
- Full vs Incremental compile
- Full-frame vs Tile render
- CPU vs GPU
- Backend version間


Differential testingは、異なる実装へ同一入力を与え、挙動の差からSemantic bug候補を検出する手法である。

不一致だけから、どちらの実装が誤っているかを自動断定しない。

#### 59. Differential equivalence

比較対象ごとに同値関係を明示する。

Equivalence =
  Exact
  | Structural
  | Normalized
  | NumericTolerance
  | PixelTolerance
  | Semantic
  | Observational


未規定・実装依存の値を比較対象へ含めない。

Raw result
↓
Observation projection
↓
Normalization
↓
Equivalence comparison

#### 60. Metamorphic testing

独立実装がない場合は、入力変換前後の意味保存関係を検査できる。

deserialize(serialize(x))
=
x

optimize(optimize(ir))
=
optimize(ir)

translate(path, 0, 0)
≡
path


Metamorphic TestはProperty Testの一形式とする。

#### 61. Schedule exploration

並行Taskの合法な実行順を複数探索する。

対象となるBranch pointには次を含む。

- Runnable Task選択
- yield
- await completion
- Timer順序
- Cancellation受理
- Native completion
- Callback delivery


Loomは並行実行の有効な順序を決定的に探索し、状態削減によって組合せ爆発を抑えるConcurrency Test toolである。Reciplexaでは、まずTask Runtime上の論理schedule探索の参考とする。

#### 62. 探索の完全性

Schedule探索結果を次へ分類する。

ExplorationStatus =
  ExhaustiveWithinModel
  | BoundedComplete
  | BudgetExhausted
  | CounterexampleFound


Budget内でFailureが見つからなかったことを、無条件な正しさの証明とはみなさない。

探索には次の上限を持つ。

- 最大schedule数
- 最大Task切替数
- 最大preemption数
- 最大Event数
- 最大探索深度

#### 63. 多次元反例

並行Property Testの反例は次の複合構造を持ち得る。

Counterexample {
  input,
  schedule,
  fault-script,
  time-choices,
  implementation-selection
}


入力、Schedule、Faultを最初から完全直積で探索しない。

推奨段階は次とする。

1. 多数入力を既定Scheduleで検査
2. 一部入力をRandomized Scheduleで検査
3. 重要入力をBounded exploration
4. 発見反例を入力Shrinking
5. Schedule Shrinking
6. Fault Script Shrinking

#### 64. Flaky Test

同じ反例、Schedule、Snapshotで結果が変わる場合は、Flakyまたは環境依存Testとして分類する。

- StableFailure
- IntermittentFailure
- TraceIncompatible
- EnvironmentDependent


Failure発見後に同一条件でReplayし、Failure fingerprintが安定していることを確認する。

#### 65. 性能Property

性能Testでは、Wall-clock時間だけでなく構造的costを優先する。

- Recompiled binding数
- Evaluated Node数
- Allocation量
- Shapingされたrun数
- Render tile数
- Cache hit率
- Scheduler step数


例：

一文字変更で、
全文書の全Paragraphを再Shapingしない


性能Propertyは機器差の影響を受けにくい規範的な上限または漸近的性質を優先する。

#### 66. Public Test API

通常packageが利用できるPublic Test APIには、次を含む。

- Assertion
- Equality／Diff
- Approximate comparison
- Expected Failure
- Expected Cancellation
- Fake／Stub／Spy／Mock
- Generator
- Shrinker
- Property Test
- Artifact Check
- Virtual Clock操作
- Test handler構成


Public Test APIからRuntime memory safetyやModule sealingを破ることはできない。

#### 67. Trusted Conformance API

次はTrusted Conformance APIへ限定する。

- Raw IRの不正構築
- Invalid handle
- 二重completion
- Rust panic／abort注入
- ABI mismatch
- Perceus refcount観測
- Scheduler内部状態
- Cleanup stack
- Terminal failure trigger


Trusted APIは、Toolchain、標準Runtime、正式に認可されたNative conformance suiteだけが利用できる。

通常の依存宣言だけで権限を取得できるものにはしない。

#### 68. 実装責任境界
通常Test package
- Assertion combinator
- Equality
- Diff
- Generator
- Shrinker
- Fake
- Spy
- Mock
- Domain固有Check

Compiler
- Test metadata
- Module-local private access
- External Contract Test環境
- Test-only declaration
- TestDescriptor
- 増分Test index
- Compile依存解析

Test Runtime
- Root Task scope
- 決定的Scheduler
- Virtual Clock
- Seeded Random
- Handler installation
- Leak検査
- Defect isolation

Runner／Host
- Test選択
- Process／Sandbox isolation
- Wall-clock watchdog
- Regression corpus
- Attachment
- Report
- CI分割

Trusted Conformance層
- Raw representation
- 内部観測
- 低水準Fault injection

#### 69. 増分BuildとTest再実行

Testの再Compileと再実行を分ける。

Compile dependency:
Test bodyの型・名前解決が変わるか

Execution dependency:
対象実装の挙動が変わるか

Test bodyだけ変更
再Compile:
該当Test

通常Artifact:
維持

Private実装変更
Module-local Test:
再Compile／再実行候補

External Contract Test:
本体の再Compileは通常不要
再実行は必要

Public interface変更
External Contract Test:
再型検査・再実行

依存Package:
再型検査候補


Changed-only Test selectionは開発時の高速化として提供する。

CI等では全Testを実行する経路を維持する。

#### 70. Test report

Test RuntimeとRunnerは構造化Reportを生成する。

Structured Test Report
↓
CLI
IDE
CI
HTML
Machine-readable output


安定情報は次とする。

- Test identity
- Outcome kind
- Failure code
- Source origin
- Structured diff
- Scheduler trace identity
- Counterexample identity
- Attachment identity


人間向けの表示文言を機械的interfaceにしない。

#### 71. 適合試験

少なくとも次を検証する。

- Test discoveryが明示metadataだけを対象とする
- 成果物bindingがTestとして発見されない
- Module-local Testがprivate memberへアクセスできる
- Private値がTest scope外へescapeしない
- External Contract Testからprivate memberが見えない
- Test-only code変更で公開API hashが変わらない
- Effect Test handlerがTest scopeへ隔離される
- 任意binding patchが行われない
- Fakeが本番signatureを満たす
- TestごとのScheduler／Clock／Randomが独立する
- 同じseedとTraceで同じ結果を再現する
- 未完了TaskとResource leakを検出する
- Assertion failureとProgram Failureを区別する
- Expected CancellationがCleanup完了まで確認する
- Property Testが反例を再現する
- ShrinkingがFailure fingerprintを維持する
- Regression corpusの反例を再実行する
- Differential mismatchを構造化報告する
- Schedule explorationのBudget超過を証明扱いしない
- Terminal failureがProcessへ隔離される

#### 72. 性能要件
PERF-TST-01:
Test discoveryのためにWorkspace全Sourceを毎回再parseしない

PERF-TST-02:
Test-only変更でProduction artifactを無効化しない

PERF-TST-03:
External Contract Testの再Compileと再実行を分離する

PERF-TST-04:
PureなTestをProcess起動なしで実行可能にする

PERF-TST-05:
必要なTestだけProcess／Sandboxへ隔離する

PERF-TST-06:
Property Testの各Caseで不要なCompiler artifactを再構築しない

PERF-TST-07:
Immutableな安全なArtifactをTest間で共有可能にする

PERF-TST-08:
Test-local mutable stateを共有しない

PERF-TST-09:
Trace記録量をProfileによって制御できる

PERF-TST-10:
Changed-only selectionと全Test実行の両経路を維持する

#### 73. 残る後続項目

##### `OPEN-TST-SURFACE-001`

- test宣言の具体構文
- require／checkの名称
- Parameterized Test構文
- Property Test構文
- Test metadata記法

##### `OPEN-TST-RUNNER-001`

- CLI
- Filter式
- Process pool
- CI shard
- Report format
- Regression corpus配置

##### `OPEN-TST-SNAPSHOT-001`

- Snapshot形式
- Version
- Normalization
- Accept workflow
- Visual Diff

##### `OPEN-TST-CONFORMANCE-001`

- Trusted API
- Fault injection
- Runtime内部観測
- Native Adapter適合試験

##### `OPEN-PKG-ENTRY-001`

- Application entry
- Runtime profile
- Root Task scope
- 起動と終了

#### 74. 最終状態

```text
OPEN-TST-001:
RESOLVED
```


本決定により、ReciplexaのTest機構は次の性質を持つ。

- 成果物制作へTest記述を強制しない
- 通常TestとArtifact Checkを分離する
- White-box TestとBlack-box Testを明確に区別する
- Private memberを公開せずにTestできる
- Test-only codeをProduction artifactから分離する
- Effect handlerとModule substitutionで依存を置換する
- 任意binding patchに依存しない
- Testごとに必要な強さの隔離を選べる
- Scheduler、Clock、Randomを決定的にする
- Task、Resource、CallbackのLeakを検出する
- Program Failure、Assertion failure、Cancellation、Defectを区別する
- Property TestとShrinkingを通常Testへ統合する
- Portable／Native、Full／Incremental等をDifferential Testできる
- 並行TaskのScheduleを再現・探索できる
- Test機能をpackage、Compiler、Runtime、Runnerへ適切に分担する
- Test-only変更を増分Buildで局所化する

### OPEN-PKG-ENTRY-001 Package Target・Entry Binding・Runtime Profile・Application Lifetime
#### DD-PKG-ENTRY-001 決定概要
##### 状態
Status:
RESOLVED

Scope:
- PackageとTargetの分離
- Targetの種類と依存関係
- Entry binding
- Artifact／Application／Backend／Test／Check Target
- Runtime Profile
- Effect Handler
- Capability
- Resource namespace
- Secret Provider
- Native package instance
- Backend Registry
- ExecutorとBudget
- Root Scope
- Application lifetime
- Shutdown
- Supervision
- ApplicationOutcome
- Process Exit
- Artifact BuildとAtomic commit
- Package Manifest
- Resolved Target Descriptor
- Build Request
- Versioning
- 増分Build

#### 0. 基本原則

Reciplexaでは、Packageそのものを直接起動・Buildする単位とはしない。

Packageは、配布、Version、依存解決、互換性を管理する単位とする。その内部に、Build、実行、出力、検査の単位であるTargetを複数定義できるものとする。

Package:
配布、Version、依存解決、互換性の単位

Target:
Build、実行、出力、検査の単位

Entry binding:
Targetの評価を開始するRPX binding


一つのPackageには、次のような複数の用途を共存させられる。

Package
├─ Library Target
├─ Artifact Target
├─ Application Target
├─ Backend Target
├─ Test Target
└─ Check Target


Targetごとに必要な依存、Effect、Capability、Runtime Profile、出力規則を分離する。

これにより、Testだけの変更で成果物を再Buildしたり、文書生成だけにGUIやGPU依存が混入したりすることを避ける。

#### 1. PackageとTarget

Packageは複数Targetを持つことができる。

Targetの基本分類は次とする。

TargetKind =
  Library
  | Artifact
  | Application
  | Backend
  | Test
  | Check


Targetは、Package内で一意のTarget IDを持つ。

TargetIdentity {
  PackageInstanceId,
  TargetId
}


Target IDは、Entry bindingのidentityおよび人間向けの表示名から分離する。

TargetId:
Target自体の安定したidentity

EntryBindingId:
Targetが現在使用するEntry

DisplayName:
人間向けの名称


Entry bindingや表示名を変更しても、Target自体を同一Targetとして追跡できる構造を許容する。

#### 2. Library Target

Library Targetは、他のPackageまたは同一Package内の別Targetから使用されるModule interfaceを提供する。

Library Targetは、Hostから直接起動するEntry bindingを必要としない。

LibraryTarget {
  exported-modules,
  public-interface,
  portable-implementation,
  native-implementations
}


Libraryの実行例やDemonstrationが必要な場合は、次を別Targetとして定義する。

- Application Target
- Artifact Target
- Documentation Test
- Test Target


Library本体へ便宜的なmainを要求しない。

Package内の複数Targetが共有するprivate Moduleは、Package外へ公開せずに共有できる。

#### 3. Artifact Target

Artifact Targetは、文書、スライド、図、Animationなどの具体的成果物の意味値を生成する。

ArtifactInput
↓
Artifact Entry
↓
Document／Presentation／Composition等
↓
Validation
↓
Backend Planning
↓
Emission
↓
Artifact


Artifact Targetの標準workflowは次とする。

Build
↓
Validate
↓
Preview
↓
Export


Artifact Targetへ通常のUnit Test作成を要求しない。

成果物本文を書いて実際に生成・Preview・Exportすることを、制作上の主要な検証手段とする。

ただし、型検査、IR Validation、Resource検査、Backend Validationは常時行う。

#### 4. Application Target

Application Targetは、継続的なInteractionまたはService lifetimeを持つProgramを表す。

ApplicationClass =
  Cli
  | Gui
  | Server
  | Worker
  | PluginHost
  | Custom


TargetKindはApplicationとし、CLI、GUI、Server等の区別はApplication Classとして別fieldに保持する。

これにより、Root Scope、Shutdown、Fault supervisionなどの共通規則を共有しつつ、起動入力とRequired EffectをApplication Classごとに定められる。

#### 5. Backend Target

Backend Targetは、Render IR等を特定の形式または実行環境へ変換するBackend implementationを提供する。

対象例は次のとおりとする。

- PDF
- SVG
- PPTX
- Raster
- GPU Preview
- Video


Backend Targetは通常のApplication Entryを持つのではなく、専用のBackend Provider契約を提供する。

BackendProvider {
  descriptor,
  capability-query,
  planning-entry,
  emission-entry,
  validation-entry
}


Backendの詳細なCapability Profileは、別途Backend仕様で定める。

#### 6. Test Target

Test Targetは、OPEN-TST-001で定めたTestDescriptor群とTest-only declarationを含む。

Test TargetはProduction Targetへ依存できる。

Test Target
→ Library／Artifact／Application Target


逆方向の依存は禁止する。

禁止:
Production Target
→ Test Target


これにより、Test-only codeやTest-only dependencyがProduction artifactへ混入することを防ぐ。

#### 7. Check Target

Check Targetは、Artifactまたは中間IRへ、利用者固有の要件を適用する。

CheckStage =
  Domain
  | Layout
  | Visual
  | Render
  | EmittedArtifact


例として次を検査できる。

- Layout overflowがない
- Link切れがない
- 指定Font以外を使用していない
- Accessibility要件を満たしている
- Raster fallbackが発生していない
- 出力Artifactが指定Profileへ適合する


Check TargetはArtifact Targetへ依存する。

Check Target
→ Artifact Target


Artifact TargetがCheck Targetへ依存することはない。

ReleaseやCIにおいてCheck成功を要求する場合は、Target dependencyではなくBuild policyまたはTarget groupで表現する。

#### 8. Target依存関係

Target間の依存関係はDAGとする。

Library
↓
Artifact
↓
Check


循環依存はBuild順序とLifecycleを不明確にするため、v1では禁止する。

Target依存とPackage依存を区別する。

Package dependency:
外部PackageのModule interfaceへの依存

Target dependency:
TargetのBuild結果や実行結果への依存


外部PackageのLibraryへは通常のPackage依存を使用する。

外部Artifactへ依存する場合は、Build artifact dependencyとして明示し、物理File名ではなく論理Artifact identityで参照する。

外部Application TargetをLibraryのようにimportすることは認めない。別Applicationの実行が必要な場合は、Process serviceまたはHost operationとして明示する。

#### 9. Targetごとの依存とVersion解決

Test、GUI、Backend等のTargetだけが必要とする追加依存を宣言できる。

PackageDependencies:
Package全体で解決される依存

TargetDependencies:
特定Targetが利用する依存


ただし、Targetごとに同一Package依存のVersionを独立解決することは原則として行わない。

Workspace／Package:
Versionを共通解決

Target:
解決済み依存の利用範囲だけを指定


Test Targetだけが別Versionを暗黙に使用する設計は避ける。

#### 10. Entry binding

Entry bindingは、通常の型付きRPX bindingである。

Entry専用の別言語や、固定された特殊関数を導入しない。

Entry binding:
通常のRPX binding

TargetDescriptor:
そのbindingをHostから呼び出すことを宣言


通常bindingが存在するだけではEntryにならない。

TargetDescriptorがEntry bindingを明示的に参照して初めて、そのTargetのEntryとなる。

固定名mainは便利な既定候補としてToolingが利用できるが、規範的なEntry identityにはしない。

#### 11. Entry bindingの参照

ManifestではEntry bindingをSymbolic referenceとして指定する。

BindingReference {
  module-path,
  binding-name,
  expected-binding-kind
}


Compilerによる名前解決後、Stable Binding IDへ変換する。

Manifest Binding Reference
↓
Name resolution
↓
BindingId


HostやBuild systemは、解決済みBinding IDを用いる。

Entry bindingは、最終的に一つのmonomorphicな実行可能instanceへ解決されなければならない。

- 型parameter解決済み
- Effect row解決済み
- Module parameter解決済み
- Implementation selection可能

#### 12. Artifact Entry

Artifact Entryの概念的な型は次とする。

ArtifactEntry<Input, Output> =
  Input -> Output
  effects {
    required-effects...
  }


出力型の例は次のとおりとする。

- Document
- Presentation
- Diagram
- Composition
- Image
- AudioComposition
- ArtifactBundle


Artifact Entryは、通常、PDFやPPTXのbytesを直接生成しない。

Artifact Entry:
成果物の意味値を生成

Build Host:
Backendを選択し、ArtifactをEmissionする


これにより、同じ本文をPreview、PDF、SVG、PPTX等に使用できる。

#### 13. Artifact Entryの入力

Artifact Entryには、型付き入力値を渡す。

ReportInput {
  reporting-period,
  language,
  source-data,
  theme
}


外部Configは、CodecによってEntry input型へdecodeする。

External Configuration
↓
Decode
↓
Typed Entry Input
↓
Validation
↓
Artifact Entry


入力decodeまたはValidationに失敗した場合、Entry bodyを実行しない。

入力を必要としないArtifactは、unit入力として正規化できる。

#### 14. Artifactの複数出力

一つのArtifact Targetが複数の論理成果物を生成する場合、型付きArtifact Bundleを返す。

ArtifactBundle {
  main,
  supplement,
  thumbnails,
  metadata
}


各成果物には安定した論理Member IDを与える。

ArtifactMemberId


論理Member IDと物理File名を分離する。

Artifact Member:
main

Output Mapping:
annual-report.pdf


Entry bodyへ物理File名を埋め込まない。

#### 15. Streaming Artifact

巨大な動画、音声、長大文書等については、Streaming Artifactを許容する。

Input
->
ArtifactProducer<Chunk>


ProducerはHost管理のEncoderまたはSinkへ構造化されたchunkを供給する。

生のFile handleへ任意に書き込む方式にはしない。

Streaming Producerは次を満たさなければならない。

- Cancellation可能
- Backpressure対応
- 順序契約が明確
- Resource lifetimeが明示的
- Failure時にrollback可能
- 部分出力を完成Artifactとして扱わない


v1ではMaterializedまたはLazy Artifactを基本とし、Streamingは必要なTargetへ限定する。

#### 16. Application Entry

Application Entryの概念的な型は次とする。

ApplicationEntry<Input, Exit> =
  Input -> Exit
  effects {
    task,
    required-application-effects...
  }


Entryへ巨大なRuntimeContextを直接渡さない。

通常の起動情報:
型付きEntry input

外部作用:
Effect

権限:
Runtime ProfileのCapability

終了管理:
Root Scope


これにより、Entryが不要なCapabilityへアクセスすることを防ぎ、Effect rowから依存関係を把握できる。

#### 17. Application Classごとの入力
CLI
CliInput {
  arguments,
  allowed-environment-view,
  working-directory-view,
  invocation-metadata
}


標準streamやFilesystemは、Resource EffectとCapabilityを通じて利用する。

GUI
GuiInput {
  launch-kind,
  initial-documents,
  restoration-state,
  invocation-metadata
}


Window、Input、GPUはEffectおよびCapabilityとして提供する。

Server
ServerInput {
  configuration,
  listener-descriptions,
  startup-state
}


実SocketやNetwork authorityはNetwork EffectとCapabilityとして提供する。

CLI、GUI、Serverのすべてを一つの万能入力recordへ統合しない。

#### 18. Configuration

Configurationを次の二種類に分ける。

Serializableで不変な設定:
Entry inputとして渡す

Dynamic service、Secret、Resource authority:
EffectとCapabilityとして提供する


Entry inputへ適するものの例：

- Theme
- Locale
- Feature selection
- Initial document list
- Output size


通常Configurationへ含めないものの例：

- Password
- API token
- GPU device
- Window handle
- Database connection
- Raw Native pointer

#### 19. Generic Entry

一般のRPX関数はgenericであってよい。

ただし、Hostが起動する正準Entry instanceでは、すべての型parameter、Effect parameter、Module parameterが解決済みでなければならない。

Generic component
↓
Target composition
↓
Specialization
↓
Monomorphic Entry Instance


Open Effect rowを残したままEntryとして起動することは認めない。

内部関数:
effects {resource | e}
を許容

正準Entry:
effects {task, resource, diagnostic}
のように閉じている必要がある

#### 20. Check Entry

Check Targetは、対象Stageの値を入力として受け取り、構造化されたCheck Reportを生成する。

CheckEntry<A> =
  A -> CheckReport
  effects {
    check,
    required-effects...
  }


Checkが対象Artifactを自分で再Buildするのではなく、Build graphから対象値またはArtifactを受け取る。

複数Findingを収集するため、Check専用Effectを利用できる。

#### 21. Runtime Profile

Runtime Profileは、Entryが要求するEffect、Capability、Resource、Native implementation等を具体化する、型付き実行契約である。

Runtime Profile =
  Effect Handler
  + Capability grants
  + Resource namespaces
  + Secret providers
  + Native instances
  + Backend registry
  + Executor set
  + Budgets
  + Fault／Shutdown policies


概念的な正規形は次とする。

RuntimeProfile {
  identity,
  profile-class,
  effect-bindings,
  capability-grants,
  resource-namespaces,
  secret-providers,
  native-instances,
  backend-registry,
  executor-set,
  budgets,
  diagnostic-policy,
  fault-policy,
  shutdown-policy,
  version
}

#### 22. Runtime Profile Class

Profile Classの基本分類は次とする。

RuntimeProfileClass =
  Cli
  | Gui
  | Server
  | Worker
  | Batch
  | Preview
  | Test
  | Sandbox
  | Custom


Profile Classは既定構成を選択するための分類であり、最終的なCapability集合そのものではない。

同じGUI Profile Classでも、EditorとViewerでは異なるCapabilityを持てる。

#### 23. Runtime Profileの解決

Runtime Profileは次の要求・供給・制約から解決する。

Target requirements
∩
Host capabilities
∩
User grants
∩
Workspace policy
∩
Execution restrictions
↓
Resolved Runtime Profile


TargetやPackageはCapabilityを要求できるが、自らgrantを作成することはできない。

最終的なCapability grantはHostまたはHostに認可された主体が行う。

#### 24. Profile解決Failure

Profile解決Failureを構造化する。

ProfileResolutionFailure =
  MissingEffectHandler
  | MissingCapability
  | VersionMismatch
  | ConflictingHandler
  | ExecutorUnavailable
  | NativeImplementationUnavailable
  | BackendUnavailable
  | BudgetUnsatisfiable
  | PolicyDenied
  | InvalidProfileComposition


Required EffectまたはRequired Capabilityを満たせない場合、Entryを開始しない。

#### 25. Effect Handler

EntryのEffect rowは、必要な作用を表す。

Runtime Profileは、それを処理する具体的Handlerを提供する。

Entry:
effects {
  task,
  resource,
  clock,
  render
}

Runtime Profile:
task     → Task Scheduler
resource → File／Package Resource Handler
clock    → System Clock
render   → GPU Preview Backend


HandlerはEffect serviceまたは一貫性domain単位で選択する。

次のような不整合な寄せ集めを既定では認めない。

clock.now:
System Clock

clock.sleep:
Virtual Clock


部分overrideを行う場合は、明示的なComposite Handlerとして構築し、整合性を検証する。

#### 26. Effect Handlerのidentity

次を区別する。

EffectIdentity
EffectContractVersion
HandlerImplementationIdentity
HandlerImplementationVersion


Handlerは、Required Effect contractへ適合する必要がある。

文字列名が一致するだけでは適合とみなさない。

#### 27. EffectとCapabilityの分離

EffectとCapabilityは関連するが別概念である。

Effect:
どの外部作用を要求するか

Capability:
どの対象に、どの範囲で作用できるか


例：

resource Effect:
Fileを開く操作を要求できる

Filesystem Capability:
特定Directory以下を読める


Entryがresource Effectを持つことは、任意Fileへのアクセス権を意味しない。

#### 28. Capability

Capabilityの概念的構造は次とする。

Capability {
  capability-identity,
  capability-class,
  authority-scope,
  allowed-operations,
  constraints,
  lifetime,
  owner-runtime,
  transfer-policy,
  audit-policy
}


Capability Classの例：

- FilesystemRead
- FilesystemWrite
- NetworkConnect
- WindowCreate
- GpuUse
- SecretRead
- ProcessSpawn
- BackendUse


Capabilityは通常のPath文字列やBoolean flagではない。

対象を知っていることと、その対象へ操作する権限を持つことを分離する。

#### 29. Capabilityの縮小

Capabilityは、より狭いCapabilityへ制限できる。

Directory ReadWrite
↓
Directory ReadOnly
↓
Specific File ReadOnly


親Capabilityより強いCapabilityへ拡大することはできない。

許可:
ReadWrite → ReadOnly

禁止:
ReadOnly → ReadWrite


Capability grantはHost authorityだけが行える。

Applicationや子Taskは、自分が保有するCapabilityを縮小して渡すことだけができる。

#### 30. CapabilityのLifetimeと移送

CapabilityのLifetimeは、Owner ScopeのLifetime以下でなければならない。

Capability lifetime
⊆
Owner Scope lifetime


Task間のCapability移送は、CapabilityのTransfer Policyへ従う。

- 継承可能
- Shared
- Exclusive
- Move only
- Non-transferable


子Taskは親Taskが持たないCapabilityを取得できない。

Exclusive Capabilityの例：

- Transactional output
- Single-owner Builder
- Exclusive GPU context


これらを暗黙に複製しない。

#### 31. Ambient authorityの禁止

Applicationが現在のRuntime Profile全体を列挙し、任意Capabilityを名前で取得できる万能APIは提供しない。

禁止:
current-runtime-profile()
get-any-capability(name)


Required CapabilityはHostが起動前に解決する。

Optional Capabilityだけを、型付きCapability queryで確認できる。

query-capability:
CapabilityRequirement
-> CapabilityAvailability

#### 32. Required・Optional・Alternative Capability
CapabilityNecessity =
  Required
  | Optional
  | Alternative

Required

なければTargetを起動できない。

Optional

存在すれば追加機能を利用できる。

Alternative

候補のいずれかを必要とする。

GPU Renderer
or
CPU Renderer


FallbackはRuntime Profile Planningで解決し、意味や品質の変化をDiagnosticへ記録する。

#### 33. Resource Namespace

Resourceの名前空間を分離する。

ResourceNamespace =
  Package
  | Document
  | UserSelected
  | Temporary
  | Cache
  | HostProvided


同じPath文字列でも、Namespaceが異なれば別Resourceである。

Package:"images/logo.png"

Document:"images/logo.png"


を混同しない。

#### 34. Package Resource

Packageに同梱された不変Resourceは、Package Resource Namespaceへ所属する。

対象例：

- Font
- Image
- Template
- Locale data


原則としてRead-onlyであり、Content-addressed identityを持つ。

#### 35. Document Resource

Document Resourceは、特定DocumentまたはWorkspaceへ所属する。

DocumentごとにCapability viewを分離できる構造を推奨する。

Application Profile
├─ Document A Profile View
└─ Document B Profile View


Document AのTaskがDocument BのResource capabilityを暗黙に持たないようにする。

#### 36. User-selected Resource

利用者がFile picker等で選択したResourceについて、そのResourceへ限定されたCapabilityを発行できる。

Applicationへ任意Filesystem capabilityを与える代わりに、利用者が選択した対象への局所Capabilityだけを与える。

#### 37. Temporary ResourceとCache

Temporary ResourceはScope終了時にcleanupする。

Cache Resourceは再生成可能でなければならず、削除によってProgramの規範結果が変化してはならない。

Temporary:
Lifecycle管理対象

Cache:
Eviction可能

#### 38. Secret Provider

Secret値を通常Config、Manifest、Build cacheへ保存しない。

SecretRequirement {
  secret-id,
  purpose,
  necessity,
  access-mode,
  lifetime
}


実際のSecret値はRuntime ProfileのSecret Providerから取得する。

Secret値は次へ出力しない。

- Diagnostic
- Test snapshot
- Cache key
- Manifest
- Scheduler trace
- Build metadata


TestではProduction Secret Providerではなく、Test用Providerを使用する。

#### 39. Native Package Instance

Native packageの状態は、可能な限りRuntime instance、Device、Worker process、Document session等の明示単位へ所属させる。

Process-global mutable singletonを既定にしない。

NativeInstance {
  package-instance-id,
  implementation-id,
  abi-version,
  instance-state,
  executor-affinity,
  capabilities,
  shutdown-contract
}


Profile構築時にDescriptor、ABI、Version、Capability、Executorを検査した後でinstanceを作成する。

#### 40. Backend Registry

Runtime Profileは、利用可能なBackend implementationを登録する。

BackendRegistry {
  backends,
  profiles,
  capability-snapshots,
  selection-policy
}


ApplicationがRegistry内部を自由に変更することはできない。

TargetまたはJobがBackend requirementを提出し、Plannerが適合するBackendを選択する。

一つのJob中はBackend selectionを固定する。

#### 41. Executor Set

Runtime Profileは利用可能なExecutorを持つ。

ExecutorSet {
  default,
  main?,
  cpu-pool?,
  io?,
  gpu?,
  dedicated-executors
}


各Executorは次を宣言する。

- Executor identity
- Executor class
- Thread／Process affinity
- Blocking policy
- Capacity
- Shutdown behavior


Entryの初期ExecutorはTargetDescriptorで指定できる。

EntryExecutor =
  Default
  | Main
  | Dedicated(ExecutorClass)


Entry全体を一Executorへ固定するものではない。Taskごとに必要なExecutorへ移動できる。

#### 42. Runtime Budget

Runtime ProfileはResource Budgetを持つ。

RuntimeBudgets {
  memory,
  cpu,
  task-count,
  open-resources,
  native-operations,
  gpu-memory,
  worker-processes,
  cache,
  output-size,
  diagnostic-size
}


Hard LimitとSoft Limitを区別する。

HardLimit:
超過を許可しない

SoftLimit:
Cache eviction、backpressure、品質調整等を試みる


Budget超過は原則として型付きFailureとする。

Runtime bookkeepingの破損による超過はDefectである。

#### 43. Runtime Profileの合成

Profileは複数のProfile fragmentから構築できる。

Base GUI Profile
+
Filesystem fragment
+
GPU Backend fragment
+
Restricted Network fragment


合成規則を次に分類する。

- Disjoint
- Compatible
- Restrictive
- Conflict


後から宣言されたHandlerが黙って優先される形式にはしない。

Effect HandlerやCapability requirementが競合する場合、明示的な解決を要求する。

#### 44. Profile Snapshot

Rendering、Export、長時間Job等の開始時に、Runtime Profile Snapshotを作成する。

RuntimeProfileSnapshot {
  profile-identity,
  profile-revision,
  handler-bindings,
  capability-grants,
  resource-revisions,
  native-instance-revisions,
  backend-capabilities,
  executor-identities,
  budget-slice
}


Job実行途中で、別Backend、別Font環境、別Native実装へ暗黙に切り替えない。

Capability revocationはSnapshotより優先する。

Revoked Capabilityを使用しようとしたJobは、Authorization failureまたはCancellationで停止する。

#### 45. JobごとのProfile縮小

JobはApplication Root Profile全体を受け取らない。

例としてPDF Export Jobには次だけを与える。

- Document Resource read
- Font Resource read
- PDF Backend
- Output write
- CPU executor


Window、Network、他Document書込み等の不要Capabilityを与えない。

Application Profile
↓ restrict
Export Job Profile Snapshot


子Taskも縮小済みProfile viewだけを継承する。

#### 46. Root Scope

一回のApplicationまたはArtifact BuildのLifetimeを、Root Scopeで管理する。

Root Scope
├─ Entry Task
├─ Child Task scopes
├─ Service Tasks
├─ Resource scopes
├─ Callback registrations
├─ Native operations
├─ Runtime Profile Snapshot
├─ Diagnostic context
├─ Fault boundary
└─ Shutdown controller


Entry TaskはRoot Scopeの最初のTaskである。

ApplicationのLifetimeは、Entry Task単体ではなくRoot Scope全体で決まる。

#### 47. Root Scopeの状態
RootScopeState =
  Preparing
  | Running
  | Quiescing
  | Cancelling
  | Finalizing
  | Completed
  | Aborted

Preparing

Profile、Input、Native instance等を準備する。

Running

通常のTask、Resource、Operationを実行する。

Quiescing

正常終了へ向け、新規の通常処理を停止し、進行中処理を完了させる。

Cancelling

Grace periodを超えたTask等へCancellationを要求する。

Finalizing

Resource、Backend、Native instance、Diagnostic等を最終化する。

Completed

正常または構造化された異常結果を確定した。

Aborted

Terminal failure等により通常Finalizationを完了できなかった。

#### 48. Entryの返却とShutdown

Entry Taskが戻っても、即座にProcessを終了しない。

Entry returns
↓
Root Scope enters Quiescing
↓
Child Taskを待つ
↓
必要ならCancellation
↓
Cleanup
↓
ApplicationOutcome確定


Entryが返った瞬間に子Taskを破棄することも、子Taskを無期限に放置することも認めない。

#### 49. 正常終了要求とCancellation

正常な終了要求とTask Cancellationを区別する。

正常な終了要求:
Application-levelの終了意思

Cancellation:
未完了Taskを途中終了させる制御


通常の終了経路は次とする。

Shutdown request
↓
Quiescing
↓
自然終了を待つ
↓ 未完了なら
Cancelling


利用者の通常の終了操作を、最初から異常Cancellationとして扱わない。

#### 50. Shutdown Request
ShutdownReason =
  EntryCompleted
  | UserRequested
  | HostRequested
  | ParentCancelled
  | OperatingSystemRequested
  | UnhandledFailure
  | Defect
  | ResourceExhaustion
  | ProfileRevoked


複数のShutdown Requestが発生した場合、最初の要求を記録しつつ、後から発生したより重大な原因へPrimary reasonを昇格できる。

#### 51. Shutdown中のTask作成

Taskの目的を分類する。

TaskPurpose =
  NormalWork
  | ShutdownWork
  | CleanupWork


状態ごとの許可は次とする。

Running:
すべて許可

Quiescing:
NormalWorkを禁止
ShutdownWork／CleanupWorkを許可

Cancelling:
原則CleanupWorkだけを許可

Finalizing:
Runtime認可済みCleanupWorkだけを許可


Shutdown中に通常処理を増やし続けることを防ぐ。

#### 52. Service Task

GUI event loop、Autosave、File watcher、Server listener等をService TaskとしてRoot Scope内で管理する。

ServiceShutdownContract {
  begin-quiesce,
  cancel,
  finalize,
  deadline-policy
}


Application外の暗黙global Taskとして動作させない。

#### 53. Detached Task

v1では、Application Root Scopeから完全に独立する一般的なDetached Taskを禁止する。

長寿命処理は次へ所属させる。

- Root Scope内のService Task
- Host-managed service scope
- 別Application Target
- Worker process


Host-managed scopeへTaskを移す場合は、明示的なOwnership transferとCapabilityを要求する。

#### 54. Grace periodとForced Shutdown

Shutdownを次の段階に分ける。

1. Graceful／Quiescing phase
2. Cancellation phase
3. Finalization
4. 必要ならHostによるForced termination


Grace periodには、Virtual Durationだけでなく、Wall Duration、Scheduler step、Pending operation数等のBudgetを使用できる。

GracePolicy {
  virtual-duration?,
  wall-duration?,
  scheduler-step-budget?,
  pending-operation-budget?
}


Cancellation後もCleanupのための猶予を与える。

Terminal failureやNative hangに対しては、ProcessまたはSandbox境界をHostが停止する。

#### 55. Fault Policy
FaultPolicy =
  FailFast
  | Supervised
  | Collect

FailFast

未処理Child FailureでRoot Scopeを終了する。

Supervised

Supervisorが再起動、隔離、無効化、RootへのEscalationを判断する。

Collect

複数JobのOutcomeを収集し、最後にまとめる。

DefectやTerminal failureを通常Failureとして収集するものではない。

#### 56. Supervisor
SupervisorPolicy {
  restart-policy,
  restart-budget,
  backoff,
  failure-classification,
  child-shutdown-policy,
  escalation-policy
}


Restart Policy：

Never
OnFailure
OnSelectedFailure
Always


無限再起動を防ぐため、回数、時間窓、累積cost等のBudgetを持つ。

Budget超過時は上位SupervisorまたはRoot ScopeへFailureを昇格する。

#### 57. 未観測Task Failure

Task handleが明示的にawaitされなかった場合でも、Root ScopeはTask Outcomeを把握する。

未観測Failureを黙って破棄しない。

UnhandledChildFailure
↓
Fault Policy


Task handleのdropによってFailureまで消える規則にはしない。

#### 58. Resource Cleanup

Task、Service、Resource、Native instanceの終了依存関係をDAGとして管理できる。

Service A depends on Service B


Shutdownは逆Topological orderで行う。

Aを停止
↓
Bを停止


依存cycleはProfile Validation failureとする。

#### 59. Cleanup Failure

Cleanup中にもFailureが起こり得る。

- File flush failure
- Backend finalize failure
- Native shutdown failure
- Temporary Resource cleanup failure


既にPrimary Failureがある場合、Cleanup Failureをsuppressedとして保持する。

Entryが成功していた場合は、Cleanup FailureがPrimaryとなり得る。

複数の独立Cleanup Failureは集合として保持できる。

#### 60. DefectとTerminal Failure

Defectの影響範囲を分類する。

DefectScope =
  Task
  | Service
  | RuntimeInstance
  | Process
  | Unknown


Task-local Defectは、Runtime全体の健全性が確認できる場合に限って隔離できる。

Runtime instanceの整合性が疑わしい場合、そのRoot Scopeを終了する。

Memory safety、Native ABI、Runtime bookkeepingの健全性が不明な場合はTerminal failureとする。

Terminal failureを通常Failureへ降格しない。

#### 61. ApplicationOutcome

Root Scope終了後の正準Outcomeを次とする。

ApplicationOutcome<A, E> =
  Completed(A)
  | Requested(ExitIntent)
  | Failed(E)
  | Cancelled(CancellationReport)
  | Defected(DefectReport)
  | Aborted(InfrastructureAbort)

Completed

EntryとRoot Scopeが正常終了した。

Requested

正常な終了意図により終了した。

Failed

型付きProgram Failureにより終了した。

Cancelled

親Host等からのCancellationで終了した。

Defected

隔離可能なDefectを構造化して報告した。

Aborted

Terminal failure、Watchdog、Worker通信断等により通常結果を確定できなかった。

#### 62. ExitIntent
ExitIntent {
  category,
  user-message?,
  restart-hint?,
  data?
}


Categoryの例：

- Success
- NoChanges
- UserCancelledOperation
- RestartRequested
- UpdateRequested
- Custom


ExitIntentはOSの整数Exit codeではない。

GUI Host内の一Application instanceだけを閉じる場合にも利用できる。

#### 63. Process Exit

Standalone CLIやApplicationでは、ApplicationOutcomeをProcess Exit Statusへ変換する。

ApplicationOutcome
↓
ExitMappingPolicy
↓
ProcessExitStatus


具体的な数値はPlatformおよびApplication policyへ委ねる。

次の変換は禁止する。

Defect → Success
Terminal Failure → Success


通常Failureを成功として扱いたい場合は、Application logic内で意味のある正常結果へ変換する。

#### 64. Restart

Application自身がRoot Scopeを直接再利用してRestartしない。

Old Root Scope
↓ Shutdown
Requested(Restart)
↓
Hostが新Profileを解決
↓
New Root Scope


旧Task、旧Resource、旧Capabilityを新Root Scopeへ暗黙に持ち越さない。

必要状態は、型付きかつPortableなRestoration Stateとして明示的に受け渡す。

#### 65. Artifact Build Root

Artifact BuildもRoot Scopeで管理する。

Artifact Build Root
├─ Entry evaluation
├─ Resource loading
├─ Layout
├─ Backend planning
├─ Emission
└─ Output transaction


正常経路：

Artifact意味値生成
↓
子Task完了
↓
Validation
↓
Backend emission
↓
Artifact validation
↓
Atomic commit
↓
Completed


FailureまたはCancellation時にはOutput transactionをrollbackする。

#### 66. Output Transaction
OutputTransactionState =
  Preparing
  | Writing
  | Finalizing
  | Committed
  | RolledBack


完成前の出力を正式Artifactとして公開しない。

Failure時には次を行う。

- Temporary file削除
- Partial upload破棄
- Manifest rollback
- Lock解除


Streaming Artifactでは、Producer、Encoder finalize、Artifact validation、Sink commitがすべて成功して初めてCompletedとなる。

#### 67. Manifestの基本方針

Package Manifestは、静的な宣言dataとする。

Manifest自体を任意のRPX Programとして評価する方式は採用しない。

Manifestは次の性質を持つ。

- 実行せずに解釈できる
- 決定的である
- IDEが読める
- Cross compilation前に検査できる
- Security reviewが可能
- Versioningできる


ManifestへProgram logic、File読込み、Network access、任意関数を入れない。

#### 68. Manifest・Descriptor・Planの三層
Manifest Declaration
↓ Compiler／Resolver
Resolved Target Descriptor
↓ Host／Build Request
Execution／Build Plan

Manifest Declaration

利用者が記述する静的宣言。

Resolved Target Descriptor

ManifestとSourceを型検査・解決した正規形。

Execution／Build Plan

具体的なHandler、Capability、Backend、Executor、Budgetを割り当てた実行計画。

#### 69. ManifestとCompiler推論の境界

CompilerがSourceから導出する情報：

- Entry input型
- Entry output型
- Required Effect
- Binding identity
- Module依存
- Generic解決状態
- Portable codecの有無


Manifestが宣言する情報：

- Target ID
- Target kind
- Entry binding
- Application Class
- Profile preset
- Capability requirement
- Target dependency
- Output preset
- Default Target


HostまたはBuild Requestが与える情報：

- 実Capability
- Secret値
- 実際のOutput先
- GPU device
- Memory budget
- Resource Snapshot


Compilerが推論できる型やEffectを、Manifestから意味的に上書きすることは認めない。

Manifestは追加制約を課すことはできる。

#### 70. Resolved Target Descriptor

概念的な正規形は次とする。

ResolvedTargetDescriptor {
  target-identity,
  display-metadata,
  target-kind,

  entry-binding-identity?,
  exported-modules?,

  input-type?,
  output-type?,
  required-effects,

  target-dependencies,
  package-dependencies,
  artifact-dependencies,

  profile-requirement,
  capability-requirements,
  executor-requirement,
  implementation-policy,
  isolation-requirement,
  reentrancy-policy,

  shutdown-policy?,
  output-contract?,
  check-contract?,

  provenance,
  descriptor-version
}


TargetKindに応じて不要なfieldは存在しないものとして扱う。

利用者へ万能recordを直接書かせず、TargetKindごとのManifest schemaから正規形へloweringする。

#### 71. Capability Requirement
CapabilityRequirement {
  requirement-id,
  capability-class,
  necessity,
  authority-shape,
  operation-set,
  constraints,
  purpose,
  fallback-group?
}


Manifestでは実際のCapability tokenや絶対Pathを保存しない。

必要なAuthorityの形とPurposeだけを宣言する。

- UserSelectedFileRead
- UserSelectedOutputWrite
- DocumentDirectoryRead
- SpecificServiceConnect
- GpuRendering


Permission UIやSecurity reviewは静的Purpose metadataを利用できる。

#### 72. Secret Requirement
SecretRequirement {
  secret-id,
  purpose,
  necessity,
  access-mode,
  lifetime
}


ManifestにはSecret requirementだけを記録する。

Secret値はManifest、Lockfile、Build Requestのportable部分、Cache keyへ保存しない。

#### 73. Build Request
BuildRequest {
  target-identity,
  input,
  build-configuration,
  runtime-profile-request,
  output-request,
  validation-policy,
  check-policy,
  cache-policy,
  execution-policy
}


ManifestはTargetの静的定義であり、Build Requestは一回の具体的なBuildまたは起動要求である。

#### 74. Build Requestの正規化
External Request
↓
Decode
↓
Typed Request
↓
Validation
↓
Normalized Build Request
↓
Fingerprint


同じ意味のRequestは、可能な限り同じFingerprintを生成する。

Capability token、Secret値、Native pointer等をFingerprintへ含めない。

結果へ影響するResource identity、Profile selection、Implementation version等を含める。

#### 75. Output Request
OutputRequest {
  artifact-member,
  output-format,
  output-profile,
  destination-capability,
  naming-policy,
  overwrite-policy,
  commit-policy
}


Output先をPath文字列だけで指定しない。

Destination Capabilityと組み合わせる。

Manifestへ利用者環境固有の絶対Pathを保存しない。

#### 76. Overwrite Policy
OverwritePolicy =
  FailIfExists
  | ReplaceAtomically
  | CreateNewVersion
  | HostPrompt


Interactive HostではHostPromptを利用できる。

Batch／CIでは決定的なPolicyを使用する。

Entry bodyが直接確認Dialogを出してOutput overwriteを解決する方式は避ける。

#### 77. Default Target

Package内にTargetが一つだけなら、自動選択できる。

複数Targetがある場合は、ManifestでDefault Targetを指定できる。

DefaultTargetId


複数Targetが存在し、Defaultがない場合は、利用者またはToolingへ選択を要求する。

宣言順や名前順で恣意的に選択しない。

#### 78. Target Configuration

同じEntry bindingと依存構造を使い、入力値やOutput Profileだけが異なる場合は、Targetを複製するのではなくBuild Configurationとして扱う。

Target:
annual-report

Configuration A:
Japanese PDF

Configuration B:
English PDF

Configuration C:
Editable PPTX


Entry bindingまたは依存構造が意味的に異なる場合は、別Targetとする。

#### 79. Manifest Versioning

Manifest schemaはPackage Versionとは独立したVersionを持つ。

ManifestVersion


次を区別する。

- Backward-compatible extension
- Migration-required change
- Unsupported future version


未知の必須fieldを無視して続行しない。

Namespace付きOptional Extensionは、規則に従って警告付きで無視できる場合がある。

#### 80. Resolved Descriptor Version

Compilerが生成するResolved Target Descriptorも独立Versionを持つ。

TargetDescriptorVersion


Build system、IDE、Runner、Runtime launcherは適合Versionを検査する。

未知VersionをMemory layoutの推測で読み込まない。

#### 81. Manifest Extension

第三者Tooling向けmetadataはNamespace付きExtensionとして保存できる。

extensions {
  namespace -> versioned-data
}


ExtensionがTargetの実行意味を密かに変更してはならない。

Execution semanticsを変更する情報は、規範Manifest schemaへ含める必要がある。

#### 82. .rpi・Test・Native metadataとの分離

論理schemaを次のように分離する。

.rpi:
Public type／Module／Effect interface

Target Metadata:
Resolved Target Descriptor

Test Metadata:
TestDescriptor index

Native Metadata:
Native implementation descriptor

Backend Metadata:
Backend Capability descriptor


これらを一つのPackage artifact containerへ格納することはできるが、意味上は分離する。

#### 83. Incremental Build

Target単位で次のHashを分ける。

TargetInterfaceHash
TargetImplementationHash
TargetConfigurationHash
TargetDependencyHash
TargetDisplayHash


表示名変更だけでCompiled codeやArtifactを無効化しない。

Capability requirement変更では、Entry codeを再利用しつつProfile ResolutionとLaunch Planだけを無効化できる。

Output mapping変更では、Artifact意味値を再利用し、EmissionとCommitだけを再実行できる。

#### 84. Target変更時の無効化
Test-only変更
Test Target:
再Build

Production Target:
維持

Artifact本文変更
Artifact Target:
再評価

依存Check:
再実行

無関係Target:
維持

Entry型変更
Target Descriptor:
再Validation

Host integration:
再Build候補

Required Effect変更
Runtime Profile:
再適合検査

Launch Plan:
無効化

Backend Profile変更
Artifact意味値:
再利用可能

Backend Planning／Emission:
再実行

#### 85. Manifest Validation

少なくとも次を検査する。

- Target IDの重複
- Default Targetの存在
- Entry bindingの存在
- Entry型の適合
- 未解決generic parameter
- Open Effect row
- Target dependency cycle
- ProductionからTestへの依存
- Capability requirementの矛盾
- Profile presetの適合
- Executor availability
- Reentrancy制約
- Artifact output型
- Secret requirementの妥当性
- Manifest Version


Validated ManifestとResolved DescriptorだけをBuild Planningへ渡す。

#### 86. Diagnostic

ManifestおよびEntry関連のDiagnosticを構造化する。

- ManifestParseFailure
- ManifestSchemaFailure
- TargetResolutionFailure
- EntryTypeMismatch
- DependencyCycle
- CapabilityRequirementConflict
- ProfileResolutionFailure
- OutputMappingFailure
- VersionMismatch


Diagnosticには次を含める。

- Stable diagnostic code
- Manifest Source span
- 関連Entry bindingのSource origin
- 推論された型・Effect
- 修正候補

#### 87. Security review

ManifestおよびResolved Descriptorから、実行前に次を一覧化できるようにする。

- Filesystem authority要求
- Network接続要求
- Secret要求
- Native implementation
- Backend requirement
- Process spawn
- Output write
- GPU利用


PackageはCapabilityを要求できるが、Manifestを書くだけでgrantを取得することはできない。

Hostまたは利用者のAuthorizationを必要とする。

#### 88. 適合試験

少なくとも次を検査する。

- Packageが複数Targetを持てる
- Library TargetがEntryを要求しない
- Target identityとEntry identityが分離される
- Target依存cycleを拒否する
- ProductionからTestへの依存を拒否する
- Artifact Entryの戻り型を正しく認識する
- Application Classと入力型の不一致を拒否する
- Open Effect rowを持つEntryを拒否する
- Profile不足時にEntryを開始しない
- Capability grantなしでResourceへアクセスできない
- Capability縮小によってAuthorityが増えない
- 子Taskが親にないCapabilityを取得できない
- Scope終了後のCapability使用を拒否する
- SecretがDiagnosticやSnapshotへ出ない
- Entry返却後にRoot ScopeのCleanupを行う
- Quiescing後のNormalWorkを拒否する
- 未観測Task FailureをOutcomeへ反映する
- Detached Task escapeを拒否する
- Artifact失敗時にOutputをcommitしない
- Manifestの推論値上書きを拒否する
- Output先にCapabilityを要求する
- Test-only変更でProduction Targetを無効化しない
- Display metadata変更でCompiled artifactを無効化しない

#### 89. 性能要件
PERF-PKG-ENTRY-01:
Target単位でBuild・Validation・Cacheを分離する

PERF-PKG-ENTRY-02:
Test-only変更でProduction Targetを再Buildしない

PERF-PKG-ENTRY-03:
Output Profile変更でArtifact意味値を再生成しない

PERF-PKG-ENTRY-04:
Profile変更時に無関係なCompiler artifactを維持する

PERF-PKG-ENTRY-05:
Manifest表示情報変更でSemantic Target Hashを変えない

PERF-PKG-ENTRY-06:
長大ArtifactでStreamingおよびPage／Frame単位処理を許可する

PERF-PKG-ENTRY-07:
Jobごとに必要CapabilityとBudgetだけを切り出す

PERF-PKG-ENTRY-08:
Runtime Profile全体の変更で無関係Cacheを捨てない

PERF-PKG-ENTRY-09:
Root Scope終了時に未管理Taskを残さない

PERF-PKG-ENTRY-10:
Same Target＋Same SnapshotのBuildを再利用可能にする

#### 90. 実装責任境界
Package Manifest／Package Manager
- Package identity
- Target declaration
- Package dependency
- Target dependency
- Default Target
- Profile／Output preset

Compiler
- Entry binding解決
- Entry型・Effect検査
- Resolved Target Descriptor
- Public interface
- Target dependency summary

Build System
- Build Request
- Target graph
- Incremental invalidation
- Artifact dependency
- Output planning
- Atomic commit

Runtime Launcher
- Runtime Profile解決
- Capability grant
- Root Scope作成
- Entry起動
- Shutdown
- ApplicationOutcome

Host
- User authorization
- Secret Provider
- Process／Sandbox
- OS signal
- Output destination
- Process Exit mapping

#### 91. 後続項目

##### `OPEN-PKG-MANIFEST-SURFACE-001`

- Manifestの具体構文
- Target宣言構文
- Capability requirement記法
- Entry binding参照記法
- Output preset記法

##### `OPEN-PKG-ENTRY-SURFACE-001`

- Entry annotation
- Artifact Entryの簡略構文
- Application Entryの簡略構文
- Typed Configurationの記述

##### `OPEN-BACKEND-PROFILE-001`

- Backend Capability Profile
- Output Profile
- Backend selection
- Backend conformance

##### `OPEN-PKG-HOT-RELOAD-001`

- Warm rebuild
- Application state migration
- Runtime Profile更新
- Hot reload可能なbinding

#### 92. 最終状態

```text
OPEN-PKG-ENTRY-001:
RESOLVED
```


本決定により、ReciplexaのPackage実行境界は次の性質を持つ。

- PackageとBuild／実行Targetを分離する
- 一Packageに複数Targetを持てる
- Libraryへ不要なEntryを要求しない
- Artifact本文とBackendを分離する
- Entryを通常の型付きRPX bindingとして扱う
- 万能RuntimeContextをEntryへ渡さない
- 外部作用をEffect、権限をCapabilityとして分離する
- Runtime Profileを型付き実行契約として扱う
- JobごとにCapabilityとBudgetを縮小する
- Task、Resource、Native operationをRoot Scopeで管理する
- 正常終了とCancellationを区別する
- Detached Taskを原則禁止する
- Cleanup完了後にApplicationOutcomeを確定する
- ArtifactをAtomic commitする
- Manifest Declaration、Resolved Descriptor、Execution Planを分離する
- Build Requestを型付きかつ再現可能にする
- Target単位で増分BuildとCacheを行う

### OPEN-BACKEND-PROFILE-001 Backend Capability・Output Profile・Planning・情報損失・Artifact検証
#### DD-BACKEND-PROFILE-001 決定概要
##### 状態
Status:
RESOLVED

Scope:
- Backend Capability Schema
- Capability Snapshot
- Output Profile
- Hard ConstraintとPreference
- ApproximationとTolerance
- Text、Accessibility、Raster、Font、Color Policy
- Backend Planning Algorithm
- Backend Planning IR
- Raster Island
- Semantic Mapping
- Resource Planning
- Output Decision
- 情報損失
- 明示承認
- Planning Failure
- Alternative Plan
- Artifact Verification
- 増分Planning
- Backend Conformance

#### 0. 基本原則

Reciplexaでは、Render IRをBackend Emitterへ直接渡し、Emission中にBackendが場当たり的なFallbackを選択する構造を採用しない。

出力処理は、次の段階に分離する。

Render IR
+
Document Semantics
+
Backend Capability Snapshot
+
Output Profile Snapshot
+
Resource Snapshot
↓
Backend Planning
↓
Validated Backend Planning IR
↓
Backend Emission
↓
Emitted Artifact
↓
Artifact Verification


各段階の責任は次のとおりとする。

Render IR:
Backend非依存の規範的な視覚意味

Backend Capability:
Backendが技術的に実現できること

Output Profile:
今回の出力で保存すべき性質と、
許される変換・損失

Backend Planning:
各NodeまたはSubtreeの具体的な表現方法

Backend Emission:
確定済みPlanに従ったArtifact生成

Artifact Verification:
実際の出力がPlanどおりであることの検査


Backendは、Output Profileに含まれない近似、Raster化、省略、Font代替等を独自判断で行ってはならない。

#### 1. Backend Capability

Backend Capabilityは、Backendが特定の機能をどの方法で表現できるかを示す構造化情報である。

単純な対応／非対応のBooleanにはしない。

CapabilitySupport =
  Native
  | EquivalentLowering
  | Approximate
  | RasterFallback
  | Unsupported


Capabilityには、適用条件、制限値、保存される性質、失われる性質、Cost、適合試験情報を含める。

#### 2. Native

Nativeは、対象BackendまたはFormatが機能を直接表現できる状態を示す。

例：

SVG Path
PDF Vector Path
PPTX Text Box
GPU Texture


Nativeであっても、すべての意味を完全に保持するとは限らない。

たとえばNative Textであっても、Viewer側のFont環境によって再配置される可能性がある。

したがって、Native Capabilityにも次を記録する。

NativeSupport {
  conditions,
  preserved-properties,
  constrained-properties,
  limits,
  cost,
  conformance-reference
}

#### 3. Equivalent Lowering

EquivalentLoweringは、別表現への変換によって、指定された観測意味を保存できる状態を示す。

例：

Rounded Rectangle
→ Path

Arc
→ Cubic Bézier

Stroke
→ Filled Outline


何に関して等価であるかを明示する。

EquivalenceDomain =
  Visual
  | Geometric
  | Textual
  | Temporal
  | Semantic
  | Accessibility


視覚上は等価でも、高水準Objectとしての編集可能性を失う場合には、その損失を別途記録する。

#### 4. Approximate

Approximateは、完全な意味保存はできないものの、規定Tolerance内で近似できる状態を示す。

例：

Complex Gradient
→ 有限個の領域へ近似

Curve
→ Polylineへ近似

Unsupported Color Space
→ 利用可能なColor Spaceへ変換


Approximationには、必ず次を要求する。

ApproximationSupport {
  conditions,
  error-model,
  error-bound,
  quality-range,
  losses,
  cost,
  deterministic
}


誤差上限を評価できない近似は、Strict Profileでは使用できない。

BestEffort Profileでは未知誤差として候補化できるが、明示的なWarningを必要とする。

#### 5. Raster Fallback

RasterFallbackは、NodeまたはSubtreeをRaster imageへ変換することで、主として視覚的な結果を保持する方式である。

Complex Filter Group
→ Raster Island

Unsupported Blend Group
→ Raster Image

Text Effect
→ Rasterized Text


Raster化では、次が失われる可能性がある。

- Text editability
- Searchability
- Semantic Text
- Accessibility
- Vector scalability
- Object separation
- Animation structure
- Interactivity


Raster fallbackを単なる「対応可能」として扱わず、情報損失を伴うPlanning Decisionとして扱う。

#### 6. Unsupported

Unsupportedは、現在のBackend CapabilityとOutput Profileの組合せでは、有効な表現経路が存在しない状態である。

Unsupported {
  reason,
  failed-requirements,
  attempted-representations,
  possible-alternatives
}


Unsupportedであることは、必ずしも直ちにBuild全体の失敗を意味しない。

Output Profileに応じて、次の処理候補を検討できる。

- 別Backend
- 近似
- Raster fallback
- Property省略
- 複数Artifactへの分割
- Build failure

#### 7. Capability Domain

Backend Capabilityの主要Domainを次とする。

BackendCapabilities {
  document,
  geometry,
  paint,
  text,
  image,
  compositing,
  filter,
  animation,
  interaction,
  accessibility,
  color,
  resource,
  emission
}

#### 8. Document Capability

Document Capabilityには次を含む。

- Page
- Slide
- Layer
- Group
- Master structure
- Section
- Metadata
- Reading order
- Hyperlink
- Annotation
- Notes


たとえばPPTXではSlideやMasterをNativeに保持できる一方、SVGでは複数PageをArtifact BundleへLoweringする必要がある。

#### 9. Geometry Capability

Geometry Capabilityには次を含む。

- Path
- Primitive Shape
- Cubic Curve
- Arc
- Transform
- Clip
- Mask
- Stroke
- Dash
- Fill Rule
- Boolean Path
- Coordinate Range


Capabilityは、機能名だけでなくParameter範囲や制限値を持つ。

Path segment数上限
座標精度
Transform範囲
Clip階層上限

#### 10. Paint Capability

Paint Capabilityには次を含む。

- Solid Color
- Linear Gradient
- Radial Gradient
- Conic Gradient
- Mesh Gradient
- Pattern
- Image Paint
- Opacity
- Blend Mode


Gradientでは、次の条件もCapabilityに含める。

- Stop数
- Spread Mode
- Color Interpolation Space
- Transform
- Alpha
- Precision

#### 11. Text Capability

Text Capabilityには次を含む。

- Semantic Text
- Positioned Glyph Run
- Font Embedding
- Font Subsetting
- Font Substitution
- Variable Font
- Color Font
- Vertical Writing
- Bidirectional Text
- Text on Path
- Per-glyph Position
- Per-glyph Transform
- Selection
- Searchability
- Editability


Textは、視覚と意味を分離して評価する。

Visual fidelity
Semantic Text
Searchability
Editability
Accessibility
Reading order


Glyph outline化は、視覚上は等価でも、意味Textや編集可能性を失う可能性がある。

#### 12. Image Capability

Image Capabilityには次を含む。

- Raster Image
- Vector Image
- Alpha
- Crop
- Transform
- Resampling
- Color Profile
- Bit Depth
- Codec
- Tiling
- Large Image Streaming


対応Codec、最大Dimension、最大Pixel数、HDR、Alpha形式等を制限値として保持する。

#### 13. Compositing Capability

Compositing Capabilityには次を含む。

- Group Opacity
- Isolation
- Knockout
- Blend Mode
- Mask
- Backdrop
- Offscreen Group


Compositing Capabilityは、Raster Islandの境界判断へ直接影響する。

#### 14. Filter Capability

Filter Capabilityには次を含む。

- Blur
- Shadow
- Color Matrix
- Displacement
- Morphology
- Convolution
- Lighting
- Custom Filter


各Filter単体の対応だけでなく、Filter chain全体の処理順、Color Space、中間Surfaceの扱いも検査する。

#### 15. Animation Capability

Animation Capabilityには次を含む。

- Timeline
- Keyframe
- Interpolation
- Easing
- Transform Animation
- Paint Animation
- Path Animation
- Text Animation
- Filter Animation
- Nested Time
- Loop
- Event Trigger


静的Backendについては、Output Profileで指定された時刻に評価するFlattening候補を生成できる。

Backendが独自判断で最初のFrameだけを出力してはならない。

#### 16. Interaction Capability

Interaction Capabilityには次を含む。

- Hyperlink
- Navigation
- Button
- Form Field
- Hover
- Input Event
- Script
- Media Control


Backendが技術的にScriptを埋め込めても、Output ProfileまたはSecurity Policyが禁止する場合がある。

#### 17. Accessibility Capability

Accessibility Capabilityには次を含む。

- Semantic Role
- Reading Order
- Alt Text
- Language
- Heading Level
- Table Structure
- Caption
- Form Label
- Decorative Marking


Accessibilityは視覚的な品質とは独立した軸とする。

Raster化で見た目を保持できても、Accessibilityを失う場合は独立したLossとして記録する。

#### 18. Color Capability

Color Capabilityには次を含む。

- RGB
- CMYK
- Gray
- Spot Color
- ICC Profile
- Wide Gamut
- HDR
- Alpha Model
- Rendering Intent
- Color Management


Color変換のError model、Gamut mapping、Profile埋込みをCapabilityへ含める。

#### 19. Resource Capability

Resource Capabilityには次を含む。

- Font Embedding
- Image Embedding
- External Reference
- Resource Deduplication
- Streaming
- Incremental Emission
- Compression


外部Resource参照が技術的に可能でも、Output ProfileがStandalone Artifactを要求する場合には使用しない。

#### 20. Emission Capability

Emission Capabilityには次を含む。

- Streaming Output
- Seek Requirement
- Incremental Update
- Atomic Finalization
- Deterministic Output
- Compression
- Encryption
- Signing


Backendがseek可能なSinkを必要とする場合、HostはTemporary seekable Artifactを経由するPlanningを選択できる。

#### 21. 条件付きCapability

CapabilityはFeature parameterによって変化する。

例：

Blur radius <= 100:
Native

Blur radius > 100:
RasterFallback

Gradient stops <= 16:
Native

Gradient stops > 16:
ApproximateまたはRasterFallback


条件付きCapabilityを次の構造で表す。

CapabilityRule {
  feature-identity,
  parameter-pattern,
  conditions,
  support-kind,
  preservation,
  losses,
  limits,
  cost,
  deterministic,
  conformance-reference
}


条件は、Capability SnapshotとFeature parameterだけから決定されるpureな条件とする。

Network、現在時刻、非固定Global stateへ依存させない。

#### 22. Capability Snapshot

Backend Capabilityは、Backend Version、Format Version、Device、Driver、Resource制限等によって変化する。

Planning開始時に、Capability Snapshotを固定する。

BackendCapabilitySnapshot {
  backend-identity,
  backend-version,
  format-version,
  device-identity?,
  driver-identity?,
  feature-rules,
  limits,
  snapshot-revision
}


Job中にCapabilityが変化した場合、古いPlanの意味を変更せず、Jobを停止して必要なら新しいSnapshotで再Planningする。

#### 23. Preservation Axis

変換による保存性を、単一の品質Scoreへまとめない。

PreservationAxes {
  visual-fidelity,
  geometric-precision,
  semantic-text,
  searchability,
  editability,
  accessibility,
  interactivity,
  temporal-behavior,
  color-fidelity,
  resource-fidelity
}


各軸を次で評価する。

PreservationLevel =
  Exact
  | Equivalent
  | Approximate
  | Lost
  | NotApplicable


Hard Constraintに違反するLossを、別軸の高得点で相殺してはならない。

#### 24. Output Profile

Output Profileは、今回の出力で何を必ず保存し、どの変換を許可し、複数候補の中で何を優先するかを表す。

OutputProfile =
  Hard Constraints
  + Permitted Transformations
  + Preferences
  + Tolerances
  + Failure Policy
  + Reporting Policy


概念的な正規形は次とする。

OutputProfile {
  identity,
  profile-version,

  hard-constraints,
  permitted-transformations,
  preferences,
  tolerances,

  text-policy,
  accessibility-policy,
  vector-policy,
  raster-policy,
  animation-policy,
  interaction-policy,
  color-policy,
  font-policy,
  resource-policy,

  performance-policy,
  artifact-size-policy,

  failure-policy,
  reporting-policy
}

#### 25. Hard Constraint

Hard Constraintは、満たさない候補を無条件に除外する。

HardConstraint {
  property,
  required-level,
  scope,
  violation-policy
}


例：

- Semantic Textを保持する
- Reading Orderを保持する
- Raster fallbackを禁止する
- Spot Colorを保持する
- Animation flatteningを禁止する
- 外部Resource参照を禁止する


Hard ConstraintをPreferenceやArtifact Sizeで相殺してはならない。

#### 26. Constraint Scope

Hard ConstraintはArtifact全体だけでなく、特定のSemantic RoleやResource Classへ適用できる。

ConstraintScope =
  WholeArtifact
  | Page
  | Slide
  | Layer
  | Subtree
  | SemanticRole
  | ResourceClass
  | NodeSelection


通常は、Source Node IDの大量列挙より、Semantic RoleやResource Classによる指定を優先する。

例：

本文Text:
Semantic Text必須

装飾Text:
Outline化を許可

Logo:
Vector必須

背景Effect:
Raster化を許可

#### 27. Preference

Preferenceは、Hard Constraintを満たす複数候補の選択順を定める。

Preference {
  objective,
  direction,
  priority-class,
  scope
}

PreferencePriority =
  Critical
  | High
  | Normal
  | Low


例：

High:
Semantic Textを最大化

Normal:
Object Editabilityを最大化

Low:
Artifact Sizeを最小化


低PriorityのPreferenceが高PriorityのPreferenceを上回らないよう、辞書式に比較する。

#### 28. Approximation Tolerance

Approximationを許可する場合、Error modelと上限を明示する。

ApproximationTolerance {
  domain,
  metric,
  maximum-error,
  aggregation-policy,
  scope
}


例：

Geometry:
最大位置誤差

Color:
知覚色差

Raster:
許容外Pixel割合

Animation:
時刻誤差と位置誤差


Toleranceを超える候補はHard Constraintと同様に除外する。

#### 29. Editability Policy

編集可能性を複数軸へ分ける。

EditabilityAxes {
  object-separation,
  text-editability,
  shape-editability,
  style-editability,
  animation-editability,
  group-structure,
  master-structure,
  semantic-binding
}


Levelを次とする。

EditabilityLevel =
  NativeHighLevel
  | NativePrimitive
  | Decomposed
  | FlattenedVector
  | Rasterized
  | Lost


PPTX等で見た目を一枚画像として完全に保持しても、Object編集性はRasterizedとなる。

#### 30. Text Policy
SemanticTextPolicy {
  preserve-character-sequence,
  preserve-language,
  preserve-reading-order,
  preserve-searchability,
  preserve-selection,
  preserve-editability,
  allow-glyph-outline,
  allow-rasterization
}


Text表現を次へ分類する。

1. Semantic Native Text
2. Semantic Text＋固定Glyph配置
3. Positioned Glyph Run
4. Glyph Outline＋Semantic Overlay
5. Raster Text＋Semantic Overlay
6. Glyph Outlineのみ
7. Rasterのみ


AccessibleまたはSearchable Profileでは、Semantic Textまたは検証可能なSemantic Overlayを要求する。

#### 31. Accessibility Policy
AccessibilityPolicy {
  required,
  reading-order,
  semantic-roles,
  headings,
  alt-text,
  language,
  table-structure,
  form-labels,
  decorative-marking,
  fallback-policy
}


Accessible Profileでは、次をHard Constraintの基本候補とする。

- Reading Order保持
- Semantic Role保持
- Language保持
- Alt Text保持
- Table Structure保持
- Heading Level保持


Accessibility要件を満たせない場合、Accessible Strict Profileでは出力Failureとする。

#### 32. Searchability Policy
SearchabilityPolicy {
  unicode-mapping-required,
  logical-order-required,
  ligature-mapping-required,
  normalization-policy
}


文字内容が内部に残っているだけでSearchableとはみなさない。

実ArtifactからUnicode Textを正しいLogical orderで抽出できることを検証可能にする。

#### 33. Vector Policy
VectorPolicy {
  required-scope,
  allow-outline-conversion,
  allow-vector-decomposition,
  allow-raster-islands,
  maximum-rasterized-area,
  maximum-raster-resolution
}


Vector保持と編集性、Artifact Size、印刷品質を同一視しない。

非常に複雑なVectorはRasterより大きくなる可能性があるため、各軸を個別に評価する。

#### 34. Raster Policy
RasterPolicy {
  mode,
  allowed-scopes,
  forbidden-semantic-roles,
  resolution-policy,
  color-policy,
  alpha-policy,
  maximum-area,
  maximum-pixel-count,
  text-handling,
  accessibility-handling
}


Modeを次とする。

RasterMode =
  Forbid
  | AllowDecorativeOnly
  | AllowSelectedSubtrees
  | AllowWithSemanticOverlay
  | AllowAnywhere
  | PreferRaster


Raster化する範囲を可能な限り局所化する。

#### 35. Raster Island

必要なSubtreeだけをRaster化する局所領域をRaster Islandとする。

Page
├─ Native Text
├─ Native Vector
└─ Raster Island
   └─ Unsupported Filter Group


Raster Island境界は、次を考慮して決定する。

- Bounds
- Clip
- Mask
- Blend
- Backdrop
- Isolation
- Color Space
- Alpha
- Text含有
- Accessibility Role
- Resolution


Backdrop依存等によって局所Raster化できない場合、親GroupまたはPageへ範囲を拡大する。

拡大理由をOutput Decision Reportへ記録する。

#### 36. Raster Resolution
RasterResolutionPolicy =
  FixedDpi
  | DevicePixelRatio
  | MaximumGeometricError
  | Adaptive
  | BackendNative


PreviewではDevice Pixel Ratio、PrintではDPI、Geometry重視では最大誤差等を使用できる。

低解像度Preview RasterをFinal Outputへ流用しない。

#### 37. Animation Policy
AnimationPolicy {
  mode,
  frame-time?,
  sampling-policy?,
  preserve-timeline,
  preserve-interactivity,
  loop-policy,
  easing-tolerance
}


Modeを次とする。

AnimationOutputMode =
  Preserve
  | FlattenAtTime
  | SampleFrames
  | ConvertToVideo
  | Remove
  | ForbidLoss


静的BackendがAnimation非対応であるという理由だけで、暗黙に最初のFrameを選択してはならない。

Flatteningには明示的なFrame timeまたはSampling policyを要求する。

#### 38. Interaction Policy
InteractivityPolicy {
  hyperlinks,
  navigation,
  forms,
  media-controls,
  scripts,
  hover,
  input-events,
  unsupported-action-policy
}


Security上禁止されているScript等は、Backendが対応していても使用しない。

Interactive要素を静的表示へ変換する場合は、FlatteningまたはOmissionとして報告する。

#### 39. Font Policy
FontPolicy {
  embedding,
  subsetting,
  substitution,
  outlining,
  fallback,
  variable-font,
  color-font,
  licensing,
  missing-font-policy
}


Font embedding Policy：

Require
Prefer
Allow
Forbid


Font substitution Policy：

Forbid
AllowMetricCompatible
AllowDeclaredFallback
AllowAnyWithWarning


Glyph Outline Policy：

Forbid
DecorativeOnly
AllowWithSemanticOverlay
Allow
Prefer


Font substitutionによってShapingやLayoutが変わる場合は、Backend Planning中に黙って置換せず、上流Layoutの再実行を要求する。

#### 40. Color Policy
ColorPolicy {
  target-color-space,
  preserve-spot-colors,
  preserve-icc-profile,
  rendering-intent,
  gamut-mapping,
  alpha-flattening,
  hdr-policy,
  color-tolerance
}


Screen、Print、Archive等で異なるProfileを使用できる。

Spot Color保持がHard Constraintの場合、RGB変換候補を除外する。

#### 41. Resource Policy
ResourcePolicy {
  embedding,
  external-reference,
  deduplication,
  compression,
  provenance,
  missing-resource,
  remote-resource,
  snapshot-requirement
}


Final Artifactでは、外部ResourceのSnapshot固定や埋込みを要求できる。

PreviewではProxy Resourceや外部参照を許可できる。

#### 42. Artifact Size Policy
ArtifactSizePolicy {
  hard-maximum?,
  preferred-maximum?,
  compression-preference,
  image-downsampling,
  font-subsetting,
  deduplication,
  oversized-policy
}


Hard Maximumを超える候補は除外する。

Preferred MaximumはPreferenceとして扱う。

Artifact Sizeを小さくするために、Accessibility等のHard Constraintを破ってはならない。

#### 43. Performance Policy
PerformancePolicy {
  planning-budget,
  emission-budget,
  memory-budget,
  latency-preference,
  streaming-preference,
  parallelism-policy
}


PreviewではLatency、FinalではFidelityを優先できる。

Wall-clock値だけでなく、次のような構造的Costを利用する。

- Raster pixel数
- Path segment数
- Temporary memory
- Backend object数
- Embedded Resource量

#### 44. Failure Policy

Output Failure Modeを次とする。

OutputFailureMode =
  Strict
  | Compatible
  | BestEffort

Strict

Hard Constraintまたは許可されていないLossがあればFailureとする。

Compatible

Hard Constraintを維持し、明示的に許可されたLowering、Approximation、Raster fallbackを使用できる。

BestEffort

可能な範囲で出力を継続する。

ただし次を緩和しない。

- Memory safety
- Artifact structural validity
- Security policy
- 明示的な禁止事項


Strict／Compatible／BestEffortは品質ProfileではなくFailure Policyである。

#### 45. Output Profileの既定Preset
Preview Profile
Failure:
BestEffort

Hard:
Security
Artifact構造
Memory Budget

Preferences:
低Latency
Progressive Quality
GPU
局所Raster fallback

Final Fidelity Profile
Failure:
CompatibleまたはStrict

Hard:
Visual Tolerance
Resource固定
Color Profile
Artifact Validation

Preferences:
Visual Fidelity
Vector保持
高解像度

Editable Profile
Hard:
Text Editability
Object Separation
必要なStructure

Raster:
原則禁止

Preferences:
Native high-level object
Native Text
Group structure

Accessible Profile
Failure:
Strict

Hard:
Semantic Text
Reading Order
Language
Alt Text
Heading／Table structure


TextのOutline化またはRaster化は、検証可能なSemantic Overlayがある場合を除いて原則禁止する。

#### 46. Output Profileの合成

Profileは無制限な継承階層ではなく、名前付きFragmentの合成によって構築する。

Strict Failure
+
Accessible Text
+
Editable Objects
+
PPTX Constraints
↓
Resolved Output Profile


合成規則：

Hard Constraint:
Intersection

Allowed Transformation:
Intersection

Tolerance:
より厳しい上限

Preference:
Priority付き統合

Security Prohibition:
常に優先

Failure Policy:
より厳しいPolicyまたは明示Conflict


矛盾したProfile fragmentを黙って上書きしない。

#### 47. Output Profile Snapshot

Resolved Output ProfileはJob開始時にSnapshot化する。

OutputProfileSnapshot {
  profile-identity,
  profile-version,
  resolved-constraints,
  resolved-transformations,
  resolved-preferences,
  resolved-tolerances,
  reporting-policy,
  snapshot-hash
}


Job中にProfile設定が変わった場合、新しいPlanning Jobを開始する。

#### 48. Backend Planning

Backend Planningは、Render IRから特定Backend向けの表現方法を確定する段階である。

Render IR
+
Document Semantics
+
Capability Snapshot
+
Output Profile Snapshot
+
Resource Snapshot
↓
Backend Planner
↓
Backend Planning IR


Planningの基本段階を次とする。

1. 入力Validation
2. Requirement抽出
3. Node Candidate生成
4. Subtree Candidate生成
5. Hard Constraint適用
6. Tolerance検査
7. Resource Planning
8. Preference比較
9. 全体整合性調整
10. Planning IR生成
11. Output Decision Report生成
12. Planning IR Validation

#### 49. Planning Candidate
PlanningCandidate {
  candidate-id,
  source-nodes,
  representation-kind,
  capability-answer,
  preserved-properties,
  losses,
  error-bound,
  required-resources,
  cost,
  constraints,
  child-requirements
}


Representation Kind：

RepresentationKind =
  Native
  | Lowered
  | Approximated
  | Rasterized
  | Omitted
  | SplitRepresentation

#### 50. Candidate生成

候補は次の順に段階的に生成する。

1. Native
2. Equivalent Lowering
3. Profileが許せばApproximation
4. Profileが許せば局所Raster
5. 必要ならRaster範囲を親へ拡大
6. Profileが許せばOmission


Hard Constraintを満たす最初の候補を即採用せず、少数の有力候補を比較する。

Loweringによって新たなFeature Requirementが発生する場合、再帰的に解決する。

Lowering cycleは禁止する。

#### 51. Node PlanningとSubtree Planning

単純なShape、Text、Image等はNode単位でPlanningできる。

Blend、Backdrop、Mask、Isolation、Filter chain等はSubtree単位で計画する。

SubtreeCandidate {
  root,
  covered-nodes,
  compositing-context,
  external-dependencies,
  representation,
  bounds,
  semantic-projection
}


局所Planningで解決できない場合、親SubtreeへPlanning範囲を広げる。

#### 52. Text Planning

Textについては、次の候補経路を検討する。

1. Semantic Native Text
2. Semantic Text＋固定Glyph位置
3. Positioned Glyph Run
4. Glyph Outline＋Semantic Overlay
5. Raster Text＋Semantic Overlay
6. Glyph Outlineのみ
7. Rasterのみ


Output ProfileのText、Searchability、Editability、Accessibility Policyによって候補を除外する。

#### 53. Semantic Mapping

Visual表現とSemantic表現を分ける場合、その対応をPlanning IRへ保存する。

SemanticMapping {
  semantic-node-id,
  visual-object-ids,
  text-range?,
  reading-order,
  role,
  language,
  alt-text
}


Artifact Verificationで、実際のArtifactに対応関係が保持されていることを検査する。

#### 54. Animation Planning

Animationには次の候補を生成できる。

- Native Timeline
- Backend EasingへのLowering
- Sampled Keyframes
- FlattenAtTime
- Frame Sequence
- Video Conversion


Output ProfileがPreserveを要求する場合、Flatten候補を除外する。

FlattenAtTimeでは明示時刻を必要とする。

#### 55. Color Planning

Planning時に次を確定する。

- Source Color Space
- Target Color Space
- Color Transform
- ICC Profile
- Rendering Intent
- Alpha Model
- HDR／SDR
- Spot Color


Gamut clippingやApproximationがある場合、Error informationとLossを記録する。

#### 56. Resource Planning

Resourceごとに次を決める。

- Embed
- Subset
- Deduplicate
- External Reference
- Transcode
- Rasterize
- Stream


概念的な構造：

BackendResourcePlan {
  resource-plan-id,
  source-resource-id,
  representation,
  encoding,
  embedding-policy,
  output-members,
  dependencies
}


Font subsetは実際に使用するGlyph集合から構築する。

Image再encodeによるLossを記録する。

#### 57. Candidate除外と選択

Plannerは次の順に候補を処理する。

1. Hard Constraint違反を除外
2. Tolerance違反を除外
3. 許可されていないTransformationを除外
4. Resource／Budget違反を除外
5. Preference Priority順に比較
6. Costで比較
7. Stable Candidate IDでtie-break


Thread scheduling、Hash table順、並列完了順によってPlanが変わってはならない。

#### 58. Planning Budget
PlanningBudget {
  max-candidates-per-node,
  max-subtree-expansions,
  max-global-alternatives,
  max-raster-boundary-search,
  max-resource-plans,
  max-planning-steps,
  max-memory
}


結果を次に分類する。

BackendPlanningResult =
  Planned
  | Unsupported
  | BudgetExceeded
  | Defected


BudgetExceededをBackend非対応と同一視しない。

#### 59. Backend Planning IR
BackendPlan {
  plan-identity,
  source-ir-identity,
  backend-capability-snapshot-id,
  output-profile-snapshot-id,
  resource-snapshot-id,

  document-plan,
  object-plans,
  resource-plans,
  semantic-mappings,
  output-members,
  output-dependencies,
  decision-report,
  verification-requirements,
  validation-summary
}


各Object Plan：

BackendObjectPlan {
  plan-node-id,
  source-node-ids,
  representation-kind,
  backend-feature,
  children,
  transform,
  clip,
  compositing,
  resource-references,
  semantic-mapping?,
  preservation,
  losses,
  error-bound?,
  provenance
}

#### 60. Backend固有Extension
BackendPlanExtension {
  backend-namespace,
  schema-version,
  data
}


Backend固有のOperator、Object、Encoding等をExtensionへ保持できる。

Hard Constraint、Loss、Approximation、Semantic MappingをExtension内部へ隠してはならない。

#### 61. Planning IRの不変条件

Validated Backend Planning IRは次を満たす。

- 全Source Nodeが処理済みまたは明示Omission
- Hard Constraint違反がない
- ApproximationにError情報がある
- Raster IslandにBoundsとResolutionがある
- Resource referenceが解決済み
- Semantic Mappingが一貫する
- Output Memberが一意
- Emission順序を構築可能
- Dependency graphにCycleがない
- Backend Extension schemaが適合する


EmitterはValidated Planning IRだけを受け取る。

#### 62. Emitterの責務

Emitterは、Planning IRをBackend形式へ忠実に変換する。

- Backend Object生成
- Resource埋込み
- Output Transaction
- Format Validation
- Finalization


Emitterは次を行ってはならない。

- 新しいFallbackを独自選択
- Propertyを黙って省略
- Fontを黙って代替
- Textを黙ってRaster化
- Output Profileを緩和


Planning前提が崩れた場合はPlanInvalidatedとして停止し、必要なら新Snapshotで再Planningする。

#### 63. Output Decision Report

Planning結果の上位ReportをOutput Decision Reportとする。

すべてのDecisionがLossを伴うわけではないため、Loss Reportだけを上位概念にはしない。

OutputDecisionKind =
  NativeEmission
  | EquivalentLowering
  | Approximation
  | RasterFallback
  | Substitution
  | Flattening
  | Omission
  | SplitRepresentation
  | Unsupported

#### 64. Output Decision
OutputDecision {
  decision-id,
  source-origins,
  source-node-ids,
  semantic-node-ids,

  decision-kind,
  selected-representation,
  backend-feature,

  preserved-properties,
  changed-properties,
  losses,
  approximation?,
  substitutions,

  alternatives,
  selection-reason,
  profile-rules,
  capability-rules,

  severity,
  approval-state,
  verification-requirements
}


利用者が、何を、なぜ、どの方法で変換したかを追跡できるようにする。

#### 65. Output Loss

一つのDecisionは複数Lossを持てる。

OutputLossKind =
  VisualFidelity
  | GeometricPrecision
  | ColorFidelity
  | SemanticText
  | Searchability
  | Editability
  | Accessibility
  | Interactivity
  | TemporalBehavior
  | VectorRepresentation
  | StructuralHierarchy
  | ResourceIdentity
  | Metadata
  | Portability


程度を次で表す。

LossExtent =
  None
  | Partial
  | Complete
  | Unknown

#### 66. SeverityとDisposition
LossSeverity =
  Informational
  | Notice
  | Warning
  | Error
  | Critical

LossDisposition =
  AllowSilently
  | Report
  | Warn
  | RequireExplicitApproval
  | Fail


Severityは一般的な影響を表し、Dispositionは今回のOutput Profileでの扱いを表す。

#### 67. Source Origin

DecisionとLossは、元のRPX Sourceまで追跡可能にする。

OutputSourceOrigin {
  package-instance,
  module-id,
  binding-id?,
  source-range?,
  generated-origin?,
  macro-expansion-chain?,
  semantic-node-id?,
  render-node-id?
}


Provenance chain：

Artifact Object
↓
Backend Planning Node
↓
Render IR Node
↓
Layout／Visual Node
↓
RPX Binding／Source


IDEから元Sourceへ移動できるようにする。

#### 68. Loss Fingerprint

承認とCI差分に、安定したLoss Fingerprintを使用する。

LossFingerprint {
  target-identity,
  semantic-origin,
  decision-kind,
  loss-kinds,
  backend-contract,
  output-profile-rule,
  relevant-parameters
}


物理行番号、一時Object ID、Process ID等には依存させない。

#### 69. 明示承認

RequireExplicitApprovalとなったDecisionは、EmissionまたはCommit前に承認を必要とする。

Planning
↓
ApprovalRequired
↓
明示承認
↓
Emission


承認Scope：

ApprovalScope =
  SingleDecision
  | SameFingerprint
  | SourceSubtree
  | ArtifactMember
  | WholeBuild


既定はSingleDecisionまたはSameFingerprintとする。

Whole Buildの包括承認は、新規Lossまで承認する危険があるため慎重に扱う。

#### 70. CI承認

CIでは既知LossのBaselineを利用できる。

既知Loss:
継続可能

新規Loss:
Failure

消滅したLoss:
改善としてReport


Baselineを自動更新しない。

明示的なReviewによって更新する。

#### 71. Loss集約

多数の同種Lossがある場合、個別Decisionを保持したまま表示を集約する。

AggregatedLoss {
  aggregation-key,
  loss-kind,
  cause,
  count,
  affected-scopes,
  representative-origins,
  decision-references
}


Machine-readable reportでは個別Decisionを保持する。

#### 72. Planning Failure
PlanningFailureKind =
  MissingCapability
  | HardConstraintUnsatisfied
  | ApproximationOutsideTolerance
  | RasterizationForbidden
  | SemanticPreservationImpossible
  | AccessibilityPreservationImpossible
  | ResourceUnavailable
  | FontResolutionFailure
  | ColorConversionImpossible
  | OutputBudgetExceeded
  | CandidateConflict
  | PlanningBudgetExceeded
  | PlanInvalidated


正規構造：

PlanningFailure {
  failure-kind,
  affected-source,
  unsatisfied-constraints,
  relevant-capabilities,
  attempted-candidates,
  rejected-candidates,
  conflict-core,
  possible-alternatives,
  source-origins
}

#### 73. Conflict Core

複数Hard Constraintが両立しない場合、その原因となる条件集合を示す。

PlanningConflictCore {
  constraints,
  capabilities,
  source-subtree,
  rejected-solutions
}


数学的な大域最小性は要求しない。

Planning Budget内で縮小された、理解可能なConflict Coreを提示する。

#### 74. Alternative Plan
AlternativePlan {
  required-changes,
  backend?,
  output-profile-changes?,
  output-artifacts,
  preserved-properties,
  losses,
  cost,
  approval-requirements
}


例：

- Editable PPTX＋Reference PDFへ分割
- Accessible PDFへ変更
- Raster fallbackを許可
- 別Backendを選択


Alternative Planを自動採用せず、利用者またはBuild Policyの選択を要求する。

#### 75. Verification Requirement

Planning Decisionごとに、Emission後の検査条件を生成できる。

VerificationRequirement {
  decision-id,
  property,
  verification-method,
  expected-result,
  tolerance?,
  severity-on-failure
}


例：

Plan:
Semantic Textを保持

Verification:
Text抽出結果と元Textを比較

Plan:
Fontを埋め込む

Verification:
Artifact内のFont Resourceを検査

#### 76. Artifact Verification Report
ArtifactVerificationReport {
  artifact-identity,
  plan-identity,
  verified-decisions,
  failed-verifications,
  unverified-properties,
  validator-identities,
  overall-status
}

VerificationStatus =
  Verified
  | VerifiedWithWarnings
  | Failed
  | Incomplete
  | ValidatorUnavailable


Strict Profileでは、必須VerificationがIncompleteまたはValidatorUnavailableなら成功扱いにしない。

#### 77. Plan違反

EmitterがPlanning IRと異なるFallbackを実行した場合、それは通常のLossではなく契約違反である。

Plan:
Native Text

Actual:
Raster Text


原因候補：

- Emitter Defect
- Capability declarationの誤り
- Plan Invalidated
- Artifact Validatorによる不一致検出


BestEffortであってもEmitterがPlanを勝手に変更してはならない。

#### 78. Plan Invalidated
PlanInvalidated {
  changed-assumption,
  affected-decisions,
  replanning-possible,
  partial-output-state
}


原因例：

- Font Resource消失
- Capability取消し
- Device loss
- Output Sink制約判明


Output Transactionをrollbackし、可能なら新Snapshotで再Planningする。

#### 79. Machine-readable Report

構造化Reportを正本とする。

BackendOutputReport {
  report-version,
  target-identity,
  build-identity,
  plan-identity,
  backend-identity,
  output-profile-identity,

  decisions,
  aggregated-losses,
  approvals,
  planning-failures,
  verification-report,
  attachments,
  redaction-level
}


CLI、IDE、HTML、CI表示は、このReportから生成する。

#### 80. Diagnostic基盤の再利用

Backend専用の独立Diagnostic Frameworkを新設せず、既存の構造化Diagnostic基盤を再利用する。

共通部分：

- Stable code
- Severity
- Source origin
- Related origins
- Structured arguments
- Notes
- Attachments
- Redaction


Backend固有部分：

- Output Decision
- Preservation Axis
- Loss
- Candidate
- Approval
- Verification

#### 81. Report Budget

巨大ArtifactではReport量を制限する。

ReportBudget {
  max-decisions,
  max-losses,
  max-origins-per-aggregate,
  max-alternatives,
  max-attachments,
  max-total-bytes
}


Budget超過時にも次は保持する。

- Error／Critical
- Approval Required
- Hard Constraint Failure
- Verification Failure
- 新規Loss


Reportのtruncation自体を明示する。

#### 82. Redaction
RedactionPolicy {
  paths,
  text-content,
  resource-identities,
  user-data,
  external-urls,
  secret-derived-values
}


Secret値は常にredactする。

Machine-readable reportへも平文Secretを含めない。

#### 83. 増分Planning

Planning結果はSubtree単位でCacheできる。

PlanningKey {
  source-subtree-identity,
  capability-dependency-summary,
  profile-dependency-summary,
  resource-dependency-summary,
  parent-compositing-context
}


変更されたSource、Capability Rule、Profile Rule、Resourceだけに依存するPlanning Nodeを無効化する。

#### 84. 局所再利用できない変更

次の変更は親、Page、Artifact全体へ影響を拡大する可能性がある。

- Backdrop Filter
- Group Blend
- Reading Order
- Font substitution
- Master Structure
- Global Color Profile
- Artifact Size Hard Limit


依存Summaryに基づき、安全側へ無効化範囲を拡張する。

#### 85. PreviewとFinal

PreviewとFinalは別のOutput Profile、Capability Snapshot、Planning IRを使用する。

Preview:
BestEffort
低解像度Raster
Proxy Resource
低Latency

Final:
Strict／Compatible
Resource固定
高解像度
全体Verification


Preview PlanをFinal Planへそのまま昇格しない。

ただし、Geometry、Text shaping、Resource decode等の有効な中間Cacheは再利用できる。

#### 86. Backend Conformance

Backendが宣言したCapabilityを、Conformance Testによって検証する。

ConformanceStatus =
  Declared
  | Tested
  | Certified
  | KnownLimited
  | Disabled


検査対象：

- Native Capability
- Equivalent Lowering
- Approximation Error Bound
- Semantic Text
- Accessibility
- Font Embedding
- Color
- Raster Island
- Artifact Validation


Portable BackendとNative Backendへ同じContract Testを適用できる。

Bytes完全一致ではなく、Capabilityに応じた同値条件を用いる。

#### 87. Capability SchemaのVersioning
BackendCapabilitySchemaVersion
BackendPlanningIrVersion
BackendOutputReportVersion


を分離して保持する。

未知の必須FeatureはUnsupportedとして扱う。

第三者拡張はNamespace付きExtensionとして表現できる。

CapabilityExtension {
  namespace,
  schema-version,
  data
}


共通Plannerが理解すべき機能は、規範Schemaへ昇格させる。

#### 88. 実装責任境界
共通Backend Package
- Capability Schema
- Output Profile
- Preservation Axis
- Loss型
- Planning IR共通型

Backend Planner
- Requirement抽出
- Candidate生成
- Constraint適用
- Preference選択
- Raster Island
- Resource Planning
- Decision Report

Backend Emitter
- Planning IRの形式固有出力
- Resource埋込み
- Output Transaction
- Format Finalization

Artifact Validator
- Plan契約と実Artifactの照合
- Semantic Text検査
- Accessibility検査
- Visual Diff
- Format Validation

Host／Build System
- Output Profile選択
- 明示承認
- Backend選択
- Attachment保存
- CI Policy
- Atomic Commit

#### 89. 適合試験

少なくとも次を検査する。

- CapabilityをBooleanだけで表さない
- 条件付きCapabilityがSnapshotから決定的に評価される
- Native／Lowering／Approximate／Raster／Unsupportedを区別する
- Hard ConstraintをPreferenceで相殺しない
- Tolerance超過候補を除外する
- Semantic TextとVisual Textを区別する
- Accessibility Lossを独立して記録する
- Font substitution時に必要ならLayoutを再実行する
- Raster IslandがConservative Boundsを包含する
- Backdrop依存時にRaster範囲を拡大する
- EmitterがPlan外Fallbackを行わない
- Planning Budget超過をUnsupportedと混同しない
- Output DecisionからSourceへ逆追跡できる
- Loss Fingerprintが一時IDへ依存しない
- 新規LossがCI Baselineで検出される
- 承認されていないLossをcommitしない
- Artifact VerificationがPlanと実出力の差を検出する
- Preview PlanをFinal Planとして使用しない
- Capability／Profile変更時に必要範囲だけ再Planningする
- SecretがReportへ出力されない

#### 90. 性能要件
PERF-BACKEND-01:
Page／Slide／Subtree単位でPlanning可能にする

PERF-BACKEND-02:
Capability Snapshot全体の変更で無関係Planを無効化しない

PERF-BACKEND-03:
Reporting Policyだけの変更で表現Planを再生成しない

PERF-BACKEND-04:
PreviewとFinalで有効な中間Cacheを共有可能にする

PERF-BACKEND-05:
Candidate数とSubtree探索へBudgetを設ける

PERF-BACKEND-06:
巨大Reportを集約しつつError情報を保持する

PERF-BACKEND-07:
同一Resourceを複数回変換・埋込みしない

PERF-BACKEND-08:
低解像度Preview RasterをFinalへ誤利用しない

PERF-BACKEND-09:
並列Planningで規範結果を変えない

PERF-BACKEND-10:
PlanningとEmissionを独立してCache可能にする

#### 91. 最終状態

```text
OPEN-BACKEND-PROFILE-001:
RESOLVED
```


本決定により、ReciplexaのBackend出力は次の性質を持つ。

- Backend Capabilityを条件付き・型付きで表現する
- Backendの能力と利用者のOutput方針を分離する
- Hard ConstraintとPreferenceを分離する
- 視覚、Text、編集性、Accessibility等を別軸で評価する
- FallbackをEmission前にPlanningする
- Raster化をSubtree単位へ局所化する
- TextのVisual layerとSemantic layerを分離できる
- Approximationへ明示的なError modelを要求する
- Planning結果を検証可能なIRとして保持する
- Emitterによる暗黙Fallbackを禁止する
- 情報損失をSourceまで追跡する
- 明示承認とCI Baselineを利用できる
- 出力後にPlan契約を検証する
- PreviewとFinalを別Profileとして扱う
- Backend実装をConformance Testで検証する

### OPEN-ERR-DIAG-001 Failure・Defect・構造化Diagnostic・Privacy・Lifecycle
#### DD-ERR-DIAG-001 決定概要
##### 状態
Status:
RESOLVED

Scope:
- Program Failure
- Cancellation
- Defect
- Terminal Failure
- Diagnosticの正規Schema
- Diagnostic CodeとSeverity
- Source OriginとProvenance
- Failure Record
- Diagnostic Projection
- Primary／Cause／Suppressed
- Privacy Label
- Redaction
- Attachment
- Process間転送
- Diagnostic Lifecycle
- Incremental更新と撤回
- Grouping、Suppression、Baseline
- RendererとLocalization
- Diagnostic Store

#### 0. 基本原則

Reciplexaでは、Program内部で発生した事象と、それを利用者やToolへ説明するDiagnosticを分離する。

Failure
≠
Diagnostic

Defect
≠
Diagnostic

Diagnostic
≠
表示文字列


処理の基本経路を次とする。

Program／Runtime Event
↓
Outcome Classification
↓
Failure／Defect Record
↓
Diagnostic Projection
↓
Structured Diagnostic
↓
Privacy Filter
↓
Renderer
├─ CLI
├─ IDE
├─ CI
├─ Test
└─ Machine-readable Report


Diagnosticの正本は、完成済みの表示文字列ではなく、Diagnostic Code、型付きArgument、Source Origin、Cause、Privacy Label等を持つ構造化Dataとする。

#### 1. OutcomeとDiagnosticの分離

次の事象を意味論上区別する。

Program Failure:
Programの公開契約に含まれる回復可能な失敗

Cancellation:
結果が不要になったか、親Scopeから停止された状態

Defect:
成立すべき契約または不変条件の違反

Terminal Failure:
RuntimeまたはProcessの健全性を保証できない状態

Diagnostic:
事象やWarningを人間またはToolへ伝える構造化情報


Diagnosticの存在だけからOutcomeを逆算しない。

Outcome:
処理の意味上の結果

Diagnostic:
その結果や関連問題の説明

Policy:
DiagnosticをBuild Failure等として扱う規則

#### 2. Program Failure

Program Failureは、型付きのDomain Dataとして表す。

例：

FileFailure {
  kind: NotFound,
  resource-id,
  operation: Read
}


Failureへ完成済みの利用者向け文面を正本として格納しない。

非推奨:
Failure {
  message: "ファイルを開けませんでした"
}


理由は次のとおりである。

- Localizationが困難になる
- Diagnostic Testが文言変更で壊れる
- Cause関係を構造化できない
- Privacy Redactionが困難になる
- CLI、IDE、CIで異なる説明を生成できない

#### 3. Cancellation

Cancellationは、Program Failureとは別の制御結果とする。

Failure:
処理を実行した結果、契約内の失敗が発生した

Cancellation:
処理結果が不要になったか、
親Scopeから停止を要求された


Cancellation自体を原則としてError Diagnosticにしない。

次のような予定されたCancellationは通常表示しない。

- 古いPreview Job
- Superseded Layout Job
- Shutdown中の予定されたService停止
- Test RuntimeによるCleanup


Cancellation中に次が発生した場合はDiagnostic対象となる。

- Cleanupが完了しない
- Cancellation受理が遅延した
- Native operationが停止しない
- Output rollbackが失敗した
- Resourceが残存した

#### 4. Cancellation Report

Cancellationの状態を次の構造で保持する。

CancellationReport {
  cancellation-id,
  reason,
  requested-by,
  affected-scope,
  expected,
  cleanup-status,
  late-results,
  remaining-resources,
  duration-or-budget
}


Diagnostic化は、Cancellationそのものではなく、Context、期待状態、Cleanup結果を含めて判断する。

#### 5. Defect

Defectは、通常のProgram契約では成立しているべき不変条件の違反である。

例：

- Validated IRにcycleが存在した
- one-shot Continuationが二重resumeされた
- Resourceが二重releaseされた
- EmitterがBackend Planning IRに違反した
- PackageInstanceIdの再計算結果が一致しなかった


Defectを通常のfailure Eへ変換し、Applicationが通常処理として握り潰すことを認めない。

#### 6. Defect Report

Defectの正規Reportを次とする。

DefectReport {
  defect-id,
  defect-code,
  violated-invariant,
  defect-scope,
  subsystem,

  origin,
  operation?,
  related-state-summary,

  recovery-status,
  runtime-trust-status,

  causes,
  suppressed,
  attachments,

  privacy-classification,
  producer
}


Defect Reportには、単なる「内部エラー」ではなく、何が成立すべきで、何が観測されたかを保持する。

#### 7. Defect Scope
DefectScope =
  Task
  | Service
  | RootScope
  | RuntimeInstance
  | WorkerProcess
  | Process
  | Unknown


Defect Scopeによって、停止・隔離する範囲を決める。

Task:
当該Taskを停止可能

Service:
Service全体を停止または再起動

RuntimeInstance:
Root ScopeまたはRuntimeを破棄

Process／Unknown:
Terminal Failure候補

#### 8. RecoveryとRuntime Trust
RecoveryStatus =
  Isolated
  | ScopeTerminated
  | RuntimeRestartRequired
  | ProcessRestartRequired
  | Unrecoverable
  | Unknown

RuntimeTrustStatus =
  Trusted
  | PartiallyTrusted
  | Untrusted
  | Unknown


Runtime TrustがUntrustedまたはUnknownの場合、通常Diagnostic Pipelineの継続を前提としない。

#### 9. Terminal Failure

Terminal Failureは、RuntimeまたはProcessの健全性を保証できない異常である。

例：

- Memory corruptionの疑い
- Native ABI破損
- Runtime bookkeeping破損
- Rust abort
- 信頼できないWorkerのCrash


Terminal Failureを通常のProgram Failureへ降格しない。

ApplicationOutcome:
Aborted(InfrastructureAbort)


として扱う。

#### 10. Emergency Diagnostic Pipeline

Terminal Failure時には、通常のPackage、Localization、Plugin、Allocator、Schedulerへ依存しない最小経路を使用する。

EmergencyDiagnostic {
  emergency-code,
  subsystem,
  isolation-boundary,
  last-known-operation,
  minimal-origin?,
  crash-artifact-reference?,
  output-transaction-state?,
  integrity-state
}


Emergency Pipelineの目的は、完全な説明ではなく、次を最低限安全に記録することである。

- どのSubsystemが停止したか
- どのProcessまたはWorkerか
- 最後に確認されたOperation
- Crash Artifactの有無
- Partial Outputを使用してよいか

#### 11. Native Panic

Native panicを捕捉できた場合でも、直ちに継続可能とはみなさない。

次を検査する。

- Ownership状態が確定しているか
- Pending Callbackが残っていないか
- Completion Tokenが一意か
- Native Resourceを安全に解放できるか
- Memory破損の可能性がないか


安全に隔離できる場合：

Native Panic
→ Defect Report
→ Task／Workerの終了


安全性を確認できない場合：

Native Panic
→ Terminal Failure
→ Process／Sandbox停止

#### 12. Failure Record

型付きFailureへ、Origin、Cause、Retry等の実行Contextを付加するため、次の概念構造を導入する。

FailureRecord<E> {
  failure-id,
  payload: E,

  origin?,
  operation?,
  context,

  causes,
  suppressed,

  retry-classification,
  actionability,
  privacy-classification,

  producer
}


Failure payload自体を表示情報で汚染せず、必要な実行ContextをFailure Recordで保持する。

#### 13. Failure Operation

Failure発生時に試みていた操作を構造化する。

FailureOperation =
  Read
  | Write
  | Decode
  | Encode
  | Resolve
  | Validate
  | Plan
  | Emit
  | Verify
  | Finalize
  | Shutdown
  | Custom


同じPermissionDeniedでも、ReadとWriteでは説明・修正方法が異なる。

#### 14. Retry Classification
RetryClass =
  Never
  | Immediate
  | AfterDelay
  | AfterResourceChange
  | AfterUserAction
  | AfterProfileChange
  | Unknown


例：

一時的Network Failure:
AfterDelay

Font不足:
AfterResourceChange

権限不足:
AfterUserAction

Backend Profile不適合:
AfterProfileChange

型不一致:
Never


Retry可能性と、自動Retryを許可するPolicyは分離する。

#### 15. Actionability
Actionability =
  UserActionable
  | PackageAuthorActionable
  | AdministratorActionable
  | ToolchainAuthorActionable
  | AutomaticallyRecoverable
  | NotActionable


Defectや組織Policy上の問題について、誤って利用者へ入力変更を要求しないようにする。

#### 16. CauseとSuppressed

Failure Recordは複数Causeを持てる。

PackageResolutionFailure
├─ Requirement A
└─ Requirement B


Primary Failure後に発生したCleanup Failure等はcausesではなくsuppressedへ保持する。

Primary:
Artifact Emission Failure

Suppressed:
Temporary File Cleanup Failure
Diagnostic Flush Failure


Suppressed Failureを破棄しない。

#### 17. Diagnostic Projection

Failure Recordから構造化Diagnosticを生成する処理をDiagnostic Projectionとする。

DiagnosticProjection<E> {
  project:
    FailureRecord<E>
    × DiagnosticContext
    -> DiagnosticBundle
}


一つのFailureから複数Diagnosticを生成できるため、戻り値をBundleとする。

DiagnosticBundle {
  primary,
  related,
  suggestions,
  attachments
}

#### 18. Projectionの責任

Diagnostic Projectionは次を決定する。

- Diagnostic Code
- Category
- Severity
- Message Template ID
- Structured Arguments
- Primary Origin
- Related Origin
- Cause relation
- Suggestion
- Attachment requirement
- Privacy Label


最終的な表示文はRendererが生成する。

#### 19. Projectionの所有者

Failureの意味を最も理解しているSubsystemまたはPackageが、基本Projectionを定義する。

Font Package:
FontFailureのProjection

Package Resolver:
ResolutionFailureのProjection

Backend Planner:
PlanningFailureのProjection

Task Runtime:
TaskFailureのProjection


共通Diagnostic基盤は次を担当する。

- PrivacyとRedaction
- Grouping
- Fingerprint
- Lifecycle
- Rendererへの引渡し


Package独自のProjectionに、Privacy Policyや出力媒体の判断まで委ねない。

#### 20. Reframing

上位層は、低水準Failureを利用者にとって意味のある上位Failureへ包み直せる。

FontReadFailure
↓
BackendPlanningFailure
↓
ArtifactBuildFailure


Diagnosticでは上位FailureをPrimaryとし、下位FailureをCauseとして表示できる。

Primary:
Accessible PDFを生成できません

Cause:
必要なFont Resourceを読み込めません

Underlying Cause:
Font埋込みがLicense Policyで許可されていません


Reframingしても、元Payload、Origin、Privacy Label、Cause、Attachmentを失わない。

#### 21. Diagnosticの正規Schema
Diagnostic {
  diagnostic-id,
  diagnostic-code,
  schema-version,

  kind,
  severity,
  category,
  lifecycle-stage,

  message,
  arguments,

  primary-origin?,
  related-origins,
  provenance?,

  causes,
  relations,
  suggestions,

  attachments,
  privacy-labels,
  redaction-state,

  producer,
  context,
  fingerprint
}

#### 22. Diagnostic IDとCode
Diagnostic ID

一回の実行中に発生した個々のDiagnostic instanceを識別する。

DiagnosticId

Diagnostic Code

Diagnosticの意味上の種類を安定して識別する。

DiagnosticCode {
  namespace,
  category,
  code
}


例：

compiler/type/TYPE-0012
runtime/task/TASK-0007
backend/text/BACKEND-0041
package/resolve/PKG-0020


表示文を改善しても、意味分類が同じならCodeを維持できる。

#### 23. Diagnostic Severity
DiagnosticSeverity =
  Fatal
  | Error
  | Warning
  | Notice
  | Info
  | Hint

Fatal

Runtime、ProcessまたはArtifactの信頼性を維持できない。

Error

現在のJob、Target、Test等を成功として完了できない。

Warning

継続可能だが、品質、互換性、安全性等に問題がある可能性がある。

Notice

重要だが通常は対応必須でない情報。

Info

実行状態や結果の補助情報。

Hint

修正や改善の候補。

SeverityをOutcomeと同一視しない。

#### 24. Diagnostic Category
DiagnosticCategory =
  Syntax
  | Macro
  | NameResolution
  | Type
  | Effect
  | Resource
  | Task
  | Runtime
  | Native
  | Package
  | Manifest
  | Layout
  | Render
  | Backend
  | Test
  | Codec
  | GUI
  | Security
  | Performance
  | Internal

#### 25. Diagnostic Lifecycle Stage

Diagnosticが発生した処理段階を記録する。

DiagnosticLifecycleStage =
  Parse
  | Expand
  | Resolve
  | TypeCheck
  | Lower
  | Evaluate
  | Plan
  | Emit
  | Verify
  | Shutdown
  | Test


同じResource Failureでも、Plan前、Emission中、Verification時を区別できる。

#### 26. MessageとArgument

Diagnosticの正本を完成済み文字列だけにしない。

DiagnosticMessage {
  template-id,
  arguments
}


例：

template-id:
backend.font.substitution

arguments:
{
  requested-font,
  replacement-font,
  affected-node-count
}


Localization Catalogが、Localeに対応する表示文を生成する。

第三者Package向けのFallbackとして、安全なPlain Text summaryを許可できるが、標準SubsystemではDiagnostic Codeと構造化Argumentを要求する。

#### 27. Origin

Diagnostic Originを次の種類に分ける。

DiagnosticOrigin =
  SourceOrigin
  | GeneratedOrigin
  | IrOrigin
  | ResourceOrigin
  | RuntimeOrigin
  | NativeOrigin
  | ArtifactOrigin
  | UnknownOrigin

#### 28. Source Origin
SourceOrigin {
  package-instance-id,
  module-id,
  source-resource-id,
  text-range,
  syntax-node-id?,
  binding-id?
}


File path文字列だけをOriginの正本にしない。

#### 29. Generated Origin
GeneratedOrigin {
  generated-range,
  generator-origin,
  expansion-chain,
  call-site,
  definition-site
}


MacroやTemplateによるDiagnosticでは、利用者が修正しやすいCall siteをPrimaryとし、Definition siteやGenerated codeをRelated Originにできる。

#### 30. IR・Runtime・Native・Artifact Origin
IrOrigin {
  ir-kind,
  ir-node-id,
  source-provenance,
  pass-identity,
  revision
}

RuntimeOrigin {
  runtime-instance-id,
  root-scope-id?,
  task-id?,
  operation-id?,
  scheduler-step?
}

NativeOrigin {
  package-implementation-id,
  adapter-id,
  abi-version,
  operation-id,
  worker-process-id?,
  native-backtrace-attachment?
}

ArtifactOrigin {
  artifact-id,
  member-id?,
  page-or-slide?,
  object-id?,
  region?,
  planning-decision-id?
}


生Memory addressや一時的なObject pointerを安定Originにしない。

#### 31. Primary OriginとRelated Origin

Diagnosticは、原則として高々一つのPrimary Originを持つ。

Primary Origin:
利用者が最初に確認・修正すべき箇所


その他の場所はRelated Originとする。

- 型の定義場所
- Package Requirementの発生元
- Macro definition
- 競合する別依存
- 関連Resource

#### 32. Provenance

複数IRを通過したOriginを連鎖として保持する。

Artifact Object
↓
Backend Plan Node
↓
Render IR Node
↓
Visual／Layout Node
↓
Domain IR Node
↓
RPX Binding
↓
Source Range


通常表示では必要な範囲だけ展開し、IDEや詳細Reportで完全Chainを確認できるようにする。

#### 33. Privacyの基本原則

Privacyは、Renderer直前の文字列置換ではなく、Diagnostic Schemaの一部として扱う。

Diagnostic内のArgument、Origin、Attachment等に個別のPrivacy Labelを付ける。

Diagnostic Code:
Public

Package名:
ProjectInternal

Source本文:
UserContent

絶対Path:
PersonalData

Native address:
SecuritySensitive

Credential:
Secret

#### 34. Privacy Class
PrivacyClass =
  Public
  | ProjectInternal
  | UserContent
  | PersonalData
  | SecuritySensitive
  | Secret

Public

公開可能な一般情報。

ProjectInternal

ProjectまたはWorkspace内部の情報。

UserContent

利用者が作成・入力した内容。

PersonalData

個人や個人環境を識別し得る情報。

SecuritySensitive

攻撃や防御回避に利用され得る内部情報。

Secret

Password、Token、Private Key等の秘密値。

Secret値はDiagnostic Dataへ原則として保存しない。

#### 35. Privacy Labelの粒度

Privacy Labelを次へ付与する。

- Diagnostic Argument
- Origin
- Related Origin
- Suggestion
- Attachment
- Stack Frame
- Resource Reference
- Structured Payload Field


Diagnostic全体のPrivacy Classは、含まれる情報から計算されるSummaryである。

#### 36. Privacy Labelの伝播

情報の変換後も、入力より弱いPrivacyへ暗黙に降格させない。

UserContent
+
Public
→
UserContent


SecretをHash化しただけでPublicにしない。

Privacyを弱化できるのは、認可されたSanitization operationだけとする。

- 件数へ要約
- Categoryへ一般化
- Workspace-relative Pathへ変換
- Report-local Pseudonymへ変換

#### 37. Disclosure Context
DisclosureContext =
  LocalInteractive
  | LocalStored
  | WorkspaceShared
  | OrganizationInternal
  | PublicBuildLog
  | ExternalCrashReport
  | Telemetry
  | TestSnapshot
  | InterProcessTransfer


同じDiagnosticでも、Contextに応じて公開可能な情報を変える。

#### 38. Redaction Action
RedactionAction =
  Preserve
  | Normalize
  | Relativize
  | Summarize
  | Pseudonymize
  | ReplaceWithCategory
  | Omit
  | DenyReport


Redaction後も、Diagnostic Code、Category、一般化された理由等を保ち、問題の意味が分かるようにする。

#### 39. Path・Source・URL

Pathの表示Policy：

PathDisplayPolicy =
  Absolute
  | WorkspaceRelative
  | PackageRelative
  | DocumentRelative
  | BasenameOnly
  | Pseudonymized
  | Hidden


Source Textの表示Policy：

SourceTextPolicy =
  FullRange
  | RelevantLines
  | TokenOnly
  | ShapeOnly
  | RangeWithoutContent
  | Hidden


URLはscheme、host、path、query等へ分解し、QueryとCredentialを既定でredactする。

#### 40. Stack Trace
StackTracePolicy =
  FullLocal
  | Symbolic
  | PackageOnly
  | SubsystemOnly
  | FingerprintOnly
  | Hidden


FrameごとにPrivacy Labelを持たせる。

生Memory addressをPublic Reportへ出力しない。

#### 41. Attachment
DiagnosticAttachment {
  attachment-id,
  kind,
  media-type,
  size,
  content-identity,
  privacy-class,
  storage-reference,
  retention-policy
}


例：

- Scheduler Trace
- Native Crash Report
- IR fragment
- Visual Diff
- Artifact
- Test Counterexample
- Package Conflict Graph


Diagnostic本文をredactしてもAttachmentに情報が残らないよう、同等以上に厳格なPolicyを適用する。

#### 42. Attachment Retention
RetentionPolicy =
  Ephemeral
  | UntilJobEnd
  | UntilSessionEnd
  | LocalPersistent
  | OrganizationRetention
  | ExplicitUserRetention


期限後はReferenceとContentをPrivacy-awareなGarbage Collectionの対象とする。

#### 43. Process間転送

DiagnosticをWorkerからHostへ転送する前に、送信側でPrivacy Filterを適用する。

受信側でも再検証する。

Sender:
Schema Validation
Privacy Filter
Size Budget

Receiver:
Schema Validation
Privacy再評価
Attachment隔離
Policy再適用


Sandboxed Workerや第三者PluginのPrivacy Labelを無条件に信頼しない。

#### 44. TelemetryとCrash Report

Telemetryでは、原則として次だけを扱う。

- Diagnostic Code
- Subsystem
- Toolchain Version
- Outcome Class
- Sanitized Platform Class
- Occurrence Count


Source本文、Path、Document内容、Secret、Stack Trace、Attachmentを既定で送信しない。

Crash ReportはLocal詳細版と外部送信用のredacted版を分離する。

外部送信には利用者同意または組織Policyを要求する。

#### 45. Diagnostic Fingerprint

Fingerprintは、異なるRun間で同じ問題を対応付けるために使う。

DiagnosticFingerprint {
  diagnostic-code,
  semantic-origin,
  relevant-arguments,
  producer-contract-version
}


次へ依存させない。

- 表示文
- 行番号だけ
- Process ID
- Taskの一時番号
- Wall-clock時刻
- Memory address
- Secret値


Sensitive値の区別が必要な場合は、Report-localなKeyed DigestまたはPseudonymを利用する。

#### 46. Diagnostic Kind

DiagnosticをLifecycleの異なる二種類へ分ける。

DiagnosticKind =
  StateDiagnostic
  | EventDiagnostic

State Diagnostic

現在のSource、IR、Artifact等の状態に対して成立する問題。

- 型不一致
- Layout overflow
- Deprecated API
- Accessibility不足


状態が変われば撤回される。

Event Diagnostic

特定の実行時点で発生した出来事。

- Export失敗
- Native Worker Crash
- Shutdown Timeout
- Test失敗


過去のExecution Historyとして保存できる。

#### 47. Diagnostic Lifecycle
DiagnosticLifecycleState =
  Pending
  | Active
  | Superseded
  | Resolved
  | Retracted
  | Stale
  | Archived

Pending

解析途中の暫定Diagnostic。

Active

現在Revisionに対して確認済み。

Superseded

より新しいDiagnosticへ置き換えられた。

Resolved

新しい解析で問題が解消したことを確認した。

Retracted

Producerが誤診または暫定判断を取り消した。

Stale

依存状態が変わり、現在も正しいか確認できていない。

Archived

過去実行の履歴として保存された。

#### 48. Revision Context

Diagnosticは対象RevisionとRunを持つ。

DiagnosticRevisionContext {
  source-revision?,
  module-revision?,
  ir-revision?,
  resource-snapshot?,
  runtime-profile-snapshot?,
  output-profile-snapshot?,
  build-identity?,
  execution-run-id
}


Producerごとに必要なRevisionだけを使用する。

#### 49. Diagnostic Owner
DiagnosticOwner {
  producer,
  analysis-unit,
  revision,
  run-id
}


Analysis Unitの例：

- Source File
- Module
- Binding
- Package
- Test
- Page
- Slide
- Render Subtree
- Artifact Member
- Root Scope


Producerは自分が所有する範囲のDiagnosticだけを更新・撤回できる。

#### 50. Diagnostic Setの更新

解析Unitごとに、Diagnosticを一件ずつ追記するのではなく、新しいDiagnostic SetとしてReconciliationする。

Previous Diagnostic Set
+
New Analysis Result
↓
Diagnostic Reconciliation


基本規則：

1. Fingerprintで旧新Diagnosticを対応付ける
2. 同じ問題なら既存IDを維持して更新
3. 意味が変わればSupersede
4. 新規問題を追加
5. 消滅した問題をResolvedまたはRetracted


解析Run自体が失敗した範囲では、旧DiagnosticをResolvedにせずStaleとする。

#### 51. Diagnostic Update
DiagnosticUpdate =
  Add
  | Update
  | Supersede
  | Resolve
  | Retract
  | MarkStale
  | Archive


IDEやToolingは差分Eventとして受け取れる。

#### 52. Originの移動

Source行の追加等でOriginが移動しても、Stable Syntax Node ID、Binding ID、Semantic Node ID、Incremental Parserの対応情報を使って、同じDiagnosticとして追跡する。

行番号だけをDiagnostic identityへ使用しない。

#### 53. Cause・Duplicate・Consequence

Diagnostic間の関係を区別する。

DiagnosticRelation =
  CausedBy
  | ConsequenceOf
  | DuplicateOf
  | SupersededBy
  | RelatedTo
  | SuppressedBy


同じ根本原因を複数Subsystemが重複報告した場合はDuplicateOfとする。

下位Failureによって上位処理も失敗した場合はConsequenceOfとする。

上位Failureを削除せず、Group内で折りたたんで表示できるようにする。

#### 54. Diagnostic Group
DiagnosticGroup {
  group-id,
  group-kind,
  primary,
  children,
  summary,
  aggregate-severity,
  outcome-relation
}

DiagnosticGroupKind =
  CauseChain
  | MultipleOrigins
  | BatchFailures
  | RepeatedIssue
  | PrimaryAndSuppressed
  | PlanningConflict
  | TestFailure


Group Severityは基本的に最大Severityを使用するが、PrimaryとSuppressedの区別を維持する。

#### 55. Diagnostic Suppression
DiagnosticSuppression =
  HideFromView
  | SuppressWarning
  | AcceptKnownIssue
  | DisableCheck
  | BaselineExisting

Hide From View

表示だけを隠し、Outcomeを変更しない。

Suppress Warning

指定WarningをPolicy上無視する。

Accept Known Issue

Fingerprint単位で既知問題を承認する。

Disable Check

Optional LintやCheck自体を無効化する。

型検査、Memory safety、IR Validation等の必須検査は無効化できない。

#### 56. Warning as Error

WarningをErrorへ書き換えず、PolicyでBuild Outcomeへの影響を定める。

Diagnostic:
Warning

Release Policy:
このWarning Codeが存在すれば失敗


Local IDEとCIで同じDiagnosticを異なるPolicyで扱える。

#### 57. Baseline

既知DiagnosticをFingerprintの集合として保存し、現在のDiagnosticと比較する。

BaselineComparison =
  Existing
  | New
  | Resolved
  | Changed


Baselineを通常Build中に自動更新しない。

Source control上でReview可能な変更として扱う。

#### 58. Renderer契約

RendererはDiagnosticの意味を変更せず、特定の媒体へ表示する。

DiagnosticRenderer {
  render:
    DiagnosticReport
    × RenderPolicy
    -> Output
}


Rendererの責務：

- Localization
- Severity表示
- Source位置表示
- Groupの展開・折りたたみ
- Suggestion表示
- Attachment Link


Rendererは禁止される処理：

- Diagnostic Code変更
- Severityの意味変更
- Cause関係の再構築
- Redacted情報の復元
- Outcomeの独自決定

#### 59. Localization
MessageCatalog {
  locale,
  catalog-version,
  templates
}


Diagnostic CodeとMessage Catalog Versionを分離する。

表示文を変更しても、意味が同じならDiagnostic Codeを維持する。

Testでは通常CodeとArgumentを比較し、Renderer自身のTestでのみ表示文を比較する。

#### 60. Suggestion
DiagnosticSuggestion {
  suggestion-id,
  title,
  applicability,
  source-revision?,
  edits,
  preconditions,
  risks,
  requires-approval
}

SuggestionApplicability =
  MachineApplicable
  | MaybeApplicable
  | NeedsReview
  | Informational


適用前にSource RevisionとPreconditionを再確認する。

意味変更を伴う修正をMachineApplicableにしない。

#### 61. Diagnostic Store
DiagnosticStore {
  active-by-owner,
  stale,
  history,
  groups,
  attachments,
  baselines
}

Active

現在Revisionに有効。

Stale

再解析待ち。

History

過去のBuild、Test、Export等のEvent Diagnostic。

Final Build時には、Staleな必須解析結果を残さない。

#### 62. Report Budget
DiagnosticBudget {
  max-active,
  max-per-code,
  max-per-origin,
  max-related-origins,
  max-cause-depth,
  max-attachments,
  max-total-bytes
}


Budget超過時にも次を保持する。

- Fatal
- Error
- Security上重要な問題
- Primary Diagnostic
- Suppressed Failureの要約
- 新規Diagnostic


省略内容をTruncated metadataとしてReportへ記録する。

#### 63. Diagnostic Report
DiagnosticReport {
  report-version,
  context,
  producers,

  diagnostics,
  groups,
  aggregates,

  attachments,
  baselines?,

  redaction-level,
  disclosure-context,
  truncation-state
}


Machine-readableなDiagnostic Reportを正本とする。

CLI、IDE、CI、HTML、Test等は同じReportから表示を生成する。

#### 64. Diagnostic生成Failure

Diagnostic Projection、Localization、Attachment保存等が失敗しても、元Failureを失わない。

Fallback順を次とする。

1. 完全なDiagnostic Bundle
2. Diagnostic Code＋安全な最小Argument
3. Diagnostic Code＋Origin
4. Producer＋一般Summary
5. Emergency Diagnostic


Privacy Filterが失敗した場合は、未redact情報を出力せず、最小限の安全なDiagnosticへ縮退する。

#### 65. Batch Failure

複数Jobの独立Failureを一つのCause chainへ連結しない。

BatchFailureReport {
  batch-id,
  item-outcomes,
  failure-groups,
  successes,
  aggregate-summary
}


Batch Policyによって全体Outcomeを決める。

Strict:
一件でも失敗なら全体失敗

Collect:
部分成功と部分失敗を保持

BestEffort:
成功分をcommitし、失敗分をReport

#### 66. DiagnosticとTest

TestではDiagnosticの表示文全体ではなく、通常次を比較する。

- Diagnostic Code
- Severity
- Category
- Primary Origin
- 重要Argument
- Cause
- Related Diagnostic


Actual／Expected値にはPrivacy Labelを伝播させる。

Secret値は表示せず、不一致であることだけを報告できる。

#### 67. DiagnosticとBackend

OPEN-BACKEND-PROFILE-001の次を同じDiagnostic基盤へ接続する。

- Output Decision
- Output Loss
- Planning Failure
- Alternative Plan
- Approval Required
- Artifact Verification Failure


Loss Fingerprint、CI Baseline、明示承認は共通Fingerprint、Privacy、Lifecycle機構を利用する。

#### 68. DiagnosticとPackage Resolver

Package Resolution Conflictは次を構造化Diagnosticへ変換する。

Primary:
解決できないPackage

Related:
各Version Requirementの発生元

Cause:
Singleton、ABI、Security等の制約

Suggestion:
更新、分離、Portable fallback等


依存Graph全体を無制限に表示せず、Conflict Coreと必要なAttachmentを使用する。

#### 69. DiagnosticとRuntime

Runtime Diagnosticは次を扱う。

- Task Failure
- Cancellation異常
- Task leak
- Resource leak
- Shutdown timeout
- Defect
- Terminal Failure


通常の予定されたTask CancellationをWarningとして大量表示しない。

Root ScopeのPrimary／suppressed関係をDiagnostic Groupへ維持する。

#### 70. 適合試験

少なくとも次を検査する。

- Failureに完成済み表示文を正本として要求しない
- 一Failureから複数Diagnosticを生成できる
- 上位Failureが下位Causeを保持できる
- Cancellationを常にErrorにしない
- Cancellation Cleanup異常を検出する
- Defect ReportがInvariantとScopeを持つ
- Runtime Trustが低い場合にEmergency Pipelineへ移る
- Diagnostic生成Failureで元Failureを失わない
- Primary／suppressed関係を維持する
- Diagnostic Codeと表示文を分離する
- OriginをSourceまで追跡できる
- Secret値をDiagnosticへ保存しない
- Public CIで絶対Pathを表示しない
- Source本文をDisclosure Contextに応じてredactする
- WorkerのPrivacy Labelを受信側で再検査する
- FingerprintへSecret値を含めない
- DiagnosticをRevision単位で撤回できる
- 解析未完了時に旧DiagnosticをResolvedにしない
- Warning as ErrorがSeverityを書き換えない
- Baselineの新規／既知／解決を区別する
- RendererがCauseやSeverityの意味を変更しない
- Report Budget超過時にもFatalとErrorを保持する

#### 71. 性能要件
PERF-ERR-DIAG-01:
増分解析で対象UnitのDiagnosticだけを更新する

PERF-ERR-DIAG-02:
Fingerprint一致時にIDE Objectを再利用できる

PERF-ERR-DIAG-03:
Source行移動だけでDiagnosticを再生成しない

PERF-ERR-DIAG-04:
大量の重複Diagnosticを表示時に集約できる

PERF-ERR-DIAG-05:
Privacy FilterをArgument単位で適用できる

PERF-ERR-DIAG-06:
AttachmentをDiagnostic本体と分離して遅延取得できる

PERF-ERR-DIAG-07:
Renderer変更で解析を再実行しない

PERF-ERR-DIAG-08:
Localization変更でDiagnostic identityを変えない

PERF-ERR-DIAG-09:
Stale Diagnosticを保持しつつ画面のちらつきを抑える

PERF-ERR-DIAG-10:
Final Reportを安定した順序で生成する

#### 72. 実装責任境界
Domain Package
- 型付きFailure payload
- 基本Diagnostic Projection
- Argumentの意味とPrivacy Label

Compiler／Runtime／Backend
- Failure Record作成
- OriginとProvenance
- CauseとSuppressed
- Defect Scope

共通Diagnostic基盤
- Diagnostic Schema
- Code
- Fingerprint
- Privacy伝播
- Redaction
- Lifecycle
- Grouping
- Baseline
- Diagnostic Store

Host
- Disclosure Context
- Privacy Policy
- CLI／IDE／CI Renderer
- Attachment Storage
- Crash Report
- Telemetry
- Retention

#### 73. 最終状態

```text
OPEN-ERR-DIAG-001:
RESOLVED
```

### OPEN-GUI-STATE-001 Stable Key・GUI State・Reconciliation・Interaction
#### DD-GUI-STATE-001 決定概要
##### 状態
Status:
RESOLVED

Scope:
- Document StateとGUI Stateの分離
- GUI Stateの分類と所有権
- Stable Key
- State Schemaと互換性
- State継承・移送・破棄
- GUI DescriptionとMounted Instance
- Reconciliation PlanningとCommit
- Component Lifecycle
- Task、Subscription、ResourceのLifetime
- Focus
- Selection
- Text Editing
- CaretとText Position
- IME Composition
- Pointer Capture
- Gesture
- Drag-and-drop
- Modal Interaction
- Privacy
- GUI Session State

#### 0. 基本原則

Reciplexaでは、GUIを再評価・再構築した際に、以前のGUI要素が持っていた状態を、構造上の位置だけを根拠として再利用しない。

GUI Stateの継承は、次を検査した上で行う。

- Stable Key
- State Owner
- State Schema
- Component Lifecycle
- Document／Text Revision
- Capability境界
- Privacy境界


基本原則は次のとおりとする。

- Document StateとGUI Stateを分離する
- GUI要素の位置とidentityを分離する
- 動的CollectionではStable Keyを必須とする
- Key一致だけでStateを再利用しない
- Node移動とNode置換を区別する
- GUI更新をPlanningとCommitに分離する
- Commit前にState、Focus、Task、Resourceの整合性を検証する
- Reconciliation中の再帰的更新を禁止する
- Focus、IME、Pointer Capture等を排他的Interactionとして管理する
- ComponentのTaskとResourceをComponent Scopeへ所属させる
- Owner消失時にはCancellationとCleanupを実行する
- GUI Session StateをDocument Snapshotから分離する
- Secret、Capability、Native handleをGUI Stateとして永続化しない

#### 1. Document StateとGUI State

文書の意味そのものに属するStateと、文書の表示・操作に属するStateを分離する。

Document State:
文書の規範的な内容

GUI State:
文書の表示・操作・一時Interactionに関する状態


Document Stateの例：

- ParagraphのText
- Shapeの位置
- Slideの順序
- Style
- Annotation
- Document metadata


GUI Stateの例：

- Scroll位置
- Zoom倍率
- Focus
- Caret
- Selection
- Panelの展開状態
- 入力中Draft
- Pointer gesture
- IME composition


Document Stateは、OPEN-EDT-CODEC-001のDocument Transaction、Snapshot、Transaction Logによって管理する。

GUI State Storeだけに文書の規範的な値を保持してはならない。

#### 2. GUI Stateの分類

GUI Stateを次へ分類する。

GuiStateClass =
  DocumentAssociatedState
  | ViewState
  | WidgetState
  | InteractionState
  | DerivedState
  | ExternalState

##### 2.1 Document-associated State

文書に関連するが、文書の規範的内容には含まれない状態である。

例：

- 文書単位の一時解析表示
- 文書に対するEditor-local annotation
- 文書単位の一時的なNavigation state


複数View間で共有する必要があるかを明示する。

文書の意味へ影響する場合は、GUI StateではなくDocument Stateへ移す。

##### 2.2 View State

特定のView instanceに属する状態である。

- Scroll位置
- Zoom倍率
- 表示中Page
- Panel layout
- Grid表示
- Ruler表示
- View上のSelection表示


同じDocumentを複数Viewで開く場合、Viewごとに別Stateを持つ。

Document A
├─ View 1: Page 1、Zoom 100%
└─ View 2: Page 20、Zoom 200%

##### 2.3 Widget State

特定Widgetの内部制御に属する状態である。

- Text FieldのCaret
- Text Fieldの局所Selection
- Tabの選択位置
- Sliderのdrag中値
- Dropdownの開閉
- ListのKeyboard navigation位置


Widget Stateは、Widget identityとState Schemaが互換である間だけ継承する。

##### 2.4 Interaction State

進行中の操作に属する短命な状態である。

- Pointer drag
- Hover
- Press gesture
- IME composition
- Drag-and-drop
- Marquee selection
- Modal interaction


Interaction Stateは、Owner、入力所有権、対象Revision、Cancellation、Cleanupを持つ。

通常の永続GUI Stateとは分離する。

##### 2.5 Derived State

入力から再構築できる派生状態である。

- Widget bounds
- Text measurement
- Hit-test index
- Visible item range
- Render cache


Derived StateはCacheとして扱う。

入力Revision、Resource、Font、Scale等が変化した場合には破棄する。

##### 2.6 External State

GUI Runtime外部のServiceやResourceに属する状態である。

- OS Window
- File picker
- Clipboard
- GPU Resource
- Network request
- Background Task
- Native control


これらを一般的なWidget Stateとして保存しない。

Task、Capability、Resource ownershipとして管理する。

#### 3. Committed StateとDraft State

Documentの正式な値と、GUIで編集中の未確定値を区別する。

Committed State:
Document Transactionへ反映済みの値

Draft State:
まだ文書へ反映されていない編集中の値


即時反映型の編集では、入力ごとにDocument Transactionを生成できる。

Input
↓
Document Transaction
↓
Document更新
↓
GUI再評価


DialogやProperty Editorでは、Draft BufferをViewまたはWidget Stateとして保持できる。

Apply:
DraftをDocument Transactionへ変換

Cancel:
Draftを破棄


DraftをDocument Stateへ暗黙反映しない。

#### 4. GUI State Identity

GUI State identityを次の階層で構成する。

Document Identity
└─ View Instance Identity
   └─ Semantic Owner Identity
      └─ Widget Key Path
         └─ State Slot Identity

##### 4.1 Document Identity

Documentに関連するView Stateには、OPEN-EDT-CODEC-001で定めたDocumentIdentityを使用する。

別DocumentへStateを誤って適用してはならない。

##### 4.2 View Instance Identity

同じDocumentに対する複数Viewを区別する。

ViewInstanceId


例：

- Main Editor View
- Secondary Window
- Outline View
- Presentation Preview
- Print Preview


View Identityの再利用範囲は、View Lifecycle Policyで定める。

##### 4.3 Semantic Owner Identity

GUI要素が表示・編集する意味対象を示す。

SemanticOwnerIdentity =
  DocumentNodeId
  | SemanticNodeId
  | ApplicationEntityId
  | None


例：

Slide thumbnail:
DocumentNodeId(slide)

Paragraph editor:
DocumentNodeId(paragraph)

Accessibility heading:
SemanticNodeId(heading)


Collection内の位置が変わっても、Semantic Ownerが同じならStateを同じ対象へ追従させられる。

##### 4.4 Widget Key Path

同一Semantic Ownerの内部にある複数Widgetを区別する。

WidgetKeyPath =
  sequence<WidgetKeySegment>


例：

paragraph-node-A
├─ text-editor
├─ style-button
└─ comment-indicator


完全なKeyは、親Component identityと局所Keyを含むPathとして構築する。

##### 4.5 State Slot Identity

一つのWidgetが所有する複数Stateを区別する。

StateSlotId =
  focus
  | selection
  | caret
  | draft
  | scroll-anchor
  | custom-state


State Slotは型identityとSchema Versionを持つ。

#### 5. GUI State Key

概念的な正規形を次とする。

GuiStateKey {
  state-domain,
  view-instance-id,
  semantic-owner-id?,
  widget-key-path,
  state-slot-id
}


State Storeでは次を保持する。

StoredGuiState<S> {
  key,
  state-schema,
  value: S,
  owner-lifecycle,
  revision-dependencies,
  compatibility-contract,
  retention-policy,
  privacy-labels
}


Key一致は、単なるLookup成功ではなく、同じState ownerとして継承してよい可能性があることを意味する。

最終的な継承には、追加の互換性検査を必要とする。

#### 6. Explicit KeyとStructural Key

GUI要素のKeyを次へ分類する。

GuiElementKey =
  ExplicitKey
  | StructuralKey

##### 6.1 Explicit Key

Componentまたは利用者が明示的に指定する。

key = StableNodeId(item)


次ではExplicit Keyを要求する。

- 動的Collection
- 並べ替え可能Collection
- Virtualized Collection
- 条件付きで生成される反復要素
- Positionが変化する要素

##### 6.2 Structural Key

静的なGUI構造からCompilerまたはRuntimeが導出する。

Toolbar
└─ Save Button


固定構造内の一意なWidgetについて許可する。

Binding ID、Component定義上のStable Syntax identity、局所slot等から導出できる。

##### 6.3 Position Indexの禁止

動的Collectionで配列IndexだけをKeyとして使用してはならない。

変更前:
0 = A
1 = B
2 = C

Bの前へXを挿入:

0 = A
1 = X
2 = B
3 = C


IndexをKeyにすると、BのStateがXへ移る。

動的CollectionではData itemのStable identityを使用する。

#### 7. Keyの型安全性

すべてのKeyを単なるTextへ変換しない。

SlideId("42")
ParagraphId("42")
WidgetKey("42")


を区別できる型付きKeyを使用する。

概念的には次とする。

WidgetKeySegment {
  namespace,
  key-kind,
  value
}


または、

GuiKey<A>


として型parameterでDomainを区別する。

#### 8. Duplicate KeyとMissing Key

同じOwner範囲で、同じ完全Keyが複数回現れてはならない。

DuplicateGuiKey:
Validation Failure


宣言順で区別して処理を継続しない。

動的CollectionでStable Keyがない場合：

MissingStableKey:
Diagnostic


BestEffort Previewでは一時Keyを生成できるが、State継承を保証してはならない。

Final、Test、Strict GUI validationではFailureとする。

#### 9. State Schema

GUI Stateごとに型identityとSchema Versionを持たせる。

GuiStateSchema {
  state-type-id,
  schema-version,
  compatibility-class,
  migration-contract?
}


State Compatibilityを次とする。

StateCompatibility =
  Exact
  | Migratable
  | ResetRequired
  | Forbidden

##### 9.1 Exact

同じState型・同じ意味であり、そのまま再利用できる。

##### 9.2 Migratable

State Schemaは変化したが、明示されたState Migrationで変換できる。

TextFieldState v1
↓
TextFieldState v2

##### 9.3 Reset Required

Stateの意味が変化したため、新Stateを初期化する。

##### 9.4 Forbidden

Securityや契約上、Stateを移送してはならない。

通常Text Field
→ Secret Input Field

#### 10. State継承条件

State継承の正規条件を次とする。

inherit-state(old, new)
iff
  old.full-key = new.full-key
  and owner-compatible(old, new)
  and schema-compatible(old, new)
  and lifecycle-allows-inheritance(old, new)
  and revision-dependencies-valid(old, new)
  and privacy-boundary-compatible(old, new)
  and capability-boundary-compatible(old, new)
  and no-explicit-reset(new)


Key一致だけを条件にしない。

#### 11. Node MoveとNode Replace

Document Nodeが別位置へ移動しても、Stable Node IDが同じなら同じ論理Nodeとして扱う。

Move:
Identity維持
State継承可能


Document Nodeが削除され、別Nodeが同じ位置へ挿入された場合は、別identityである。

Replace:
新Identity
旧Stateを暗黙継承しない


Widget kindやSecurity classが変わった場合もReplaceとして扱う。

#### 12. 明示的State Transfer

別identity間で一部Stateだけを移したい場合、KeyやNode IDを使い回さず、明示的なState Transferを使用する。

GuiStateTransfer {
  source-owner,
  target-owner,
  allowed-slots,
  state-mapping,
  compatibility-proof,
  capability-proof,
  privacy-proof,
  transfer-policy
}


例：

Caret:
Text mappingがある場合に移送可能

Scroll:
意味あるAnchorがある場合に移送可能

Secret Draft:
移送禁止


同じ論理Nodeならidentityを維持する。

別Nodeなら新identityとState Transferを使用する。

#### 13. GUI State所有権
GuiStateOwner =
  Document
  | View
  | ComponentInstance
  | WidgetInstance
  | InteractionSession
  | Host


OwnerごとにLifetimeを分離する。

ComponentやWidgetが消えた場合、そのOwnerに属するState、Task、Resourceをcleanupする。

#### 14. GUI State Store

概念的には次の構造を持つ。

GuiStateStore {
  active-state,
  retained-state,
  pending-disposal,
  state-migrations,
  owner-index,
  revision
}

##### 14.1 Active State

現在のMounted GUIから参照されているStateである。

##### 14.2 Retained State

現在は表示されていないが、短期間の再表示やVirtualizationに備えて保持されるStateである。

##### 14.3 Pending Disposal

Ownerが消え、Task、Resource、Subscription等の終了処理中にあるStateである。

Cleanup完了前に完全削除しない。

#### 15. State Retention
GuiStateRetention =
  WhileMounted
  | WhileViewAlive
  | WhileDocumentOpen
  | UntilSessionEnd
  | PersistentSession
  | Custom


例：

Hover:
WhileMounted

Virtualized Item State:
WhileViewAlive

Scroll:
WhileDocumentOpenまたはPersistentSession

Window layout:
PersistentSession


文書内容そのものをGUI StateのRetentionによって保存しない。

#### 16. State BudgetとEviction
GuiStateBudget {
  max-state-count,
  max-total-bytes,
  max-retained-state,
  max-per-view,
  max-per-component,
  retention-duration?
}


Evictionの基本優先順位を次とする。

1. Derived Cache
2. Unmount済みの再構築可能State
3. 古い軽微なWidget State
4. 古いView State
5. User Draft等の非再構築State


User Draftを黙ってEvictしてはならない。

次のいずれかを必要とする。

- DocumentへCommit
- Recovery Storeへの退避
- 利用者確認
- 明示破棄

#### 17. GUI Stateの永続化

一部のView StateはSession間で保存できる。

- Window配置
- Panel layout
- Zoom
- 最後に表示したPage
- Scroll Anchor


Document Snapshotへは含めず、別のGUI Session State Codecへ保存する。

GuiSessionState {
  workspace-or-document-reference,
  view-kind,
  state-entries,
  gui-schema-version,
  privacy-metadata
}


State Slotごとに永続化Policyを持たせる。

GuiStatePersistence =
  Never
  | SessionOnly
  | LocalPersistent
  | WorkspaceShared


次は原則としてNeverとする。

- IME Composition
- Pointer Capture
- Gesture
- Secret Draft
- Native Session handle
- Capability token
- Task handle

#### 18. GUI DescriptionとMounted Instance

宣言的なGUI構造と、実行中のInstanceを分離する。

GuiDescription:
次に望まれるGUI構造

MountedGuiInstance:
現在実際に存在し、
State、Task、Resourceを所有する構造


概念構造：

GuiDescription {
  element-kind,
  key,
  properties,
  children,
  event-bindings,
  state-requirements
}

MountedGuiInstance {
  instance-id,
  full-key,
  element-kind,
  mounted-state,
  child-instances,
  component-scope,
  resources,
  subscriptions,
  lifecycle-state
}


GuiDescriptionは原則として副作用を持たない不変値とする。

#### 19. GUI Instance Lifecycle
GuiInstanceLifecycle =
  Unmounted
  | Preparing
  | Mounted
  | Updating
  | Suspended
  | Unmounting
  | Disposed
  | Defected

##### 19.1 Unmounted

Runtime上にInstanceが存在しない。

##### 19.2 Preparing

Mountに必要なState、Resource、Child等を準備している。

通常のEvent受付はまだ行わない。

##### 19.3 Mounted

GUI tree内で有効である。

- Event受付可能
- State利用可能
- Task／Subscription稼働可能
- Focus取得可能

##### 19.4 Updating

既存Instanceを維持しつつ、新状態へ更新する準備中である。

Commitまでは旧状態を有効とする。

##### 19.5 Suspended

論理的なInstanceとStateを保持しつつ、表示や活動を一時停止している。

##### 19.6 Unmounting

Event受付を停止し、Task、Subscription、Focus、Resource等を終了している。

##### 19.7 Disposed

Cleanupが完了し、Instanceを再利用できない。

##### 19.8 Defected

Lifecycle契約またはGUI Runtime不変条件が破られた。

#### 20. Reconciliation

旧Mounted Treeと新GuiDescriptionを比較し、新状態への更新計画を作る。

Old Mounted GUI
+
New GUI Description
+
Existing GUI State
↓
Reconciliation Planning
↓
Validated Reconciliation Plan
↓
Commit
↓
Lifecycle Effects


GUIを差分検出順に逐次変更してはならない。

#### 21. Reconciliation Operation
GuiReconciliationOperation =
  Mount
  | Update
  | Move
  | Suspend
  | Resume
  | Unmount
  | Replace
  | Retain

##### 21.1 Mount

新Descriptionにだけ存在する要素を新規作成する。

##### 21.2 Update

完全Key、Owner、Element kind、State Schemaが互換な既存要素を更新する。

##### 21.3 Move

同じInstanceを別親または別位置へ移動する。

Stateを維持できるが、Capability、Focus、Modal、Accessibility、Environment境界を再検証する。

##### 21.4 Suspend

Instance identityとStateを維持し、活動を一時停止する。

##### 21.5 Resume

Suspended Instanceを再有効化する。

Resource、Capability、Document Nodeの有効性を再検査する。

##### 21.6 Unmount

InstanceのLifetimeを終了する。

##### 21.7 Replace

旧要素をUnmountし、別identityの新要素をMountする。

Stateを暗黙移送しない。

##### 21.8 Retain

変更なし、またはState継承だけで維持できる要素である。

#### 22. Reconciliation Plan
ReconciliationPlan {
  plan-id,
  source-gui-revision,
  target-gui-revision,

  instance-matches,
  state-transfers,
  mount-operations,
  update-operations,
  move-operations,
  suspend-operations,
  resume-operations,
  unmount-operations,

  focus-plan,
  pointer-capture-plan,
  ime-plan,
  accessibility-plan,

  resource-plan,
  task-scope-plan,
  subscription-plan,

  validation-summary
}


検証済みPlanを作成するまで、現在のMounted GUIへ不可逆な変更を加えない。

#### 23. Reconciliationの処理段階
1. Description Evaluation
2. Matching
3. Planning
4. Validation
5. Prepare
6. Commit
7. Post-commit Lifecycle
8. Cleanup

##### 23.1 Description Evaluation

Document State、Application State、View Stateから新しいGuiDescriptionを評価する。

原則としてpureとし、Document Transactionや外部作用を実行しない。

##### 23.2 Matching

Stable Key、Semantic Owner、Structural Keyを使って旧Instanceと新Descriptionを対応付ける。

##### 23.3 Planning

State、Task、Resource、Focus、Interaction等の移行計画を生成する。

##### 23.4 Validation

少なくとも次を検査する。

- Keyの一意性
- Owner compatibility
- State Schema compatibility
- Focus一意性
- Pointer Capture一意性
- Task Scope ownership
- Capability境界
- Privacy境界
- Accessibility tree
- Native Resource移送


失敗した場合、旧GUIを維持する。

##### 23.5 Prepare

新State、Resource、Native control等をCommit可能な状態まで準備する。

外部から観測可能な状態へまだ公開しない。

##### 23.6 Commit

Mounted tree、State ownership、Focus、Interaction routing等を論理的に一つの境界で切り替える。

##### 23.7 Post-commit Lifecycle

Commit後に次を開始する。

- Component Task
- Subscription
- Animation
- Focus request
- Accessibility notification


Commit前に長寿命副作用を開始してはならない。

##### 23.8 Cleanup

旧InstanceのTask、Resource、Subscription、Native control等を終了する。

Cleanup Failureを黙って無視しない。

#### 24. Commit Barrier

Commit中に届いたEventを旧treeへ配送しないよう、ViewごとのCommit barrierを設ける。

Begin Commit
↓
Event配送を保留
↓
TreeとStateを切替
↓
Focus／Capture／IME routingを更新
↓
End Commit
↓
保留Eventを新treeで再評価


高頻度Pointer Event等は、安全な範囲でcoalesceできる。

#### 25. Reconciliation中の更新

Description EvaluationまたはCommit中に、同じViewのReconciliationを同期的に再開始してはならない。

更新要求はQueueへ送り、現在Cycle終了後に処理する。

Current Reconciliation
↓
Update Request
↓
Queue
↓
Next Reconciliation Cycle


短時間の複数更新は、Transaction orderingを壊さない範囲でcoalesceできる。

#### 26. Lifecycle Effect

Lifecycle Hookを制限された段階へ分類する。

GuiLifecycleHook =
  BeforePrepare
  | Prepare
  | AfterCommit
  | BeginSuspend
  | Resume
  | BeginUnmount
  | Dispose


Prepareでは、外部から観測可能な永続副作用を行ってはならない。

AfterCommitではTaskやSubscriptionを開始できる。

BeginUnmount以降では新しい通常Taskを開始できない。

Dispose後にStateやResourceへアクセスしてはならない。

#### 27. Component Task Scope

Stateful Componentは専用のTask Scopeを持てる。

Component Scope
├─ Data Load Task
├─ Animation Task
├─ Subscription Task
└─ Debounce Timer


Component KeyとOwnerが維持される場合、Scopeを維持できる。

Replace、Unmount、非互換State Migrationの場合は旧Scopeを終了する。

Taskを別Componentへ暗黙移送しない。

#### 28. Subscription

Subscription更新を次の順で処理する。

1. 新Subscriptionを準備
2. Commit barrier
3. Routingを新Subscriptionへ切替
4. 旧Subscriptionを解除


GenerationまたはEpochを使い、同一Eventが新旧両方へ届かないようにする。

#### 29. SuspendとOffscreen Retention

Offscreen要素の扱いを次へ分類する。

OffscreenMode =
  KeepMounted
  | Suspend
  | UnmountRetainState
  | UnmountDiscardState


例：

短時間のTab切替:
Suspend

Virtualized List:
UnmountRetainState

再構築可能なDecoration:
UnmountDiscardState


Unmount後に保持するのは選択されたState Slotだけであり、Component TaskやResourceは残さない。

#### 30. Error Boundary

Component Subtreeの通常Failureを隔離するため、Error Boundaryを利用できる。

GuiErrorBoundary {
  boundary-key,
  handled-failure-classes,
  fallback-description,
  reset-policy,
  escalation-policy
}


通常Failureでは、Child Scopeを終了してFallback UIをMountできる。

Defectについては、GUI Runtime全体の健全性が確認できる場合に限りSubtree隔離を許可する。

State StoreやMounted treeが不整合な場合、ViewまたはGUI Runtime全体を再構築する。

#### 31. Interaction State

Focus、Selection、IME、Pointer Capture等を一般Widget Stateから分離し、Interaction Sessionとして管理する。

InteractionSession {
  interaction-id,
  interaction-kind,

  owner,
  participants,
  scope,

  input-sources,
  target-document?,
  target-revision?,
  target-elements,

  lifecycle-state,
  exclusivity-policy,
  transfer-policy,
  rebase-policy,
  cancellation-policy,

  interaction-state,
  task-scope?,
  native-session-reference?,

  privacy-labels,
  diagnostic-context
}

#### 32. Interaction Lifecycle
InteractionLifecycle =
  Proposed
  | Active
  | Suspended
  | Committing
  | Cancelling
  | Completed
  | Cancelled
  | Failed
  | Defected


InteractionはOwner、入力所有権、対象Revision、Cleanupを持つ。

Owner消失時の既定はCancellationとする。

#### 33. Focus

FocusをChannelへ分ける。

FocusChannel =
  Keyboard
  | Accessibility
  | Pointer
  | Programmatic
  | Custom


Focus ScopeとChannelの組合せごとに、Ownerは高々一つとする。

FocusState {
  focus-channel,
  focus-scope,
  owner-widget-key?,
  focus-path,
  revision,
  reason
}


Accessibility FocusとKeyboard Focusを同一視しない。

##### 33.1 Focus Traversal

Focus移動の順序を次の優先順位で決める。

1. Explicit traversal order
2. Semantic reading order
3. Stable structural order


画面座標だけでTraversal orderを決めない。

Cycle、Duplicate order、無効Ownerを検出する。

##### 33.2 Focus Owner消失
FocusFallbackPolicy =
  Clear
  | FocusNearestFocusableAncestor
  | FocusExplicitFallback
  | FocusDefaultInScope
  | RestorePreviousOwner


同じ画面位置の別Widgetへ暗黙移動しない。

##### 33.3 FocusとDraft Commit

Focus移動とDraftのDocument commitを常に不可分にしない。

入力値が不正でもFocusを移動できる場合がある。

Validation Errorは別途Diagnosticとして表示する。

#### 34. Selection

Selectionを型で区別する。

SelectionKind =
  DocumentNodeSelection
  | TextRangeSelection
  | TableCellSelection
  | TimelineRangeSelection
  | ViewRegionSelection
  | Custom


SelectionはOwner View、対象Document、対象Revisionを持つ。

##### 34.1 Document Node Selection

Stable Node IDで対象を追跡する。

Node移動ではSelectionを維持できる。

Node削除時には次のPolicyを適用する。

SelectionDeletionPolicy =
  ClearDeletedEntries
  | ClearAll
  | SelectParent
  | SelectNearestSibling
  | PreserveTombstone

##### 34.2 Multi-selection
MultiSelection {
  selected-items,
  primary-item?,
  anchor-item?,
  ordering
}


集合だけでなく、Primary item、Anchor、選択順を保持できる。

#### 35. Text Position

v1のText Positionを次とする。

TextPosition {
  text-node-id,
  text-revision,
  scalar-offset,
  affinity,
  semantic-anchor?
}

TextAffinity =
  Before
  | After


規範位置にはUnicode scalar offsetを用い、UI操作ではGrapheme boundaryへ変換する。

Byte offsetやPlatform依存のcode-unit offsetを正本にしない。

#### 36. Text Selection
TextSelection {
  anchor-position,
  focus-position,
  preferred-column?,
  direction,
  source-revision
}


AnchorとFocusを分け、Selection方向を保持する。

Selectionを単なる昇順Rangeへ正規化して方向情報を失わない。

#### 37. Text Position Rebase

Document Transaction後にText Positionを変換する。

TextPositionTransform {
  source-text-revision,
  target-text-revision,
  operation-mappings
}


基本規則：

挿入点より前:
位置維持

挿入点より後:
挿入長だけ移動

挿入点と同じ:
Affinityに従う

削除範囲内:
規定境界へcollapse

DeletedPositionPolicy =
  CollapseToStart
  | CollapseToEnd
  | UseSemanticAnchor


Rebase不能な場合、古い位置へcommitしてはならない。

RebaseFailurePolicy =
  Clear
  | CollapseToNodeBoundary
  | UseSemanticAnchor
  | RequireUserDecision

#### 38. Text Editing Session
TextEditingSession {
  interaction-id,
  owner-widget-key,
  text-node-id,

  base-document-revision,
  base-text-revision,

  selection,
  draft-mode,
  pending-operations,
  validation-state,

  ime-session?,
  commit-policy,
  cancellation-policy
}


Editing modeを次へ分ける。

Immediate:
入力ごとにDocument Transaction

Draft:
一時Bufferへ入力し、Apply時にTransaction


Draft中にDocumentが外部更新された場合、Base RevisionとのConflictを検出する。

Draftで現在値を無条件上書きしない。

#### 39. IME Composition

IMEの未確定TextをDocumentへ直ちに保存せず、Interaction Stateとして保持する。

ImeCompositionSession {
  interaction-id,
  owner-widget-key,
  text-node-id,

  base-text-revision,
  replacement-range,
  marked-text,
  selected-range-in-marked-text,

  native-session-reference,
  privacy-class,
  lifecycle-state
}


処理：

IME update:
Overlayを更新

IME commit:
Document Text Transactionを生成

IME cancel:
Overlayを破棄


外部Transaction後にReplacement Rangeを安全にRebaseできない場合、Compositionをcancelする。

##### 39.1 Secret Input

Secret InputのStateには次を要求する。

- Marked TextをDiagnosticへ出さない
- GUI Session Stateへ保存しない
- 通常Text FieldへState Transferしない
- Unmount時にcancelする
- Secret DraftをDebug dumpへ含めない

#### 40. Pointer Capture
PointerCapture {
  pointer-id,
  owner-widget-key,
  interaction-id,
  view-id,
  capture-generation,
  capture-mode
}

PointerCaptureMode =
  Explicit
  | GestureOwned
  | DragAndDropOwned
  | HostOwned


同じViewとPointerについて、Capture Ownerは高々一つとする。

Owner Unmount、Pointer up、Device disconnect、Modal変更等で解放する。

別Widgetへ暗黙移送しない。

#### 41. Gesture

競合するGesture候補をGesture Arenaで管理する。

GestureArena {
  arena-id,
  input-streams,
  candidates,
  resolution-state,
  winner?,
  cancellation-results
}

GestureCandidateState =
  Possible
  | Accepted
  | Rejected
  | Won
  | Cancelled
  | Completed


排他的Gestureが確定した場合、競合候補をcancelする。

同時成立可能なGestureには明示的なCompatibility contractを要求する。

Gesture解決結果をCandidate登録順やHashMap順に依存させない。

#### 42. Drag-and-drop
DragSession {
  interaction-id,
  source-owner,
  payload-descriptors,
  allowed-operations,
  current-target?,
  preview?,
  capability-grants,
  lifecycle-state
}


Payloadを次のDescriptorで表す。

DragPayloadDescriptor {
  media-type,
  size?,
  privacy-label,
  transfer-mode,
  provider
}

TransferMode =
  InMemory
  | Stream
  | ResourceReference
  | HostMediated


Capability tokenやSecretを一般Payloadとして転送しない。

##### 42.1 Drop Target
DropTargetContract {
  accepted-media-types,
  allowed-operations,
  required-capabilities,
  validation,
  preview-policy
}


Drop結果がDocumentへ影響する場合、Document Transactionとして原子的にcommitする。

Drop validation
↓
Resource preparation
↓
Document Transaction
↓
Commit
↓
Drag completed


Transaction失敗時にDrop成功として扱わない。

#### 43. Modal Interaction
ModalScope {
  modal-id,
  owner,
  blocked-scopes,
  allowed-interactions,
  focus-boundary,
  dismissal-policy
}


Modal開始時：

- FocusをModal内へ移す
- 外部Pointer interactionをcancelまたは保留
- Accessibility traversalをModal内へ制限


Modal終了時：

- 以前のFocus ownerが有効なら復元
- 無効ならFocus Fallback Policy


Modal外へFocusやPointer Captureが漏れないようにする。

#### 44. Interaction Cancellation
InteractionCancellationReason =
  OwnerUnmounted
  | TargetDeleted
  | RevisionConflict
  | ReconciliationReplacedOwner
  | ModalBoundaryChanged
  | DeviceDisconnected
  | HostRequested
  | UserCancelled
  | CapabilityRevoked
  | Timeout
  | DefectIsolation


Cancellation順序：

1. 新規Input受付停止
2. Native Sessionへcancel通知
3. 入力Ownership解放
4. Draft／Overlay処理
5. Child Task停止
6. Cleanup
7. Cancelled Outcome確定


予定されたCancellationをError Diagnosticとして大量表示しない。

#### 45. Reconciliation時のInteraction継承

Interactionを継承できる条件を次とする。

- 完全Keyが一致
- Owner contractが互換
- State Schemaが互換
- Target Nodeが維持される
- Revision mappingが存在する
- Capability境界が互換
- Privacy classが弱まらない
- Interaction kindが継続可能


Updateや同一View内Moveでは継承できる場合がある。

Replaceでは原則としてcancelする。

別Ownerへの移送には明示的なInteractionTransferを要求する。

InteractionTransfer {
  source-owner,
  target-owner,
  interaction-kind,
  state-mapping,
  revision-mapping,
  capability-proof,
  privacy-proof
}

#### 46. Optimistic Interaction

Document TransactionのCommit前に一時的な結果を表示できる。

OptimisticInteractionState {
  base-revision,
  preview-delta,
  pending-transaction,
  rollback-plan
}


Commit成功後に正式状態へ移行する。

失敗時にはPreviewをrollbackする。

Rollback不能な一時変更を、GUI外部へ不可逆に公開してはならない。

#### 47. InteractionとUndo

Interaction完了時に生成されるTransactionへIntent metadataを付ける。

- Move Shape
- Type Text
- Resize Object
- Drop Image


多数の中間更新を一つのUndo Groupへまとめられる。

Cancelled Interactionを通常Undo historyへ追加しない。

すでに中間Transactionをcommitしていた場合は、Inverse TransactionまたはHead移動によって元へ戻す。

#### 48. Privacy

GUI StateとInteraction StateにはPrivacy Labelを付ける。

対象例：

- Draft Text
- Search query
- IME Marked Text
- Secret Selection
- Drag payload
- Clipboard content
- Navigation history


次を禁止する。

- GUI State Store全体の無制限Debug dump
- Secret StateのSession保存
- Secret DraftのDiagnostic表示
- Drag payload内容の無断Telemetry送信
- Native Session handleのSerialization

#### 49. Budget
GUI State Budget
GuiStateBudget {
  max-state-count,
  max-total-bytes,
  max-retained-state,
  max-per-view,
  max-per-component
}

Reconciliation Budget
ReconciliationBudget {
  max-elements,
  max-depth,
  max-key-comparisons,
  max-state-migrations,
  max-mounts,
  max-unmounts,
  max-native-operations,
  max-preparation-memory,
  max-commit-steps
}

Interaction Budget
InteractionBudget {
  max-active-interactions,
  max-buffered-events,
  max-draft-bytes,
  max-drag-payload-size,
  max-gesture-candidates,
  max-session-duration?,
  max-rebase-steps
}


Budget超過時には、半端なGUI状態をcommitせず、安全にRollbackまたはCancelする。

#### 50. Failure
GUI State Failure
GuiStateFailureKind =
  DuplicateKey
  | MissingStableKey
  | StateSchemaMismatch
  | StateMigrationFailed
  | InvalidStateTransfer
  | OwnerMissing
  | FocusConflict
  | StaleTextPosition
  | RetentionBudgetExceeded
  | CleanupFailed
  | ForbiddenPersistence
  | PrivacyViolation

Reconciliation Failure
GuiReconciliationFailureKind =
  DuplicateKey
  | MissingRequiredKey
  | OwnerMismatch
  | StateSchemaMismatch
  | StateMigrationFailed
  | InvalidMove
  | CapabilityBoundaryViolation
  | FocusConflict
  | PointerCaptureConflict
  | ImeTransferInvalid
  | PrepareFailed
  | CommitFailed
  | CleanupFailed
  | BudgetExceeded

Interaction Failure
InteractionFailureKind =
  OwnerUnavailable
  | TargetUnavailable
  | RevisionConflict
  | RebaseFailed
  | FocusConflict
  | PointerCaptureConflict
  | GestureResolutionFailed
  | DropRejected
  | CapabilityDenied
  | NativeSessionFailed
  | CommitFailed
  | CleanupFailed
  | BudgetExceeded


これらはOPEN-ERR-DIAG-001に従って構造化Diagnosticへ変換する。

#### 51. Defect

次は通常FailureではなくDefectとする。

- 一Pointerに複数の排他的Capture Ownerが存在する
- Focus Scope内にFocus Ownerが複数存在する
- Disposed InstanceへEventが配送された
- State Schema不一致を検査せず再利用した
- Completed Interactionを再度commitした
- Rebase失敗後に古いText Revisionへcommitした
- Secret Stateを通常Fieldへ移送した
- Modal境界外へFocusが漏れた
- Validation済みPlanに重複Ownershipがある
- Component Taskが別Ownerへ暗黙移送された


Defect Scopeに応じて、Interaction、Component Subtree、View、GUI Runtimeを隔離する。

#### 52. Diagnostic

GUI Diagnosticの例：

Primary:
動的Collection内でGUI Keyが重複しています

Related:
最初の要素の生成元
二つ目の要素の生成元

Suggestion:
Document NodeのStable IDをKeyとして指定してください


State Reset、Draft喪失、Focus異常、Interaction Cancellation、Cleanup Failure等の利用者影響がある事象をDiagnosticへ投影する。

Derived Cacheの通常破棄を逐一Warningにしない。

#### 53. Reconciliation Report
GuiReconciliationReport {
  plan-id,
  source-revision,
  target-revision,

  retained-instances,
  mounted-instances,
  moved-instances,
  suspended-instances,
  unmounted-instances,
  reset-states,
  migrated-states,

  focus-change,
  cancelled-interactions,
  task-scope-changes,
  cleanup-results,

  diagnostics,
  outcome
}

#### 54. 実装責任境界
Document Model
- Stable Node ID
- Document Revision
- Text Revision
- Transaction Position Mapping
- Selection対象の意味identity

GUI Runtime
- State Store
- Key Validation
- Reconciliation
- Component Lifecycle
- Focus
- Interaction ownership
- Retention
- Cleanup

Component
- Stable local Key
- State Schema
- State Migration
- Lifecycle requirement
- Interaction contract

Host
- Native Window
- IME
- Pointer device
- Clipboard
- Drag-and-drop service
- Accessibility service
- Session State storage

Diagnostic基盤
- GUI Failure／Defectの構造化Report
- Privacy
- Fingerprint
- Attachment
- Lifecycle

#### 55. 適合試験

少なくとも次を検査する。

- List先頭へItemを追加しても既存ItemのStateが維持される
- 並べ替え後もStable IDへStateが追従する
- 同じ位置の別Nodeへ旧Stateが移らない
- Duplicate KeyをCommit前に検出する
- Missing Keyを動的Collectionで報告する
- Widget kind変更で非互換Stateをresetする
- Secret Fieldへ通常Text Stateを移送しない
- Node MoveでStateとSelectionを維持できる
- Node Replaceで旧Interactionをcancelする
- Validation Failure時に旧GUIを維持する
- Mount TaskがCommit前に開始されない
- Commit中のEventが旧treeへ誤配送されない
- Component Unmount時にTaskとSubscriptionを終了する
- SuspendでStateを保持し可視Resourceを解放できる
- Resume時にCapabilityとRevisionを再検査する
- Focus OwnerがScope内で一つである
- Accessibility FocusとKeyboard Focusを別管理できる
- Text Insert後にCaretをAffinityどおりRebaseする
- Text Revision不一致時に古い位置へcommitしない
- IME commitがDocument Transactionになる
- IME Owner削除時にCompositionをcancelする
- Pointer Capture conflictを検出する
- Gesture競合を決定的に解決する
- Drop失敗時にDocumentを部分変更しない
- Modal中にFocusが外部へ漏れない
- Draftを黙ってEvictしない
- Secret StateをSession保存しない
- Budget超過時に半端な状態をcommitしない

#### 56. 性能要件
PERF-GUI-STATE-01:
変更SubtreeだけをReconcileできる

PERF-GUI-STATE-02:
Stable Key一致時にStateとInstanceを再利用できる

PERF-GUI-STATE-03:
位置変更だけでWidget Stateを再生成しない

PERF-GUI-STATE-04:
Virtualized ItemのStateをData identityで復元できる

PERF-GUI-STATE-05:
Derived Stateを優先的にEvictできる

PERF-GUI-STATE-06:
複数更新要求を安全にcoalesceできる

PERF-GUI-STATE-07:
Commit barrierを短時間に限定する

PERF-GUI-STATE-08:
Interaction Eventを必要な範囲でcoalesceできる

PERF-GUI-STATE-09:
State Store全体を走査せずOwner単位でcleanupできる

PERF-GUI-STATE-10:
Reconciliation結果をThread schedulingへ依存させない

#### 57. 最終状態

```text
OPEN-GUI-STATE-001:
RESOLVED
```

本決定により、ReciplexaのGUI State基盤は次の性質を持つ。

- Document StateとGUI Stateを分離する
- Stable KeyによってState ownerを追跡する
- 動的Collectionで位置Indexをidentityにしない
- Key、Owner、Schema、Revision、Privacyを検査してStateを継承する
- Node MoveとNode Replaceを区別する
- 別identity間のState移送を明示的に行う
- GUI更新をPlanning、Validation、Commitへ分離する
- Commit前に長寿命副作用を開始しない
- Component TaskとResourceをLifecycleへ所属させる
- SuspendとUnmountを区別する
- Focus、Selection、IME、Pointer CaptureをInteraction Sessionとして管理する
- Text PositionをTransaction後にRebaseする
- Gesture競合を決定的に解決する
- Drag-and-drop結果をDocument Transactionとしてcommitする
- Secretや一時Interactionを永続化しない
- Reconciliation Failure時に旧GUIを維持する
- Cleanup不能やRuntime不整合をDefectとして扱う

### OPEN-NATIVE-PKG-001 Native Package・実装選定・Lifecycle・Fallback・適合性
#### DD-NATIVE-PKG-001 決定文書
##### 状態
Status:
RESOLVED

Scope:
- Native packageの独立単位
- Portable Contractとの関係
- Contract PackageとImplementation Package
- 実装選定Policy
- Native Implementation Descriptor
- Package・Contract・ABI Version
- Artifact discovery
- Candidate validation
- ABI negotiation
- Transactional initialization
- Native instance lifecycle
- Sharing・Multiplicity・Lifetime
- Capability・Executor・Callback
- Shutdown・Quarantine
- Failure分類
- Fallback
- Conformance Test
- Differential Test
- Conformance evidence

#### 0. 基本原則

Reciplexaでは、Native implementationを言語の公開意味の正本にはしない。

公開される契約の正本は、RPXで記述されたPortable Contractとする。

Portable Contract:
型
Effect
Failure
Ownership
Resource lifetime
Concurrency
Cancellation
Determinism
Capability requirement
Semantic invariant


Native implementationは、このContractを満たす交換可能なImplementation Providerとして扱う。

Application／Library
↓
Portable Contract
↓
Runtime Profileによる実装選定
├─ Portable implementation
├─ Native implementation
├─ Worker implementation
└─ Host service


Applicationは原則として、特定のRust Library、C Library、GPU Driver、OS APIへ直接依存しない。

Native ABIや外部Library固有の型はTrusted Adapterの内部へ閉じ込める。

Rustでも、外部関数のABIはexternで明示され、extern "C"等による外部境界が区別される。また、symbol exportやcalling conventionもABI設計上の明示事項である。これは、言語内部の型・呼出し規約をそのまま外部契約にしないという本設計と整合する。

#### 1. Native packageの役割

Native packageは、次のいずれかを目的として導入される。

- Portable実装より高い性能を得る
- OS固有機能を利用する
- GPUや専用Deviceを利用する
- 既存Native Libraryを再利用する
- Codec、Font、暗号等の専門実装を利用する
- Host serviceと接続する


しかし、Nativeであるという理由だけで複数機能を一つの巨大Packageへまとめてはならない。

たとえば次の構造は原則として採用しない。

reciplexa-native-runtime
├─ Font
├─ Image Codec
├─ Video
├─ PDF
├─ GPU
├─ Filesystem
├─ Clipboard
└─ Crypto


この構造では、更新、Capability、ABI、Failure、Isolation、Shutdownの境界が大きすぎる。

Native implementationは、独立して選択、Version解決、初期化、停止、隔離、Quarantineできる単位へ分割する。

#### 2. Native packageの独立単位

次のいずれかが異なる場合、別Native packageとすることを原則とする。

- 提供するContract family
- 外部Library
- Native ABI
- Capability requirement
- Isolation requirement
- Failure domain
- Version lifecycle
- Platform／Device requirement
- Instance multiplicity
- Resource ownership
- Shutdown method
- Portable fallbackの有無


独立Packageの候補例は次である。

- Font shaping
- Image decode
- Video encode
- PDF emission
- GPU rendering
- Filesystem watcher
- OS clipboard
- Cryptographic signing


一方、同じContract family、ABI、Capability、Isolation、Runtime instanceを共有する操作は、一つのImplementation Packageへまとめられる。

例としてFont engineが同一instanceを共有する場合、次は一つのPackageに含められる。

Font Native Implementation
├─ Font face load
├─ Glyph metadata
├─ Text shaping
└─ Glyph outline


Native関数一つごとにPackageを分割する必要はない。

#### 3. Contract PackageとImplementation Package

基本構成は次とする。

Contract Package
├─ Public type
├─ Effect contract
├─ Failure contract
├─ Ownership contract
├─ Semantic invariant
└─ Conformance suite

Implementation Packages
├─ Portable implementation
├─ Native implementation A
├─ Native implementation B
└─ Worker／Host implementation


Contract PackageとImplementation Packageは原則として分離する。

これにより、次が可能になる。

- Native実装だけを独立更新する
- Platform別実装を追加する
- Portable-only環境からNative依存を除く
- Native packageを別Processへ隔離する
- 複数実装をDifferential Testする
- 公開型をNative ABIから独立させる


十分に小さく、一体性が高く、独立選択や独立更新の利益がない場合に限って、同一Package内の別Implementation memberを許可できる。

#### 4. Portable Contract

Native implementationが満たすContractを、概念的に次のように定義する。

NativeContract {
  contract-identity,
  contract-version,

  input-types,
  output-types,

  effects,
  failures,

  determinism,
  replayability,

  concurrency,
  reentrancy,
  cancellation,

  ownership,
  resource-lifetime,

  capability-requirements,
  semantic-invariants,

  equivalence-contract
}


Native implementationは、公開型、正常結果だけでなく、次についてもContractへ適合しなければならない。

- Failureの種類
- Cancellation
- Partial result
- Resource cleanup
- Callback回数
- Thread affinity
- Reentrancy
- Determinism
- Budget
- Observable Effect


Status codeだけが同じでも、Native Resourceを漏らした実装はContract適合とはみなさない。

#### 5. Portable implementationの必要性

すべてのContractにPortable実装を要求することはしない。

OS固有機能、Hardware固有機能、外部Service等では、Portable実装が意味を持たない場合があるためである。

Contractごとに次を宣言する。

PortabilityClass =
  PortableRequired
  | PortablePreferred
  | NativePermitted
  | NativeRequired
  | HostSpecific

Portable Required

言語の基礎意味、再現性、検証にPortable実装が必要である。

Portable Preferred

Portable実装を持つことを強く推奨するが、特定構成では省略できる。

Native Permitted

PortableとNativeの双方を許可する。

Native Required

Native機能なしではContractを実現できない。

Host Specific

Clipboard、Window、Accessibility Service等、Hostが実装を提供する。

Portable実装がある場合でも、名目的な低品質Fallbackを用意するだけでは不十分である。Portable実装も同じContractと品質条件を満たさなければならない。

#### 6. 実装選定Policy

TargetまたはRuntime Profileは、Contractごとに実装選定Policyを指定する。

ImplementationSelectionPolicy =
  RequireNative
  | PreferNative
  | PreferPortable
  | PortableOnly
  | RequireSpecificImplementation

Require Native

適合するNative実装がなければ、Profile Resolutionまたは起動を失敗させる。

Portable実装へ暗黙Fallbackしない。

Prefer Native

適合するNative実装を優先し、利用不能な場合はPortable実装を候補にできる。

Native実行途中のFailureからFallbackできるかは、別のFallback Policyで判断する。

Prefer Portable

移植性、決定性、Test容易性を優先する。

Native実装は、Portableで必要条件を満たせない場合に利用する。

Portable Only

Native codeを候補から除外する。

高信頼Sandbox、Test Runtime、Compatibility検査等で使用する。

Require Specific Implementation

特定Implementation Identityを要求する。

次のような限定用途で使用する。

- 再現可能Build
- ABI検証
- Differential Test
- 認証済み暗号実装


通常Applicationが具体実装へ直接依存する既定にはしない。

#### 7. 実装選定の二段階化

Implementation選定を、Build段階とRuntime段階へ分ける。

Build／Package Resolution
↓
利用可能なImplementation PackageとArtifactを固定

Runtime Profile Resolution
↓
現在のPlatform、Device、Capabilityからinstanceを選択

Build段階
- Implementation Packageを解決
- Package Release Identityを固定
- Binary Artifactを準備
- Descriptorを検証
- Native ABI metadataを検査
- Lockfileへ記録

Runtime段階
- Device capabilityを検査
- Capability grantを検査
- Isolation条件を検査
- Conformance evidenceを検査
- Multiplicityを検査
- Resource Budgetを割り当て
- 最終Implementationを選択


Job開始時に選択結果をSnapshot化し、Job中に暗黙変更しない。

#### 8. Implementation Selection Snapshot
ImplementationSelectionSnapshot {
  contract-identity,
  contract-version,

  selected-implementation,
  package-instance-id,
  implementation-version,
  native-abi-version?,

  platform-identity,
  device-identity?,

  capability-view,
  isolation-boundary,
  conformance-status,

  fallback-plan,
  snapshot-revision
}


同じJob中にDevice負荷、Filesystem順、Library discovery順によって選択を変えてはならない。

#### 9. Versionの分離

次を別々のVersionとして扱う。

PackageVersion
ContractVersion
ImplementationVersion
NativeAbiVersion
AdapterContractVersion
BinaryArtifactIdentity

Package Version

Implementation Packageの配布Releaseを表す。

Contract Version

RPX上の型、Effect、Failure、Ownership、意味契約のVersionを表す。

Implementation Version

Native実装固有のVersionを表す。

Native ABI Version

Native Binaryとの低水準Binary interfaceを表す。

Adapter Contract Version

Trusted AdapterとReciplexa Runtime間のprotocol Versionを表す。

Binary Artifact Identity

具体的にBuildされたBinary contentを表す。

Package Versionが同じでも、Platform、Compiler、Linker、依存Libraryが異なれば別Binary Artifactになり得る。

#### 10. Native Implementation Descriptor

Native implementationは、静的なDescriptorを持つ。

NativeImplementationDescriptor {
  implementation-identity,
  package-release-identity,

  provided-contracts,
  implementation-version,

  native-abi-range,
  adapter-contract-range,

  platform-requirements,
  device-requirements,
  capability-requirements,

  isolation-support,
  instance-multiplicity,
  instance-lifetime,
  instance-sharing,

  executor-requirement,

  determinism,
  replayability,
  equivalence-class,
  fallback-support,

  resource-budgets,
  failure-contract,
  cancellation-contract,
  shutdown-contract,

  conformance-status,
  known-limitations,
  provenance
}


Descriptorは、可能な限り実行Binaryから分離した静的Dataとして配布する。

これにより、BinaryをProcessへ読み込む前に候補を検証できる。

Binary load後にRuntime Handshake Descriptorを取得する場合、静的Descriptorとの一致を検証する。

#### 11. Registryの分離

Registryを次の三層へ分ける。

Implementation Catalog
Validated Candidate Registry
Runtime Instance Registry

Implementation Catalog

利用可能かもしれない実装の静的情報を保持する。

Binary codeはまだ実行しない。

Validated Candidate Registry

Descriptor、Content identity、Platform、Trust等を検証済みの候補を保持する。

可能な限り、まだBinaryを読み込まない。

Runtime Instance Registry

ABI negotiationと初期化を完了し、Ready状態にあるinstanceだけを保持する。

Applicationへ公開できるのは、原則としてRuntime Instance Registry内のReady instanceだけである。

#### 12. Artifact discovery

Native Artifactは、次の確定されたSourceから発見する。

- Lockfileで固定されたArtifact
- Content-addressed Package Store
- Toolchain built-in Artifact
- 明示されたWorkspace implementation
- Host-managed implementation


次のような偶然の探索結果だけで採用してはならない。

- Current directoryに同名Libraryがある
- Environment pathで最初に見つかった
- Filesystem列挙で先に現れた


DiscoveryとBinary loadを分離する。

Artifact Discovery
↓
Descriptor Validation
↓
Candidate Selection
↓
Binary Load

#### 13. Descriptor Validation

Candidate登録前に次を検証する。

- Package Release Identity
- Implementation Identity
- Binary Content Identity
- Contract Identity
- Contract Version range
- Native ABI range
- Adapter Contract range
- Platform／Architecture
- Required system library
- Capability requirement
- Isolation requirement
- Instance multiplicity
- Shutdown contract
- Conformance evidence


結果を次へ分類する。

DescriptorValidation =
  Valid
  | ValidWithRestrictions
  | Unsupported
  | Untrusted
  | Invalid


ValidWithRestrictionsでは、利用可能条件をCandidate Selectionへ伝える。

- 特定Deviceだけで利用可能
- Sandbox内だけで利用可能
- Previewだけで利用可能
- 最大入力sizeに制限

#### 14. Candidate ordering

複数候補は、次の辞書式優先順位で選択する。

1. Hard Constraintを満たす
2. Lockfile／Build Planの固定選択
3. Selection Policyへの適合
4. Security／Isolation Policyへの適合
5. Conformance Level
6. 健全な既存instanceの再利用可能性
7. Platform／Device適合度
8. Resource Cost
9. Stable Implementation Identity順


登録順、Library load順、Filesystem順、Thread完了順へ依存させない。

#### 15. Binary LoadとIsolation

BinaryをProcessへ読み込むのは、Candidate選定後に限定する。

Validated Candidate
↓
Isolation Boundary準備
↓
Binary Load
↓
Handshake


Isolation方式を次へ分類する。

NativeIsolation =
  InProcess
  | DedicatedThread
  | WorkerProcess
  | SandboxProcess
  | HostService


Native implementationは対応可能方式と最低条件を宣言する。

IsolationRequirement =
  InProcessAllowed
  | DedicatedThreadRequired
  | ProcessIsolationRequired
  | SandboxRequired
  | HostManaged


未信頼入力を扱うCodec、Memory safety保証が弱いLibrary、強制停止が必要な機能は、ProcessまたはSandbox隔離を優先する。

#### 16. ABI Negotiation

Binary load後、Trusted AdapterとHandshakeを行う。

AbiNegotiationInput {
  runtime-adapter-range,
  required-contracts,
  platform-identity,
  pointer-width,
  endianness,
  calling-convention,
  ownership-contract,
  callback-contract
}


結果：

AbiNegotiationResult =
  Compatible
  | VersionMismatch
  | MissingContract
  | OwnershipMismatch
  | CallbackMismatch
  | PlatformMismatch
  | InvalidHandshake


整数Version範囲が重なるだけでは互換とみなさない。

次も照合する。

- Data layout
- Alignment
- Ownership transfer
- Failure representation
- Callback lifetime
- Thread affinity
- Cancellation protocol


成功した結果はNegotiatedAbiSnapshotとしてinstance lifetime中固定する。

#### 17. Transactional Initialization

Native instanceの初期化を原子的な状態遷移として扱う。

Validated Candidate
+
Runtime Profile View
+
Capability Grants
+
Budget Slice
↓
Initialization Transaction
↓
Ready Instance


基本段階：

1. Isolation Boundary準備
2. Binary Load
3. ABI Negotiation
4. Capability View作成
5. Native State確保
6. Callback Table登録
7. Health Check
8. Contract Probe
9. Ready Commit


途中Failureでは、取得済みResourceを逆順にrollbackする。

初期化中のOperationを次へ分類する。

InitializationOperationClass =
  PureValidation
  | ReversibleAllocation
  | CompensatableRegistration
  | IrreversibleExternalEffect


不可逆な外部作用を通常初期化に含めることは避ける。

#### 18. Native Instance Lifecycle
NativeInstanceLifecycle =
  Discovered
  | Validated
  | Loading
  | Negotiating
  | Initializing
  | Ready
  | Degraded
  | Quiescing
  | ShuttingDown
  | Disposed
  | Quarantined
  | Defected


Runtime Instance Registryへ公開できるのは、原則としてReadyだけである。

Degradedは、Descriptorで限定動作が明示されている場合だけ利用できる。

QuarantinedになったinstanceまたはImplementationを、新しいJobへ選択しない。

#### 19. Instance所有権・Lifetime・共有

Instance ownerを次へ分類する。

NativeInstanceOwner =
  Operation
  | Job
  | Document
  | RootScope
  | RuntimeInstance
  | ProcessHost
  | HostService


Instance lifetimeも同じ境界に関連付ける。

NativeInstanceLifetime =
  Operation
  | Job
  | Document
  | RootScope
  | RuntimeInstance
  | Process
  | Host


共有契約を次へ分類する。

InstanceSharing =
  Exclusive
  | SharedReadOnly
  | SharedSerialized
  | SharedConcurrent
  | Partitioned


単なるthread-safeという一語だけで、共有、Partition、Reentrancy、Orderingを表現しない。

#### 20. Instance Multiplicity
InstanceMultiplicity =
  Multiple
  | SingletonPerDevice
  | SingletonPerRuntime
  | SingletonPerProcess
  | HostSingleton


例：

Image decoder:
Multiple

GPU context:
SingletonPerDevice候補

GUI event loop:
SingletonPerProcess候補

OS clipboard:
HostSingleton


Singleton制約と複数Version要求が競合する場合、Package ResolutionまたはRuntime Profile ResolutionでFailureにする。

#### 21. Capability View

Native instanceへRuntime Profile全体を渡さない。

必要なCapabilityだけを縮小して渡す。

NativeCapabilityView {
  resource-capabilities,
  operation-capabilities,
  lifetime,
  owner-runtime,
  revocation-channel
}


強いCapabilityを持つinstanceを、異なる権限範囲のJobへ無条件共有しない。

Capabilityがrevokeされた場合、新Operationを拒否し、必要に応じてQuiescingへ移行する。

#### 22. Executor Affinity
NativeExecutorRequirement =
  Any
  | Main
  | DedicatedSerial
  | DedicatedParallel
  | DeviceExecutor
  | HostManaged


Native OperationとCallbackは、Negotiated contractで指定されたExecutorへ従う。

誤ったExecutor上のCallbackはContract Violationとして扱う。

#### 23. OperationとCallback

進行中OperationをRuntime側でも追跡する。

NativeOperationRecord {
  operation-id,
  instance-id,
  owner-scope,
  input-snapshot-id,
  cancellation-state,
  completion-state,
  callback-generation,
  resource-borrows
}


Callback受信時に次を検査する。

- Instanceが有効
- Callback generationが一致
- Owner Scopeが生存
- Operationが未完了
- Executorが正しい
- Payload validatorに適合


Completion Callbackの二重到着はDefectとする。

Shutdown後のLate Callbackは、Callbackの性質に応じて記録、無視、Quarantine、Defectへ分類する。

#### 24. Shutdown

Shutdownを次の順序で行う。

1. Registryから新規取得を停止
2. 新しいOperationを拒否
3. Active operationをQuiesceまたはCancel
4. Completionを待つ
5. Callback routingを閉鎖
6. Native Resourceを解放
7. Worker／Threadを停止
8. Disposedへ移行


概念Contract：

NativeShutdownContract {
  begin-quiesce,
  cancel-operation,
  await-drain,
  unregister-callbacks,
  release-resources,
  finalize,
  deadlines,
  forced-termination
}


In-process Native codeを安全に強制停止できない場合がある。

強制停止が要求されるContractであれば、WorkerまたはSandbox隔離を要求する。

#### 25. Dynamic LoadとUnload

Native implementationのLinkageを次へ分類する。

NativeLinkage =
  Static
  | Dynamic
  | WorkerExecutable
  | HostService


Instance disposalとLibrary imageの物理Unloadを分離する。

Instance disposal:
必須

Dynamic library unload:
安全が証明された場合だけ


v1では、Dynamic LibraryをProcess終了まで保持することを許可する。

Quarantined Binaryは新instance生成へ使用しない。

#### 26. Hot Replacement

v1では、実行中instanceの一般的なHot replacementを採用しない。

理由：

- ABIが変わり得る
- Native Stateを移送できない
- 旧Callbackが残る
- Function pointerが旧Binaryを参照する
- Jobの再現性が崩れる


既存instanceは旧実装で終了まで動作するか、安全上必要ならCancelする。

新しいImplementationは、新しいJobまたはRuntimeから選択する。

#### 27. Failure分類

Native実行結果を次へ分類する。

NativeExecutionOutcome<E, A> =
  Completed(A)
  | ProgramFailed(E)
  | Cancelled
  | ImplementationUnavailable
  | ImplementationFailed
  | Defected
  | Aborted

Program Failure

Portable Contractに定義された通常Failureである。

Native implementationが壊れたことを意味しない。

Implementation Unavailable

処理開始前に利用条件を満たせない。

PreferNativeではPortable候補を選択できる。

Implementation Failure

Runtime Trustを維持したまま発生した実装・環境側のFailureである。

CleanupとRollbackが完了すればFallback候補となる。

Contract Violation

Native実装がPortable Contractへ違反した。

Defectとして扱い、Quarantineへ接続する。

Terminal Failure

Memory safetyやRuntime健全性を保証できない。

同一Process内でPortableへFallbackしない。

#### 28. Fallbackの基本原則

Fallbackは単なる例外処理ではない。

次をすべて満たす場合だけ許可する。

- Runtime Trustが維持されている
- Native instanceのCleanupが完了している
- 旧出力のRollbackが完了している
- 同じ入力を再現できる
- Operationが再実行可能
- 代替Implementationが利用可能
- 外部Effectを重複させない
- Fallback Budget内である


Job途中の内部Stateを別Implementationへ暗黙移送しない。

基本となるFallback方式は、現在Jobをrollbackし、固定Input Snapshotから別Implementationで最初から再実行する方式である。

#### 29. Operation Replayability
OperationReplayability =
  PureReplayable
  | SnapshotReplayable
  | IdempotentEffect
  | TransactionallyReplayable
  | RequiresIdempotencyKey
  | NonReplayable
  | Unknown


NonReplayableまたはUnknownでは自動Fallbackしない。

Artifact emissionのようにOutput Transactionで完全Rollbackできる処理は、TransactionallyReplayableにできる。

Network送信や不可逆なDevice操作は、明示的なIdempotencyまたはTransaction契約がなければ再実行しない。

#### 30. Fallback Input Snapshot

Fallback時に結果へ影響する入力を固定する。

NativeJobInputSnapshot {
  typed-input,
  document-revision?,
  resource-snapshot,

  clock-snapshot?,
  random-seed-or-stream?,

  locale?,
  font-environment?,
  color-environment?,

  output-profile?,
  capability-view,
  relevant-configuration
}


Fallback中にClock、Random、Resource、Font、Output Profileを最新状態へ変更しない。

代替Implementationへ、元Implementationより強いCapabilityを自動付与しない。

#### 31. RollbackとCleanup

次を分離して検査する。

Instance Cleanup:
Native Resource、Callback、Taskの終了

Output Rollback:
File、Artifact、Upload等の部分結果破棄


Rollback状態：

RollbackStatus =
  NotNeeded
  | Completed
  | PartiallyCompleted
  | Failed
  | Unknown


自動Fallbackを許可するのは原則としてNotNeededまたはCompletedの場合だけとする。

#### 32. Fallback Plan

Fallback経路をJob開始前に構築する。

NativeFallbackPlan {
  primary-implementation,
  alternative-implementations,

  eligible-failure-classes,
  replayability-requirements,

  snapshot-requirements,
  rollback-contract,
  cleanup-contract,

  restart-scope,
  maximum-attempts,
  reporting-policy
}


Fallback試行数にはBudgetを設ける。

FallbackBudget {
  max-attempts,
  max-restarts,
  max-total-cost,
  max-worker-restarts
}


Implementationを順に無制限試行しない。

#### 33. Fallbackの再開始範囲
FallbackExecution =
  RestartOperation
  | RestartJob
  | RestartRootScope
  | RestartWorker
  | RestartProcess
  | NoFallback


同じOperationの途中から継続するCheckpointHandoffは、Portable Checkpoint SchemaがContract化されている特殊な場合に限る。

Worker crashではWorkerを破棄し、Jobを先頭から再実行できるか判断する。

Terminal FailureではProcessまたはHost境界からの再開始を検討する。

#### 34. Quarantine

Contract Violation、Crash、Ownership不一致等が発生したImplementationまたはinstanceをQuarantineできる。

QuarantineRecord {
  implementation-identity,
  instance-id?,
  reason,
  scope,
  runtime-revision,
  evidence,
  retry-policy
}

QuarantineScope =
  Instance
  | Device
  | RuntimeInstance
  | Process
  | PackageRelease
  | BinaryArtifact


Memory safety疑い、Contract Violation、Binary改ざんでは、同じRuntime内の自動再試行を行わない。

一時的Device Failureでは、Host Policyにより別Deviceや新instanceを試せる。

#### 35. 同値性

PortableとNativeの結果を比較する契約を次へ分類する。

ConformanceEquivalence =
  Exact
  | CanonicalEquivalent
  | ToleranceBounded
  | ObservationalEquivalent
  | TraceEquivalent
  | ProfileDependent

Exact

型付き結果が完全に一致する。

Canonical Equivalent

正規化後の結果が一致する。

Tolerance Bounded

規定Metricと最大誤差の範囲で一致する。

Observational Equivalent

上位Contractから観測可能な値、Failure、Effectが一致する。

Trace Equivalent

Callback、Resource、Cancellation等のProtocol traceが許容Partial order内で一致する。

#### 36. FailureとCancellationの互換性

Conformanceは正常値だけを対象としない。

次を比較する。

- Failure kind
- Failure payloadの規範部分
- Retry classification
- Partial result
- Cleanup状態
- Cancellation point
- Callback closure


Native OS Errorは、AdapterによってContract-defined Failureへ変換する。

OS固有詳細はRelated Diagnosticとして保持できるが、公開Failure型へ漏らさない。

#### 37. Conformance Test

Conformance Testは、ImplementationをContractへ対して検査する。

NativeConformanceSuite {
  contract-identity,
  contract-version,
  suite-version,

  deterministic-cases,
  property-tests,
  failure-cases,
  cancellation-cases,
  ownership-cases,
  concurrency-cases,
  lifecycle-cases,
  budget-cases,
  isolation-cases,

  required-platform-classes,
  equivalence-contract,
  reporting-policy
}


Portable実装が存在しなくても実行できる。

#### 38. Differential Test

Differential Testは複数Implementationを互いに比較する。

同じInput Corpus
├─ Portable implementation
├─ Native implementation A
└─ Native implementation B
↓
Contract-defined equivalenceで比較


両Implementationが同じ誤りを持つ可能性があるため、Differential TestだけでContract適合を証明しない。

Conformance TestとDifferential Testを併用する。

#### 39. Test対象

少なくとも次を検査する。

- 正常結果
- 境界値
- 不正入力
- Failure分類
- Cancellation
- Resource cleanup
- Ownership
- Callback回数
- Reentrancy
- Concurrency
- Shutdown race
- Budget
- Worker crash
- Panic containment


Property Testの反例はOPEN-TST-001のShrinkingとAttachment機構へ接続する。

#### 40. Differential Test Corpus
DifferentialCorpus =
  NormativeCases
  | GeneratedCases
  | RegressionCases
  | RealWorldSanitizedCases
  | BoundaryCases
  | AdversarialCases


User Contentや機密文書を無断でCorpusへ保存しない。

実データから反例を採取する場合、Privacy-safeな最小反例へ縮小するか、明示Retention Policyを適用する。

#### 41. Known Limitation

Contractの一部条件だけを満たすImplementationは、制限を構造化して宣言する。

KnownLimitation {
  limitation-id,
  affected-operations,
  conditions,
  behavior,
  fallback-availability,
  diagnostic-code,
  conformance-evidence
}


Known LimitationはDocumentationだけでなく、Candidate SelectionのHard Constraintとして使用する。

制限条件外でImplementationを選択してはならない。

#### 42. Conformance Status
ConformanceStatus =
  Declared
  | Tested
  | Certified
  | KnownLimited
  | Regressed
  | Disabled


Certifiedは特定のCertification Policyを満たすことを意味し、絶対的な正しさを意味しない。

RegressedまたはDisabledのImplementationを通常の新Jobへ選択しない。

#### 43. Conformance Evidence
ConformanceEvidence {
  implementation-identity,
  binary-artifact-identity,

  contract-version,
  suite-version,

  platform-identity,
  device-class?,
  runtime-adapter-version,

  result,
  executed-cases,
  skipped-cases,

  report-identity
}


EvidenceはBinary Artifact、Platform、Suite Versionへ結び付ける。

Sourceが同じでもBinaryが変われば、原則としてEvidenceを再評価する。

#### 44. Runtime Contract Monitor

実行時にも低CostのContract検査を行える。

- Completionが高々一回
- Enum tagが有効
- Sizeが宣言上限内
- Callback generationが一致
- Owner Scopeが生存
- Resource countがBudget内


Runtime Monitorが違反を検出した場合、Program FailureではなくDefectとして扱う。

高Costな検査はTestまたはDebug Profileに限定できる。

#### 45. Diagnostic

Native package関連のFailure、Fallback、Quarantine、Contract Violationは、OPEN-ERR-DIAG-001の構造化Diagnosticへ変換する。

例：

Primary:
Native image decoderを初期化できません

Cause:
Native ABIがRuntime Adapterと適合しません

Required:
Adapter Contract 3以上4未満

Provided:
Adapter Contract 2

Alternative:
Portable decoderを利用可能


Fallbackで最終的に成功しても、Contract Violation、Worker crash、Quarantineを隠してはならない。

Native backtrace、Binary Path、Device identityにはPrivacy Policyを適用する。

#### 46. Reproducible Build

再現可能Buildでは、Implementation選択を原則として固定する。

- Implementation Identity
- Binary Artifact Identity
- ABI Version
- Adapter Version
- Selection Snapshot


固定Implementationが利用不能なら、Strict ProfileではFailureとする。

Fallbackを許可する場合は、実際に選ばれたImplementationとFallback経路をBuild Reportへ記録し、Build identityへ反映する。

#### 47. 適合試験

少なくとも次を検査する。

- 未選択BinaryをProcessへ読み込まない
- Descriptor不正候補を登録しない
- Binary identity不一致を検出する
- ABI不適合時にinstance初期化を開始しない
- Ready前のinstanceをRegistryへ公開しない
- 初期化途中FailureでResourceをrollbackする
- Singletonの二重作成を防ぐ
- Capabilityの強いinstanceを無制限共有しない
- Owner Scope終了後にHandleを使用できない
- Completion Callbackの二重到着をDefectとする
- Shutdown後のLate Callbackを検出する
- Quarantined Implementationを再選択しない
- Dynamic instance disposalとBinary unloadを区別する
- Contract上のInvalidInputで不要なFallbackを行わない
- RequireNativeがPortableへFallbackしない
- Runtime Trust喪失後に同一ProcessでFallbackしない
- Cleanup未完了時に別実装を開始しない
- Rollback失敗時にJobを再実行しない
- Clock、Random、Resource SnapshotをFallbackで維持する
- NonReplayable Effectを自動再実行しない
- Known Limitation外でImplementationを選択しない
- Failure、Cancellation、Resource lifecycleをConformance対象にする
- Differential TestだけでCertifiedにしない
- Binary更新後に旧Evidenceを無条件再利用しない
- Regression検出後に新Jobの候補から除外する

#### 48. 実装責任境界
Contract Package
- Public Contract
- Failure
- Ownership
- Equivalence
- Replayability
- Conformance Suite

Implementation Package
- Native Binary
- Static Descriptor
- Adapter metadata
- Known Limitation
- Conformance metadata

Trusted Adapter
- ABI negotiation
- ForeignValue validation
- Ownership変換
- Callback validation
- Panic／Failure変換

Runtime Profile Resolver
- Candidate Selection
- Capability View
- Isolation Plan
- Fallback Plan
- Implementation Snapshot

Native Runtime
- Transactional Initialization
- Instance Registry
- Operation Tracking
- Callback Routing
- Shutdown
- Quarantine

Test Runtime
- Conformance Test
- Differential Test
- Resource Trace
- Regression Detection

#### 49. 性能要件
PERF-NATIVE-PKG-01:
未選択Binaryを読み込まない

PERF-NATIVE-PKG-02:
Validated DescriptorをCache可能にする

PERF-NATIVE-PKG-03:
健全なinstanceを安全条件下で再利用できる

PERF-NATIVE-PKG-04:
CapabilityとDevice変更時にSelectionだけを再評価できる

PERF-NATIVE-PKG-05:
Contract不変のImplementation更新でRPX型検査を再利用できる

PERF-NATIVE-PKG-06:
Worker通信をBatch化可能にする

PERF-NATIVE-PKG-07:
Runtime Contract MonitorのCostをProfile別に制御できる

PERF-NATIVE-PKG-08:
Conformance EvidenceをBinary Artifact単位で再利用できる

PERF-NATIVE-PKG-09:
Fallback試行へBudgetを設定する

PERF-NATIVE-PKG-10:
Candidate SelectionをFilesystem順やThread順へ依存させない

#### 50. 最終状態

```text
OPEN-NATIVE-PKG-001:
RESOLVED
```

本決定により、ReciplexaのNative package基盤は次の性質を持つ。

- Portable Contractを公開意味の正本とする
- Native implementationを交換可能なProviderとして扱う
- ContractとImplementationを原則として別Packageにする
- Native機能を選択・更新・隔離可能な単位へ分割する
- Package、Contract、Implementation、ABI Versionを分離する
- 実装選定をBuild段階とRuntime段階へ分離する
- 未選択Binaryを不必要に読み込まない
- Descriptor、Binary、ABI、Capabilityを段階的に検証する
- Native instanceをTransactionalに初期化する
- Ready前のinstanceを公開しない
- Instanceに明確なOwner、Lifetime、Sharing契約を持たせる
- CallbackとOperationをRuntime側でも追跡する
- ShutdownをQuiesce、Drain、Callback解除、Resource解放へ分ける
- Contract ViolationをDefectとしてQuarantineする
- Job途中の暗黙実装切替を原則禁止する
- Fallback前にInput、Cleanup、Rollback、Runtime Trustを検証する
- Conformance TestとDifferential Testを分離する
- Failure、Cancellation、Ownership、Resource lifecycleも検証する
- EvidenceをBinary、Platform、Suite Versionへ関連付ける
- Native実装を利用できない環境でも、Policyに従ってPortable実装を選択できる

---

# 第IV部 実装時の遵守事項

## 1. 本書への追従

コードを本書の代替にしてはならない。重要な判断がコードにしか存在しない場合、その実装は完成していない。背景、目的、要件、外部仕様、判断理由、適合条件を本書へ反映してから確定する。

## 2. 未決定事項

未決定事項は暗黙実装で固定せず、背景、目的、要件、外部仕様、採用案、代替案、Failure、Identity、Version、Lifetime、Migration、Privacy、Budget、適合試験を記録して決定する。

## 3. 実装で問題が判明した場合

設計が実装不能、非効率、安全でない、または相互矛盾すると判明した場合、局所回避だけで済ませない。大域要件と外部仕様まで戻り、設計変更として記録する。

## 4. 今後の統合方針

今後の決定は本書へ直接統合し、別の`solve.md`を恒久的な正本として増やさない。
