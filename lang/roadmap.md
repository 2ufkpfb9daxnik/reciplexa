# reciplexa / RPX 実装ロードマップ

> **対象仕様**: `specification.md`  
> **位置付け**: 仕様を実装へ落とす順序、各段階の成果物、完了条件、依存関係を定める  
> **基本方針**: 言語基盤を先行させるが、言語全体の完成を待たず、早期に最小の双方向GUIを接続する

## 0. ロードマップの使い方

この文書は日付による予定表ではなく、依存関係に基づく実装順序である。各Phaseは、その完了条件を満たしてから次のPhaseの規範的基盤として利用する。

実装中に仕様と矛盾する問題が判明した場合は、コードだけで回避せず、`specification.md`を先に更新する。各Phaseの変更には、少なくとも次を含める。

```text
- 実装
- Unit Test
- 適合試験
- Diagnostic
- 必要な仕様更新
- 既知の制約
```

## 1. 全体戦略

reciplexaの本質は、文字によるRPX SourceとWYSIWYG GUIの双方向編集である。したがって、次の二つの極端な進め方は採用しない。

```text
避ける進め方A:
GUIを先に作り、後から言語へ接続する

問題:
GUI独自のDocument Stateが正本になり、Sourceへ戻せなくなる
```

```text
避ける進め方B:
言語、Compiler、Backendを完全に作り終えてからGUIを作る

問題:
Stable Identity、Provenance、Transaction、Sourceへの逆反映が
後付けになり、双方向編集に適さないIRが固定される
```

採用する順序は次である。

```text
言語の最小Kernel
↓
編集可能Document Model
↓
最小の双方向Vertical Slice
↓
Effect・Task・Memory管理
↓
段階化IRとBackend
↓
GUI本格化
↓
永続化・Package・Native・出力拡張
↓
最適化・形式的検証
```

**Vertical Slice（垂直断面）**とは、機能をSubsystemごとに横へ作り切るのではなく、Source入力からGUI表示、GUI操作、Source反映、保存、Testまでを最小機能で一度貫通させる実装単位である。

## 2. Phase 0: Repositoryと仕様適合基盤

### 2.1 目的

後続実装が、仕様、Test、Diagnostic、生成物へ一貫して接続できる土台を作る。

### 2.2 実装項目

- Rust WorkspaceとCrate責任境界
- Source位置、Stable ID、Revision、Package ID等の共通型
- `Outcome`、`Failure`、`Cancellation`、`Defect`の基礎型
- 構造化Diagnosticの最小Schema
- Test harness
- Golden fileではなく構造化結果を比較するTest utility
- 仕様節または適合試験IDとTestを対応付ける仕組み

### 2.3 推奨Crate境界

具体名は変更可能だが、責任は混ぜない。

```text
rpx-source       Source、Range、Syntax identity
rpx-syntax       Lexer、Parser、Syntax tree
rpx-core         Core IR、型付きBinding
rpx-types        Type、Effect、Constraint
rpx-runtime      Evaluator、Task、Capability
rpx-document     Editable Document、Transaction
rpx-gui          GUI Description、State、Interaction
rpx-ir           Domain／Visual／Motion／Render IR
rpx-backend      Planning、Emission、Verification
rpx-package      Manifest、Resolver、Target
rpx-native       Adapter、Registry、Instance lifecycle
rpx-diagnostic   Diagnostic schema、Renderer
rpx-test         Test runtime、Property test、Conformance
```

### 2.4 完了条件

- 共通Identity型が文字列や整数の乱用なしに利用できる
- FailureとDefectを別経路でTestできる
- DiagnosticをMachine-readableな構造として取得できる
- 最小のCIまたはLocal test commandで全Crateを検査できる

## 3. Phase 1: Source、構文、Binding

### 3.1 目的

RPX Sourceを安定して読み、位置とIdentityを失わずにSyntaxへ変換する。

### 3.2 実装順

1. UTF-8 Source ResourceとText Range
2. Lexer
3. S式Parser
4. Error recovery
5. Syntax NodeのStable Identity
6. BindingとScope
7. Module skeleton
8. Source Formatterの最小版

### 3.3 重要な設計条件

- ParserはSource RangeとProvenanceを保持する
- Error recovery後のNodeと正常Nodeを区別する
- 行番号をNode Identityにしない
- Formatterは意味を変えない
- GUIからの将来のSource更新を阻害する不可逆な正規化を避ける

### 3.4 最初の外部動作

```text
rpx parse input.rpx
rpx format input.rpx
rpx inspect-syntax input.rpx
```

CLI名は暫定でよいが、Parse結果とDiagnosticは構造化Dataとして取得できなければならない。

### 3.5 完了条件

- 正常SourceをSyntax Treeへ変換できる
- 不正Sourceでも複数Diagnosticを報告できる
- 小さなSource編集後に未変更NodeのIdentityを可能な範囲で維持できる
- Parse、Format、再Parseで意味が保存される

## 4. Phase 2: Type、Effect、Core意味論

### 4.1 目的

RPX Programの静的意味と、最小の評価意味を確立する。

### 4.2 実装順

1. Type表現
2. Type VariableとSubstitution
3. Constraint生成
4. Unification
5. Record Row
6. Algebraic Data Type
7. Function Type
8. Effect Row
9. Bidirectional Type Checking
10. Typed CoreへのLowering
11. 参照Evaluator

### 4.3 参照Evaluator

最初のEvaluatorは最適化を目的としない。仕様上の評価順、Closure、Pattern Match、Failure、Handlerの動作を明確に検査できる実装とする。

### 4.4 完了条件

- 型付け済みCoreを生成できる
- TypeとEffectのDiagnosticにSource Originがある
- 評価順が仕様どおり左から右である
- PureなFunction、Data Type、Pattern Match、Recordが実行できる
- 小さな意味論Testが構造化結果で通る

## 5. Phase 3: Editable Document ModelとTransaction

### 5.1 目的

言語とGUIの間に置く規範的な編集対象を作る。

### 5.2 実装項目

- `DocumentIdentity`
- `DocumentRevision`
- `StableNodeId`
- Ownership TreeとReference Graph
- 基本Node: Document、Group、Rectangle、Text
- PropertyとStyle
- Resource Referenceの最小版
- Document Transaction
- Transaction Preconditions
- Atomic commitとrollback
- Node追加、削除、移動、Property変更、Text変更
- Source Provenance

### 5.3 完了条件

- Document SnapshotへRectangleとTextを構築できる
- Node移動でStable IDが変わらない
- Node複製では新IDになる
- Transaction途中Failureで一部変更を残さない
- Transaction前後のRevisionを追跡できる
- NodeからSource Originへたどれる

## 6. Phase 4: 最小の双方向Vertical Slice

### 6.1 目的

reciplexaの中心仮説を早期に検証する。

### 6.2 対象機能を限定する

```text
Node:
Rectangle
Text
Group

Property:
x, y, width, height
fill color
text content
```

### 6.3 文字からGUI

```text
RPX Source
↓ Parse／Type／Evaluate
Editable Document
↓ GUI Description
Canvas
```

Source変更後、Document差分とGUI Reconciliationを通じてCanvasを更新する。

### 6.4 GUIから文字

```text
CanvasでRectangleを移動
↓
Document Transaction
↓
Source Provenanceと編集Policy
↓
RPX Source更新
↓
再解析
```

Source上の値を直接更新できない場合は、次を区別する。

- 更新可能なLiteral
- 更新可能なBinding
- Shared Definitionへ反映すべき変更
- 高水準の式で一意に逆変換できない変更
- GUI Overrideとして保持すべき変更
- 利用者選択が必要な変更

### 6.5 このPhaseで決め切る事項

- Source更新の最小Edit表現
- Provenanceが複数Sourceへ由来する場合のPolicy
- FunctionやShared Styleを壊さない更新規則
- GUI Overrideの位置付け
- 双方向反映不能時のDiagnosticとUI

### 6.6 完了条件

- SourceでRectangleの位置を変えるとCanvasへ反映される
- CanvasでRectangleを動かすとSourceへ反映される
- Text内容をSourceとGUIの双方から編集できる
- Node Identity、Selection、GUI Stateが不要に失われない
- 循環更新や自己再適用が起こらない
- 逆変換不能な変更を黙って低水準化しない

このPhaseを通過するまで、複雑なGUI機能や多数Backendの実装へ進まない。

## 7. Phase 5: Effect LoweringとStructured Runtime

### 7.1 目的

Effect、Failure、Cancellation、Taskを実行可能なControl Flowへ変換する。

### 7.2 Effect Lowering

実装順:

1. HandlerとOperationのTyped Core表現
2. Continuationの明示化
3. Deep Handler semantics
4. One-shot検査
5. Failure Effect
6. Cancellationとの分離
7. Task操作への接続
8. Lowered IR Verifier

KokaのEffect Handler Compilerを参考にする。ただし、RPXの規範意味を正本とする。

### 7.3 Structured Concurrency

実装順:

1. Root Scope
2. Child Task Scope
3. `spawn`
4. `await`
5. Cancellation Token
6. Fail-fast
7. Collect-all
8. Scope cleanup
9. 決定的Test Scheduler

### 7.4 完了条件

- Handlerの正常、Failure、Resume、Cancellation Pathが動く
- One-shot Continuationを二重Resumeできない
- Scope終了時に子Taskが残らない
- Cancellation時にもResource cleanupが行われる
- Test Schedulerで同じScheduleを再現できる

## 8. Phase 6: Perceus、Ownership、Reuse

### 8.1 目的

参照Evaluatorで確認した意味を保ったまま、実用的なMemory管理を導入する。

### 8.2 段階的導入

#### Step A: 保守的Reference Counting

- 明示的`dup`と`drop`
- Reuseなし
- Debug Resource trace

#### Step B: Perceus Pass

- Control-flowごとの所有状態
- 精密なReference Count挿入
- Drop specialization
- Borrowの導入範囲

#### Step C: Reuse Analysis

- 一意所有の判定
- Constructor reuse
- Copy fallback
- Reuse候補のCost model

#### Step D: Ownership／Reuse Verifier

- すべてのControl-flow Pathを検査
- Failure、Cancellation、Continuation Pathを含める
- Pass後IRを独立にRejectできる

### 8.3 参考実装の使い分け

- KokaとPerceus論文: Reference CountingとReuseの意味、Pass構成
- Rust MIR Borrow Checker: CFG上のData-flow、Move、Liveness、独立検証Passの構成

RustのSource-level Borrow規則をRPX利用者へそのまま導入しない。

### 8.4 完了条件

- 参照Evaluatorと最適化Evaluatorが同じ観測結果を返す
- Leak、二重release、Move後利用をVerifierが検出する
- Reuse有無で外部意味が変わらない
- CancellationとFailure PathでResourceが残らない

## 9. Phase 7: 段階化IRと最初のBackend

### 9.1 目的

Documentの意味を、Layout、Motion、描画、出力へ段階的に変換する。

### 9.2 実装順

1. Domain IR
2. Visual IR
3. Layoutの最小版
4. Render IR
5. Provenance Map
6. Backend Capability
7. Output Profile
8. Backend Planning IR
9. SVG Backend
10. Interactive Preview Backend
11. Artifact Validation

Motion IRは静的Vertical Sliceの後に追加してよい。

### 9.3 最初にSVGを推奨する理由

- Textとして検査しやすい
- Vector構造を維持できる
- BrowserまたはViewerで確認しやすい
- Debug Artifactとして利用しやすい

これは実装上の推奨であり、RPXの意味をSVGへ固定するものではない。

### 9.4 完了条件

- DocumentからSVGを生成できる
- Output ProfileとCapabilityの不一致をPlanning時に検出する
- EmitterがPlan外Fallbackを行わない
- Artifact Validatorが基本構造を検査する
- SourceからArtifactまでProvenanceを追跡できる

## 10. Phase 8: GUI Runtimeの本格化

### 10.1 目的

最小Canvasを、日常的な制作に耐える編集Applicationへ拡張する。

### 10.2 実装順

1. GUI DescriptionとMounted Instance
2. Stable Key
3. Reconciliation Plan
4. Commit Barrier
5. Component Lifecycle
6. View StateとWidget State
7. Focus
8. Node Selection
9. Text CaretとSelection Rebase
10. IME
11. Pointer Capture
12. Gesture Arena
13. Drag-and-drop
14. Inspector
15. Tree／Layer View
16. Timeline
17. Virtualization
18. Accessibility

### 10.3 完了条件

- 動的Listの並べ替えでStateが別Nodeへ移らない
- Reconciliation Failureで旧GUIを維持する
- Focus、IME、Pointer CaptureがOwner削除時に安全に終了する
- Text Transaction後にCaretをRebaseできる
- GUI StateをDocumentの正本にしない

## 11. Phase 9: 永続化、Undo、Migration

### 11.1 実装順

1. Portable Document Snapshot
2. Resource Manifest
3. Canonical Encoding
4. Atomic Save
5. Transaction Log Segment
6. Undo／Redo
7. Crash Recovery
8. Snapshot Compaction
9. Schema Version
10. Migration Graph
11. Unknown Extension
12. Partial Recovery

### 11.2 完了条件

- Save／LoadでStable Node IDが維持される
- Capability、Secret、Task、Native Pointerが保存されない
- 切断Transactionを部分適用しない
- Migration前後で規範状態が同値である
- Recovery文書を元Fileへ自動上書きしない

## 12. Phase 10: Package、Target、Build

### 12.1 実装順

1. Package Identity
2. Manifest Model
3. Module Export／Import
4. Dependency Requirement
5. Resolver
6. Lockfile
7. Target
8. Entry Binding
9. Runtime Profile
10. Build Graph
11. Incremental Cache
12. Package Diagnostic

### 12.2 完了条件

- 同じLockfileとInputから同じPackage Graphを得る
- TargetごとにEntryとProfileを検査する
- NetworkやFilesystem列挙順で解決結果が変わらない
- Contract変更とImplementation変更の無効化範囲を分ける

## 13. Phase 11: Native PackageとForeign Boundary

### 13.1 実装順

1. Foreign Value
2. Validator
3. Trusted Adapter Contract
4. Static Descriptor
5. Candidate Registry
6. ABI Negotiation
7. Transactional Initialization
8. Instance Lifecycle
9. Callback Generation
10. Shutdown
11. Worker Isolation
12. Quarantine
13. Fallback Plan
14. Conformance Suite
15. Differential Test

### 13.2 最初のNative対象

最初のNative統合は、外部作用が少なく、Portable版と比較しやすい機能を選ぶ。

候補:

```text
Image decode
Compression
Text shaping
```

File書込み、Network送信、GPU Device Process-global State等から始めない。

### 13.3 完了条件

- 未選択Binaryを読み込まない
- Ready前のInstanceを公開しない
- ABI不一致を開始前に拒否する
- Contract ViolationでQuarantineする
- Pure Replayable Operationだけ安全にFallbackできる
- PortableとNativeをContract-defined equivalenceで比較できる

## 14. Phase 12: 出力形式と動的表現の拡張

### 14.1 出力Backend

推奨順:

1. SVG
2. Raster Image
3. PDF
4. PPTX
5. Video
6. Platform-specific Interactive Output

各BackendはCapability、Planning、Loss、Verificationを個別に実装する。

### 14.2 Motion

- Time Domain
- Timeline
- Animation Track
- Easing
- Event
- Motion IR
- Frame sampling
- Video Backend
- Interactive Preview同期

### 14.3 完了条件

- 静的と動的表現が同じDocument Identity体系を使う
- PreviewとFinal OutputでProfile差を明示する
- Backend差を暗黙の見た目変更として隠さない

## 15. Phase 13: 最適化、Hardening、証明

### 15.1 最適化

- Incremental Type Checking
- Fine-grained invalidation
- Parallel Compile
- IR memoization
- Layout cache
- Render cache
- Native instance cache
- GUI virtualization

### 15.2 Hardening

**Hardening（堅牢化）**とは、異常入力、Resource枯渇、Crash、Security境界に対する耐性を高める作業を指す。

- Decode Budget
- Sandbox
- Fuzzing
- Adversarial Test
- Crash Recovery Test
- Resource leak test
- Privacy audit
- Capability audit
- Reproducible build audit

### 15.3 形式的検証

後続で優先する証明:

1. Core型安全性
2. Effect Soundness
3. Handler Loweringの意味保存
4. Perceus Passの音全性
5. Ownership／Reuse Verifierの音全性
6. IR Loweringの意味保存
7. Transaction Atomicity
8. Incremental結果とFull Rebuildの同値性

## 16. 横断的なTest戦略

各Phaseで次のTest層を維持する。

```text
Unit Test
↓
Subsystem Property Test
↓
Cross-subsystem Integration Test
↓
Specification Conformance Test
↓
End-to-end Vertical Slice Test
```

重要なEnd-to-end Case:

```text
Source変更
→ Type Check
→ Document差分
→ GUI更新
→ GUI操作
→ Source反映
→ Save
→ Load
→ Backend Planning
→ Artifact生成
→ Artifact Verification
```

## 17. Milestone

### M1: RPX Core

Parser、Binding、Type、Effect、参照Evaluatorが動く。

### M2: Editable Document

Stable Node IDとTransactionを持つDocumentをRPXから生成できる。

### M3: Bidirectional Seed

RectangleとTextをSourceとGUIの双方から編集できる。

### M4: Structured Runtime

Effect Lowering、Task Scope、Cancellationが動く。

### M5: Managed Memory

PerceusとOwnership／Reuse Verifierが動く。

### M6: First Artifact

Planning済みRender IRから検証済みSVGを生成できる。

### M7: Editor Alpha

Canvas、Inspector、Selection、Text Editing、Undo、Save／Loadが動く。

### M8: Package and Native

Package Resolverと最初のNative ProviderがConformance Testを通る。

### M9: Multi-backend

PDF、Raster、PPTX等の複数BackendがOutput ProfileとLoss Reportを扱う。

### M10: Dynamic Graphics

Motion IR、Timeline、Preview、Video出力が動く。

## 18. 実装上の優先順位

優先順位は次の通りである。

```text
1. 意味の正しさ
2. 双方向編集の成立
3. IdentityとTransactionの安定性
4. DiagnosticとTest可能性
5. Portableな動作
6. Performance
7. Native最適化
8. 形式的証明
```

性能上の理由で、双方向編集、Stable Identity、Failure分類、Transaction Atomicityを壊してはならない。

## 19. 最初に着手する具体的な作業

```text
1. Workspaceと共通Identity型
2. Source ResourceとRange
3. S式Lexer／Parser
4. Syntax Diagnostic
5. BindingとScope
6. 最小Type Checker
7. 参照Evaluator
8. Stable Node ID付きDocument Model
9. Document Transaction
10. Rectangle／Textの最小Canvas
11. Source → Canvas
12. Canvas操作 → Source
```

この12項目で最初のVertical Sliceを完成させる。以後のSubsystemは、この経路を壊さない形で拡張する。

## 20. Phase 14: 標準Package群による実用化

### 20.1 目的

言語処理系、Document Model、GUI、Backendが動いても、利用者が最初から使える標準Packageがなければ、何を作れるSystemなのかを確認できない。標準Packageは単なる見本ではなく、RPXの外部仕様、Package API、双方向編集、Layout、Backend Capabilityが実用上十分かを検証するReference Clientである。

標準Packageは言語完成後に一括実装するのではなく、各Vertical Sliceの利用者側APIとして段階的に育てる。ただし、互換性を約束する`stable`公開面は、Language CoreとDocument Modelが安定してから確定する。

### 20.2 Package層

```text
rpx.std.core
    長さ、角度、色、座標、範囲、基本Collection

rpx.std.visual
    Canvas、Group、Transform、Style、基本Shape

rpx.std.text
    Text、TextBox、Paragraph、Span、Font、行分割

rpx.std.document
    Page、Flow、Section、Heading、List、Table、Figure、Reference

rpx.std.math
    数式Tree、演算子、分数、根号、上下付き、Delimiter、整列

rpx.std.slide
    Slide、Master、Theme、Placeholder、Notes、Transition

rpx.std.vector
    Path、Boolean Operation、Stroke、Gradient、Marker、Constraint

rpx.std.motion
    Timeline、Track、Keyframe、Easing、Event
```

Package名は実装時にManifest naming規則へ合わせて確定する。責任分離は維持する。

### 20.3 標準最小Package

最初に、次を一つの利用可能な標準Package集合として完成させる。

```text
Primitive:
Point, Size, Rect, Transform, Color, Length

Visual:
Canvas, Group, Rectangle, Ellipse, Line, Path, Image

Text:
Text, TextBox, Span, Paragraph

Style:
Fill, Stroke, Opacity, Font, TextStyle

Layout:
Stack, Row, Column, Padding, Align
```

必須条件:

- Sourceから生成できる
- GUIで選択・移動・Resize・Style変更できる
- GUI変更をSourceまたは規範的Overrideへ戻せる
- SVG／Preview Backendで表示できる
- Snapshotへ保存できる
- Stable Node IDを維持する
- Propertyごとに型と単位がある

### 20.4 日本語文書Package

標準最小Packageの後に、`rpx.std.document`と日本語組版Profileを実装する。

W3CのJLReqは、CSS、SVG、XSL-FO等で求められる一般的な日本語組版要件を記述し、JIS X 4051を主な基礎としつつ、見出し、図版・表、注、合印等も扱っている。日本語文書Packageの要件抽出と適合Caseの主要参考資料とする。citeturn262search161turn262search162

実装順:

```text
1. Inline／Blockの基本Box
2. Paragraphと行分割
3. 禁則処理
4. 約物と字間
5. 文字方向と縦書きの基礎
6. Ruby
7. Pageと版面
8. Heading、List、Note
9. FigureとTable配置
10. Footnote、Reference、Index
```

JLReqをそのまま一つの巨大Algorithmへしない。要件を小さなRule、Constraint、Penalty、Profileへ分解し、適合試験と対応付ける。

### 20.5 数式Package

日本語文書の基本Flowが動いた後、`rpx.std.math`を実装する。SATySFiは静的型付き関数型言語を備え、Text LayerとProgram Layerから柔軟なCommandを定義する組版Systemであり、Package設計と型付き組版APIの参考になる。citeturn262search167turn262search169

SATySFiの数式機能には、Inline／Display数式、分数、根号、上下付き、Delimiter、数式整列等があるため、RPX数式Packageの機能分解と適合Caseの参考にする。citeturn262search168turn262search170

実装順:

```text
1. Math AtomとClass
2. SymbolとOperator
3. Row
4. Fraction
5. Superscript／Subscript
6. Radical
7. Delimiter
8. Accent
9. Matrix／Cases
10. Display Math
11. Alignment
12. Equation NumberとReference
13. Font／Glyph fallback
```

数式を一般Textの装飾として表現しない。編集可能なMath Treeと、Layout後のVisual IRを分離する。

### 20.6 Slide Package

文書Packageと基本Visual Packageの上に`rpx.std.slide`を実装する。

```text
1. SlideとSlide Size
2. Theme
3. Master
4. Placeholder
5. Title／Body／Figure Layout
6. Speaker Notes
7. Page Number
8. Transition
9. Presenter Preview
10. PPTX／PDF／Video Profile
```

SlideはPageの別名ではない。Master、Theme、Placeholder、Transition、Notes等の意味をDocument Modelで保持する。

### 20.7 Vector Graphics Package

`rpx.std.vector`を本格化する。

```text
1. Path Segment
2. Fill Rule
3. Stroke Join／Cap／Dash
4. Gradient
5. Clip／Mask
6. Boolean Path Operation
7. Marker
8. Symbol／Component
9. Constraint／Guide／Snap
10. Blend／Filter
11. Export Profile
```

Glispは、GUIの直接操作とProgramming Languageの抽象性を組み合わせ、Lisp dialectをProject Fileとして使用するDesign Toolである。標準Vector Packageの直接操作、Metadata、Handle、Source round-tripの比較対象とする。

### 20.8 Package Showcase

各標準Packageは、APIだけでなく実際に開けるShowcaseを持つ。

```text
basic-shapes.rpx
text-boxes.rpx
japanese-article.rpx
math-formulas.rpx
slide-deck.rpx
vector-poster.rpx
motion-title.rpx
```

Showcaseは次を同時に検査する。

- Sourceとして読みやすい
- GUIで編集できる
- 双方向反映できる
- Save／Loadできる
- 複数Backendへ出力できる
- Diagnosticが理解可能である

### 20.9 実用化Milestone

```text
M11: Standard Visual
基本Shape、TextBox、Style、LayoutでPosterを作れる

M12: Japanese Document
JLReq由来の基本日本語組版で記事と冊子を作れる

M13: Mathematics
Inline／Display数式と整列数式を文書・Slideで使える

M14: Slides
ThemeとMasterを持つSlide Deckを編集・出力できる

M15: Vector Design
Path、Gradient、Constraintを使うVector artworkを制作できる。これは、glispでできることを参考にする
```

## 21. 最終的な実装順序

```text
Core Infrastructure
↓
Language Frontend／Type／Effect
↓
Editable Document／Transaction
↓
Minimum Standard Visual Package
↓
Bidirectional Canvas Vertical Slice
↓
Structured Runtime／Effect Lowering
↓
Perceus／Ownership Verifier
↓
Layered IR／SVG／Preview
↓
Full GUI Runtime
↓
Persistence／Package／Native
↓
Japanese Document Package
↓
Math Package
↓
Slide Package
↓
Vector Graphics Package
↓
Motion／Additional Backends
↓
Optimization／Hardening／Proofs
```
